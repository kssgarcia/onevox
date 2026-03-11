//! Obsidian Tool - Create, update, and search Obsidian notes

use super::tool_trait::{Tool, ToolError, ToolErrorKind, ToolOutput, ToolParameter, ToolResult};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

/// Obsidian vault tool
pub struct ObsidianTool {
    vault_path: PathBuf,
}

impl ObsidianTool {
    /// Create a new Obsidian tool
    pub fn new(vault_path: PathBuf) -> crate::Result<Self> {
        // Validate vault path exists
        if !vault_path.exists() {
            return Err(crate::Error::Config(format!(
                "Obsidian vault path does not exist: {:?}",
                vault_path
            )));
        }

        if !vault_path.is_dir() {
            return Err(crate::Error::Config(format!(
                "Obsidian vault path is not a directory: {:?}",
                vault_path
            )));
        }

        info!("📝 Initialized Obsidian tool with vault: {:?}", vault_path);
        Ok(Self { vault_path })
    }

    /// Create a new note
    fn create_note(&self, title: &str, content: &str) -> ToolResult {
        let sanitized_title = self.sanitize_filename(title);
        let note_path = self.vault_path.join(format!("{}.md", sanitized_title));

        // Check if note already exists
        if note_path.exists() {
            return Err(ToolError {
                message: format!("Note '{}' already exists", title),
                kind: ToolErrorKind::ExecutionFailed,
            });
        }

        // Create note with frontmatter
        let frontmatter = format!(
            "---\ncreated: {}\ntags: [ai-created]\n---\n\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        let full_content = format!("{}{}", frontmatter, content);

        fs::write(&note_path, full_content).map_err(|e| ToolError {
            message: format!("Failed to create note: {}", e),
            kind: ToolErrorKind::ExecutionFailed,
        })?;

        info!("✅ Created Obsidian note: {}", sanitized_title);
        Ok(ToolOutput::success(format!(
            "Created note '{}' in Obsidian vault",
            title
        )))
    }

    /// Update an existing note (append content)
    fn update_note(&self, title: &str, content: &str) -> ToolResult {
        let sanitized_title = self.sanitize_filename(title);
        let note_path = self.vault_path.join(format!("{}.md", sanitized_title));

        if !note_path.exists() {
            return Err(ToolError {
                message: format!("Note '{}' not found", title),
                kind: ToolErrorKind::NotFound,
            });
        }

        // Append content with timestamp
        let update_marker = format!(
            "\n\n---\n**Updated: {}**\n\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        let append_content = format!("{}{}", update_marker, content);

        fs::OpenOptions::new()
            .append(true)
            .open(&note_path)
            .and_then(|mut file| {
                use std::io::Write;
                file.write_all(append_content.as_bytes())
            })
            .map_err(|e| ToolError {
                message: format!("Failed to update note: {}", e),
                kind: ToolErrorKind::ExecutionFailed,
            })?;

        info!("✅ Updated Obsidian note: {}", sanitized_title);
        Ok(ToolOutput::success(format!(
            "Updated note '{}' in Obsidian vault",
            title
        )))
    }

    /// Search notes by keyword
    fn search_notes(&self, query: &str) -> ToolResult {
        debug!("Searching Obsidian notes for: {}", query);

        let mut results = Vec::new();

        // Walk through vault directory
        for entry in walkdir::WalkDir::new(&self.vault_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file()
                && entry.path().extension().is_some_and(|ext| ext == "md")
            {
                // Read file content
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    // Simple case-insensitive search
                    if content.to_lowercase().contains(&query.to_lowercase()) {
                        let title = entry
                            .path()
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Unknown");
                        results.push(title.to_string());
                    }
                }
            }
        }

        if results.is_empty() {
            Ok(ToolOutput::success(format!(
                "No notes found matching '{}'",
                query
            )))
        } else {
            let result_list = results.join(", ");
            info!("📚 Found {} note(s) matching '{}'", results.len(), query);
            Ok(ToolOutput::with_data(
                format!("Found {} note(s): {}", results.len(), result_list),
                serde_json::json!({ "notes": results }),
            ))
        }
    }

    /// Sanitize filename (remove special characters)
    fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                _ => c,
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl Tool for ObsidianTool {
    fn name(&self) -> &str {
        "obsidian"
    }

    fn description(&self) -> &str {
        "Create, update, or search notes in your Obsidian vault. Use 'create' to make new notes, 'update' to append to existing notes, or 'search' to find notes by keyword."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "action".to_string(),
                description: "Action to perform: 'create', 'update', or 'search'".to_string(),
                param_type: "string".to_string(),
                required: true,
            },
            ToolParameter {
                name: "title".to_string(),
                description: "Note title (for create/update actions)".to_string(),
                param_type: "string".to_string(),
                required: false,
            },
            ToolParameter {
                name: "content".to_string(),
                description: "Note content (for create/update actions)".to_string(),
                param_type: "string".to_string(),
                required: false,
            },
            ToolParameter {
                name: "query".to_string(),
                description: "Search query (for search action)".to_string(),
                param_type: "string".to_string(),
                required: false,
            },
        ]
    }

    async fn execute(&self, args: HashMap<String, serde_json::Value>) -> ToolResult {
        // Parse action
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError {
                message: "Missing required parameter: action".to_string(),
                kind: ToolErrorKind::InvalidArguments,
            })?;

        match action {
            "create" => {
                let title =
                    args.get("title")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ToolError {
                            message: "Missing required parameter: title".to_string(),
                            kind: ToolErrorKind::InvalidArguments,
                        })?;

                let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");

                self.create_note(title, content)
            }
            "update" => {
                let title =
                    args.get("title")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ToolError {
                            message: "Missing required parameter: title".to_string(),
                            kind: ToolErrorKind::InvalidArguments,
                        })?;

                let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");

                self.update_note(title, content)
            }
            "search" => {
                let query =
                    args.get("query")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ToolError {
                            message: "Missing required parameter: query".to_string(),
                            kind: ToolErrorKind::InvalidArguments,
                        })?;

                self.search_notes(query)
            }
            _ => Err(ToolError {
                message: format!(
                    "Invalid action: {}. Must be 'create', 'update', or 'search'",
                    action
                ),
                kind: ToolErrorKind::InvalidArguments,
            }),
        }
    }
}
