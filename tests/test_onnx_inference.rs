//! Integration test for ONNX Runtime inference
//!
//! These tests require the Parakeet model to be downloaded.
//! Run: `cargo run --release -- models download parakeet-ctc-0.6b`
//!
//! To skip these tests if model is not available, they check for model
//! presence and skip gracefully.

#[cfg(feature = "onnx")]
#[test]
fn test_onnx_transcription_silence() {
    use onevox::models::{ModelConfig, ModelRuntime, OnnxRuntime};

    // Create ONNX Runtime backend
    let mut model = OnnxRuntime::new().expect("Failed to create ONNX Runtime backend");

    // Load model
    let config = ModelConfig {
        model_path: "parakeet-ctc-0.6b".to_string(),
        use_gpu: false,
        n_threads: 4,
        beam_size: 1,
    };

    // Try to load model - skip test if model not downloaded
    match model.load(config) {
        Ok(_) => {
            // Model loaded successfully, continue test
        }
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("Model download incomplete")
                || err_msg.contains("Missing files")
                || err_msg.contains("Model directory not found")
                || err_msg.contains("Download with:")
            {
                eprintln!("⚠️  Skipping test: Parakeet model not downloaded");
                eprintln!("   Run: cargo run --release -- models download parakeet-ctc-0.6b");
                return; // Skip test gracefully
            } else {
                panic!("Failed to load model: {:?}", e);
            }
        }
    }

    // Create test audio: 1 second of silence at 16kHz
    let sample_rate: u32 = 16000;
    let samples: Vec<f32> = vec![0.0; sample_rate as usize];

    // Transcribe
    let result = model.transcribe(&samples, sample_rate);

    // Should succeed (even if result is empty due to silence)
    assert!(result.is_ok(), "Transcription failed: {:?}", result.err());

    let transcription = result.unwrap();
    println!("Silence transcription: '{}'", transcription.text);
    println!("Processing time: {} ms", transcription.processing_time_ms);
    println!(
        "Speed: {:.1}x real-time",
        1000.0 / transcription.processing_time_ms as f32
    );

    // Verify it completed quickly (should be much faster than real-time)
    assert!(
        transcription.processing_time_ms < 5000,
        "Transcription too slow: {} ms",
        transcription.processing_time_ms
    );
}

#[cfg(feature = "onnx")]
#[test]
fn test_onnx_transcription_synthetic_audio() {
    use onevox::models::{ModelConfig, ModelRuntime, OnnxRuntime};
    use std::f32::consts::PI;

    // Create ONNX Runtime backend
    let mut model = OnnxRuntime::new().expect("Failed to create ONNX Runtime backend");

    // Load model
    let config = ModelConfig {
        model_path: "parakeet-ctc-0.6b".to_string(),
        use_gpu: false,
        n_threads: 4,
        beam_size: 1,
    };

    // Try to load model - skip test if model not downloaded
    match model.load(config) {
        Ok(_) => {
            // Model loaded successfully, continue test
        }
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("Model download incomplete")
                || err_msg.contains("Missing files")
                || err_msg.contains("Model directory not found")
                || err_msg.contains("Download with:")
            {
                eprintln!("⚠️  Skipping test: Parakeet model not downloaded");
                eprintln!("   Run: cargo run --release -- models download parakeet-ctc-0.6b");
                return; // Skip test gracefully
            } else {
                panic!("Failed to load model: {:?}", e);
            }
        }
    }

    // Create synthetic speech-like audio: 2 seconds of varying frequencies
    // This simulates speech formants with modulated sine waves
    let sample_rate: u32 = 16000;
    let duration_secs = 2.0;
    let num_samples = (sample_rate as f32 * duration_secs) as usize;

    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Simulate speech with multiple frequency components
        let f1 = 200.0 + 100.0 * (t * 3.0).sin(); // Base frequency (vocal fold vibration)
        let f2 = 800.0 + 200.0 * (t * 2.0).sin(); // First formant
        let f3 = 2400.0 + 300.0 * (t * 1.5).sin(); // Second formant

        let sample = 0.3 * (2.0 * PI * f1 * t).sin()
            + 0.2 * (2.0 * PI * f2 * t).sin()
            + 0.1 * (2.0 * PI * f3 * t).sin();

        // Add envelope to simulate speech pauses
        let envelope = (0.5 + 0.5 * (t * 4.0).sin()).max(0.0);

        samples.push(sample * envelope);
    }

    // Transcribe
    let result = model.transcribe(&samples, sample_rate);

    // Should succeed
    assert!(result.is_ok(), "Transcription failed: {:?}", result.err());

    let transcription = result.unwrap();
    println!("\nSynthetic audio transcription: '{}'", transcription.text);
    println!("Processing time: {} ms", transcription.processing_time_ms);
    println!(
        "Speed: {:.1}x real-time",
        duration_secs * 1000.0 / transcription.processing_time_ms as f32
    );
    println!("Tokens generated: {:?}", transcription.tokens);

    // Verify performance
    assert!(
        transcription.processing_time_ms < 10000,
        "Transcription too slow: {} ms",
        transcription.processing_time_ms
    );

    // Should be faster than real-time
    let real_time_factor = duration_secs * 1000.0 / transcription.processing_time_ms as f32;
    assert!(
        real_time_factor > 1.0,
        "Not faster than real-time: {:.1}x",
        real_time_factor
    );
}
