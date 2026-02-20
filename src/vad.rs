//! Voice Activity Detection
//!
//! Intelligent silence filtering and speech detection.

pub mod detector;
pub mod energy;
pub mod processor;

// Re-export commonly used types
pub use detector::{VadDecision, VadDetector};
pub use energy::{EnergyVad, EnergyVadConfig};
pub use processor::{SpeechSegment, VadProcessor, VadProcessorConfig};
