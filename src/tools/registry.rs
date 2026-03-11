//! Tool Registry - Manages available tools

use super::tool_trait::{Tool, ToolCall, ToolDefinition, ToolError, ToolErrorKind, ToolResult};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};

/// Registry of available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        debug!("Registering tool: {}", name);
        self.tools.insert(name, tool);
    }

    /// Get tool by name
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// Execute a tool call
    pub async fn execute(&self, call: &ToolCall) -> ToolResult {
        debug!(
            "Executing tool: {} with args: {:?}",
            call.name, call.arguments
        );

        match self.tools.get(&call.name) {
            Some(tool) => tool.execute(call.arguments.clone()).await,
            None => {
                warn!("Tool not found: {}", call.name);
                Err(ToolError {
                    message: format!("Tool '{}' not found", call.name),
                    kind: ToolErrorKind::NotFound,
                })
            }
        }
    }

    /// Get all tool definitions for LLM context
    pub fn get_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|tool| tool.definition()).collect()
    }

    /// Get tool names
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
