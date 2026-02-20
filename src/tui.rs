//! Terminal User Interface
//!
//! Interactive TUI for monitoring and configuration.

#[cfg(feature = "tui")]
pub mod app;

#[cfg(feature = "tui")]
pub use app::TuiApp;

/// TUI module placeholder
pub struct TuiApp;
