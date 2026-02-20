//! Whisper ONNX Runtime Backend
//!
//! Full ONNX-based Whisper implementation for cross-platform compatibility.
//!
//! This implementation includes:
//! - Mel spectrogram computation
//! - ONNX encoder inference
//! - ONNX decoder inference with autoregressive generation
//! - Basic token decoding

#[cfg(feature = "onnx")]
use super::downloader::ModelDownloader;
#[cfg(feature = "onnx")]
use super::runtime::{ModelConfig, ModelInfo, ModelRuntime, Transcription};
#[cfg(feature = "onnx")]
use super::tokenizer::SimpleTokenizer;
#[cfg(feature = "onnx")]
use anyhow::Result;
#[cfg(feature = "onnx")]
use ndarray::{Array1, Array2, Array3, Axis};
#[cfg(feature = "onnx")]
use ort::session::{Session, builder::GraphOptimizationLevel};
#[cfg(feature = "onnx")]
use ort::value::Tensor;
#[cfg(feature = "onnx")]
use parking_lot::Mutex;
#[cfg(feature = "onnx")]
use std::path::Path;
#[cfg(feature = "onnx")]
use tracing::{debug, info, warn};

#[cfg(feature = "onnx")]
/// Whisper ONNX model
pub struct WhisperOnnx {
    encoder: Option<Mutex<Session>>,
    decoder: Option<Mutex<Session>>,
    config: Option<ModelConfig>,
    model_id: String,
    mel_filters: Option<Array2<f32>>,
    tokenizer: SimpleTokenizer,
}

#[cfg(feature = "onnx")]
/// Whisper model constants
mod constants {
    pub const SAMPLE_RATE: usize = 16000;
    pub const N_FFT: usize = 400;
    pub const HOP_LENGTH: usize = 160;
    pub const N_MELS: usize = 80;
    pub const CHUNK_LENGTH: usize = 30; // seconds
    pub const N_SAMPLES: usize = CHUNK_LENGTH * SAMPLE_RATE; // 480,000 samples
    pub const N_FRAMES: usize = N_SAMPLES / HOP_LENGTH; // 3,000 frames

    // Whisper special tokens (verified from tokenizer.json)
    pub const EOT_TOKEN: i64 = 50256; // <|endoftext|>
    pub const SOT_TOKEN: i64 = 50257; // <|startoftranscript|>
    pub const ENGLISH_TOKEN: i64 = 50258; // <|en|>
    pub const TRANSLATE_TOKEN: i64 = 50357; // <|translate|>
    pub const TRANSCRIBE_TOKEN: i64 = 50358; // <|transcribe|>
    pub const NO_TIMESTAMPS_TOKEN: i64 = 50362; // <|notimestamps|>
    pub const TIMESTAMP_BEGIN: i64 = 50363; // First timestamp token

    // Generation parameters
    pub const MAX_LENGTH: usize = 448; // Maximum tokens to generate
    pub const NO_SPEECH_THRESHOLD: f32 = 0.6;
}

#[cfg(feature = "onnx")]
impl WhisperOnnx {
    /// Create a new Whisper ONNX model
    pub fn new() -> Result<Self> {
        // Auto-detect ONNX Runtime library if not set
        Self::ensure_onnx_runtime_available()?;

        Ok(Self {
            encoder: None,
            decoder: None,
            config: None,
            model_id: String::new(),
            mel_filters: None,
            tokenizer: SimpleTokenizer::new(),
        })
    }

