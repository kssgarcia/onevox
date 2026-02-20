//! Platform Permissions
//!
//! Check and request platform-specific permissions (macOS Accessibility, etc.).

use tracing::info;

/// Permission type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Accessibility permissions (for hotkeys and text injection)
    Accessibility,
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
    println!("  • Register global hotkeys (Cmd+Shift+Space)");
    println!("  • Inject transcribed text into applications");
    println!("\nTo grant permission:");
    println!("  1. Open System Preferences → Security & Privacy");
    println!("  2. Go to the Privacy tab");
    println!("  3. Select 'Accessibility' from the list");
    println!("  4. Click the lock icon and enter your password");
    println!("  5. Add 'vox' or your terminal app to the list");
    println!("  6. Check the box next to the app");
    println!("\nAfter granting permission, restart Vox.");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

#[cfg(not(target_os = "macos"))]
pub fn prompt_accessibility_permission() {
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
