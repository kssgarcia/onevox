use criterion::{Criterion, black_box, criterion_group, criterion_main};
use onevox::models::{ModelConfig, ModelRuntime, WhisperCpp};

fn bench_transcription_cpu(c: &mut Criterion) {
    // Skip if model not available
    let model_path = "ggml-base.en";

    let mut model = match WhisperCpp::new() {
        Ok(m) => m,
        Err(_) => return, // Skip benchmark if whisper-cpp not available
    };

    let config = ModelConfig {
        model_path: model_path.to_string(),
        use_gpu: false,
        n_threads: 4,
        ..Default::default()
    };

    if model.load(config).is_err() {
        eprintln!(
            "⚠️ Skipping CPU benchmark - model '{}' not found",
            model_path
        );
        eprintln!("💡 Download with: onevox models download {}", model_path);
        return;
    }

    // 3 seconds of test audio (silence for consistent timing)
    let audio = vec![0.0f32; 16000 * 3];

    c.bench_function("whisper_transcribe_cpu_3s", |b| {
        b.iter(|| {
            let _ = model.transcribe(black_box(&audio), 16000);
        });
    });
}

fn bench_transcription_gpu(c: &mut Criterion) {
    // Skip if model not available
    let model_path = "ggml-base.en";

    let mut model = match WhisperCpp::new() {
        Ok(m) => m,
        Err(_) => return, // Skip benchmark if whisper-cpp not available
    };

    let config = ModelConfig {
        model_path: model_path.to_string(),
        use_gpu: true,
        n_threads: 4,
        ..Default::default()
    };

    if model.load(config).is_err() {
        eprintln!(
            "⚠️ Skipping GPU benchmark - model '{}' not found or GPU unavailable",
            model_path
        );
        eprintln!("💡 Download with: onevox models download {}", model_path);
        eprintln!("💡 Enable GPU with: cargo build --release --features metal");
        return;
    }

    // 3 seconds of test audio (silence for consistent timing)
    let audio = vec![0.0f32; 16000 * 3];

    c.bench_function("whisper_transcribe_gpu_3s", |b| {
        b.iter(|| {
            let _ = model.transcribe(black_box(&audio), 16000);
        });
    });
}

fn bench_model_loading(c: &mut Criterion) {
    let model_path = "ggml-base.en";

    c.bench_function("whisper_model_load", |b| {
        b.iter(|| {
            let mut model = WhisperCpp::new().unwrap();
            let config = ModelConfig {
                model_path: model_path.to_string(),
                use_gpu: true,
                ..Default::default()
            };
            let _ = model.load(config);
        });
    });
}

criterion_group!(
    name = gpu_benches;
    config = Criterion::default().sample_size(10); // Fewer samples for slower operations
    targets = bench_transcription_cpu, bench_transcription_gpu, bench_model_loading
);
criterion_main!(gpu_benches);
