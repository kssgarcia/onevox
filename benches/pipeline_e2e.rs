use criterion::{Criterion, black_box, criterion_group, criterion_main};
use onevox::audio::buffer::AudioChunk;
use onevox::models::{MockModel, ModelConfig, ModelRuntime};
use onevox::vad::{EnergyVad, EnergyVadConfig, VadProcessor, VadProcessorConfig};

fn bench_pipeline_audio_to_text_mock(c: &mut Criterion) {
    c.bench_function("pipeline_audio_to_text_mock_1s", |b| {
        let mut model = MockModel::new();
        let _ = model.load(ModelConfig::default());

        let vad_config = EnergyVadConfig {
            adaptive: false,
            min_speech_chunks: 1,
            min_silence_chunks: 2,
            ..EnergyVadConfig::default()
        };
        let detector = Box::new(EnergyVad::new(vad_config));
        let mut processor = VadProcessor::new(VadProcessorConfig::default(), detector);

        let speech_chunk = AudioChunk::new(
            (0..1_600)
                .map(|i| 0.2_f32 * (i as f32 * 0.01).sin())
                .collect(),
            16_000,
        );
        let silence_chunk = AudioChunk::new(vec![0.0; 1_600], 16_000);

        b.iter(|| {
            processor.reset();

            for _ in 0..10 {
                let _ = processor.process(speech_chunk.clone());
            }

            let mut maybe_segment = None;
            for _ in 0..4 {
                if let Ok(seg) = processor.process(silence_chunk.clone()) {
                    if seg.is_some() {
                        maybe_segment = seg;
                        break;
                    }
                }
            }

            if let Some(segment) = maybe_segment {
                let samples = segment.get_samples();
                let _ = model.transcribe(black_box(&samples), 16_000);
            }
        });
    });
}

criterion_group!(pipeline_e2e_benches, bench_pipeline_audio_to_text_mock);
criterion_main!(pipeline_e2e_benches);
