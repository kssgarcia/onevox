//! VAD Processor
//!
//! Streaming VAD with pre-roll and post-roll buffering.

use super::detector::{VadDecision, VadDetector};
use crate::audio::buffer::AudioChunk;
use std::collections::VecDeque;
use tracing::{debug, info};

/// VAD processor configuration
#[derive(Debug, Clone)]
pub struct VadProcessorConfig {
    /// Pre-roll buffer duration in milliseconds
    /// This is how much audio before speech detection to include
    pub pre_roll_ms: u32,
    /// Post-roll buffer duration in milliseconds
    /// This is how much audio after speech ends to include
    pub post_roll_ms: u32,
}

impl Default for VadProcessorConfig {
    fn default() -> Self {
        Self {
            pre_roll_ms: 300,
            post_roll_ms: 500,
        }
    }
}

/// Speech segment with buffered audio
#[derive(Debug, Clone)]
pub struct SpeechSegment {
    /// Concatenated audio chunks forming the speech segment
    pub chunks: Vec<AudioChunk>,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp of first chunk
    pub start_time: std::time::Instant,
}

impl SpeechSegment {
    /// Create a new speech segment
    pub fn new(chunks: Vec<AudioChunk>) -> Self {
        let duration_ms = chunks.iter().map(|c| c.duration_ms()).sum();
        let start_time = chunks
            .first()
            .map(|c| c.timestamp)
            .unwrap_or_else(std::time::Instant::now);

        Self {
            chunks,
            duration_ms,
            start_time,
        }
    }

    /// Get all samples concatenated
    pub fn get_samples(&self) -> Vec<f32> {
        self.chunks
            .iter()
            .flat_map(|chunk| chunk.samples.iter().copied())
            .collect()
    }

    /// Get sample rate (from first chunk)
    pub fn sample_rate(&self) -> u32 {
        self.chunks.first().map(|c| c.sample_rate).unwrap_or(16000)
    }

    /// Check if segment is empty
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Get number of chunks
    pub fn len(&self) -> usize {
        self.chunks.len()
    }
}

/// VAD processor state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessorState {
    /// Waiting for speech to begin
    Idle,
    /// Currently processing speech
    InSpeech,
}

/// VAD processor for streaming audio
pub struct VadProcessor {
    config: VadProcessorConfig,
    detector: Box<dyn VadDetector>,
    state: ProcessorState,
    pre_roll_buffer: VecDeque<AudioChunk>,
    speech_buffer: Vec<AudioChunk>,
    max_pre_roll_chunks: usize,
}

impl VadProcessor {
    /// Create a new VAD processor
    pub fn new(config: VadProcessorConfig, detector: Box<dyn VadDetector>) -> Self {
        Self {
            config,
            detector,
            state: ProcessorState::Idle,
            pre_roll_buffer: VecDeque::new(),
            speech_buffer: Vec::new(),
            max_pre_roll_chunks: 10, // Will be updated based on chunk duration
        }
    }

