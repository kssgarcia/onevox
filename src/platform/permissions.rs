//! Platform Permissions
//!
//! Check and request platform-specific permissions (macOS Accessibility, etc.).

use tracing::info;

/// Permission type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Accessibility permissions (for text injection)
    Accessibility,
    /// Input Monitoring (for global hotkeys)
    InputMonitoring,
    /// Microphone access
    Microphone,
    /// Screen recording (for some accessibility features)
    ScreenRecording,
}

/// Permission status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionStatus {
    /// Permission granted
    Granted,
    /// Permission denied
    Denied,
    /// Permission not determined yet
    NotDetermined,
    /// Not applicable on this platform
    NotApplicable,
}

/// Check if accessibility permission is granted (macOS)
#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> PermissionStatus {
    // For now, assume granted since checking is complex
    // In production, we'd use CGEventTap or AXIsProcessTrusted
    info!("Skipping accessibility permission check (assumed granted)");
    PermissionStatus::Granted
}

/// Check if accessibility permission is granted (Linux)
#[cfg(target_os = "linux")]
pub fn check_accessibility_permission() -> PermissionStatus {
    // On Linux, check if we're running under X11 or Wayland
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        // Wayland - permissions depend on compositor
        info!("Running under Wayland - accessibility depends on compositor support");
        PermissionStatus::Granted // Assume granted, will fail at runtime if not
    } else if std::env::var("DISPLAY").is_ok() {
        // X11 - generally no special permissions needed
        info!("Running under X11 - accessibility available");
        PermissionStatus::Granted
    } else {
        // No display server
        info!("No display server detected");
        PermissionStatus::NotDetermined
    }
}

/// Check if accessibility permission is granted (Windows)
#[cfg(target_os = "windows")]
pub fn check_accessibility_permission() -> PermissionStatus {
    // Windows doesn't require special accessibility permissions
    PermissionStatus::Granted
}

/// Check microphone permission (Linux)
#[cfg(target_os = "linux")]
pub fn check_microphone_permission() -> PermissionStatus {
    // Check if we can access audio devices
    if std::path::Path::new("/dev/snd").exists() {
        PermissionStatus::Granted
    } else {
        PermissionStatus::Denied
    }
}

/// Check microphone permission (Windows)
#[cfg(target_os = "windows")]
pub fn check_microphone_permission() -> PermissionStatus {
    // Windows 10+ has microphone privacy settings
    // For now, assume granted - will prompt at runtime
    PermissionStatus::Granted
}

/// Check microphone permission (macOS)
#[cfg(target_os = "macos")]
pub fn check_microphone_permission() -> PermissionStatus {
    // macOS will prompt automatically
    PermissionStatus::Granted
}

