//! Sentence Splitting Utility
//!
//! Detects complete sentences from streaming text for incremental TTS processing.

/// Sentence splitter for streaming text
pub struct SentenceSplitter {
    /// Buffer holding incomplete text
    buffer: String,
}

impl SentenceSplitter {
    /// Create a new sentence splitter
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Add text chunk and extract complete sentences
    ///
    /// Returns a vector of complete sentences that can be synthesized.
    /// Incomplete text remains in the buffer.
    pub fn add_chunk(&mut self, chunk: &str) -> Vec<String> {
        self.buffer.push_str(chunk);
        self.extract_sentences()
    }

    /// Force flush remaining buffer as a sentence
    ///
    /// Call this when the stream is complete to get any remaining text.
    pub fn flush(&mut self) -> Option<String> {
        if self.buffer.trim().is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.buffer))
        }
    }

    /// Extract complete sentences from buffer
    fn extract_sentences(&mut self) -> Vec<String> {
        let mut sentences = Vec::new();

        // Sentence endings: . ! ? followed by space or newline
        // Also consider newlines as potential sentence boundaries
        while !self.buffer.is_empty() {
            if let Some((sentence, remainder)) = self.split_next_sentence(&self.buffer) {
                if !sentence.trim().is_empty() {
                    sentences.push(sentence);
                }
                self.buffer = remainder.to_string();
            } else {
                break;
            }
        }

        sentences
    }

    /// Try to split off the next complete sentence
    ///
    /// Returns (sentence, remainder) if a complete sentence is found, None otherwise.
    fn split_next_sentence<'a>(&self, text: &'a str) -> Option<(String, &'a str)> {
        // First check for paragraph breaks (higher priority than sentence endings within a paragraph)
        if let Some(pos) = text.find("\n\n") {
            let sentence = text[..pos].to_string();
            let remainder = &text[pos + 2..];
            return Some((sentence, remainder));
        }

        // Look for sentence-ending punctuation followed by space or newline
        let endings = [". ", ".\n", "! ", "!\n", "? ", "?\n"];

        let mut earliest_match: Option<(usize, &str)> = None;

        for ending in &endings {
            if let Some(pos) = text.find(ending) {
                match earliest_match {
                    None => earliest_match = Some((pos, ending)),
                    Some((earliest_pos, _)) if pos < earliest_pos => {
                        earliest_match = Some((pos, ending));
                    }
                    _ => {}
                }
            }
        }

        if let Some((pos, ending)) = earliest_match {
            let end_pos = pos + ending.len();
            let sentence = text[..end_pos].to_string();
            let remainder = &text[end_pos..];
            return Some((sentence, remainder));
        }

        None
    }

    /// Reset the splitter
    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    /// Get current buffer content (for debugging)
    #[allow(dead_code)]
    pub fn buffer(&self) -> &str {
        &self.buffer
    }
}

impl Default for SentenceSplitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_sentence() {
        let mut splitter = SentenceSplitter::new();
        let sentences = splitter.add_chunk("Hello world. ");
        assert_eq!(sentences, vec!["Hello world. "]);
        assert_eq!(splitter.buffer, "");
    }

    #[test]
    fn test_multiple_sentences() {
        let mut splitter = SentenceSplitter::new();
        let sentences = splitter.add_chunk("First sentence. Second sentence! ");
        assert_eq!(sentences, vec!["First sentence. ", "Second sentence! "]);
    }

    #[test]
    fn test_incomplete_sentence() {
        let mut splitter = SentenceSplitter::new();
        let sentences = splitter.add_chunk("This is incomplete");
        assert_eq!(sentences, Vec::<String>::new());
        assert_eq!(splitter.buffer, "This is incomplete");
    }

    #[test]
    fn test_streaming_completion() {
        let mut splitter = SentenceSplitter::new();

        let s1 = splitter.add_chunk("Hello");
        assert_eq!(s1, Vec::<String>::new());

        let s2 = splitter.add_chunk(" world");
        assert_eq!(s2, Vec::<String>::new());

        let s3 = splitter.add_chunk(". ");
        assert_eq!(s3, vec!["Hello world. "]);
    }

    #[test]
    fn test_question_and_exclamation() {
        let mut splitter = SentenceSplitter::new();
        let sentences = splitter.add_chunk("What is this? Amazing! ");
        assert_eq!(sentences, vec!["What is this? ", "Amazing! "]);
    }

    #[test]
    fn test_flush() {
        let mut splitter = SentenceSplitter::new();
        splitter.add_chunk("Incomplete sentence");

        let flushed = splitter.flush();
        assert_eq!(flushed, Some("Incomplete sentence".to_string()));
        assert_eq!(splitter.buffer, "");
    }

    #[test]
    fn test_flush_empty() {
        let mut splitter = SentenceSplitter::new();
        let flushed = splitter.flush();
        assert_eq!(flushed, None);
    }

    #[test]
    fn test_newline_boundaries() {
        let mut splitter = SentenceSplitter::new();
        let sentences = splitter.add_chunk("First line.\nSecond line. ");
        assert_eq!(sentences, vec!["First line.\n", "Second line. "]);
    }

    #[test]
    fn test_paragraph_break() {
        let mut splitter = SentenceSplitter::new();
        let sentences = splitter.add_chunk("First paragraph\n\nSecond paragraph. ");
        assert_eq!(sentences, vec!["First paragraph", "Second paragraph. "]);
    }

    #[test]
    fn test_reset() {
        let mut splitter = SentenceSplitter::new();
        splitter.add_chunk("Some text");
        splitter.reset();
        assert_eq!(splitter.buffer, "");
    }
}
