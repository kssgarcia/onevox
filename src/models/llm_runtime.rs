//! LLM Runtime Trait
//!
//! Abstract interface for Large Language Model backends.

use crate::Result;
use crate::tools::ToolCall;

/// LLM generation result
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// Generated text
    pub text: String,

    /// Tool calls requested by LLM (if any)
    pub tool_calls: Vec<ToolCall>,

    /// Tokens generated
    pub tokens: usize,

    /// Generation time in milliseconds
    pub generation_time_ms: u64,

    /// Tokens per second
    pub tokens_per_second: f32,

    /// Finish reason (e.g., "stop", "length", "error")
    pub finish_reason: Option<String>,
}

impl LlmResponse {
    /// Create a new LLM response
    pub fn new(text: String) -> Self {
        Self {
            text,
            tool_calls: Vec::new(),
            tokens: 0,
            generation_time_ms: 0,
            tokens_per_second: 0.0,
            finish_reason: None,
        }
    }

    /// Check if response is empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// LLM runtime configuration
#[derive(Debug, Clone)]
pub struct LlmRuntimeConfig {
    /// Path to model file
    pub model_path: String,

    /// Use GPU acceleration
    pub use_gpu: bool,

    /// Context length (max tokens in context)
    pub context_length: usize,

    /// Temperature (0.0 - 2.0)
    pub temperature: f32,

    /// Max tokens to generate
    pub max_tokens: usize,

    /// Top-p (nucleus sampling)
    pub top_p: f32,

    /// Top-k sampling
    pub top_k: u32,

    /// Repetition penalty
    pub repetition_penalty: f32,
}

impl Default for LlmRuntimeConfig {
    fn default() -> Self {
        Self {
            model_path: "models/lfm2-1.2b-tool.gguf".to_string(),
            use_gpu: false,
            context_length: 2048,
            temperature: 0.7,
            max_tokens: 256,
            top_p: 0.95,
            top_k: 40,
            repetition_penalty: 1.1,
        }
    }
}

/// Conversation message
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// Message role
    pub role: MessageRole,

    /// Message content
    pub content: String,

    /// Optional timestamp
    pub timestamp: Option<std::time::SystemTime>,
}

impl ChatMessage {
    /// Create a new chat message
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content,
            timestamp: Some(std::time::SystemTime::now()),
        }
    }

    /// Create a system message
    pub fn system(content: String) -> Self {
        Self::new(MessageRole::System, content)
    }

    /// Create a user message
    pub fn user(content: String) -> Self {
        Self::new(MessageRole::User, content)
    }

    /// Create an assistant message
    pub fn assistant(content: String) -> Self {
        Self::new(MessageRole::Assistant, content)
    }

    /// Create a tool result message (used to feed tool output back to the LLM)
    pub fn tool(content: String) -> Self {
        Self::new(MessageRole::Tool, content)
    }
}

/// Message role in conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    /// System message (instructions, context)
    System,

    /// User message (user input)
    User,

    /// Assistant message (AI response)
    Assistant,

    /// Tool result message (fed back to LLM after tool execution)
    Tool,
}

impl MessageRole {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool",
        }
    }
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// LLM runtime trait
///
/// Provides a unified interface for different LLM backends (GGUF, ONNX, etc.)
pub trait LlmRuntime: Send + Sync {
    /// Load the model
    fn load(&mut self, config: LlmRuntimeConfig) -> Result<()>;

    /// Check if model is loaded
    fn is_loaded(&self) -> bool;

    /// Generate response from conversation history
    ///
    /// # Arguments
    /// * `messages` - Conversation history (system, user, assistant messages)
    ///
    /// # Returns
    /// Generated response with metadata
    fn generate(&mut self, messages: &[ChatMessage]) -> Result<LlmResponse>;

    /// Generate response with streaming (optional, default non-streaming)
    ///
    /// # Arguments
    /// * `messages` - Conversation history
    /// * `callback` - Called for each generated token/text chunk
    ///
    /// # Returns
    /// Complete response when generation finishes
    fn generate_stream(
        &mut self,
        messages: &[ChatMessage],
        callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<LlmResponse> {
        // Default implementation: non-streaming, call callback once with full text
        let response = self.generate(messages)?;
        let mut cb = callback;
        cb(&response.text);
        Ok(response)
    }

    /// Unload the model and free resources
    fn unload(&mut self);

    /// Get model name/identifier
    fn name(&self) -> &str;

    /// Get model information
    fn info(&self) -> LlmInfo {
        LlmInfo {
            name: self.name().to_string(),
            loaded: self.is_loaded(),
            backend: "unknown".to_string(),
        }
    }
}

/// LLM model information
#[derive(Debug, Clone)]
pub struct LlmInfo {
    /// Model name
    pub name: String,

    /// Whether model is currently loaded
    pub loaded: bool,

    /// Backend name (e.g., "gguf", "onnx")
    pub backend: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_creation() {
        let msg = ChatMessage::user("Hello".to_string());
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
        assert!(msg.timestamp.is_some());
    }

    #[test]
    fn test_message_role_display() {
        assert_eq!(MessageRole::System.as_str(), "system");
        assert_eq!(MessageRole::User.as_str(), "user");
        assert_eq!(MessageRole::Assistant.as_str(), "assistant");
        assert_eq!(MessageRole::Tool.as_str(), "tool");
    }

    #[test]
    fn test_llm_response() {
        let response = LlmResponse::new("Test response".to_string());
        assert_eq!(response.text, "Test response");
        assert!(!response.is_empty());

        let empty = LlmResponse::new("".to_string());
        assert!(empty.is_empty());
    }

    #[test]
    fn test_default_config() {
        let config = LlmRuntimeConfig::default();
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 256);
        assert_eq!(config.context_length, 2048);
    }
}
