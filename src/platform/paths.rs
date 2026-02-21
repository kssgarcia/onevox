//! Cross-platform path resolution using industry-standard directory layout
//!
//! Uses the `directories` crate with `ProjectDirs` for professional-grade
//! cross-platform path management that follows platform conventions.

use crate::Result;
use directories::ProjectDirs;
use std::path::PathBuf;

/// Get ProjectDirs instance for onevox
fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("com", "onevox", "onevox")
        .ok_or_else(|| crate::Error::Config("Cannot determine project directories".into()))
}

/// Get the application cache directory
///
/// Platform-specific paths:
/// - macOS: `~/Library/Caches/com.onevox.onevox`
/// - Linux: `~/.cache/onevox`
/// - Windows: `%LOCALAPPDATA%\onevox\onevox\cache`
pub fn cache_dir() -> Result<PathBuf> {
    let proj_dirs = project_dirs()?;
    let cache = proj_dirs.cache_dir().to_path_buf();

    // Ensure directory exists
    if !cache.exists() {
        std::fs::create_dir_all(&cache)?;
        set_dir_permissions(&cache)?;
    }

    Ok(cache)
}

/// Get the application config directory
///
/// Platform-specific paths:
/// - macOS: `~/Library/Application Support/com.onevox.onevox`
/// - Linux: `~/.config/onevox`
/// - Windows: `%APPDATA%\onevox\onevox\config`
pub fn config_dir() -> Result<PathBuf> {
    let proj_dirs = project_dirs()?;
    let config = proj_dirs.config_dir().to_path_buf();

    // Ensure directory exists
    if !config.exists() {
        std::fs::create_dir_all(&config)?;
        set_dir_permissions(&config)?;
    }

    Ok(config)
}

/// Get the application data directory (for history, databases, etc.)
///
/// Platform-specific paths:
/// - macOS: `~/Library/Application Support/com.onevox.onevox`
/// - Linux: `~/.local/share/onevox`
/// - Windows: `%APPDATA%\onevox\onevox\data`
pub fn data_dir() -> Result<PathBuf> {
    let proj_dirs = project_dirs()?;
    let data = proj_dirs.data_dir().to_path_buf();

    // Ensure directory exists
    if !data.exists() {
        std::fs::create_dir_all(&data)?;
        set_dir_permissions(&data)?;
    }

    Ok(data)
}

/// Get the models directory
///
/// Models are stored in cache since they can be re-downloaded if needed
pub fn models_dir() -> Result<PathBuf> {
    let models = cache_dir()?.join("models");

    if !models.exists() {
        std::fs::create_dir_all(&models)?;
        set_dir_permissions(&models)?;
    }

    Ok(models)
}

/// Get the path for a specific model
pub fn model_path(model_id: &str) -> Result<PathBuf> {
    Ok(models_dir()?.join(model_id))
}

/// Get the history database path
pub fn history_db_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("history.json"))
}

/// Get the config file path
pub fn config_file_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

/// Get the runtime directory for IPC sockets
///
/// Platform-specific paths:
/// - macOS: `/tmp/onevox`
/// - Linux: `$XDG_RUNTIME_DIR/onevox` or `/tmp/onevox`
/// - Windows: Fallback to cache directory
pub fn runtime_dir() -> Result<PathBuf> {
    #[cfg(unix)]
    {
        // Use XDG_RUNTIME_DIR on Linux if available
        #[cfg(target_os = "linux")]
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let dir = PathBuf::from(runtime_dir).join("onevox");
            if !dir.exists() {
                std::fs::create_dir_all(&dir)?;
                set_dir_permissions(&dir)?;
            }
            return Ok(dir);
        }

        // Fallback to /tmp/onevox (both macOS and Linux)
        let tmp_dir = std::env::temp_dir().join("onevox");
        if !tmp_dir.exists() {
            std::fs::create_dir_all(&tmp_dir)?;
            set_dir_permissions(&tmp_dir)?;
        }
        Ok(tmp_dir)
    }

    #[cfg(windows)]
    {
        // Windows doesn't have a runtime dir concept, use cache
        cache_dir()
    }
}

