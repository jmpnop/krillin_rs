use crate::provider::Transcriber;
use crate::types::subtitle::{TranscriptionData, Word};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;
use std::process::Stdio;

pub struct FasterWhisperProcessor {
    pub python_path: String,
    pub model: String,
    pub gpu: bool,
}

#[derive(Debug, Deserialize)]
struct FwOutput {
    segments: Vec<FwSegment>,
    language: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FwSegment {
    text: Option<String>,
    words: Option<Vec<FwWord>>,
}

#[derive(Debug, Deserialize)]
struct FwWord {
    start: f64,
    end: f64,
    word: String,
}

impl FasterWhisperProcessor {
    pub fn new(python_path: &str, model: &str, gpu: bool) -> Self {
        Self {
            python_path: python_path.to_string(),
            model: model.to_string(),
            gpu,
        }
    }
}

#[async_trait]
impl Transcriber for FasterWhisperProcessor {
    async fn transcription(
        &self,
        audio_file: &Path,
        language: &str,
        work_dir: &Path,
    ) -> anyhow::Result<TranscriptionData> {
        let audio_str = audio_file
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 path: {}", audio_file.display()))?;
        let work_str = work_dir
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 path: {}", work_dir.display()))?;

        let mut args = vec![
            "scripts/fasterwhisper_cli.py",
            audio_str,
            "--model", &self.model,
            "--model_dir", "./models/",
            "--output_dir", work_str,
        ];

        let compute_type;
        if self.gpu {
            compute_type = "float16".to_string();
            args.extend(["--compute_type", &compute_type]);
        }

        let lang_owned;
        if !language.is_empty() {
            lang_owned = language.to_string();
            args.extend(["--language", &lang_owned]);
        }

        let output = tokio::process::Command::new(&self.python_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("FasterWhisper failed: {stderr}");
        }

        // Parse output JSON
        let stem = audio_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("audio");
        let json_file = format!("{work_str}/{stem}.json");
        let json_content = tokio::fs::read_to_string(&json_file).await?;
        let fw: FwOutput = serde_json::from_str(&json_content)?;

        let mut words = Vec::new();
        let mut full_text = String::new();

        for segment in &fw.segments {
            if let Some(text) = &segment.text {
                full_text.push_str(text);
            }
            if let Some(w_list) = &segment.words {
                for w in w_list {
                    let text = w.word.replace("--", " ").trim().to_string();
                    if !text.is_empty() {
                        words.push(Word {
                            num: words.len(),
                            text,
                            start: w.start,
                            end: w.end,
                        });
                    }
                }
            }
        }

        Ok(TranscriptionData {
            language: fw.language.unwrap_or_default(),
            text: fw.text.unwrap_or(full_text),
            words,
        })
    }
}
