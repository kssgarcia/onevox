//! Whisper.cpp Backend
//!
//! High-performance local speech recognition using whisper.cpp.
//! This backend uses battle-tested C++ implementation with optimized
//! preprocessing and inference.

#[cfg(feature = "whisper-cpp")]
use super::runtime::{ModelConfig, ModelRuntime, Transcription};
#[cfg(feature = "whisper-cpp")]
use anyhow::{Context, Result};
#[cfg(feature = "whisper-cpp")]
use std::path::PathBuf;
#[cfg(feature = "whisper-cpp")]
use tracing::{debug, info, warn};
#[cfg(feature = "whisper-cpp")]
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Whisper.cpp model backend
#[cfg(feature = "whisper-cpp")]
pub struct WhisperCpp {
    ctx: Option<WhisperContext>,
    config: Option<ModelConfig>,
    model_path: Option<PathBuf>,
}

#[cfg(feature = "whisper-cpp")]
impl WhisperCpp {
    /// Create a new Whisper.cpp backend
    pub fn new() -> Result<Self> {
        info!("Initializing Whisper.cpp backend");

        Ok(Self {
            ctx: None,
            config: None,
            model_path: None,
        })
    }

    /// Get the model path from cache
    fn resolve_model_path(&self, model_id: &str) -> Result<PathBuf> {
        // Check common locations for GGML models
        let cache_dir = dirs::cache_dir()
            .context("Failed to get cache directory")?
            .join("onevox")
            .join("models");

        // Try different possible locations
        let possible_paths = vec![
            cache_dir.join(model_id).join("ggml-model.bin"),
            cache_dir.join(format!("{}.bin", model_id)),
            cache_dir.join(model_id).join(format!("{}.bin", model_id)),
            cache_dir.join(model_id).join("model.bin"),
        ];

        for path in &possible_paths {
            if path.exists() {
                info!("Found model at: {:?}", path);
                return Ok(path.clone());
            }
        }

        // Return the expected path even if it doesn't exist
        // (will fail on load with helpful error)
        let expected = cache_dir.join(model_id).join("ggml-model.bin");
        warn!("Model not found at any expected location");
        debug!("Searched paths: {:?}", possible_paths);
        debug!("Expected path: {:?}", expected);

        Ok(expected)
    }
}

#[cfg(feature = "whisper-cpp")]
impl Default for WhisperCpp {
    fn default() -> Self {
        Self::new().expect("Failed to create WhisperCpp")
    }
}

#[cfg(feature = "whisper-cpp")]
impl ModelRuntime for WhisperCpp {
    fn load(&mut self, config: ModelConfig) -> crate::Result<()> {
        info!("Loading Whisper.cpp model: {:?}", config.model_path);

        // Resolve model path
        let model_path = self
            .resolve_model_path(&config.model_path)
            .map_err(|e| crate::Error::Model(format!("Failed to resolve model path: {}", e)))?;

        if !model_path.exists() {
            return Err(crate::Error::Model(format!(
                "Model file not found: {:?}\nDownload GGML models with: onevox models download {}",
                model_path, config.model_path
            )));
        }

        info!("Loading model from: {:?}", model_path);

        // Create context parameters
        let ctx_params = WhisperContextParameters::default();

        // Load the model
        let ctx = WhisperContext::new_with_params(
            model_path
                .to_str()
                .context("Invalid UTF-8 in model path")
                .map_err(|e| crate::Error::Model(e.to_string()))?,
            ctx_params,
        )
        .map_err(|e| crate::Error::Model(format!("Failed to load whisper.cpp model: {}", e)))?;

        info!("✅ Whisper.cpp model loaded successfully");

        self.ctx = Some(ctx);
        self.config = Some(config);
        self.model_path = Some(model_path);

        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.ctx.is_some()
    }

    fn transcribe(&mut self, samples: &[f32], sample_rate: u32) -> crate::Result<Transcription> {
        let ctx = self
            .ctx
            .as_mut()
            .ok_or_else(|| crate::Error::Model("Model not loaded".to_string()))?;

        let config = self
            .config
            .as_ref()
            .ok_or_else(|| crate::Error::Model("Config not set".to_string()))?;

        // Verify sample rate
        if sample_rate != 16000 {
            warn!(
                "Sample rate is {} Hz, but Whisper expects 16kHz. Resampling required.",
                sample_rate
            );
            // TODO: Add resampling if needed
            return Err(crate::Error::Model(
                "Sample rate must be 16kHz. Please resample audio.".to_string(),
            ));
        }

        let start = std::time::Instant::now();

        info!(
            "Transcribing {} samples ({:.2}s of audio)",
            samples.len(),
            samples.len() as f32 / sample_rate as f32
        );

        // Create transcription parameters
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Configure parameters from ModelConfig
        params.set_n_threads(config.n_threads as i32);
        params.set_language(Some(&config.language));
        params.set_translate(config.translate);
        params.set_print_progress(false);
        params.set_print_special(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_token_timestamps(false);
        params.set_suppress_blank(true);
        params.set_suppress_non_speech_tokens(true);

        // Run transcription
        ctx.full(params, samples)
            .map_err(|e| crate::Error::Model(format!("Transcription failed: {}", e)))?;

        // Extract results
        let num_segments = ctx
            .full_n_segments()
            .map_err(|e| crate::Error::Model(format!("Failed to get segments: {}", e)))?;

        debug!("Generated {} segments", num_segments);

        // Concatenate all segments
        let mut full_text = String::new();
        for i in 0..num_segments {
            let segment = ctx
                .full_get_segment_text(i)
                .map_err(|e| crate::Error::Model(format!("Failed to get segment {}: {}", i, e)))?;
            full_text.push_str(&segment);
        }

        let processing_time = start.elapsed();

        info!(
            "Transcription complete: \"{}\" ({} ms)",
            full_text.trim(),
            processing_time.as_millis()
        );

        Ok(Transcription {
            text: full_text.trim().to_string(),
            language: Some(config.language.clone()),
            confidence: None, // whisper-rs doesn't expose confidence easily
            processing_time_ms: processing_time.as_millis() as u64,
            tokens: Some(num_segments as usize),
        })
    }

    fn unload(&mut self) {
        info!("Unloading Whisper.cpp model");
        self.ctx = None;
        self.config = None;
        self.model_path = None;
    }

    fn name(&self) -> &str {
        "whisper.cpp"
    }

    fn info(&self) -> crate::Result<super::runtime::ModelInfo> {
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| crate::Error::Model("Model not loaded".to_string()))?;

        Ok(super::runtime::ModelInfo {
            name: self.name().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            backend: "whisper.cpp".to_string(),
            model_path: self
                .model_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            language: Some(config.language.clone()),
            is_multilingual: config.language == "auto",
        })
    }
}

#[cfg(test)]
#[cfg(feature = "whisper-cpp")]
mod tests {
    use super::*;

    #[test]
    fn test_create_backend() {
        let backend = WhisperCpp::new();
        assert!(backend.is_ok());
    }

    #[test]
    fn test_not_loaded_initially() {
        let backend = WhisperCpp::new().unwrap();
        assert!(!backend.is_loaded());
    }
}
