//! Chat Module - Voice-based Conversational AI
//!
//! Provides STT→LLM→TTS pipeline for natural voice conversations.

pub mod engine;
pub mod sentence_splitter;

// Re-export commonly used types
pub use engine::{ChatEngine, ChatEngineStatus, ChatResponse};
pub use sentence_splitter::SentenceSplitter;
