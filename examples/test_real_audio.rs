//! Test decoder with real captured audio
//!
//! Loads a saved audio file and runs it through the decoder

use anyhow::Result;
use onevox::models::runtime::{ModelConfig, ModelRuntime};
use onevox::models::whisper_cpp::WhisperCpp;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("🎯 Testing Whisper with real captured audio\n");

    // Find the latest captured audio file
    let debug_dir =
        PathBuf::from(std::env::var("HOME").unwrap()).join("Library/Caches/onevox/debug");

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
    let mut model = WhisperCpp::new()?;

    let config = ModelConfig {
        model_path: "ggml-base.en".to_string(), // Model ID for whisper.cpp
        use_gpu: false,
        n_threads: 4,
        beam_size: 1,
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
