//! Daemon State Management
//!
//! Centralized state for the daemon process.

use crate::config::Config;
use crate::history::HistoryManager;
use crate::ipc::protocol::{DaemonState as State, DaemonStatus};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use sysinfo::System;

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
    is_dictating: bool,

    /// System info provider
    sys_info: System,

    /// History manager
    history_manager: Arc<HistoryManager>,
}

impl DaemonState {
    /// Create a new daemon state
    pub fn new(config: Config) -> Self {
        // Create history manager
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

        Self {
            config,
            start_time: Instant::now(),
            pid: std::process::id(),
            state: State::Starting,
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            model_loaded: false,
            model_name: None,
            is_dictating: false,
            sys_info: System::new(),
            history_manager: Arc::new(history_manager),
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
            is_dictating: self.is_dictating,
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
        self.is_dictating = is_dictating;
        if is_dictating {
            self.set_active();
        } else {
            self.set_ready();
        }
    }

    /// Get memory usage in bytes
    fn get_memory_usage(&self) -> u64 {
        // Use sysinfo to get process memory
        // For now, return a placeholder
        // TODO: Implement actual memory tracking
        0
    }

    /// Get CPU usage percentage
    fn get_cpu_usage(&self) -> f32 {
        // Use sysinfo to get CPU usage
        // For now, return a placeholder
        // TODO: Implement actual CPU tracking
        0.0
    }

    /// Reload configuration
    pub fn reload_config(&mut self) -> crate::Result<()> {
        let new_config = Config::load_default()?;
        self.config = new_config;
        tracing::info!("Configuration reloaded");
        Ok(())
    }

    /// Get reference to history manager
    pub fn history_manager(&self) -> &Arc<HistoryManager> {
        &self.history_manager
    }
}
