//! TTS Runtime Trait
//!
//! Abstract interface for Text-to-Speech backends.

use crate::Result;

/// TTS synthesis result
#[derive(Debug, Clone)]
pub struct TtsSynthesis {
    /// Audio samples (f32, mono)
    pub samples: Vec<f32>,

    /// Sample rate (Hz)
    pub sample_rate: u32,

    /// Synthesis time in milliseconds
    pub synthesis_time_ms: u64,

    /// Audio duration in milliseconds
    pub audio_duration_ms: u64,

    /// Real-time factor (< 1.0 is faster than real-time)
    pub rtf: f32,
}

impl TtsSynthesis {
    /// Create a new TTS synthesis result
    pub fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        let audio_duration_ms = (samples.len() as f32 / sample_rate as f32 * 1000.0) as u64;

        Self {
            samples,
            sample_rate,
            synthesis_time_ms: 0,
            audio_duration_ms,
            rtf: 0.0,
        }
    }

    /// Check if synthesis is empty
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Get duration in seconds
    pub fn duration_secs(&self) -> f32 {
        self.audio_duration_ms as f32 / 1000.0
    }

    /// Calculate RTF from synthesis time
    pub fn with_synthesis_time(mut self, synthesis_time_ms: u64) -> Self {
        self.synthesis_time_ms = synthesis_time_ms;
        if self.audio_duration_ms > 0 {
            self.rtf = synthesis_time_ms as f32 / self.audio_duration_ms as f32;
        }
        self
    }
}

/// TTS runtime configuration
#[derive(Debug, Clone)]
pub struct TtsRuntimeConfig {
    /// Path to model file
    pub model_path: String,

    /// Use GPU acceleration
    pub use_gpu: bool,

    /// Voice/speaker ID
    pub voice_id: String,

    /// Speech rate (0.5 - 2.0, 1.0 is normal)
    pub speech_rate: f32,

    /// Pitch adjustment (-1.0 to 1.0, 0.0 is normal)
    pub pitch: f32,

    /// Volume (0.0 - 1.0, 1.0 is max)
    pub volume: f32,
}

impl Default for TtsRuntimeConfig {
    fn default() -> Self {
        Self {
            model_path: "models/kokoro-82m-onnx".to_string(),
            use_gpu: false,
            voice_id: "af_sarah".to_string(),
            speech_rate: 1.0,
            pitch: 0.0,
            volume: 1.0,
        }
    }
}

/// Voice information
#[derive(Debug, Clone)]
pub struct VoiceInfo {
    /// Voice identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Language code (e.g., "en-US", "en-GB")
    pub language: String,

    /// Gender (e.g., "female", "male", "neutral")
    pub gender: Option<String>,

    /// Description
    pub description: Option<String>,
}

impl VoiceInfo {
    /// Create a new voice info
    pub fn new(id: String, name: String, language: String) -> Self {
        Self {
            id,
            name,
            language,
            gender: None,
            description: None,
        }
    }

    /// Set gender
    pub fn with_gender(mut self, gender: String) -> Self {
        self.gender = Some(gender);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// TTS runtime trait
///
/// Provides a unified interface for different TTS backends (ONNX, etc.)
pub trait TtsRuntime: Send + Sync {
    /// Load the model
    fn load(&mut self, config: TtsRuntimeConfig) -> Result<()>;

    /// Check if model is loaded
    fn is_loaded(&self) -> bool;

    /// Synthesize speech from text
    ///
    /// # Arguments
    /// * `text` - Text to synthesize
    ///
    /// # Returns
    /// Audio samples with metadata
    fn synthesize(&mut self, text: &str) -> Result<TtsSynthesis>;

    /// List available voices
    fn list_voices(&self) -> Vec<VoiceInfo>;

    /// Set voice (if model supports multiple voices)
    fn set_voice(&mut self, voice_id: &str) -> Result<()> {
        // Default: no-op (override if model supports voice switching)
        tracing::warn!("Voice switching not supported by this TTS backend");
        let _ = voice_id;
        Ok(())
    }

    /// Unload the model and free resources
    fn unload(&mut self);

    /// Get model name/identifier
    fn name(&self) -> &str;

    /// Get model information
    fn info(&self) -> TtsInfo {
        TtsInfo {
            name: self.name().to_string(),
            loaded: self.is_loaded(),
            backend: "unknown".to_string(),
            available_voices: self.list_voices().len(),
        }
    }
}

/// TTS model information
#[derive(Debug, Clone)]
pub struct TtsInfo {
    /// Model name
    pub name: String,

    /// Whether model is currently loaded
    pub loaded: bool,

    /// Backend name (e.g., "kokoro", "vits")
    pub backend: String,

    /// Number of available voices
    pub available_voices: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_synthesis() {
        let samples = vec![0.0, 0.1, 0.2, 0.3];
        let synthesis = TtsSynthesis::new(samples.clone(), 22050);

        assert_eq!(synthesis.samples.len(), 4);
        assert_eq!(synthesis.sample_rate, 22050);
        assert!(!synthesis.is_empty());

        let empty = TtsSynthesis::new(vec![], 22050);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_tts_synthesis_duration() {
        let sample_rate = 22050;
        let samples = vec![0.0; sample_rate as usize]; // 1 second of audio

        let synthesis = TtsSynthesis::new(samples, sample_rate);
        assert!((synthesis.duration_secs() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_tts_synthesis_rtf() {
        let synthesis = TtsSynthesis::new(vec![0.0; 22050], 22050).with_synthesis_time(500); // 500ms to synthesize 1 second

        assert!((synthesis.rtf - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_voice_info() {
        let voice = VoiceInfo::new(
            "af_heart".to_string(),
            "Heart".to_string(),
            "en-US".to_string(),
        )
        .with_gender("female".to_string())
        .with_description("Warm, friendly voice".to_string());

        assert_eq!(voice.id, "af_heart");
        assert_eq!(voice.gender, Some("female".to_string()));
        assert!(voice.description.is_some());
    }

    #[test]
    fn test_default_config() {
        let config = TtsRuntimeConfig::default();
        assert_eq!(config.speech_rate, 1.0);
        assert_eq!(config.pitch, 0.0);
        assert_eq!(config.volume, 1.0);
    }
}
