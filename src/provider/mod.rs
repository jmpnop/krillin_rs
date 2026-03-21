pub mod openai;
pub mod local;

use crate::types::subtitle::TranscriptionData;
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait Transcriber: Send + Sync {
    async fn transcription(
        &self,
        audio_file: &Path,
        language: &str,
        work_dir: &Path,
    ) -> anyhow::Result<TranscriptionData>;
}

#[async_trait]
pub trait ChatCompleter: Send + Sync {
    async fn chat_completion(&self, query: &str) -> anyhow::Result<String>;
}

#[async_trait]
pub trait Ttser: Send + Sync {
    async fn text_to_speech(
        &self,
        text: &str,
        voice: &str,
        output_file: &Path,
    ) -> anyhow::Result<()>;

    /// Whether this provider supports voice cloning from reference audio
    fn supports_voice_cloning(&self) -> bool { false }

    /// Whether this provider supports inline emotion tags (e.g. [excited], [whisper])
    fn supports_emotion_tags(&self) -> bool { false }

    /// Prepare voice cloning from reference audio. Called once before TTS begins.
    async fn prepare_voice(
        &self,
        _reference_audio: &Path,
        _reference_transcript: Option<&str>,
        _work_dir: &Path,
    ) -> anyhow::Result<()> { Ok(()) }
}
