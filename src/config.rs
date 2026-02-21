//! Configuration management
//!
//! Handles loading, validation, and hot-reloading of configuration.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub hotkey: HotkeyConfig,
    pub audio: AudioConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub vad: VadConfig,
    #[serde(default)]
    pub model: ModelConfig,
    #[serde(default)]
    pub post_processing: PostProcessingConfig,
    #[serde(default)]
    pub injection: InjectionConfig,
    #[serde(default)]
    pub history: HistoryConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub recording_overlay: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadConfig {
    pub enabled: bool,
    pub backend: String,
    pub threshold: f32,
    pub pre_roll_ms: u32,
    pub post_roll_ms: u32,
    pub min_speech_chunks: usize,
    pub min_silence_chunks: usize,
    pub adaptive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub backend: String,
    pub model_path: String,
    pub device: String,
    pub language: String,
    pub task: String,
    pub preload: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostProcessingConfig {
    pub auto_punctuation: bool,
    pub auto_capitalize: bool,
    pub remove_filler_words: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionConfig {
    pub method: String,
    pub paste_delay_ms: u32,
    #[serde(default = "default_focus_settle_ms")]
    pub focus_settle_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    pub enabled: bool,
    pub max_entries: usize,
    pub auto_save: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            daemon: DaemonConfig {
                auto_start: true,
                log_level: "info".to_string(),
            },
            hotkey: HotkeyConfig {
                trigger: "Cmd+Shift+0".to_string(),
                mode: "push-to-talk".to_string(),
            },
            audio: AudioConfig {
                device: "default".to_string(),
                sample_rate: 16000,
                chunk_duration_ms: 200,
            },
            ui: UiConfig::default(),
            vad: VadConfig::default(),
            model: ModelConfig::default(),
            post_processing: PostProcessingConfig::default(),
            injection: InjectionConfig::default(),
            history: HistoryConfig::default(),
        }
    }
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "energy".to_string(),
            threshold: 0.02,
            pre_roll_ms: 300,
            post_roll_ms: 500,
            min_speech_chunks: 2,
            min_silence_chunks: 3,
            adaptive: true,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            recording_overlay: true,
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            backend: "whisper_cpp".to_string(),
            model_path: "ggml-base.en.bin".to_string(),
            device: "auto".to_string(),
            language: "en".to_string(),
            task: "transcribe".to_string(),
            preload: true,
        }
    }
}

impl Default for PostProcessingConfig {
    fn default() -> Self {
        Self {
            auto_punctuation: true,
            auto_capitalize: true,
            remove_filler_words: false,
        }
    }
}

impl Default for InjectionConfig {
    fn default() -> Self {
        Self {
            method: "accessibility".to_string(),
            paste_delay_ms: 50,
            focus_settle_ms: default_focus_settle_ms(),
        }
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 1000,
            auto_save: true,
        }
    }
}

fn default_focus_settle_ms() -> u32 {
    80
}

impl Config {
    /// Load configuration from file
    pub fn load(path: &PathBuf) -> crate::Result<Self> {
        if !path.exists() {
            tracing::warn!("Config file not found at {:?}, using defaults", path);
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)
            .map_err(|e| crate::Error::Config(format!("Failed to read config: {}", e)))?;

        let config: Config = toml::from_str(&contents)
            .map_err(|e| crate::Error::Config(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> crate::Result<()> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| crate::Error::Config(format!("Failed to serialize config: {}", e)))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| crate::Error::Config(format!("Failed to create config dir: {}", e)))?;
        }

        fs::write(path, contents)
            .map_err(|e| crate::Error::Config(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    /// Get default config path
    pub fn default_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("onevox");

        config_dir.join("config.toml")
    }

    /// Load from default location
    pub fn load_default() -> crate::Result<Self> {
        Self::load(&Self::default_path())
    }

    /// Save to default location
    pub fn save_default(&self) -> crate::Result<()> {
        self.save(&Self::default_path())
    }
}

impl VadConfig {
    /// Convert to EnergyVadConfig
    pub fn to_energy_vad_config(&self) -> crate::vad::EnergyVadConfig {
        crate::vad::EnergyVadConfig {
            threshold: self.threshold,
            min_speech_chunks: self.min_speech_chunks,
            min_silence_chunks: self.min_silence_chunks,
            adaptive: self.adaptive,
            adaptive_window_size: 30,
        }
    }

    /// Convert to VadProcessorConfig
    pub fn to_processor_config(&self) -> crate::vad::VadProcessorConfig {
        crate::vad::VadProcessorConfig {
            pre_roll_ms: self.pre_roll_ms,
            post_roll_ms: self.post_roll_ms,
        }
    }
}