    /// Process an audio chunk through VAD
    /// Returns Some(SpeechSegment) when a complete speech segment is detected
    pub fn process(&mut self, chunk: AudioChunk) -> crate::Result<Option<SpeechSegment>> {
        // Update max pre-roll chunks based on first chunk duration
        if self.max_pre_roll_chunks == 10 && !chunk.is_empty() {
            let chunk_duration_ms = chunk.duration_ms();
            if chunk_duration_ms > 0 {
                self.max_pre_roll_chunks =
                    ((self.config.pre_roll_ms as u64 / chunk_duration_ms) + 1) as usize;
                debug!(
                    "Set max pre-roll chunks to {} based on {}ms chunks",
                    self.max_pre_roll_chunks, chunk_duration_ms
                );
            }
        }

        // Run VAD detection on this chunk
        let decision = self.detector.detect(&chunk)?;

        match self.state {
            ProcessorState::Idle => {
                // Add to pre-roll buffer
                self.pre_roll_buffer.push_back(chunk.clone());

                // Maintain pre-roll buffer size
                while self.pre_roll_buffer.len() > self.max_pre_roll_chunks {
                    self.pre_roll_buffer.pop_front();
                }

                // Check if speech started
                if decision == VadDecision::Speech {
                    info!("Speech started");
                    self.state = ProcessorState::InSpeech;

                    // Move pre-roll buffer to speech buffer
                    self.speech_buffer.extend(self.pre_roll_buffer.drain(..));

                    // Add current chunk
                    self.speech_buffer.push(chunk);
                }

                Ok(None)
            }

            ProcessorState::InSpeech => {
                // Add chunk to speech buffer
                self.speech_buffer.push(chunk);

                // Check if speech ended
                if decision == VadDecision::Silence {
                    info!(
                        "Speech ended, {} chunks collected",
                        self.speech_buffer.len()
                    );

                    // Create speech segment
                    let segment = SpeechSegment::new(std::mem::take(&mut self.speech_buffer));

                    // Reset state
                    self.state = ProcessorState::Idle;
                    self.pre_roll_buffer.clear();

                    info!(
                        "Speech segment complete: {} chunks, {}ms duration",
                        segment.len(),
                        segment.duration_ms
                    );

                    Ok(Some(segment))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Reset processor state
    pub fn reset(&mut self) {
        self.state = ProcessorState::Idle;
        self.pre_roll_buffer.clear();
        self.speech_buffer.clear();
        self.detector.reset();
        info!("VAD processor reset");
    }

    /// Get current processor state
    pub fn is_in_speech(&self) -> bool {
        self.state == ProcessorState::InSpeech
    }

    /// Get detector name
    pub fn detector_name(&self) -> &str {
        self.detector.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vad::energy::{EnergyVad, EnergyVadConfig};

    fn create_silent_chunk(duration_ms: u64, sample_rate: u32) -> AudioChunk {
        let samples_count = (sample_rate as u64 * duration_ms / 1000) as usize;
        AudioChunk::new(vec![0.0; samples_count], sample_rate)
    }

    fn create_speech_chunk(duration_ms: u64, sample_rate: u32) -> AudioChunk {
        let samples_count = (sample_rate as u64 * duration_ms / 1000) as usize;
        // Generate samples with energy above threshold (0.05 RMS is well above 0.02 threshold)
        let samples: Vec<f32> = (0..samples_count)
            .map(|i| 0.2 * (i as f32 * 0.01).sin())
            .collect();
        AudioChunk::new(samples, sample_rate)
    }

    #[test]
    fn test_idle_state() {
        let vad_config = EnergyVadConfig::default();
        let detector = Box::new(EnergyVad::new(vad_config));
        let config = VadProcessorConfig::default();
        let mut processor = VadProcessor::new(config, detector);

        // Process silent chunk
        let chunk = create_silent_chunk(100, 16000);
        let result = processor.process(chunk).unwrap();

        assert!(result.is_none());
        assert!(!processor.is_in_speech());
    }

    #[test]
    fn test_speech_detection() {
        // Use non-adaptive VAD for predictable testing
        let vad_config = EnergyVadConfig {
            threshold: 0.02,
            min_speech_chunks: 2,
            min_silence_chunks: 3,
            adaptive: false, // Disable adaptive for test
            adaptive_window_size: 30,
        };
        let detector = Box::new(EnergyVad::new(vad_config));
        let config = VadProcessorConfig::default();
        let mut processor = VadProcessor::new(config, detector);

        // Process several speech chunks (need at least min_speech_chunks)
        for i in 0..5 {
            let chunk = create_speech_chunk(100, 16000);
            let result = processor.process(chunk).unwrap();
            // Should not return segment yet (still in speech)
            assert!(
                result.is_none(),
                "Unexpected speech segment at chunk {} during active speech",
                i
            );
        }

        assert!(
            processor.is_in_speech(),
            "Processor should be in speech state after processing speech chunks"
        );

        // Process silence to end speech (need at least min_silence_chunks)
        let mut segment_found = false;
        for _ in 0..5 {
            let chunk = create_silent_chunk(100, 16000);
            let result = processor.process(chunk).unwrap();

            if let Some(segment) = result {
                assert!(!segment.is_empty(), "Segment should not be empty");
                assert!(segment.duration_ms > 0, "Segment duration should be > 0");
                segment_found = true;
                break;
            }
        }

        assert!(
            segment_found,
            "Speech segment should have been detected after silence"
        );
    }
}
