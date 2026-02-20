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

        // Create enigo instance and type the text
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

                info!("Text injected successfully");
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionStrategy {
    /// Type text directly
    Type,
    /// Copy to clipboard and paste
    Clipboard,
}

impl Default for InjectionStrategy {
    fn default() -> Self {
        Self::Type
    }
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
