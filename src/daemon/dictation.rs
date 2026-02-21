//! Dictation Engine
//!
//! Orchestrates the full dictation pipeline:
//! Hotkey → Audio Capture → VAD → Model → Text Injection

use crate::audio::{AudioEngine, CaptureConfig};
use crate::config::Config;
use crate::indicator::RecordingIndicator;
use crate::models::{ModelConfig, ModelRuntime, Transcription, WhisperCppCli};
use crate::platform::{HotkeyConfig as PlatformHotkeyConfig, HotkeyEvent, HotkeyManager, InjectorConfig, TextInjector};
use crate::vad::{EnergyVad, VadDetector, VadProcessor};
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Dictation engine state
pub struct DictationEngine {
    /// Configuration
    config: Config,

    /// Hotkey manager
    hotkey_manager: HotkeyManager,

    /// Text injector
    text_injector: TextInjector,

    /// Audio engine
    audio_engine: AudioEngine,

    /// Model runtime
    model: Arc<Mutex<Box<dyn ModelRuntime>>>,

    /// Is currently dictating
    is_dictating: Arc<AtomicBool>,

    /// Shutdown signal
    shutdown_signal: Arc<AtomicBool>,

    /// Floating UI indicator
    indicator: Arc<RecordingIndicator>,
}

