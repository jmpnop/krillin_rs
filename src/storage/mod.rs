pub mod task_store;

/// Paths to external CLI tool binaries
#[derive(Debug, Clone, Default)]
pub struct BinPaths {
    pub ffmpeg: String,
    pub ffprobe: String,
    pub ytdlp: String,
    pub fasterwhisper: String,
    pub whisperx: String,
    pub whisperkit: String,
    pub whispercpp: String,
    pub edge_tts: String,
}

impl BinPaths {
    /// Try to locate tools on PATH, fall back to ./bin/
    pub fn detect() -> Self {
        Self {
            ffmpeg: which("ffmpeg"),
            ffprobe: which("ffprobe"),
            ytdlp: which("yt-dlp"),
            fasterwhisper: which("fasterwhisper"),
            whisperx: which("whisperx"),
            whisperkit: which("whisperkit"),
            whispercpp: which("whisper-cpp"),
            edge_tts: which("edge-tts"),
        }
    }
}

fn which(name: &str) -> String {
    // Check ./bin/ first, then PATH
    let local = format!("./bin/{name}");
    if std::path::Path::new(&local).exists() {
        return local;
    }
    // Fall back to bare name (relies on PATH)
    name.to_string()
}
