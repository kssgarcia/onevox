//! Mock Model Backend
//!
//! Simple mock model for testing the transcription pipeline.
//! Returns fake transcriptions based on audio duration.

use super::runtime::{ModelConfig, ModelInfo, ModelRuntime, Transcription};
use std::time::Instant;
use tracing::info;

/// Mock model for testing
pub struct MockModel {
    is_loaded: bool,
    config: Option<ModelConfig>,
    transcription_count: usize,
}

impl MockModel {
    /// Create a new mock model
    pub fn new() -> Self {
        Self {
            is_loaded: false,
            config: None,
            transcription_count: 0,
        }
    }
}

impl Default for MockModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRuntime for MockModel {
    fn load(&mut self, config: ModelConfig) -> crate::Result<()> {
        info!("Loading mock model (no actual model file needed)");
        self.config = Some(config);
        self.is_loaded = true;
        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    fn transcribe(&self, samples: &[f32], sample_rate: u32) -> crate::Result<Transcription> {
        if !self.is_loaded {
            return Err(crate::Error::Model("Model not loaded".to_string()));
        }

        let start = Instant::now();

        // Calculate duration
        let duration_secs = samples.len() as f32 / sample_rate as f32;

        // Generate fake transcription based on duration
        let text = if duration_secs < 0.5 {
            "[too short]".to_string()
        } else if duration_secs < 2.0 {
            format!("[Mock transcription of {:.1}s audio]", duration_secs)
        } else {
            format!(
                "[Mock transcription: This is a simulated transcription of {:.1} seconds of audio.]",
                duration_secs
            )
        };

        let processing_time = start.elapsed();

        Ok(Transcription {
            text,
            language: Some("en".to_string()),
            confidence: Some(0.95),
            processing_time_ms: processing_time.as_millis() as u64,
            tokens: Some((duration_secs * 2.0) as usize), // Fake: ~2 tokens per second
        })
    }

    fn unload(&mut self) {
        info!("Unloading mock model");
        self.is_loaded = false;
        self.config = None;
        self.transcription_count = 0;
    }

    fn name(&self) -> &str {
        "Mock Model (Testing Only)"
    }

    fn info(&self) -> ModelInfo {
        ModelInfo {
            name: "mock-model".to_string(),
            size_bytes: 0,
            model_type: "mock".to_string(),
            backend: "mock".to_string(),
            gpu_enabled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_model() {
        let mut model = MockModel::new();
        assert!(!model.is_loaded());

        // Load model
        let config = ModelConfig::default();
        model.load(config).unwrap();
        assert!(model.is_loaded());

        // Transcribe
        let samples = vec![0.0; 16000]; // 1 second at 16kHz
        let result = model.transcribe(&samples, 16000).unwrap();
        assert!(!result.is_empty());
        assert!(result.text.contains("Mock"));

        // Unload
        model.unload();
        assert!(!model.is_loaded());
    }
}
