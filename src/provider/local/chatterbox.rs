use crate::provider::Ttser;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

/// Chatterbox TTS client — MIT licensed, emotion exaggeration control, 23 languages.
/// Voice cloning from 5s reference, single exaggeration parameter (0.0=monotone, 1.0=dramatic).
/// Uses PyTorch MPS on Apple Silicon.
pub struct ChatterboxClient {
    pub python_path: String,
    pub exaggeration: f64,
    reference_audio: RwLock<Option<PathBuf>>,
}

impl ChatterboxClient {
    pub fn new(python_path: &str, exaggeration: f64) -> Self {
        Self {
            python_path: python_path.to_string(),
            exaggeration,
            reference_audio: RwLock::new(None),
        }
    }
}

#[async_trait]
impl Ttser for ChatterboxClient {
    async fn text_to_speech(
        &self,
        text: &str,
        _voice: &str,
        output_file: &Path,
    ) -> anyhow::Result<()> {
        let output_str = output_file
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 path: {}", output_file.display()))?;

        let temp_dir = output_file.parent().unwrap_or(Path::new("."));
        let stem = output_file.file_stem().unwrap_or_default().to_string_lossy();
        let temp_file = temp_dir.join(format!("chatterbox_input_{stem}.txt"));
        tokio::fs::write(&temp_file, text).await?;

        let temp_str = temp_file
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 path: {}", temp_file.display()))?;

        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/chatterbox_tts.py");
        let exag_str = format!("{}", self.exaggeration);

        let mut args = vec![
            script,
            "--text-file", temp_str,
            "--output", output_str,
            "--exaggeration", &exag_str,
        ];

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

    fn supports_emotion_tags(&self) -> bool { false }

    async fn prepare_voice(
        &self,
        reference_audio: &Path,
        _reference_transcript: Option<&str>,
        _work_dir: &Path,
    ) -> anyhow::Result<()> {
        let mut guard = self.reference_audio.write().await;
        *guard = Some(reference_audio.to_path_buf());
        tracing::info!("   🎤 Chatterbox: voice cloning prepared from {}", reference_audio.display());
        Ok(())
    }
}
