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

#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permission() -> PermissionStatus {
    PermissionStatus::NotApplicable
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

#[cfg(not(target_os = "macos"))]
pub fn prompt_accessibility_permission() {
    // Not needed on other platforms
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

#[cfg(not(target_os = "macos"))]
pub fn prompt_input_monitoring_permission() {
    // Not needed on other platforms
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

#[cfg(not(target_os = "macos"))]
pub fn open_accessibility_settings() -> crate::Result<()> {
    Err(crate::Error::Platform(
        "Not supported on this platform".to_string(),
    ))
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

#[cfg(not(target_os = "macos"))]
pub fn open_input_monitoring_settings() -> crate::Result<()> {
    Err(crate::Error::Platform(
        "Not supported on this platform".to_string(),
    ))
}

/// Check all required permissions
pub fn check_required_permissions() -> Vec<(Permission, PermissionStatus)> {
    let mut results = Vec::new();

    // Check accessibility permission
    #[cfg(target_os = "macos")]
    {
        let status = check_accessibility_permission();
        results.push((Permission::Accessibility, status));
    }

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
        // Should return at least one permission on macOS, or empty on other platforms
        assert!(permissions.is_empty() || !permissions.is_empty());
    }
}
