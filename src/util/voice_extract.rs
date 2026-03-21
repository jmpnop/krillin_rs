use crate::types::subtitle::parse_timestamp;
use crate::util::cmd;
use crate::util::srt::parse_srt;
use std::path::{Path, PathBuf};

/// Extract a reference voice sample from the original audio using subtitle timestamps.
/// Picks the longest contiguous speech segments up to `max_duration` seconds.
/// Returns (path to extracted WAV, transcript text of selected segments).
pub async fn extract_reference_voice(
    ffmpeg: &str,
    audio_file: &Path,
    srt_file: &Path,
    work_dir: &Path,
    max_duration: f64,
) -> anyhow::Result<(PathBuf, String)> {
    let srt_content = tokio::fs::read_to_string(srt_file).await?;
    let subtitles = parse_srt(&srt_content);

    if subtitles.is_empty() {
        anyhow::bail!("No subtitles found for voice extraction");
    }

    // Sort by segment duration (longest first) for cleanest speech samples
    let mut scored: Vec<(usize, f64, f64)> = subtitles
        .iter()
        .enumerate()
        .filter_map(|(i, sub)| {
            let start = parse_timestamp(&sub.start)?;
            let end = parse_timestamp(&sub.end)?;
            let dur = end - start;
            if dur > 0.5 { Some((i, start, dur)) } else { None }
        })
        .collect();

    scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Select segments until we reach max_duration
    let mut selected = Vec::new();
    let mut selected_indices = Vec::new();
    let mut total = 0.0;
    for (i, start, dur) in &scored {
        if total >= max_duration {
            break;
        }
        let end_ts = parse_timestamp(&subtitles[*i].end).unwrap_or(start + dur);
        selected.push((*start, end_ts));
        selected_indices.push(*i);
        total += dur;
    }

    if selected.is_empty() {
        anyhow::bail!("No usable speech segments found for voice extraction");
    }

    // Sort by start time for natural concatenation
    selected.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Extract each segment
    let ref_dir = work_dir.join("voice_ref");
    tokio::fs::create_dir_all(&ref_dir).await?;

    let mut concat_list = String::new();
    let audio_str = audio_file.to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 path: {}", audio_file.display()))?;

    for (i, (start, end)) in selected.iter().enumerate() {
        let seg_path = ref_dir.join(format!("seg_{i}.wav"));
        let seg_str = seg_path.to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 path: {}", seg_path.display()))?;

        let start_str = format!("{start}");
        let end_str = format!("{end}");

        // Extract segment with audio cleaning (borrowed from yt-dbl):
        // highpass=80Hz removes rumble, afftdn reduces background noise
        cmd::run_cmd_status(ffmpeg, &[
            "-y", "-i", audio_str,
            "-ss", &start_str, "-to", &end_str,
            "-af", "highpass=f=80,afftdn=nf=-25",
            "-ac", "1", "-ar", "24000",
            "-acodec", "pcm_s16le",
            seg_str,
        ]).await?;

        concat_list.push_str(&format!("file '{}'\n", seg_str));
    }

    // Concatenate all segments
    let concat_file = ref_dir.join("concat_list.txt");
    let output = work_dir.join("reference_voice.wav");
    let concat_str = concat_file.to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 path: {}", concat_file.display()))?;
    let output_str = output.to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 path: {}", output.display()))?;

    tokio::fs::write(&concat_file, &concat_list).await?;

    cmd::run_cmd_status(ffmpeg, &[
        "-y", "-f", "concat", "-safe", "0",
        "-i", concat_str,
        "-c", "copy",
        output_str,
    ]).await?;

    let dur = crate::util::audio::get_duration("ffprobe", Path::new(output_str))
        .await
        .unwrap_or(0.0);
    // Collect transcript text from selected segments (origin language, first line)
    let ref_transcript: String = selected_indices
        .iter()
        .filter_map(|&i| subtitles.get(i))
        .map(|s| s.text.lines().next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join(" ");

    tracing::info!("   🎤 Reference voice extracted: {:.1}s from {} segments", dur, selected.len());

    Ok((output, ref_transcript))
}
