use crate::provider::ChatCompleter;
use crate::types::subtitle::SrtSentenceWithStrTime;
use std::sync::Arc;
use tokio::sync::Semaphore;

const VALID_TAGS: &[&str] = &[
    "[neutral]", "[excited]", "[angry]", "[sad]", "[whisper]",
    "[laughing]", "[serious]", "[sarcastic]", "[fearful]",
    "[tender]", "[professional broadcast tone]",
];

/// Detect emotions for each subtitle segment using the LLM.
/// Returns a vector of emotion tags (one per subtitle).
pub async fn detect_emotions(
    chat_completer: &Arc<dyn ChatCompleter>,
    subtitles: &[SrtSentenceWithStrTime],
    max_parallel: usize,
) -> anyhow::Result<Vec<String>> {
    let sem = Arc::new(Semaphore::new(max_parallel));
    let mut handles = Vec::with_capacity(subtitles.len());

    // Extract text from each subtitle's first line (origin language)
    let texts: Vec<String> = subtitles
        .iter()
        .map(|s| s.text.lines().next().unwrap_or("").to_string())
        .collect();

    let start = std::time::Instant::now();
    tracing::info!("   🎭 Detecting emotions for {} segments", texts.len());

    for i in 0..texts.len() {
        let prev = if i > 0 { texts[i - 1].clone() } else { String::new() };
        let text = texts[i].clone();
        let next = if i + 1 < texts.len() { texts[i + 1].clone() } else { String::new() };

        let completer = chat_completer.clone();
        let sem = sem.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await?;

            let prompt = crate::types::prompts::EMOTION_DETECTION_PROMPT
                .replace("{prev}", &prev)
                .replace("{text}", &text)
                .replace("{next}", &next);

            match completer.chat_completion(&prompt).await {
                Ok(response) => {
                    let tag = parse_emotion_tag(&response);
                    Ok::<_, anyhow::Error>((i, tag))
                }
                Err(e) => {
                    tracing::warn!("   ⚠️  Emotion detection failed for segment {i}: {e}");
                    Ok((i, "[neutral]".to_string()))
                }
            }
        }));
    }

    let mut emotions = vec!["[neutral]".to_string(); subtitles.len()];
    for handle in handles {
        let (i, tag) = handle.await??;
        emotions[i] = tag;
    }

    let elapsed = start.elapsed();
    let non_neutral = emotions.iter().filter(|e| *e != "[neutral]").count();
    tracing::info!(
        "   🎭 Emotion detection complete: {non_neutral}/{} non-neutral in {:.1}s",
        emotions.len(),
        elapsed.as_secs_f64()
    );

    Ok(emotions)
}

/// Parse the LLM response into a valid emotion tag
fn parse_emotion_tag(response: &str) -> String {
    let trimmed = response.trim().to_lowercase();
    for tag in VALID_TAGS {
        if trimmed.contains(tag) {
            return tag.to_string();
        }
    }
    // Try without brackets
    for tag in VALID_TAGS {
        let inner = &tag[1..tag.len() - 1]; // strip brackets
        if trimmed.contains(inner) {
            return tag.to_string();
        }
    }
    "[neutral]".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_tags() {
        assert_eq!(parse_emotion_tag("[excited]"), "[excited]");
        assert_eq!(parse_emotion_tag("  [whisper]  "), "[whisper]");
        assert_eq!(parse_emotion_tag("[ANGRY]"), "[angry]");
    }

    #[test]
    fn parse_without_brackets() {
        assert_eq!(parse_emotion_tag("excited"), "[excited]");
        assert_eq!(parse_emotion_tag("The emotion is sad"), "[sad]");
    }

    #[test]
    fn fallback_to_neutral() {
        assert_eq!(parse_emotion_tag("I don't know"), "[neutral]");
        assert_eq!(parse_emotion_tag(""), "[neutral]");
    }
}
