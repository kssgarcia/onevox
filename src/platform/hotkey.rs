//! Global Hotkey Management
//!
//! System-wide hotkey registration and handling for push-to-talk.

use handy_keys::{Hotkey as HandyHotkey, HotkeyManager as HandyHotkeyManager, Key, Modifiers};

use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Hotkey event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    /// Hotkey pressed
    Pressed,
    /// Hotkey released
    Released,
}

/// Hotkey configuration
#[derive(Debug, Clone)]
pub struct HotkeyConfig {
    /// Modifiers (Cmd, Shift, Alt, Ctrl)
    pub modifiers: Vec<String>,
    /// Key code
    pub key: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        // Platform-specific default hotkeys
        #[cfg(target_os = "macos")]
        let default_hotkey = "Cmd+Shift+0";
        
        #[cfg(target_os = "linux")]
        let default_hotkey = "Ctrl+Shift+Space";
        
        #[cfg(target_os = "windows")]
        let default_hotkey = "Ctrl+Shift+Space";
        
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        let default_hotkey = "Ctrl+Shift+Space";
        
        Self {
            modifiers: vec!["Cmd".to_string(), "Shift".to_string()],
            key: "Space".to_string(),
        }
    }
}

impl HotkeyConfig {
    /// Parse from string (e.g., "Cmd+Shift+Space")
    pub fn from_string(s: &str) -> crate::Result<Self> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return Err(crate::Error::Config("Empty hotkey string".to_string()));
        }

        let key = parts
            .last()
            .ok_or_else(|| crate::Error::Config("No key specified".to_string()))?
            .to_string();

        let modifiers = parts[..parts.len() - 1]
            .iter()
            .map(|s| s.to_string())
            .collect();

        Ok(Self { modifiers, key })
    }

    /// Convert to handy-keys Hotkey
    fn to_hotkey(&self) -> crate::Result<HandyHotkey> {
        // Parse modifiers
        let mut mods = Modifiers::empty();
        for modifier in &self.modifiers {
            match modifier.to_lowercase().as_str() {
                "cmd" | "super" | "meta" => mods |= Modifiers::CMD,
                "shift" => mods |= Modifiers::SHIFT,
                "alt" | "option" => mods |= Modifiers::OPT,
                "ctrl" | "control" => mods |= Modifiers::CTRL,
                _ => {
                    warn!("Unknown modifier: {}", modifier);
                }
            }
        }

        // Parse key code
        let key = self.parse_key(&self.key)?;

        HandyHotkey::new(mods, key)
            .map_err(|e| crate::Error::Platform(format!("Failed to create hotkey: {}", e)))
    }

    /// Parse key from string
    fn parse_key(&self, key_str: &str) -> crate::Result<Key> {
        let key = match key_str.to_lowercase().as_str() {
            "space" => Key::Space,
            "enter" | "return" => Key::Return,
            "tab" => Key::Tab,
            "escape" | "esc" => Key::Escape,
            "delete" => Key::Delete,
            "forwarddelete" => Key::ForwardDelete,
            "home" => Key::Home,
            "end" => Key::End,
            "pageup" => Key::PageUp,
            "pagedown" => Key::PageDown,
            "left" => Key::LeftArrow,
            "right" => Key::RightArrow,
            "up" => Key::UpArrow,
            "down" => Key::DownArrow,
            // Function keys
            "f1" => Key::F1,
            "f2" => Key::F2,
            "f3" => Key::F3,
            "f4" => Key::F4,
            "f5" => Key::F5,
            "f6" => Key::F6,
            "f7" => Key::F7,
            "f8" => Key::F8,
            "f9" => Key::F9,
            "f10" => Key::F10,
            "f11" => Key::F11,
            "f12" => Key::F12,
            "f13" => Key::F13,
            "f14" => Key::F14,
            "f15" => Key::F15,
            "f16" => Key::F16,
            "f17" => Key::F17,
            "f18" => Key::F18,
            "f19" => Key::F19,
            "f20" => Key::F20,
            // Letters
            "a" => Key::A,
            "b" => Key::B,
            "c" => Key::C,
            "d" => Key::D,
            "e" => Key::E,
            "f" => Key::F,
            "g" => Key::G,
            "h" => Key::H,
            "i" => Key::I,
            "j" => Key::J,
            "k" => Key::K,
            "l" => Key::L,
            "m" => Key::M,
            "n" => Key::N,
            "o" => Key::O,
            "p" => Key::P,
            "q" => Key::Q,
            "r" => Key::R,
            "s" => Key::S,
            "t" => Key::T,
            "u" => Key::U,
            "v" => Key::V,
            "w" => Key::W,
            "x" => Key::X,
            "y" => Key::Y,
            "z" => Key::Z,
            // Numbers
            "0" => Key::Num0,
            "1" => Key::Num1,
            "2" => Key::Num2,
            "3" => Key::Num3,
            "4" => Key::Num4,
            "5" => Key::Num5,
            "6" => Key::Num6,
            "7" => Key::Num7,
            "8" => Key::Num8,
            "9" => Key::Num9,
            _ => {
                return Err(crate::Error::Config(format!("Unknown key: {}", key_str)));
            }
        };

        Ok(key)
    }
}

