//! Model runtime abstraction
//!
//! Unified interface for multiple transcription backends.

pub mod mock;
pub mod runtime;
// pub mod whisper;  // TODO: Re-enable when build issues resolved

// Re-export commonly used types
pub use mock::MockModel;
pub use runtime::{ModelConfig, ModelInfo, ModelRuntime, Transcription};
// pub use whisper::WhisperModel;
