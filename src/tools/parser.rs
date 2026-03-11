//! Tool Call Parser - Extract tool calls from LLM responses

use crate::tools::ToolCall;
use serde_json::Value;
use std::collections::HashMap;
use tracing::debug;

/// Parse tool calls from LLM response text
///
/// Supports multiple formats:
/// 1. JSON array: [{"name": "tool_name", "arguments": {...}}]
/// 2. Single JSON object: {"name": "tool_name", "arguments": {...}}
/// 3. Markdown code block: ```json\n[...]\n```
pub fn parse_tool_calls(text: &str) -> Vec<ToolCall> {
    let mut tool_calls = Vec::new();

    // Try to find JSON content (could be in markdown code blocks)
    let json_content = extract_json_content(text);

    for content in json_content {
        if let Ok(value) = serde_json::from_str::<Value>(&content) {
            match value {
                Value::Array(arr) => {
                    // Multiple tool calls
                    for item in arr {
                        if let Some(call) = parse_single_tool_call(&item) {
                            tool_calls.push(call);
                        }
                    }
                }
                Value::Object(_) => {
                    // Single tool call
                    if let Some(call) = parse_single_tool_call(&value) {
                        tool_calls.push(call);
                    }
                }
                _ => {
                    debug!("Unexpected JSON type in tool call: {:?}", value);
                }
            }
        }
    }

    if !tool_calls.is_empty() {
        debug!("Parsed {} tool calls from LLM response", tool_calls.len());
    }

    tool_calls
}

/// Extract JSON content from text (handles markdown code blocks)
fn extract_json_content(text: &str) -> Vec<String> {
    let mut contents = Vec::new();

    // Check for markdown code blocks with json
    let code_block_pattern = r"```(?:json)?\s*([\s\S]*?)```";
    if let Ok(re) = regex::Regex::new(code_block_pattern) {
        for cap in re.captures_iter(text) {
            if let Some(json_str) = cap.get(1) {
                contents.push(json_str.as_str().trim().to_string());
            }
        }
    }

    // If no code blocks found, try the entire text
    if contents.is_empty() {
        // Look for JSON-like structures (starts with { or [)
        let trimmed = text.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            contents.push(trimmed.to_string());
        }
    }

    contents
}

