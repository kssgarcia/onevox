//! ONNX Runtime Backend
//!
//! High-performance ASR using ONNX Runtime with support for CTC-based models like NVIDIA Parakeet.
//! Designed for production use with cross-platform support and robust error handling.

#[cfg(feature = "onnx")]
use super::runtime::{ModelConfig, ModelInfo, ModelRuntime, Transcription};
#[cfg(feature = "onnx")]
use std::path::{Path, PathBuf};
#[cfg(feature = "onnx")]
use tracing::{debug, info, warn};

#[cfg(feature = "onnx")]
use ort::{session::Session, session::builder::GraphOptimizationLevel, value::Value};

// Initialize ONNX Runtime environment once at module load
#[cfg(feature = "onnx")]
use std::sync::Once;

#[cfg(feature = "onnx")]
static INIT_ORT: Once = Once::new();

#[cfg(feature = "onnx")]
fn init_ort_environment() {
    INIT_ORT.call_once(|| {
        // Initialize ONNX Runtime environment
        // This triggers the dylib loading with the statically-linked library
        let _ = ort::init().with_name("onevox").commit();
        info!("ONNX Runtime environment initialized");
    });
}

/// ONNX Runtime model backend
#[cfg(feature = "onnx")]
pub struct OnnxRuntime {
    encoder_session: Option<Session>,
    vocab: Option<Vec<String>>,
    config: Option<ModelConfig>,
    model_dir: Option<PathBuf>,
    n_mel_bins: usize, // Number of mel bins (80 for Parakeet CTC, 128 for TDT)
}

#[cfg(feature = "onnx")]
impl OnnxRuntime {
    /// Create a new ONNX Runtime backend
    pub fn new() -> crate::Result<Self> {
        info!("Initializing ONNX Runtime backend");

        // Initialize the ONNX Runtime environment
        init_ort_environment();

        Ok(Self {
            encoder_session: None,
            vocab: None,
            config: None,
            model_dir: None,
            n_mel_bins: 80, // Default to 80 for Parakeet CTC
        })
    }

    /// Resolve model directory from cache
    fn resolve_model_dir(&self, model_id: &str) -> crate::Result<PathBuf> {
        // Check if it's an absolute path
        let direct_path = PathBuf::from(model_id);
        if direct_path.is_absolute() {
            if direct_path.is_dir() {
                info!("Using absolute model directory: {:?}", direct_path);
                return Ok(direct_path);
            } else if direct_path.exists() && direct_path.is_file() {
                // If it's a file, return its parent directory
                if let Some(parent) = direct_path.parent() {
                    info!("Using parent directory of absolute path: {:?}", parent);
                    return Ok(parent.to_path_buf());
                }
            }
        }

        // Use platform-appropriate models directory
        let models_dir =
            crate::platform::paths::models_dir().unwrap_or_else(|_| PathBuf::from("./models"));

        // Model directory structure: models/<model_id>/
        let model_dir = models_dir.join(model_id);

        if !model_dir.exists() {
            warn!("Model directory not found at: {:?}", model_dir);
            debug!(
                "Expected structure: {}/{{encoder-model.onnx, vocab.txt}}",
                model_dir.display()
            );
        }

        Ok(model_dir)
    }

    /// Find the encoder model file (prefer INT8 quantized)
    fn find_encoder_path(&self, model_dir: &PathBuf) -> crate::Result<PathBuf> {
        // Priority order: INT8 quantized > full precision
        let candidates = vec![
            model_dir.join("encoder-model.int8.onnx"),
            model_dir.join("encoder-model.onnx"),
            model_dir.join("model.int8.onnx"),
            model_dir.join("model.onnx"),
        ];

        for path in &candidates {
            if path.exists() && path.is_file() {
                info!("Found encoder model at: {:?}", path);
                return Ok(path.clone());
            }
        }

        // Return expected path for better error message
        Err(crate::Error::Model(format!(
            "Encoder model not found in {:?}\nExpected one of: encoder-model.int8.onnx, encoder-model.onnx, model.onnx",
            model_dir
        )))
    }

