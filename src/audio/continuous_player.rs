//! Continuous Streaming Audio Player
//!
//! Plays audio samples continuously from a queue without gaps between chunks.
//! Optimized for low-latency TTS streaming.

use crate::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, Stream, StreamConfig, SupportedStreamConfig};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Audio sample buffer that can be fed continuously
struct ContinuousBuffer {
    /// Samples waiting to be played
    samples: Vec<f32>,
    /// Current read position
    position: usize,
    /// Whether the stream is finished (no more chunks coming)
    finished: bool,
}

impl ContinuousBuffer {
    fn new() -> Self {
        Self {
            samples: Vec::new(),
            position: 0,
            finished: false,
        }
    }

    /// Add more samples to the buffer
    fn push_samples(&mut self, new_samples: &[f32]) {
        self.samples.extend_from_slice(new_samples);
    }

    /// Get the next sample, returns 0.0 (silence) if buffer is empty
    fn next_sample(&mut self) -> f32 {
        if self.position < self.samples.len() {
            let sample = self.samples[self.position];
            self.position += 1;
            sample
        } else {
            0.0 // Silence
        }
    }

    /// Check if we're done playing everything
    fn is_complete(&self) -> bool {
        self.finished && self.position >= self.samples.len()
    }

    /// Mark that no more chunks are coming
    fn mark_finished(&mut self) {
        self.finished = true;
    }

    /// Get remaining samples count
    fn remaining(&self) -> usize {
        self.samples.len().saturating_sub(self.position)
    }
}

/// Continuous streaming audio player
///
/// Plays audio chunks with no gaps, suitable for streaming TTS.
pub struct ContinuousAudioPlayer {
    device: Device,
    config: StreamConfig,
    supported_config: SupportedStreamConfig,
}

impl ContinuousAudioPlayer {
    /// Create a new continuous audio player
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| crate::Error::Audio("No output device available".to_string()))?;

        let supported_config = device
            .default_output_config()
            .map_err(|e| crate::Error::Audio(format!("Failed to get output config: {}", e)))?;

        let config = supported_config.config();

        info!(
            "🔊 Continuous audio player initialized: {}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );

        Ok(Self {
            device,
            config,
            supported_config,
        })
    }

    /// Play audio chunks continuously from a stream
    ///
    /// Returns immediately after starting playback. Audio will play as chunks arrive.
    /// When the receiver is closed, playback will finish the remaining buffer and stop.
    pub async fn play_stream(
        &self,
        mut receiver: mpsc::Receiver<Vec<f32>>,
        sample_rate: u32,
    ) -> Result<()> {
        info!("🎵 Starting continuous audio stream playback");

        // Resample if needed (do this for each chunk as it arrives)
        let needs_resampling = sample_rate != self.config.sample_rate.0;
        if needs_resampling {
            info!(
                "Audio will be resampled from {}Hz to {}Hz",
                sample_rate, self.config.sample_rate.0
            );
        }

        // Create shared buffer
        let buffer = Arc::new(Mutex::new(ContinuousBuffer::new()));
        let buffer_clone = Arc::clone(&buffer);

        // Start playback stream
        let stream = self.build_stream(buffer_clone)?;
        stream
            .play()
            .map_err(|e| crate::Error::Audio(format!("Failed to start stream: {}", e)))?;

        info!("✅ Audio stream started, feeding chunks...");

        // Feed chunks as they arrive
        let mut chunk_count = 0;
        while let Some(samples) = receiver.recv().await {
            chunk_count += 1;

            // Resample if needed
            let samples = if needs_resampling {
                self.resample(&samples, sample_rate, self.config.sample_rate.0)?
            } else {
                samples
            };

            debug!(
                "📥 Received audio chunk {} ({} samples, {:.2}s)",
                chunk_count,
                samples.len(),
                samples.len() as f32 / self.config.sample_rate.0 as f32
            );

            // Add to buffer
            {
                let mut buf = buffer.lock().unwrap();
                buf.push_samples(&samples);
                debug!(
                    "📊 Buffer: {} samples remaining ({:.2}s)",
                    buf.remaining(),
                    buf.remaining() as f32 / self.config.sample_rate.0 as f32
                );
            }
        }

        info!(
            "✅ All chunks received ({}), waiting for playback to finish...",
            chunk_count
        );

        // Mark buffer as finished
        {
            let mut buf = buffer.lock().unwrap();
            buf.mark_finished();
        }

        // Wait for buffer to drain
        loop {
            let complete = {
                let buf = buffer.lock().unwrap();
                buf.is_complete()
            };

            if complete {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // Explicitly drop stream to stop playback
        drop(stream);

        info!("✅ Continuous audio playback complete");
        Ok(())
    }

    /// Build the output stream
    fn build_stream(&self, buffer: Arc<Mutex<ContinuousBuffer>>) -> Result<Stream> {
        let channels = self.config.channels as usize;

        match self.supported_config.sample_format() {
            cpal::SampleFormat::F32 => self.build_stream_typed::<f32>(buffer, channels),
            cpal::SampleFormat::I16 => self.build_stream_typed::<i16>(buffer, channels),
            cpal::SampleFormat::U16 => self.build_stream_typed::<u16>(buffer, channels),
            _ => Err(crate::Error::Audio(format!(
                "Unsupported sample format: {:?}",
                self.supported_config.sample_format()
            ))),
        }
    }

    /// Build typed stream
    fn build_stream_typed<T>(
        &self,
        buffer: Arc<Mutex<ContinuousBuffer>>,
        channels: usize,
    ) -> Result<Stream>
    where
        T: Sample + cpal::SizedSample + cpal::FromSample<f32>,
    {
        let stream = self
            .device
            .build_output_stream(
                &self.config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    let mut buf = buffer.lock().unwrap();

                    for frame in data.chunks_mut(channels) {
                        let sample = buf.next_sample();

                        // Write to all channels (mono to stereo/multi-channel)
                        for channel_sample in frame.iter_mut() {
                            *channel_sample = T::from_sample(sample);
                        }
                    }
                },
                |err| {
                    warn!("⚠️  Audio stream error: {}", err);
                },
                None,
            )
            .map_err(|e| crate::Error::Audio(format!("Failed to build stream: {}", e)))?;

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
}

impl Default for ContinuousAudioPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create continuous audio player")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continuous_buffer() {
        let mut buf = ContinuousBuffer::new();

        buf.push_samples(&[0.1, 0.2, 0.3]);
        assert_eq!(buf.remaining(), 3);

        assert_eq!(buf.next_sample(), 0.1);
        assert_eq!(buf.next_sample(), 0.2);
        assert_eq!(buf.remaining(), 1);

        buf.push_samples(&[0.4, 0.5]);
        assert_eq!(buf.remaining(), 3);

        assert_eq!(buf.next_sample(), 0.3);
        assert_eq!(buf.next_sample(), 0.4);
        assert_eq!(buf.next_sample(), 0.5);

        assert_eq!(buf.remaining(), 0);
        assert_eq!(buf.next_sample(), 0.0); // Silence when empty
    }

    #[test]
    fn test_buffer_completion() {
        let mut buf = ContinuousBuffer::new();

        buf.push_samples(&[0.1, 0.2]);
        assert!(!buf.is_complete());

        buf.mark_finished();
        assert!(!buf.is_complete()); // Still have samples

        buf.next_sample();
        buf.next_sample();
        assert!(buf.is_complete()); // Now done
    }
}
