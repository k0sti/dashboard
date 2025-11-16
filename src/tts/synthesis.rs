/// Text preprocessing and synthesis coordination

use anyhow::Result;

/// Preprocess text for TTS synthesis
pub fn preprocess_text(text: &str) -> String {
    let mut processed = text.to_string();

    // Normalize whitespace
    processed = processed
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    // Expand common abbreviations
    processed = expand_abbreviations(&processed);

    // Normalize punctuation for better prosody
    processed = normalize_punctuation(&processed);

    // Collapse multiple spaces into one
    while processed.contains("  ") {
        processed = processed.replace("  ", " ");
    }

    processed
}

/// Expand common abbreviations for better speech
fn expand_abbreviations(text: &str) -> String {
    let expansions = [
        ("Dr.", "Doctor"),
        ("Mr.", "Mister"),
        ("Mrs.", "Missus"),
        ("Ms.", "Miss"),
        ("Prof.", "Professor"),
        ("Sr.", "Senior"),
        ("Jr.", "Junior"),
        ("vs.", "versus"),
        ("etc.", "et cetera"),
        ("e.g.", "for example"),
        ("i.e.", "that is"),
    ];

    let mut result = text.to_string();
    for (abbrev, expansion) in &expansions {
        result = result.replace(abbrev, expansion);
    }
    result
}

/// Normalize punctuation for natural prosody
fn normalize_punctuation(text: &str) -> String {
    let mut result = text.to_string();

    // Ensure single space after sentence-ending punctuation
    result = result.replace(".  ", ". ");
    result = result.replace("!  ", "! ");
    result = result.replace("?  ", "? ");

    // Remove excessive exclamation/question marks
    while result.contains("!!") {
        result = result.replace("!!", "!");
    }
    while result.contains("??") {
        result = result.replace("??", "?");
    }

    result
}

/// Split text into sentences for chunking
#[allow(dead_code)]
pub fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            if let Some(next) = text.chars().nth(current.len()) {
                if next.is_whitespace() || next.is_uppercase() {
                    sentences.push(current.trim().to_string());
                    current.clear();
                }
            } else {
                // End of text
                sentences.push(current.trim().to_string());
                current.clear();
            }
        }
    }

    // Add remaining text
    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }

    sentences
}

/// Synthesize long text by chunking
#[allow(dead_code)]
pub async fn synthesize_long_text(
    text: &str,
    synthesize_fn: impl Fn(&str) -> Result<Vec<f32>>,
) -> Result<Vec<f32>> {
    let preprocessed = preprocess_text(text);

    // If text is short, synthesize directly
    if preprocessed.len() < 500 {
        return synthesize_fn(&preprocessed);
    }

    // Split into sentences and synthesize each
    let sentences = split_sentences(&preprocessed);
    let mut all_samples = Vec::new();

    for sentence in sentences {
        if sentence.is_empty() {
            continue;
        }

        let samples = synthesize_fn(&sentence)?;
        all_samples.extend(samples);

        // Add brief pause between sentences (100ms at 22050 Hz)
        let pause_samples = 2205;
        all_samples.extend(vec![0.0; pause_samples]);
    }

    Ok(all_samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_text() {
        let input = "Hello   world.\nThis is Dr. Smith.";
        let output = preprocess_text(input);
        assert!(output.contains("Doctor Smith"));
        assert!(!output.contains("  "));
    }

    #[test]
    fn test_expand_abbreviations() {
        assert_eq!(expand_abbreviations("Dr. Smith"), "Doctor Smith");
        assert_eq!(expand_abbreviations("Mr. Jones"), "Mister Jones");
    }

    #[test]
    fn test_split_sentences() {
        let text = "Hello world. How are you? I am fine!";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0], "Hello world.");
        assert_eq!(sentences[1], "How are you?");
        assert_eq!(sentences[2], "I am fine!");
    }
}