    /// Load vocabulary file
    fn load_vocab(&self, model_dir: &Path) -> crate::Result<Vec<String>> {
        let vocab_path = model_dir.join("vocab.txt");

        if !vocab_path.exists() {
            return Err(crate::Error::Model(format!(
                "Vocabulary file not found: {:?}\n\
                 This usually means the model download is incomplete.\n\
                 Fix: Run 'onevox models download parakeet-ctc-0.6b' again to complete the download.\n\
                 Or manually download from: https://huggingface.co/istupakov/parakeet-ctc-0.6b-onnx/resolve/main/vocab.txt",
                vocab_path
            )));
        }

        info!("Loading vocabulary from: {:?}", vocab_path);

        let content = std::fs::read_to_string(&vocab_path)
            .map_err(|e| crate::Error::Model(format!("Failed to read vocabulary file: {}", e)))?;

        // Parse vocabulary - format is typically "token index" or just "token"
        let vocab: Vec<String> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                // Handle format: "token index" - extract just the token
                line.split_whitespace().next().unwrap_or("").to_string()
            })
            .collect();

        if vocab.is_empty() {
            return Err(crate::Error::Model("Vocabulary is empty".to_string()));
        }

        info!("Loaded vocabulary with {} tokens", vocab.len());
        Ok(vocab)
    }

    /// Load model configuration file
    fn load_model_config(&self, model_dir: &Path) -> crate::Result<usize> {
        let config_path = model_dir.join("config.json");

        // If config.json doesn't exist, default to 80 mel bins (Parakeet CTC)
        if !config_path.exists() {
            info!("config.json not found, using default 80 mel bins");
            return Ok(80);
        }

        info!("Loading model config from: {:?}", config_path);

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| crate::Error::Model(format!("Failed to read config file: {}", e)))?;

        // Parse JSON to extract features_size
        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| crate::Error::Model(format!("Failed to parse config JSON: {}", e)))?;

        let n_mel_bins = json
            .get("features_size")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(80); // Default to 80 if not found

        info!("Model requires {} mel bins", n_mel_bins);
        Ok(n_mel_bins)
    }

    /// Decode CTC token IDs to text using greedy decoding
    fn decode_ctc_tokens(&self, token_ids: &[i64]) -> crate::Result<String> {
        let vocab = self
            .vocab
            .as_ref()
            .ok_or_else(|| crate::Error::Model("Vocabulary not loaded".to_string()))?;

        let blank_token_id = (vocab.len() - 1) as i64; // CTC blank is typically the last token
        eprintln!(
            "🔍 decode_ctc_tokens: {} tokens, blank_id={}",
            token_ids.len(),
            blank_token_id
        );

        let mut result = String::new();
        let mut prev_token_id: Option<i64> = None;
        let mut skipped_blank = 0;
        let mut skipped_repeat = 0;
        let mut skipped_special = 0;
        let mut kept_tokens = 0;

        for &token_id in token_ids {
            // Validate token ID
            if token_id < 0 || token_id as usize >= vocab.len() {
                debug!("Skipping invalid token ID: {}", token_id);
                continue;
            }

            // CTC decoding: skip blank tokens and repeated tokens
            if token_id == blank_token_id {
                prev_token_id = Some(token_id);
                skipped_blank += 1;
                continue;
            }

            if Some(token_id) == prev_token_id {
                skipped_repeat += 1;
                continue; // Skip repeated tokens
            }

            // Get the token string
            let token = &vocab[token_id as usize];

            // Skip special tokens (format: <token>)
            if token.starts_with('<') && token.ends_with('>') {
                prev_token_id = Some(token_id);
                skipped_special += 1;
                continue;
            }

            kept_tokens += 1;

            // Handle SentencePiece subword tokens (▁ indicates word boundary)
            if let Some(stripped) = token.strip_prefix('▁') {
                if !result.is_empty() {
                    result.push(' ');
                }
                // Remove the ▁ marker
                result.push_str(stripped); // ▁ is 3 bytes in UTF-8
            } else {
                result.push_str(token);
            }

            prev_token_id = Some(token_id);
        }

        eprintln!(
            "🔍 Decoding summary: skipped {} blank, {} repeat, {} special → kept {} tokens → result: '{}'",
            skipped_blank, skipped_repeat, skipped_special, kept_tokens, result
        );

        Ok(result.trim().to_string())
    }

    /// Normalize audio samples
    fn normalize_audio(&self, samples: &[f32]) -> Vec<f32> {
        // Find max absolute value for normalization
        let max_abs = samples.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);

        if max_abs > 0.0 {
            samples.iter().map(|&x| x / max_abs).collect()
        } else {
            samples.to_vec()
        }
    }

    /// Extract mel spectrogram features from audio
    /// Returns [num_mel_bins, num_frames] shaped feature matrix
    fn extract_mel_features(&self, samples: &[f32], _sample_rate: u32) -> crate::Result<Vec<f32>> {
        use rustfft::FftPlanner;
        use rustfft::num_complex::Complex;
        use std::f32::consts::PI;

        // Feature extraction parameters
        let n_mel_bins = self.n_mel_bins;
        const WINDOW_SIZE: usize = 400; // 25ms at 16kHz
        const HOP_SIZE: usize = 160; // 10ms at 16kHz
        const FFT_SIZE: usize = 512;
        const SAMPLE_RATE: f32 = 16000.0;
        const MEL_MIN_HZ: f32 = 0.0;
        const MEL_MAX_HZ: f32 = 8000.0;

        if samples.len() < WINDOW_SIZE {
            return Err(crate::Error::Model(format!(
                "Audio too short: {} samples (need at least {})",
                samples.len(),
                WINDOW_SIZE
            )));
        }

        // Calculate number of frames
        let num_frames = (samples.len() - WINDOW_SIZE) / HOP_SIZE + 1;

        // Create Hann window
        let window: Vec<f32> = (0..WINDOW_SIZE)
            .map(|i| 0.5 * (1.0 - ((2.0 * PI * i as f32) / (WINDOW_SIZE as f32 - 1.0)).cos()))
            .collect();

        // Create mel filterbank
        let mel_filters =
            Self::create_mel_filterbank(n_mel_bins, FFT_SIZE, MEL_MIN_HZ, MEL_MAX_HZ, SAMPLE_RATE);

        // Setup FFT
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);

        // Extract features for each frame
        let mut features = Vec::with_capacity(n_mel_bins * num_frames);

        for frame_idx in 0..num_frames {
            let start = frame_idx * HOP_SIZE;
            let end = start + WINDOW_SIZE;

            // Get frame and apply window
            let mut frame_complex: Vec<Complex<f32>> = samples[start..end]
                .iter()
                .zip(window.iter())
                .map(|(&s, &w)| Complex::new(s * w, 0.0))
                .collect();

            // Zero-pad to FFT size
            frame_complex.resize(FFT_SIZE, Complex::new(0.0, 0.0));

            // Compute FFT
            fft.process(&mut frame_complex);

            // Compute power spectrum
            let power_spectrum: Vec<f32> = frame_complex
                .iter()
                .take(FFT_SIZE / 2 + 1)
                .map(|c| c.norm_sqr())
                .collect();

            // Apply mel filterbank
            for mel_filter in mel_filters.iter().take(n_mel_bins) {
                let mut mel_energy = 0.0f32;
                for (freq_bin, &power) in power_spectrum.iter().enumerate() {
                    mel_energy += power * mel_filter[freq_bin];
                }

                // Apply log scale (add small epsilon to avoid log(0))
                let log_mel = (mel_energy + 1e-10).ln();
                features.push(log_mel);
            }
        }

        debug!(
            "Extracted mel features: {} bins x {} frames",
            n_mel_bins, num_frames
        );
        Ok(features)
    }

    /// Create mel filterbank matrix
    /// Returns [n_mels][fft_bins] matrix
    fn create_mel_filterbank(
        n_mels: usize,
        fft_size: usize,
        min_hz: f32,
        max_hz: f32,
        sample_rate: f32,
    ) -> Vec<Vec<f32>> {
        let n_fft_bins = fft_size / 2 + 1;

        // Helper: Hz to Mel
        let hz_to_mel = |hz: f32| 2595.0 * (1.0 + hz / 700.0).log10();

        // Helper: Mel to Hz
        let mel_to_hz = |mel: f32| 700.0 * (10.0f32.powf(mel / 2595.0) - 1.0);

        // Create mel scale
        let min_mel = hz_to_mel(min_hz);
        let max_mel = hz_to_mel(max_hz);
        let mel_points: Vec<f32> = (0..=n_mels + 1)
            .map(|i| mel_to_hz(min_mel + (max_mel - min_mel) * i as f32 / (n_mels + 1) as f32))
            .collect();

        // Convert mel points to FFT bin indices
        let bin_points: Vec<f32> = mel_points
            .iter()
            .map(|&hz| (fft_size as f32 * hz / sample_rate).floor())
            .collect();

        // Create filterbank
        let mut filterbank = vec![vec![0.0f32; n_fft_bins]; n_mels];

        for mel_idx in 0..n_mels {
            let left = bin_points[mel_idx] as usize;
            let center = bin_points[mel_idx + 1] as usize;
            let right = bin_points[mel_idx + 2] as usize;

            // Rising slope
            for (bin, value) in filterbank[mel_idx]
                .iter_mut()
                .enumerate()
                .take(center)
                .skip(left)
            {
                *value = (bin as f32 - left as f32) / (center as f32 - left as f32);
            }

            // Falling slope
            for (bin, value) in filterbank[mel_idx]
                .iter_mut()
                .enumerate()
                .take(right.min(n_fft_bins))
                .skip(center)
            {
                *value = (right as f32 - bin as f32) / (right as f32 - center as f32);
            }
        }

        filterbank
    }
}