impl DictationEngine {
    /// Create a new dictation engine
    pub fn new(config: Config) -> Result<Self> {
        info!("Initializing dictation engine");

        // Create hotkey manager
        let hotkey_manager = HotkeyManager::new()?;

        // Create text injector
        let injector_config = InjectorConfig {
            key_delay_ms: config.injection.paste_delay_ms as u64,
            initial_delay_ms: 50,
        };
        let text_injector = TextInjector::new(injector_config);

        // Create audio engine
        let audio_engine = AudioEngine::new();

        // Create Whisper CLI model
        let mut model: Box<dyn ModelRuntime> = Box::new(WhisperCppCli::new(None));
        let model_config = ModelConfig {
            model_path: config.model.model_path.clone(),
            language: config.model.language.clone(),
            use_gpu: config.model.device == "gpu",
            ..Default::default()
        };
        model.load(model_config)?;

        info!("✅ Dictation engine initialized");

        Ok(Self {
            indicator: Arc::new(RecordingIndicator::new(config.ui.recording_overlay)),
            config,
            hotkey_manager,
            text_injector,
            audio_engine,
            model: Arc::new(Mutex::new(model)),
            is_dictating: Arc::new(AtomicBool::new(false)),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Start the dictation engine
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting dictation engine");

        // List available audio devices for debugging
        self.list_audio_devices();

        // Register global hotkey
        let hotkey_str = self.config.hotkey.trigger.clone();
        let hotkey_config = PlatformHotkeyConfig::from_string(&hotkey_str)
            .context("Failed to parse hotkey configuration")?;

        let event_rx = self
            .hotkey_manager
            .register(hotkey_config)
            .context("Failed to register hotkey")?;

        info!("✅ Hotkey registered: {}", hotkey_str);

        // Take ownership of hotkey_manager to start the listener
        // (it consumes self and moves into the listener thread)
        let hotkey_manager = std::mem::replace(
            &mut self.hotkey_manager,
            HotkeyManager::new().unwrap(), // Temporary placeholder
        );
        
        hotkey_manager
            .start_listener()
            .context("Failed to start hotkey listener")?;

        info!("✅ Hotkey listener started");

        // Start hotkey event loop
        self.run_event_loop(event_rx).await?;

        Ok(())
    }

    /// Run the hotkey event loop
    async fn run_event_loop(&mut self, mut event_rx: mpsc::UnboundedReceiver<HotkeyEvent>) -> Result<()> {
        info!("Dictation engine event loop started");

        while !self.shutdown_signal.load(Ordering::SeqCst) {
            tokio::select! {
                Some(event) = event_rx.recv() => {
                    self.handle_hotkey_event(event).await;
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    // Check shutdown signal periodically
                }
            }
        }

        info!("Dictation engine event loop stopped");
        Ok(())
    }

    /// Handle hotkey event
    async fn handle_hotkey_event(&mut self, event: HotkeyEvent) {
        match event {
            HotkeyEvent::Pressed => {
                info!("🎹 Hotkey pressed - starting dictation");
                if let Err(e) = self.start_dictation().await {
                    error!("Failed to start dictation: {}", e);
                }
            }
            HotkeyEvent::Released => {
                info!("🎹 Hotkey released - stopping dictation");
                if let Err(e) = self.stop_dictation().await {
                    error!("Failed to stop dictation: {}", e);
                }
            }
        }
    }

    /// Start dictation session
    async fn start_dictation(&mut self) -> Result<()> {
        if self.is_dictating.load(Ordering::SeqCst) {
            warn!("Already dictating, ignoring start request");
            return Ok(());
        }

        info!("🎤 Starting dictation");
        self.is_dictating.store(true, Ordering::SeqCst);
        self.indicator.recording();

        // Start audio capture
        let capture_config = CaptureConfig {
            sample_rate: self.config.audio.sample_rate,
            device_name: self.config.audio.device.clone(),
            chunk_duration_ms: self.config.audio.chunk_duration_ms,
            buffer_capacity_secs: 2,
        };

        let mut audio_rx = self.audio_engine.start_capture(capture_config)?;

        // Clone needed values for the processing task
        let is_dictating = Arc::clone(&self.is_dictating);
        let injector = self.text_injector.clone();
        let model = Arc::clone(&self.model);
        let vad_enabled = self.config.vad.enabled;
        let indicator = Arc::clone(&self.indicator);
        let focus_settle_ms = self.config.injection.focus_settle_ms;

        if vad_enabled {
            // VAD-based processing: detect speech segments and transcribe them
            info!("🔊 VAD enabled - using speech detection");

            // Create VAD processor
            let vad_config = self.config.vad.to_energy_vad_config();
            let processor_config = self.config.vad.to_processor_config();
            let detector: Box<dyn VadDetector> = Box::new(EnergyVad::new(vad_config));
            let mut vad_processor = VadProcessor::new(processor_config, detector);

            // Spawn audio processing task
            tokio::spawn(async move {
                info!("📡 Audio processing task started (VAD mode)");

                loop {
                    match tokio::time::timeout(
                        tokio::time::Duration::from_millis(100),
                        audio_rx.recv(),
                    )
                    .await
                    {
                        Ok(Some(chunk)) => {
                            // Process through VAD
                            match vad_processor.process(chunk) {
                                Ok(Some(segment)) => {
                                    info!("🎯 Speech segment detected ({} chunks)", segment.len());
                                    indicator.processing();

                                    // Transcribe
                                    match Self::transcribe_with_model(Arc::clone(&model), segment).await {
                                        Ok(transcript) => {
                                            info!("📝 Transcription: {}", transcript.text);

                                            // Hide overlay before injection so target app keeps focus.
                                            indicator.hide();
                                            if focus_settle_ms > 0 {
                                                tokio::time::sleep(tokio::time::Duration::from_millis(
                                                    focus_settle_ms as u64,
                                                ))
                                                .await;
                                            }

                                            // Inject text into active application
                                            if let Err(e) = injector.inject(&transcript.text) {
                                                error!("Failed to inject text: {}", e);
                                            } else {
                                                info!("✅ Text injected successfully");
                                            }
                                        }
                                        Err(e) => {
                                            error!("Transcription failed: {}", e);
                                        }
                                    }

                                    if is_dictating.load(Ordering::SeqCst) {
                                        indicator.recording();
                                    }
                                }
                                Ok(None) => {
                                    // No complete segment yet
                                }
                                Err(e) => {
                                    error!("VAD processing failed: {}", e);
                                }
                            }
                        }
                        Ok(None) => {
                            debug!("Audio channel closed");
                            break;
                        }
                        Err(_) => {
                            if !is_dictating.load(Ordering::SeqCst) {
                                break;
                            }
                        }
                    }
                }

                indicator.hide();
                info!("📡 Audio processing task stopped");
            });
        } else {
            // Non-VAD mode: collect all audio and transcribe when hotkey is released
            info!("🔇 VAD disabled - transcribing all captured audio");

            // Spawn audio collection task
            tokio::spawn(async move {
                info!("📡 Audio collection task started (no VAD)");
                let mut collected_chunks = Vec::new();

                loop {
                    match tokio::time::timeout(
                        tokio::time::Duration::from_millis(100),
                        audio_rx.recv(),
                    )
                    .await
                    {
                        Ok(Some(chunk)) => {
                            debug!("Collected audio chunk: {} samples", chunk.samples.len());
                            collected_chunks.push(chunk);
                        }
                        Ok(None) => {
                            debug!("Audio channel closed");
                            break;
                        }
                        Err(_) => {
                            if !is_dictating.load(Ordering::SeqCst) {
                                break;
                            }
                        }
                    }
                }

                // Hotkey released - transcribe all collected audio
                if !collected_chunks.is_empty() {
                    info!("🎤 Hotkey released - transcribing {} chunks", collected_chunks.len());
                    indicator.processing();

                    // Create a speech segment from all collected chunks
                    let segment = crate::vad::SpeechSegment::new(collected_chunks);
                    
                    // DEBUG: Analyze captured audio
                    let samples = segment.get_samples();
                    let sample_rate = segment.sample_rate();
                    
                    // Calculate audio statistics
                    let duration_secs = samples.len() as f32 / sample_rate as f32;
                    let max_amplitude = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
                    let rms = (samples.iter().map(|&s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
                    let non_zero_samples = samples.iter().filter(|&&s| s.abs() > 0.0001).count();
                    
                    info!("📊 Audio statistics:");
                    info!("  - Total samples: {}", samples.len());
                    info!("  - Sample rate: {} Hz", sample_rate);
                    info!("  - Duration: {:.2} seconds", duration_secs);
                    info!("  - Max amplitude: {:.4}", max_amplitude);
                    info!("  - RMS level: {:.4}", rms);
                    info!("  - Non-zero samples: {} ({:.1}%)", 
                        non_zero_samples,
                        100.0 * non_zero_samples as f32 / samples.len() as f32
                    );

                    // Transcribe
                    match Self::transcribe_with_model(Arc::clone(&model), segment).await {
                        Ok(transcript) => {
                            info!("📝 Transcription: {}", transcript.text);

                            // Hide overlay before injection so target app keeps focus.
                            indicator.hide();
                            if focus_settle_ms > 0 {
                                tokio::time::sleep(tokio::time::Duration::from_millis(
                                    focus_settle_ms as u64,
                                ))
                                .await;
                            }

                            // Inject text into active application
                            if let Err(e) = injector.inject(&transcript.text) {
                                error!("Failed to inject text: {}", e);
                            } else {
                                info!("✅ Text injected successfully");
                            }
                        }
                        Err(e) => {
                            error!("Transcription failed: {}", e);
                        }
                    }
                } else {
                    info!("No audio collected during dictation session");
                }

                indicator.hide();
                info!("📡 Audio collection task stopped");
            });
        }

        Ok(())
    }

    /// Stop dictation session
    async fn stop_dictation(&mut self) -> Result<()> {
        if !self.is_dictating.load(Ordering::SeqCst) {
            warn!("Not dictating, ignoring stop request");
            return Ok(());
        }

        info!("🛑 Stopping dictation");
        self.is_dictating.store(false, Ordering::SeqCst);
        self.indicator.processing();

        // Stop audio capture
        self.audio_engine.stop_capture()?;

        Ok(())
    }

    async fn transcribe_with_model(
        model: Arc<Mutex<Box<dyn ModelRuntime>>>,
        segment: crate::vad::SpeechSegment,
    ) -> std::result::Result<Transcription, String> {
        match tokio::task::spawn_blocking(move || {
            let mut guard = model
                .lock()
                .map_err(|_| "Model mutex poisoned".to_string())?;
            guard
                .transcribe_segment(&segment)
                .map_err(|e| e.to_string())
        })
        .await
        {
            Ok(result) => result,
            Err(e) => Err(format!("Transcription task failed: {}", e)),
        }
    }

    /// List available audio devices for debugging
    fn list_audio_devices(&self) {
        use crate::audio::devices::AudioDeviceManager;
        
        let device_manager = AudioDeviceManager::new();
        match device_manager.list_input_devices() {
            Ok(devices) => {
                info!("🎙️  Available audio input devices:");
                for device in devices {
                    info!("  - {}", device);
                }
            }
            Err(e) => {
                warn!("Failed to list audio devices: {}", e);
            }
        }
    }

    /// Shutdown the dictation engine
    pub fn shutdown(&mut self) {
        info!("Shutting down dictation engine");
        self.shutdown_signal.store(true, Ordering::SeqCst);

        // Stop dictation if active
        if self.is_dictating.load(Ordering::SeqCst) {
            let _ = self.audio_engine.stop_capture();
            self.is_dictating.store(false, Ordering::SeqCst);
        }
        self.indicator.hide();

        if let Err(e) = self.hotkey_manager.unregister() {
            error!("Failed to unregister hotkeys: {}", e);
        }

        if let Ok(mut model) = self.model.lock() {
            model.unload();
        } else {
            error!("Failed to acquire model lock during shutdown");
        }
    }

    /// Check if currently dictating
    pub fn is_dictating(&self) -> bool {
        self.is_dictating.load(Ordering::SeqCst)
    }
}

impl Drop for DictationEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}
