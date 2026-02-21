use criterion::{Criterion, black_box, criterion_group, criterion_main};
use onevox::audio::buffer::AudioBuffer;

fn bench_audio_ring_buffer_write(c: &mut Criterion) {
    c.bench_function("audio_ring_buffer_write_100ms", |b| {
        let mut producer = AudioBuffer::new(16_000 * 2).split().0;
        let chunk = vec![0.1_f32; 1_600];
        b.iter(|| {
            producer.push(black_box(&chunk));
        });
    });
}

fn bench_audio_ring_buffer_read(c: &mut Criterion) {
    c.bench_function("audio_ring_buffer_read_100ms", |b| {
        let (mut producer, mut consumer) = AudioBuffer::new(16_000 * 2).split();
        let chunk = vec![0.2_f32; 1_600];
        let mut out = vec![0.0_f32; 1_600];

        b.iter(|| {
            producer.push(&chunk);
            consumer.pop(black_box(&mut out));
        });
    });
}

criterion_group!(
    audio_processing_benches,
    bench_audio_ring_buffer_write,
    bench_audio_ring_buffer_read
);
criterion_main!(audio_processing_benches);
