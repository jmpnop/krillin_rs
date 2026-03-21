use crate::provider::Ttser;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

/// Fish Speech S2 Pro TTS client — voice cloning + emotion tags on Apple Silicon via MLX.
/// Supports: 15K+ emotion tags, zero-shot voice cloning, Russian (tier 2), 44.1 kHz output.
#[cfg(target_os = "macos")]
pub struct FishSpeechClient {
    pub model: String,
    pub python_path: String,
    reference_audio: RwLock<Option<PathBuf>>,
}

#[cfg(target_os = "macos")]
impl FishSpeechClient {
    pub fn new(model: &str, python_path: &str) -> Self {
        Self {
            model: model.to_string(),
            python_path: python_path.to_string(),
            reference_audio: RwLock::new(None),
        }
    }
}

#[cfg(target_os = "macos")]
#[async_trait]
impl Ttser for FishSpeechClient {
    async fn text_to_speech(
        &self,
        text: &str,
        _voice: &str,
        output_file: &Path,
    ) -> anyhow::Result<()> {
        let output_str = output_file
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 path: {}", output_file.display()))?;

        // Write text to unique temp file to avoid concurrency issues
        let temp_dir = output_file.parent().unwrap_or(Path::new("."));
        let stem = output_file.file_stem().unwrap_or_default().to_string_lossy();
        let temp_file = temp_dir.join(format!("fish_tts_input_{stem}.txt"));
        tokio::fs::write(&temp_file, text).await?;

        let temp_str = temp_file
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 path: {}", temp_file.display()))?;

        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/fish_speech_tts.py");

        let mut args = vec![
            script,
            "--model", &self.model,
            "--text-file", temp_str,
            "--output", output_str,
        ];

        // Add reference audio for voice cloning if prepared
        let ref_audio = self.reference_audio.read().await;
        let ref_str;
        if let Some(ref_path) = ref_audio.as_ref() {
            ref_str = ref_path.to_string_lossy().to_string();
            args.push("--ref-audio");
            args.push(&ref_str);
        }

        let result = crate::util::cmd::run_cmd_status(&self.python_path, &args).await;

        let _ = tokio::fs::remove_file(&temp_file).await;
        result
    }

    fn supports_voice_cloning(&self) -> bool { true }

    fn supports_emotion_tags(&self) -> bool { true }

    async fn prepare_voice(
        &self,
        reference_audio: &Path,
        _reference_transcript: Option<&str>,
        _work_dir: &Path,
    ) -> anyhow::Result<()> {
        let mut guard = self.reference_audio.write().await;
        *guard = Some(reference_audio.to_path_buf());
        tracing::info!("   🎤 Fish Speech: voice cloning prepared from {}", reference_audio.display());
        Ok(())
    }
}