#[cfg(feature = "onnx")]
impl Default for OnnxRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create OnnxRuntime")
    }
}

#[cfg(feature = "onnx")]
impl ModelRuntime for OnnxRuntime {
    fn load(&mut self, config: ModelConfig) -> crate::Result<()> {
        info!("Loading ONNX Runtime model: {}", config.model_path);

        // Resolve model directory
        let model_dir = self.resolve_model_dir(&config.model_path)?;

        if !model_dir.exists() {
            return Err(crate::Error::Model(format!(
                "Model directory not found: {:?}\nDownload with: onevox models download {}",
                model_dir, config.model_path
            )));
        }

        // Validate all required files exist before loading
        let required_files = vec!["vocab.txt"];
        let mut missing_files = Vec::new();

        for file in &required_files {
            let file_path = model_dir.join(file);
            if !file_path.exists() {
                missing_files.push(file.to_string());
            }
        }

        if !missing_files.is_empty() {
            return Err(crate::Error::Model(format!(
                "Model download incomplete. Missing files: {}\n\
                 Run 'onevox models download {}' to complete the download.",
                missing_files.join(", "),
                config.model_path
            )));
        }

        // Find encoder model file
        let encoder_path = self.find_encoder_path(&model_dir)?;

        info!("Loading encoder from: {:?}", encoder_path);

        // Load vocabulary
        let vocab = self.load_vocab(&model_dir)?;

        // Load model config to get mel bins count
        let n_mel_bins = self.load_model_config(&model_dir)?;

        // Read model file into memory
        let model_bytes = std::fs::read(&encoder_path).map_err(|e| {
            crate::Error::Model(format!("Failed to read encoder model file: {}", e))
        })?;

        info!("Model file size: {} MB", model_bytes.len() / (1024 * 1024));

        // Configure ONNX Runtime session
        let encoder_session = Session::builder()
            .map_err(|e| crate::Error::Model(format!("Failed to create session builder: {}", e)))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| crate::Error::Model(format!("Failed to set optimization level: {}", e)))?
            .with_intra_threads(config.n_threads as usize)
            .map_err(|e| crate::Error::Model(format!("Failed to set thread count: {}", e)))?
            .commit_from_memory(&model_bytes)
            .map_err(|e| crate::Error::Model(format!("Failed to load ONNX model: {}", e)))?;

