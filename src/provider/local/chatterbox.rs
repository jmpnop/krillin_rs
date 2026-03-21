use crate::provider::Ttser;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

/// Chatterbox TTS client — MIT licensed, emotion exaggeration control, 23 languages.
/// Voice cloning from 5s reference, exaggeration parameter (0.0=monotone, 1.0=dramatic).
/// Runs on Apple Silicon via mlx-audio (Metal GPU native).
#[cfg(target_os = "macos")]
pub struct ChatterboxClient {
    pub model: String,
    pub python_path: String,
    pub exaggeration: f64,
    reference_audio: RwLock<Option<PathBuf>>,
}

#[cfg(target_os = "macos")]
impl ChatterboxClient {
    pub fn new(model: &str, python_path: &str, exaggeration: f64) -> Self {
        Self {
            model: model.to_string(),
            python_path: python_path.to_string(),
            exaggeration,
            reference_audio: RwLock::new(None),
        }
    }
}

#[cfg(target_os = "macos")]
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
            script.to_string(),
            "--model".to_string(), self.model.clone(),
            "--text-file".to_string(), temp_str.to_string(),
            "--output".to_string(), output_str.to_string(),
            "--exaggeration".to_string(), exag_str,
        ];

        let ref_audio = self.reference_audio.read().await;
        if let Some(ref_path) = ref_audio.as_ref() {
            args.push("--ref-audio".to_string());
            args.push(ref_path.to_string_lossy().to_string());
        }

        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = crate::util::cmd::run_cmd_status(&self.python_path, &arg_refs).await;

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
