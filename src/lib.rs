// Onevox - Local Speech-to-Text Daemon
// Main library entry point

#![warn(clippy::all)]
#![allow(dead_code, unused_variables)]

pub mod audio;
pub mod config;
pub mod daemon;
pub mod ipc;
pub mod models;
pub mod platform;
pub mod vad;

#[cfg(feature = "tui")]
pub mod tui;

// Re-export commonly used types
pub use config::Config;
pub use daemon::Daemon;

/// Result type alias for onevox operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for onevox
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("VAD error: {0}")]
    Vad(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}
