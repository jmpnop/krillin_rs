use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Word {
    pub num: usize,
    pub text: String,
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionData {
    pub language: String,
    pub text: String,
    pub words: Vec<Word>,
}

#[derive(Debug, Clone)]
pub struct SrtSentence {
    pub text: String,
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone)]
pub struct SrtSentenceWithStrTime {
    pub text: String,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone)]
pub struct SrtBlock {
    pub index: usize,
    pub timestamp: String,
    pub target_language_sentence: String,
    pub origin_language_sentence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedItem {
    pub origin_text: String,
    pub translated_text: String,
}

#[derive(Debug, Clone)]
pub struct SmallAudio {
    pub audio_file: String,
    pub transcription_data: Option<TranscriptionData>,
    pub srt_no_ts_file: String,
}

/// Format seconds to SRT timestamp: HH:MM:SS,mmm
pub fn format_time(seconds: f64) -> String {
    let total_ms = (seconds * 1000.0) as u64;
    let h = total_ms / 3_600_000;
    let m = (total_ms % 3_600_000) / 60_000;
    let s = (total_ms % 60_000) / 1_000;
    let ms = total_ms % 1_000;
    format!("{h:02}:{m:02}:{s:02},{ms:03}")
}

/// Format a time range for SRT
pub fn format_time_range(start: f64, end: f64) -> String {
    format!("{} --> {}", format_time(start), format_time(end))
}

/// Parse SRT timestamp "HH:MM:SS,mmm" to seconds
pub fn parse_timestamp(ts: &str) -> Option<f64> {
    let parts: Vec<&str> = ts.split(&[':', ','][..]).collect();
    if parts.len() != 4 {
        return None;
    }
    let h: f64 = parts[0].parse().ok()?;
    let m: f64 = parts[1].parse().ok()?;
    let s: f64 = parts[2].parse().ok()?;
    let ms: f64 = parts[3].parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + s + ms / 1000.0)
}
