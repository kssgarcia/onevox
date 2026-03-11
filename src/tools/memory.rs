//! Memory Tool - Save and retrieve information using RAG (Retrieval Augmented Generation)

use super::tool_trait::{Tool, ToolError, ToolErrorKind, ToolOutput, ToolParameter, ToolResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Memory entry with embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub text: String,
    pub embedding: Vec<f32>,
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub metadata: HashMap<String, String>,
}

/// Memory store using simple vector search
pub struct MemoryTool {
    storage_path: PathBuf,
    memories: Arc<RwLock<Vec<MemoryEntry>>>,
    max_results: usize,
}

impl MemoryTool {
    /// Create a new memory tool
    pub fn new(storage_path: PathBuf) -> crate::Result<Self> {
        // Create storage directory if it doesn't exist
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                crate::Error::Config(format!("Failed to create memory storage dir: {}", e))
            })?;
        }

        // Load existing memories
        let memories = if storage_path.exists() {
            let content = fs::read_to_string(&storage_path).map_err(|e| {
                crate::Error::Config(format!("Failed to read memory storage: {}", e))
            })?;
            serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };

        info!(
            "🧠 Initialized Memory tool with {} memories",
            memories.len()
        );
        Ok(Self {
            storage_path,
            memories: Arc::new(RwLock::new(memories)),
            max_results: 5,
        })
    }

    /// Save a memory
    async fn save_memory(&self, text: &str, tags: Vec<String>) -> ToolResult {
        debug!("Saving memory: {}", text);

        // Generate simple embedding (bag-of-words for now, can be improved with ONNX model)
        let embedding = self.generate_simple_embedding(text);

        let memory = MemoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            text: text.to_string(),
            embedding,
            timestamp: chrono::Local::now(),
            metadata: tags
                .into_iter()
                .enumerate()
                .map(|(i, tag)| (format!("tag_{}", i), tag))
                .collect(),
        };

        let mut memories = self.memories.write().await;
        memories.push(memory);

        // Persist to disk
        self.save_to_disk(&memories)?;

        info!("✅ Saved memory (total: {})", memories.len());
        Ok(ToolOutput::success(format!(
            "Memory saved successfully. Total memories: {}",
            memories.len()
        )))
    }

    /// Search memories
    async fn search_memory(&self, query: &str) -> ToolResult {
        debug!("Searching memories for: {}", query);

        let memories = self.memories.read().await;

        if memories.is_empty() {
            return Ok(ToolOutput::success("No memories saved yet".to_string()));
        }

        // Generate query embedding
        let query_embedding = self.generate_simple_embedding(query);

        // Calculate cosine similarity with all memories
        let mut scored_memories: Vec<(f32, &MemoryEntry)> = memories
            .iter()
            .map(|memory| {
                let similarity = self.cosine_similarity(&query_embedding, &memory.embedding);
                (similarity, memory)
            })
            .collect();

        // Sort by similarity (descending)
        scored_memories.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        let results: Vec<String> = scored_memories
            .iter()
            .take(self.max_results)
            .filter(|(score, _)| *score > 0.1) // Filter low similarity
            .map(|(score, memory)| {
                format!(
                    "[{:.2}] {} ({})",
                    score,
                    memory.text,
                    memory.timestamp.format("%Y-%m-%d %H:%M")
                )
            })
            .collect();

        if results.is_empty() {
            Ok(ToolOutput::success(
                "No relevant memories found".to_string(),
            ))
        } else {
            info!("📚 Found {} relevant memories", results.len());
            Ok(ToolOutput::with_data(
                format!(
                    "Found {} relevant memories:\n{}",
                    results.len(),
                    results.join("\n")
                ),
                serde_json::json!({ "memories": results }),
            ))
        }
    }

    /// Generate simple bag-of-words embedding
    /// TODO: Replace with proper ONNX embedding model (e.g., all-MiniLM-L6-v2)
    fn generate_simple_embedding(&self, text: &str) -> Vec<f32> {
        // Simple word hashing for demo
        let lowercase_text = text.to_lowercase();
        let words: Vec<&str> = lowercase_text.split_whitespace().collect();

        let mut embedding = vec![0.0f32; 384]; // Standard embedding size

        for word in words {
            let hash = self.simple_hash(word) % 384;
            embedding[hash] += 1.0;
        }

        // Normalize
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in &mut embedding {
                *val /= magnitude;
            }
        }

        embedding
    }

    /// Simple string hash
    fn simple_hash(&self, s: &str) -> usize {
        s.bytes().map(|b| b as usize).sum()
    }

    /// Cosine similarity between two vectors
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a > 0.0 && magnitude_b > 0.0 {
            dot_product / (magnitude_a * magnitude_b)
        } else {
            0.0
        }
    }

    /// Save memories to disk
    fn save_to_disk(&self, memories: &[MemoryEntry]) -> ToolResult {
        let json = serde_json::to_string_pretty(memories).map_err(|e| ToolError {
            message: format!("Failed to serialize memories: {}", e),
            kind: ToolErrorKind::ExecutionFailed,
        })?;

        std::fs::write(&self.storage_path, json).map_err(|e| ToolError {
            message: format!("Failed to write memories to disk: {}", e),
            kind: ToolErrorKind::ExecutionFailed,
        })?;

        Ok(ToolOutput::success("Memories saved to disk".to_string()))
    }
}

#[async_trait::async_trait]
impl Tool for MemoryTool {
    fn name(&self) -> &str {
        "memory"
    }

    fn description(&self) -> &str {
        "Save important information to long-term memory or search through saved memories. Use 'save' to store new information, 'search' to find relevant memories."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "action".to_string(),
                description: "Action to perform: 'save' or 'search'".to_string(),
                param_type: "string".to_string(),
                required: true,
            },
            ToolParameter {
                name: "text".to_string(),
                description: "Text to save (for save action) or search query (for search action)"
                    .to_string(),
                param_type: "string".to_string(),
                required: true,
            },
            ToolParameter {
                name: "tags".to_string(),
                description: "Tags for categorization (comma-separated, optional)".to_string(),
                param_type: "string".to_string(),
                required: false,
            },
        ]
    }

    async fn execute(&self, args: HashMap<String, serde_json::Value>) -> ToolResult {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError {
                message: "Missing required parameter: action".to_string(),
                kind: ToolErrorKind::InvalidArguments,
            })?;

        let text = args
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError {
                message: "Missing required parameter: text".to_string(),
                kind: ToolErrorKind::InvalidArguments,
            })?;

        match action {
            "save" => {
                let tags = args
                    .get("tags")
                    .and_then(|v| v.as_str())
                    .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
                    .unwrap_or_default();

                self.save_memory(text, tags).await
            }
            "search" => self.search_memory(text).await,
            _ => Err(ToolError {
                message: format!("Invalid action: {}. Must be 'save' or 'search'", action),
                kind: ToolErrorKind::InvalidArguments,
            }),
        }
    }
}
