//! Daemon core
//!
//! Main daemon process and lifecycle management.

pub mod dictation;
pub mod lifecycle;
pub mod state;

// Re-export commonly used types
pub use dictation::DictationEngine;
pub use lifecycle::Lifecycle;
pub use state::DaemonState;

/// Main Daemon struct - wrapper around lifecycle
pub struct Daemon {
    lifecycle: Lifecycle,
}

impl Daemon {
    /// Create a new daemon with configuration
    pub fn new(config: crate::Config) -> Self {
        Self {
            lifecycle: Lifecycle::new(config),
        }
    }

    /// Create a new daemon with async initialization (recommended)
    pub async fn new_async(config: crate::Config) -> Self {
        Self {
            lifecycle: Lifecycle::new_async(config).await,
        }
    }

    /// Start the daemon
    pub async fn start(&mut self) -> crate::Result<()> {
        self.lifecycle
            .start()
            .await
            .map_err(|e| crate::Error::Other(e.to_string()))
    }

    /// Stop the daemon (static method for CLI)
    pub async fn stop() -> crate::Result<()> {
        Lifecycle::stop()
            .await
            .map_err(|e| crate::Error::Other(e.to_string()))
    }

    /// Get daemon status (static method for CLI)
    pub async fn status() -> crate::Result<crate::ipc::DaemonStatus> {
        Lifecycle::status()
            .await
            .map_err(|e| crate::Error::Other(e.to_string()))
    }
}
