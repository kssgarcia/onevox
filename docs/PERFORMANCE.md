# Performance Benchmarks & Optimization Guide

## Performance Goals

### Latency Targets

| Metric | Target | Stretch Goal | Critical |
|--------|--------|--------------|----------|
| Hotkey to audio start | <10ms | <5ms | Yes |
| Audio capture latency | <10ms | <5ms | Yes |
| VAD processing (per chunk) | <5ms | <2ms | Yes |
| Model inference (tiny, 1sec) | <100ms | <50ms | Yes |
| Model inference (base, 1sec) | <300ms | <200ms | No |
| Text injection | <20ms | <10ms | Yes |
| **End-to-end (1sec audio)** | **<350ms** | **<200ms** | **Yes** |

### Resource Targets

| Resource | Idle | Active | Peak |
|----------|------|--------|------|
| Memory | <500MB | <1GB | <1.5GB |
| CPU (1 core) | <5% | 50-100% | 100% |
| Disk I/O | <1MB/s | <10MB/s | <50MB/s |

---

## Benchmark Suite

### Micro-Benchmarks (Criterion)

Located in `benches/` directory.

#### Audio Processing
```rust
// benches/audio_processing.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_audio_capture(c: &mut Criterion) {
    c.bench_function("audio_ring_buffer_write", |b| {
        let mut buffer = RingBuffer::new(16000 * 2); // 1 sec
        let chunk = vec![0i16; 1600]; // 100ms
        b.iter(|| {
            buffer.write(black_box(&chunk))
        });
    });
}

fn bench_resampling(c: &mut Criterion) {
    c.bench_function("resample_48khz_to_16khz", |b| {
        let input = vec![0.0f32; 4800]; // 100ms at 48kHz
        let mut resampler = Resampler::new(48000, 16000);
        b.iter(|| {
            resampler.process(black_box(&input))
        });
    });
}
```

**Expected Results**:
- Ring buffer write: <100ns
- Resampling (100ms): <500μs

---

#### VAD Processing
```rust
// benches/vad.rs
fn bench_vad_detection(c: &mut Criterion) {
    let vad = SileroVAD::new("models/silero_vad.onnx").unwrap();
    let chunk = vec![0.0f32; 1600]; // 100ms at 16kHz
    
    c.bench_function("vad_silero_100ms", |b| {
        b.iter(|| {
            vad.detect(black_box(&chunk))
        });
    });
}
```

**Expected Results**:
- Silero VAD (100ms chunk): <2ms
- WebRTC VAD (100ms chunk): <500μs

---

#### Model Inference
```rust
// benches/model_inference.rs
fn bench_whisper_tiny(c: &mut Criterion) {
    let model = WhisperModel::load("models/ggml-tiny.en.bin").unwrap();
    let audio = load_test_audio("fixtures/1sec_speech.wav");
    
    c.bench_function("whisper_tiny_1sec", |b| {
        b.iter(|| {
            model.transcribe(black_box(&audio))
        });
    });
}

fn bench_whisper_base(c: &mut Criterion) {
    let model = WhisperModel::load("models/ggml-base.en.bin").unwrap();
    let audio = load_test_audio("fixtures/1sec_speech.wav");
    
    c.bench_function("whisper_base_1sec", |b| {
        b.iter(|| {
            model.transcribe(black_box(&audio))
        });
    });
}
```

**Expected Results (M1 Pro, Metal)**:
- Tiny model (1sec): 30-50ms
- Base model (1sec): 150-250ms
- Small model (1sec): 500-800ms

**Expected Results (x86_64, CPU-only)**:
- Tiny model (1sec): 80-120ms
- Base model (1sec): 300-500ms
- Small model (1sec): 1200-2000ms

---

### Integration Benchmarks

#### End-to-End Latency
```rust
// benches/e2e.rs
fn bench_full_pipeline(c: &mut Criterion) {
    let daemon = TestDaemon::new();
    let audio_file = "fixtures/1sec_speech.wav";
    
    c.bench_function("e2e_pipeline_1sec", |b| {
        b.iter(|| {
            let start = Instant::now();
            daemon.simulate_dictation(audio_file);
            start.elapsed()
        });
    });
}
```

**Expected Results**:
- Full pipeline (1sec, tiny model): 150-250ms
- Full pipeline (1sec, base model): 400-600ms

---

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench audio_processing

# Generate flamegraph
cargo flamegraph --bench model_inference

# Profile with perf (Linux)
perf record --call-graph dwarf cargo bench
perf report
```

---

## Profiling Tools

### 1. **CPU Profiling**

#### macOS (Instruments)
```bash
# Build with debug symbols
cargo build --release --profile bench

# Profile with Instruments
instruments -t "Time Profiler" target/release/onevox daemon

