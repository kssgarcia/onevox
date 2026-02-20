//! Test decoder with real captured audio
//!
//! Loads a saved audio file and runs it through the decoder

use anyhow::Result;
use std::path::PathBuf;
use vox::models::runtime::{ModelConfig, ModelRuntime};
use vox::models::whisper_onnx::WhisperOnnx;

fn main() -> Result<()> {
    // Set up ONNX Runtime
    unsafe {
        std::env::set_var("ORT_DYLIB_PATH", "/opt/homebrew/lib/libonnxruntime.dylib");
    }

    println!("🎯 Testing Whisper with real captured audio\n");

    // Find the latest captured audio file
    let debug_dir = PathBuf::from(std::env::var("HOME").unwrap()).join("Library/Caches/vox/debug");

    let audio_files: Vec<_> = std::fs::read_dir(&debug_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("wav"))
        .collect();

    if audio_files.is_empty() {
        eprintln!("❌ No audio files found in {:?}", debug_dir);
        eprintln!("💡 Press Cmd+Shift+0 to capture some audio first");
        return Ok(());
    }

    // Use the latest file
    let latest = audio_files
        .iter()
        .max_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()))
        .unwrap();

    let audio_path = latest.path();
    println!("📁 Loading audio from: {}", audio_path.display());

    // Load WAV file
    let mut reader = hound::WavReader::open(&audio_path)?;
    let spec = reader.spec();
    println!(
        "📊 Audio format: {} Hz, {} channels, {} bits",
        spec.sample_rate, spec.channels, spec.bits_per_sample
    );

    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / 32768.0)
        .collect();

    println!(
        "📊 Loaded {} samples ({:.2}s)\n",
        samples.len(),
        samples.len() as f32 / spec.sample_rate as f32
    );

    // Create and load model
    println!("🔄 Loading Whisper model...");
    let mut model = WhisperOnnx::new()?;

    let config = ModelConfig {
        model_path: "whisper-tiny.en".to_string(), // Model ID, not full path
        language: "en".to_string(),
        use_gpu: false,
        n_threads: 4,
        beam_size: 1,
        translate: false,
    };

    model.load(config)?;
    println!("✅ Model loaded\n");

    // Transcribe
    println!("🎙️  Transcribing...");
    let transcription = model.transcribe(&samples, spec.sample_rate)?;

    println!("\n📝 Results:");
    println!("  Text: '{}'", transcription.text);
    println!(
        "  Language: {}",
        transcription.language.as_deref().unwrap_or("unknown")
    );
    println!("  Processing time: {}ms", transcription.processing_time_ms);

    Ok(())
}