        info!("✅ ONNX Runtime model loaded successfully");
        info!("   Model directory: {:?}", model_dir);
        info!("   Vocabulary size: {}", vocab.len());
        info!("   Mel bins: {}", n_mel_bins);
        info!("   Thread count: {}", config.n_threads);

        self.encoder_session = Some(encoder_session);
        self.vocab = Some(vocab);
        self.config = Some(config);
        self.model_dir = Some(model_dir);
        self.n_mel_bins = n_mel_bins;

        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.encoder_session.is_some() && self.vocab.is_some()
    }

    fn transcribe(&mut self, samples: &[f32], sample_rate: u32) -> crate::Result<Transcription> {
        // Validate input
        if !self.is_loaded() {
            return Err(crate::Error::Model("Model not loaded".to_string()));
        }

        if sample_rate != 16000 {
            return Err(crate::Error::Model(format!(
                "Sample rate must be 16kHz, got {}Hz. Please resample audio.",
                sample_rate
            )));
        }

        if samples.is_empty() {
            return Err(crate::Error::Model("No audio samples provided".to_string()));
        }

        let start_time = std::time::Instant::now();
        let audio_duration = samples.len() as f32 / sample_rate as f32;

        info!(
            "Transcribing {} samples ({:.2}s of audio)",
            samples.len(),
            audio_duration
        );

        // Normalize audio
        let normalized_audio = self.normalize_audio(samples);

        // Debug: check audio statistics
        let max_audio = normalized_audio
            .iter()
            .map(|&x| x.abs())
            .fold(0.0f32, f32::max);
        let mean_audio = normalized_audio.iter().sum::<f32>() / normalized_audio.len() as f32;
        eprintln!(
            "🔍 Audio stats: max={:.4}, mean={:.4}, samples={}",
            max_audio,
            mean_audio,
            normalized_audio.len()
        );

        // Extract mel spectrogram features
        let mel_features_raw = self.extract_mel_features(&normalized_audio, sample_rate)?;

        // Calculate dimensions
        // mel_features_raw is [frame0_mel0, frame0_mel1, ..., frame1_mel0, frame1_mel1, ...]
        // We need to transpose to [batch=1, n_mels, n_frames]
        let n_mel_bins = self.n_mel_bins;
        let n_frames = mel_features_raw.len() / n_mel_bins;
        let feature_length = n_frames as i64;

        debug!("Mel features: {} bins x {} frames", n_mel_bins, n_frames);

        // Transpose: convert from [n_frames, n_mels] to [n_mels, n_frames]
        // Input: row-major [frame][mel_bin]
        // Output: [mel_bin][frame] for ONNX [batch=1, features, time]
        let mut mel_features = vec![0.0f32; n_mel_bins * n_frames];
        for frame_idx in 0..n_frames {
            for mel_idx in 0..n_mel_bins {
                let src_idx = frame_idx * n_mel_bins + mel_idx;
                let dst_idx = mel_idx * n_frames + frame_idx;
                mel_features[dst_idx] = mel_features_raw[src_idx];
            }
        }

        // Debug: check mel feature statistics BEFORE normalization
        let mel_min = mel_features.iter().copied().fold(f32::INFINITY, f32::min);
        let mel_max = mel_features
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        let mel_mean = mel_features.iter().sum::<f32>() / mel_features.len() as f32;
        eprintln!(
            "🔍 Mel features (before norm): {} frames, min={:.2}, max={:.2}, mean={:.2}",
            n_frames, mel_min, mel_max, mel_mean
        );

        // Normalize mel features to mean=0, std=1 (per-utterance normalization)
        // This is required by NeMo models
        let mel_std = {
            let variance = mel_features
                .iter()
                .map(|&x| (x - mel_mean).powi(2))
                .sum::<f32>()
                / mel_features.len() as f32;
            variance.sqrt()
        };

        if mel_std > 1e-6 {
            for feature in mel_features.iter_mut() {
                *feature = (*feature - mel_mean) / mel_std;
            }
        }

        // Debug: check mel feature statistics AFTER normalization
        let mel_min_norm = mel_features.iter().copied().fold(f32::INFINITY, f32::min);
        let mel_max_norm = mel_features
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        let mel_mean_norm = mel_features.iter().sum::<f32>() / mel_features.len() as f32;
        eprintln!(
            "🔍 Mel features (after norm): min={:.2}, max={:.2}, mean={:.2}, std={:.2}",
            mel_min_norm, mel_max_norm, mel_mean_norm, mel_std
        );

        // Prepare ONNX Runtime inputs
        // Parakeet expects shape: [batch_size=1, features, time_frames]
        let audio_shape = vec![1, n_mel_bins, n_frames];
        let length_shape = vec![1];

        // Convert to boxed slices for ort
        let audio_data: Box<[f32]> = mel_features.into_boxed_slice();
        let length_data: Box<[i64]> = vec![feature_length].into_boxed_slice();

        // Create ort Values
        let audio_value = Value::from_array((audio_shape.as_slice(), audio_data))
            .map_err(|e| crate::Error::Model(format!("Failed to create audio tensor: {}", e)))?;

        let length_value = Value::from_array((length_shape.as_slice(), length_data))
            .map_err(|e| crate::Error::Model(format!("Failed to create length tensor: {}", e)))?;

        // Prepare inputs
        let inputs = ort::inputs![
            "audio_signal" => audio_value,
            "length" => length_value
        ];

        // Run inference
        let token_ids = {
            let session = self.encoder_session.as_mut().ok_or_else(|| {
                crate::Error::Model("Encoder session not initialized".to_string())
            })?;

            let outputs = session
                .run(inputs)
                .map_err(|e| crate::Error::Model(format!("Inference failed: {}", e)))?;

            // Extract logits output
            // Expected shape: [batch_size=1, time_steps, vocab_size]
            // Try common output names: "outputs", "logits", "output", "logprobs"
            let output_names = ["outputs", "logits", "output", "logprobs"];
            let logits_value = output_names
                .iter()
                .find_map(|&name| outputs.get(name))
                .ok_or_else(|| {
                    // Log available outputs for debugging
                    let available: Vec<String> =
                        outputs.iter().map(|(k, _)| k.to_string()).collect();
                    crate::Error::Model(format!(
                        "Could not find output tensor. Available outputs: {:?}",
                        available
                    ))
                })?;

            let logits = logits_value.try_extract_tensor::<f32>().map_err(|e| {
                crate::Error::Model(format!("Failed to extract logits tensor: {}", e))
            })?;

            // Get shape and data
            let shape = logits.0;
            let data: Vec<f32> = logits.1.to_vec();

            if shape.len() != 3 {
                return Err(crate::Error::Model(format!(
                    "Expected 3D logits tensor [batch, time, vocab], got shape: {:?}",
                    shape
                )));
            }

            let time_steps = shape[1] as usize;
            let vocab_size = shape[2] as usize;

            debug!("Logits shape: {:?}", shape);
            debug!("Time steps: {}, Vocab size: {}", time_steps, vocab_size);

            eprintln!(
                "🔍 Model output: shape={:?}, time_steps={}, vocab_size={}",
                shape, time_steps, vocab_size
            );

            // Debug: check first timestep logits
            let first_10_logits: Vec<f32> = data.iter().take(10).copied().collect();
            eprintln!("🔍 First 10 logits at t=0: {:?}", first_10_logits);

            // Debug: check blank token value at t=0
            let blank_idx = vocab_size - 1;
            if blank_idx < data.len() {
                eprintln!(
                    "🔍 Blank token (ID {}) at t=0: {:.4}",
                    blank_idx, data[blank_idx]
                );
            }

            // Debug: check top 5 values at t=0
            let mut t0_values: Vec<(usize, f32)> = data
                .iter()
                .take(vocab_size)
                .enumerate()
                .map(|(i, &v)| (i, v))
                .collect();
            t0_values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            eprintln!("🔍 Top 5 tokens at t=0:");
            for (i, &(token_id, value)) in t0_values.iter().enumerate().take(5.min(t0_values.len()))
            {
                eprintln!("  #{}: ID {}, value {:.4}", i + 1, token_id, value);
            }

            // Debug: check top 5 values at t=0
            let mut t0_values: Vec<(usize, f32)> = data
                .iter()
                .take(vocab_size)
                .enumerate()
                .map(|(i, &v)| (i, v))
                .collect();
            t0_values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            eprintln!("🔍 Top 5 tokens at t=0:");
            for &(token_id, value) in t0_values.iter().take(5.min(t0_values.len())) {
                eprintln!("  ID {}: {:.4}", token_id, value);
            }

            // Debug: check if blank token (8192) has high probability at any timestep
            let blank_id = 8192;
            if blank_id < vocab_size {
                let mut blank_max = f32::NEG_INFINITY;
                for t in 0..time_steps.min(10) {
                    let idx = t * vocab_size + blank_id;
                    if idx < data.len() {
                        blank_max = blank_max.max(data[idx]);
                    }
                }
                eprintln!(
                    "🔍 Blank token (ID {}) max value in first 10 steps: {:.4}",
                    blank_id, blank_max
                );
            }

            // Greedy CTC decoding: argmax over vocab dimension for each timestep
            let mut token_ids = Vec::with_capacity(time_steps);

            for t in 0..time_steps {
                let mut max_idx = 0;
                let mut max_val = f32::NEG_INFINITY;

                for v in 0..vocab_size {
                    let idx = t * vocab_size + v;
                    if idx >= data.len() {
                        return Err(crate::Error::Model(format!(
                            "Index out of bounds: {} >= {}",
                            idx,
                            data.len()
                        )));
                    }
                    let val = data[idx];
                    if val > max_val {
                        max_val = val;
                        max_idx = v as i64;
                    }
                }

                token_ids.push(max_idx);

                // Debug first few timesteps
                if t < 5 {
                    eprintln!("🔍 t={}: max_idx={}, max_val={:.4}", t, max_idx, max_val);
                }
            }

            token_ids
        }; // Drop session borrow here

        // Debug: log token statistics
        let vocab = self.vocab.as_ref().unwrap();
        let blank_token_id = (vocab.len() - 1) as i64;
        let num_blank = token_ids.iter().filter(|&&id| id == blank_token_id).count();
        let num_non_blank = token_ids.len() - num_blank;
        eprintln!(
            "🔍 Token stats: {} total, {} non-blank ({:.1}%), {} blank (ID={})",
            token_ids.len(),
            num_non_blank,
            (num_non_blank as f32 / token_ids.len() as f32) * 100.0,
            num_blank,
            blank_token_id
        );

        // Sample first 20 non-blank tokens for debugging
        let sample_tokens: Vec<(i64, String)> = token_ids
            .iter()
            .filter(|&&id| id != blank_token_id)
            .take(20)
            .map(|&id| {
                (
                    id,
                    vocab
                        .get(id as usize)
                        .cloned()
                        .unwrap_or_else(|| format!("<id:{}>", id)),
                )
            })
            .collect();
        if !sample_tokens.is_empty() {
            eprintln!("🔍 Sample non-blank tokens: {:?}", sample_tokens);
        } else {
            eprintln!("⚠️  NO non-blank tokens found!");
        }

        // Decode tokens to text
        let text = self.decode_ctc_tokens(&token_ids)?;

        let processing_time = start_time.elapsed();
        let processing_ms = processing_time.as_millis() as u64;

        info!(
            "✅ Transcription complete: \"{}\" ({} ms, {:.1}x real-time)",
            text,
            processing_ms,
            audio_duration * 1000.0 / processing_ms as f32
        );

        // ONNX models (like Parakeet) support multilingual transcription
        // Language is auto-detected by the model
        Ok(Transcription {
            text,
            language: None,   // Auto-detected by model
            confidence: None, // CTC models don't easily provide confidence scores
            processing_time_ms: processing_ms,
            tokens: Some(token_ids.len()),
        })
    }

    fn unload(&mut self) {
        info!("Unloading ONNX Runtime model");
        self.encoder_session = None;
        self.vocab = None;
        self.config = None;
        self.model_dir = None;
    }

    fn name(&self) -> &str {
        "onnx-runtime"
    }

    fn info(&self) -> ModelInfo {
        let config = self.config.as_ref();

        // Calculate total size of all model files in directory
        let size_bytes = self
            .model_dir
            .as_ref()
            .and_then(|dir| {
                std::fs::read_dir(dir).ok().map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter_map(|e| e.metadata().ok())
                        .filter(|m| m.is_file())
                        .map(|m| m.len())
                        .sum()
                })
            })
            .unwrap_or(0);

        ModelInfo {
            name: self.name().to_string(),
            size_bytes,
            model_type: config
                .as_ref()
                .map(|c| c.model_path.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            backend: "onnx-runtime".to_string(),
            gpu_enabled: config.map(|c| c.use_gpu).unwrap_or(false),
        }
    }
}

