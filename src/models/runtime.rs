//! Model Runtime Trait
//!
//! Abstract interface for speech-to-text model backends.

use crate::audio::buffer::AudioChunk;
use crate::vad::SpeechSegment;

/// Transcription result
#[derive(Debug, Clone)]
pub struct Transcription {
    /// Transcribed text
    pub text: String,
    /// Language detected (ISO 639-1 code, e.g., "en")
    pub language: Option<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: Option<f32>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Number of tokens generated
    pub tokens: Option<usize>,
}

impl Transcription {
    /// Create a new transcription result
    pub fn new(text: String) -> Self {
        Self {
            text,
            language: None,
            confidence: None,
            processing_time_ms: 0,
            tokens: None,
        }
    }

    /// Check if transcription is empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// Model runtime configuration
#[derive(Debug, Clone)]
pub struct ModelConfig {
    /// Path to model file
    pub model_path: String,
    /// Language (e.g., "en", "auto" for auto-detect)
    pub language: String,
    /// Use GPU acceleration if available
    pub use_gpu: bool,
    /// Number of threads for CPU inference
    pub n_threads: u32,
    /// Beam size for decoding (higher = better quality, slower)
    pub beam_size: u32,
    /// Enable translate mode (translate to English)
    pub translate: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_path: "models/ggml-base.en.bin".to_string(),
            language: "en".to_string(),
            use_gpu: true,
            n_threads: default_thread_count(),
            beam_size: 5,
            translate: false,
        }
    }
}

fn default_thread_count() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1)
        .clamp(1, 8)
}

/// Model runtime trait
pub trait ModelRuntime: Send + Sync {
    /// Load the model
    fn load(&mut self, config: ModelConfig) -> crate::Result<()>;

    /// Check if model is loaded
    fn is_loaded(&self) -> bool;

    /// Transcribe raw audio samples
    /// Samples should be mono, f32, 16kHz
    fn transcribe(&mut self, samples: &[f32], sample_rate: u32) -> crate::Result<Transcription>;

    /// Transcribe an audio chunk
    fn transcribe_chunk(&mut self, chunk: &AudioChunk) -> crate::Result<Transcription> {
        self.transcribe(&chunk.samples, chunk.sample_rate)
    }

    /// Transcribe a speech segment
    fn transcribe_segment(&mut self, segment: &SpeechSegment) -> crate::Result<Transcription> {
        let samples = segment.get_samples();
        let sample_rate = segment.sample_rate();
        self.transcribe(&samples, sample_rate)
    }

    /// Unload the model and free resources
    fn unload(&mut self);

    /// Get model name/identifier
    fn name(&self) -> &str;

    /// Get model info (size, version, etc.)
    fn info(&self) -> ModelInfo;
}

/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model name
    pub name: String,
    /// Model size in bytes
    pub size_bytes: u64,
    /// Model type (e.g., "whisper-tiny", "whisper-base")
    pub model_type: String,
    /// Backend (e.g., "whisper.cpp", "faster-whisper")
    pub backend: String,
    /// GPU enabled
    pub gpu_enabled: bool,
}

impl Default for ModelInfo {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            size_bytes: 0,
            model_type: "Unknown".to_string(),
            backend: "Unknown".to_string(),
            gpu_enabled: false,
        }
    }
}
