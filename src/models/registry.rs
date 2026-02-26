//! Model Registry
//!
//! Central registry of available Whisper models with metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model format/backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    /// GGML format (whisper.cpp)
    GGML,
    /// ONNX format
    ONNX,
    /// PyTorch format
    PyTorch,
}

/// Available Whisper model sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelSize {
    Tiny,
    Base,
    Small,
    Medium,
    Large,
}

impl ModelSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelSize::Tiny => "tiny",
            ModelSize::Base => "base",
            ModelSize::Small => "small",
            ModelSize::Medium => "medium",
            ModelSize::Large => "large",
        }
    }
}

/// Model variant (multilingual vs English-only)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelVariant {
    Multilingual,
    EnglishOnly,
}

/// Model metadata from registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Model identifier (e.g., "whisper-tiny.en")
    pub id: String,

    /// Display name
    pub name: String,

    /// Model size category
    pub size: ModelSize,

    /// Model variant
    pub variant: ModelVariant,

    /// Model format/backend
    pub format: ModelFormat,

    /// Approximate size in bytes
    pub size_bytes: u64,

    /// Hugging Face repository
    pub hf_repo: String,

    /// Required files to download
    pub files: Vec<String>,

    /// Optional SHA256 checksums keyed by file path
    #[serde(default)]
    pub file_sha256: HashMap<String, String>,

    /// Speed factor (relative to real-time, 1.0 = real-time)
    pub speed_factor: f32,

    /// Memory requirements in MB
    pub memory_mb: u32,

    /// Description
    pub description: String,
}

impl ModelMetadata {
    /// Get download URLs for all required files
    pub fn download_urls(&self) -> Vec<(String, String)> {
        self.files
            .iter()
            .map(|file| {
                let url = format!(
                    "https://huggingface.co/{}/resolve/main/{}",
                    self.hf_repo, file
                );
                (file.clone(), url)
            })
            .collect()
    }
}

/// Model registry with all available models
pub struct ModelRegistry {
    models: Vec<ModelMetadata>,
}

impl ModelRegistry {
    /// Create a new model registry with all available models
    pub fn new() -> Self {
        Self {
            models: vec![
                // ============================================================
                // GGML Models (whisper.cpp) - RECOMMENDED
                // ============================================================

                // Tiny English-only GGML
                ModelMetadata {
                    id: "ggml-tiny.en".to_string(),
                    name: "Whisper Tiny English (GGML)".to_string(),
                    size: ModelSize::Tiny,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::GGML,
                    size_bytes: 75 * 1024 * 1024, // ~75 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-tiny.en.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 32.0,
                    memory_mb: 200,
                    description: "Fastest model using whisper.cpp. English only. Recommended for real-time dictation.".to_string(),
                },

                // Base English-only GGML
                ModelMetadata {
                    id: "ggml-base.en".to_string(),
                    name: "Whisper Base English (GGML)".to_string(),
                    size: ModelSize::Base,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::GGML,
                    size_bytes: 140 * 1024 * 1024, // ~140 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-base.en.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 16.0,
                    memory_mb: 300,
                    description: "Best balance of speed and accuracy. Recommended for most users.".to_string(),
                },

                // Small English-only GGML
                ModelMetadata {
                    id: "ggml-small.en".to_string(),
                    name: "Whisper Small English (GGML)".to_string(),
                    size: ModelSize::Small,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::GGML,
                    size_bytes: 470 * 1024 * 1024, // ~470 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-small.en.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 8.0,
                    memory_mb: 600,
                    description: "Higher accuracy, still fast enough for real-time use.".to_string(),
                },

                // ============================================================
                // ONNX Models (NVIDIA Parakeet - Production Ready)
                // ============================================================

                // Parakeet CTC 0.6B - Multilingual (INT8 Quantized)
                ModelMetadata {
                    id: "parakeet-ctc-0.6b".to_string(),
                    name: "NVIDIA Parakeet CTC 0.6B (Multilingual)".to_string(),
                    size: ModelSize::Base,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::ONNX,
                    size_bytes: 653 * 1024 * 1024, // ~653 MB (INT8 quantized)
                    hf_repo: "istupakov/parakeet-ctc-0.6b-onnx".to_string(),
                    files: vec![
                        "model.int8.onnx".to_string(),
                        "vocab.txt".to_string(),
                        "config.json".to_string(),
                    ],
                    file_sha256: HashMap::new(),
                    speed_factor: 60.0, // 60x faster than real-time on CPU
                    memory_mb: 400,
                    description: "High-performance multilingual ASR (INT8 quantized). Supports 100+ languages with CTC architecture. Optimized for CPU inference."
                        .to_string(),
                },
            ],
        }
    }

    /// Get all available models
    pub fn list_models(&self) -> &[ModelMetadata] {
        &self.models
    }

    /// Find a model by ID
    pub fn get_model(&self, id: &str) -> Option<&ModelMetadata> {
        self.models.iter().find(|m| m.id == id)
    }

    /// Get recommended model (ggml-base.en for most users)
    pub fn recommended(&self) -> &ModelMetadata {
        self.get_model("ggml-base.en")
            .expect("ggml-base.en model should exist")
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let registry = ModelRegistry::new();
        assert!(!registry.list_models().is_empty());
        assert!(registry.get_model("ggml-base.en").is_some());
        assert!(registry.get_model("parakeet-ctc-0.6b").is_some());
        assert!(registry.get_model("nonexistent").is_none());
    }

    #[test]
    fn test_download_urls() {
        let registry = ModelRegistry::new();
        let model = registry.get_model("ggml-tiny.en").unwrap();
        let urls = model.download_urls();
        assert!(!urls.is_empty());
        assert!(urls[0].1.contains("huggingface.co"));
    }
}