// Stub implementation when feature is disabled
#[cfg(not(feature = "onnx"))]
pub struct OnnxRuntime;

#[cfg(not(feature = "onnx"))]
impl OnnxRuntime {
    pub fn new() -> crate::Result<Self> {
        Err(crate::Error::Model(
            "ONNX feature not enabled. Rebuild with --features onnx".to_string(),
        ))
    }
}

#[cfg(not(feature = "onnx"))]
impl super::runtime::ModelRuntime for OnnxRuntime {
    fn load(&mut self, _config: super::runtime::ModelConfig) -> crate::Result<()> {
        Err(crate::Error::Model("ONNX feature not enabled".to_string()))
    }

    fn is_loaded(&self) -> bool {
        false
    }

    fn transcribe(
        &mut self,
        _samples: &[f32],
        _sample_rate: u32,
    ) -> crate::Result<super::runtime::Transcription> {
        Err(crate::Error::Model("ONNX feature not enabled".to_string()))
    }

    fn unload(&mut self) {}

    fn name(&self) -> &str {
        "onnx-runtime (disabled)"
    }

    fn info(&self) -> super::runtime::ModelInfo {
        super::runtime::ModelInfo {
            name: "onnx-runtime".to_string(),
            size_bytes: 0,
            model_type: "onnx".to_string(),
            backend: "onnx-runtime (feature disabled)".to_string(),
            gpu_enabled: false,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "onnx")]
mod tests {
    use super::*;

    #[test]
    fn test_create_backend() {
        let backend = OnnxRuntime::new();
        assert!(backend.is_ok());
    }

    #[test]
    fn test_not_loaded_initially() {
        let backend = OnnxRuntime::new().unwrap();
        assert!(!backend.is_loaded());
    }

    #[test]
    fn test_normalize_audio() {
        let backend = OnnxRuntime::new().unwrap();
        let samples = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let normalized = backend.normalize_audio(&samples);

        // After normalization, max absolute value should be 1.0
        let max_abs = normalized.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        assert!((max_abs - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_decode_empty_tokens() {
        let backend = OnnxRuntime {
            encoder_session: None,
            vocab: Some(vec![
                "<blank>".to_string(),
                "a".to_string(),
                "b".to_string(),
            ]),
            config: None,
            model_dir: None,
            n_mel_bins: 80,
        };

        let result = backend.decode_ctc_tokens(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
}