    /// Ensure ONNX Runtime library is available
    fn ensure_onnx_runtime_available() -> Result<()> {
        // Check if ORT_DYLIB_PATH is already set
        if std::env::var("ORT_DYLIB_PATH").is_ok() {
            return Ok(());
        }

        // Try to find ONNX Runtime in common locations
        let possible_paths = vec![
            "/opt/homebrew/lib/libonnxruntime.dylib",
            "/opt/homebrew/Cellar/onnxruntime/1.24.2/lib/libonnxruntime.dylib",
            "/usr/local/lib/libonnxruntime.dylib",
            "/usr/lib/libonnxruntime.dylib",
        ];

        for path in possible_paths {
            if Path::new(path).exists() {
                info!("Auto-detected ONNX Runtime at: {}", path);
                // SAFETY: We're setting an environment variable that the ort crate needs.
                // This is safe because we're doing it before any ONNX Runtime operations.
                unsafe {
                    std::env::set_var("ORT_DYLIB_PATH", path);
                }
                return Ok(());
            }
        }

        // If not found, provide helpful error message
        warn!("ONNX Runtime library not found in standard locations");
        anyhow::bail!(
            "ONNX Runtime library not found!\n\
            Please install it with: brew install onnxruntime\n\
            Or set ORT_DYLIB_PATH environment variable to the library path"
        );
    }

    /// Load model from downloaded files
    fn load_model(&mut self, model_id: &str) -> Result<()> {
        info!("Loading Whisper ONNX model: {}", model_id);

        // Get model directory
        let downloader = ModelDownloader::new()?;
        let model_dir = downloader.model_dir(model_id);

        if !model_dir.exists() {
            anyhow::bail!(
                "Model not found: {}. Download it with: onevox models download {}",
                model_id,
                model_id
            );
        }

        // Load encoder
        let encoder_path = model_dir.join("onnx/encoder_model.onnx");
        if !encoder_path.exists() {
            anyhow::bail!("Encoder model not found at: {:?}", encoder_path);
        }

        info!("Loading encoder: {:?}", encoder_path);
        let encoder = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .commit_from_file(&encoder_path)?;

        // Load decoder
        let decoder_path = model_dir.join("onnx/decoder_model_merged.onnx");
        if !decoder_path.exists() {
            anyhow::bail!("Decoder model not found at: {:?}", decoder_path);
        }

        info!("Loading decoder: {:?}", decoder_path);
        let decoder = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .commit_from_file(&decoder_path)?;

        // Initialize mel filters
        self.mel_filters = Some(Self::create_mel_filters());

        self.encoder = Some(Mutex::new(encoder));
        self.decoder = Some(Mutex::new(decoder));
        self.model_id = model_id.to_string();

        info!("✅ Whisper ONNX model loaded successfully");

        Ok(())
    }

    /// Create mel filterbank
    fn create_mel_filters() -> Array2<f32> {
        use constants::*;

        // Create mel filterbank matrix (simplified version)
        let mut filters = Array2::<f32>::zeros((N_MELS, N_FFT / 2 + 1));

        // Simplified mel scale mapping
        let mel_min = Self::hz_to_mel(0.0);
        let mel_max = Self::hz_to_mel(SAMPLE_RATE as f32 / 2.0);

        for i in 0..N_MELS {
            let mel_center = mel_min + (mel_max - mel_min) * (i as f32 / N_MELS as f32);
            let hz_center = Self::mel_to_hz(mel_center);
            let bin_center = (hz_center * N_FFT as f32 / SAMPLE_RATE as f32) as usize;

            // Triangular filter
            let bandwidth = N_FFT / 2 / N_MELS;
            for j in bin_center.saturating_sub(bandwidth)
                ..std::cmp::min(bin_center + bandwidth, N_FFT / 2 + 1)
            {
                let distance =
                    ((j as i32 - bin_center as i32).abs() as f32 / bandwidth as f32).min(1.0);
                filters[[i, j]] = 1.0 - distance;
            }

            // Normalize
            let sum: f32 = filters.row(i).sum();
            if sum > 0.0 {
                for j in 0..filters.ncols() {
                    filters[[i, j]] /= sum;
                }
            }
        }

        filters
    }

    /// Convert Hz to mel scale
    fn hz_to_mel(hz: f32) -> f32 {
        2595.0 * (1.0 + hz / 700.0).log10()
    }

    /// Convert mel to Hz scale
    fn mel_to_hz(mel: f32) -> f32 {
        700.0 * (10f32.powf(mel / 2595.0) - 1.0)
    }

