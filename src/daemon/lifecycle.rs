//! Daemon Lifecycle Management
//!
//! Handles daemon startup, shutdown, and lifecycle events.

use crate::chat::ChatEngine;
use crate::config::{Config, LlmConfig, TtsConfig};
use crate::daemon::chat_handler::ChatHandler;
use crate::daemon::dictation::DictationEngine;
use crate::daemon::state::DaemonState;
use crate::ipc::{IpcClient, IpcServer};
use crate::models::{
    ModelConfig, ModelRuntime, WhisperCpp,
    llm_runtime::{LlmRuntime, LlmRuntimeConfig},
    tts_runtime::{TtsRuntime, TtsRuntimeConfig},
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Daemon lifecycle manager
pub struct Lifecycle {
    config: Config,
    state: Arc<RwLock<DaemonState>>,
}

impl Lifecycle {
    /// Create a new lifecycle manager
    pub fn new(config: Config) -> Self {
        let state = Arc::new(RwLock::new(DaemonState::new(config.clone())));
        Self { config, state }
    }

    /// Create a new lifecycle manager with async initialization (recommended)
    pub async fn new_async(config: Config) -> Self {
        let state = Arc::new(RwLock::new(DaemonState::new_async(config.clone()).await));
        Self { config, state }
    }

    /// Start the daemon
    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 Starting Onevox daemon v{}", env!("CARGO_PKG_VERSION"));

        // Check if daemon is already running
        if self.is_already_running().await {
            warn!("Daemon is already running");
            return Err(anyhow::anyhow!("Daemon is already running"));
        }

        // Initialize IPC server
        let socket_path = IpcClient::default_socket_path();
        let mut ipc_server = IpcServer::new(socket_path.clone(), Arc::clone(&self.state));

        ipc_server
            .start()
            .await
            .context("Failed to start IPC server")?;

        info!("✅ IPC server started at {:?}", socket_path);

        // Mark daemon as ready
        {
            let mut state = self.state.write().await;
            state.set_ready();
        }

        info!("✅ Onevox daemon is ready");

        // Run the event loop
        self.run_event_loop(ipc_server).await?;

        Ok(())
    }

    /// Run the main event loop
    async fn run_event_loop(&self, mut ipc_server: IpcServer) -> Result<()> {
        info!("📡 Starting event loop");

        // Spawn IPC server task
        let ipc_handle = tokio::spawn(async move {
            if let Err(e) = ipc_server.run().await {
                error!("IPC server error: {}", e);
            }
        });

        // Initialize and start dictation engine in the background
        // We'll use a separate thread since HotkeyManager is not Send
        let config = self.config.clone();
        let state_clone = Arc::clone(&self.state);
        let _dictation_handle = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async {
                // Get history manager from state
                let history_manager = {
                    let state = state_clone.read().await;
                    Arc::clone(state.history_manager())
                };

                // Create command channel for IPC control
                let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel();

                // Register the channel with state so IPC can send commands
                {
                    let mut state = state_clone.write().await;
                    state.set_dictation_channel(cmd_tx);
                }

                // Try to initialize dictation engine with retries
                let mut retry_count = 0;
                let max_retries = 3;

                info!("Initializing dictation engine (IPC handler)");
                loop {
                    match DictationEngine::with_history(config.clone(), Arc::clone(&history_manager)) {
                        Ok(mut engine) => {
                            info!("✅ Dictation engine initialized");

                            // Start the engine's hotkey listener in a background thread
                            // This engine instance handles hotkey events
                            let config_for_hotkey = config.clone();
                            let history_for_hotkey = Arc::clone(&history_manager);

                            info!("Starting hotkey listener thread");
                            std::thread::spawn(move || {
                                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                                rt.block_on(async {
                                    debug!("Initializing dictation engine (hotkey handler)");
                                    match DictationEngine::with_history(config_for_hotkey, history_for_hotkey) {
                                        Ok(mut hotkey_engine) => {
                                            if let Err(e) = hotkey_engine.start().await {
                                                error!("Dictation engine hotkey listener error: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to create engine for hotkey listener: {}", e);
                                        }
                                    }
                                });
                            });

                            // Listen for IPC commands in the main loop
                            // This engine instance handles IPC commands
                            while let Some(cmd) = cmd_rx.recv().await {
                                match cmd {
                                    crate::daemon::state::DictationCommand::Start => {
                                        info!("📡 IPC command: Start dictation");
                                        if let Err(e) = engine.start_dictation().await {
                                            error!("Failed to start dictation: {}", e);
                                        }
                                    }
                                    crate::daemon::state::DictationCommand::Stop => {
                                        info!("📡 IPC command: Stop dictation");
                                        if let Err(e) = engine.stop_dictation().await {
                                            error!("Failed to stop dictation: {}", e);
                                        }
                                    }
                                }
                            }
                            break;
                        }
                        Err(e) => {
                            let error_msg = e.to_string();

                            // Check if this is a model-related error (missing model file)
                            let is_model_error = error_msg.contains("Model file not found")
                                || error_msg.contains("Model not found")
                                || error_msg.contains("Download GGML models")
                                || error_msg.contains("Model download incomplete");

                            if retry_count == 0 {
                                error!("Failed to create dictation engine: {}", e);

                                // Only show permission hints for non-model errors
                                if !is_model_error {
                                    error!("⚠️  This is usually a permission issue. Please grant:");
                                    error!("   1. Input Monitoring permission");
                                    error!("   2. Accessibility permission");
                                    #[cfg(target_os = "macos")]
                                    error!("   Then restart: launchctl kickstart -k gui/$(id -u)/com.onevox.daemon");
                                    #[cfg(target_os = "linux")]
                                    error!("   Then restart: systemctl --user restart onevox");
                                    #[cfg(target_os = "windows")]
                                    error!("   Then restart: onevox stop && onevox daemon --foreground");
                                }
                            }

                            // Don't retry for model errors - they won't fix themselves
                            if is_model_error {
                                error!("❌ Cannot start without a valid model");
                                error!("   Daemon will continue running but dictation won't work");
                                error!("   Download a model and restart the daemon");
                                break;
                            }

                            retry_count += 1;
                            if retry_count >= max_retries {
                                error!("❌ Dictation engine failed after {} attempts", max_retries);
                                error!("   Daemon will continue running but hotkeys won't work");
                                error!("   Grant permissions and restart the daemon");
                                break;
                            }

                            // Wait before retry
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                            info!("🔄 Retrying dictation engine initialization ({}/{})", retry_count, max_retries);
                        }
                    }
                }
            });
        });

        // Initialize and start chat handler in the background (if chat enabled)
        let config_for_chat = self.config.clone();
        let state_clone_for_chat = Arc::clone(&self.state);

        let _chat_handle = if config_for_chat.chat.enabled {
            Some(std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new()
                    .expect("Failed to create tokio runtime for chat");
                rt.block_on(async {
                    info!("🤖 Chat feature enabled - initializing chat handler");

                    // Create command channel for IPC control
                    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel();

                    // Get is_chatting flag from state
                    let is_chatting = {
                        let state = state_clone_for_chat.read().await;
                        state.is_chatting_flag()
                    };

                    // Register the channel with state so IPC can send commands
                    {
                        let mut state = state_clone_for_chat.write().await;
                        state.set_chat_channel(cmd_tx);
                    }

                    // Try to initialize chat engine with retries
                    let mut retry_count = 0;
                    let max_retries = 3;

                    info!("Initializing chat engine (IPC handler)");
                    // Create a temporary lifecycle instance to access helper methods
                    let temp_lifecycle = Lifecycle {
                        config: config_for_chat.clone(),
                        state: Arc::clone(&state_clone_for_chat),
                    };

                    loop {
                        match temp_lifecycle.initialize_chat_engine().await {
                            Ok(chat_engine) => {
                                info!("✅ Chat engine initialized");
                                let chat_engine = Arc::new(chat_engine);

                                // Mark models as loaded
                                {
                                    let mut state = state_clone_for_chat.write().await;
                                    state.set_chat_models_loaded(true);
                                }

                                // Create chat handler with the engine
                                match ChatHandler::with_chatting_flag(
                                    config_for_chat.clone(),
                                    Arc::clone(&chat_engine),
                                    Arc::clone(&is_chatting),
                                ) {
                                    Ok(mut handler) => {
                                        // Start the hotkey listener in a background thread
                                        let config_for_hotkey = config_for_chat.clone();
                                        let chat_engine_for_hotkey = Arc::clone(&chat_engine);
                                        let is_chatting_for_hotkey = Arc::clone(&is_chatting);

                                        info!("Starting chat hotkey listener thread");
                                        std::thread::spawn(move || {
                                            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime for chat hotkey");
                                            rt.block_on(async {
                                                debug!("Initializing chat handler (hotkey handler)");
                                                match ChatHandler::with_chatting_flag(
                                                    config_for_hotkey,
                                                    chat_engine_for_hotkey,
                                                    is_chatting_for_hotkey,
                                                ) {
                                                    Ok(mut hotkey_handler) => {
                                                        if let Err(e) = hotkey_handler.start().await {
                                                            error!("Chat handler hotkey listener error: {}", e);
                                                        }
                                                    }
                                                    Err(e) => {
                                                        error!("Failed to create chat handler for hotkey listener: {}", e);
                                                    }
                                                }
                                            });
                                        });

                                        // Listen for IPC commands in the main loop
                                        while let Some(cmd) = cmd_rx.recv().await {
                                            match cmd {
                                                crate::daemon::state::ChatCommand::Start => {
                                                    info!("📡 IPC command: Start chat");
                                                    if let Err(e) = handler.start_chat().await {
                                                        error!("Failed to start chat: {}", e);
                                                    }
                                                }
                                                crate::daemon::state::ChatCommand::Stop => {
                                                    info!("📡 IPC command: Stop chat");
                                                    if let Err(e) = handler.stop_chat().await {
                                                        error!("Failed to stop chat: {}", e);
                                                    }
                                                }
                                                crate::daemon::state::ChatCommand::ClearHistory => {
                                                    info!("📡 IPC command: Clear chat history");
                                                    if let Err(e) = handler.clear_history().await {
                                                        error!("Failed to clear chat history: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to create chat handler: {}", e);
                                        error!("   Chat will be disabled for this session");
                                    }
                                }
                                break;
                            }
                            Err(e) => {
                                let error_msg = e.to_string();

                                // Check if this is a model-related error
                                let is_model_error = error_msg.contains("Model file not found")
                                    || error_msg.contains("Model not found")
                                    || error_msg.contains("llama-cpp feature not enabled")
                                    || error_msg.contains("LLM requires llama-cpp feature");

                                if retry_count == 0 {
                                    error!("Failed to create chat engine: {}", e);

                                    if is_model_error {
                                        error!("⚠️  Chat models missing or feature not enabled");
                                        error!("   Download chat models or rebuild with --features llama-cpp");
                                    }
                                }

                                // Don't retry for model/feature errors
                                if is_model_error {
                                    error!("❌ Cannot start chat without models");
                                    error!("   Daemon will continue running but chat won't work");
                                    error!("   Download models or rebuild, then restart daemon");
                                    break;
                                }

                                retry_count += 1;
                                if retry_count >= max_retries {
                                    error!("❌ Chat engine failed after {} attempts", max_retries);
                                    error!("   Daemon will continue running but chat won't work");
                                    break;
                                }

                                // Wait before retry
                                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                info!("🔄 Retrying chat engine initialization ({}/{})", retry_count, max_retries);
                            }
                        }
                    }
                });
            }))
        } else {
            info!("📵 Chat feature disabled in configuration");
            None
        };

        // Wait for shutdown signal
        tokio::select! {
            _ = self.wait_for_shutdown_signal() => {
                info!("Shutdown signal received");
            }
            _ = self.wait_for_state_shutdown() => {
                info!("Shutdown requested via IPC");
            }
        }

        // Cleanup
        info!("🛑 Shutting down daemon...");
        {
            let mut state = self.state.write().await;
            state.shutdown();
        }

        // Abort tasks
        ipc_handle.abort();
        // Note: dictation_handle will be cleaned up when the thread exits

        info!("✅ Daemon stopped");
        Ok(())
    }

    /// Wait for OS shutdown signal (SIGTERM, SIGINT)
    async fn wait_for_shutdown_signal(&self) {
        #[cfg(unix)]
        {
            let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to register SIGTERM handler");
            let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
                .expect("Failed to register SIGINT handler");

            tokio::select! {
                _ = sigterm.recv() => {
                    info!("Received SIGTERM");
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT");
                }
            }
        }

        #[cfg(not(unix))]
        {
            if let Err(e) = signal::ctrl_c().await {
                error!("Failed to wait for Ctrl+C: {}", e);
            }
            info!("Received Ctrl+C");
        }
    }

    /// Wait for shutdown request from state
    async fn wait_for_state_shutdown(&self) {
        loop {
            {
                let state = self.state.read().await;
                if state.is_shutdown_requested() {
                    break;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Check if daemon is already running
    async fn is_already_running(&self) -> bool {
        let mut client = IpcClient::default();
        client.ping().await.unwrap_or(false)
    }

    /// Stop the daemon (called from CLI)
    pub async fn stop() -> Result<()> {
        info!("Stopping daemon...");

        let mut client = IpcClient::default();

        match client.ping().await {
            Ok(true) => {
                client
                    .shutdown()
                    .await
                    .context("Failed to send shutdown command")?;
                info!("✅ Daemon shutdown command sent");
                Ok(())
            }
            Ok(false) => {
                warn!("Daemon is not responding");
                Err(anyhow::anyhow!("Daemon is not responding"))
            }
            Err(_) => {
                warn!("Daemon is not running");
                Err(anyhow::anyhow!("Daemon is not running"))
            }
        }
    }

    /// Get daemon status (called from CLI)
    pub async fn status() -> Result<crate::ipc::DaemonStatus> {
        let mut client = IpcClient::default();
        client
            .get_status()
            .await
            .context("Failed to get daemon status")
    }

    /// Load STT model (Whisper)
    fn load_stt_model(&self, model_config: &ModelConfig) -> Result<Box<dyn ModelRuntime>> {
        info!("Loading STT model: {}", model_config.model_path);

        // Auto-detect backend from model path
        let is_onnx_model = model_config.model_path.contains("parakeet")
            || model_config.model_path.ends_with(".onnx")
            || model_config.model_path.contains("onnx");

        let mut model: Box<dyn ModelRuntime> = if is_onnx_model {
            #[cfg(feature = "onnx")]
            {
                info!("Auto-detected ONNX model from path");
                Box::new(crate::models::OnnxRuntime::new()?)
            }
            #[cfg(not(feature = "onnx"))]
            {
                return Err(anyhow::anyhow!(
                    "ONNX model detected but feature not enabled"
                ));
            }
        } else {
            info!("Auto-detected GGML model from path");
            Box::new(WhisperCpp::new()?)
        };

        model.load(model_config.clone())?;
        info!("✅ STT model loaded");
        Ok(model)
    }

    /// Load LLM model (GGUF via llama.cpp)
    fn load_llm_model(&self, llm_config: &LlmConfig) -> Result<Box<dyn LlmRuntime>> {
        info!("Loading LLM model: {}", llm_config.model_path);

        #[cfg(feature = "llama-cpp")]
        {
            let mut llm: Box<dyn LlmRuntime> = Box::new(crate::models::LlmGguf::new()?);

            let runtime_config = LlmRuntimeConfig {
                model_path: llm_config.model_path.clone(),
                use_gpu: llm_config.device == "gpu" || llm_config.device == "auto",
                context_length: llm_config.context_length,
                temperature: llm_config.temperature,
                max_tokens: llm_config.max_tokens,
                top_p: 0.9,
                top_k: 40,
                repetition_penalty: 1.1,
            };

            llm.load(runtime_config)?;
            info!("✅ LLM model loaded");
            Ok(llm)
        }

        #[cfg(not(feature = "llama-cpp"))]
        {
            Err(anyhow::anyhow!("LLM requires llama-cpp feature"))
        }
    }

    /// Load TTS model (Kokoro ONNX)
    fn load_tts_model(&self, tts_config: &TtsConfig) -> Result<Box<dyn TtsRuntime>> {
        info!("Loading TTS model: {}", tts_config.model_path);

        let mut tts: Box<dyn TtsRuntime> = Box::new(crate::models::TtsKokoro::new());

        let runtime_config = TtsRuntimeConfig {
            model_path: tts_config.model_path.clone(),
            use_gpu: tts_config.device == "gpu" || tts_config.device == "auto",
            voice_id: tts_config.voice_id.clone(),
            speech_rate: tts_config.speech_rate,
            pitch: 0.0,
            volume: 1.0,
        };

        tts.load(runtime_config)?;
        info!("✅ TTS model loaded");
        Ok(tts)
    }

    /// Initialize ChatEngine with all 3 models (STT, LLM, TTS)
    async fn initialize_chat_engine(&self) -> Result<ChatEngine> {
        info!("🤖 Initializing Chat Engine");

        let config_guard = self.state.read().await;
        let config = config_guard.config().clone();
        drop(config_guard);

        // Load STT model (shared with dictation)
        let model_config = ModelConfig {
            model_path: config.model.model_path.clone(),
            use_gpu: config.model.device == "gpu" || config.model.device == "auto",
            ..Default::default()
        };
        let stt_model = self.load_stt_model(&model_config)?;
        let stt_model = Arc::new(RwLock::new(stt_model));

        // Load LLM model
        let llm_runtime = self.load_llm_model(&config.chat.llm)?;
        let llm_runtime = Arc::new(RwLock::new(llm_runtime));

        // Load TTS model
        let tts_runtime = self.load_tts_model(&config.chat.tts)?;
        let tts_runtime = Arc::new(RwLock::new(tts_runtime));

        // Create ChatEngine
        let config_arc = Arc::new(RwLock::new(config));
        let engine = ChatEngine::new(config_arc, stt_model, llm_runtime, tts_runtime)?;

        info!("✅ Chat Engine initialized");
        Ok(engine)
    }
}

/// Get the PID file path
pub fn pid_file_path() -> PathBuf {
    let base = crate::platform::paths::runtime_dir()
        .or_else(|_| crate::platform::paths::cache_dir())
        .unwrap_or_else(|_| {
            #[cfg(unix)]
            {
                PathBuf::from("/tmp").join("onevox")
            }
            #[cfg(windows)]
            {
                std::env::temp_dir().join("onevox")
            }
            #[cfg(not(any(unix, windows)))]
            {
                PathBuf::from("/tmp").join("onevox")
            }
        });

    base.join("onevox.pid")
}

/// Write PID file
pub fn write_pid_file() -> Result<()> {
    let pid = std::process::id();
    let pid_path = pid_file_path();

    if let Some(parent) = pid_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&pid_path, pid.to_string())?;
    info!("PID file written: {:?}", pid_path);
    Ok(())
}

/// Remove PID file
pub fn remove_pid_file() -> Result<()> {
    let pid_path = pid_file_path();
    if pid_path.exists() {
        std::fs::remove_file(&pid_path)?;
        info!("PID file removed");
    }
    Ok(())
}
