//! Debug Decoder - Inspect ONNX decoder behavior step-by-step
//!
//! This script helps diagnose the repetition loop issue by:
//! 1. Generating simple sine wave test audio
//! 2. Computing mel spectrogram
//! 3. Running encoder
//! 4. Running decoder with detailed logging
//! 5. Analyzing logits and token probabilities

use anyhow::Result;
use ndarray::{Array1, Array2, Array3};
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;
use std::path::PathBuf;

// Whisper constants
const SAMPLE_RATE: usize = 16000;
const N_FFT: usize = 400;
const HOP_LENGTH: usize = 160;
const N_MELS: usize = 80;
const CHUNK_LENGTH: usize = 30;
const N_SAMPLES: usize = CHUNK_LENGTH * SAMPLE_RATE;
const N_FRAMES: usize = N_SAMPLES / HOP_LENGTH;

// Special tokens
const EOT_TOKEN: i64 = 50256;
const SOT_TOKEN: i64 = 50257;
const ENGLISH_TOKEN: i64 = 50258;
const TRANSCRIBE_TOKEN: i64 = 50358;
const NO_TIMESTAMPS_TOKEN: i64 = 50362;

fn main() -> Result<()> {
    // Set up ONNX Runtime
    unsafe {
        std::env::set_var("ORT_DYLIB_PATH", "/opt/homebrew/lib/libonnxruntime.dylib");
    }

    println!("🔍 Whisper ONNX Decoder Debugger");
    println!("{}", "=".repeat(80));

    // 1. Generate test audio (1 second of 440Hz sine wave)
    println!("📊 Step 1: Generating test audio");
    let samples = generate_test_audio();
    println!("  Generated {} samples", samples.len());
    println!(
        "  Max amplitude: {:.4}",
        samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
    );
    println!(
        "  Min amplitude: {:.4}\n",
        samples.iter().cloned().fold(f32::INFINITY, f32::min)
    );

    // 2. Compute mel spectrogram
    println!("📊 Step 2: Computing mel spectrogram");
    let mel_filters = compute_mel_filters();
    let mel_spec = compute_mel_spectrogram(&samples, &mel_filters)?;
    println!("  Mel spectrogram shape: {:?}", mel_spec.shape());
    println!(
        "  Mel spec max: {:.4}",
        mel_spec.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
    );
    println!(
        "  Mel spec min: {:.4}\n",
        mel_spec.iter().cloned().fold(f32::INFINITY, f32::min)
    );

    // 3. Load encoder
    println!("📊 Step 3: Loading ONNX encoder");
    let model_dir = PathBuf::from(std::env::var("HOME").unwrap())
        .join("Library/Caches/vox/models/whisper-tiny.en/onnx");
    let encoder_path = model_dir.join("encoder_model.onnx");

    let mut encoder = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .commit_from_file(&encoder_path)?;
    println!("  ✅ Encoder loaded\n");

    // 4. Run encoder
    println!("📊 Step 4: Running encoder");
    let input_tensor = Tensor::from_array(mel_spec)?;
    let encoder_outputs = encoder.run(ort::inputs!["input_features" => input_tensor])?;
    let audio_features = encoder_outputs["last_hidden_state"].try_extract_tensor::<f32>()?;
    let (shape, data) = audio_features;

    println!("  Encoder output shape: {:?}", shape);
    println!(
        "  Encoder output max: {:.4}",
        data.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
    );
    println!(
        "  Encoder output min: {:.4}",
        data.iter().cloned().fold(f32::INFINITY, f32::min)
    );
    println!(
        "  Encoder output mean: {:.4}\n",
        data.iter().sum::<f32>() / data.len() as f32
    );

    // Copy to owned array
    let audio_features = Array3::from_shape_vec(
        (shape[0] as usize, shape[1] as usize, shape[2] as usize),
        data.to_vec(),
    )?;

    // 5. Load decoder
    println!("📊 Step 5: Loading ONNX decoder");
    let decoder_path = model_dir.join("decoder_model_merged.onnx");
    let mut decoder = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .commit_from_file(&decoder_path)?;
    println!("  ✅ Decoder loaded\n");

    // 6. Run decoder with detailed debugging
    println!("📊 Step 6: Running decoder (first 10 steps with detailed logging)");
    println!("{}", "=".repeat(80));

    let mut tokens = vec![
        SOT_TOKEN,
        ENGLISH_TOKEN,
        TRANSCRIBE_TOKEN,
        NO_TIMESTAMPS_TOKEN,
    ];
    println!("\n Initial tokens: {:?}", tokens);
    println!(
        "  SOT={}, ENGLISH={}, TRANSCRIBE={}, NO_TIMESTAMPS={}\n",
        SOT_TOKEN, ENGLISH_TOKEN, TRANSCRIBE_TOKEN, NO_TIMESTAMPS_TOKEN
    );

    for step in 0..10 {
        println!("{}", "─".repeat(80));
        println!("STEP {}: Current sequence length = {}", step, tokens.len());
        println!("Current tokens: {:?}", tokens);

        // Prepare inputs
        let tokens_array = Array2::from_shape_vec((1, tokens.len()), tokens.clone())?;
        let audio_features_tensor = Tensor::from_array(audio_features.clone())?;
        let tokens_tensor = Tensor::from_array(tokens_array)?;
        let use_cache = Array1::from_vec(vec![false]);
        let use_cache_tensor = Tensor::from_array(use_cache)?;

        // Run decoder
        let outputs = decoder.run(ort::inputs![
            "encoder_hidden_states" => audio_features_tensor,
            "input_ids" => tokens_tensor,
            "use_cache_branch" => use_cache_tensor,
        ])?;

        // Extract logits
        let logits_tensor = outputs["logits"].try_extract_tensor::<f32>()?;
        let (logits_shape, logits_data) = logits_tensor;

        println!("  Logits shape: {:?}", logits_shape);

        // Get last token's logits
        let vocab_size = logits_shape[logits_shape.len() - 1] as usize;
        let last_token_idx = (logits_shape[1] as usize - 1) * vocab_size;
        let last_logits = &logits_data[last_token_idx..last_token_idx + vocab_size];

        // Find top 10 tokens
        let mut indexed: Vec<(usize, f32)> = last_logits
            .iter()
            .enumerate()
            .map(|(i, &v)| (i, v))
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        println!("\n  📈 Top 10 logits:");
        for (rank, (token_id, logit)) in indexed.iter().take(10).enumerate() {
            println!(
                "    {}: token {:5} -> logit {:8.4}",
                rank + 1,
                token_id,
                logit
            );
        }

        // Apply softmax to top 10 to see probabilities
        let top_10_logits: Vec<f32> = indexed.iter().take(10).map(|(_, l)| *l).collect();
        let max_logit = top_10_logits[0];
        let exp_sum: f32 = top_10_logits.iter().map(|&x| (x - max_logit).exp()).sum();

        println!("\n  📊 Top 10 probabilities (after softmax):");
        for (rank, (token_id, logit)) in indexed.iter().take(10).enumerate() {
            let prob = (logit - max_logit).exp() / exp_sum;
            println!(
                "    {}: token {:5} -> prob {:6.2}%",
                rank + 1,
                token_id,
                prob * 100.0
            );
        }

        // Get next token
        let next_token = indexed[0].0 as i64;
        println!("\n  ✅ Selected token: {}", next_token);

        // Check for special tokens
        if next_token == EOT_TOKEN {
            println!("  🛑 End of transcript token generated");
            break;
        }

        tokens.push(next_token);
        println!();
    }

    println!("{}", "=".repeat(80));
    println!("\n✅ Decoder debugging complete");
    println!("\nFinal token sequence: {:?}", tokens);

    Ok(())
}

