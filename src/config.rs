//! Configuration management
//!
//! Handles loading, validation, and hot-reloading of configuration.

use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub hotkey: HotkeyConfig,
    pub audio: AudioConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub auto_start: bool,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub trigger: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub device: String,
    pub sample_rate: u32,
    pub chunk_duration_ms: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            daemon: DaemonConfig {
                auto_start: true,
                log_level: "info".to_string(),
            },
            hotkey: HotkeyConfig {
                trigger: "Cmd+Shift+Space".to_string(),
                mode: "push-to-talk".to_string(),
            },
            audio: AudioConfig {
                device: "default".to_string(),
                sample_rate: 16000,
                chunk_duration_ms: 200,
            },
        }
    }
}
