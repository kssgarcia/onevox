//! Model runtime abstraction
//!
//! Unified interface for multiple transcription backends.
//!
//! Primary backend: whisper.cpp (native bindings)
//! Optional backend: Candle (pure Rust, experimental)

pub mod downloader;
pub mod mock;
pub mod registry;
pub mod runtime;
pub mod tokenizer;
pub mod whisper_cpp;

#[cfg(feature = "candle")]
pub mod whisper_candle;

// Re-export commonly used types
pub use downloader::ModelDownloader;
pub use mock::MockModel;
pub use registry::{ModelMetadata, ModelRegistry, ModelSize, ModelVariant};
pub use runtime::{ModelConfig, ModelInfo, ModelRuntime, Transcription};
pub use tokenizer::SimpleTokenizer;
pub use whisper_cpp::WhisperCpp;

#[cfg(feature = "candle")]
pub use whisper_candle::WhisperCandle;
