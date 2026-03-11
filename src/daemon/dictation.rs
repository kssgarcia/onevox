//! Dictation Engine
//!
//! Orchestrates the full dictation pipeline:
//! Hotkey → Audio Capture → VAD → Model → Text Injection

use crate::audio::{AudioEngine, CaptureConfig};
use crate::config::Config;
use crate::history::{HistoryEntry, HistoryManager};
use crate::indicator::RecordingIndicator;
use crate::models::{ModelConfig, ModelRuntime, Transcription, WhisperCpp};

#[cfg(feature = "onnx")]
use crate::models::OnnxRuntime;
use crate::platform::{
    HotkeyConfig as PlatformHotkeyConfig, HotkeyEvent, HotkeyManager, InjectorConfig, TextInjector,
};
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

    /// Hotkey manager (optional when global hotkeys are unavailable, e.g. some Wayland setups)
    hotkey_manager: Option<HotkeyManager>,

    /// Text injector
    text_injector: TextInjector,

    /// Audio engine
    audio_engine: AudioEngine,

    /// Model runtime
    model: Arc<Mutex<Box<dyn ModelRuntime>>>,

    /// History manager
    history_manager: Arc<HistoryManager>,

    /// Is currently dictating
    is_dictating: Arc<AtomicBool>,

    /// Toggle state (for toggle mode)
    is_toggle_active: Arc<AtomicBool>,

    /// Shutdown signal
    shutdown_signal: Arc<AtomicBool>,

    /// Floating UI indicator
    indicator: Arc<RecordingIndicator>,
}

impl DictationEngine {
    /// Create a new dictation engine
    pub fn new(config: Config) -> Result<Self> {
        info!("Initializing dictation engine");

        // Create history manager
        let history_config = config.history.clone();
        let history_manager = HistoryManager::new(history_config)
            .map_err(|e| anyhow::anyhow!("Failed to create history manager: {}", e))?;

        Self::with_history(config, Arc::new(history_manager))
    }

