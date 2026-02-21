use criterion::{Criterion, black_box, criterion_group, criterion_main};
use onevox::audio::buffer::AudioChunk;
use onevox::vad::{EnergyVad, EnergyVadConfig, VadDetector};

fn bench_vad_silence_chunk(c: &mut Criterion) {
    c.bench_function("vad_energy_silence_100ms", |b| {
        let mut vad = EnergyVad::new(EnergyVadConfig::default());
        let chunk = AudioChunk::new(vec![0.0; 1_600], 16_000);
        b.iter(|| {
            let _ = vad.detect(black_box(&chunk));
        });
    });
}

fn bench_vad_speech_chunk(c: &mut Criterion) {
    c.bench_function("vad_energy_speech_100ms", |b| {
        let mut vad = EnergyVad::new(EnergyVadConfig::default());
        let chunk = AudioChunk::new(
            (0..1_600)
                .map(|i| 0.2_f32 * (i as f32 * 0.01).sin())
                .collect(),
            16_000,
        );
        b.iter(|| {
            let _ = vad.detect(black_box(&chunk));
        });
    });
}

criterion_group!(
    vad_processing_benches,
    bench_vad_silence_chunk,
    bench_vad_speech_chunk
);
criterion_main!(vad_processing_benches);
