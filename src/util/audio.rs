use std::path::Path;
use std::process::Stdio;

/// Get audio duration in seconds via ffprobe
pub async fn get_duration(ffprobe: &str, input: &Path) -> anyhow::Result<f64> {
    let output = tokio::process::Command::new(ffprobe)
        .args([
            "-i",
            input.to_str().unwrap(),
            "-show_entries",
            "format=duration",
            "-v",
            "quiet",
            "-of",
            "csv=p=0",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await?;

    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    s.parse::<f64>()
        .map_err(|_| anyhow::anyhow!("Failed to parse duration from ffprobe: '{s}'"))
}

/// Convert audio to mono 16kHz for ASR compatibility
pub async fn process_audio(ffmpeg: &str, input: &Path) -> anyhow::Result<String> {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");
    let parent = input.parent().unwrap_or(Path::new("."));
    let output = parent.join(format!("{stem}_mono_16K.mp3"));

    let status = tokio::process::Command::new(ffmpeg)
        .args([
            "-y",
            "-i",
            input.to_str().unwrap(),
            "-ac",
            "1",
            "-ar",
            "16000",
            "-b:a",
            "192k",
            output.to_str().unwrap(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("ffmpeg audio processing failed");
    }

    Ok(output.to_string_lossy().to_string())
}
