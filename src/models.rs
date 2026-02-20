//! Model runtime abstraction
//!
//! Unified interface for multiple transcription backends.

pub mod downloader;
pub mod mock;
pub mod registry;
pub mod runtime;
pub mod tokenizer;
pub mod whisper_onnx;
// pub mod whisper;  // TODO: Re-enable when build issues resolved

// Re-export commonly used types
pub use downloader::ModelDownloader;
pub use mock::MockModel;
pub use registry::{ModelMetadata, ModelRegistry, ModelSize, ModelVariant};
pub use runtime::{ModelConfig, ModelInfo, ModelRuntime, Transcription};
pub use tokenizer::SimpleTokenizer;

#[cfg(feature = "onnx")]
pub use whisper_onnx::WhisperOnnx;

// pub use whisper::WhisperModel;
