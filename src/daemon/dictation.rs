//! Dictation Engine
//!
//! Orchestrates the full dictation pipeline:
//! Hotkey → Audio Capture → VAD → Model → Text Injection

use crate::audio::{AudioEngine, CaptureConfig};
use crate::config::Config;
use crate::models::{MockModel, ModelConfig, ModelRuntime};
use crate::platform::{HotkeyConfig as PlatformHotkeyConfig, HotkeyEvent, HotkeyManager, InjectorConfig, TextInjector};
use crate::vad::{EnergyVad, VadDetector, VadProcessor};
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
    model: Box<dyn ModelRuntime>,

    /// Is currently dictating
    is_dictating: Arc<AtomicBool>,

    /// Shutdown signal
    shutdown_signal: Arc<AtomicBool>,
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

        // Create model (using mock for now)
        let mut model: Box<dyn ModelRuntime> = Box::new(MockModel::new());
        let model_config = ModelConfig {
            model_path: config.model.model_path.clone(),
            language: config.model.language.clone(),
            use_gpu: config.model.device == "gpu",
            ..Default::default()
        };
        model.load(model_config)?;

        info!("✅ Dictation engine initialized");

        Ok(Self {
            config,
            hotkey_manager,
            text_injector,
            audio_engine,
            model,
            is_dictating: Arc::new(AtomicBool::new(false)),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Start the dictation engine
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting dictation engine");

        // Register global hotkey
        let hotkey_str = self.config.hotkey.trigger.clone();
        let hotkey_config = PlatformHotkeyConfig::from_string(&hotkey_str)
            .context("Failed to parse hotkey configuration")?;

        let event_rx = self
            .hotkey_manager
            .register(hotkey_config)
            .context("Failed to register hotkey")?;

        info!("✅ Hotkey registered: {}", hotkey_str);

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
                debug!("Hotkey pressed - starting dictation");
                if let Err(e) = self.start_dictation().await {
                    error!("Failed to start dictation: {}", e);
                }
            }
            HotkeyEvent::Released => {
                debug!("Hotkey released - stopping dictation");
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

        // Start audio capture
        let capture_config = CaptureConfig {
            sample_rate: self.config.audio.sample_rate,
            device_name: self.config.audio.device.clone(),
            chunk_duration_ms: self.config.audio.chunk_duration_ms,
            buffer_capacity_secs: 2,
        };

        let mut audio_rx = self.audio_engine.start_capture(capture_config)?;

        // Create VAD processor
        let vad_config = self.config.vad.to_energy_vad_config();
        let processor_config = self.config.vad.to_processor_config();
        let detector: Box<dyn VadDetector> = Box::new(EnergyVad::new(vad_config));
        let mut vad_processor = VadProcessor::new(processor_config, detector);

        // Clone needed values for the processing task
        let is_dictating = Arc::clone(&self.is_dictating);
        let injector = self.text_injector.clone();
        let model = self.model_clone();

        // Spawn audio processing task
        tokio::spawn(async move {
            info!("📡 Audio processing task started");

            while is_dictating.load(Ordering::SeqCst) {
                // Receive audio chunk
                match audio_rx.recv().await {
                    Some(chunk) => {
                        // Process through VAD
                        match vad_processor.process(chunk) {
                            Ok(Some(segment)) => {
                                info!("🎯 Speech segment detected ({} chunks)", segment.len());

                                // Transcribe
                                match model.transcribe_segment(&segment) {
                                    Ok(transcript) => {
                                        info!("📝 Transcription: {}", transcript.text);

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
                            }
                            Ok(None) => {
                                // No complete segment yet
                            }
                            Err(e) => {
                                error!("VAD processing failed: {}", e);
                            }
                        }
                    }
                    None => {
                        debug!("Audio channel closed");
                        break;
                    }
                }
            }

            info!("📡 Audio processing task stopped");
        });

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

        // Stop audio capture
        self.audio_engine.stop_capture()?;

        Ok(())
    }

    /// Clone the model (workaround for non-Clone trait)
    /// TODO: Improve this with proper model management
    fn model_clone(&self) -> Box<dyn ModelRuntime> {
        // For now, create a new mock model
        // In production, we'd use Arc<Mutex<>> or similar
        let mut model: Box<dyn ModelRuntime> = Box::new(MockModel::new());
        let config = ModelConfig::default();
        let _ = model.load(config);
        model
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

        if let Err(e) = self.hotkey_manager.unregister() {
            error!("Failed to unregister hotkeys: {}", e);
        }

        self.model.unload();
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
