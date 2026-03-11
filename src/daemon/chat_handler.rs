//! Chat Handler
//!
//! Orchestrates the chat pipeline with hotkey support:
//! Hotkey → Audio Capture → ChatEngine (STT→LLM→TTS) → Audio Playback

use crate::audio::{AudioEngine, CaptureConfig};
use crate::chat::ChatEngine;
use crate::config::Config;
use crate::indicator::RecordingIndicator;
use crate::platform::{HotkeyConfig as PlatformHotkeyConfig, HotkeyEvent, HotkeyManager};
use anyhow::{Context, Result};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Chat handler state
pub struct ChatHandler {
    /// Configuration
    config: Config,

    /// Chat engine
    chat_engine: Arc<ChatEngine>,

    /// Hotkey manager (optional when global hotkeys are unavailable)
    hotkey_manager: Option<HotkeyManager>,

    /// Audio engine
    audio_engine: AudioEngine,

    /// Is currently chatting
    is_chatting: Arc<AtomicBool>,

    /// Toggle state (for toggle mode)
    is_toggle_active: Arc<AtomicBool>,

    /// Shutdown signal
    shutdown_signal: Arc<AtomicBool>,

    /// Floating UI indicator
    indicator: Arc<RecordingIndicator>,

    /// Last hotkey event time (for debouncing)
    last_hotkey_time: Arc<std::sync::Mutex<Instant>>,
}

impl ChatHandler {
    /// Create a new chat handler
    pub fn new(config: Config, chat_engine: Arc<ChatEngine>) -> Result<Self> {
        info!("Initializing chat handler");

        // Create hotkey manager. If this fails (common on some Wayland setups),
        // keep the handler available for manual IPC start/stop commands.
        let hotkey_manager = match HotkeyManager::new() {
            Ok(manager) => Some(manager),
            Err(e) => {
                warn!(
                    "Global hotkeys unavailable for chat ({}). Manual IPC commands will still work.",
                    e
                );
                None
            }
        };

        // Create audio engine
        let audio_engine = AudioEngine::new();

        info!("✅ Chat handler initialized");

        Ok(Self {
            indicator: Arc::new(RecordingIndicator::new(config.ui.recording_overlay)),
            config,
            chat_engine,
            hotkey_manager,
            audio_engine,
            is_chatting: Arc::new(AtomicBool::new(false)),
            is_toggle_active: Arc::new(AtomicBool::new(false)),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            last_hotkey_time: Arc::new(std::sync::Mutex::new(Instant::now())),
        })
    }

    /// Create a new chat handler with shared is_chatting flag
    pub fn with_chatting_flag(
        config: Config,
        chat_engine: Arc<ChatEngine>,
        is_chatting: Arc<AtomicBool>,
    ) -> Result<Self> {
        let mut handler = Self::new(config, chat_engine)?;
        handler.is_chatting = is_chatting;
        Ok(handler)
    }

    /// Start the chat handler
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting chat handler");

        // List available audio devices for debugging
        self.list_audio_devices();

        let hotkey_manager = self.hotkey_manager.as_mut().ok_or_else(|| {
            anyhow::anyhow!(
                "Global hotkey backend unavailable for chat on this system. Use IPC commands instead."
            )
        })?;

        // Register global hotkey
        let hotkey_str = self.config.chat.hotkey.clone();
        let hotkey_config = PlatformHotkeyConfig::from_string(&hotkey_str)
            .context("Failed to parse chat hotkey configuration")?;

        let event_rx = hotkey_manager
            .register(hotkey_config)
            .context("Failed to register chat hotkey")?;

        info!("✅ Chat hotkey registered: {}", hotkey_str);

        // Take ownership of hotkey_manager to start the listener
        let hotkey_manager = self
            .hotkey_manager
            .take()
            .ok_or_else(|| anyhow::anyhow!("Hotkey manager missing after registration"))?;

        hotkey_manager
            .start_listener()
            .context("Failed to start chat hotkey listener")?;

        info!("✅ Chat hotkey listener started");

        // Start hotkey event loop
        self.run_event_loop(event_rx).await?;

