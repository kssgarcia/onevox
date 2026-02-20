//! Model runtime abstraction
//!
//! Unified interface for multiple transcription backends.

pub mod downloader;
pub mod mock;
pub mod registry;
pub mod runtime;
pub mod tokenizer;
pub mod whisper_cpp_cli;

#[cfg(feature = "onnx")]
pub mod whisper_onnx;

// Re-export commonly used types
pub use downloader::ModelDownloader;
pub use mock::MockModel;
pub use registry::{ModelMetadata, ModelRegistry, ModelSize, ModelVariant};
pub use runtime::{ModelConfig, ModelInfo, ModelRuntime, Transcription};
pub use tokenizer::SimpleTokenizer;
pub use whisper_cpp_cli::WhisperCppCli;

#[cfg(feature = "onnx")]
pub use whisper_onnx::WhisperOnnx;
