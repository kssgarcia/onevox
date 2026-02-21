//! Audio Capture
//!
//! Real-time microphone input using cpal.

use super::buffer::AudioChunk;
use super::devices::AudioDeviceManager;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Sample as CpalSample, SampleFormat, Stream, StreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Parameters for building an audio stream
struct StreamParams {
    chunk_tx: mpsc::UnboundedSender<AudioChunk>,
    chunk_size: usize,
    target_sample_rate: u32,
    device_sample_rate: u32,
    is_running: Arc<AtomicBool>,
    channel_open: Arc<AtomicBool>,
}

/// Audio capture configuration
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Device name (or "default")
    pub device_name: String,
    /// Target sample rate
    pub sample_rate: u32,
    /// Chunk duration in milliseconds
    pub chunk_duration_ms: u32,
    /// Buffer capacity in seconds
    pub buffer_capacity_secs: u32,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            device_name: "default".to_string(),
            sample_rate: 16000,
            chunk_duration_ms: 200,
            buffer_capacity_secs: 2,
        }
    }
}

/// Audio capture engine
pub struct AudioCapture {
    config: CaptureConfig,
    device_manager: AudioDeviceManager,
    stream: Option<Stream>,
    is_running: Arc<AtomicBool>,
    chunk_tx: Option<mpsc::UnboundedSender<AudioChunk>>,
}

impl AudioCapture {
    /// Create a new audio capture instance
    pub fn new(config: CaptureConfig) -> Self {
        Self {
            config,
            device_manager: AudioDeviceManager::new(),
            stream: None,
            is_running: Arc::new(AtomicBool::new(false)),
            chunk_tx: None,
        }
    }

    /// Start capturing audio
    pub fn start(&mut self) -> crate::Result<mpsc::UnboundedReceiver<AudioChunk>> {
        if self.is_running.load(Ordering::SeqCst) {
            return Err(crate::Error::Audio("Capture already running".to_string()));
        }

        info!("Starting audio capture");

        // Get device
        let device = if self.config.device_name == "default" {
            self.device_manager.default_input_device()?
        } else {
            self.device_manager
                .get_device_by_name(&self.config.device_name)?
        };

        let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        info!("Using audio device: {}", device_name);

        // Get device config
        let supported_config = self.device_manager.get_device_config(&device)?;
        let sample_format = supported_config.sample_format();
        let device_sample_rate = supported_config.sample_rate().0;

        info!(
            "Device config: {}Hz, format: {:?}",
            device_sample_rate, sample_format
        );

        // Create stream config
        let stream_config = StreamConfig {
            channels: 1, // We want mono
            sample_rate: cpal::SampleRate(device_sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        // Create channel for chunks
        let (chunk_tx, chunk_rx) = mpsc::unbounded_channel();
        self.chunk_tx = Some(chunk_tx.clone());

        // Calculate chunk size
        let chunk_size = (self.config.sample_rate * self.config.chunk_duration_ms / 1000) as usize;

        let target_sample_rate = self.config.sample_rate;
        let is_running = Arc::clone(&self.is_running);
        let channel_open = Arc::new(AtomicBool::new(true));

        // Build stream config
        let stream_params = StreamParams {
            chunk_tx,
            chunk_size,
            target_sample_rate,
            device_sample_rate,
            is_running: Arc::clone(&is_running),
            channel_open,
        };

        // Build the input stream
        let stream = match sample_format {
            SampleFormat::F32 => {
                self.build_input_stream::<f32>(&device, &stream_config, stream_params)?
            }
            SampleFormat::I16 => {
                self.build_input_stream::<i16>(&device, &stream_config, stream_params)?
            }
            SampleFormat::U16 => {
                self.build_input_stream::<u16>(&device, &stream_config, stream_params)?
            }
            _ => {
                return Err(crate::Error::Audio(format!(
                    "Unsupported sample format: {:?}",
                    sample_format
                )))
            }
        };

        // Start the stream
        stream
            .play()
            .map_err(|e| crate::Error::Audio(format!("Failed to start stream: {}", e)))?;

        self.stream = Some(stream);
        is_running.store(true, Ordering::SeqCst);

        info!("Audio capture started");
        Ok(chunk_rx)
    }

    /// Build input stream for a specific sample type
    fn build_input_stream<T>(
        &self,
        device: &Device,
        config: &StreamConfig,
        params: StreamParams,
    ) -> crate::Result<Stream>
    where
        T: CpalSample + cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        let StreamParams {
            chunk_tx,
            chunk_size,
            target_sample_rate,
            device_sample_rate,
            is_running,
            channel_open,
        } = params;

        let mut local_accumulator = Vec::with_capacity(chunk_size);
        let needs_resampling = device_sample_rate != target_sample_rate;

        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    if !is_running.load(Ordering::Relaxed) || !channel_open.load(Ordering::Relaxed)
                    {
                        return;
                    }

                    // Convert samples to f32
                    for &sample in data.iter() {
                        let f32_sample: f32 = cpal::Sample::from_sample(sample);
                        local_accumulator.push(f32_sample);

                        // When we have enough samples for a chunk
                        if local_accumulator.len() >= chunk_size {
                            let samples = std::mem::replace(
                                &mut local_accumulator,
                                Vec::with_capacity(chunk_size),
                            );
                            let chunk = if needs_resampling {
                                // TODO: Implement resampling
                                // For now, just use the samples as-is
                                AudioChunk::new(samples, device_sample_rate)
                            } else {
                                AudioChunk::new(samples, target_sample_rate)
                            };

                            // Send chunk
                            if let Err(e) = chunk_tx.send(chunk) {
                                if channel_open.swap(false, Ordering::Relaxed) {
                                    debug!("Audio receiver closed, stopping chunk delivery: {}", e);
                                }
                                return;
                            }
                        }
                    }
                },
                move |err| {
                    error!("Audio stream error: {}", err);
                },
                None,
            )
            .map_err(|e| crate::Error::Audio(format!("Failed to build stream: {}", e)))?;

        Ok(stream)
    }

    /// Stop capturing audio
    pub fn stop(&mut self) -> crate::Result<()> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }

        info!("Stopping audio capture");
        self.is_running.store(false, Ordering::SeqCst);

        if let Some(stream) = self.stream.take() {
            // Explicitly pause the stream before dropping to ensure proper cleanup
            if let Err(e) = stream.pause() {
                error!("Failed to pause audio stream: {}", e);
            }
            drop(stream);
        }

        self.chunk_tx = None;

        info!("Audio capture stopped");
        Ok(())
    }

    /// Check if capture is running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
