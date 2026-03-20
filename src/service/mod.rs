pub mod link_to_file;
pub mod audio_to_subtitle;
pub mod srt_to_speech;
pub mod srt_embed;
pub mod upload_subtitles;
pub mod split_audio;
pub mod timestamps;

use crate::config::Config;
use crate::provider::openai::OpenAiClient;
use crate::provider::{ChatCompleter, Transcriber, Ttser};
use std::sync::Arc;

pub struct Service {
    pub transcriber: Arc<dyn Transcriber>,
    pub chat_completer: Arc<dyn ChatCompleter>,
    pub tts_client: Arc<dyn Ttser>,
}

impl Service {
    pub fn from_config(config: &Config) -> Self {
        let proxy = if config.app.proxy.is_empty() {
            None
        } else {
            Some(config.app.proxy.as_str())
        };

        // Transcriber
        let transcriber: Arc<dyn Transcriber> = match config.transcribe.provider.as_str() {
            "openai" | _ => Arc::new(OpenAiClient::new(
                &config.transcribe.openai.base_url,
                if config.transcribe.openai.api_key.is_empty() {
                    &config.llm.api_key
                } else {
                    &config.transcribe.openai.api_key
                },
                &config.transcribe.openai.model,
                proxy,
            )),
        };

        // Chat completer (always OpenAI-compatible)
        let chat_completer: Arc<dyn ChatCompleter> = Arc::new(OpenAiClient::new(
            &config.llm.base_url,
            &config.llm.api_key,
            &config.llm.model,
            proxy,
        ));

        // TTS
        let tts_client: Arc<dyn Ttser> = match config.tts.provider.as_str() {
            "openai" | _ => Arc::new(OpenAiClient::new(
                &config.tts.openai.base_url,
                if config.tts.openai.api_key.is_empty() {
                    &config.llm.api_key
                } else {
                    &config.tts.openai.api_key
                },
                &config.tts.openai.model,
                proxy,
            )),
        };

        Self {
            transcriber,
            chat_completer,
            tts_client,
        }
    }
}
