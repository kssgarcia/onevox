//! Audio capture and processing module
//!
//! Provides real-time microphone capture, buffering, and streaming audio processing.

pub mod buffer;
pub mod capture;
pub mod devices;

// Re-export commonly used types
pub use buffer::{AudioBuffer, AudioChunk, AudioConsumer, AudioProducer};
pub use capture::{AudioCapture, CaptureConfig};
pub use devices::{AudioDeviceInfo, AudioDeviceManager};

/// Audio engine - main interface for audio system
pub struct AudioEngine {
    device_manager: AudioDeviceManager,
    capture: Option<AudioCapture>,
}

impl AudioEngine {
    /// Create a new audio engine
    pub fn new() -> Self {
        Self {
            device_manager: AudioDeviceManager::new(),
            capture: None,
        }
    }

    /// List available input devices
    pub fn list_devices(&self) -> crate::Result<Vec<AudioDeviceInfo>> {
        self.device_manager.list_input_devices()
    }

    /// Start audio capture with config
    pub fn start_capture(
        &mut self,
        config: CaptureConfig,
    ) -> crate::Result<tokio::sync::mpsc::Receiver<AudioChunk>> {
        // Ensure any existing capture is fully stopped before starting a new one
        if let Some(mut existing_capture) = self.capture.take() {
            existing_capture.stop()?;
        }

        let mut capture = AudioCapture::new(config);
        let rx = capture.start()?;
        self.capture = Some(capture);
        Ok(rx)
    }

    /// Stop audio capture
    pub fn stop_capture(&mut self) -> crate::Result<()> {
        if let Some(mut capture) = self.capture.take() {
            capture.stop()?;
        }
        Ok(())
    }

    /// Check if capture is running
    pub fn is_capturing(&self) -> bool {
        self.capture
            .as_ref()
            .map(|c| c.is_running())
            .unwrap_or(false)
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        // Ensure audio is properly stopped when engine is dropped
        let _ = self.stop_capture();
    }
}
