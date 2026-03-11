//! Tool Trait - Core abstraction for executable tools

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Tool execution result
pub type ToolResult = Result<ToolOutput, ToolError>;

/// Tool output with structured data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// Human-readable success message
    pub message: String,

    /// Structured data (optional)
    pub data: Option<serde_json::Value>,
}

impl ToolOutput {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            message: message.into(),
            data: Some(data),
        }
    }
}

impl fmt::Display for ToolOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Tool execution error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolError {
    pub message: String,
    pub kind: ToolErrorKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolErrorKind {
    InvalidArguments,
    ExecutionFailed,
    NotFound,
    PermissionDenied,
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for ToolError {}

/// Tool call from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool name (e.g., "obsidian_create")
    pub name: String,

    /// Tool arguments as JSON
    pub arguments: HashMap<String, serde_json::Value>,
}

/// Tool definition for LLM context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub param_type: String, // "string", "number", "boolean", "array"
    pub required: bool,
}

/// Core Tool trait
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (e.g., "obsidian_create")
    fn name(&self) -> &str;

    /// Tool description for LLM
    fn description(&self) -> &str;

    /// Tool parameters definition
    fn parameters(&self) -> Vec<ToolParameter>;

    /// Execute the tool with given arguments
    async fn execute(&self, args: HashMap<String, serde_json::Value>) -> ToolResult;

    /// Get tool definition for LLM context
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: self.parameters(),
        }
    }
}
