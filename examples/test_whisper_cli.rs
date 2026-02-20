// Test the WhisperCppCli backend with real captured audio

use hound::WavReader;
use onevox::models::{ModelConfig, ModelRuntime, WhisperCppCli};

fn main() -> onevox::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("Testing WhisperCppCli backend...\n");

    // Find a captured audio file
    let audio_file = std::fs::read_dir(dirs::cache_dir().unwrap().join("onevox/debug"))?
        .filter_map(|e| e.ok())
        .find(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "wav")
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            onevox::Error::Model("No audio files found in debug directory".to_string())
        })?;

    println!("Using audio file: {}", audio_file.path().display());

    // Read the WAV file
    let mut reader = WavReader::open(audio_file.path())
        .map_err(|e| onevox::Error::Audio(format!("Failed to open WAV file: {}", e)))?;
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    println!(
        "Loaded {} samples ({:.2}s at 16kHz)\n",
        samples.len(),
        samples.len() as f32 / 16000.0
    );

    // Create model config
    let model_path = dirs::cache_dir()
        .unwrap()
        .join("onevox/models/ggml-base.en/ggml-base.en.bin");

    let config = ModelConfig {
        model_path: model_path.to_string_lossy().to_string(),
        language: "en".to_string(),
        use_gpu: false,
        n_threads: 4,
        beam_size: 5,
        translate: false,
    };

    // Create and load the model
    let mut model = WhisperCppCli::new(None);
    println!("Loading model: {}", config.model_path);
    model.load(config)?;
    println!("Model loaded successfully!\n");

    // Check if loaded
    assert!(model.is_loaded());

    // Get model info
    let info = model.info();
    println!("Model info:");
    println!("  Name: {}", info.name);
    println!("  Type: {}", info.model_type);
    println!("  Backend: {}", info.backend);
    println!("  Size: {:.2} MB", info.size_bytes as f64 / 1024.0 / 1024.0);
    println!();

    // Transcribe
    println!("Transcribing audio...");
    let start = std::time::Instant::now();
    let transcript = model.transcribe(&samples, 16000)?;
    let elapsed = start.elapsed();

    println!("\n=== RESULTS ===");
    println!("Text: {}", transcript.text);
    println!("Language: {:?}", transcript.language);
    println!(
        "Processing time: {:.2}s",
        transcript.processing_time_ms as f32 / 1000.0
    );
    println!("Wall clock time: {:.2}s", elapsed.as_secs_f32());
    println!("===============\n");

    // Unload
    model.unload();
    assert!(!model.is_loaded());

    Ok(())
}
