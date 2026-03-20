use super::OpenAiClient;
use crate::provider::ChatCompleter;
use async_trait::async_trait;
use futures_util::StreamExt;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ChatChunkChoice {
    delta: Option<ChatDelta>,
}

#[derive(Debug, Deserialize)]
struct ChatDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatChunk {
    choices: Vec<ChatChunkChoice>,
}

#[async_trait]
impl ChatCompleter for OpenAiClient {
    async fn chat_completion(&self, query: &str) -> anyhow::Result<String> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": crate::types::prompts::SYSTEM_PROMPT},
                {"role": "user", "content": query}
            ],
            "temperature": 0.9,
            "max_tokens": 8192,
            "stream": true
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
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Chat API error {status}: {body}");
        }

        let mut result = String::new();
        let mut stream = resp.bytes_stream();

        let mut buffer = String::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete SSE lines
            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }

                if let Some(json_str) = line.strip_prefix("data: ") {
                    if let Ok(chunk) = serde_json::from_str::<ChatChunk>(json_str) {
                        for choice in &chunk.choices {
                            if let Some(delta) = &choice.delta {
                                if let Some(content) = &delta.content {
                                    result.push_str(content);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}
