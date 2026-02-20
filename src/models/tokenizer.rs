//! GPT-2 Tokenizer for Whisper
//!
//! Loads the full GPT-2 vocabulary used by OpenAI Whisper models.

use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tracing::{debug, info, warn};

/// GPT-2 tokenizer for Whisper with full vocabulary support
pub struct SimpleTokenizer {
    /// Token ID to string mapping (reverse vocab)
    vocab: HashMap<i64, String>,
    /// Special token IDs to skip during decoding
    special_tokens: HashSet<i64>,
}

impl SimpleTokenizer {
    /// Create a new tokenizer by loading vocab.json from the model directory
    pub fn new() -> Self {
        // Default path to vocab.json
        let vocab_path = dirs::home_dir()
            .expect("Failed to get home directory")
            .join("Library/Caches/vox/models/whisper-tiny.en/onnx/vocab.json");

        Self::from_file(&vocab_path).unwrap_or_else(|e| {
            warn!("Failed to load vocab.json: {}. Using minimal fallback.", e);
            Self::minimal_fallback()
        })
    }

    /// Create a minimal fallback tokenizer for testing
    fn minimal_fallback() -> Self {
        let mut vocab = HashMap::new();

        // Just add some basic ASCII characters
        for i in 32..127 {
            vocab.insert(i as i64, (i as u8 as char).to_string());
        }

        Self {
            vocab,
            special_tokens: HashSet::from([50256, 50257, 50258]),
        }
    }

    /// Load tokenizer from vocab.json file
    pub fn from_file(path: &Path) -> Result<Self> {
        info!("Loading tokenizer from: {}", path.display());

        // Read and parse vocab.json
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read vocab file: {}", path.display()))?;

        let forward_vocab: HashMap<String, i64> =
            serde_json::from_str(&content).with_context(|| "Failed to parse vocab.json")?;

        // Create reverse mapping (ID -> string)
        let mut vocab = HashMap::new();
        for (token_str, token_id) in forward_vocab {
            vocab.insert(token_id, token_str);
        }

        // Define Whisper special tokens
        // These are task/language tokens that should be filtered from output
        let special_tokens = HashSet::from([
            50256, // <|endoftext|>
            50257, // <|startoftranscript|>
            50258, // <|en|>
            50259, // <|zh|>
            50260, // <|de|>
            50261, // <|es|>
            50262, // <|ru|>
            50263, // <|ko|>
            50264, // <|fr|>
            50265, // <|ja|>
            50266, // <|pt|>
            50267, // <|tr|>
            50268, // <|pl|>
            50269, // <|ca|>
            50270, // <|nl|>
            50271, // <|ar|>
            50272, // <|sv|>
            50273, // <|it|>
            50274, // <|id|>
            50275, // <|hi|>
            50276, // <|fi|>
            50277, // <|vi|>
            50278, // <|he|>
            50279, // <|uk|>
            50280, // <|el|>
            50281, // <|ms|>
            50282, // <|cs|>
            50283, // <|ro|>
            50284, // <|da|>
            50285, // <|hu|>
            50286, // <|ta|>
            50287, // <|no|>
            50288, // <|th|>
            50289, // <|ur|>
            50290, // <|hr|>
            50291, // <|bg|>
            50292, // <|lt|>
            50293, // <|la|>
            50294, // <|mi|>
            50295, // <|ml|>
            50296, // <|cy|>
            50297, // <|sk|>
            50298, // <|te|>
            50299, // <|fa|>
            50300, // <|lv|>
            50301, // <|bn|>
            50302, // <|sr|>
            50303, // <|az|>
            50304, // <|sl|>
            50305, // <|kn|>
            50306, // <|et|>
            50307, // <|mk|>
            50308, // <|br|>
            50309, // <|eu|>
            50310, // <|is|>
            50311, // <|hy|>
            50312, // <|ne|>
            50313, // <|mn|>
            50314, // <|bs|>
            50315, // <|kk|>
            50316, // <|sq|>
            50317, // <|sw|>
            50318, // <|gl|>
            50319, // <|mr|>
            50320, // <|pa|>
            50321, // <|si|>
            50322, // <|km|>
            50323, // <|sn|>
            50324, // <|yo|>
            50325, // <|so|>
            50326, // <|af|>
            50327, // <|oc|>
            50328, // <|ka|>
            50329, // <|be|>
            50330, // <|tg|>
            50331, // <|sd|>
            50332, // <|gu|>
            50333, // <|am|>
            50334, // <|yi|>
            50335, // <|lo|>
            50336, // <|uz|>
            50337, // <|fo|>
            50338, // <|ht|>
            50339, // <|ps|>
            50340, // <|tk|>
            50341, // <|nn|>
            50342, // <|mt|>
            50343, // <|sa|>
            50344, // <|lb|>
            50345, // <|my|>
            50346, // <|bo|>
            50347, // <|tl|>
            50348, // <|mg|>
            50349, // <|as|>
            50350, // <|tt|>
            50351, // <|haw|>
            50352, // <|ln|>
            50353, // <|ha|>
            50354, // <|ba|>
            50355, // <|jw|>
            50356, // <|su|>
            // Also add task tokens
            50357, // <|translate|>
            50358, // <|transcribe|>
            50359, // <|startoflm|>
            50360, // <|startofprev|>
            50361, // <|nospeech|>
            50362, // <|notimestamps|>
        ]);

        debug!("Loaded {} tokens from vocabulary", vocab.len());
        debug!("Configured {} special tokens", special_tokens.len());

        Ok(Self {
            vocab,
            special_tokens,
        })
    }

    /// Decode a sequence of token IDs to text
    pub fn decode(&self, tokens: &[i64]) -> Result<String> {
        if tokens.is_empty() {
            return Ok(String::new());
        }

        let mut text = String::new();
        let mut unknown_count = 0;

        for &token in tokens {
            // Skip special tokens (Whisper task/language markers)
            if self.special_tokens.contains(&token) {
                debug!("Skipping special token: {}", token);
                continue;
            }

            if let Some(token_str) = self.vocab.get(&token) {
                text.push_str(token_str);
            } else {
                // For unknown tokens, skip or represent them
                unknown_count += 1;
                debug!("Unknown token: {}", token);
            }
        }

        if unknown_count > 0 {
            warn!(
                "Decoded {} unknown tokens out of {}",
                unknown_count,
                tokens.len()
            );
        }

        // GPT-2 uses "Ġ" (U+0120) to represent spaces
        // Replace it with actual spaces
        let text = text.replace('Ġ', " ");

        // Clean up: trim whitespace and collapse multiple spaces
        let cleaned = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        Ok(cleaned)
    }
}

impl Default for SimpleTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_decoding() {
        let tokenizer = SimpleTokenizer::new();

        // Test some basic tokens
        let tokens = vec![262, 220, 290]; // " the", " ", " a"
        let text = tokenizer.decode(&tokens).unwrap();
        assert!(!text.is_empty());
    }

    #[test]
    fn test_empty_tokens() {
        let tokenizer = SimpleTokenizer::new();
        let text = tokenizer.decode(&[]).unwrap();
        assert_eq!(text, "");
    }
}
