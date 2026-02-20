//! Energy-based Voice Activity Detection
//!
//! Simple and fast VAD using RMS energy detection with adaptive threshold.

use super::detector::{VadDecision, VadDetector};
use crate::audio::buffer::{AudioChunk, Sample};
use std::collections::VecDeque;

/// Energy-based VAD configuration
#[derive(Debug, Clone)]
pub struct EnergyVadConfig {
    /// Energy threshold (0.0 - 1.0)
    pub threshold: f32,
    /// Minimum speech duration in chunks
    pub min_speech_chunks: usize,
    /// Minimum silence duration in chunks
    pub min_silence_chunks: usize,
    /// Use adaptive threshold
    pub adaptive: bool,
    /// Window size for adaptive threshold (in chunks)
    pub adaptive_window_size: usize,
}

impl Default for EnergyVadConfig {
    fn default() -> Self {
        Self {
            threshold: 0.02,
            min_speech_chunks: 2,
            min_silence_chunks: 3,
            adaptive: true,
            adaptive_window_size: 30,
        }
    }
}

/// Energy-based VAD detector
pub struct EnergyVad {
    config: EnergyVadConfig,
    speech_count: usize,
    silence_count: usize,
    current_state: VadDecision,
    energy_history: VecDeque<f32>,
    background_energy: f32,
}

impl EnergyVad {
    /// Create a new energy-based VAD
    pub fn new(config: EnergyVadConfig) -> Self {
        Self {
            config,
            speech_count: 0,
            silence_count: 0,
            current_state: VadDecision::Silence,
            energy_history: VecDeque::with_capacity(30),
            background_energy: 0.0,
        }
    }

    /// Calculate RMS energy of audio samples
    fn calculate_rms_energy(samples: &[Sample]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Update background energy estimate
    fn update_background_energy(&mut self, energy: f32) {
        if !self.config.adaptive {
            return;
        }

        // Add to history
        self.energy_history.push_back(energy);
        if self.energy_history.len() > self.config.adaptive_window_size {
            self.energy_history.pop_front();
        }

        // Calculate median energy as background estimate
        if !self.energy_history.is_empty() {
            let mut sorted: Vec<f32> = self.energy_history.iter().copied().collect();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            self.background_energy = sorted[sorted.len() / 2];
        }
    }

    /// Get effective threshold
    fn get_threshold(&self) -> f32 {
        if self.config.adaptive {
            // Adaptive threshold: background + offset
            self.background_energy + self.config.threshold
        } else {
            // Fixed threshold
            self.config.threshold
        }
    }
}

impl VadDetector for EnergyVad {
    fn detect(&mut self, chunk: &AudioChunk) -> crate::Result<VadDecision> {
        // Calculate energy for this chunk
        let energy = Self::calculate_rms_energy(&chunk.samples);

        // Update background energy estimate
        self.update_background_energy(energy);

        // Get current threshold
        let threshold = self.get_threshold();

        // Determine if this chunk has speech
        let has_speech = energy > threshold;

        // State machine with hysteresis
        let decision = match self.current_state {
            VadDecision::Silence => {
                if has_speech {
                    self.speech_count += 1;
                    self.silence_count = 0;

                    // Transition to speech after min_speech_chunks
                    if self.speech_count >= self.config.min_speech_chunks {
                        self.current_state = VadDecision::Speech;
                        VadDecision::Speech
                    } else {
                        VadDecision::Silence
                    }
                } else {
                    self.speech_count = 0;
                    VadDecision::Silence
                }
            }
            VadDecision::Speech => {
                if has_speech {
                    self.silence_count = 0;
                    self.speech_count += 1;
                    VadDecision::Speech
                } else {
                    self.silence_count += 1;
                    self.speech_count = 0;

                    // Transition to silence after min_silence_chunks
                    if self.silence_count >= self.config.min_silence_chunks {
                        self.current_state = VadDecision::Silence;
                        VadDecision::Silence
                    } else {
                        // Still in speech (post-roll)
                        VadDecision::Speech
                    }
                }
            }
        };

        Ok(decision)
    }

    fn name(&self) -> &str {
        "Energy-based VAD"
    }

    fn reset(&mut self) {
        self.speech_count = 0;
        self.silence_count = 0;
        self.current_state = VadDecision::Silence;
        self.energy_history.clear();
        self.background_energy = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rms_energy_calculation() {
        let samples = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let energy = EnergyVad::calculate_rms_energy(&samples);
        assert!(energy > 0.0);
        assert!(energy < 1.0);
    }

    #[test]
    fn test_silence_detection() {
        let mut vad = EnergyVad::new(EnergyVadConfig::default());
        let chunk = AudioChunk::new(vec![0.0; 1000], 16000);

        let decision = vad.detect(&chunk).unwrap();
        assert_eq!(decision, VadDecision::Silence);
    }
}
