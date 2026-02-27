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

                // Tiny Multilingual GGML
                ModelMetadata {
                    id: "ggml-tiny".to_string(),
                    name: "Whisper Tiny Multilingual (GGML)".to_string(),
                    size: ModelSize::Tiny,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 75 * 1024 * 1024, // ~75 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-tiny.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 32.0,
                    memory_mb: 200,
                    description: "Fastest multilingual model. Supports 99 languages. Good for real-time dictation.".to_string(),
                },

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
                    description: "Fastest English-only model. Optimized for English transcription.".to_string(),
                },

                // Base Multilingual GGML
                ModelMetadata {
                    id: "ggml-base".to_string(),
                    name: "Whisper Base Multilingual (GGML)".to_string(),
                    size: ModelSize::Base,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 142 * 1024 * 1024, // ~142 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-base.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 16.0,
                    memory_mb: 300,
                    description: "Best balance of speed and accuracy for multiple languages. Supports 99 languages.".to_string(),
                },

                // Base English-only GGML
                ModelMetadata {
                    id: "ggml-base.en".to_string(),
                    name: "Whisper Base English (GGML)".to_string(),
                    size: ModelSize::Base,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::GGML,
                    size_bytes: 142 * 1024 * 1024, // ~142 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-base.en.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 16.0,
                    memory_mb: 300,
                    description: "Best balance of speed and accuracy. Recommended for English users.".to_string(),
                },

                // Small Multilingual GGML
                ModelMetadata {
                    id: "ggml-small".to_string(),
                    name: "Whisper Small Multilingual (GGML)".to_string(),
                    size: ModelSize::Small,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 466 * 1024 * 1024, // ~466 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-small.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 8.0,
                    memory_mb: 600,
                    description: "Higher accuracy for multiple languages. Still fast enough for real-time use.".to_string(),
                },

                // Small English-only GGML
                ModelMetadata {
                    id: "ggml-small.en".to_string(),
                    name: "Whisper Small English (GGML)".to_string(),
                    size: ModelSize::Small,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::GGML,
                    size_bytes: 466 * 1024 * 1024, // ~466 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-small.en.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 8.0,
                    memory_mb: 600,
                    description: "Higher accuracy for English. Still fast enough for real-time use.".to_string(),
                },

                // Medium Multilingual GGML
                ModelMetadata {
                    id: "ggml-medium".to_string(),
                    name: "Whisper Medium Multilingual (GGML)".to_string(),
                    size: ModelSize::Medium,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 1500 * 1024 * 1024, // ~1.5 GB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-medium.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 4.0,
                    memory_mb: 1200,
                    description: "High accuracy for multiple languages. Slower but more accurate.".to_string(),
                },

                // Medium English-only GGML
                ModelMetadata {
                    id: "ggml-medium.en".to_string(),
                    name: "Whisper Medium English (GGML)".to_string(),
                    size: ModelSize::Medium,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::GGML,
                    size_bytes: 1500 * 1024 * 1024, // ~1.5 GB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-medium.en.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 4.0,
                    memory_mb: 1200,
                    description: "High accuracy for English. Slower but more accurate.".to_string(),
                },

                // Large-v2 Multilingual GGML
                ModelMetadata {
                    id: "ggml-large-v2".to_string(),
                    name: "Whisper Large v2 Multilingual (GGML)".to_string(),
                    size: ModelSize::Large,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 2900 * 1024 * 1024, // ~2.9 GB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-large-v2.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 2.0,
                    memory_mb: 2500,
                    description: "Best accuracy for multiple languages. Requires significant resources.".to_string(),
                },

                // Large-v3 Multilingual GGML
                ModelMetadata {
                    id: "ggml-large-v3".to_string(),
                    name: "Whisper Large v3 Multilingual (GGML)".to_string(),
                    size: ModelSize::Large,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 2900 * 1024 * 1024, // ~2.9 GB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-large-v3.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 2.0,
                    memory_mb: 2500,
                    description: "Latest large model with improved accuracy. Best for demanding use cases.".to_string(),
                },

                // Large-v3 Turbo Multilingual GGML
                ModelMetadata {
                    id: "ggml-large-v3-turbo".to_string(),
                    name: "Whisper Large v3 Turbo Multilingual (GGML)".to_string(),
                    size: ModelSize::Large,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 1500 * 1024 * 1024, // ~1.5 GB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-large-v3-turbo.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 3.5,
                    memory_mb: 1500,
                    description: "Faster variant of large-v3 with comparable accuracy. Best large model for real-time use.".to_string(),
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