/// Parse a single tool call from JSON value
fn parse_single_tool_call(value: &Value) -> Option<ToolCall> {
    let obj = value.as_object()?;

    // Get tool name (try different field names)
    let name_option = obj
        .get("name")
        .or_else(|| obj.get("tool"))
        .or_else(|| obj.get("function"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Get arguments (try different field names)
    let args_value = obj
        .get("arguments")
        .or_else(|| obj.get("args"))
        .or_else(|| obj.get("parameters"));

    let mut arguments = if let Some(args) = args_value {
        match args {
            Value::Object(map) => {
                // Convert to HashMap<String, Value>
                map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            }
            Value::String(s) => {
                // Arguments might be a JSON string that needs parsing
                if let Ok(Value::Object(map)) = serde_json::from_str(s) {
                    map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                } else {
                    HashMap::new()
                }
            }
            _ => HashMap::new(),
        }
    } else {
        HashMap::new()
    };

    // Fallback: If no "name" field found, try to infer tool from parameters
    let name = if let Some(n) = name_option {
        n
    } else {
        debug!("No 'name' field found, attempting to infer tool from parameters");

        // Check for common parameter patterns to infer tool
        if obj.contains_key("app_name") {
            // This looks like an app launcher call
            debug!("Found 'app_name' parameter, inferring tool as 'open_app'");
            // Move app_name from top level to arguments
            if let Some(app_name) = obj.get("app_name") {
                arguments.insert("app_name".to_string(), app_name.clone());
            }
            "open_app".to_string()
        } else if obj.contains_key("title") || obj.contains_key("action") {
            // This looks like an Obsidian call
            debug!("Found Obsidian-related parameters, inferring tool as 'obsidian'");
            // Move parameters from top level to arguments if needed
            for key in &["title", "action", "content", "query"] {
                if let Some(val) = obj.get(*key) {
                    arguments.insert(key.to_string(), val.clone());
                }
            }
            "obsidian".to_string()
        } else if obj.contains_key("memory") || obj.contains_key("query") {
            // This looks like a memory call
            debug!("Found memory-related parameters, inferring tool as 'memory'");
            for key in &["memory", "query", "action"] {
                if let Some(val) = obj.get(*key) {
                    arguments.insert(key.to_string(), val.clone());
                }
            }
            "memory".to_string()
        } else {
            debug!(
                "Could not infer tool from parameters: {:?}",
                obj.keys().collect::<Vec<_>>()
            );
            return None;
        }
    };

    Some(ToolCall { name, arguments })
}

/// Format tool definitions for LLM system prompt
pub fn format_tool_definitions(definitions: &[crate::tools::ToolDefinition]) -> String {
    let mut prompt = String::from("\n\n## Available Tools\n\n");
    prompt.push_str("You have access to these tools to help the user:\n\n");

    for def in definitions {
        prompt.push_str(&format!("### {}\n", def.name));
        prompt.push_str(&format!("{}\n\n", def.description));

        if !def.parameters.is_empty() {
            prompt.push_str("Parameters:\n");
            for param in &def.parameters {
                let required = if param.required {
                    "**required**"
                } else {
                    "optional"
                };
                prompt.push_str(&format!(
                    "- `{}` ({}): {} [{}]\n",
                    param.name, param.param_type, param.description, required
                ));
            }
        }
        prompt.push('\n');
    }

    prompt.push_str("\n## When to Use Tools\n\n");
    prompt.push_str("ONLY use tools when the user EXPLICITLY requests an action:\n");
    prompt.push_str("- ✅ \"Create a note called Meeting Notes\" → Use obsidian tool\n");
    prompt.push_str("- ✅ \"Open Safari\" → Use open_app tool\n");
    prompt.push_str("- ✅ \"Remember that I have a meeting at 3pm\" → Use memory tool\n");
    prompt.push_str(
        "- ❌ \"What tools can you use?\" → Just respond conversationally, DON'T call tools\n",
    );
    prompt.push_str(
        "- ❌ \"Tell me about yourself\" → Just respond conversationally, DON'T call tools\n",
    );
    prompt.push_str("- ❌ \"How are you?\" → Just respond conversationally, DON'T call tools\n\n");

    prompt.push_str("## Tool Call Format\n\n");
    prompt.push_str("CRITICAL: When calling a tool, you MUST use this EXACT JSON structure:\n\n");

    prompt.push_str("Step 1: Identify the tool name from the list above\n");
    prompt.push_str("Step 2: Put the tool name in the \"name\" field\n");
    prompt.push_str("Step 3: Put ALL parameters inside the \"arguments\" object\n\n");

    prompt.push_str("Example 1 - Opening Safari:\n");
    prompt.push_str("```json\n");
    prompt.push_str("{\n");
    prompt.push_str("  \"name\": \"open_app\",\n");
    prompt.push_str("  \"arguments\": {\"app_name\": \"Safari\"}\n");
    prompt.push_str("}\n");
    prompt.push_str("```\n\n");

    prompt.push_str("Example 2 - Opening Spotify:\n");
    prompt.push_str("```json\n");
    prompt.push_str("{\n");
    prompt.push_str("  \"name\": \"open_app\",\n");
    prompt.push_str("  \"arguments\": {\"app_name\": \"Spotify\"}\n");
    prompt.push_str("}\n");
    prompt.push_str("```\n\n");

    prompt.push_str("Example 3 - Creating a note:\n");
    prompt.push_str("```json\n");
    prompt.push_str("{\n");
    prompt.push_str("  \"name\": \"obsidian\",\n");
    prompt.push_str("  \"arguments\": {\"action\": \"create\", \"title\": \"Ideas\"}\n");
    prompt.push_str("}\n");
    prompt.push_str("```\n\n");

    prompt.push_str("WRONG - Do NOT do this:\n");
    prompt.push_str("```json\n");
    prompt.push_str("{\n");
    prompt.push_str("  \"app_name\": \"Safari\",    // ❌ WRONG: parameter at top level\n");
    prompt.push_str("  \"arguments\": {}\n");
    prompt.push_str("}\n");
    prompt.push_str("```\n\n");

    prompt.push_str("For normal conversation, respond with plain text (no JSON).\n");

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_tool_call() {
        let text = r#"{"name": "obsidian", "arguments": {"action": "create", "title": "Test"}}"#;
        let calls = parse_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "obsidian");
    }

    #[test]
    fn test_parse_tool_call_in_code_block() {
        let text = r#"I'll create that note for you.
```json
{"name": "obsidian", "arguments": {"action": "create", "title": "Test"}}
```"#;
        let calls = parse_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "obsidian");
    }

    #[test]
    fn test_parse_multiple_tool_calls() {
        let text = r#"```json
[
  {"name": "obsidian", "arguments": {"action": "create", "title": "Note1"}},
  {"name": "open_app", "arguments": {"app_name": "Safari"}}
]
```"#;
        let calls = parse_tool_calls(text);
        assert_eq!(calls.len(), 2);
    }
}