# Or use cargo-instruments
cargo install cargo-instruments
cargo instruments --release -t time
```

#### Linux (perf)
```bash
# Record profile
perf record --call-graph dwarf target/release/onevox daemon
# View report
perf report

# Or use flamegraph
cargo install flamegraph
cargo flamegraph --bin onevox -- daemon
```

---

### 2. **Memory Profiling**

#### Heaptrack (Linux)
```bash
heaptrack target/release/onevox daemon
heaptrack_gui heaptrack.onevox.*.gz
```

#### Valgrind (Linux)
```bash
valgrind --tool=massif target/release/onevox daemon
ms_print massif.out.*
```

#### macOS (Instruments)
```bash
instruments -t "Allocations" target/release/onevox daemon
```

---

### 3. **Tracing & Spans**

```rust
// Add tracing to hot paths
use tracing::{info_span, instrument};

#[instrument(skip(audio))]
async fn transcribe_chunk(audio: &AudioBuffer) -> Result<Transcript> {
    let _span = info_span!("model_inference", 
        audio_len = audio.len(),
        model = "tiny"
    ).entered();
    
    // ... inference code
}
```

**Generate trace**:
```bash
# Export to Chrome trace format
RUST_LOG=trace cargo run --release
# Open chrome://tracing
```

---

## Optimization Strategies

### 1. **Audio Pipeline Optimizations**

#### Zero-Copy Ring Buffer
```rust
use ringbuf::{HeapRb, traits::*};

pub struct AudioCapture {
    // Lock-free SPSC ring buffer
    rb: HeapRb<f32>,
}

impl AudioCapture {
    pub fn new(capacity: usize) -> Self {
        Self {
            rb: HeapRb::new(capacity),
        }
    }
    
    // Called from audio thread (real-time)
    pub fn write_samples(&mut self, samples: &[f32]) {
        // Zero-copy write, drops if full (backpressure)
        let _ = self.rb.push_slice(samples);
    }
    
    // Called from processing thread
    pub fn read_chunk(&mut self, out: &mut [f32]) -> usize {
        self.rb.pop_slice(out)
    }
}
```

**Gain**: Eliminates allocations in audio callback (5-10μs → <100ns)

---

#### SIMD Resampling
```rust
use std::simd::{f32x8, SimdFloat};

fn resample_simd(input: &[f32], ratio: f32) -> Vec<f32> {
    let lanes = 8;
    input.chunks_exact(lanes)
        .map(|chunk| {
            let vec = f32x8::from_slice(chunk);
            // SIMD operations...
        })
        .collect()
}
```

**Gain**: 3-4x faster resampling on AVX2/NEON

---

### 2. **VAD Optimizations**

#### Batch Processing
```rust
pub struct StreamingVAD {
    model: SileroVAD,
    buffer: Vec<f32>,
    chunk_size: usize,
}

impl StreamingVAD {
    pub fn process_stream(&mut self, audio: &[f32]) -> Vec<VADResult> {
        self.buffer.extend_from_slice(audio);
        
        let mut results = Vec::new();
        while self.buffer.len() >= self.chunk_size {
            let chunk = self.buffer.drain(..self.chunk_size).collect::<Vec<_>>();
            results.push(self.model.detect(&chunk));
        }
        results
    }
}
```

**Gain**: Amortize model overhead across chunks

---

### 3. **Model Inference Optimizations**

#### GPU Acceleration
```rust
// whisper.cpp with Metal/CUDA
let model = WhisperModel::load_with_config(ModelConfig {
    path: "ggml-tiny.en.bin",
    device: Device::GPU, // Metal on macOS, CUDA on Linux
    n_threads: 1,        // GPU doesn't need many threads
})?;
```

**Gain**: 3-5x faster inference on GPU vs CPU

---

#### Quantization
```bash
# Use Q5_K quantized models (5-bit weights)
curl -L https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en-q5_k.bin \
  -o ~/.onevox/models/ggml-tiny.en-q5_k.bin
```

**Gain**: 30-40% faster, minimal accuracy loss

---

#### Model Warmup
```rust
pub struct ModelPool {
    model: Arc<WhisperModel>,
}

impl ModelPool {
    pub async fn new(path: &str) -> Result<Self> {
        let model = WhisperModel::load(path)?;
        
        // Warmup: run dummy inference
        let dummy_audio = vec![0.0f32; 16000]; // 1 sec silence
        model.transcribe(&dummy_audio).await?;
        
        Ok(Self {
            model: Arc::new(model),
        })
    }
}
```

**Gain**: First inference is fast (no cold-start delay)

---

### 4. **Memory Optimizations**

#### Object Pooling
```rust
use crossbeam::channel::{bounded, Sender, Receiver};

pub struct AudioChunkPool {
    pool: Receiver<Vec<f32>>,
    return_tx: Sender<Vec<f32>>,
}

