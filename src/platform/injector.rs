//! Text Injection
//!
//! Insert transcribed text into the active application.

use enigo::{Enigo, Keyboard, Settings};
use std::thread;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Text injector configuration
#[derive(Debug, Clone)]
pub struct InjectorConfig {
    /// Delay between keypresses in milliseconds
    pub key_delay_ms: u64,
    /// Delay before typing starts in milliseconds
    pub initial_delay_ms: u64,
}

impl Default for InjectorConfig {
    fn default() -> Self {
        Self {
            key_delay_ms: 10,
            initial_delay_ms: 50,
        }
    }
}

/// Text injector
#[derive(Clone)]
pub struct TextInjector {
    config: InjectorConfig,
}

impl TextInjector {
    /// Create a new text injector
    pub fn new(config: InjectorConfig) -> Self {
        Self { config }
    }

    /// Type text into the active application
    pub fn inject(&self, text: &str) -> crate::Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        info!("Injecting text: {} chars", text.len());
        debug!("Text: {}", text);

        // Wait a bit to ensure the app is ready
        if self.config.initial_delay_ms > 0 {
            thread::sleep(Duration::from_millis(self.config.initial_delay_ms));
        }

        // Try Wayland-specific tools first on Linux
        #[cfg(target_os = "linux")]
        {
            if std::env::var("WAYLAND_DISPLAY").is_ok() {
                // Try wtype first (most reliable on Wayland)
                if let Ok(result) = self.inject_with_wtype(text) {
                    return result;
                }

                // Fallback to ydotool
                if let Ok(result) = self.inject_with_ydotool(text) {
                    return result;
                }

                warn!("Wayland detected but no injection tools found. Install wtype or ydotool.");
            }
        }

        // Fallback to enigo for X11 and other platforms
        self.inject_with_enigo(text)
    }

    #[cfg(target_os = "linux")]
    fn inject_with_wtype(&self, text: &str) -> Result<crate::Result<()>, ()> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        // Check if wtype is available
        if Command::new("which")
            .arg("wtype")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| !s.success())
            .unwrap_or(true)
        {
            return Err(());
        }

        debug!("Using wtype for text injection");

        let mut child = Command::new("wtype")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| ())?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).map_err(|_| ())?;
        }

        let output = child.wait_with_output().map_err(|_| ())?;

        if output.status.success() {
            info!("Text injected successfully with wtype");
            Ok(Ok(()))
        } else {
            warn!(
                "wtype failed: {:?}",
                String::from_utf8_lossy(&output.stderr)
            );
            Err(())
        }
    }

    #[cfg(target_os = "linux")]
    fn inject_with_ydotool(&self, text: &str) -> Result<crate::Result<()>, ()> {
        use std::process::{Command, Stdio};

        // Check if ydotool is available
        if Command::new("which")
            .arg("ydotool")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| !s.success())
            .unwrap_or(true)
        {
            return Err(());
        }

        debug!("Using ydotool for text injection");

        let output = Command::new("ydotool")
            .arg("type")
            .arg(text)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|_| ())?;

        if output.status.success() {
            info!("Text injected successfully with ydotool");
            Ok(Ok(()))
        } else {
            warn!(
                "ydotool failed: {:?}",
                String::from_utf8_lossy(&output.stderr)
            );
            Err(())
        }
    }

    fn inject_with_enigo(&self, text: &str) -> crate::Result<()> {
        let settings = Settings::default();
        match Enigo::new(&settings) {
            Ok(mut enigo) => {
                enigo.text(text).map_err(|e| {
                    crate::Error::Platform(format!("Failed to inject text: {:?}", e))
                })?;

                // Small delay after typing
                if self.config.key_delay_ms > 0 {
                    thread::sleep(Duration::from_millis(self.config.key_delay_ms));
                }

                info!("Text injected successfully with enigo");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to create Enigo instance: {:?}", e);
                Err(crate::Error::Platform(format!(
                    "Failed to initialize text injector: {:?}",
                    e
                )))
            }
        }
    }
}

impl Default for TextInjector {
    fn default() -> Self {
        Self::new(InjectorConfig::default())
    }
}

/// Injection strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InjectionStrategy {
    /// Type text directly
    #[default]
    Type,
    /// Copy to clipboard and paste
    Clipboard,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injector_creation() {
        let injector = TextInjector::default();
        assert_eq!(injector.config.key_delay_ms, 10);
        assert_eq!(injector.config.initial_delay_ms, 50);
    }
}