    /// Compute mel spectrogram from audio samples
    fn compute_mel_spectrogram(&self, samples: &[f32]) -> Result<Array3<f32>> {
        use constants::*;
        use rustfft::{FftPlanner, num_complex::Complex};

        let filters = self
            .mel_filters
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mel filters not initialized"))?;

        // Pad or truncate to exactly N_SAMPLES
        let mut padded = vec![0.0f32; N_SAMPLES];
        let copy_len = samples.len().min(N_SAMPLES);
        padded[..copy_len].copy_from_slice(&samples[..copy_len]);

        // Initialize FFT planner
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(N_FFT);

        // Compute STFT
        let mut spectrogram = Array2::<f32>::zeros((N_MELS, N_FRAMES));

        for frame_idx in 0..N_FRAMES {
            let start = frame_idx * HOP_LENGTH;
            let end = (start + N_FFT).min(N_SAMPLES);

            if start >= N_SAMPLES {
                break;
            }

            // Extract frame with Hann window
            let mut frame = vec![Complex::new(0.0f32, 0.0f32); N_FFT];
            for i in 0..(end - start) {
                let window =
                    0.5 * (1.0 - ((2.0 * std::f32::consts::PI * i as f32) / N_FFT as f32).cos());
                frame[i] = Complex::new(padded[start + i] * window, 0.0);
            }

            // Compute FFT
            fft.process(&mut frame);

            // Compute magnitudes (only need first half + DC for real signal)
            let mut magnitudes = vec![0.0f32; N_FFT / 2 + 1];
            for (k, mag) in magnitudes.iter_mut().enumerate() {
                *mag = frame[k].norm();
            }

            // Apply mel filterbank
            let magnitudes_array = Array1::from(magnitudes);
            for mel_idx in 0..N_MELS {
                let filter = filters.row(mel_idx);
                let mel_value: f32 = magnitudes_array
                    .iter()
                    .zip(filter.iter())
                    .map(|(m, f)| m * f)
                    .sum();
                spectrogram[[mel_idx, frame_idx]] = mel_value;
            }
        }

        // Convert to log scale (with small epsilon to avoid log(0))
        let log_spec = spectrogram.mapv(|x| (x.max(1e-10)).ln());

        // Normalize to match Whisper's expected input range
        let max_val = log_spec.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let normalized = log_spec.mapv(|x| ((x - max_val) / 4.0).max(-8.0));

        // Reshape to [1, N_MELS, N_FRAMES] for ONNX input
        let shaped = normalized
            .insert_axis(Axis(0))
            .into_shape_with_order((1, N_MELS, N_FRAMES))?;

        Ok(shaped)
    }

    /// Run encoder inference
    fn encode(&self, mel_spectrogram: Array3<f32>) -> Result<Array3<f32>> {
        let encoder = self
            .encoder
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Encoder not loaded"))?;

        // Create tensor directly from Array3 (ort v2.0 supports owned arrays)
        let input_tensor = Tensor::from_array(mel_spectrogram)?;

        // Run encoder - lock() because ort v2.0 requires &mut self for run()
        let mut encoder_locked = encoder.lock();
        let outputs = encoder_locked.run(ort::inputs!["input_features" => input_tensor])?;

        // Extract audio features using try_extract_tensor (v2.0 API) and copy data
        let audio_features = outputs["last_hidden_state"].try_extract_tensor::<f32>()?;

        // Get shape and data slice - shape is a tuple of (&Shape, &[T])
        let (shape, data) = audio_features;

        // Shape can be indexed directly
        if shape.len() != 3 {
            anyhow::bail!("Expected 3D tensor, got shape: {:?}", shape);
        }

        // Copy the data to owned array before borrow ends
        let audio_features_owned = Array3::from_shape_vec(
            (shape[0] as usize, shape[1] as usize, shape[2] as usize),
            data.to_vec(),
        )?;

        Ok(audio_features_owned)
    }