impl AudioChunkPool {
    pub fn new(size: usize, chunk_len: usize) -> Self {
        let (tx, rx) = bounded(size);
        for _ in 0..size {
            tx.send(vec![0.0; chunk_len]).unwrap();
        }
        Self {
            pool: rx,
            return_tx: tx,
        }
    }
    
    pub fn acquire(&self) -> Option<PooledChunk> {
        self.pool.try_recv().ok().map(|mut buf| {
            buf.clear();
            PooledChunk {
                buffer: buf,
                return_tx: self.return_tx.clone(),
            }
        })
    }
}

pub struct PooledChunk {
    buffer: Vec<f32>,
    return_tx: Sender<Vec<f32>>,
}

impl Drop for PooledChunk {
    fn drop(&mut self) {
        let buf = std::mem::take(&mut self.buffer);
        let _ = self.return_tx.send(buf);
    }
}
```

**Gain**: Eliminates allocations in hot path (100-200μs → 0)

---

#### Arena Allocation
```rust
use bumpalo::Bump;

pub struct TranscriptionContext {
    arena: Bump,
}

impl TranscriptionContext {
    pub fn process_batch(&mut self, chunks: &[AudioChunk]) {
        // All allocations in arena
        for chunk in chunks {
            let processed = self.arena.alloc_slice_fill_copy(chunk.len(), 0.0);
            // ... process
        }
        
        // Reset arena after batch
        self.arena.reset();
    }
}
```

**Gain**: Faster allocations, better cache locality

---

### 5. **Concurrency Optimizations**

#### Thread Pinning (Linux)
```rust
use core_affinity::{CoreId, set_for_current};

fn audio_thread() {
    // Pin to dedicated core
    set_for_current(CoreId { id: 0 });
    
    // Audio capture loop
    loop {
        // ...
    }
}
```

**Gain**: Reduced context switching, more consistent latency

---

#### Lock-Free Queues
```rust
use crossbeam::queue::ArrayQueue;

pub struct Pipeline {
    audio_queue: Arc<ArrayQueue<AudioChunk>>,
    vad_queue: Arc<ArrayQueue<AudioChunk>>,
}
```

**Gain**: No mutex contention (10-50μs → <1μs)

---

## Performance Monitoring

### Runtime Metrics Collection

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct Metrics {
    transcriptions: AtomicU64,
    total_latency_ns: AtomicU64,
}

impl Metrics {
    pub fn record_transcription(&self, latency: Duration) {
        self.transcriptions.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ns.fetch_add(
            latency.as_nanos() as u64,
            Ordering::Relaxed
        );
    }
    
    pub fn avg_latency_ms(&self) -> f64 {
        let count = self.transcriptions.load(Ordering::Relaxed);
        if count == 0 { return 0.0; }
        
        let total_ns = self.total_latency_ns.load(Ordering::Relaxed);
        (total_ns as f64 / count as f64) / 1_000_000.0
    }
}
```

---

### Continuous Benchmarking (CI)

```yaml
# .github/workflows/benchmark.yml
name: Benchmark

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Run benchmarks
        run: cargo bench --bench e2e -- --save-baseline main
      
      - name: Compare with baseline
        run: |
          cargo bench --bench e2e -- --baseline main
          
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion
```

---

## Performance Regression Tests

### Automated Checks

```rust
// tests/performance_tests.rs
#[test]
fn test_e2e_latency_regression() {
    let daemon = TestDaemon::new();
    let audio = load_test_audio("fixtures/1sec_speech.wav");
    
    let start = Instant::now();
    daemon.transcribe_sync(&audio);
    let latency = start.elapsed();
    
    // Fail if latency exceeds target
    assert!(
        latency < Duration::from_millis(350),
        "E2E latency too high: {:?}",
        latency
    );
}
```

---

## Hardware-Specific Tuning

### Apple Silicon (M1/M2)
- Use Metal for GPU acceleration
- Enable NEON SIMD
- Thread count: 4-6 (efficiency cores)

### x86_64 (Intel/AMD)
- Use AVX2/AVX512 SIMD
- Thread count: 50% of cores
- Consider CUDA for NVIDIA GPUs

### ARM (Raspberry Pi)
- Use NEON SIMD
- Lower model size (tiny only)
- Consider INT8 quantization

---

## Performance Checklist

- [ ] Profile critical paths with flamegraph
- [ ] Benchmark before/after each optimization
- [ ] Test on target hardware (not just dev machine)
- [ ] Monitor memory usage over 24+ hours
- [ ] Stress test with continuous dictation
- [ ] Validate accuracy isn't degraded
- [ ] Document all optimizations

---

**Last Updated**: February 20, 2026  
**Version**: 1.0
