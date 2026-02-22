//! Whisper Candle Backend (Optional)
//!
//! Pure Rust implementation using the Candle ML framework.
//! This is an experimental backend for future extensibility.
//!
//! Note: This backend is optional and not the primary production backend.
//! The primary backend is whisper.cpp (native bindings).

#[cfg(feature = "candle")]
use super::runtime::{ModelConfig, ModelInfo, ModelRuntime, Transcription};

#[cfg(feature = "candle")]
pub struct WhisperCandle {
    // TODO: Implement Candle-based Whisper model
    _placeholder: (),
}

#[cfg(feature = "candle")]
impl WhisperCandle {
    pub fn new() -> crate::Result<Self> {
        Err(crate::Error::Model(
            "Candle backend not yet implemented. Use whisper-cpp backend instead.".to_string(),
        ))
    }
}

#[cfg(feature = "candle")]
impl ModelRuntime for WhisperCandle {
    fn load(&mut self, _config: ModelConfig) -> crate::Result<()> {
        Err(crate::Error::Model(
            "Candle backend not yet implemented".to_string(),
        ))
    }

    fn is_loaded(&self) -> bool {
        false
    }

    fn transcribe(&mut self, _samples: &[f32], _sample_rate: u32) -> crate::Result<Transcription> {
        Err(crate::Error::Model(
            "Candle backend not yet implemented".to_string(),
        ))
    }

    fn unload(&mut self) {}

    fn name(&self) -> &str {
        "whisper-candle"
    }

    fn info(&self) -> ModelInfo {
        ModelInfo {
            name: "whisper-candle".to_string(),
            size_bytes: 0,
            model_type: "whisper".to_string(),
            backend: "candle (experimental)".to_string(),
            gpu_enabled: false,
        }
    }
}

// Stub when feature is disabled
#[cfg(not(feature = "candle"))]
pub struct WhisperCandle;

#[cfg(not(feature = "candle"))]
impl WhisperCandle {
    pub fn new() -> crate::Result<Self> {
        Err(crate::Error::Model(
            "Candle feature not enabled. Rebuild with --features candle".to_string(),
        ))
    }
}
