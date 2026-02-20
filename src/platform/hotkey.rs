//! Global Hotkey Management
//!
//! System-wide hotkey registration and handling for push-to-talk.

use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

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

    /// Convert to global-hotkey HotKey
    fn to_hotkey(&self) -> crate::Result<HotKey> {
        // Parse modifiers
        let mut mods = Modifiers::empty();
        for modifier in &self.modifiers {
            match modifier.to_lowercase().as_str() {
                "cmd" | "super" | "meta" => mods |= Modifiers::SUPER,
                "shift" => mods |= Modifiers::SHIFT,
                "alt" | "option" => mods |= Modifiers::ALT,
                "ctrl" | "control" => mods |= Modifiers::CONTROL,
                _ => {
                    warn!("Unknown modifier: {}", modifier);
                }
            }
        }

        // Parse key code
        let code = self.parse_key_code(&self.key)?;

        Ok(HotKey::new(Some(mods), code))
    }

    /// Parse key code from string
    fn parse_key_code(&self, key: &str) -> crate::Result<Code> {
        let code = match key.to_lowercase().as_str() {
            "space" => Code::Space,
            "enter" | "return" => Code::Enter,
            "tab" => Code::Tab,
            "backspace" => Code::Backspace,
            "escape" | "esc" => Code::Escape,
            "delete" => Code::Delete,
            "home" => Code::Home,
            "end" => Code::End,
            "pageup" => Code::PageUp,
            "pagedown" => Code::PageDown,
            "insert" => Code::Insert,
            "left" => Code::ArrowLeft,
            "right" => Code::ArrowRight,
            "up" => Code::ArrowUp,
            "down" => Code::ArrowDown,
            // Function keys
            "f1" => Code::F1,
            "f2" => Code::F2,
            "f3" => Code::F3,
            "f4" => Code::F4,
            "f5" => Code::F5,
            "f6" => Code::F6,
            "f7" => Code::F7,
            "f8" => Code::F8,
            "f9" => Code::F9,
            "f10" => Code::F10,
            "f11" => Code::F11,
            "f12" => Code::F12,
            // Letters
            "a" => Code::KeyA,
            "b" => Code::KeyB,
            "c" => Code::KeyC,
            "d" => Code::KeyD,
            "e" => Code::KeyE,
            "f" => Code::KeyF,
            "g" => Code::KeyG,
            "h" => Code::KeyH,
            "i" => Code::KeyI,
            "j" => Code::KeyJ,
            "k" => Code::KeyK,
            "l" => Code::KeyL,
            "m" => Code::KeyM,
            "n" => Code::KeyN,
            "o" => Code::KeyO,
            "p" => Code::KeyP,
            "q" => Code::KeyQ,
            "r" => Code::KeyR,
            "s" => Code::KeyS,
            "t" => Code::KeyT,
            "u" => Code::KeyU,
            "v" => Code::KeyV,
            "w" => Code::KeyW,
            "x" => Code::KeyX,
            "y" => Code::KeyY,
            "z" => Code::KeyZ,
            // Numbers
            "0" => Code::Digit0,
            "1" => Code::Digit1,
            "2" => Code::Digit2,
            "3" => Code::Digit3,
            "4" => Code::Digit4,
            "5" => Code::Digit5,
            "6" => Code::Digit6,
            "7" => Code::Digit7,
            "8" => Code::Digit8,
            "9" => Code::Digit9,
            _ => {
                return Err(crate::Error::Config(format!("Unknown key: {}", key)));
            }
        };

        Ok(code)
    }
}

/// Global hotkey manager
pub struct HotkeyManager {
    manager: Arc<GlobalHotKeyManager>,
    hotkey: Option<HotKey>,
    event_tx: Option<mpsc::UnboundedSender<HotkeyEvent>>,
}

impl HotkeyManager {
    /// Create a new hotkey manager
    pub fn new() -> crate::Result<Self> {
        let manager = GlobalHotKeyManager::new().map_err(|e| {
            crate::Error::Platform(format!("Failed to create hotkey manager: {}", e))
        })?;

        Ok(Self {
            manager: Arc::new(manager),
            hotkey: None,
            event_tx: None,
        })
    }

    /// Register a global hotkey
    pub fn register(
        &mut self,
        config: HotkeyConfig,
    ) -> crate::Result<mpsc::UnboundedReceiver<HotkeyEvent>> {
        info!("Registering hotkey: {:?}", config);

        // Convert config to HotKey
        let hotkey = config.to_hotkey()?;

        // Register the hotkey
        self.manager
            .register(hotkey)
            .map_err(|e| crate::Error::Platform(format!("Failed to register hotkey: {}", e)))?;

        self.hotkey = Some(hotkey);

        // Create event channel
        let (tx, rx) = mpsc::unbounded_channel();
        self.event_tx = Some(tx);

        info!("Hotkey registered successfully");

        Ok(rx)
    }

    /// Start listening for hotkey events
    pub fn start_listener(&self) -> crate::Result<()> {
        let tx = self
            .event_tx
            .as_ref()
            .ok_or_else(|| crate::Error::Platform("No hotkey registered".to_string()))?
            .clone();

        let hotkey_id = self
            .hotkey
            .as_ref()
            .ok_or_else(|| crate::Error::Platform("No hotkey registered".to_string()))?
            .id();

        // Spawn event listener thread
        std::thread::spawn(move || {
            debug!("Hotkey listener thread started");

            loop {
                if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    if event.id() == hotkey_id {
                        let hotkey_event = match event.state() {
                            HotKeyState::Pressed => HotkeyEvent::Pressed,
                            HotKeyState::Released => HotkeyEvent::Released,
                        };

                        debug!("Hotkey event: {:?}", hotkey_event);

                        if tx.send(hotkey_event).is_err() {
                            error!("Failed to send hotkey event, receiver dropped");
                            break;
                        }
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            debug!("Hotkey listener thread stopped");
        });

        Ok(())
    }

    /// Unregister the current hotkey
    pub fn unregister(&mut self) -> crate::Result<()> {
        if let Some(hotkey) = self.hotkey.take() {
            info!("Unregistering hotkey");
            self.manager.unregister(hotkey).map_err(|e| {
                crate::Error::Platform(format!("Failed to unregister hotkey: {}", e))
            })?;
        }
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
        let hotkey = config.to_hotkey().unwrap();
        assert_eq!(hotkey.id(), hotkey.id()); // Just verify it was created
    }
}
