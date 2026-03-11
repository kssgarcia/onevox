//! Mock TTS Implementation
//!
//! Simple mock TTS for testing without requiring actual model files.

use super::tts_runtime::*;
use crate::Result;
use std::time::Instant;

/// Mock TTS runtime for testing
pub struct MockTts {
    loaded: bool,
    config: Option<TtsRuntimeConfig>,
    /// Simulated delay in milliseconds
    delay_ms: u64,
    /// Available voices
    voices: Vec<VoiceInfo>,
}

impl MockTts {
    /// Create a new mock TTS
    pub fn new() -> Self {
        Self {
            loaded: false,
            config: None,
            delay_ms: 50,
            voices: Self::default_voices(),
        }
    }

    /// Set simulated delay
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Default voice list
    fn default_voices() -> Vec<VoiceInfo> {
        vec![
            VoiceInfo::new(
                "af_sarah".to_string(),
                "Sarah (Female, American)".to_string(),
                "en-US".to_string(),
            )
            .with_gender("female".to_string())
            .with_description("Calm, soothing voice".to_string()),
            VoiceInfo::new(
                "af_sky".to_string(),
                "Sky (Female, American)".to_string(),
                "en-US".to_string(),
            )
            .with_gender("female".to_string())
            .with_description("Bright, clear voice".to_string()),
            VoiceInfo::new(
                "am_adam".to_string(),
                "Adam (Male, American)".to_string(),
                "en-US".to_string(),
            )
            .with_gender("male".to_string())
            .with_description("Deep, authoritative voice".to_string()),
        ]
    }

    /// Generate mock audio samples
    fn generate_samples(&self, text: &str, sample_rate: u32) -> Vec<f32> {
        // Generate silent audio with length proportional to text length
        // Assume ~150 words per minute, ~5 chars per word average
        let estimated_duration_secs = (text.len() as f32 / 5.0) / 150.0 * 60.0;
        let num_samples = (estimated_duration_secs * sample_rate as f32) as usize;

        // Generate simple sine wave for mock audio
        let frequency = 440.0; // A4 note
        (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.1
            })
            .collect()
    }
}

impl Default for MockTts {
    fn default() -> Self {
        Self::new()
    }
}

impl TtsRuntime for MockTts {
    fn load(&mut self, config: TtsRuntimeConfig) -> Result<()> {
        tracing::info!("🔊 Loading mock TTS: {}", config.model_path);

        // Simulate loading delay
        std::thread::sleep(std::time::Duration::from_millis(50));

        self.config = Some(config);
        self.loaded = true;

        tracing::info!("✅ Mock TTS loaded successfully");
        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn synthesize(&mut self, text: &str) -> Result<TtsSynthesis> {
        if !self.loaded {
            return Err(crate::Error::Model("Mock TTS not loaded".to_string()));
        }

        let start = Instant::now();

        // Simulate synthesis delay
        std::thread::sleep(std::time::Duration::from_millis(self.delay_ms));

        let sample_rate = 22050;
        let samples = self.generate_samples(text, sample_rate);

        let synthesis_time_ms = start.elapsed().as_millis() as u64;
        let audio_duration_ms = (samples.len() as f32 / sample_rate as f32 * 1000.0) as u64;
        let rtf = if audio_duration_ms > 0 {
            synthesis_time_ms as f32 / audio_duration_ms as f32
        } else {
            0.0
        };

        tracing::debug!(
            "Synthesized mock audio: {:.2}s in {}ms (RTF: {:.2})",
            audio_duration_ms as f32 / 1000.0,
            synthesis_time_ms,
            rtf
        );

        Ok(TtsSynthesis {
            samples,
            sample_rate,
            synthesis_time_ms,
            audio_duration_ms,
            rtf,
        })
    }

    fn list_voices(&self) -> Vec<VoiceInfo> {
        self.voices.clone()
    }

    fn set_voice(&mut self, voice_id: &str) -> Result<()> {
        if !self.loaded {
            return Err(crate::Error::Model("Mock TTS not loaded".to_string()));
        }

        // Check if voice exists
        if !self.voices.iter().any(|v| v.id == voice_id) {
            return Err(crate::Error::Model(format!(
                "Voice '{}' not found",
                voice_id
            )));
        }

        // Update config
        if let Some(config) = &mut self.config {
            config.voice_id = voice_id.to_string();
            tracing::info!("Voice changed to: {}", voice_id);
        }

        Ok(())
    }

    fn unload(&mut self) {
        tracing::info!("Unloading mock TTS");
        self.loaded = false;
        self.config = None;
    }

    fn name(&self) -> &str {
        "mock-tts"
    }

    fn info(&self) -> TtsInfo {
        TtsInfo {
            name: self.name().to_string(),
            loaded: self.loaded,
            backend: "mock".to_string(),
            available_voices: self.voices.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_tts_creation() {
        let tts = MockTts::new();
        assert!(!tts.is_loaded());
        assert_eq!(tts.name(), "mock-tts");
    }

    #[test]
    fn test_mock_tts_load() {
        let mut tts = MockTts::new();
        let config = TtsRuntimeConfig::default();

        tts.load(config).unwrap();
        assert!(tts.is_loaded());
    }

    #[test]
    fn test_mock_tts_synthesize() {
        let mut tts = MockTts::new().with_delay(10);
        let config = TtsRuntimeConfig::default();
        tts.load(config).unwrap();

        let synthesis = tts.synthesize("Hello, world!").unwrap();
        assert!(!synthesis.samples.is_empty());
        assert_eq!(synthesis.sample_rate, 22050);
        assert!(synthesis.synthesis_time_ms >= 10); // At least delay_ms
        assert!(synthesis.audio_duration_ms > 0);
    }

    #[test]
    fn test_mock_tts_list_voices() {
        let tts = MockTts::new();
        let voices = tts.list_voices();
        assert_eq!(voices.len(), 3);
        assert_eq!(voices[0].id, "af_sarah");
        assert_eq!(voices[1].id, "af_sky");
        assert_eq!(voices[2].id, "am_adam");
    }

    #[test]
    fn test_mock_tts_set_voice() {
        let mut tts = MockTts::new();
        let config = TtsRuntimeConfig::default();
        tts.load(config).unwrap();

        // Valid voice
        tts.set_voice("af_sky").unwrap();
        assert_eq!(tts.config.as_ref().unwrap().voice_id, "af_sky");

        // Invalid voice
        let result = tts.set_voice("invalid_voice");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_tts_unload() {
        let mut tts = MockTts::new();
        let config = TtsRuntimeConfig::default();
        tts.load(config).unwrap();

        assert!(tts.is_loaded());
        tts.unload();
        assert!(!tts.is_loaded());
    }

    #[test]
    fn test_mock_tts_synthesize_without_load() {
        let mut tts = MockTts::new();
        let result = tts.synthesize("Test");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_tts_audio_length_scales_with_text() {
        let mut tts = MockTts::new().with_delay(0);
        let config = TtsRuntimeConfig::default();
        tts.load(config).unwrap();

        let short = tts.synthesize("Hi").unwrap();
        let long = tts
            .synthesize("This is a much longer sentence with many more words.")
            .unwrap();

        assert!(long.samples.len() > short.samples.len());
        assert!(long.audio_duration_ms > short.audio_duration_ms);
    }
}