    /// Run decoder inference (autoregressive)
    fn decode(&self, audio_features: &Array3<f32>, language: &str) -> Result<Vec<i64>> {
        use constants::*;

        let decoder = self
            .decoder
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Decoder not loaded"))?;

        // Initialize with start tokens
        let mut tokens = vec![SOT_TOKEN];

        // Add language token (English for whisper-tiny.en)
        let lang_token = if language == "en" {
            ENGLISH_TOKEN
        } else {
            ENGLISH_TOKEN // whisper-tiny.en only supports English
        };
        tokens.push(lang_token);

        // Add task token (transcribe)
        tokens.push(TRANSCRIBE_TOKEN);

        // Add no-timestamps token
        tokens.push(NO_TIMESTAMPS_TOKEN);

        debug!("Initial tokens: {:?}", tokens);

        // Track token history for repetition detection
        let mut token_counts: std::collections::HashMap<i64, usize> =
            std::collections::HashMap::new();
        let mut consecutive_repeats = 0;
        let mut last_token: Option<i64> = None;
        let mut recent_tokens: Vec<i64> = Vec::new(); // Track last N tokens for pattern detection

        // Autoregressive generation
        for step in 0..MAX_LENGTH {
            // Prepare decoder inputs
            let tokens_array = Array2::from_shape_vec((1, tokens.len()), tokens.clone())?;

            // Create tensors directly from owned arrays (ort v2.0 API)
            let audio_features_tensor = Tensor::from_array(audio_features.clone())?;
            let tokens_tensor = Tensor::from_array(tokens_array)?;

            // Create use_cache_branch input (false = don't use KV cache for simplicity)
            let use_cache = Array1::from_vec(vec![false]);
            let use_cache_tensor = Tensor::from_array(use_cache)?;

            // Run decoder and extract next token in a single scope to manage Mutex lock
            let next_token = {
                let mut decoder_locked = decoder.lock();
                let outputs = decoder_locked.run(ort::inputs![
                    "encoder_hidden_states" => audio_features_tensor,
                    "input_ids" => tokens_tensor,
                    "use_cache_branch" => use_cache_tensor,
                ])?;

                // Extract logits using v2.0 API and process immediately
                let logits_tensor = outputs["logits"].try_extract_tensor::<f32>()?;
                let (shape, logits_data) = logits_tensor;

                // Logits shape is [batch_size, sequence_length, vocab_size]
                // Get the actual dimensions
                if step < 3 {
                    debug!("Logits shape: {:?}", shape);
                    debug!("Logits data length: {}", logits_data.len());
                    debug!("Current token count: {}", tokens.len());
                }

                // Calculate vocab size from the shape
                let vocab_size = shape[shape.len() - 1] as usize;

                // The logits are for ALL input tokens, we want the LAST one
                // Position = (sequence_length - 1) * vocab_size
                let last_token_idx = (shape[1] as usize - 1) * vocab_size;
                let last_logits_start = last_token_idx;
                let last_logits_end = last_logits_start + vocab_size;

                // Get the flat slice
                if last_logits_end > logits_data.len() {
                    anyhow::bail!(
                        "Logits tensor smaller than expected: {} vs {} (shape={:?}, vocab_size={}, last_token_idx={})",
                        logits_data.len(),
                        last_logits_end,
                        shape,
                        vocab_size,
                        last_token_idx
                    );
                }

                let last_logits = &logits_data[last_logits_start..last_logits_end];

                // Apply repetition penalty
                let mut adjusted_logits: Vec<f32> = last_logits.to_vec();
                let repetition_penalty: f32 = 2.0; // Strong penalty for repeated tokens

                for (token_id, count) in &token_counts {
                    if *count > 0 && (*token_id as usize) < adjusted_logits.len() {
                        // Reduce logit score for repeated tokens exponentially
                        let penalty = repetition_penalty.powi(*count as i32);
                        adjusted_logits[*token_id as usize] /= penalty;
                    }
                }

                // Boost EOT token to encourage ending
                if step > 20 {
                    // After 20 tokens, start boosting EOT
                    let eot_boost = 1.0 + (step as f32 / 100.0); // Gradually increase
                    if (EOT_TOKEN as usize) < adjusted_logits.len() {
                        adjusted_logits[EOT_TOKEN as usize] *= eot_boost;
                    }
                }

                // Greedy decoding with penalties applied
                adjusted_logits
                    .iter()
                    .enumerate()
                    .max_by(|(_, a): &(usize, &f32), (_, b): &(usize, &f32)| {
                        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(idx, _)| idx as i64)
                    .unwrap_or(EOT_TOKEN)
            }; // decoder_locked is dropped here

            if step < 10 || step % 20 == 0 {
                debug!("Step {}: Generated token {}", step, next_token);
            }

            // Check for end of transcript
            if next_token == EOT_TOKEN {
                debug!("End of transcript at step {}", step);
                break;
            }

            // Detect repetition loops (both consecutive and alternating patterns)
            if Some(next_token) == last_token {
                consecutive_repeats += 1;
                if consecutive_repeats > 5 {
                    warn!(
                        "Detected consecutive repetition (token {} repeated {} times), stopping",
                        next_token, consecutive_repeats
                    );
                    break;
                }
            } else {
                consecutive_repeats = 0;
            }

            // Detect alternating patterns (A-B-A-B or A-B-C-A-B-C)
            recent_tokens.push(next_token);
            if recent_tokens.len() > 10 {
                recent_tokens.remove(0);
            }

            // Check for A-B-A-B pattern (at least 8 repetitions to be sure)
            if recent_tokens.len() >= 8 {
                let len = recent_tokens.len();
                // Check if last 8 tokens form A-B-A-B-A-B-A-B pattern
                if recent_tokens[len - 1] == recent_tokens[len - 3]
                    && recent_tokens[len - 1] == recent_tokens[len - 5]
                    && recent_tokens[len - 1] == recent_tokens[len - 7]
                    && recent_tokens[len - 2] == recent_tokens[len - 4]
                    && recent_tokens[len - 2] == recent_tokens[len - 6]
                    && recent_tokens[len - 2] == recent_tokens[len - 8]
                    && recent_tokens[len - 1] != recent_tokens[len - 2]
                {
                    warn!(
                        "Detected alternating repetition pattern ({} <-> {}), stopping",
                        recent_tokens[len - 1],
                        recent_tokens[len - 2]
                    );
                    break;
                }
            }

            // Check for A-B-A-B pattern (at least 4 tokens)
            if recent_tokens.len() >= 6 {
                let len = recent_tokens.len();
                // Check if last 6 tokens form A-B-A-B-A-B pattern
                if recent_tokens[len - 1] == recent_tokens[len - 3]
                    && recent_tokens[len - 1] == recent_tokens[len - 5]
                    && recent_tokens[len - 2] == recent_tokens[len - 4]
                    && recent_tokens[len - 2] == recent_tokens[len - 6]
                    && recent_tokens[len - 1] != recent_tokens[len - 2]
                {
                    warn!(
                        "Detected alternating repetition pattern ({} <-> {}), stopping",
                        recent_tokens[len - 1],
                        recent_tokens[len - 2]
                    );
                    break;
                }
            }

            last_token = Some(next_token);

            // Track token occurrences
            *token_counts.entry(next_token).or_insert(0) += 1;

            // Check for end of transcript
            if next_token == EOT_TOKEN {
                debug!("End of transcript at step {}", step);
                break;
            }

            // Skip timestamp tokens
            if next_token >= TIMESTAMP_BEGIN {
                continue;
            }

            tokens.push(next_token);

            // Safety check
            if tokens.len() > MAX_LENGTH {
                warn!("Reached maximum length");
                break;
            }
        }

        // Remove special tokens from the beginning
        let text_tokens = tokens
            .into_iter()
            .skip(4) // Skip SOT, language, task, no-timestamps
            .filter(|&t| t < TIMESTAMP_BEGIN && t != EOT_TOKEN)
            .collect();

        debug!("Generated tokens: {:?}", text_tokens);

        Ok(text_tokens)
    }

    /// Decode tokens to text
    fn tokens_to_text(&self, tokens: &[i64]) -> Result<String> {
        if tokens.is_empty() {
            return Ok(String::new());
        }

        debug!("Decoding {} tokens: {:?}", tokens.len(), tokens);

        // Use the tokenizer to decode
        let text = self.tokenizer.decode(tokens)?;

        debug!("Decoded text: '{}'", text);

        // Return the decoded text (even if empty)
        // The tokenizer will skip special tokens and handle unknowns
        Ok(text)
    }
}

#[cfg(feature = "onnx")]
impl Default for WhisperOnnx {
    fn default() -> Self {
        Self::new().expect("Failed to create WhisperOnnx")
    }
}

#[cfg(feature = "onnx")]
impl ModelRuntime for WhisperOnnx {
    fn load(&mut self, config: ModelConfig) -> crate::Result<()> {
        info!("Loading Whisper ONNX runtime with config: {:?}", config);

        // Extract model ID from path (e.g., "whisper-tiny.en")
        let model_id = config
            .model_path
            .trim_end_matches(".onnx")
            .split('/')
            .last()
            .unwrap_or("whisper-tiny.en")
            .to_string();

        self.load_model(&model_id)
            .map_err(|e| crate::Error::Model(e.to_string()))?;

        self.config = Some(config);

        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.encoder.is_some() && self.decoder.is_some()
    }