/// Get the log directory
///
/// Platform-specific paths:
/// - macOS: `~/Library/Logs/com.onevox.onevox`
/// - Linux: `~/.local/share/onevox/logs`
/// - Windows: `%APPDATA%\onevox\onevox\data\logs`
pub fn log_dir() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    let log_dir = {
        // macOS has a dedicated Logs directory
        directories::BaseDirs::new()
            .ok_or_else(|| crate::Error::Config("Cannot determine base directories".into()))?
            .home_dir()
            .join("Library")
            .join("Logs")
            .join("com.onevox.onevox")
    };

    #[cfg(not(target_os = "macos"))]
    let log_dir = data_dir()?.join("logs");

    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)?;
        set_dir_permissions(&log_dir)?;
    }

    Ok(log_dir)
}

/// Get the IPC socket path
///
/// Platform-specific paths:
/// - macOS: `/tmp/onevox.sock`
/// - Linux: `$XDG_RUNTIME_DIR/onevox.sock` or `/tmp/onevox.sock`
/// - Windows: `\\.\pipe\onevox` (named pipe)
pub fn ipc_socket_path() -> Result<PathBuf> {
    #[cfg(unix)]
    {
        // Use XDG_RUNTIME_DIR on Linux if available (better for systemd integration)
        #[cfg(target_os = "linux")]
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            return Ok(PathBuf::from(runtime_dir).join("onevox.sock"));
        }

        // Fallback to /tmp (both macOS and Linux)
        let tmp_dir = std::env::temp_dir();
        Ok(tmp_dir.join("onevox.sock"))
    }

    #[cfg(windows)]
    {
        // Windows uses named pipes, not file paths
        Ok(PathBuf::from(r"\\.\pipe\onevox"))
    }
}

/// Set appropriate directory permissions
///
/// On Unix: Sets to 0o700 (owner read/write/execute only) for security
/// On Windows: No-op (Windows uses ACLs)
#[allow(unused_variables)]
fn set_dir_permissions(dir: &PathBuf) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o700);
        std::fs::set_permissions(dir, perms)?;
    }

    Ok(())
}

/// Create all necessary directories with appropriate permissions
///
/// This should be called once on daemon startup to ensure all directories exist
pub fn ensure_directories() -> Result<()> {
    // Calling these functions will automatically create directories if they don't exist
    cache_dir()?;
    config_dir()?;
    data_dir()?;
    models_dir()?;
    log_dir()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_dir() {
        let dir = cache_dir().unwrap();
        assert!(dir.to_string_lossy().contains("onevox"));
        println!("Cache dir: {}", dir.display());
    }

    #[test]
    fn test_config_dir() {
        let dir = config_dir().unwrap();
        assert!(dir.to_string_lossy().contains("onevox"));
        println!("Config dir: {}", dir.display());
    }

    #[test]
    fn test_data_dir() {
        let dir = data_dir().unwrap();
        assert!(dir.to_string_lossy().contains("onevox"));
        println!("Data dir: {}", dir.display());
    }

    #[test]
    fn test_models_dir() {
        let dir = models_dir().unwrap();
        assert!(dir.to_string_lossy().contains("models"));
        println!("Models dir: {}", dir.display());
    }

    #[test]
    fn test_model_path() {
        let path = model_path("whisper-tiny.en").unwrap();
        assert!(path.to_string_lossy().contains("whisper-tiny.en"));
        println!("Model path: {}", path.display());
    }

    #[test]
    fn test_log_dir() {
        let dir = log_dir().unwrap();
        println!("Log dir: {}", dir.display());

        #[cfg(target_os = "macos")]
        assert!(dir.to_string_lossy().contains("Library/Logs"));

        #[cfg(not(target_os = "macos"))]
        assert!(dir.to_string_lossy().contains("logs"));
    }

    #[test]
    fn test_ipc_socket_path() {
        let path = ipc_socket_path().unwrap();
        println!("IPC socket path: {}", path.display());

        #[cfg(unix)]
        assert!(path.to_string_lossy().contains(".sock"));

        #[cfg(windows)]
        assert!(path.to_string_lossy().contains("pipe"));
    }

    #[test]
    fn test_ensure_directories() {
        // This should create all directories without error
        ensure_directories().unwrap();

        // Verify they exist
        assert!(cache_dir().unwrap().exists());
        assert!(config_dir().unwrap().exists());
        assert!(data_dir().unwrap().exists());
        assert!(models_dir().unwrap().exists());
        assert!(log_dir().unwrap().exists());
    }
}
