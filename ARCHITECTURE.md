# Architecture

## Overview

OneVox uses native whisper.cpp bindings for maximum stability and performance.

**Key Benefits:**
- 2-4x faster transcription (50-200ms vs 150-400ms)
- 30-50% less memory usage
- Single self-contained binary
- No subprocess or IPC overhead
- No Python or ONNX Runtime dependencies

## Pipeline

```
Hotkey Press
    ↓
Audio Capture (cpal)
    ↓
VAD Processing
    ↓
Speech Detection
    ↓
Transcription (whisper.cpp)
    ↓
Text Injection
```

## Backend: whisper.cpp

**Why whisper.cpp?**
- Proven cross-platform stability
- Direct library calls (no subprocess overhead)
- No external runtime dependencies
- Optimized for speech transcription
- Deterministic performance
- GPU acceleration (Metal, CUDA, Vulkan, OpenBLAS)

**Implementation:** `src/models/whisper_cpp.rs`

```rust
use onevox::models::{WhisperCpp, ModelRuntime, ModelConfig};

let mut model = WhisperCpp::new()?;
let config = ModelConfig {
    model_path: "ggml-base.en.bin".to_string(),
    language: "en".to_string(),
    use_gpu: true,
    ..Default::default()
};
model.load(config)?;

let transcription = model.transcribe(&audio_samples, 16000)?;
```

## Model Runtime Trait

All backends implement `ModelRuntime`:

```rust
pub trait ModelRuntime: Send + Sync {
    fn load(&mut self, config: ModelConfig) -> Result<()>;
    fn is_loaded(&self) -> bool;
    fn transcribe(&mut self, samples: &[f32], sample_rate: u32) -> Result<Transcription>;
    fn unload(&mut self);
    fn name(&self) -> &str;
}
```

## Performance

**Model Loading:**
- Once at daemon startup
- 100-500ms (depending on model size)
- Memory: 100MB-1GB

**Transcription:**
- Latency: 50-200ms for 2-3 seconds of audio
- Real-time factor: 0.1-0.3x
- Minimal allocation during inference

**Audio Capture:**
- Sample rate: 16kHz
- Chunk size: 200ms
- Buffer: 2 seconds capacity

## Cross-Platform

**Linux:**
- Native whisper.cpp compilation
- X11/Wayland text injection
- ALSA/PulseAudio audio

**macOS:**
- Metal acceleration
- CoreAudio
- Accessibility API

**Windows:**
- CUDA/Vulkan acceleration
- WASAPI audio
- Windows API text injection

## Configuration

```toml
[model]
backend = "whisper_cpp"
model_path = "ggml-base.en.bin"
device = "auto"  # auto, cpu, gpu
language = "en"
preload = true
```

## Build Features

```toml
[features]
default = ["whisper-cpp"]

# Model backends
whisper-cpp = ["whisper-rs"]
candle = ["candle-core", "candle-nn", "candle-transformers"]

# GPU acceleration
metal = ["whisper-rs/metal"]
cuda = ["whisper-rs/cuda"]
vulkan = ["whisper-rs/vulkan"]
openblas = ["whisper-rs/openblas"]
```

## Design Principles

**What we prioritize:**
- Cross-platform stability
- Fast, low-latency transcription
- Deterministic behavior
- Easy distribution
- Minimal dependencies
- Long-term maintainability

**What we avoid:**
- General-purpose ML hosting
- Dynamic plugin systems
- Experimental backend explosion
- Python or ONNX Runtime dependencies

## Future Considerations

**Potential additions:**
- Candle backend (pure Rust)
- Newer Whisper model variants
- Model quantization support
- Preprocessing optimizations

**Not planned:**
- Multiple backend support
- External runtime dependencies
- Complex plugin architecture
