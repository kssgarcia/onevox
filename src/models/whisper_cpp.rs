//! Whisper.cpp Backend
//!
//! High-performance local speech recognition using whisper.cpp native bindings.
//! This is the primary production backend for cross-platform stability.

use super::runtime::{ModelConfig, ModelInfo, ModelRuntime, Transcription};
use std::path::PathBuf;
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
    pub fn new() -> crate::Result<Self> {
        info!("Initializing Whisper.cpp backend");

        Ok(Self {
            ctx: None,
            config: None,
            model_path: None,
        })
    }

    /// Get the model path from cache
    fn resolve_model_path(&self, model_id: &str) -> crate::Result<PathBuf> {
        // If it's already an absolute path that exists, use it directly
        let direct_path = PathBuf::from(model_id);
        if direct_path.is_absolute() && direct_path.exists() {
            info!("Using absolute model path: {:?}", direct_path);
            return Ok(direct_path);
        }

        // Get the models directory
        let models_dir = crate::platform::paths::models_dir()
            .unwrap_or_else(|_| PathBuf::from("./models"));

        // Try different possible locations in order of likelihood
        let possible_paths = vec![
            // 1. Direct file in models directory (most common for direct .bin files)
            models_dir.join(model_id),
            // 2. In subdirectory: models/model-id/model-id.bin
            models_dir.join(model_id).join(format!("{}.bin", model_id)),
            // 3. In subdirectory with .bin extension: models/model-id.bin/model-id.bin
            models_dir.join(format!("{}.bin", model_id)).join(format!("{}.bin", model_id)),
            // 4. Standard naming: model_id/ggml-model.bin
            models_dir.join(model_id).join("ggml-model.bin"),
            // 5. If model_id already has .bin, try as-is in subdirectory
            models_dir.join(model_id).join(model_id),
        ];

        for path in &possible_paths {
            if path.exists() && path.is_file() {
                info!("Found model at: {:?}", path);
                return Ok(path.clone());
            }
        }

        // Return the most likely expected path for a helpful error message
        let expected = models_dir.join(model_id).join(format!("{}.bin", model_id));
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
        let model_path = self.resolve_model_path(&config.model_path)?;

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
                .ok_or_else(|| crate::Error::Model("Invalid UTF-8 in model path".to_string()))?,
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
            .as_ref()
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
        params.set_suppress_nst(true); // Suppress non-speech tokens (renamed in newer API)

        // Create a state for this transcription (whisper-rs 0.14+ API)
        let mut state = ctx
            .create_state()
            .map_err(|e| crate::Error::Model(format!("Failed to create state: {}", e)))?;

        // Run transcription
        state
            .full(params, samples)
            .map_err(|e| crate::Error::Model(format!("Transcription failed: {}", e)))?;

        // Extract results using the new iterator API
        let mut full_text = String::new();
        let mut num_segments = 0;

        for segment in state.as_iter() {
            num_segments += 1;
            let segment_text = segment.to_string();
            full_text.push_str(&segment_text);
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
            tokens: Some(num_segments),
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

    fn info(&self) -> ModelInfo {
        let config = self.config.as_ref();

        ModelInfo {
            name: self.name().to_string(),
            size_bytes: self
                .model_path
                .as_ref()
                .and_then(|p| std::fs::metadata(p).ok())
                .map(|m| m.len())
                .unwrap_or(0),
            model_type: config
                .as_ref()
                .map(|c| c.model_path.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            backend: "whisper.cpp".to_string(),
            gpu_enabled: config.map(|c| c.use_gpu).unwrap_or(false),
        }
    }
}

// Stub implementation when feature is disabled
#[cfg(not(feature = "whisper-cpp"))]
pub struct WhisperCpp;

#[cfg(not(feature = "whisper-cpp"))]
impl WhisperCpp {
    pub fn new() -> crate::Result<Self> {
        Err(crate::Error::Model(
            "whisper-cpp feature not enabled.".to_string(),
        ))
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
