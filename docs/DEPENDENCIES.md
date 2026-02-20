# Technology Stack & Dependencies

## Core Language

- **Rust** (Edition 2021, Stable Channel)
  - Minimum version: 1.75.0
  - Target platforms: x86_64 macOS, Linux, Windows (future)

---

## Primary Dependencies

### 🎙️ Audio Processing

```toml
[dependencies]
# Cross-platform audio I/O
cpal = "0.15"

# Audio encoding/decoding
hound = "3.5"

# Sample rate conversion
rubato = "0.14"

# Ring buffer for real-time audio
ringbuf = "0.3"
```

**Rationale**:
- `cpal`: Industry-standard for low-latency audio capture
- `hound`: Lightweight WAV support for testing/debugging
- `rubato`: High-quality resampling to 16kHz
- `ringbuf`: Lock-free for zero-copy audio pipeline

---

### 🧠 Voice Activity Detection (VAD)

```toml
[dependencies]
# Silero VAD (ONNX-based)
ort = "2.0"  # ONNX Runtime bindings

# Alternative: WebRTC VAD
webrtc-vad = "0.4"
```

**Rationale**:
- Silero VAD: State-of-the-art accuracy, runs on ONNX
- WebRTC VAD: Lightweight fallback, CPU-only

---

### 🤖 Model Inference Backends

#### whisper.cpp (Primary)
```toml
[dependencies]
whisper-rs = "0.12"  # Rust bindings for whisper.cpp
```
- **Pros**: Fastest, quantized GGML models, Metal/CUDA support
- **Cons**: C++ dependency, requires build tools

#### Faster-Whisper (Python Bridge)
```toml
[dependencies]
pyo3 = "0.21"  # Python interop
pyo3-asyncio = "0.21"
```
- **Pros**: Best accuracy, CTranslate2 optimizations
- **Cons**: Python runtime required, heavier

#### ONNX Runtime
```toml
[dependencies]
ort = "2.0"  # ONNX Runtime
```
- **Pros**: Cross-platform, quantization support
- **Cons**: Model conversion needed

#### Candle (Rust-Native ML)
```toml
[dependencies]
candle-core = "0.3"
candle-nn = "0.3"
candle-transformers = "0.3"
```
- **Pros**: Pure Rust, no external deps
- **Cons**: Slower than whisper.cpp, less mature

---

### ⌨️ Platform Integration

#### Global Hotkey
```toml
[dependencies]
global-hotkey = "0.5"  # Cross-platform hotkey handling
```

#### Text Injection

**macOS**:
```toml
[target.'cfg(target_os = "macos")'.dependencies]
core-graphics = "0.23"
core-foundation = "0.9"
accessibility-sys = "0.1"  # macOS Accessibility APIs
```

**Linux**:
```toml
[target.'cfg(target_os = "linux")'.dependencies]
x11 = "2.21"
x11-clipboard = "0.8"
evdev = "0.12"  # Keyboard input handling
```

**Windows** (future):
```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.52", features = ["Win32_UI_Input_KeyboardAndMouse"] }
```

---

### 🔧 Daemon & IPC

```toml
[dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
tokio-util = "0.7"

# Unix sockets / Named pipes
interprocess = "1.2"

# Serialization
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"  # Binary protocol
```

**Rationale**:
- `tokio`: De facto async runtime, required by many deps
- `interprocess`: Abstracts Unix sockets and Windows pipes
- `bincode`: Fast binary serialization for IPC

---

### 🖥️ Terminal UI (TUI)

```toml
[dependencies]
# TUI framework
ratatui = "0.25"

# Terminal backend
crossterm = "0.27"

# Async TUI input handling
crossterm-tokio = "0.1"
```

**Rationale**:
- `ratatui`: Modern, actively maintained TUI framework
- `crossterm`: Cross-platform terminal control

---

### 📝 Configuration & Logging

```toml
[dependencies]
# Configuration
toml = "0.8"
serde = { version = "1.0", features = ["derive"] }
dirs = "5.0"  # Platform-specific directories

# File watching for hot reload
notify = "6.1"

# Logging and tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
tracing-appender = "0.2"  # Log rotation
```

---

### 🛠️ Utilities

