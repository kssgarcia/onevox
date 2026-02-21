//! Platform-specific integrations
//!
//! OS-specific code for hotkeys, text injection, etc.

pub mod hotkey;
pub mod injector;
pub mod paths;
pub mod permissions;

// Re-export commonly used types
pub use hotkey::{HotkeyConfig, HotkeyEvent, HotkeyManager};
pub use injector::{InjectionStrategy, InjectorConfig, TextInjector};
pub use paths::{
    cache_dir, config_dir, config_file_path, data_dir, ensure_directories, history_db_path,
    ipc_socket_path, log_dir, model_path, models_dir,
};
pub use permissions::{
    Permission, PermissionStatus, check_accessibility_permission, check_required_permissions,
    open_accessibility_settings, prompt_accessibility_permission, verify_permissions,
};
