//! Dependency management — auto-detect and install required tools
//!
//! Uses `uv` for Python package management (installed automatically if missing).
//! Homebrew tools: ffmpeg, yt-dlp, whisper-cpp, whisperkit-cli
//! Python (uv) tools: faster-whisper, mlx-whisper, edge-tts, mlx-audio

use crate::config::{Config, TranscribeProvider, TtsProvider};
use std::path::{Path, PathBuf};
use std::process::Stdio;

/// Where the managed Python venv lives
const VENV_DIR: &str = "./venv";

/// A dependency that may need to be installed
struct Dep {
    name: &'static str,
    kind: DepKind,
    check: Check,
}

enum DepKind {
    Brew(&'static str),      // brew formula name
    Uv(&'static str),        // uv pip package name
}

enum Check {
    Binary(&'static str),    // check `which <binary>`
    UvPkg(&'static str),     // check `uv pip show <pkg>` in venv
}

/// Ensure all dependencies for the current config are installed.
/// Returns the venv bin directory path (if a venv was created).
pub async fn ensure_dependencies(config: &Config) -> anyhow::Result<Option<PathBuf>> {
    let mut deps: Vec<Dep> = Vec::new();

    // Always required
    deps.push(Dep {
        name: "ffmpeg",
        kind: DepKind::Brew("ffmpeg"),
        check: Check::Binary("ffmpeg"),
    });
    deps.push(Dep {
        name: "yt-dlp",
        kind: DepKind::Brew("yt-dlp"),
        check: Check::Binary("yt-dlp"),
    });

    // ASR provider
    match config.transcribe.provider {
        TranscribeProvider::Fasterwhisper => {
            deps.push(Dep {
                name: "faster-whisper",
                kind: DepKind::Uv("faster-whisper"),
                check: Check::UvPkg("faster-whisper"),
            });
        }
        TranscribeProvider::Whispercpp => {
            deps.push(Dep {
                name: "whisper-cpp",
                kind: DepKind::Brew("whisper-cpp"),
                check: Check::Binary("whisper-cpp"),
            });
        }
        TranscribeProvider::Whisperkit => {
            deps.push(Dep {
                name: "whisperkit-cli",
                kind: DepKind::Brew("whisperkit-cli"),
                check: Check::Binary("whisperkit-cli"),
            });
        }
        TranscribeProvider::MlxWhisper => {
            deps.push(Dep {
                name: "mlx-whisper",
                kind: DepKind::Uv("mlx-whisper"),
                check: Check::UvPkg("mlx-whisper"),
            });
        }
    }

    // TTS provider
    match config.tts.provider {
        TtsProvider::EdgeTts => {
            deps.push(Dep {
                name: "edge-tts",
                kind: DepKind::Uv("edge-tts"),
                check: Check::UvPkg("edge-tts"),
            });
        }
        TtsProvider::MlxAudio | TtsProvider::FishSpeech | TtsProvider::Qwen3Tts => {
            deps.push(Dep {
                name: "mlx-audio",
                kind: DepKind::Uv("mlx-audio"),
                check: Check::UvPkg("mlx-audio"),
            });
        }
        TtsProvider::Chatterbox => {
            deps.push(Dep {
                name: "chatterbox-tts",
                kind: DepKind::Uv("chatterbox-tts"),
                check: Check::UvPkg("chatterbox-tts"),
            });
        }
    }

    // Check which deps are missing
    let needs_venv = deps.iter().any(|d| matches!(d.kind, DepKind::Uv(_)));
    let venv_bin = PathBuf::from(VENV_DIR).join("bin");

    let mut missing_brew: Vec<&Dep> = Vec::new();
    let mut missing_uv: Vec<&Dep> = Vec::new();

    for dep in &deps {
        let installed = match &dep.check {
            Check::Binary(bin) => is_binary_available(bin).await,
            Check::UvPkg(pkg) => {
                if venv_bin.exists() {
                    is_uv_pkg_installed(pkg, VENV_DIR).await
                } else {
                    false
                }
            }
        };

        if installed {
            tracing::info!("   ✅ {} — installed", dep.name);
        } else {
            match &dep.kind {
                DepKind::Brew(_) => missing_brew.push(dep),
                DepKind::Uv(_) => missing_uv.push(dep),
            }
        }
    }

    if missing_brew.is_empty() && missing_uv.is_empty() {
        tracing::info!("   📦 All dependencies satisfied");
        if needs_venv && venv_bin.exists() {
            return Ok(Some(venv_bin));
        }
        return Ok(None);
    }

    // Install missing brew packages
    if !missing_brew.is_empty() {
        ensure_homebrew().await?;
        for dep in &missing_brew {
            if let DepKind::Brew(formula) = &dep.kind {
                tracing::info!("   📥 Installing {} via Homebrew...", dep.name);
                let output = tokio::process::Command::new("brew")
                    .args(["install", formula])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .await?;
                if output.status.success() {
                    tracing::info!("   ✅ {} installed", dep.name);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::error!("   ❌ Failed to install {}: {}", dep.name, stderr.lines().last().unwrap_or(&stderr));
                }
            }
        }
    }

    // Install missing Python packages via uv
    if !missing_uv.is_empty() {
        // Ensure uv is available
        ensure_uv().await?;

        // Ensure venv exists
        if !Path::new(VENV_DIR).exists() {
            tracing::info!("   🐍 Creating Python virtual environment at {VENV_DIR}/...");
            let output = tokio::process::Command::new("uv")
                .args(["venv", VENV_DIR])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to create venv: {stderr}");
            }
            tracing::info!("   ✅ Virtual environment created");
        }

        // Collect all packages to install in one go
        let packages: Vec<&str> = missing_uv
            .iter()
            .filter_map(|d| match &d.kind {
                DepKind::Uv(pkg) => Some(*pkg),
                _ => None,
            })
            .collect();

        let names: Vec<&str> = missing_uv.iter().map(|d| d.name).collect();
        tracing::info!("   📥 Installing via uv: {}...", names.join(", "));

        let mut args = vec!["pip", "install", "--python", VENV_DIR];
        args.extend(&packages);

        let output = tokio::process::Command::new("uv")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if output.status.success() {
            for name in &names {
                tracing::info!("   ✅ {} installed", name);
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("   ❌ uv pip install failed: {}", stderr.lines().last().unwrap_or(&stderr));
            // Try installing one by one to see which ones fail
            for dep in &missing_uv {
                if let DepKind::Uv(pkg) = &dep.kind {
                    let output = tokio::process::Command::new("uv")
                        .args(["pip", "install", "--python", VENV_DIR, pkg])
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                        .await?;
                    if output.status.success() {
                        tracing::info!("   ✅ {} installed", dep.name);
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        tracing::error!("   ❌ Failed to install {}: {}", dep.name, stderr.lines().last().unwrap_or(&stderr));
                    }
                }
            }
        }
    }

    if needs_venv && venv_bin.exists() {
        Ok(Some(venv_bin))
    } else {
        Ok(None)
    }
}

async fn ensure_uv() -> anyhow::Result<()> {
    if is_binary_available("uv").await {
        return Ok(());
    }
    tracing::info!("   📥 Installing uv...");
    let output = tokio::process::Command::new("curl")
        .args(["-LsSf", "https://astral.sh/uv/install.sh"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;
    if !output.status.success() {
        anyhow::bail!("Failed to download uv installer");
    }
    // pipe the curl output into sh
    let install = {
        use tokio::io::AsyncWriteExt;
        let mut child = tokio::process::Command::new("sh")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&output.stdout).await?;
            drop(stdin);
        }
        child.wait_with_output().await?
    };
    if !install.status.success() {
        let stderr = String::from_utf8_lossy(&install.stderr);
        anyhow::bail!("Failed to install uv: {stderr}");
    }
    // Verify it's now available
    if !is_binary_available("uv").await {
        // uv installs to ~/.local/bin or ~/.cargo/bin — check common locations
        let home = std::env::var("HOME").unwrap_or_default();
        for dir in &[
            format!("{home}/.local/bin"),
            format!("{home}/.cargo/bin"),
        ] {
            let uv_path = PathBuf::from(dir).join("uv");
            if uv_path.exists() {
                // Add to PATH for this process
                let path = std::env::var("PATH").unwrap_or_default();
                std::env::set_var("PATH", format!("{dir}:{path}"));
                tracing::info!("   ✅ uv installed at {dir}/uv");
                return Ok(());
            }
        }
        anyhow::bail!("uv was installed but not found on PATH");
    }
    tracing::info!("   ✅ uv installed");
    Ok(())
}

async fn ensure_homebrew() -> anyhow::Result<()> {
    if is_binary_available("brew").await {
        return Ok(());
    }
    tracing::error!("   ❌ Homebrew not found. Install it from https://brew.sh");
    tracing::error!("      /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"");
    anyhow::bail!("Homebrew is required but not installed");
}

async fn is_binary_available(name: &str) -> bool {
    tokio::process::Command::new("which")
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

async fn is_uv_pkg_installed(package: &str, venv: &str) -> bool {
    tokio::process::Command::new("uv")
        .args(["pip", "show", "--python", venv, package])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}
