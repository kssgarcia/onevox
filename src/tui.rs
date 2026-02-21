//! Terminal User Interface
//!
//! Launches the OpenTUI-based TypeScript TUI for interactive configuration and monitoring.

use crate::Result;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Launch the OpenTUI-based terminal interface
///
/// This function spawns the Bun-based TypeScript TUI that provides:
/// - Interactive configuration management
/// - Transcription history viewer
/// - Audio device selection
/// - Real-time daemon monitoring
///
/// The TUI is located in the `tui/` directory and requires Bun to be installed.
pub fn launch() -> Result<()> {
    // Find the TUI directory
    let tui_dir = find_tui_directory()?;

    // Check if Bun is installed
    if !is_bun_installed() {
        return Err(crate::Error::Other(
            "Bun is not installed. Please install Bun from https://bun.sh".to_string(),
        ));
    }

    // Check if dependencies are installed
    if !has_node_modules(&tui_dir) {
        eprintln!("📦 Installing TUI dependencies...");
        install_dependencies(&tui_dir)?;
    }

    // Launch the TUI
    eprintln!("🖥️  Launching ONEVOX TUI...\n");

    let status = Command::new("bun")
        .arg("run")
        .arg("src/index.ts")
        .current_dir(&tui_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| crate::Error::Other(format!("Failed to launch TUI: {}", e)))?;

    if !status.success() {
        return Err(crate::Error::Other(format!(
            "TUI exited with status: {}",
            status.code().unwrap_or(-1)
        )));
    }

    Ok(())
}

/// Find the TUI directory by walking up from the binary location
fn find_tui_directory() -> Result<PathBuf> {
    // Get the actual binary path (resolve symlinks)
    let current_exe = std::env::current_exe()
        .map_err(|e| crate::Error::Other(format!("Failed to get current exe path: {}", e)))?;
    
    let resolved_exe = std::fs::canonicalize(&current_exe)
        .unwrap_or_else(|_| current_exe.clone());

    // Installed app layout: Onevox.app/Contents/MacOS/onevox
    // Bundled TUI path:      Onevox.app/Contents/Resources/tui
    if let Some(macos_dir) = resolved_exe.parent() {
        if let Some(contents_dir) = macos_dir.parent() {
            let bundled_tui = contents_dir.join("Resources").join("tui");
            if bundled_tui.exists() && bundled_tui.is_dir() {
                return Ok(bundled_tui);
            }
        }
    }

    // Try to find the project root by looking for Cargo.toml
    let mut current = resolved_exe.clone();
    for _ in 0..10 {
        current.pop();
        let tui_dir = current.join("tui");
        let cargo_toml = current.join("Cargo.toml");

        if tui_dir.exists() && tui_dir.is_dir() && cargo_toml.exists() {
            return Ok(tui_dir);
        }
    }

    // Fallback: try current working directory
    let cwd = std::env::current_dir()
        .map_err(|e| crate::Error::Other(format!("Failed to get current directory: {}", e)))?;

    let tui_dir = cwd.join("tui");
    if tui_dir.exists() && tui_dir.is_dir() {
        return Ok(tui_dir);
    }

    Err(crate::Error::Other(
        "Could not find TUI directory. Install app resources or run from project root.".to_string(),
    ))
}

/// Check if Bun is installed
fn is_bun_installed() -> bool {
    Command::new("bun")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Check if node_modules exists
fn has_node_modules(tui_dir: &PathBuf) -> bool {
    tui_dir.join("node_modules").exists()
}

/// Install TUI dependencies
fn install_dependencies(tui_dir: &PathBuf) -> Result<()> {
    let status = Command::new("bun")
        .arg("install")
        .current_dir(tui_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| crate::Error::Other(format!("Failed to install dependencies: {}", e)))?;

    if !status.success() {
        return Err(crate::Error::Other(
            "Failed to install TUI dependencies".to_string(),
        ));
    }

    Ok(())
}
