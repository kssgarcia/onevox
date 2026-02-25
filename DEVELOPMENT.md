# Development

## Quick Start

```bash
git clone https://github.com/kssgarcia/onevox.git
cd onevox
cargo build --release
```

## Build

### First Build (macOS only)

macOS requires environment variables on first build:

```bash
CC=clang CXX=clang++ SDKROOT=$(xcrun --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=13.0 \
  cargo build --release
```

**Why?** whisper.cpp compiles from source and needs proper SDK paths.

### Subsequent Builds

After first build, no env vars needed:

```bash
cargo build --release
```

**When you need env vars again:**
- After `cargo clean`
- When updating whisper-rs
- On fresh machine/CI

### Build Script

```bash
./build.sh          # Debug
./build.sh release  # Release
```

### GPU Acceleration

```bash
cargo build --release --features metal      # macOS
cargo build --release --features cuda       # NVIDIA
cargo build --release --features vulkan     # Cross-platform
cargo build --release --features openblas   # CPU optimization
```

## Run

```bash
# Foreground
./target/release/onevox daemon --foreground

# Install locally
./scripts/install_macos.sh   # macOS
./scripts/install_linux.sh   # Linux
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
├── models/              # Whisper models
│   ├── whisper_cpp.rs   # Native whisper.cpp (PRIMARY)
│   ├── whisper_candle.rs # Pure Rust (experimental)
│   └── runtime.rs       # Model trait
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
```

## Dependencies

**Core:**
- `whisper-rs` - Native whisper.cpp bindings
- `handy-keys` - Global hotkey detection
- `cpal` - Cross-platform audio
- `enigo` - Text injection
- `tokio` - Async runtime

**Platform:**
- macOS: `core-graphics`, `core-foundation`
- Linux: `x11`, `evdev`

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
# macOS
CC=clang CXX=clang++ SDKROOT=$(xcrun --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=13.0 \
  cargo build --release --locked

# Linux
cargo build --release --locked
strip target/release/onevox

# Windows
cargo build --release --locked
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
