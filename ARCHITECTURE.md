# Architecture

## Overview

OneVox uses a model-centric architecture where the backend is automatically selected based on your model choice:

1. **whisper.cpp** (Default, Recommended) - Native C++ bindings for GGML models
2. **ONNX Runtime** (Experimental) - Alternative runtime for ONNX models

**Comparison:**

| Feature | whisper.cpp | ONNX Runtime |
|---------|-------------|--------------|
| **Build** | Default | Default (included) |
| **Selection** | Auto (GGML models) | Auto (ONNX/Parakeet models) |
| **Stability** | Production-ready | Experimental |
| **Speed** | 50-200ms | Varies by model |
| **Memory** | ~100MB | ~250MB |
| **Languages** | English-only or multilingual (99+) | Model-dependent |
| **Models** | Whisper GGML (tiny to large-v3) | Parakeet, custom ONNX |
| **GPU** | Metal, CUDA, Vulkan | CPU-optimized INT8 |
| **Binary Size** | ~20MB | ~50MB |

**Key Benefits of whisper.cpp:**
- 2-4x faster transcription (50-200ms vs 150-400ms)
- 30-50% less memory usage
- Single self-contained binary
- No subprocess or IPC overhead
- No Python or external runtime dependencies
- Cross-platform GPU acceleration
- Automatic language detection for multilingual models

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

## Backend: whisper.cpp (Default)

**When to use:**
- Production deployments
- English or multilingual transcription
- Need for GPU acceleration
- Minimum resource usage
- Maximum stability

**Why whisper.cpp?**
- Proven cross-platform stability
- Direct library calls (no subprocess overhead)
- No external runtime dependencies
- Optimized for speech transcription
- Deterministic performance
- GPU acceleration (Metal, CUDA, Vulkan, OpenBLAS)
- Supports both English-only and multilingual models (99+ languages)

**Build:**
```bash
cargo build --release  # Default, no flags needed
```

**Implementation:** `src/models/whisper_cpp.rs`

```rust
use onevox::models::{WhisperCpp, ModelRuntime, ModelConfig};

let mut model = WhisperCpp::new()?;
let config = ModelConfig {
    model_path: "ggml-base.en".to_string(),  // or "ggml-base" for multilingual
    use_gpu: true,
    ..Default::default()
};
model.load(config)?;

let transcription = model.transcribe(&audio_samples, 16000)?;
// Language auto-detected for multilingual models
```

## Backend: ONNX Runtime (Experimental)

**When to use:**
- Multilingual transcription (25+ languages)
- Research and experimentation
- Need for CTC-based models
- CPU-only deployments with INT8 optimization

**Why ONNX Runtime?**
- Cross-platform performance (15-25x real-time on CPU with Parakeet)
- Multilingual support (25+ languages with models like Parakeet)
- Production-ready inference engine with static linking
- CTC-based models for fast streaming transcription
- INT8 quantization support for reduced memory footprint (250MB for Parakeet 0.6B)
- Automatic platform-specific binary downloads via ort-sys

**Build:**
```bash
cargo build --release  # ONNX support included by default
```

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
# Backend auto-detected from model_path
# - GGML models (ggml-*) use whisper.cpp
# - Parakeet/ONNX models use ONNX Runtime (included by default)

model_path = "ggml-base.en"      # English-only (whisper.cpp)
# model_path = "ggml-base"       # Multilingual, 99+ languages (whisper.cpp)
# model_path = "parakeet-ctc-0.6b"  # ONNX model (included by default)

# Device selection
device = "auto"  # auto, cpu, gpu

# Preload model at daemon startup (recommended)
preload = true
```

**Available Models:**

*English-only (whisper.cpp):*
- `ggml-tiny.en` (75MB) - Fastest
- `ggml-base.en` (142MB) - Recommended
- `ggml-small.en` (466MB) - Better accuracy
- `ggml-medium.en` (1.5GB) - High accuracy

*Multilingual (whisper.cpp, 99+ languages):*
- `ggml-tiny` (75MB)
- `ggml-base` (142MB)
- `ggml-small` (466MB)
- `ggml-medium` (1.5GB)
- `ggml-large-v2` (2.9GB)
- `ggml-large-v3` (2.9GB)
- `ggml-large-v3-turbo` (1.6GB)

*ONNX (included by default):*
- `parakeet-ctc-0.6b` - Multilingual, INT8 quantized

**Switching models:**
1. Download model: `onevox models download <model_id>`
2. Update config: `model_path = "<model_id>"`
3. Restart daemon: `onevox daemon restart` or `systemctl --user restart onevox`

## Build Features

```toml
[features]
default = ["whisper-cpp", "onnx", "overlay-indicator"]

# Model backends
whisper-cpp = ["whisper-rs"]                # Native whisper.cpp (default)
onnx = ["ort", "ort-sys", "ndarray"]        # ONNX Runtime (default)
candle = ["candle-core", "candle-nn", "candle-transformers"]  # Pure Rust (experimental)

# GPU acceleration (whisper-cpp only)
metal = ["whisper-rs/metal"]       # macOS GPU acceleration
cuda = ["whisper-rs/cuda"]         # NVIDIA GPU acceleration
vulkan = ["whisper-rs/vulkan"]     # Cross-platform GPU
openblas = ["whisper-rs/openblas"] # CPU optimization

# Additional features
tui = ["ratatui", "crossterm"]           # Terminal UI
overlay-indicator = ["eframe", "winit"]  # Visual recording indicator
```

**Build examples:**
```bash
# Default (includes both whisper.cpp and ONNX)
cargo build --release

# Whisper.cpp only (minimal build)
cargo build --release --no-default-features --features whisper-cpp

# GPU-accelerated whisper.cpp (macOS)
cargo build --release --features metal

# GPU-accelerated with ONNX
cargo build --release --features "metal"
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