/// Global hotkey manager
pub struct HotkeyManager {
    manager: HandyHotkeyManager,
    event_tx: Option<mpsc::UnboundedSender<HotkeyEvent>>,
    listener_handle: Option<std::thread::JoinHandle<()>>,
}

impl HotkeyManager {
    /// Create a new hotkey manager
    pub fn new() -> crate::Result<Self> {
        let manager = HandyHotkeyManager::new().map_err(|e| {
            crate::Error::Platform(format!("Failed to create hotkey manager: {}", e))
        })?;

        Ok(Self {
            manager,
            event_tx: None,
            listener_handle: None,
        })
    }

    /// Register a global hotkey
    pub fn register(
        &mut self,
        config: HotkeyConfig,
    ) -> crate::Result<mpsc::UnboundedReceiver<HotkeyEvent>> {
        info!("Registering hotkey: {:?}", config);

        // Convert config to HandyHotkey
        let hotkey = config.to_hotkey()?;

        // Register the hotkey
        self.manager
            .register(hotkey)
            .map_err(|e| crate::Error::Platform(format!("Failed to register hotkey: {}", e)))?;

        // Create event channel
        let (tx, rx) = mpsc::unbounded_channel();
        self.event_tx = Some(tx);

        info!("Hotkey registered successfully");

        Ok(rx)
    }

    /// Start listening for hotkey events
    ///
    /// Note: This consumes self because HotkeyManager needs to be moved into the listener thread
    pub fn start_listener(mut self) -> crate::Result<()> {
        let tx = self
            .event_tx
            .take()
            .ok_or_else(|| crate::Error::Platform("No hotkey registered".to_string()))?;

        // Spawn event listener thread - move the manager into it
        let handle = std::thread::spawn(move || {
            loop {
                // Use blocking recv to wait for events
                match self.manager.recv() {
                    Ok(event) => {
                        let hotkey_event = match event.state {
                            handy_keys::HotkeyState::Pressed => HotkeyEvent::Pressed,
                            handy_keys::HotkeyState::Released => HotkeyEvent::Released,
                        };

                        if tx.send(hotkey_event).is_err() {
                            error!("Failed to send hotkey event, receiver dropped");
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error receiving hotkey event: {:?}", e);
                        break;
                    }
                }
            }
        });

        info!("Hotkey listener started");

        Ok(())
    }

    /// Unregister all hotkeys
    pub fn unregister(&mut self) -> crate::Result<()> {
        info!("Unregistering hotkeys");
        // handy-keys automatically unregisters on drop
        Ok(())
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        let _ = self.unregister();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_config_parse() {
        let config = HotkeyConfig::from_string("Cmd+Shift+Space").unwrap();
        assert_eq!(config.modifiers.len(), 2);
        assert_eq!(config.key, "Space");
    }

    #[test]
    fn test_hotkey_config_to_hotkey() {
        let config = HotkeyConfig::default();
        let hotkey = config.to_hotkey();
        assert!(hotkey.is_ok());
    }
}
