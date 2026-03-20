use super::OpenAiClient;
use crate::provider::Transcriber;
use crate::types::subtitle::{TranscriptionData, Word};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    language: Option<String>,
    text: Option<String>,
    words: Option<Vec<WhisperWord>>,
}

#[derive(Debug, Deserialize)]
struct WhisperWord {
    word: String,
    start: f64,
    end: f64,
}

#[async_trait]
impl Transcriber for OpenAiClient {
    async fn transcription(
        &self,
        audio_file: &Path,
        language: &str,
        _work_dir: &Path,
    ) -> anyhow::Result<TranscriptionData> {
        let file_bytes = tokio::fs::read(audio_file).await?;
        let file_name = audio_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.mp3")
            .to_string();

        let file_part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(file_name)
            .mime_str("audio/mpeg")?;

        let mut form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("model", self.model.clone())
            .text("response_format", "verbose_json")
            .text("timestamp_granularities[]", "word");

        if !language.is_empty() {
            form = form.text("language", language.to_string());
        }

        let url = format!("{}/audio/transcriptions", self.base_url);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Whisper API error {status}: {body}");
        }

        let whisper: WhisperResponse = resp.json().await?;

        let mut words = Vec::new();
        if let Some(w_list) = whisper.words {
            for (i, w) in w_list.into_iter().enumerate() {
                let text = w.word.replace("--", " ").trim().to_string();
                if text.is_empty() {
                    continue;
                }
                words.push(Word {
                    num: i,
                    text,
                    start: w.start,
                    end: w.end,
                });
            }
        }

        // Re-number after filtering
        for (i, w) in words.iter_mut().enumerate() {
            w.num = i;
        }

        Ok(TranscriptionData {
            language: whisper.language.unwrap_or_default(),
            text: whisper.text.unwrap_or_default(),
            words,
        })
    }
}
