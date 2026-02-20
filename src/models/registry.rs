//! Model Registry
//!
//! Central registry of available Whisper models with metadata.

use serde::{Deserialize, Serialize};

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
                    speed_factor: 8.0,
                    memory_mb: 600,
                    description: "Higher accuracy, still fast enough for real-time use.".to_string(),
                },
                
                // ============================================================
                // ONNX Models (Alternative backend)
                // ============================================================
                
                // Tiny English-only (fastest, lowest quality)
                ModelMetadata {
                    id: "whisper-tiny.en".to_string(),
                    name: "Whisper Tiny English (ONNX)".to_string(),
                    size: ModelSize::Tiny,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::ONNX,
                    size_bytes: 75 * 1024 * 1024, // ~75 MB
                    hf_repo: "onnx-community/whisper-tiny.en".to_string(),
                    files: vec![
                        "onnx/decoder_model_merged.onnx".to_string(),
                        "onnx/encoder_model.onnx".to_string(),
                    ],
                    speed_factor: 32.0, // 32x faster than real-time on CPU
                    memory_mb: 200,
                    description: "ONNX backend (experimental). Use GGML models instead."
                        .to_string(),
                },
                // Base English-only (good balance)
                ModelMetadata {
                    id: "whisper-base.en".to_string(),
                    name: "Whisper Base English (ONNX)".to_string(),
                    size: ModelSize::Base,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::ONNX,
                    size_bytes: 140 * 1024 * 1024, // ~140 MB
                    hf_repo: "onnx-community/whisper-base.en".to_string(),
                    files: vec![
                        "onnx/decoder_model_merged.onnx".to_string(),
                        "onnx/encoder_model.onnx".to_string(),
                    ],
                    speed_factor: 16.0, // 16x faster than real-time
                    memory_mb: 300,
                    description: "ONNX backend (experimental). Use GGML models instead."
                        .to_string(),
                },
                // Small English-only (better quality)
                ModelMetadata {
                    id: "whisper-small.en".to_string(),
                    name: "Whisper Small English (ONNX)".to_string(),
                    size: ModelSize::Small,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::ONNX,
                    size_bytes: 470 * 1024 * 1024, // ~470 MB
                    hf_repo: "onnx-community/whisper-small.en".to_string(),
                    files: vec![
                        "onnx/decoder_model_merged.onnx".to_string(),
                        "onnx/encoder_model.onnx".to_string(),
                    ],
                    speed_factor: 8.0, // 8x faster than real-time
                    memory_mb: 600,
                    description: "ONNX backend (experimental). Use GGML models instead."
                        .to_string(),
                },
                // Medium English-only (high quality)
                ModelMetadata {
                    id: "whisper-medium.en".to_string(),
                    name: "Whisper Medium English (ONNX)".to_string(),
                    size: ModelSize::Medium,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::ONNX,
                    size_bytes: 1500 * 1024 * 1024, // ~1.5 GB
                    hf_repo: "onnx-community/whisper-medium.en".to_string(),
                    files: vec![
                        "onnx/decoder_model_merged.onnx".to_string(),
                        "onnx/encoder_model.onnx".to_string(),
                    ],
                    speed_factor: 4.0, // 4x faster than real-time
                    memory_mb: 1200,
                    description: "ONNX backend (experimental). Use GGML models instead."
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
        assert!(registry.get_model("whisper-base.en").is_some());
        assert!(registry.get_model("nonexistent").is_none());
    }

    #[test]
    fn test_download_urls() {
        let registry = ModelRegistry::new();
        let model = registry.get_model("whisper-tiny.en").unwrap();
        let urls = model.download_urls();
        assert!(!urls.is_empty());
        assert!(urls[0].1.contains("huggingface.co"));
    }
}
