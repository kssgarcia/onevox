//! Daemon Lifecycle Management
//!
//! Handles daemon startup, shutdown, and lifecycle events.

use crate::config::Config;
use crate::daemon::dictation::DictationEngine;
use crate::daemon::state::DaemonState;
use crate::ipc::{IpcClient, IpcServer};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

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

                loop {
                    match DictationEngine::with_history(config.clone(), Arc::clone(&history_manager)) {
                        Ok(mut engine) => {
                            info!("✅ Dictation engine initialized");

                            // Start the engine's hotkey listener in a background thread
                            // This engine instance handles hotkey events
                            let config_for_hotkey = config.clone();
                            let history_for_hotkey = Arc::clone(&history_manager);
                            std::thread::spawn(move || {
                                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                                rt.block_on(async {
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
                            if retry_count == 0 {
                                error!("Failed to create dictation engine: {}", e);
                                error!("⚠️  This is usually a permission issue. Please grant:");
                                error!("   1. Input Monitoring permission");
                                error!("   2. Accessibility permission");
                                error!("   Then restart: launchctl kickstart -k gui/$(id -u)/com.onevox.daemon");
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
}

/// Get the PID file path
pub fn pid_file_path() -> PathBuf {
    IpcClient::default_socket_path()
        .parent()
        .map(|p| p.join("onevox.pid"))
        .unwrap_or_else(|| PathBuf::from("/tmp/onevox.pid"))
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
