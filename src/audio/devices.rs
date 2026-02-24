//! Audio Device Enumeration
//!
//! Handles listing and selecting audio input devices.

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host, SupportedStreamConfig};
use std::fmt;

/// Audio device information
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub is_default: bool,
    pub sample_rate: u32,
    pub channels: u16,
}

impl fmt::Display for AudioDeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{} - {}Hz, {} ch",
            self.name,
            if self.is_default { " (default)" } else { "" },
            self.sample_rate,
            self.channels
        )
    }
}

/// Audio device manager
pub struct AudioDeviceManager {
    host: Host,
}

impl AudioDeviceManager {
    /// Create a new device manager
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    /// List all available input devices
    pub fn list_input_devices(&self) -> crate::Result<Vec<AudioDeviceInfo>> {
        let default_device = self.host.default_input_device();
        let default_name = default_device
            .as_ref()
            .and_then(|d| d.name().ok())
            .unwrap_or_default();

        let mut devices = Vec::new();

        for device in self
            .host
            .input_devices()
            .map_err(|e| crate::Error::Audio(format!("Failed to enumerate devices: {}", e)))?
        {
            let name = device
                .name()
                .unwrap_or_else(|_| "Unknown Device".to_string());

            let is_default = name == default_name;

            // Get default config
            let config = device
                .default_input_config()
                .map_err(|e| crate::Error::Audio(format!("Failed to get device config: {}", e)))?;

            devices.push(AudioDeviceInfo {
                name,
                is_default,
                sample_rate: config.sample_rate().0,
                channels: config.channels(),
            });
        }

        Ok(devices)
    }

    /// Get the default input device
    pub fn default_input_device(&self) -> crate::Result<Device> {
        self.host
            .default_input_device()
            .ok_or_else(|| crate::Error::Audio("No default input device found".to_string()))
    }

    /// Get device by name, with fallback to default
    pub fn get_device_by_name(&self, name: &str) -> crate::Result<Device> {
        // Try to find the exact device
        for device in self
            .host
            .input_devices()
            .map_err(|e| crate::Error::Audio(format!("Failed to enumerate devices: {}", e)))?
        {
            if let Ok(device_name) = device.name()
                && device_name == name
            {
                return Ok(device);
            }
        }

        // Device not found - try default as fallback
        tracing::warn!(
            "Device '{}' not found, falling back to default input device",
            name
        );
        self.default_input_device()
    }

    /// Get device config
    pub fn get_device_config(&self, device: &Device) -> crate::Result<SupportedStreamConfig> {
        device
            .default_input_config()
            .map_err(|e| crate::Error::Audio(format!("Failed to get device config: {}", e)))
    }
}

impl Default for AudioDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}