        Ok(())
    }

    /// Run the hotkey event loop
    async fn run_event_loop(
        &mut self,
        mut event_rx: mpsc::UnboundedReceiver<HotkeyEvent>,
    ) -> Result<()> {
        info!("Chat handler event loop started");

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

        info!("Chat handler event loop stopped");
        Ok(())
    }

    /// Handle hotkey event
    async fn handle_hotkey_event(&mut self, event: HotkeyEvent) {
        // Debouncing: Ignore events that occur too quickly (within 200ms)
        const DEBOUNCE_MS: u64 = 200;

        let now = Instant::now();
        let should_process = {
            let mut last_time = self.last_hotkey_time.lock().unwrap();
            let elapsed = now.duration_since(*last_time);

            if elapsed < Duration::from_millis(DEBOUNCE_MS) {
                debug!(
                    "🔇 Ignoring hotkey event (debounce): {}ms since last event",
                    elapsed.as_millis()
                );
                false
            } else {
                *last_time = now;
                true
            }
        };

        if !should_process {
            return;
        }

        let mode = &self.config.chat.mode;

        if mode == "toggle" {
            // Toggle mode: press once to start, press again to stop
            if let HotkeyEvent::Pressed = event {
                let is_active = self.is_toggle_active.load(Ordering::SeqCst);
                if is_active {
                    // Currently chatting, stop it
                    info!("🎹 Chat hotkey pressed (toggle mode) - stopping chat");
                    self.is_toggle_active.store(false, Ordering::SeqCst);
                    if let Err(e) = self.stop_chat().await {
                        error!("Failed to stop chat: {}", e);
                    }
                } else {
                    // Not chatting, start it
                    info!("🎹 Chat hotkey pressed (toggle mode) - starting chat");
                    self.is_toggle_active.store(true, Ordering::SeqCst);
                    if let Err(e) = self.start_chat().await {
                        error!("Failed to start chat: {}", e);
                    }
                }
            }
            // Ignore Released events in toggle mode
        } else {
            // Push-to-talk mode: hold to record
            match event {
                HotkeyEvent::Pressed => {
                    info!("🎹 Chat hotkey pressed (push-to-talk mode) - starting chat session");
                    if let Err(e) = self.start_chat().await {
                        error!("Failed to start chat: {}", e);
                    }
                }
                HotkeyEvent::Released => {
                    info!("🎹 Chat hotkey released (push-to-talk mode) - processing chat");
                    if let Err(e) = self.stop_chat().await {
                        error!("Failed to stop chat: {}", e);
                    }
                }
            }
        }
    }

    /// Start chat session
    pub async fn start_chat(&mut self) -> Result<()> {
        if self.is_chatting.load(Ordering::SeqCst) {
            warn!("Already chatting, ignoring start request");
            return Ok(());
        }

        info!("🎤 Starting chat session");
        self.is_chatting.store(true, Ordering::SeqCst);
        self.indicator.recording();

        // Start audio capture
        let capture_config = CaptureConfig {
            sample_rate: self.config.audio.sample_rate,
            device_name: self.config.audio.device.clone(),
            chunk_duration_ms: self.config.audio.chunk_duration_ms,
            buffer_capacity_secs: 10, // Longer buffer for chat (conversations can be longer)
        };

        let audio_rx = match self.audio_engine.start_capture(capture_config) {
            Ok(rx) => rx,
            Err(e) => {
                // Failed to start - clean up state
                error!("Failed to start audio capture for chat: {}", e);
                self.is_chatting.store(false, Ordering::SeqCst);
                self.indicator.hide();
                return Err(e.into());
            }
        };

        let mut audio_rx = audio_rx;

        // Clone needed values for the processing task
        let is_chatting = Arc::clone(&self.is_chatting);
        let chat_engine = Arc::clone(&self.chat_engine);
        let indicator = Arc::clone(&self.indicator);

        // Spawn audio processing task (use spawn_blocking since audio playback is not Send)
        std::thread::spawn(move || {
            // Create a runtime for this thread
            let rt = tokio::runtime::Runtime::new()
                .expect("Failed to create runtime for chat processing");

            rt.block_on(async {
                info!("📊 Chat audio processing task started");

                // Collect audio chunks (no VAD needed for chat - we process when user releases hotkey)
                let mut audio_buffer = Vec::new();

                // Keep collecting until channel is closed or we're told to stop
                loop {
                    // Check if we should stop collecting
                    if !is_chatting.load(Ordering::SeqCst) {
                        // Stop signal received - drain remaining chunks with a timeout
                        info!("📊 Stop signal received, draining remaining audio chunks...");

                        // Give audio capture a moment to finish sending chunks
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

                        // Drain all remaining chunks
                        while let Ok(chunk) = audio_rx.try_recv() {
                            debug!("📦 Draining audio chunk: {} samples", chunk.samples.len());
                            audio_buffer.extend_from_slice(&chunk.samples);
                        }
                        break;
                    }

                    // Try to receive with a timeout so we can check the flag periodically
                    match tokio::time::timeout(
                        tokio::time::Duration::from_millis(100),
                        audio_rx.recv(),
                    )
                    .await
                    {
                        Ok(Some(chunk)) => {
                            debug!("📦 Received audio chunk: {} samples", chunk.samples.len());
                            audio_buffer.extend_from_slice(&chunk.samples);
                        }
                        Ok(None) => {
                            info!("📡 Audio stream closed");
                            break;
                        }
                        Err(_) => {
                            // Timeout - continue loop to check flag again
                            continue;
                        }
                    }
                }

                info!(
                    "📊 Chat audio collection complete: {} samples",
                    audio_buffer.len()
                );

                // Only process if we have audio
                if !audio_buffer.is_empty() {
                    indicator.processing();
                    info!("🤖 Processing chat through pipeline...");

                    // Process through chat engine (STREAMING: STT → LLM → TTS → Playback)
                    match chat_engine.process_audio_streaming(audio_buffer).await {
                        Ok(response) => {
                            info!("✅ Chat response complete!");
                            info!("   User: \"{}\"", response.user_text);
                            info!("   Assistant: \"{}\"", response.assistant_text);
                            info!("   Total time: {}ms", response.total_duration_ms);
                            info!(
                                "   STT: {}ms, LLM: {}ms, TTS: {}ms, Playback: {}ms",
                                response.stt_duration_ms,
                                response.llm_response.generation_time_ms,
                                response.tts_synthesis.synthesis_time_ms,
                                response.playback_duration_ms
                            );
                        }
                        Err(e) => {
                            error!("❌ Chat processing failed: {}", e);
                        }
                    }
                } else {
                    warn!("⚠️  No audio captured for chat");
                }

                indicator.hide();
                info!("📊 Chat processing task complete");
            });
        });

        Ok(())
    }

    /// Stop chat session
    pub async fn stop_chat(&mut self) -> Result<()> {
        if !self.is_chatting.load(Ordering::SeqCst) {
            debug!("Not chatting, ignoring stop request");
            return Ok(());
        }

        info!("🛑 Stopping chat session");

        // Stop audio capture FIRST (this ensures all audio is in the channel)
        self.audio_engine.stop_capture()?;

        // THEN signal the background thread to stop collecting and process
        // This ensures the thread can drain all remaining chunks from the channel
        self.is_chatting.store(false, Ordering::SeqCst);

        // Indicator will be hidden by the processing task after completion

        Ok(())
    }

    /// Clear chat history
    pub async fn clear_history(&self) -> Result<()> {
        info!("🗑️  Clearing chat history");
        self.chat_engine.clear_history().await;
        Ok(())
    }

    /// List available audio devices
    fn list_audio_devices(&self) {
        match self.audio_engine.list_devices() {
            Ok(devices) => {
                info!("Available audio input devices for chat:");
                for (idx, device) in devices.iter().enumerate() {
                    info!("  [{}] {}", idx, device);
                }
            }
            Err(e) => {
                warn!("Failed to list audio devices: {}", e);
            }
        }
    }

    /// Get is_chatting flag reference
    pub fn is_chatting_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_chatting)
    }

    /// Set shutdown signal
    pub fn set_shutdown_signal(&mut self, signal: Arc<AtomicBool>) {
        self.shutdown_signal = signal;
    }
}
