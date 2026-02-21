//! Audio Capture
//!
//! Real-time microphone input using cpal.

use super::buffer::AudioChunk;
use super::devices::AudioDeviceManager;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Sample as CpalSample, SampleFormat, Stream, StreamConfig};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use tracing::{error, info, trace, warn};

/// Parameters for building an audio stream
struct StreamParams {
    chunk_tx: mpsc::Sender<AudioChunk>,
    chunk_size: usize,
    target_sample_rate: u32,
    device_sample_rate: u32,
    is_running: Arc<AtomicBool>,
    channel_open: Arc<AtomicBool>,
}

/// Audio resampler for converting between sample rates
struct AudioResampler {
    resampler: SincFixedIn<f32>,
    input_buffer: Vec<Vec<f32>>,
    output_buffer: Vec<Vec<f32>>,
}

impl AudioResampler {
    /// Create a new resampler
    fn new(from_rate: u32, to_rate: u32, chunk_size: usize) -> crate::Result<Self> {
        let resample_ratio = to_rate as f64 / from_rate as f64;
        
        // Configure high-quality sinc resampler
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        let resampler = SincFixedIn::<f32>::new(
            resample_ratio,
            2.0, // max_resample_ratio_relative
            params,
            chunk_size,
            1, // mono channel
        )
        .map_err(|e| crate::Error::Audio(format!("Failed to create resampler: {}", e)))?;

        let input_buffer = resampler.input_buffer_allocate(true);
        let output_buffer = resampler.output_buffer_allocate(true);

        info!(
            "Created resampler: {}Hz -> {}Hz (ratio: {:.4})",
            from_rate, to_rate, resample_ratio
        );

        Ok(Self {
            resampler,
            input_buffer,
            output_buffer,
        })
    }

    /// Resample audio samples
    fn resample(&mut self, input: &[f32]) -> crate::Result<Vec<f32>> {
        // Copy input to resampler buffer
        self.input_buffer[0][..input.len()].copy_from_slice(input);

        // Perform resampling
        let (_, out_len) = self
            .resampler
            .process_into_buffer(&self.input_buffer, &mut self.output_buffer, None)
            .map_err(|e| crate::Error::Audio(format!("Resampling failed: {}", e)))?;

        // Extract output samples
        Ok(self.output_buffer[0][..out_len].to_vec())
    }
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

impl CaptureConfig {
    /// Validate capture settings before starting audio stream.
    fn validate(&self) -> crate::Result<()> {
        const VALID_SAMPLE_RATES: &[u32] = &[8000, 16000, 22050, 44100, 48000];

        if !VALID_SAMPLE_RATES.contains(&self.sample_rate) {
            return Err(crate::Error::Config(format!(
                "Unsupported sample rate {}. Supported values: {:?}",
                self.sample_rate, VALID_SAMPLE_RATES
            )));
        }

        if !(10..=1000).contains(&self.chunk_duration_ms) {
            return Err(crate::Error::Config(
                "chunk_duration_ms must be between 10 and 1000".to_string(),
            ));
        }

        if !(1..=60).contains(&self.buffer_capacity_secs) {
            return Err(crate::Error::Config(
                "buffer_capacity_secs must be between 1 and 60".to_string(),
            ));
        }

        Ok(())
    }
}

/// Audio capture engine
pub struct AudioCapture {
    config: CaptureConfig,
    device_manager: AudioDeviceManager,
    stream: Option<Stream>,
    is_running: Arc<AtomicBool>,
    chunk_tx: Option<mpsc::Sender<AudioChunk>>,
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
    pub fn start(&mut self) -> crate::Result<mpsc::Receiver<AudioChunk>> {
        if self.is_running.load(Ordering::SeqCst) {
            return Err(crate::Error::Audio("Capture already running".to_string()));
        }

        self.config.validate()?;

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

        // Create bounded channel for audio chunks
        // Buffer size = (sample_rate * buffer_capacity_secs) / chunk_size
        // This ensures we don't buffer more than buffer_capacity_secs of audio
        let chunk_size = (self.config.sample_rate * self.config.chunk_duration_ms / 1000) as usize;
        let buffer_capacity = ((self.config.sample_rate * self.config.buffer_capacity_secs)
            / chunk_size as u32) as usize;
        let (chunk_tx, chunk_rx) = mpsc::channel(buffer_capacity.max(10)); // At least 10 chunks

        info!(
            "Audio buffer capacity: {} chunks (~{}s of audio), chunk_size: {} samples",
            buffer_capacity, self.config.buffer_capacity_secs, chunk_size
        );

        self.chunk_tx = Some(chunk_tx.clone());

        // Create stream config
        let stream_config = StreamConfig {
            channels: 1, // We want mono
            sample_rate: cpal::SampleRate(device_sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

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
                )));
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
        
        // Create resampler if needed
        let mut resampler = if needs_resampling {
            match AudioResampler::new(device_sample_rate, target_sample_rate, chunk_size) {
                Ok(r) => Some(r),
                Err(e) => {
                    warn!("Failed to create resampler, audio quality may be degraded: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let mut dropped_chunks = 0u64;
        let mut last_warning = std::time::Instant::now();

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
                            
                            // Resample if necessary
                            let chunk = if let Some(ref mut resampler) = resampler {
                                match resampler.resample(&samples) {
                                    Ok(resampled) => AudioChunk::new(resampled, target_sample_rate),
                                    Err(e) => {
                                        // Fallback to original samples on error
                                        warn!("Resampling error, using original samples: {}", e);
                                        AudioChunk::new(samples, device_sample_rate)
                                    }
                                }
                            } else {
                                AudioChunk::new(samples, target_sample_rate)
                            };

                            // Send chunk (with backpressure handling)
                            // Use try_send to avoid blocking the audio thread
                            match chunk_tx.try_send(chunk) {
                                Ok(_) => {
                                    // Reset dropped counter on success
                                    if dropped_chunks > 0 {
                                        dropped_chunks = 0;
                                    }
                                }
                                Err(mpsc::error::TrySendError::Full(_)) => {
                                    // Buffer full - drop this chunk to avoid blocking audio callback
                                    dropped_chunks += 1;
                                    
                                    // Warn periodically about dropped chunks
                                    if last_warning.elapsed().as_secs() >= 5 {
                                        warn!(
                                            "Audio buffer full, dropped {} chunks (transcription too slow)",
                                            dropped_chunks
                                        );
                                        last_warning = std::time::Instant::now();
                                    }
                                }
                                Err(mpsc::error::TrySendError::Closed(_)) => {
                                    if channel_open.swap(false, Ordering::Relaxed) {
                                        trace!("Audio receiver closed, stopping chunk delivery");
                                    }
                                }
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
