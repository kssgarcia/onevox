//! Platform-specific integrations
//!
//! OS-specific code for hotkeys, text injection, etc.

pub mod hotkey;
pub mod injector;
pub mod permissions;

// Re-export commonly used types
pub use hotkey::{HotkeyConfig, HotkeyEvent, HotkeyManager};
pub use injector::{InjectionStrategy, InjectorConfig, TextInjector};
pub use permissions::{
    check_accessibility_permission, check_required_permissions, open_accessibility_settings,
    prompt_accessibility_permission, verify_permissions, Permission, PermissionStatus,
};
