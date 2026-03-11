//! App Launcher Tool - Open applications on macOS

use super::tool_trait::{Tool, ToolError, ToolErrorKind, ToolOutput, ToolParameter, ToolResult};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};

/// Application launcher for macOS
pub struct AppLauncherTool;

impl AppLauncherTool {
    pub fn new() -> Self {
        info!("🚀 Initialized App Launcher tool");
        Self
    }

    /// Open an application
    fn open_app(&self, app_name: &str) -> ToolResult {
        debug!("Opening application: {}", app_name);

        // Use macOS 'open' command
        let output = Command::new("open")
            .arg("-a")
            .arg(app_name)
            .output()
            .map_err(|e| ToolError {
                message: format!("Failed to execute open command: {}", e),
                kind: ToolErrorKind::ExecutionFailed,
            })?;

        if output.status.success() {
            info!("✅ Opened application: {}", app_name);
            Ok(ToolOutput::success(format!("Opened {}", app_name)))
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(ToolError {
                message: format!("Failed to open {}: {}", app_name, error),
                kind: ToolErrorKind::ExecutionFailed,
            })
        }
    }
}

#[async_trait::async_trait]
impl Tool for AppLauncherTool {
    fn name(&self) -> &str {
        "open_app"
    }

    fn description(&self) -> &str {
        "Open an application on your Mac. Provide the application name (e.g., 'Safari', 'Obsidian', 'Finder')."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "app_name".to_string(),
            description: "Name of the application to open (e.g., 'Safari', 'Obsidian')".to_string(),
            param_type: "string".to_string(),
            required: true,
        }]
    }

    async fn execute(&self, args: HashMap<String, serde_json::Value>) -> ToolResult {
        let app_name = args
            .get("app_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError {
                message: "Missing required parameter: app_name".to_string(),
                kind: ToolErrorKind::InvalidArguments,
            })?;

        self.open_app(app_name)
    }
}

impl Default for AppLauncherTool {
    fn default() -> Self {
        Self::new()
    }
}
