//! Kokoro TTS ONNX Backend
//!
//! High-quality, fast text-to-speech using Kokoro-82M model.
//! Supports multiple voices (American/British, male/female).

use super::tts_runtime::*;
use crate::Result;
use ort::{
    ep,
    session::{Session, builder::GraphOptimizationLevel},
    value::Value,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Voice style data loaded from .bin files
#[derive(Debug, Clone)]
struct VoiceStyleData {
    /// Style vectors: [context_length, 256]
    /// Each context length (0-511) has a corresponding 256-dim style vector
    vectors: Vec<f32>,
    info: VoiceInfo,
}

impl VoiceStyleData {
    /// Get style vector for a given token count
    fn get_style_for_length(&self, token_count: usize) -> Result<Vec<f32>> {
        // Kokoro style vectors are organized as [512, 256]
        // We need the vector at index min(token_count, 511)
        let idx = token_count.min(511);
        let start = idx * 256;
        let end = start + 256;

        if end > self.vectors.len() {
            return Err(crate::Error::Model(format!(
                "Style vector index out of bounds: {} (have {} vectors)",
                idx,
                self.vectors.len() / 256
            )));
        }

        Ok(self.vectors[start..end].to_vec())
    }
}

/// Kokoro TTS Backend
pub struct TtsKokoro {
    session: Option<Session>,
    config: Option<TtsRuntimeConfig>,
    voices: HashMap<String, VoiceStyleData>,
    vocab: Option<HashMap<String, i64>>,
    model_dir: Option<PathBuf>,
}

impl TtsKokoro {
    /// Create a new Kokoro TTS backend
    pub fn new() -> Self {
        info!("Initializing Kokoro TTS backend");

        Self {
            session: None,
            config: None,
            voices: HashMap::new(),
            vocab: None,
            model_dir: None,
        }
    }

    /// Initialize available voices metadata
    fn initialize_voice_list() -> Vec<VoiceInfo> {
        vec![
            VoiceInfo::new(
                "af".to_string(),
                "Default (Female, American)".to_string(),
                "en-US".to_string(),
            )
            .with_gender("female".to_string())
            .with_description("Warm, friendly default voice".to_string()),
            VoiceInfo::new(
                "af_bella".to_string(),
                "Bella (Female, American)".to_string(),
                "en-US".to_string(),
            )
            .with_gender("female".to_string())
            .with_description("Clear, professional voice".to_string()),
            VoiceInfo::new(
                "af_nicole".to_string(),
                "Nicole (Female, American)".to_string(),
                "en-US".to_string(),
            )
            .with_gender("female".to_string())
            .with_description("Energetic, youthful voice".to_string()),
            VoiceInfo::new(
                "af_sarah".to_string(),
                "Sarah (Female, American)".to_string(),
                "en-US".to_string(),
            )
            .with_gender("female".to_string())
            .with_description("Calm, soothing voice".to_string()),
            VoiceInfo::new(
                "af_sky".to_string(),
                "Sky (Female, American)".to_string(),
                "en-US".to_string(),
            )
            .with_gender("female".to_string())
            .with_description("Bright, clear voice".to_string()),
            VoiceInfo::new(
                "am_adam".to_string(),
                "Adam (Male, American)".to_string(),
                "en-US".to_string(),
            )
            .with_gender("male".to_string())
            .with_description("Deep, authoritative voice".to_string()),
            VoiceInfo::new(
                "am_michael".to_string(),
                "Michael (Male, American)".to_string(),
                "en-US".to_string(),
            )
            .with_gender("male".to_string())
            .with_description("Friendly, conversational voice".to_string()),
            VoiceInfo::new(
                "bf_emma".to_string(),
                "Emma (Female, British)".to_string(),
                "en-GB".to_string(),
            )
            .with_gender("female".to_string())
            .with_description("Elegant, sophisticated voice".to_string()),
            VoiceInfo::new(
                "bf_isabella".to_string(),
                "Isabella (Female, British)".to_string(),
                "en-GB".to_string(),
            )
            .with_gender("female".to_string())
            .with_description("Refined, articulate voice".to_string()),
            VoiceInfo::new(
                "bm_george".to_string(),
                "George (Male, British)".to_string(),
                "en-GB".to_string(),
            )
            .with_gender("male".to_string())
            .with_description("Distinguished, commanding voice".to_string()),
            VoiceInfo::new(
                "bm_lewis".to_string(),
                "Lewis (Male, British)".to_string(),
                "en-GB".to_string(),
            )
            .with_gender("male".to_string())
            .with_description("Warm, approachable voice".to_string()),
        ]
    }

    /// Resolve model directory from cache
    fn resolve_model_dir(&self, model_id: &str) -> Result<PathBuf> {
        // Check if it's an absolute path
        let direct_path = PathBuf::from(model_id);
        if direct_path.is_absolute() && direct_path.is_dir() {
            info!("Using absolute model directory: {:?}", direct_path);
            return Ok(direct_path);
        }

        // Use platform-appropriate models directory
        let models_dir =
            crate::platform::paths::models_dir().unwrap_or_else(|_| PathBuf::from("./models"));

        // Model directory structure: models/<model_id>/
        let model_dir = models_dir.join(model_id);

        if !model_dir.exists() {
            warn!("Model directory not found at: {:?}", model_dir);
            debug!(
                "Expected structure: {}/{{model.onnx, vocab.json, voices/*.bin}}",
                model_dir.display()
            );
        }

        Ok(model_dir)
    }

    /// Load vocabulary from vocab.json or tokenizer.json
    fn load_vocab(&self, model_dir: &Path) -> Result<HashMap<String, i64>> {
        // Try vocab.json first (old format)
        let vocab_path = model_dir.join("vocab.json");
        if vocab_path.exists() {
            info!("Loading vocabulary from: {:?}", vocab_path);
            let vocab_content = std::fs::read_to_string(&vocab_path)
                .map_err(|e| crate::Error::Model(format!("Failed to read vocab.json: {}", e)))?;

            let vocab: HashMap<String, i64> = serde_json::from_str(&vocab_content)
                .map_err(|e| crate::Error::Model(format!("Failed to parse vocab.json: {}", e)))?;

            info!("✅ Loaded {} phoneme tokens", vocab.len());
            return Ok(vocab);
        }

        // Try tokenizer.json (Kokoro format)
        let tokenizer_path = model_dir.join("tokenizer.json");
        if tokenizer_path.exists() {
            info!("Loading vocabulary from: {:?}", tokenizer_path);
            let tokenizer_content = std::fs::read_to_string(&tokenizer_path).map_err(|e| {
                crate::Error::Model(format!("Failed to read tokenizer.json: {}", e))
            })?;

            // Parse tokenizer.json to extract vocab from .model.vocab
            let tokenizer_json: serde_json::Value = serde_json::from_str(&tokenizer_content)
                .map_err(|e| {
                    crate::Error::Model(format!("Failed to parse tokenizer.json: {}", e))
                })?;

            let vocab_obj = tokenizer_json
                .get("model")
                .and_then(|m| m.get("vocab"))
                .ok_or_else(|| {
                    crate::Error::Model("No vocab found in tokenizer.json".to_string())
                })?;

            let vocab: HashMap<String, i64> =
                serde_json::from_value(vocab_obj.clone()).map_err(|e| {
                    crate::Error::Model(format!("Failed to parse vocab from tokenizer.json: {}", e))
                })?;

            info!("✅ Loaded {} phoneme tokens from tokenizer", vocab.len());
            return Ok(vocab);
        }

        Err(crate::Error::Model(format!(
            "Vocabulary file not found: neither {:?} nor {:?} exist",
            vocab_path, tokenizer_path
        )))
    }

    /// Load voice style vectors from .bin file
    fn load_voice(
        &self,
        model_dir: &Path,
        voice_id: &str,
        voice_info: VoiceInfo,
    ) -> Result<VoiceStyleData> {
        let voice_path = model_dir.join(format!("voices/{}.bin", voice_id));

        if !voice_path.exists() {
            return Err(crate::Error::Model(format!(
                "Voice file not found: {:?}",
                voice_path
            )));
        }

        debug!("Loading voice {} from: {:?}", voice_id, voice_path);

        // Read binary file as f32 array
        let bytes = std::fs::read(&voice_path)
            .map_err(|e| crate::Error::Model(format!("Failed to read voice file: {}", e)))?;

        // Convert bytes to f32 vector
        // Kokoro voices are stored as [512, 256] = 131,072 f32 values = 524,288 bytes
        if bytes.len() != 512 * 256 * 4 {
            warn!(
                "Voice file size mismatch for {}: expected {} bytes, got {}",
                voice_id,
                512 * 256 * 4,
                bytes.len()
            );
        }

        let vectors: Vec<f32> = bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        debug!(
            "✅ Loaded voice {} with {} style vectors ({}x256)",
            voice_id,
            vectors.len() / 256,
            vectors.len() / 256
        );

        Ok(VoiceStyleData {
            vectors,
            info: voice_info,
        })
    }

    /// Simple text normalization (MVP - basic cleanup)
    fn normalize_text(&self, text: &str) -> String {
        // Basic normalization:
        // - Trim whitespace
        // - Convert to lowercase
        // - Remove special characters except basic punctuation
        // - Collapse multiple spaces

        text.trim()
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || " .,!?'-".contains(c) {
                    c
                } else {
                    ' '
                }
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Convert text to phoneme string using espeak-ng
    /// Falls back to passing through text if espeak-ng is not available
    fn phonemize(&self, text: &str) -> Result<String> {
        // Try to use espeak-ng for phonemization
        // espeak-ng -v en-us --ipa -q "text"

        let output = std::process::Command::new("espeak-ng")
            .args(["-v", "en-us", "--ipa", "-q", text])
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let phonemes = String::from_utf8_lossy(&result.stdout).trim().to_string();

                if !phonemes.is_empty() {
                    debug!("Phonemized: '{}' -> '{}'", text, phonemes);
                    return Ok(phonemes);
                }
            }
            Ok(result) => {
                warn!(
                    "espeak-ng failed with status {:?}: {}",
                    result.status,
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            Err(e) => {
                warn!("espeak-ng not available ({}), using fallback", e);
            }
        }

        // Fallback: return normalized text
        // This won't produce perfect phonemes, but Kokoro may still work
        debug!("Using fallback phonemization for: '{}'", text);
        Ok(text.to_string())
    }

    /// Tokenize phonemes to token IDs
    fn tokenize(&self, phonemes: &str) -> Result<Vec<i64>> {
        let vocab = self
            .vocab
            .as_ref()
            .ok_or_else(|| crate::Error::Model("Vocabulary not loaded".to_string()))?;

        let mut tokens = vec![0]; // Start with pad token

        // Split phonemes by character (IPA symbols) and spaces
        for phoneme in phonemes.chars() {
            let phoneme_str = phoneme.to_string();

            // Look up token ID
            if let Some(&token_id) = vocab.get(&phoneme_str) {
                tokens.push(token_id);
            } else if phoneme.is_whitespace() {
                // Space might have special token
                if let Some(&token_id) = vocab.get(" ") {
                    tokens.push(token_id);
                }
            } else {
                // Unknown phoneme - skip or use unknown token
                debug!("Unknown phoneme: '{}'", phoneme);
                if let Some(&unk_id) = vocab.get("<unk>") {
                    tokens.push(unk_id);
                }
            }
        }

        tokens.push(0); // End with pad token

        // Kokoro has max context of 512 tokens
        if tokens.len() > 512 {
            warn!(
                "Token sequence too long ({} tokens), truncating to 512",
                tokens.len()
            );
            tokens.truncate(512);
        }

        debug!("Tokenized to {} tokens", tokens.len());
        Ok(tokens)
    }

    /// Run ONNX inference to synthesize audio
    fn run_inference(&mut self, tokens: &[i64], style: &[f32], speed: f32) -> Result<Vec<f32>> {
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| crate::Error::Model("Model not loaded".to_string()))?;

        debug!(
            "Running inference with {} tokens, style dim {}, speed {}",
            tokens.len(),
            style.len(),
            speed
        );

        // Prepare inputs using ort::Value
        // input_ids: [1, seq_len]
        let input_ids_shape = vec![1, tokens.len()];
        let input_ids_data: Box<[i64]> = tokens.to_vec().into_boxed_slice();
        let input_ids_value = Value::from_array((input_ids_shape.as_slice(), input_ids_data))
            .map_err(|e| {
                crate::Error::Model(format!("Failed to create input_ids tensor: {}", e))
            })?;

        // style: [1, 256]
        let style_shape = vec![1, 256];
        let style_data: Box<[f32]> = style.to_vec().into_boxed_slice();
        let style_value = Value::from_array((style_shape.as_slice(), style_data))
            .map_err(|e| crate::Error::Model(format!("Failed to create style tensor: {}", e)))?;

        // speed: [1]
        let speed_shape = vec![1];
        let speed_data: Box<[f32]> = vec![speed].into_boxed_slice();
        let speed_value = Value::from_array((speed_shape.as_slice(), speed_data))
            .map_err(|e| crate::Error::Model(format!("Failed to create speed tensor: {}", e)))?;

        // Run inference
        let outputs = session
            .run(ort::inputs![
                "input_ids" => input_ids_value,
                "style" => style_value,
                "speed" => speed_value,
            ])
            .map_err(|e| crate::Error::Model(format!("ONNX inference failed: {}", e)))?;

        // Extract audio output
        // Output shape should be [1, num_samples]
        let audio_tensor = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| crate::Error::Model(format!("Failed to extract audio tensor: {}", e)))?;

        // Extract the slice from the tensor tuple (shape, data)
        let audio_data = audio_tensor.1;

        debug!("Generated {} audio samples", audio_data.len());
        Ok(audio_data.to_vec())
    }
}

