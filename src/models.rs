//! Model runtime abstraction
//!
//! Unified interface for multiple transcription backends.
//!
//! Primary backend: whisper.cpp (native bindings)
//! ONNX backend: ONNX Runtime (production-ready, supports Parakeet and other models)
//! Optional backend: Candle (pure Rust, experimental)

pub mod downloader;
pub mod gpu;
pub mod llm_gguf;
pub mod llm_mock;
pub mod llm_runtime;
pub mod mock;
pub mod onnx_runtime;
pub mod registry;
pub mod runtime;
pub mod tokenizer;
pub mod tts_kokoro;
pub mod tts_mock;
pub mod tts_runtime;
pub mod whisper_cpp;

#[cfg(feature = "candle")]
pub mod whisper_candle;

// Re-export commonly used types
pub use downloader::ModelDownloader;
pub use gpu::{GpuBackend, GpuCapabilities};
pub use llm_gguf::LlmGguf;
pub use llm_mock::MockLlm;
pub use llm_runtime::{
    ChatMessage, LlmInfo, LlmResponse, LlmRuntime, LlmRuntimeConfig, MessageRole,
};
pub use mock::MockModel;
pub use onnx_runtime::OnnxRuntime;
pub use registry::{ModelMetadata, ModelRegistry, ModelSize, ModelType, ModelVariant};
pub use runtime::{ModelConfig, ModelInfo, ModelRuntime, Transcription};
pub use tokenizer::SimpleTokenizer;
pub use tts_kokoro::TtsKokoro;
pub use tts_mock::MockTts;
pub use tts_runtime::{TtsInfo, TtsRuntime, TtsRuntimeConfig, TtsSynthesis, VoiceInfo};
pub use whisper_cpp::WhisperCpp;

#[cfg(feature = "candle")]
pub use whisper_candle::WhisperCandle;
