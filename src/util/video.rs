use std::path::Path;
use std::process::Stdio;

/// Replace audio track in video
pub async fn replace_audio(
    ffmpeg: &str,
    video: &Path,
    audio: &Path,
    output: &Path,
) -> anyhow::Result<()> {
    let status = tokio::process::Command::new(ffmpeg)
        .args([
            "-y",
            "-i", video.to_str().unwrap(),
            "-i", audio.to_str().unwrap(),
            "-c:v", "copy",
            "-map", "0:v:0",
            "-map", "1:a:0",
            output.to_str().unwrap(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("ffmpeg audio replacement failed");
    }
    Ok(())
}

/// Get video resolution via ffprobe
pub async fn get_resolution(ffprobe: &str, video: &Path) -> anyhow::Result<(u32, u32)> {
    let output = tokio::process::Command::new(ffprobe)
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height",
            "-of", "csv=p=0:s=x",
            video.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await?;

    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() != 2 {
        anyhow::bail!("Failed to parse resolution: '{s}'");
    }
    let w: u32 = parts[0].parse()?;
    let h: u32 = parts[1].parse()?;
    Ok((w, h))
}
