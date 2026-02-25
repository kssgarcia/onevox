//! Daemon State Management
//!
//! Centralized state for the daemon process.

use crate::config::Config;
use crate::history::HistoryManager;
use crate::ipc::protocol::{DaemonState as State, DaemonStatus};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use sysinfo::{Pid, System};
use tokio::sync::mpsc;

/// Message types for dictation control
pub enum DictationCommand {
    Start,
    Stop,
}

/// Shared daemon state
pub struct DaemonState {
    /// Configuration
    config: Config,

    /// Process start time
    start_time: Instant,

    /// Process ID
    pid: u32,

    /// Current operational state
    state: State,

    /// Shutdown flag
    shutdown_requested: Arc<AtomicBool>,

    /// Is model loaded
    model_loaded: bool,

    /// Current model name
    model_name: Option<String>,

    /// Is currently dictating
    is_dictating: Arc<AtomicBool>,

    /// System info provider
    sys_info: Mutex<System>,

    /// History manager
    history_manager: Arc<HistoryManager>,

    /// Channel to send commands to dictation engine
    dictation_tx: Option<mpsc::UnboundedSender<DictationCommand>>,
}

impl DaemonState {
    /// Create a new daemon state
    pub fn new(config: Config) -> Self {
        // Create history manager (using sync constructor, will not load from disk)
        let history_config = config.history.clone();
        let history_manager = HistoryManager::new(history_config).unwrap_or_else(|e| {
            tracing::warn!("Failed to create history manager: {}", e);
            // Create a disabled history manager as fallback
            HistoryManager::new(crate::config::HistoryConfig {
                enabled: false,
                max_entries: 0,
                auto_save: false,
            })
            .expect("Failed to create fallback history manager")
        });

        let pid = std::process::id();
        let mut sys_info = System::new_all();

        // Initial refresh for current process
        sys_info.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        Self {
            config,
            start_time: Instant::now(),
            pid,
            state: State::Starting,
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            model_loaded: false,
            model_name: None,
            is_dictating: Arc::new(AtomicBool::new(false)),
            sys_info: Mutex::new(sys_info),
            history_manager: Arc::new(history_manager),
            dictation_tx: None,
        }
    }

    /// Create a new daemon state with async initialization (recommended)
    pub async fn new_async(config: Config) -> Self {
        // Create history manager with async initialization
        let history_config = config.history.clone();
        let history_manager = HistoryManager::new_async(history_config)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to create history manager: {}", e);
                // Create a disabled history manager as fallback (sync version is fine for fallback)
                HistoryManager::new(crate::config::HistoryConfig {
                    enabled: false,
                    max_entries: 0,
                    auto_save: false,
                })
                .expect("Failed to create fallback history manager")
            });

        let pid = std::process::id();
        let mut sys_info = System::new_all();

        // Initial refresh for current process
        sys_info.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        Self {
            config,
            start_time: Instant::now(),
            pid,
            state: State::Starting,
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            model_loaded: false,
            model_name: None,
            is_dictating: Arc::new(AtomicBool::new(false)),
            sys_info: Mutex::new(sys_info),
            history_manager: Arc::new(history_manager),
            dictation_tx: None,
        }
    }

    /// Get current status
    pub fn status(&self) -> DaemonStatus {
        let uptime_secs = self.start_time.elapsed().as_secs();

        DaemonStatus {
            version: env!("CARGO_PKG_VERSION").to_string(),
            pid: self.pid,
            uptime_secs,
            state: self.state,
            model_loaded: self.model_loaded,
            model_name: self.model_name.clone(),
            is_dictating: self.is_dictating.load(Ordering::SeqCst),
            memory_usage_bytes: self.get_memory_usage(),
            cpu_usage_percent: self.get_cpu_usage(),
        }
    }

    /// Get configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Set state
    pub fn set_state(&mut self, state: State) {
        tracing::info!("State transition: {} -> {}", self.state, state);
        self.state = state;
    }

    /// Get current state
    pub fn state(&self) -> State {
        self.state
    }

    /// Mark daemon as ready
    pub fn set_ready(&mut self) {
        self.set_state(State::Idle);
    }

    /// Mark daemon as active
    pub fn set_active(&mut self) {
        self.set_state(State::Active);
    }

    /// Mark daemon as error
    pub fn set_error(&mut self) {
        self.set_state(State::Error);
    }

    /// Request shutdown
    pub fn shutdown(&mut self) {
        self.set_state(State::ShuttingDown);
        self.shutdown_requested.store(true, Ordering::SeqCst);
    }

    /// Check if shutdown is requested
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

    /// Get shutdown signal for cloning
    pub fn shutdown_signal(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.shutdown_requested)
    }

    /// Set model loaded state
    pub fn set_model_loaded(&mut self, name: Option<String>) {
        self.model_loaded = name.is_some();
        self.model_name = name;
    }

    /// Set dictating state
    pub fn set_dictating(&mut self, is_dictating: bool) {
        self.is_dictating.store(is_dictating, Ordering::SeqCst);
        if is_dictating {
            self.set_active();
        } else {
            self.set_ready();
        }
    }

    /// Set dictation command channel
    pub fn set_dictation_channel(&mut self, tx: mpsc::UnboundedSender<DictationCommand>) {
        self.dictation_tx = Some(tx);
    }

    /// Start dictation via IPC
    pub fn start_dictation(&self) -> crate::Result<()> {
        if let Some(tx) = &self.dictation_tx {
            tx.send(DictationCommand::Start)
                .map_err(|_| crate::Error::Other("Dictation engine not available".to_string()))?;
            Ok(())
        } else {
            Err(crate::Error::Other(
                "Dictation engine not initialized".to_string(),
            ))
        }
    }

    /// Stop dictation via IPC
    pub fn stop_dictation(&self) -> crate::Result<()> {
        if let Some(tx) = &self.dictation_tx {
            tx.send(DictationCommand::Stop)
                .map_err(|_| crate::Error::Other("Dictation engine not available".to_string()))?;
            Ok(())
        } else {
            Err(crate::Error::Other(
                "Dictation engine not initialized".to_string(),
            ))
        }
    }

    /// Get is_dictating flag for sharing with dictation engine
    pub fn is_dictating_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_dictating)
    }

    /// Get memory usage in bytes
    fn get_memory_usage(&self) -> u64 {
        let pid = Pid::from_u32(self.pid);
        let mut sys_info = self.sys_info.lock();

        sys_info.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), false);

        if let Some(process) = sys_info.process(pid) {
            process.memory()
        } else {
            tracing::warn!("Failed to get process memory usage");
            0
        }
    }

    /// Get CPU usage percentage
    fn get_cpu_usage(&self) -> f32 {
        let pid = Pid::from_u32(self.pid);
        let mut sys_info = self.sys_info.lock();

        sys_info.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), false);

        if let Some(process) = sys_info.process(pid) {
            process.cpu_usage()
        } else {
            tracing::warn!("Failed to get process CPU usage");
            0.0
        }
    }

    /// Reload configuration
    pub fn reload_config(&mut self) -> crate::Result<()> {
        let new_config = Config::load_default()?;
        self.config = new_config;
        tracing::info!("Configuration reloaded - daemon will be restarted to apply changes");
        Ok(())
    }

    /// Get reference to history manager
    pub fn history_manager(&self) -> &Arc<HistoryManager> {
        &self.history_manager
    }
}
