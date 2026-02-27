# Development

## Quick Start

```bash
git clone https://github.com/kssgarcia/onevox.git
cd onevox
cargo build --release
```

## Build

```bash
# Debug build (includes ONNX by default)
cargo build

# Release build (includes ONNX by default)
cargo build --release

# Minimal build (whisper.cpp only, no ONNX)
cargo build --release --no-default-features --features whisper-cpp
```

### ONNX Runtime Support

ONNX support is **included by default** in all builds. This enables the Parakeet model and other ONNX models.

**What's included:**
- Downloads ONNX Runtime binaries automatically (~150MB) via `ort-sys`
- Builds ONNX inference backend
- Enables ONNX model support (Parakeet, etc.)
- Increases binary size by ~30MB

**Testing ONNX:**
```bash
# ONNX is available by default
./target/release/onevox config init
# Edit config.toml: model_path = "parakeet-ctc-0.6b"
./target/release/onevox daemon --foreground
```

**Why?** whisper.cpp compiles from source and needs proper SDK paths.

**Subsequent Builds:**

After first build, no env vars needed:

```bash
cargo build --release
```

**When you need env vars again:**
- After `cargo clean`
- When updating whisper-rs
- On fresh machine/CI

### ONNX Backend (Experimental)

To build with ONNX Runtime support for multilingual models:

```bash
# macOS (first build)
CC=clang CXX=clang++ SDKROOT=$(xcrun --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=13.0 \
  cargo build --release --features onnx

# Linux/Windows or subsequent builds
cargo build --release --features onnx
```

**What happens:**
- Downloads ONNX Runtime binaries (~150MB) automatically via `ort-sys`
- Builds ONNX inference backend
- Enables ONNX model support (Parakeet, etc.)
- Increases binary size by ~30MB

**Testing ONNX:**
```bash
# Run with ONNX model
./target/release/onevox config init
# Edit config.toml: model_path = "parakeet-ctc-0.6b"
./target/release/onevox daemon --foreground
```

### Build Script

```bash
./build.sh          # Debug
./build.sh release  # Release
```

### GPU Acceleration

OneVox supports GPU acceleration for whisper.cpp backend:

```bash
# macOS - Metal (default on Apple Silicon)
cargo build --release

# NVIDIA - CUDA
cargo build --release --features cuda

# Cross-platform - Vulkan
cargo build --release --features vulkan

# CPU optimization - OpenBLAS
cargo build --release --features openblas
```

**Note**: GPU features only apply to `whisper.cpp` backend. ONNX backend uses CPU-optimized INT8 models.

## Run

```bash
# Foreground (for development)
./target/release/onevox daemon --foreground

# With debug logging
RUST_LOG=debug ./target/release/onevox daemon --foreground

# Install locally for testing
# macOS: builds and installs from source
./install.sh

# Linux: builds and installs from source  
./scripts/install_linux.sh
```

## Test

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# Benchmarks
cargo bench

# Platform tests
./target/release/onevox test-hotkey
./target/release/onevox test-audio --duration 3
./target/release/onevox test-vad --duration 10
```

## Project Structure

```
src/
├── main.rs              # CLI entry
├── lib.rs               # Library exports
├── daemon/              # Daemon lifecycle
│   ├── lifecycle.rs     # Start/stop/signals
│   ├── dictation.rs     # Hotkey → Audio → Model → Inject
│   └── state.rs         # Shared state
├── audio/               # Audio capture (cpal)
├── vad/                 # Voice Activity Detection
├── models/              # Transcription models
│   ├── whisper_cpp.rs   # whisper.cpp backend (default)
│   ├── onnx_runtime.rs  # ONNX Runtime backend (default)
│   ├── whisper_candle.rs # Pure Rust backend (experimental)
│   └── runtime.rs       # ModelRuntime trait
├── platform/            # Platform-specific
│   ├── hotkey.rs        # Global hotkey
│   ├── injector.rs      # Text injection
│   ├── paths.rs         # Cross-platform paths
│   └── permissions.rs   # Permission checks
├── ipc/                 # Inter-process communication
├── history.rs           # Transcription history
├── config.rs            # Configuration
└── tui.rs               # Terminal UI

