//! Transcription History Management
//!
//! Tracks all transcriptions performed by the daemon, allowing users to:
//! - View past transcriptions
//! - Delete specific entries
//! - Clear all history
//! - Configure history retention

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// A single transcription history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique entry ID
    pub id: u64,

    /// Unix timestamp (seconds since epoch)
    pub timestamp: u64,

    /// Transcribed text
    pub text: String,

    /// Model used for transcription
    pub model: String,

    /// Duration of transcription in milliseconds
    pub duration_ms: u64,

    /// Confidence score (0.0 to 1.0), if available
    pub confidence: Option<f32>,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(text: String, model: String, duration_ms: u64, confidence: Option<f32>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: timestamp, // Use timestamp as ID for simplicity
            timestamp,
            text,
            model,
            duration_ms,
            confidence,
        }
    }
}

/// Manages transcription history
pub struct HistoryManager {
    /// History configuration
    config: crate::config::HistoryConfig,

    /// History file path
    history_path: PathBuf,

    /// In-memory history entries
    entries: Arc<Mutex<Vec<HistoryEntry>>>,
}

impl HistoryManager {
    /// Create a new history manager
    pub fn new(config: crate::config::HistoryConfig) -> crate::Result<Self> {
        let history_path = Self::default_history_path();

        // Create data directory if it doesn't exist
        if let Some(parent) = history_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                crate::Error::Other(format!("Failed to create history directory: {}", e))
            })?;
        }

        let mut manager = Self {
            config,
            history_path,
            entries: Arc::new(Mutex::new(Vec::new())),
        };

        // Load existing history
        if manager.config.enabled {
            if let Err(e) = manager.load() {
                warn!("Failed to load history: {}", e);
            }
        }

        Ok(manager)
    }

    /// Get the default history file path
    pub fn default_history_path() -> PathBuf {
        let data_dir = if let Ok(dir) = std::env::var("ONEVOX_DATA_DIR") {
            PathBuf::from(dir)
        } else if let Ok(dir) = std::env::var("VOX_DATA_DIR") {
            PathBuf::from(dir)
        } else {
            #[cfg(target_os = "windows")]
            {
                let appdata = std::env::var("APPDATA").unwrap_or_else(|_| {
                    dirs::home_dir()
                        .unwrap()
                        .join("AppData\\Roaming")
                        .to_string_lossy()
                        .to_string()
                });
                PathBuf::from(appdata).join("onevox")
            }

            #[cfg(target_os = "macos")]
            {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("Library/Application Support/onevox")
            }

            #[cfg(target_os = "linux")]
            {
                if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
                    PathBuf::from(xdg_data).join("onevox")
                } else {
                    dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join(".local/share/onevox")
                }
            }
        };

        data_dir.join("history.json")
    }

    /// Add a new entry to history
    pub fn add_entry(&self, entry: HistoryEntry) -> crate::Result<()> {
        if !self.config.enabled {
            debug!("History disabled, skipping entry");
            return Ok(());
        }

        let mut entries = self
            .entries
            .lock()
            .map_err(|e| crate::Error::Other(format!("Failed to lock history: {}", e)))?;

        entries.push(entry.clone());
        info!(
            "Added history entry #{}: {}",
            entry.id,
            if entry.text.len() > 50 {
                format!("{}...", &entry.text[..50])
            } else {
                entry.text.clone()
            }
        );

        // Enforce max_entries limit
        if self.config.max_entries > 0 && entries.len() > self.config.max_entries {
            let excess = entries.len() - self.config.max_entries;
            entries.drain(0..excess);
            debug!(
                "Removed {} old entries to maintain max_entries limit",
                excess
            );
        }

        // Auto-save if enabled
        if self.config.auto_save {
            drop(entries); // Release lock before saving
            self.save()?;
        }

        Ok(())
    }

    /// Get all history entries
    pub fn get_all(&self) -> crate::Result<Vec<HistoryEntry>> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| crate::Error::Other(format!("Failed to lock history: {}", e)))?;
        Ok(entries.clone())
    }

    /// Get a specific entry by ID
    pub fn get_entry(&self, id: u64) -> crate::Result<Option<HistoryEntry>> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| crate::Error::Other(format!("Failed to lock history: {}", e)))?;
        Ok(entries.iter().find(|e| e.id == id).cloned())
    }

    /// Delete a specific entry by ID
    pub fn delete_entry(&self, id: u64) -> crate::Result<bool> {
        if !self.config.enabled {
            return Err(crate::Error::Other("History is disabled".to_string()));
        }

        let mut entries = self
            .entries
            .lock()
            .map_err(|e| crate::Error::Other(format!("Failed to lock history: {}", e)))?;

        let original_len = entries.len();
        entries.retain(|e| e.id != id);
        let deleted = entries.len() < original_len;

        if deleted {
            info!("Deleted history entry #{}", id);
            drop(entries);
            self.save()?;
        } else {
            debug!("Entry #{} not found for deletion", id);
        }

        Ok(deleted)
    }

    /// Clear all history
    pub fn clear(&self) -> crate::Result<()> {
        if !self.config.enabled {
            return Err(crate::Error::Other("History is disabled".to_string()));
        }

        let mut entries = self
            .entries
            .lock()
            .map_err(|e| crate::Error::Other(format!("Failed to lock history: {}", e)))?;

        let count = entries.len();
        entries.clear();
        info!("Cleared {} history entries", count);

        drop(entries);
        self.save()?;

        Ok(())
    }

    /// Get the number of entries
    pub fn count(&self) -> usize {
        self.entries.lock().map(|e| e.len()).unwrap_or(0)
    }

    /// Load history from disk
    fn load(&mut self) -> crate::Result<()> {
        if !self.history_path.exists() {
            debug!("History file not found, starting with empty history");
            return Ok(());
        }

        let contents = fs::read_to_string(&self.history_path)
            .map_err(|e| crate::Error::Other(format!("Failed to read history file: {}", e)))?;

        let loaded_entries: Vec<HistoryEntry> = serde_json::from_str(&contents)
            .map_err(|e| crate::Error::Other(format!("Failed to parse history file: {}", e)))?;

        let mut entries = self
            .entries
            .lock()
            .map_err(|e| crate::Error::Other(format!("Failed to lock history: {}", e)))?;

        *entries = loaded_entries;
        info!(
            "Loaded {} history entries from {:?}",
            entries.len(),
            self.history_path
        );

        Ok(())
    }

    /// Save history to disk
    fn save(&self) -> crate::Result<()> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| crate::Error::Other(format!("Failed to lock history: {}", e)))?;

        let json = serde_json::to_string_pretty(&*entries)
            .map_err(|e| crate::Error::Other(format!("Failed to serialize history: {}", e)))?;

        fs::write(&self.history_path, json)
            .map_err(|e| crate::Error::Other(format!("Failed to write history file: {}", e)))?;

        debug!("Saved {} entries to {:?}", entries.len(), self.history_path);

        Ok(())
    }

    /// Manually save history (for when auto-save is disabled)
    pub fn manual_save(&self) -> crate::Result<()> {
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_entry_creation() {
        let entry = HistoryEntry::new(
            "Test transcription".to_string(),
            "whisper-base".to_string(),
            1500,
            Some(0.95),
        );

        assert_eq!(entry.text, "Test transcription");
        assert_eq!(entry.model, "whisper-base");
        assert_eq!(entry.duration_ms, 1500);
        assert_eq!(entry.confidence, Some(0.95));
        assert!(entry.timestamp > 0);
    }

    #[test]
    fn test_history_manager_add_and_get() {
        let config = crate::config::HistoryConfig {
            enabled: true,
            max_entries: 10,
            auto_save: false,
        };

        let manager = HistoryManager::new(config).unwrap();

        let entry = HistoryEntry::new("Test".to_string(), "whisper".to_string(), 1000, None);

        manager.add_entry(entry.clone()).unwrap();
        assert_eq!(manager.count(), 1);

        let entries = manager.get_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text, "Test");
    }

    #[test]
    fn test_history_manager_max_entries() {
        let config = crate::config::HistoryConfig {
            enabled: true,
            max_entries: 3,
            auto_save: false,
        };

        let manager = HistoryManager::new(config).unwrap();

        for i in 0..5 {
            let entry = HistoryEntry::new(format!("Test {}", i), "whisper".to_string(), 1000, None);
            manager.add_entry(entry).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure unique IDs
        }

        assert_eq!(manager.count(), 3);
    }

    #[test]
    fn test_delete_entry() {
        let config = crate::config::HistoryConfig {
            enabled: true,
            max_entries: 10,
            auto_save: false,
        };

        let manager = HistoryManager::new(config).unwrap();

        let entry = HistoryEntry::new("Test".to_string(), "whisper".to_string(), 1000, None);

        let id = entry.id;
        manager.add_entry(entry).unwrap();
        assert_eq!(manager.count(), 1);

        manager.delete_entry(id).unwrap();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_clear_history() {
        let config = crate::config::HistoryConfig {
            enabled: true,
            max_entries: 10,
            auto_save: false,
        };

        let manager = HistoryManager::new(config).unwrap();

        for i in 0..3 {
            let entry = HistoryEntry::new(format!("Test {}", i), "whisper".to_string(), 1000, None);
            manager.add_entry(entry).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        assert_eq!(manager.count(), 3);
        manager.clear().unwrap();
        assert_eq!(manager.count(), 0);
    }
}
