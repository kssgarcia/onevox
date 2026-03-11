//! Mock LLM Implementation
//!
//! Simple mock LLM for testing without requiring actual model files.

use super::llm_runtime::*;
use crate::Result;
use std::time::Instant;

/// Mock LLM runtime for testing
pub struct MockLlm {
    loaded: bool,
    config: Option<LlmRuntimeConfig>,
    /// Predefined responses for testing
    responses: Vec<String>,
    /// Current response index
    response_index: usize,
    /// Simulated delay in milliseconds
    delay_ms: u64,
}

impl MockLlm {
    /// Create a new mock LLM
    pub fn new() -> Self {
        Self {
            loaded: false,
            config: None,
            responses: vec![
                "This is a mock response.".to_string(),
                "I understand your question.".to_string(),
                "Let me help you with that.".to_string(),
            ],
            response_index: 0,
            delay_ms: 100,
        }
    }

    /// Create a mock LLM with custom responses
    pub fn with_responses(responses: Vec<String>) -> Self {
        Self {
            loaded: false,
            config: None,
            responses,
            response_index: 0,
            delay_ms: 100,
        }
    }

    /// Set simulated delay
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Get next response (cycles through responses)
    fn next_response(&mut self) -> String {
        let response = self.responses[self.response_index].clone();
        self.response_index = (self.response_index + 1) % self.responses.len();
        response
    }
}

impl Default for MockLlm {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmRuntime for MockLlm {
    fn load(&mut self, config: LlmRuntimeConfig) -> Result<()> {
        tracing::info!("🤖 Loading mock LLM: {}", config.model_path);

        // Simulate loading delay
        std::thread::sleep(std::time::Duration::from_millis(50));

        self.config = Some(config);
        self.loaded = true;

        tracing::info!("✅ Mock LLM loaded successfully");
        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn generate(&mut self, messages: &[ChatMessage]) -> Result<LlmResponse> {
        if !self.loaded {
            return Err(crate::Error::Model("Mock LLM not loaded".to_string()));
        }

        let start = Instant::now();

        // Simulate generation delay
        std::thread::sleep(std::time::Duration::from_millis(self.delay_ms));

        // Get next mock response
        let text = self.next_response();
        let tokens = text.split_whitespace().count();

        let generation_time_ms = start.elapsed().as_millis() as u64;
        let tokens_per_second = if generation_time_ms > 0 {
            tokens as f32 / (generation_time_ms as f32 / 1000.0)
        } else {
            0.0
        };

        tracing::debug!(
            "Generated mock response: {} tokens in {}ms ({:.1} tok/s)",
            tokens,
            generation_time_ms,
            tokens_per_second
        );

        // Log the conversation for debugging
        tracing::trace!("Conversation history:");
        for msg in messages {
            tracing::trace!("  {}: {}", msg.role, msg.content);
        }

        Ok(LlmResponse {
            text,
            tool_calls: Vec::new(), // Mock doesn't use tools
            tokens,
            generation_time_ms,
            tokens_per_second,
            finish_reason: Some("stop".to_string()),
        })
    }

    fn unload(&mut self) {
        tracing::info!("Unloading mock LLM");
        self.loaded = false;
        self.config = None;
    }

    fn name(&self) -> &str {
        "mock-llm"
    }

    fn info(&self) -> LlmInfo {
        LlmInfo {
            name: self.name().to_string(),
            loaded: self.loaded,
            backend: "mock".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_llm_creation() {
        let llm = MockLlm::new();
        assert!(!llm.is_loaded());
        assert_eq!(llm.name(), "mock-llm");
    }

    #[test]
    fn test_mock_llm_load() {
        let mut llm = MockLlm::new();
        let config = LlmRuntimeConfig::default();

        llm.load(config).unwrap();
        assert!(llm.is_loaded());
    }

    #[test]
    fn test_mock_llm_generate() {
        let mut llm = MockLlm::new();
        let config = LlmRuntimeConfig::default();
        llm.load(config).unwrap();

        let messages = vec![
            ChatMessage::system("You are a helpful assistant.".to_string()),
            ChatMessage::user("Hello!".to_string()),
        ];

        let response = llm.generate(&messages).unwrap();
        assert!(!response.text.is_empty());
        assert!(response.tokens > 0);
        assert!(response.generation_time_ms >= 100); // At least delay_ms
    }

    #[test]
    fn test_mock_llm_cycles_responses() {
        let responses = vec!["First".to_string(), "Second".to_string()];
        let mut llm = MockLlm::with_responses(responses).with_delay(0);

        let config = LlmRuntimeConfig::default();
        llm.load(config).unwrap();

        let messages = vec![ChatMessage::user("Test".to_string())];

        let resp1 = llm.generate(&messages).unwrap();
        assert_eq!(resp1.text, "First");

        let resp2 = llm.generate(&messages).unwrap();
        assert_eq!(resp2.text, "Second");

        let resp3 = llm.generate(&messages).unwrap();
        assert_eq!(resp3.text, "First"); // Cycles back
    }

    #[test]
    fn test_mock_llm_unload() {
        let mut llm = MockLlm::new();
        let config = LlmRuntimeConfig::default();
        llm.load(config).unwrap();

        assert!(llm.is_loaded());
        llm.unload();
        assert!(!llm.is_loaded());
    }

    #[test]
    fn test_mock_llm_generate_without_load() {
        let mut llm = MockLlm::new();
        let messages = vec![ChatMessage::user("Test".to_string())];

        let result = llm.generate(&messages);
        assert!(result.is_err());
    }
}
