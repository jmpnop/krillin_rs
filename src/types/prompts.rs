pub const SYSTEM_PROMPT: &str = "You are an assistant that helps with subtitle translation.";

pub const SPLIT_TEXT_WITH_CONTEXT_PROMPT: &str = r#"Below is a text that needs to be translated from {origin_lang} to {target_lang}.

Context (previous sentences for reference, do NOT translate these):
{prev_context}

Text to translate:
{text}

Context (following sentences for reference, do NOT translate these):
{next_context}

Requirements:
1. Translate only the "Text to translate" section
2. Use the context to ensure consistency and accuracy
3. Return ONLY the translated text, nothing else
4. Keep the translation natural and fluent
5. Do not add any explanations or notes"#;

pub const SPLIT_LONG_TEXT_BY_MEANING_PROMPT: &str = r#"Please split the following text into shorter sentences. Each sentence should be a complete thought and no longer than {max_length} characters.

Text: {text}

Requirements:
1. Split at natural sentence boundaries
2. Each resulting sentence must be a complete thought
3. Return one sentence per line
4. Do not add numbering or any other formatting
5. Preserve all original text without modification"#;

pub const TRANSLATE_VIDEO_TITLE_AND_DESCRIPTION_PROMPT: &str = "Translate the following video title and description from {origin_lang} to {target_lang}.\n\nTitle: {title}\n\nDescription: {description}\n\nReturn the translated title and description separated by \"####\". Format:\ntranslated_title####translated_description";
