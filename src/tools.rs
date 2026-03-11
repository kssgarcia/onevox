//! Tool System Module
//!
//! Provides framework for LLM tool calling to execute actions like:
//! - Creating/searching Obsidian notes
//! - Saving/retrieving memories (RAG)
//! - Opening applications
//! - Setting reminders

pub mod app_launcher;
pub mod memory;
pub mod obsidian;
pub mod parser;
pub mod registry;
pub mod tool_trait;

pub use app_launcher::AppLauncherTool;
pub use memory::MemoryTool;
pub use obsidian::ObsidianTool;
pub use parser::{format_tool_definitions, parse_tool_calls};
pub use registry::ToolRegistry;
pub use tool_trait::{Tool, ToolCall, ToolDefinition, ToolError, ToolParameter, ToolResult};