tui/                     # TypeScript TUI (Bun)
scripts/                 # Installation and packaging scripts
```

## Dependencies

**Core:**
- `whisper-rs` - Native whisper.cpp bindings (default backend)
- `ort` + `ort-sys` - ONNX Runtime bindings (default, included in all builds)
- `handy-keys` - Global hotkey detection
- `cpal` - Cross-platform audio
- `enigo` - Text injection
- `tokio` - Async runtime

**Platform:**
- macOS: `core-graphics`, `core-foundation`
- Linux: `x11`, `evdev`

**Build Features:**
```toml
default = ["whisper-cpp", "overlay-indicator"]
whisper-cpp = ["whisper-rs"]        # Native whisper.cpp (recommended)
onnx = ["ort", "ort-sys", "ndarray"] # ONNX Runtime (multilingual)
candle = [...]                       # Pure Rust (experimental)
tui = ["ratatui", "crossterm"]      # Terminal UI
```

## Platform Setup

**macOS:**
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install build-essential pkg-config libasound2-dev libpulse-dev
```

**Linux (Fedora):**
```bash
sudo dnf install gcc pkg-config alsa-lib-devel pulseaudio-libs-devel
```

**Linux (Arch):**
```bash
sudo pacman -S base-devel alsa-lib pulseaudio
```

**Windows:**
- Visual Studio Build Tools
- Rust from rustup.rs

## Release

### Build Release

```bash
# macOS - Default (whisper.cpp)
CC=clang CXX=clang++ SDKROOT=$(xcrun --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=13.0 \
  cargo build --release --locked

# macOS - With ONNX
CC=clang CXX=clang++ SDKROOT=$(xcrun --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=13.0 \
  cargo build --release --locked --features onnx

# Linux - Default
cargo build --release --locked
strip target/release/onevox

# Linux - With ONNX
cargo build --release --locked --features onnx
strip target/release/onevox

# Windows - Default
cargo build --release --locked

# Windows - With ONNX
cargo build --release --locked --features onnx
```

### Package

**macOS:**
```bash
./scripts/package_macos_app.sh
# Creates: dist/Onevox.app

# Create DMG
hdiutil create -volname "OneVox" -srcfolder dist/Onevox.app \
  -ov -format UDZO dist/Onevox-macos-$(uname -m).dmg
```

**Linux:**
```bash
mkdir -p dist/onevox-linux-x86_64
cp target/release/onevox dist/onevox-linux-x86_64/
cp scripts/*.sh README.md config.example.toml dist/onevox-linux-x86_64/
cd dist && tar -czf onevox-linux-x86_64.tar.gz onevox-linux-x86_64
```

**Windows:**
```powershell
# Create ZIP
Compress-Archive -Path target/release/onevox.exe,README.md,config.example.toml `
  -DestinationPath dist/onevox-windows-x64.zip
```

### Release Script

```bash
./scripts/release.sh v0.2.0
```

## CI/CD

GitHub Actions workflow in `.github/workflows/release.yml`:

```yaml
name: Release
on:
  push:
    tags: ['v*']

jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: |
          CC=clang CXX=clang++ SDKROOT=$(xcrun --show-sdk-path) \
          MACOSX_DEPLOYMENT_TARGET=13.0 cargo build --release --locked
      - name: Package
        run: ./scripts/package_macos_app.sh

  build-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install deps
        run: sudo apt install build-essential libasound2-dev libpulse-dev
      - name: Build
        run: cargo build --release --locked

  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build --release --locked
```

## Code Style

```bash
# Format
cargo fmt

# Lint
cargo clippy

# Check
cargo check
```

## Logging

```bash
# Debug mode
RUST_LOG=debug onevox daemon --foreground

# Trace mode
RUST_LOG=trace onevox daemon --foreground

# Module-specific
RUST_LOG=onevox::models=debug onevox daemon --foreground
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Before submitting:**
1. Run `cargo fmt`
2. Run `cargo clippy`
3. Run `cargo test`
4. Test on your platform
5. Update documentation