/// Prompt user to grant accessibility permission (macOS)
#[cfg(target_os = "macos")]
pub fn prompt_accessibility_permission() {
    println!("\n⚠️  Accessibility Permission Required");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nVox needs accessibility permissions to:");
    println!("  • Inject transcribed text into applications");
    println!("\nTo grant permission:");
    println!("  1. Open System Settings → Privacy & Security");
    println!("  2. Select 'Accessibility' from the list");
    println!("  3. Click the lock icon and enter your password");
    println!("  4. Click the '+' button and add your Terminal app");
    println!("  5. Make sure the toggle is ON (blue)");
    println!("\nAfter granting permission, restart Onevox.");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

/// Prompt user to grant accessibility permission (Linux)
#[cfg(target_os = "linux")]
pub fn prompt_accessibility_permission() {
    println!("\n⚠️  Accessibility Setup (Linux)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nOnevox needs to:");
    println!("  • Capture global hotkeys");
    println!("  • Inject text into applications");
    println!("\nFor X11:");
    println!("  - Usually works out of the box");
    println!("  - If issues occur, check your window manager settings");
    println!("\nFor Wayland:");
    println!("  - GNOME: May need 'gnome-shell-extension-appindicator'");
    println!("  - KDE: Should work out of the box");
    println!("  - Sway/wlroots: Check compositor configuration");
    println!("\nFor microphone access:");
    println!("  - Ensure your user is in the 'audio' group:");
    println!("    sudo usermod -aG audio $USER");
    println!("  - Log out and back in for changes to take effect");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

/// Prompt user to grant accessibility permission (Windows)
#[cfg(target_os = "windows")]
pub fn prompt_accessibility_permission() {
    println!("\n⚠️  Permissions Setup (Windows)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nOnevox needs:");
    println!("  • Microphone access for audio capture");
    println!("  • Keyboard access for global hotkeys");
    println!("\nTo grant microphone permission:");
    println!("  1. Open Settings → Privacy → Microphone");
    println!("  2. Enable 'Allow apps to access your microphone'");
    println!("  3. Scroll down and enable for 'Onevox'");
    println!("\nOr run: start ms-settings:privacy-microphone");
    println!("\nNote: Windows may prompt for permissions automatically");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

/// Prompt user to grant Input Monitoring permission (macOS)
#[cfg(target_os = "macos")]
pub fn prompt_input_monitoring_permission() {
    println!("\n⚠️  Input Monitoring Permission Required");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nVox needs Input Monitoring permissions to:");
    println!("  • Listen for global hotkeys (Cmd+Shift+Space)");
    println!("  • Detect when you press and release the hotkey");
    println!("\nTo grant permission:");
    println!("  1. Open System Settings → Privacy & Security");
    println!("  2. Scroll down and select 'Input Monitoring' from the list");
    println!("  3. Click the lock icon and enter your password");
    println!("  4. Find your Terminal app in the list");
    println!("  5. Make sure the toggle is ON (blue)");
    println!("\n📝 If your Terminal app is not in the list:");
    println!("  1. Click the '+' button");
    println!("  2. Navigate to /Applications/Utilities/Terminal.app");
    println!("  3. Select it and click Open");
    println!("\nAfter granting permission, restart Onevox.");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

/// Prompt user to grant Input Monitoring permission (Linux)
#[cfg(target_os = "linux")]
pub fn prompt_input_monitoring_permission() {
    println!("\n💡 Input Monitoring (Linux)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nGlobal hotkeys should work automatically on Linux.");
    println!("\nIf hotkeys don't work:");
    println!("  • Ensure your user is in the 'input' group:");
    println!("    sudo usermod -aG input $USER");
    println!("  • Check for conflicting hotkey bindings in your DE");
    println!("  • Try running with: sudo onevox daemon --foreground");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

/// Prompt user to grant Input Monitoring permission (Windows)
#[cfg(target_os = "windows")]
pub fn prompt_input_monitoring_permission() {
    println!("\n💡 Input Monitoring (Windows)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nGlobal hotkeys should work automatically on Windows.");
    println!("\nIf hotkeys don't work:");
    println!("  • Check Windows Defender settings");
    println!("  • Ensure no other app is using the same hotkey");
    println!("  • Try running as Administrator");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

/// Open System Preferences to accessibility settings (macOS)
#[cfg(target_os = "macos")]
pub fn open_accessibility_settings() -> crate::Result<()> {
    use std::process::Command;

    info!("Opening accessibility settings");

    Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .map_err(|e| crate::Error::Platform(format!("Failed to open system preferences: {}", e)))?;

    Ok(())
}

/// Open accessibility settings (Linux)
#[cfg(target_os = "linux")]
pub fn open_accessibility_settings() -> crate::Result<()> {
    use std::process::Command;

    info!("Opening system settings");

    // Try different desktop environments
    let commands = vec![
        ("gnome-control-center", vec!["privacy"]),
        ("systemsettings5", vec![]),
        ("xfce4-settings-manager", vec![]),
    ];

    for (cmd, args) in commands {
        if Command::new(cmd).args(&args).spawn().is_ok() {
            return Ok(());
        }
    }

    Err(crate::Error::Platform(
        "Could not open system settings. Please open manually.".to_string(),
    ))
}

/// Open accessibility settings (Windows)
#[cfg(target_os = "windows")]
pub fn open_accessibility_settings() -> crate::Result<()> {
    use std::process::Command;

    info!("Opening Windows settings");

    Command::new("cmd")
        .args(["/C", "start", "ms-settings:privacy-microphone"])
        .spawn()
        .map_err(|e| crate::Error::Platform(format!("Failed to open settings: {}", e)))?;

    Ok(())
}

/// Open System Preferences to Input Monitoring settings (macOS)
#[cfg(target_os = "macos")]
pub fn open_input_monitoring_settings() -> crate::Result<()> {
    use std::process::Command;

    info!("Opening Input Monitoring settings");

    Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")
        .spawn()
        .map_err(|e| crate::Error::Platform(format!("Failed to open system preferences: {}", e)))?;

    Ok(())
}

/// Open Input Monitoring settings (Linux)
#[cfg(target_os = "linux")]
pub fn open_input_monitoring_settings() -> crate::Result<()> {
    // Same as accessibility on Linux
    open_accessibility_settings()
}

/// Open Input Monitoring settings (Windows)
#[cfg(target_os = "windows")]
pub fn open_input_monitoring_settings() -> crate::Result<()> {
    // Same as accessibility on Windows
    open_accessibility_settings()
}

/// Check all required permissions
pub fn check_required_permissions() -> Vec<(Permission, PermissionStatus)> {
    let mut results = Vec::new();

    // Check accessibility permission
    let status = check_accessibility_permission();
    results.push((Permission::Accessibility, status));

    // Check microphone permission
    let mic_status = check_microphone_permission();
    results.push((Permission::Microphone, mic_status));

    results
}

/// Verify all permissions are granted
pub fn verify_permissions() -> crate::Result<()> {
    let permissions = check_required_permissions();

    for (perm, status) in permissions {
        if status == PermissionStatus::Denied || status == PermissionStatus::NotDetermined {
            match perm {
                Permission::Accessibility => {
                    prompt_accessibility_permission();
                    return Err(crate::Error::Platform(
                        "Accessibility permission required".to_string(),
                    ));
                }
                Permission::InputMonitoring => {
                    prompt_input_monitoring_permission();
                    return Err(crate::Error::Platform(
                        "Input Monitoring permission required".to_string(),
                    ));
                }
                Permission::Microphone => {
                    return Err(crate::Error::Platform(
                        "Microphone permission required".to_string(),
                    ));
                }
                Permission::ScreenRecording => {
                    return Err(crate::Error::Platform(
                        "Screen recording permission required".to_string(),
                    ));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_permissions() {
        let permissions = check_required_permissions();
        // Should return at least one permission on any platform
        assert!(!permissions.is_empty());
    }
}
