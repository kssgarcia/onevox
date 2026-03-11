//! Audio Playback Module
//!
//! Provides audio output capabilities for TTS synthesis playback.

use crate::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, Stream, StreamConfig, SupportedStreamConfig};
use std::sync::{Arc, Mutex};

/// Audio player for TTS output
pub struct AudioPlayer {
    device: Device,
    config: StreamConfig,
    supported_config: SupportedStreamConfig,
}

impl AudioPlayer {
    /// Create a new audio player with default output device
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| crate::Error::Audio("No output device available".to_string()))?;

        let supported_config = device
            .default_output_config()
            .map_err(|e| crate::Error::Audio(format!("Failed to get output config: {}", e)))?;

        let config = supported_config.config();

        tracing::info!(
            "🔊 Audio player initialized with device: {}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );

        Ok(Self {
            device,
            config,
            supported_config,
        })
    }

    /// Create an audio player with a specific device
    pub fn with_device(device_name: &str) -> Result<Self> {
        let host = cpal::default_host();

        // Find the device by name
        let device = host
            .output_devices()
            .map_err(|e| crate::Error::Audio(format!("Failed to enumerate devices: {}", e)))?
            .find(|d| d.name().map(|n| n.contains(device_name)).unwrap_or(false))
            .ok_or_else(|| {
                crate::Error::Audio(format!("Output device '{}' not found", device_name))
            })?;

        let supported_config = device
            .default_output_config()
            .map_err(|e| crate::Error::Audio(format!("Failed to get output config: {}", e)))?;

        let config = supported_config.config();

        tracing::info!(
            "🔊 Audio player initialized with device: {}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );

        Ok(Self {
            device,
            config,
            supported_config,
        })
    }

    /// Play audio samples
    ///
    /// # Arguments
    /// * `samples` - Audio samples (f32, mono)
    /// * `sample_rate` - Sample rate of the audio
    ///
    /// # Returns
    /// Ok(()) when playback completes successfully
    pub async fn play(&self, samples: &[f32], sample_rate: u32) -> Result<()> {
        if samples.is_empty() {
            tracing::warn!("Attempted to play empty audio buffer");
            return Ok(());
        }

        let duration_secs = samples.len() as f32 / sample_rate as f32;
        tracing::info!(
            "▶️  Playing audio: {:.2}s ({} samples @ {}Hz)",
            duration_secs,
            samples.len(),
            sample_rate
        );

        // Resample if needed
        let resampled = if sample_rate != self.config.sample_rate.0 {
            tracing::debug!(
                "Resampling from {}Hz to {}Hz",
                sample_rate,
                self.config.sample_rate.0
            );
            self.resample(samples, sample_rate, self.config.sample_rate.0)?
        } else {
            samples.to_vec()
        };

        // Create shared buffer for playback
        let playback_buffer = Arc::new(Mutex::new(PlaybackBuffer {
            samples: resampled,
            position: 0,
        }));

        let buffer_clone = Arc::clone(&playback_buffer);

        // Build output stream based on format
        let stream = match self.supported_config.sample_format() {
            cpal::SampleFormat::F32 => self.build_stream::<f32>(buffer_clone)?,
            cpal::SampleFormat::I16 => self.build_stream::<i16>(buffer_clone)?,
            cpal::SampleFormat::U16 => self.build_stream::<u16>(buffer_clone)?,
            _ => {
                return Err(crate::Error::Audio(format!(
                    "Unsupported sample format: {:?}",
                    self.supported_config.sample_format()
                )));
            }
        };

        // Start playback
        stream
            .play()
            .map_err(|e| crate::Error::Audio(format!("Failed to start playback: {}", e)))?;

        // Wait for playback to complete
        let total_samples = {
            let buffer = playback_buffer.lock().unwrap();
            buffer.samples.len()
        };

        let playback_duration_ms =
            (total_samples as f32 / self.config.sample_rate.0 as f32 * 1000.0) as u64;

        // Add a small buffer to ensure playback completes
        let wait_duration_ms = playback_duration_ms + 100;

        tokio::time::sleep(tokio::time::Duration::from_millis(wait_duration_ms)).await;

        // Explicitly drop the stream to stop playback
        drop(stream);

        tracing::info!("✅ Audio playback completed");
        Ok(())
    }

    /// Build an output stream for a specific sample format
    fn build_stream<T>(&self, buffer: Arc<Mutex<PlaybackBuffer>>) -> Result<Stream>
    where
        T: Sample + cpal::SizedSample + cpal::FromSample<f32>,
    {
        let channels = self.config.channels as usize;

        let stream = self
            .device
            .build_output_stream(
                &self.config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    let mut buffer = buffer.lock().unwrap();

                    for frame in data.chunks_mut(channels) {
                        // Get next sample or silence
                        let sample = if buffer.position < buffer.samples.len() {
                            let s = buffer.samples[buffer.position];
                            buffer.position += 1;
                            s
                        } else {
                            0.0 // Silence after buffer ends
                        };

                        // Write to all channels (mono to stereo/multi-channel)
                        for channel_sample in frame.iter_mut() {
                            *channel_sample = T::from_sample(sample);
                        }
                    }
                },
                |err| {
                    tracing::error!("Audio playback error: {}", err);
                },
                None, // No timeout
            )
            .map_err(|e| crate::Error::Audio(format!("Failed to build output stream: {}", e)))?;

        Ok(stream)
    }

    /// Resample audio to target sample rate
    fn resample(&self, samples: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>> {
        use rubato::{
            Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType,
            WindowFunction,
        };

        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        let mut resampler = SincFixedIn::<f32>::new(
            to_rate as f64 / from_rate as f64,
            2.0,
            params,
            samples.len(),
            1, // mono
        )
        .map_err(|e| crate::Error::Audio(format!("Failed to create resampler: {}", e)))?;

        let input = vec![samples.to_vec()];
        let output = resampler
            .process(&input, None)
            .map_err(|e| crate::Error::Audio(format!("Resampling failed: {}", e)))?;

        Ok(output[0].clone())
    }

    /// Get the output device name
    pub fn device_name(&self) -> String {
        self.device.name().unwrap_or_else(|_| "Unknown".to_string())
    }

    /// Get the output sample rate
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }

    /// Get the number of output channels
    pub fn channels(&self) -> u16 {
        self.config.channels
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create default audio player")
    }
}