impl Default for TtsKokoro {
    fn default() -> Self {
        Self::new()
    }
}

impl TtsRuntime for TtsKokoro {
    fn load(&mut self, config: TtsRuntimeConfig) -> Result<()> {
        info!("Loading Kokoro TTS model: {}", config.model_path);

        // Resolve model directory
        let model_dir = self.resolve_model_dir(&config.model_path)?;

        if !model_dir.exists() {
            return Err(crate::Error::Model(format!(
                "Model directory not found: {:?}\nDownload with: onevox models download {}",
                model_dir, config.model_path
            )));
        }

        // Check for required files - try multiple possible locations
        let possible_model_paths = vec![
            model_dir.join("model.onnx"),
            model_dir.join("onnx").join("model.onnx"),
            model_dir.join("onnx").join("model_q8f16.onnx"),
            model_dir.join("onnx").join("model_fp16.onnx"),
            model_dir.join("onnx").join("model_q4f16.onnx"),
        ];

        let mut model_path = None;
        for path in &possible_model_paths {
            if path.exists() {
                model_path = Some(path.clone());
                break;
            }
        }

        // If still not found, try to find any .onnx file
        if model_path.is_none()
            && let Ok(entries) = std::fs::read_dir(model_dir.join("onnx"))
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("onnx") {
                    info!("Found ONNX model file: {:?}", path);
                    model_path = Some(path);
                    break;
                }
            }
        }

        let model_path = model_path.ok_or_else(|| {
            crate::Error::Model(format!(
                "Model file not found in: {:?}\nRun 'onevox models download {}' to download",
                model_dir, config.model_path
            ))
        })?;

        // Load vocabulary
        let vocab = self.load_vocab(&model_dir)?;

        // Load ONNX model
        info!("Loading ONNX model from: {:?}", model_path);
        let model_bytes = std::fs::read(&model_path)
            .map_err(|e| crate::Error::Model(format!("Failed to read model file: {}", e)))?;

        info!("Model file size: {} MB", model_bytes.len() / (1024 * 1024));

        // Configure ONNX Runtime session with GPU support
        info!(
            "Configuring ONNX Runtime session (use_gpu: {})",
            config.use_gpu
        );

        let mut session_builder = Session::builder()
            .map_err(|e| crate::Error::Model(format!("Failed to create session builder: {}", e)))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| crate::Error::Model(format!("Failed to set optimization level: {}", e)))?
            .with_intra_threads(4)
            .map_err(|e| crate::Error::Model(format!("Failed to set thread count: {}", e)))?;

        // Explicitly enable CoreML execution provider on macOS when GPU is enabled
        if config.use_gpu && cfg!(target_os = "macos") {
            info!("🎮 Enabling CoreML execution provider for GPU acceleration");
            session_builder = session_builder
                .with_execution_providers([ep::CoreML::default().build()])
                .map_err(|e| crate::Error::Model(format!("Failed to enable CoreML EP: {}", e)))?;
        }

        let session = session_builder
            .commit_from_memory(&model_bytes)
            .map_err(|e| crate::Error::Model(format!("Failed to load ONNX model: {}", e)))?;

        info!("✅ ONNX session created with CoreML EP");

        // Load available voices
        info!("Loading voice style vectors...");
        let voice_list = Self::initialize_voice_list();
        let mut voices = HashMap::new();

        for voice_info in voice_list {
            match self.load_voice(&model_dir, &voice_info.id, voice_info.clone()) {
                Ok(voice_data) => {
                    voices.insert(voice_info.id.clone(), voice_data);
                }
                Err(e) => {
                    warn!("Failed to load voice {}: {}", voice_info.id, e);
                }
            }
        }

        if voices.is_empty() {
            return Err(crate::Error::Model(
                "No voice files found. Model download may be incomplete.".to_string(),
            ));
        }

        info!("✅ Kokoro TTS model loaded successfully");
        info!("   Model directory: {:?}", model_dir);
        info!("   Vocabulary size: {}", vocab.len());
        info!("   Available voices: {}", voices.len());
        info!("   Default voice: {}", config.voice_id);

        self.session = Some(session);
        self.vocab = Some(vocab);
        self.voices = voices;
        self.config = Some(config);
        self.model_dir = Some(model_dir);

        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.session.is_some() && self.vocab.is_some() && !self.voices.is_empty()
    }

    fn synthesize(&mut self, text: &str) -> Result<TtsSynthesis> {
        if !self.is_loaded() {
            return Err(crate::Error::Model("Model not loaded".to_string()));
        }

        let start = std::time::Instant::now();

        // Get config and extract values to avoid borrow issues
        let (voice_id, speech_rate, volume) = {
            let config = self
                .config
                .as_ref()
                .ok_or_else(|| crate::Error::Model("Config not set".to_string()))?;
            (config.voice_id.clone(), config.speech_rate, config.volume)
        };

        // Normalize text
        let normalized = self.normalize_text(text);
        if normalized.is_empty() {
            return Ok(TtsSynthesis::new(vec![], 24000));
        }

        debug!("Synthesizing: '{}'", normalized);

        // Phonemize
        let phonemes = self.phonemize(&normalized)?;

        // Tokenize
        let tokens = self.tokenize(&phonemes)?;

        // Get voice style vector
        let voice_data = self
            .voices
            .get(&voice_id)
            .ok_or_else(|| crate::Error::Model(format!("Voice not found: {}", voice_id)))?;

        let style = voice_data.get_style_for_length(tokens.len())?;

        // Run inference
        let mut audio_samples = self.run_inference(&tokens, &style, speech_rate)?;

        // Apply volume adjustment
        if (volume - 1.0).abs() > 0.01 {
            for sample in &mut audio_samples {
                *sample *= volume;
            }
        }

        let synthesis_time_ms = start.elapsed().as_millis() as u64;

        // Create synthesis result
        let synthesis =
            TtsSynthesis::new(audio_samples, 24000).with_synthesis_time(synthesis_time_ms);

        info!(
            "✅ Synthesized {:.2}s of audio in {}ms (RTF: {:.3})",
            synthesis.duration_secs(),
            synthesis_time_ms,
            synthesis.rtf
        );

        Ok(synthesis)
    }

    fn list_voices(&self) -> Vec<VoiceInfo> {
        self.voices.values().map(|v| v.info.clone()).collect()
    }

    fn set_voice(&mut self, voice_id: &str) -> Result<()> {
        // Check if voice exists
        if !self.voices.contains_key(voice_id) {
            return Err(crate::Error::Model(format!(
                "Voice '{}' not found. Available: {:?}",
                voice_id,
                self.voices.keys().collect::<Vec<_>>()
            )));
        }

        // Update config
        if let Some(config) = &mut self.config {
            info!(
                "Switching voice from '{}' to '{}'",
                config.voice_id, voice_id
            );
            config.voice_id = voice_id.to_string();
        }

        Ok(())
    }

    fn unload(&mut self) {
        info!("Unloading Kokoro TTS model");
        self.session = None;
        self.vocab = None;
        self.voices.clear();
        self.config = None;
        self.model_dir = None;
    }

    fn name(&self) -> &str {
        "tts-kokoro"
    }

    fn info(&self) -> TtsInfo {
        TtsInfo {
            name: self.name().to_string(),
            loaded: self.is_loaded(),
            backend: "kokoro-onnx".to_string(),
            available_voices: self.voices.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_backend() {
        let backend = TtsKokoro::new();
        assert!(!backend.is_loaded());
        assert_eq!(backend.name(), "tts-kokoro");
    }

    #[test]
    fn test_not_loaded_initially() {
        let backend = TtsKokoro::new();
        assert!(!backend.is_loaded());
    }

    #[test]
    fn test_normalize_text() {
        let backend = TtsKokoro::new();

        assert_eq!(
            backend.normalize_text("  Hello,  World!  "),
            "hello, world!"
        );

        assert_eq!(backend.normalize_text("Test@123#456"), "test 123 456");

        assert_eq!(
            backend.normalize_text("It's a nice day."),
            "it's a nice day."
        );
    }

    #[test]
    fn test_voice_list() {
        let voices = TtsKokoro::initialize_voice_list();
        assert_eq!(voices.len(), 11);

        // Check default voice exists
        assert!(voices.iter().any(|v| v.id == "af"));

        // Check American voices
        assert!(voices.iter().any(|v| v.id == "am_adam"));

        // Check British voices
        assert!(voices.iter().any(|v| v.id == "bf_emma"));
    }

    #[test]
    fn test_voice_style_data() {
        let vectors = vec![0.0f32; 512 * 256]; // Mock style data
        let info = VoiceInfo::new("test".to_string(), "Test".to_string(), "en-US".to_string());

        let voice_data = VoiceStyleData { vectors, info };

        // Test getting style for different lengths
        let style = voice_data.get_style_for_length(10).unwrap();
        assert_eq!(style.len(), 256);

        let style = voice_data.get_style_for_length(511).unwrap();
        assert_eq!(style.len(), 256);

        // Test clamping at 511
        let style = voice_data.get_style_for_length(600).unwrap();
        assert_eq!(style.len(), 256);
    }

    #[test]
    fn test_empty_text_synthesis() {
        let backend = TtsKokoro::new();
        let normalized = backend.normalize_text("   ");
        assert!(normalized.is_empty());
    }
}