```toml
[dependencies]
# Error handling
anyhow = "1.0"  # For application errors
thiserror = "1.0"  # For library errors

# CLI argument parsing
clap = { version = "4.4", features = ["derive"] }

# Async utilities
futures = "0.3"
async-trait = "0.1"

# Concurrency
crossbeam = "0.8"  # Lock-free data structures
parking_lot = "0.12"  # Faster mutexes

# Performance monitoring
sysinfo = "0.30"  # System metrics
```

---

### 🧪 Development Dependencies

```toml
[dev-dependencies]
# Testing
tokio-test = "0.4"
mockall = "0.12"  # Mocking framework
proptest = "1.4"  # Property-based testing

# Benchmarking
criterion = { version = "0.5", features = ["html_reports"] }

# Test utilities
tempfile = "3.8"
assert_cmd = "2.0"
predicates = "3.0"
```

---

## Build Dependencies

### System Requirements

**macOS**:
```bash
# Xcode Command Line Tools
xcode-select --install

# Homebrew packages
brew install cmake pkg-config portaudio
```

**Linux (Ubuntu/Debian)**:
```bash
sudo apt-get install \
    build-essential \
    cmake \
    pkg-config \
    libasound2-dev \
    libx11-dev \
    libxtst-dev \
    libxss-dev \
    portaudio19-dev
```

**Windows**:
- Visual Studio 2022 Build Tools
- CMake
- vcpkg for dependencies

---

## Cargo Features

```toml
[features]
default = ["whisper-cpp", "silero-vad", "tui"]

# Model backends
whisper-cpp = ["whisper-rs"]
faster-whisper = ["pyo3", "pyo3-asyncio"]
onnx = ["ort"]
candle = ["candle-core", "candle-nn", "candle-transformers"]

# VAD backends
silero-vad = ["ort"]
webrtc-vad = ["webrtc-vad"]

# Features
tui = ["ratatui", "crossterm"]
gpu = []  # Enable GPU acceleration
static = []  # Static linking

# Development
profiling = ["tracing-flame"]
```

---

## Build Configuration

### Optimized Release Profile

```toml
[profile.release]
opt-level = 3
lto = "fat"  # Full link-time optimization
codegen-units = 1
strip = true  # Remove debug symbols
panic = "abort"  # Smaller binary
```

### Development Profile

```toml
[profile.dev]
opt-level = 0
debug = true
```

### Benchmark Profile

```toml
[profile.bench]
inherits = "release"
debug = true  # Keep symbols for profiling
```

---

## External Model Dependencies

### Whisper Models (GGML Format)

Download from [ggerganov/whisper.cpp](https://github.com/ggerganov/whisper.cpp/tree/master/models):

```bash
# Tiny model (75 MB) - Fastest
curl -L https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin \
  -o ~/.onevox/models/ggml-tiny.en.bin

# Base model (142 MB) - Balanced
curl -L https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin \
  -o ~/.onevox/models/ggml-base.en.bin

# Small model (466 MB) - Better accuracy
curl -L https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin \
  -o ~/.onevox/models/ggml-small.en.bin
```

### Silero VAD Model (ONNX)

```bash
curl -L https://github.com/snakers4/silero-vad/raw/master/files/silero_vad.onnx \
  -o ~/.onevox/models/silero_vad.onnx
```

---

## CI/CD Dependencies

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        rust: [stable]
    
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Install system dependencies (Linux)
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libasound2-dev libx11-dev portaudio19-dev
      
      - name: Install system dependencies (macOS)
        if: runner.os == 'macOS'
        run: brew install cmake portaudio
      
      - name: Run tests
        run: cargo test --all-features
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
      
      - name: Check formatting
        run: cargo fmt -- --check
```

---

## Dependency Update Policy

1. **Security patches**: Apply immediately
2. **Minor updates**: Monthly review
3. **Major updates**: Quarterly evaluation
4. **Lock file**: Commit to ensure reproducible builds

### Tools

```bash
# Check for outdated dependencies
cargo outdated

# Security audit
cargo audit

# Unused dependencies
cargo udeps
```

---

## License Compatibility

All dependencies must be compatible with **MIT** or **Apache-2.0** licenses:

- ✅ MIT
- ✅ Apache-2.0
- ✅ BSD-3-Clause
- ❌ GPL (avoid, copyleft)

---

**Last Updated**: February 20, 2026  
**Version**: 1.0