/// Internal playback buffer
struct PlaybackBuffer {
    samples: Vec<f32>,
    position: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_player_creation() {
        // This test may fail in CI without audio devices
        match AudioPlayer::new() {
            Ok(player) => {
                assert!(!player.device_name().is_empty());
                assert!(player.sample_rate() > 0);
                assert!(player.channels() > 0);
            }
            Err(_) => {
                // Skip test if no audio device available
                println!("Skipping test: No audio device available");
            }
        }
    }

    #[test]
    fn test_playback_buffer() {
        let buffer = PlaybackBuffer {
            samples: vec![0.1, 0.2, 0.3],
            position: 0,
        };

        assert_eq!(buffer.samples.len(), 3);
        assert_eq!(buffer.position, 0);
    }

    #[tokio::test]
    async fn test_play_empty_samples() {
        match AudioPlayer::new() {
            Ok(player) => {
                let result = player.play(&[], 22050).await;
                assert!(result.is_ok());
            }
            Err(_) => {
                println!("Skipping test: No audio device available");
            }
        }
    }

    #[tokio::test]
    async fn test_play_short_audio() {
        match AudioPlayer::new() {
            Ok(player) => {
                // Generate 0.1 second of 440Hz sine wave
                let sample_rate = 22050;
                let duration_secs = 0.1;
                let frequency = 440.0;
                let num_samples = (sample_rate as f32 * duration_secs) as usize;

                let samples: Vec<f32> = (0..num_samples)
                    .map(|i| {
                        let t = i as f32 / sample_rate as f32;
                        (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.1
                    })
                    .collect();

                let result = player.play(&samples, sample_rate).await;
                assert!(result.is_ok());
            }
            Err(_) => {
                println!("Skipping test: No audio device available");
            }
        }
    }
}