    /// Create a new dictation engine with an existing history manager
    pub fn with_history(config: Config, history_manager: Arc<HistoryManager>) -> Result<Self> {
        // Note: logging is done in new(), not here to avoid duplicate messages

        // Create hotkey manager. If this fails (common on some Wayland setups),
        // keep the engine available for manual IPC start/stop dictation commands.
        let hotkey_manager = match HotkeyManager::new() {
            Ok(manager) => Some(manager),
            Err(e) => {
                warn!(
                    "Global hotkeys unavailable ({}). Manual IPC commands will still work.",
                    e
                );
                None
            }
        };

        // Create text injector
        let injector_config = InjectorConfig {
            key_delay_ms: config.injection.paste_delay_ms as u64,
            initial_delay_ms: 50,
        };
        let text_injector = TextInjector::new(injector_config);

        // Create audio engine
        let audio_engine = AudioEngine::new();

        // Auto-detect backend from model path
        let model_path = &config.model.model_path;
        let is_onnx_model = model_path.contains("parakeet")
            || model_path.ends_with(".onnx")
            || model_path.contains("onnx");

        let mut model: Box<dyn ModelRuntime> = if is_onnx_model {
            #[cfg(feature = "onnx")]
            {
                info!("Auto-detected ONNX model from path: {}", model_path);
                info!("Using ONNX Runtime backend");
                Box::new(OnnxRuntime::new()?)
            }
            #[cfg(not(feature = "onnx"))]
            {
                error!(
                    "ONNX model detected ('{}') but feature not enabled. Rebuild with --features onnx",
                    model_path
                );
                return Err(crate::Error::Model(format!(
                    "ONNX model requires --features onnx build. Model: {}",
                    model_path
                ))
                .into());
            }
        } else {
            // Default to whisper.cpp for GGML models
            info!("Auto-detected GGML model from path: {}", model_path);
            info!("Using whisper.cpp backend");
            Box::new(WhisperCpp::new()?)
        };

        let model_config = ModelConfig {
            model_path: config.model.model_path.clone(),
            use_gpu: config.model.device == "gpu" || config.model.device == "auto",
            ..Default::default()
        };

        // Report GPU status at startup
        Self::report_gpu_status(&model_config);

        model.load(model_config)?;

        info!("✅ Dictation engine initialized");

        Ok(Self {
            indicator: Arc::new(RecordingIndicator::new(config.ui.recording_overlay)),
            config,
            hotkey_manager,
            text_injector,
            audio_engine,
            model: Arc::new(Mutex::new(model)),
            history_manager,
            is_dictating: Arc::new(AtomicBool::new(false)),
            is_toggle_active: Arc::new(AtomicBool::new(false)),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Start the dictation engine
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting dictation engine");

        // List available audio devices for debugging
        self.list_audio_devices();

        // Try to start hotkey listener if available
        if let Some(hotkey_manager) = self.hotkey_manager.as_mut() {
            // Register global hotkey
            let hotkey_str = self.config.hotkey.trigger.clone();
            let hotkey_config = PlatformHotkeyConfig::from_string(&hotkey_str)
                .context("Failed to parse hotkey configuration")?;

            let event_rx = hotkey_manager
                .register(hotkey_config)
                .context("Failed to register hotkey")?;

            info!("✅ Hotkey registered: {}", hotkey_str);

            // Take ownership of hotkey_manager to start the listener
            // (it consumes self and moves into the listener thread)
            let hotkey_manager = self
                .hotkey_manager
                .take()
                .ok_or_else(|| anyhow::anyhow!("Hotkey manager missing after registration"))?;

            hotkey_manager
                .start_listener()
                .context("Failed to start hotkey listener")?;

            info!("✅ Hotkey listener started");

            // Start hotkey event loop
            self.run_event_loop(event_rx).await?;
        } else {
            warn!(
                "Global hotkeys unavailable. Use IPC commands: 'onevox start-dictation' and 'onevox stop-dictation'"
            );
            // Keep the engine running but without hotkey support
            // Just wait for shutdown signal
            while !self.shutdown_signal.load(Ordering::SeqCst) {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }

        Ok(())
    }

    /// Run the hotkey event loop
    async fn run_event_loop(
        &mut self,
        mut event_rx: mpsc::UnboundedReceiver<HotkeyEvent>,
    ) -> Result<()> {
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
        let mode = &self.config.hotkey.mode;

        if mode == "toggle" {
            // Toggle mode: press once to start, press again to stop
            if let HotkeyEvent::Pressed = event {
                let is_active = self.is_toggle_active.load(Ordering::SeqCst);
                if is_active {
                    // Currently recording, stop it
                    info!("🎹 Hotkey pressed (toggle mode) - stopping dictation");
                    self.is_toggle_active.store(false, Ordering::SeqCst);
                    if let Err(e) = self.stop_dictation().await {
                        error!("Failed to stop dictation: {}", e);
                    }
                } else {
                    // Not recording, start it
                    info!("🎹 Hotkey pressed (toggle mode) - starting dictation");
                    self.is_toggle_active.store(true, Ordering::SeqCst);
                    if let Err(e) = self.start_dictation().await {
                        error!("Failed to start dictation: {}", e);
                    }
                }
            }
            // Ignore Released events in toggle mode
        } else {
            // Push-to-talk mode: hold to record
            match event {
                HotkeyEvent::Pressed => {
                    info!("🎹 Hotkey pressed (push-to-talk mode) - starting dictation");
                    if let Err(e) = self.start_dictation().await {
                        error!("Failed to start dictation: {}", e);
                    }
                }
                HotkeyEvent::Released => {
                    info!("🎹 Hotkey released (push-to-talk mode) - stopping dictation");
                    if let Err(e) = self.stop_dictation().await {
                        error!("Failed to stop dictation: {}", e);
                    }
                }
            }
        }
    }

    /// Start dictation session
    pub async fn start_dictation(&mut self) -> Result<()> {
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

        let audio_rx = match self.audio_engine.start_capture(capture_config) {
            Ok(rx) => rx,
            Err(e) => {
                // Failed to start - clean up state
                error!("Failed to start audio capture: {}", e);
                self.is_dictating.store(false, Ordering::SeqCst);
                self.indicator.hide();
                return Err(e.into());
            }
        };

        let mut audio_rx = audio_rx;

        // Clone needed values for the processing task
        let is_dictating = Arc::clone(&self.is_dictating);
        let injector = self.text_injector.clone();
        let model = Arc::clone(&self.model);
        let model_name = self.config.model.model_path.clone();
        let history_manager = Arc::clone(&self.history_manager);
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
                                    // Skip segments that are too short (likely noise or button clicks)
                                    const MIN_AUDIO_DURATION_MS: u64 = 300;
                                    if segment.duration_ms < MIN_AUDIO_DURATION_MS {
                                        debug!(
                                            "⏭️  Skipping short segment ({}ms < {}ms threshold)",
                                            segment.duration_ms, MIN_AUDIO_DURATION_MS
                                        );
                                        // Return to recording state without transcribing
                                        if is_dictating.load(Ordering::SeqCst) {
                                            indicator.recording();
                                        }
                                        continue;
                                    }

                                    info!(
                                        "🎯 Speech segment detected ({} chunks, {}ms)",
                                        segment.len(),
                                        segment.duration_ms
                                    );
                                    indicator.processing();

                                    // Transcribe
                                    let model_clone = Arc::clone(&model);
                                    let model_name_clone = model_name.clone();
                                    let history_clone = Arc::clone(&history_manager);

                                    match Self::transcribe_with_model(model_clone, segment).await {
                                        Ok(transcript) => {
                                            info!("📝 Transcription: {}", transcript.text);

                                            // Record to history
                                            let history_entry = HistoryEntry::new(
                                                transcript.text.clone(),
                                                model_name_clone,
                                                transcript.processing_time_ms,
                                                transcript.confidence,
                                            );

                                            if let Err(e) =
                                                history_clone.add_entry(history_entry).await
                                            {
                                                error!("Failed to record history: {}", e);
                                            }

                                            // Hide overlay before injection so target app keeps focus.
                                            indicator.hide();
                                            if focus_settle_ms > 0 {
                                                tokio::time::sleep(
                                                    tokio::time::Duration::from_millis(
                                                        focus_settle_ms as u64,
                                                    ),
                                                )
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
                    // Create a speech segment from all collected chunks
                    let mut segment = crate::vad::SpeechSegment::new(collected_chunks);

                    // Skip segments that are too short (likely accidental key presses)
                    const MIN_AUDIO_DURATION_MS: u64 = 300;
                    if segment.duration_ms < MIN_AUDIO_DURATION_MS {
                        info!(
                            "⏭️  Skipping short recording ({}ms < {}ms threshold)",
                            segment.duration_ms, MIN_AUDIO_DURATION_MS
                        );
                        indicator.hide();
                        return;
                    }

                    info!(
                        "🎤 Hotkey released - transcribing {} chunks ({}ms)",
                        segment.len(),
                        segment.duration_ms
                    );
                    indicator.processing();

                    // DEBUG: Analyze captured audio
                    let sample_rate = segment.sample_rate();
                    let samples = segment.get_samples();

                    // Calculate audio statistics
                    let duration_secs = samples.len() as f32 / sample_rate as f32;
                    let max_amplitude = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
                    let rms =
                        (samples.iter().map(|&s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
                    let non_zero_samples = samples.iter().filter(|&&s| s.abs() > 0.0001).count();

                    info!("📊 Audio statistics:");
                    info!("  - Total samples: {}", samples.len());
                    info!("  - Sample rate: {} Hz", sample_rate);
                    info!("  - Duration: {:.2} seconds", duration_secs);
                    info!("  - Max amplitude: {:.4}", max_amplitude);
                    info!("  - RMS level: {:.4}", rms);
                    info!(
                        "  - Non-zero samples: {} ({:.1}%)",
                        non_zero_samples,
                        100.0 * non_zero_samples as f32 / samples.len() as f32
                    );

                    // Transcribe
                    match Self::transcribe_with_model(Arc::clone(&model), segment).await {
                        Ok(transcript) => {
                            info!("📝 Transcription: {}", transcript.text);

                            // Record to history
                            let history_entry = HistoryEntry::new(
                                transcript.text.clone(),
                                model_name,
                                transcript.processing_time_ms,
                                transcript.confidence,
                            );

                            if let Err(e) = history_manager.add_entry(history_entry).await {
                                error!("Failed to record history: {}", e);
                            }

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
    pub async fn stop_dictation(&mut self) -> Result<()> {
        if !self.is_dictating.load(Ordering::SeqCst) {
            warn!("Not dictating, ignoring stop request");
            return Ok(());
        }

        info!("🛑 Stopping dictation");
        self.is_dictating.store(false, Ordering::SeqCst);
        self.indicator.processing();

        // Stop audio capture
        self.audio_engine.stop_capture()?;

        // On macOS, give the audio system time to fully release the device
        // This prevents audio quality degradation issues specific to CoreAudio
        #[cfg(target_os = "macos")]
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(())
    }

    async fn transcribe_with_model(
        model: Arc<Mutex<Box<dyn ModelRuntime>>>,
        mut segment: crate::vad::SpeechSegment,
    ) -> std::result::Result<Transcription, String> {
        let total_start = std::time::Instant::now();
        let audio_duration_ms = segment.duration_ms;

        let lock_start = std::time::Instant::now();
        let result = tokio::task::spawn_blocking(move || {
            let mut guard = model
                .lock()
                .map_err(|_| "Model mutex poisoned".to_string())?;

            let lock_time = lock_start.elapsed();
            if lock_time.as_millis() > 50 {
                warn!(
                    "⚠️  Model lock took {}ms (potential contention)",
                    lock_time.as_millis()
                );
            }

            guard
                .transcribe_segment(&mut segment)
                .map_err(|e| e.to_string())
        })
        .await;

        let total_time = total_start.elapsed();

        match result {
            Ok(r) => r,
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

    /// Report GPU acceleration status at startup
    fn report_gpu_status(config: &ModelConfig) {
        if config.use_gpu {
            #[cfg(all(feature = "metal", target_os = "macos"))]
            {
                info!("🎮 GPU Acceleration: ENABLED (Metal)");
            }

            #[cfg(all(feature = "cuda", not(target_os = "macos")))]
            {
                info!("🎮 GPU Acceleration: ENABLED (CUDA)");
            }

            #[cfg(all(feature = "vulkan", not(feature = "cuda"), not(feature = "metal")))]
            {
                info!("🎮 GPU Acceleration: ENABLED (Vulkan)");
            }

            #[cfg(not(any(feature = "metal", feature = "cuda", feature = "vulkan")))]
            {
                warn!("⚠️  GPU requested but no GPU backend compiled");
                warn!("💡 Rebuild with appropriate GPU feature:");

                #[cfg(target_os = "macos")]
                warn!("   cargo build --release --features metal");

                #[cfg(target_os = "linux")]
                warn!("   cargo build --release --features cuda   # or vulkan");

                #[cfg(target_os = "windows")]
                warn!("   cargo build --release --features cuda   # or vulkan");
            }
        } else {
            info!("💻 GPU Acceleration: DISABLED");
            debug!("Using CPU-only mode as configured");
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

        if let Some(hotkey_manager) = self.hotkey_manager.as_mut()
            && let Err(e) = hotkey_manager.unregister()
        {
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

    /// Get reference to history manager
    pub fn history_manager(&self) -> &Arc<HistoryManager> {
        &self.history_manager
    }
}

impl Drop for DictationEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}