    fn transcribe(&self, samples: &[f32], _sample_rate: u32) -> crate::Result<Transcription> {
        if !self.is_loaded() {
            return Err(crate::Error::Model("Model not loaded".to_string()));
        }

        let start = std::time::Instant::now();

        info!("Computing mel spectrogram for {} samples", samples.len());

        // 1. Compute mel spectrogram
        let mel_spec = self
            .compute_mel_spectrogram(samples)
            .map_err(|e| crate::Error::Model(format!("Mel spectrogram failed: {}", e)))?;

        debug!("Mel spectrogram shape: {:?}", mel_spec.shape());

        // 2. Encode audio
        info!("Running encoder");
        let audio_features = self
            .encode(mel_spec)
            .map_err(|e| crate::Error::Model(format!("Encoder failed: {}", e)))?;

        debug!("Audio features shape: {:?}", audio_features.shape());

        // 3. Decode to tokens
        info!("Running decoder");
        let language = self
            .config
            .as_ref()
            .and_then(|c| Some(c.language.as_str()))
            .unwrap_or("en");

        let tokens = self
            .decode(&audio_features, language)
            .map_err(|e| crate::Error::Model(format!("Decoder failed: {}", e)))?;

        debug!("Generated {} tokens", tokens.len());

        // 4. Convert tokens to text
        info!("Decoding tokens to text");
        let text = self
            .tokens_to_text(&tokens)
            .map_err(|e| crate::Error::Model(format!("Token decoding failed: {}", e)))?;

        let processing_time = start.elapsed();

        info!(
            "Transcription complete: \"{}\" ({} ms)",
            text,
            processing_time.as_millis()
        );

        Ok(Transcription {
            text,
            language: Some(language.to_string()),
            confidence: Some(0.95), // TODO: Compute actual confidence from logits
            processing_time_ms: processing_time.as_millis() as u64,
            tokens: Some(tokens.len()),
        })
    }

    fn unload(&mut self) {
        info!("Unloading Whisper ONNX model");
        self.encoder = None;
        self.decoder = None;
        self.config = None;
        self.mel_filters = None;
        // tokenizer stays - it's lightweight
    }

    fn name(&self) -> &str {
        "Whisper ONNX"
    }

    fn info(&self) -> ModelInfo {
        ModelInfo {
            name: format!("whisper-onnx-{}", self.model_id),
            size_bytes: 0,
            model_type: "whisper".to_string(),
            backend: "onnx".to_string(),
            gpu_enabled: false,
        }
    }
}

#[cfg(not(feature = "onnx"))]
pub struct WhisperOnnx;

#[cfg(not(feature = "onnx"))]
impl WhisperOnnx {
    pub fn new() -> anyhow::Result<Self> {
        anyhow::bail!("ONNX feature not enabled. Rebuild with --features onnx")
    }
}