/// Generate test audio: 1 second of 440Hz sine wave
fn generate_test_audio() -> Vec<f32> {
    let frequency = 440.0; // A4 note
    let duration = 1.0; // 1 second
    let num_samples = (SAMPLE_RATE as f32 * duration) as usize;

    (0..num_samples)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5
        })
        .collect()
}

/// Compute mel filterbank
fn compute_mel_filters() -> Array2<f32> {
    let n_freqs = N_FFT / 2 + 1;
    let mut filters = Array2::<f32>::zeros((N_MELS, n_freqs));

    let mel_min = hz_to_mel(0.0);
    let mel_max = hz_to_mel(SAMPLE_RATE as f32 / 2.0);
    let mel_points: Vec<f32> = (0..=N_MELS + 1)
        .map(|i| mel_min + (mel_max - mel_min) * i as f32 / (N_MELS + 1) as f32)
        .collect();

    let hz_points: Vec<f32> = mel_points.iter().map(|&m| mel_to_hz(m)).collect();
    let bin_points: Vec<usize> = hz_points
        .iter()
        .map(|&hz| ((N_FFT + 1) as f32 * hz / SAMPLE_RATE as f32).floor() as usize)
        .collect();

    for i in 0..N_MELS {
        let start = bin_points[i];
        let mid = bin_points[i + 1];
        let end = bin_points[i + 2];

        for j in start..mid {
            if mid > start {
                filters[[i, j]] = (j - start) as f32 / (mid - start) as f32;
            }
        }
        for j in mid..end {
            if end > mid {
                filters[[i, j]] = (end - j) as f32 / (end - mid) as f32;
            }
        }

        // Normalize
        let sum: f32 = filters.row(i).sum();
        if sum > 0.0 {
            for j in 0..n_freqs {
                filters[[i, j]] /= sum;
            }
        }
    }

    filters
}

fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10f32.powf(mel / 2595.0) - 1.0)
}

/// Compute mel spectrogram
fn compute_mel_spectrogram(samples: &[f32], filters: &Array2<f32>) -> Result<Array3<f32>> {
    // Pad to N_SAMPLES
    let mut padded = vec![0.0f32; N_SAMPLES];
    let copy_len = samples.len().min(N_SAMPLES);
    padded[..copy_len].copy_from_slice(&samples[..copy_len]);

    let mut spectrogram = Array2::<f32>::zeros((N_MELS, N_FRAMES));

    for frame_idx in 0..N_FRAMES {
        let start = frame_idx * HOP_LENGTH;
        let end = (start + N_FFT).min(N_SAMPLES);

        if start >= N_SAMPLES {
            break;
        }

        // Extract frame with Hann window
        let mut frame = vec![0.0f32; N_FFT];
        for i in 0..(end - start) {
            let window =
                0.5 * (1.0 - ((2.0 * std::f32::consts::PI * i as f32) / N_FFT as f32).cos());
            frame[i] = padded[start + i] * window;
        }

        // Compute FFT magnitudes
        let mut magnitudes = vec![0.0f32; N_FFT / 2 + 1];
        for k in 0..magnitudes.len() {
            let mut real = 0.0f32;
            let mut imag = 0.0f32;
            for (n, &sample) in frame.iter().enumerate() {
                let angle = -2.0 * std::f32::consts::PI * k as f32 * n as f32 / N_FFT as f32;
                real += sample * angle.cos();
                imag += sample * angle.sin();
            }
            magnitudes[k] = (real * real + imag * imag).sqrt();
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

    // Convert to log scale
    let log_spec = spectrogram.mapv(|x| (x.max(1e-10)).ln());

    // Normalize
    let max_val = log_spec.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let normalized = log_spec.mapv(|x| ((x - max_val) / 4.0).max(-8.0));

    // Reshape to [1, N_MELS, N_FRAMES]
    let shaped = normalized
        .insert_axis(ndarray::Axis(0))
        .into_shape_with_order((1, N_MELS, N_FRAMES))?;

    Ok(shaped)
}
