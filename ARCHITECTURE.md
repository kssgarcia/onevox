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
    model_path: "ggml-base.en".to_string(),
    language: "en".to_string(),
    use_gpu: true,
    ..Default::default()
};
model.load(config)?;

let transcription = model.transcribe(&audio_samples, 16000)?;
```

## Backend: ONNX Runtime

**Why ONNX Runtime?**
- Cross-platform performance (15-25x real-time on CPU with Parakeet)
- Multilingual support (25+ languages with models like Parakeet)
- Production-ready inference engine with static linking
- CTC-based models for fast streaming transcription
- INT8 quantization support for reduced memory footprint (250MB for Parakeet 0.6B)
- Automatic platform-specific binary downloads via ort-sys

**Implementation:** `src/models/onnx_runtime.rs` (571 lines)

**Key Features:**
- Mel spectrogram feature extraction (128 mel bins, 25ms window, 10ms hop)
- Greedy CTC decoding with blank token removal
- SentencePiece tokenization (8193 token vocabulary)
- Cross-platform model path resolution
- Comprehensive error handling and logging

```rust
use onevox::models::{OnnxRuntime, ModelRuntime, ModelConfig};

let mut model = OnnxRuntime::new()?;
let config = ModelConfig {
    model_path: "parakeet-ctc-0.6b".to_string(),
    language: "en".to_string(),
    use_gpu: false, // CPU-optimized with INT8
    ..Default::default()
};
model.load(config)?;

let transcription = model.transcribe(&audio_samples, 16000)?;
```

**Model Requirements:**
- Input: Mel spectrogram features [batch=1, features=128, time_frames]
- Output: CTC logits [batch=1, time_steps, vocab_size]
- Sample rate: 16kHz
- Model files: encoder-model.int8.onnx (621MB), vocab.txt (8KB)

**Supported Models:**
- NVIDIA Parakeet TDT 0.6B v3 (multilingual, INT8 quantized, 250MB runtime)
- Any ONNX-exported CTC-based ASR model with compatible input/output shapes

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
# Backend selection
backend = "whisper_cpp"  # or "onnx_runtime" (requires --features onnx)

# Model identifier (without extension)
model_path = "ggml-base.en"  # or "parakeet-ctc-0.6b"

# Device selection
device = "auto"  # auto, cpu, gpu

# Language (ISO 639-1 code)
language = "en"

# Preload model at daemon startup
preload = true
```

## Build Features

```toml
[features]
default = ["whisper-cpp"]

# Model backends
whisper-cpp = ["whisper-rs"]
onnx = ["ort", "ndarray"]
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
- Minimal runtime dependencies
- Long-term maintainability
- Production-ready performance

**What we avoid:**
- Experimental or unstable backends
- Complex dependency chains
- Unpredictable behavior
- Heavy runtime requirements

## Future Considerations

**Potential additions:**
- Candle backend (pure Rust)
- Newer Whisper model variants
- Additional ONNX-optimized models
- Model quantization support
- Preprocessing optimizations

**Not planned:**
- Multiple simultaneous backends
- Dynamic plugin architecture
- Experimental ML frameworks
