use crate::provider::Ttser;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

/// Qwen3-TTS client — voice cloning via ICL on Apple Silicon (MLX native).
/// 1.7B params, 24 kHz output, Russian tier 1, zero-shot cloning from 3-8s reference.
/// Emotion is implicit from reference prosody + text understanding (no explicit tags).
#[cfg(target_os = "macos")]
pub struct Qwen3TtsClient {
    pub model: String,
    pub python_path: String,
    pub lang: String,
    reference_audio: RwLock<Option<PathBuf>>,
    reference_text: RwLock<Option<String>>,
}

#[cfg(target_os = "macos")]
impl Qwen3TtsClient {
    pub fn new(model: &str, python_path: &str, lang: &str) -> Self {
        Self {
            model: model.to_string(),
            python_path: python_path.to_string(),
            lang: lang.to_string(),
            reference_audio: RwLock::new(None),
            reference_text: RwLock::new(None),
        }
    }
}

#[cfg(target_os = "macos")]
#[async_trait]
impl Ttser for Qwen3TtsClient {
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
        let temp_file = temp_dir.join(format!("qwen3_tts_input_{stem}.txt"));
        tokio::fs::write(&temp_file, text).await?;

        let temp_str = temp_file
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 path: {}", temp_file.display()))?;

        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/qwen3_tts.py");

        let mut args = vec![
            script.to_string(),
            "--model".to_string(), self.model.clone(),
            "--text-file".to_string(), temp_str.to_string(),
            "--output".to_string(), output_str.to_string(),
            "--lang".to_string(), self.lang.clone(),
        ];

        let ref_audio = self.reference_audio.read().await;
        if let Some(ref_path) = ref_audio.as_ref() {
            args.push("--ref-audio".to_string());
            args.push(ref_path.to_string_lossy().to_string());

            let ref_text = self.reference_text.read().await;
            if let Some(rt) = ref_text.as_ref() {
                args.push("--ref-text".to_string());
                args.push(rt.clone());
            }
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
        reference_transcript: Option<&str>,
        _work_dir: &Path,
    ) -> anyhow::Result<()> {
        {
            let mut guard = self.reference_audio.write().await;
            *guard = Some(reference_audio.to_path_buf());
        }
        {
            let mut guard = self.reference_text.write().await;
            *guard = reference_transcript.map(|s| s.to_string());
        }
        tracing::info!("   🎤 Qwen3-TTS: voice cloning prepared from {}", reference_audio.display());
        Ok(())
    }
}
