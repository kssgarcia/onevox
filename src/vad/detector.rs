//! VAD Detection Trait
//!
//! Abstract interface for Voice Activity Detection backends.

use crate::audio::buffer::AudioChunk;

/// Voice activity detection result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadDecision {
    /// Speech detected
    Speech,
    /// Silence/noise detected
    Silence,
}

/// Voice Activity Detection trait
pub trait VadDetector: Send + Sync {
    /// Process an audio chunk and detect voice activity
    fn detect(&mut self, chunk: &AudioChunk) -> crate::Result<VadDecision>;

    /// Get the detector name
    fn name(&self) -> &str;

    /// Reset internal state
    fn reset(&mut self);
}
