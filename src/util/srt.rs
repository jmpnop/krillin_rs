use crate::types::subtitle::SrtSentenceWithStrTime;
use std::path::Path;

/// Parse an SRT file into a list of subtitle entries
pub fn parse_srt(content: &str) -> Vec<SrtSentenceWithStrTime> {
    let mut results = Vec::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        let line = line.trim();

        // Skip empty lines and index numbers
        if line.is_empty() || line.parse::<u32>().is_ok() {
            continue;
        }

        // Check for timestamp line
        if line.contains("-->") {
            let parts: Vec<&str> = line.split("-->").collect();
            if parts.len() == 2 {
                let start = parts[0].trim().to_string();
                let end = parts[1].trim().to_string();

                // Collect text lines until next empty line
                let mut text_lines = Vec::new();
                while let Some(next) = lines.peek() {
                    let next = next.trim();
                    if next.is_empty() {
                        break;
                    }
                    text_lines.push(next.to_string());
                    lines.next();
                }

                if !text_lines.is_empty() {
                    results.push(SrtSentenceWithStrTime {
                        text: text_lines.join("\n"),
                        start,
                        end,
                    });
                }
            }
        }
    }

    results
}

/// Write SRT entries to a file
pub async fn write_srt(
    entries: &[SrtSentenceWithStrTime],
    output: &Path,
) -> anyhow::Result<()> {
    let mut content = String::new();
    for (i, entry) in entries.iter().enumerate() {
        content.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            i + 1,
            entry.start,
            entry.end,
            entry.text
        ));
    }
    tokio::fs::write(output, content).await?;
    Ok(())
}

/// Merge multiple SRT files into one, renumbering blocks
pub async fn merge_srt_files(output: &Path, inputs: &[&Path]) -> anyhow::Result<()> {
    let mut all_entries = Vec::new();
    for input in inputs {
        let content = tokio::fs::read_to_string(input).await?;
        all_entries.extend(parse_srt(&content));
    }
    write_srt(&all_entries, output).await
}
