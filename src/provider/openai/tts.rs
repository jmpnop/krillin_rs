use super::OpenAiClient;
use crate::provider::Ttser;
use async_trait::async_trait;
use futures_util::StreamExt;
use std::path::Path;
use tokio::io::AsyncWriteExt;

#[async_trait]
impl Ttser for OpenAiClient {
    async fn text_to_speech(
        &self,
        text: &str,
        voice: &str,
        output_file: &Path,
    ) -> anyhow::Result<()> {
        let url = format!("{}/audio/speech", self.base_url);

        let body = serde_json::json!({
            "model": self.model,
            "input": text,
            "voice": voice,
            "response_format": "wav"
        });

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_body = resp.text().await.unwrap_or_default();
            anyhow::bail!("TTS API error {status}: {err_body}");
        }

        let mut file = tokio::fs::File::create(output_file).await?;
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            file.write_all(&bytes).await?;
        }

        file.flush().await?;
        Ok(())
    }
}
