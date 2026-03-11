//! Model Registry
//!
//! Central registry of available Whisper models with metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    /// Speech-to-Text
    STT,
    /// Large Language Model
    LLM,
    /// Text-to-Speech
    TTS,
}

/// Model format/backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    /// GGML format (whisper.cpp)
    GGML,
    /// ONNX format
    ONNX,
    /// PyTorch format
    PyTorch,
    /// GGUF format (llama.cpp)
    GGUF,
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

    /// Model type (STT, LLM, or TTS)
    pub model_type: ModelType,

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

    /// GPU recommended (not required)
    #[serde(default)]
    pub gpu_recommended: bool,

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
                    model_type: ModelType::STT,
                    size: ModelSize::Tiny,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 75 * 1024 * 1024, // ~75 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-tiny.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 32.0,
                    memory_mb: 200,
                    gpu_recommended: false,
                    description: "Fastest multilingual model. Supports 99 languages. Good for real-time dictation.".to_string(),
                },

                // Tiny English-only GGML
                ModelMetadata {
                    id: "ggml-tiny.en".to_string(),
                    name: "Whisper Tiny English (GGML)".to_string(),
                    model_type: ModelType::STT,
                    size: ModelSize::Tiny,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::GGML,
                    size_bytes: 75 * 1024 * 1024, // ~75 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-tiny.en.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 32.0,
                    memory_mb: 200,
                    gpu_recommended: false,
                    description: "Fastest English-only model. Optimized for English transcription.".to_string(),
                },

                // Base Multilingual GGML
                ModelMetadata {
                    id: "ggml-base".to_string(),
                    name: "Whisper Base Multilingual (GGML)".to_string(),
                    model_type: ModelType::STT,
                    size: ModelSize::Base,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 142 * 1024 * 1024, // ~142 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-base.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 16.0,
                    memory_mb: 300,
                    gpu_recommended: false,
                    description: "Best balance of speed and accuracy for multiple languages. Supports 99 languages.".to_string(),
                },

                // Base English-only GGML
                ModelMetadata {
                    id: "ggml-base.en".to_string(),
                    name: "Whisper Base English (GGML)".to_string(),
                    model_type: ModelType::STT,
                    size: ModelSize::Base,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::GGML,
                    size_bytes: 142 * 1024 * 1024, // ~142 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-base.en.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 16.0,
                    memory_mb: 300,
                    gpu_recommended: false,
                    description: "Best balance of speed and accuracy. Recommended for English users.".to_string(),
                },

                // Small Multilingual GGML
                ModelMetadata {
                    id: "ggml-small".to_string(),
                    name: "Whisper Small Multilingual (GGML)".to_string(),
                    model_type: ModelType::STT,
                    size: ModelSize::Small,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 466 * 1024 * 1024, // ~466 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-small.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 8.0,
                    memory_mb: 600,
                    gpu_recommended: false,
                    description: "Higher accuracy for multiple languages. Still fast enough for real-time use.".to_string(),
                },

                // Small English-only GGML
                ModelMetadata {
                    id: "ggml-small.en".to_string(),
                    name: "Whisper Small English (GGML)".to_string(),
                    model_type: ModelType::STT,
                    size: ModelSize::Small,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::GGML,
                    size_bytes: 466 * 1024 * 1024, // ~466 MB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-small.en.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 8.0,
                    memory_mb: 600,
                    gpu_recommended: false,
                    description: "Higher accuracy for English. Still fast enough for real-time use.".to_string(),
                },

                // Medium Multilingual GGML
                ModelMetadata {
                    id: "ggml-medium".to_string(),
                    name: "Whisper Medium Multilingual (GGML)".to_string(),
                    model_type: ModelType::STT,
                    size: ModelSize::Medium,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 1500 * 1024 * 1024, // ~1.5 GB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-medium.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 4.0,
                    memory_mb: 1200,
                    gpu_recommended: false,
                    description: "High accuracy for multiple languages. Slower but more accurate.".to_string(),
                },

                // Medium English-only GGML
                ModelMetadata {
                    id: "ggml-medium.en".to_string(),
                    name: "Whisper Medium English (GGML)".to_string(),
                    model_type: ModelType::STT,
                    size: ModelSize::Medium,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::GGML,
                    size_bytes: 1500 * 1024 * 1024, // ~1.5 GB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-medium.en.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 4.0,
                    memory_mb: 1200,
                    gpu_recommended: false,
                    description: "High accuracy for English. Slower but more accurate.".to_string(),
                },

                // Large-v2 Multilingual GGML
                ModelMetadata {
                    id: "ggml-large-v2".to_string(),
                    name: "Whisper Large v2 Multilingual (GGML)".to_string(),
                    model_type: ModelType::STT,
                    size: ModelSize::Large,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 2900 * 1024 * 1024, // ~2.9 GB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-large-v2.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 2.0,
                    memory_mb: 2500,
                    gpu_recommended: true,
                    description: "Best accuracy for multiple languages. Requires significant resources.".to_string(),
                },

                // Large-v3 Multilingual GGML
                ModelMetadata {
                    id: "ggml-large-v3".to_string(),
                    name: "Whisper Large v3 Multilingual (GGML)".to_string(),
                    model_type: ModelType::STT,
                    size: ModelSize::Large,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 2900 * 1024 * 1024, // ~2.9 GB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-large-v3.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 2.0,
                    memory_mb: 2500,
                    gpu_recommended: true,
                    description: "Latest large model with improved accuracy. Best for demanding use cases.".to_string(),
                },

                // Large-v3 Turbo Multilingual GGML
                ModelMetadata {
                    id: "ggml-large-v3-turbo".to_string(),
                    name: "Whisper Large v3 Turbo Multilingual (GGML)".to_string(),
                    model_type: ModelType::STT,
                    size: ModelSize::Large,
                    variant: ModelVariant::Multilingual,
                    format: ModelFormat::GGML,
                    size_bytes: 1500 * 1024 * 1024, // ~1.5 GB
                    hf_repo: "ggerganov/whisper.cpp".to_string(),
                    files: vec!["ggml-large-v3-turbo.bin".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 3.5,
                    memory_mb: 1500,
                    gpu_recommended: false,
                    description: "Faster variant of large-v3 with comparable accuracy. Best large model for real-time use.".to_string(),
                },

                // ============================================================
                // ONNX Models (NVIDIA Parakeet - Production Ready)
                // ============================================================

                // Parakeet CTC 0.6B - Multilingual (INT8 Quantized)
                ModelMetadata {
                    id: "parakeet-ctc-0.6b".to_string(),
                    name: "NVIDIA Parakeet CTC 0.6B (Multilingual)".to_string(),
                    model_type: ModelType::STT,
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
                    gpu_recommended: false,
                    description: "High-performance multilingual ASR (INT8 quantized). Supports 100+ languages with CTC architecture. Optimized for CPU inference."
                        .to_string(),
                },

                // ============================================================
                // LLM Models
                // ============================================================

                // LFM2 1.2B Tool - Q4_K_M (recommended, good balance)
                ModelMetadata {
                    id: "lfm2-1.2b-tool-q4".to_string(),
                    name: "Liquid LFM2 1.2B Tool Q4_K_M (fast)".to_string(),
                    model_type: ModelType::LLM,
                    size: ModelSize::Base,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::GGUF,
                    size_bytes: 731 * 1024 * 1024, // ~731 MB
                    hf_repo: "LiquidAI/LFM2-1.2B-Tool-GGUF".to_string(),
                    files: vec!["LFM2-1.2B-Tool-Q4_K_M.gguf".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 40.0, // ~40 tokens/sec on CPU
                    memory_mb: 1200,
                    gpu_recommended: false,
                    description: "Fast Q4 quantization with tool calling support. Good balance of speed and quality. Recommended for most users.".to_string(),
                },

                // LFM2 1.2B Tool - Q5_K_M (better quality)
                ModelMetadata {
                    id: "lfm2-1.2b-tool-q5".to_string(),
                    name: "Liquid LFM2 1.2B Tool Q5_K_M (balanced)".to_string(),
                    model_type: ModelType::LLM,
                    size: ModelSize::Base,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::GGUF,
                    size_bytes: 843 * 1024 * 1024, // ~843 MB
                    hf_repo: "LiquidAI/LFM2-1.2B-Tool-GGUF".to_string(),
                    files: vec!["LFM2-1.2B-Tool-Q5_K_M.gguf".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 35.0, // ~35 tokens/sec on CPU
                    memory_mb: 1500,
                    gpu_recommended: false,
                    description: "Q5 quantization with tool calling support. Better quality than Q4 with slightly larger size. Good for quality-focused users.".to_string(),
                },

                // LFM2 1.2B Tool - Q8_0 (best quality)
                ModelMetadata {
                    id: "lfm2-1.2b-tool-q8".to_string(),
                    name: "Liquid LFM2 1.2B Tool Q8_0 (quality)".to_string(),
                    model_type: ModelType::LLM,
                    size: ModelSize::Base,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::GGUF,
                    size_bytes: 1246 * 1024 * 1024, // ~1.25 GB
                    hf_repo: "LiquidAI/LFM2-1.2B-Tool-GGUF".to_string(),
                    files: vec!["LFM2-1.2B-Tool-Q8_0.gguf".to_string()],
                    file_sha256: HashMap::new(),
                    speed_factor: 30.0, // ~30 tokens/sec on CPU
                    memory_mb: 2000,
                    gpu_recommended: false,
                    description: "High quality Q8 quantization with tool calling support. Best quality, larger size. For users who prioritize response quality.".to_string(),
                },

                // ============================================================
                // TTS Models
                // ============================================================

                // Kokoro TTS - Q8F16 (recommended, best quality/size balance)
                ModelMetadata {
                    id: "kokoro-tts-q8f16".to_string(),
                    name: "Kokoro TTS Q8F16 (recommended)".to_string(),
                    model_type: ModelType::TTS,
                    size: ModelSize::Tiny,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::ONNX,
                    size_bytes: 86 * 1024 * 1024, // ~86 MB
                    hf_repo: "onnx-community/Kokoro-82M-ONNX".to_string(),
                    files: vec![
                        "onnx/model_q8f16.onnx".to_string(),
                        "config.json".to_string(),
                        "tokenizer.json".to_string(),
                        "tokenizer_config.json".to_string(),
                        "voices/af.bin".to_string(),
                        "voices/af_bella.bin".to_string(),
                        "voices/af_nicole.bin".to_string(),
                        "voices/af_sarah.bin".to_string(),
                        "voices/af_sky.bin".to_string(),
                        "voices/am_adam.bin".to_string(),
                        "voices/am_michael.bin".to_string(),
                        "voices/bf_emma.bin".to_string(),
                        "voices/bf_isabella.bin".to_string(),
                        "voices/bm_george.bin".to_string(),
                        "voices/bm_lewis.bin".to_string(),
                    ],
                    file_sha256: HashMap::new(),
                    speed_factor: 0.3, // RTF < 0.5 (faster than real-time)
                    memory_mb: 200,
                    gpu_recommended: false,
                    description: "Q8F16 quantization. Best balance of quality and size. Multiple voices available. Recommended for most users.".to_string(),
                },

                // Kokoro TTS - FP16 (best quality)
                ModelMetadata {
                    id: "kokoro-tts-fp16".to_string(),
                    name: "Kokoro TTS FP16 (quality)".to_string(),
                    model_type: ModelType::TTS,
                    size: ModelSize::Tiny,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::ONNX,
                    size_bytes: 163 * 1024 * 1024, // ~163 MB
                    hf_repo: "onnx-community/Kokoro-82M-ONNX".to_string(),
                    files: vec![
                        "onnx/model_fp16.onnx".to_string(),
                        "config.json".to_string(),
                        "tokenizer.json".to_string(),
                        "tokenizer_config.json".to_string(),
                        "voices/af.bin".to_string(),
                        "voices/af_bella.bin".to_string(),
                        "voices/af_nicole.bin".to_string(),
                        "voices/af_sarah.bin".to_string(),
                        "voices/af_sky.bin".to_string(),
                        "voices/am_adam.bin".to_string(),
                        "voices/am_michael.bin".to_string(),
                        "voices/bf_emma.bin".to_string(),
                        "voices/bf_isabella.bin".to_string(),
                        "voices/bm_george.bin".to_string(),
                        "voices/bm_lewis.bin".to_string(),
                    ],
                    file_sha256: HashMap::new(),
                    speed_factor: 0.4, // RTF < 0.5 (faster than real-time)
                    memory_mb: 300,
                    gpu_recommended: false,
                    description: "Full FP16 precision. Highest quality but larger size. For users who prioritize voice quality.".to_string(),
                },

                // Kokoro TTS - Q4F16 (fastest)
                ModelMetadata {
                    id: "kokoro-tts-q4f16".to_string(),
                    name: "Kokoro TTS Q4F16 (fast)".to_string(),
                    model_type: ModelType::TTS,
                    size: ModelSize::Tiny,
                    variant: ModelVariant::EnglishOnly,
                    format: ModelFormat::ONNX,
                    size_bytes: 154 * 1024 * 1024, // ~154 MB
                    hf_repo: "onnx-community/Kokoro-82M-ONNX".to_string(),
                    files: vec![
                        "onnx/model_q4f16.onnx".to_string(),
                        "config.json".to_string(),
                        "tokenizer.json".to_string(),
                        "tokenizer_config.json".to_string(),
                        "voices/af.bin".to_string(),
                        "voices/af_bella.bin".to_string(),
                        "voices/af_nicole.bin".to_string(),
                        "voices/af_sarah.bin".to_string(),
                        "voices/af_sky.bin".to_string(),
                        "voices/am_adam.bin".to_string(),
                        "voices/am_michael.bin".to_string(),
                        "voices/bf_emma.bin".to_string(),
                        "voices/bf_isabella.bin".to_string(),
                        "voices/bm_george.bin".to_string(),
                        "voices/bm_lewis.bin".to_string(),
                    ],
                    file_sha256: HashMap::new(),
                    speed_factor: 0.2, // RTF < 0.3 (very fast)
                    memory_mb: 250,
                    gpu_recommended: false,
                    description: "Q4F16 quantization. Fastest inference with good quality. For users who need lowest latency.".to_string(),
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

    /// Get models by type
    pub fn list_models_by_type(&self, model_type: ModelType) -> Vec<&ModelMetadata> {
        self.models
            .iter()
            .filter(|m| m.model_type == model_type)
            .collect()
    }

    /// Get recommended LLM model
    pub fn recommended_llm(&self) -> &ModelMetadata {
        self.get_model("lfm2-1.2b-tool-q4")
            .expect("lfm2-1.2b-tool-q4 model should exist")
    }

    /// Get recommended TTS model
    pub fn recommended_tts(&self) -> &ModelMetadata {
        self.get_model("kokoro-tts-q8f16")
            .expect("kokoro-tts-q8f16 model should exist")
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
