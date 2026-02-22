# Development

## Build

### First Build (Required Environment Variables)

The first build requires environment variables to compile whisper.cpp from source:

```bash
# macOS - First build only
CC=clang CXX=clang++ SDKROOT=$(xcrun --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=13.0 \
  cargo build --release

# Linux - No special env vars needed
cargo build --release

# Windows - No special env vars needed
cargo build --release
```

**Why macOS needs env vars on first build:**
- whisper-rs-sys compiles whisper.cpp C/C++ code from source
- Needs clang compiler (not GCC if you have Nix/Homebrew)
- Needs macOS SDK path for proper linking
- Needs to link against Accelerate framework for BLAS

**Note:** `whisper-cpp` is enabled by default, so no `--features` flag needed!

### Subsequent Builds (No Env Vars Needed)

After the first successful build, you can build normally:

```bash
# All platforms - after first build
cargo build --release
```

The build configuration files (`.cargo/config.toml` and `build.rs`) handle the linking automatically.

**When you need env vars again:**
- After `cargo clean`
- When updating whisper-rs version
- When changing features (e.g., adding `metal`)
- On a fresh machine/CI environment

Binary: `./target/release/onevox` (macOS/Linux) or `.\target\release\onevox.exe` (Windows)

### Build Script (Recommended)

```bash
# Debug build
./build.sh

# Release build
./build.sh release

# With additional features (e.g., Metal GPU acceleration)
./build.sh release "metal"
```

### macOS Build Notes

The project uses native whisper.cpp bindings which require specific build configuration on macOS:

**Required environment variables:**
- `CC=clang CXX=clang++` - Use system clang (not GCC)
- `SDKROOT=$(xcrun --show-sdk-path)` - macOS SDK path
- `MACOSX_DEPLOYMENT_TARGET=13.0` - Minimum macOS version

**Why?** The build links against the Accelerate framework for BLAS operations and requires proper symbol resolution.

**Troubleshooting:**

*Error: `library not found for -liconv`*
```bash
export SDKROOT=$(xcrun --show-sdk-path)
```

*Error: `_cblas_sgemm$NEWLAPACK$ILP64` symbol not found*
- The `build.rs` script handles this automatically
- Make sure you're using clang: `CC=clang CXX=clang++`

*Error: GCC compilation errors with macOS headers*
```bash
# Force clang if you have GCC from Nix/Homebrew
CC=clang CXX=clang++ cargo build
```

**GPU Acceleration (Metal):**
```bash
# Requires macOS 13.0+
cargo build --release --features metal
```

## Run

```bash
# Run daemon in foreground
./target/release/onevox daemon --foreground

# Or install locally
# macOS
./scripts/install_macos.sh

# Linux
./scripts/install_linux.sh

# Windows
# Run as administrator
cargo build --release
# Then run manually or set up as Windows Service
```

## Test

```bash
# Test hotkey (use platform-specific key combo)
# macOS
./target/release/onevox test-hotkey --hotkey "Cmd+Shift+0"
# Linux/Windows
./target/release/onevox test-hotkey --hotkey "Ctrl+Shift+Space"

# Test audio
./target/release/onevox test-audio --duration 3

# Test VAD
./target/release/onevox test-vad --duration 10

# Test full pipeline
./target/release/onevox test-transcribe --duration 10
```

## Package

### macOS
```bash
# Create app bundle
./scripts/package_macos_app.sh

# Output: dist/Onevox.app
```

### Linux
```bash
# Build release binary
cargo build --release

# Create tarball
tar -czf onevox-linux-x64.tar.gz -C target/release onevox scripts/install_linux.sh scripts/uninstall_linux.sh
```

### Windows
```bash
# Build release binary
cargo build --release

# Create zip
# (Use 7-Zip or PowerShell Compress-Archive)
```

## Release

### macOS Release

```bash
# Build release (first build needs env vars)
CC=clang CXX=clang++ SDKROOT=$(xcrun --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=13.0 \
  cargo build --release --locked

# Package app bundle
./scripts/package_macos_app.sh

# Create DMG (recommended for distribution)
hdiutil create -volname "OneVox" -srcfolder dist/Onevox.app -ov -format UDZO dist/Onevox-macos-$(uname -m).dmg

# Or create tarball
cd dist
tar -czf onevox-macos-$(uname -m).tar.gz Onevox.app
cd ..

# Result:
# - dist/Onevox-macos-arm64.dmg (Apple Silicon)
# - dist/Onevox-macos-x86_64.dmg (Intel)
```

### Linux Release

```bash
# Build release
cargo build --release --locked

# Strip binary to reduce size
strip target/release/onevox

# Create tarball with scripts
mkdir -p dist/onevox-linux-x86_64
cp target/release/onevox dist/onevox-linux-x86_64/
cp scripts/install_linux.sh dist/onevox-linux-x86_64/
cp scripts/uninstall_linux.sh dist/onevox-linux-x86_64/
cp README.md dist/onevox-linux-x86_64/
cp config.example.toml dist/onevox-linux-x86_64/

cd dist
tar -czf onevox-linux-x86_64.tar.gz onevox-linux-x86_64
cd ..

# Optional: Create AppImage (requires appimagetool)
# ./scripts/create_appimage.sh
# Result: dist/onevox-x86_64.AppImage

# Optional: Create Debian package
# cargo install cargo-deb
# cargo deb
# Result: target/debian/onevox_*.deb

# Result:
# - dist/onevox-linux-x86_64.tar.gz
```

### Windows Release

```bash
# Build release (on Windows or cross-compile)
cargo build --release --locked

# Strip binary to reduce size (if using MinGW)
strip target/release/onevox.exe

# Create ZIP archive
mkdir -p dist/onevox-windows-x64
cp target/release/onevox.exe dist/onevox-windows-x64/
cp README.md dist/onevox-windows-x64/
cp config.example.toml dist/onevox-windows-x64/

# On Windows (PowerShell):
Compress-Archive -Path dist/onevox-windows-x64/* -DestinationPath dist/onevox-windows-x64.zip

# On Linux/macOS (cross-compile):
cd dist
zip -r onevox-windows-x64.zip onevox-windows-x64
cd ..

# Optional: Create MSI installer (requires WiX toolset)
# cargo install cargo-wix
# cargo wix
# Result: target/wix/onevox-*.msi

# Result:
# - dist/onevox-windows-x64.zip
```

### All Platforms Script

Create a release script `scripts/release.sh`:

```bash
#!/usr/bin/env bash
set -e

VERSION=${1:-$(git describe --tags --always)}
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

echo "Building OneVox $VERSION for $PLATFORM-$ARCH"

# Create dist directory
mkdir -p dist

case "$PLATFORM" in
    darwin)
        echo "Building for macOS..."
        CC=clang CXX=clang++ SDKROOT=$(xcrun --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=13.0 \
          cargo build --release --locked
        
        ./scripts/package_macos_app.sh
        
        # Create DMG
        hdiutil create -volname "OneVox" -srcfolder dist/Onevox.app \
          -ov -format UDZO "dist/Onevox-${VERSION}-macos-${ARCH}.dmg"
        
        echo "✅ Created: dist/Onevox-${VERSION}-macos-${ARCH}.dmg"
        ;;
        
    linux)
        echo "Building for Linux..."
        cargo build --release --locked
        strip target/release/onevox
        
        # Create tarball
        mkdir -p "dist/onevox-${VERSION}-linux-${ARCH}"
        cp target/release/onevox "dist/onevox-${VERSION}-linux-${ARCH}/"
        cp scripts/install_linux.sh "dist/onevox-${VERSION}-linux-${ARCH}/"
        cp scripts/uninstall_linux.sh "dist/onevox-${VERSION}-linux-${ARCH}/"
        cp README.md "dist/onevox-${VERSION}-linux-${ARCH}/"
        cp config.example.toml "dist/onevox-${VERSION}-linux-${ARCH}/"
        
        cd dist
        tar -czf "onevox-${VERSION}-linux-${ARCH}.tar.gz" "onevox-${VERSION}-linux-${ARCH}"
        cd ..
        
        echo "✅ Created: dist/onevox-${VERSION}-linux-${ARCH}.tar.gz"
        ;;
        
    *)
        echo "Unsupported platform: $PLATFORM"
        exit 1
        ;;
esac

echo ""
echo "Release artifacts in dist/:"
ls -lh dist/
```

Make it executable:
```bash
chmod +x scripts/release.sh
```

### Create Release

```bash
# Tag the release
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0

# Build for your platform
./scripts/release.sh v0.2.0

# Upload to GitHub releases
gh release create v0.2.0 dist/* --title "OneVox v0.2.0" --notes "Release notes here"
```

### Cross-Platform Release (GitHub Actions)

For automated releases on all platforms, see the CI/CD section in this document.

## Project Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library exports
├── daemon/              # Daemon lifecycle
│   ├── lifecycle.rs     # Start/stop/signals
│   ├── dictation.rs     # Hotkey → Audio → VAD → Model → Inject
│   └── state.rs         # Shared state
├── audio/               # Audio capture
│   ├── capture.rs       # CoreAudio wrapper
│   ├── buffer.rs        # Audio buffers
│   └── devices.rs       # Device enumeration
├── vad/                 # Voice Activity Detection
│   ├── detector.rs      # VAD trait
│   ├── energy.rs        # Energy-based VAD
│   └── processor.rs     # VAD pipeline
├── models/              # Whisper models
│   ├── runtime.rs       # Model trait
│   ├── whisper_cpp.rs   # Native whisper.cpp bindings (PRIMARY)
│   ├── whisper_candle.rs # Pure Rust backend (optional)
│   ├── registry.rs      # Model metadata
│   ├── downloader.rs    # HuggingFace downloader
│   └── tokenizer.rs     # Simple tokenizer
├── platform/            # Platform-specific
│   ├── hotkey.rs        # Global hotkey (handy-keys)
│   ├── injector.rs      # Text injection (enigo)
│   ├── paths.rs         # Cross-platform paths
│   └── permissions.rs   # Permission checks
├── ipc/                 # Inter-process communication
│   ├── server.rs        # Unix socket server
│   ├── client.rs        # Client library
│   └── protocol.rs      # Message protocol
├── history.rs           # Transcription history
├── config.rs            # Configuration (TOML)
├── indicator.rs         # Recording overlay
└── tui.rs               # Terminal UI launcher

tui/                     # TypeScript TUI (Bun + OpenTUI)
├── src/
│   ├── index.ts         # Entry point
│   ├── app.ts           # Main app
│   ├── panels/          # UI panels
│   ├── components/      # Reusable components
│   └── data/            # Data layer
└── package.json
```

## Architecture

```
User presses Cmd+Shift+0
    ↓
HotkeyManager detects event
    ↓
DictationEngine starts audio capture
    ↓
AudioEngine captures from microphone
    ↓
VadProcessor detects speech segments
    ↓
WhisperModel transcribes audio
    ↓
TextInjector pastes into active app
    ↓
HistoryManager saves entry
```

## Dependencies

Key crates:
- `whisper-rs` - Native whisper.cpp bindings (PRIMARY MODEL BACKEND)
- `handy-keys` - Global hotkey detection
- `cpal` - Cross-platform audio
- `enigo` - Text injection
- `tokio` - Async runtime
- `serde` - Serialization
- `toml` - Config parsing
- `bincode` - IPC protocol

### Model Backend Architecture

OneVox uses **native whisper.cpp bindings** as the primary backend:

**Why whisper.cpp?**
- ✅ No subprocess overhead (direct library calls)
- ✅ No Python or ONNX Runtime dependencies
- ✅ Cross-platform stability (Linux, macOS, Windows)
- ✅ GPU acceleration support (Metal, CUDA, Vulkan, OpenBLAS)
- ✅ Deterministic performance
- ✅ Easy distribution (single binary)

**Build features:**
```bash
# CPU only (default)
cargo build

# With GPU acceleration
cargo build --features metal      # macOS
cargo build --features cuda       # NVIDIA
cargo build --features vulkan     # Cross-platform GPU
cargo build --features openblas   # CPU optimization
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for design decisions and [MIGRATION.md](MIGRATION.md) for migration from old backends.

## Configuration

Default config: `config.example.toml`

User config locations:
- macOS: `~/Library/Application Support/com.onevox.onevox/config.toml`
- Linux: `~/.config/onevox/config.toml`
- Windows: `%APPDATA%\onevox\config.toml`

## Models

Models stored in:
- macOS: `~/Library/Application Support/com.onevox.onevox/models/`
- Linux: `~/.local/share/onevox/models/`
- Windows: `%APPDATA%\onevox\models\`

Downloaded from HuggingFace: `ggerganov/whisper.cpp`

## Logs

Daemon logs:
- macOS: `~/Library/Logs/onevox/stdout.log`
- Linux: `~/.local/share/onevox/logs/onevox.log` or `journalctl --user -u onevox`
- Windows: `%APPDATA%\onevox\logs\onevox.log`

Set log level: `RUST_LOG=debug onevox daemon --foreground`

## Distribution

### Binary Size

| Platform | Debug | Release | Release + Strip |
|----------|-------|---------|-----------------|
| macOS ARM64 | ~80MB | ~25MB | ~20MB |
| macOS x86_64 | ~90MB | ~30MB | ~25MB |
| Linux x86_64 | ~85MB | ~28MB | ~22MB |
| Windows x64 | ~95MB | ~35MB | ~30MB |

### Static Linking

The native whisper.cpp binding compiles directly into the binary:
- ✅ Single self-contained executable
- ✅ No runtime dependencies (except system frameworks)
- ✅ No library version conflicts
- ✅ Works on any system (within OS version constraints)

**What's included in the binary:**
- whisper.cpp (statically linked)
- All Rust dependencies
- Audio processing pipeline
- VAD implementation

**What users need separately:**
- Model files (GGML format) - downloaded on first run or bundled

### Packaging

**macOS:**
```bash
./scripts/package_macos_app.sh
# Creates: dist/Onevox.app (~25-45MB)
```

**Linux:**
```bash
cargo build --release
# Binary: target/release/onevox (~22-28MB)
```

**Windows:**
```bash
cargo build --release
# Binary: target/release/onevox.exe (~30-35MB)
```

See [DISTRIBUTION.md](DISTRIBUTION.md) for detailed packaging strategies.

## CI/CD

### GitHub Actions

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build
        run: |
          CC=clang CXX=clang++ \
          SDKROOT=$(xcrun --show-sdk-path) \
          MACOSX_DEPLOYMENT_TARGET=13.0 \
          cargo build --release
      - name: Package
        run: ./scripts/package_macos_app.sh
      - name: Upload
        uses: actions/upload-artifact@v3
        with:
          name: onevox-macos
          path: dist/Onevox.app

  build-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build
        run: cargo build --release
      - name: Upload
        uses: actions/upload-artifact@v3
        with:
          name: onevox-linux
          path: target/release/onevox

  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build
        run: cargo build --release
      - name: Upload
        uses: actions/upload-artifact@v3
        with:
          name: onevox-windows
          path: target/release/onevox.exe
```


### Complete GitHub Actions Workflow

For a production-ready workflow with all platforms, create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build-macos:
    name: Build macOS (${{ matrix.target }})
    runs-on: macos-latest
    strategy:
      matrix:
        target: [x86_64-apple-darwin, aarch64-apple-darwin]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Build
        run: |
          CC=clang CXX=clang++ SDKROOT=$(xcrun --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=13.0 \
          cargo build --release --locked --target ${{ matrix.target }}
      - name: Create DMG
        run: |
          mkdir -p dist/Onevox.app/Contents/MacOS
          cp target/${{ matrix.target }}/release/onevox dist/Onevox.app/Contents/MacOS/
          cp packaging/macos/Info.plist dist/Onevox.app/Contents/
          ARCH=$(echo ${{ matrix.target }} | cut -d'-' -f1)
          hdiutil create -volname "OneVox" -srcfolder dist/Onevox.app -ov -format UDZO "Onevox-${{ github.ref_name }}-macos-${ARCH}.dmg"
      - uses: actions/upload-artifact@v4
        with:
          name: onevox-macos-${{ matrix.target }}
          path: "*.dmg"

  build-linux:
    name: Build Linux
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Dependencies
        run: sudo apt-get update && sudo apt-get install -y build-essential cmake libasound2-dev libpulse-dev
      - uses: dtolnay/rust-toolchain@stable
      - name: Build
        run: cargo build --release --locked
      - name: Create Tarball
        run: |
          strip target/release/onevox
          mkdir -p dist/onevox-${{ github.ref_name }}-linux-x86_64
          cp target/release/onevox dist/onevox-${{ github.ref_name }}-linux-x86_64/
          cp scripts/*.sh README.md config.example.toml dist/onevox-${{ github.ref_name }}-linux-x86_64/
          cd dist && tar -czf onevox-${{ github.ref_name }}-linux-x86_64.tar.gz onevox-${{ github.ref_name }}-linux-x86_64
      - uses: actions/upload-artifact@v4
        with:
          name: onevox-linux-x86_64
          path: dist/*.tar.gz

  build-windows:
    name: Build Windows
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build
        run: cargo build --release --locked
      - name: Create ZIP
        run: |
          mkdir dist\onevox-${{ github.ref_name }}-windows-x64
          copy target\release\onevox.exe dist\onevox-${{ github.ref_name }}-windows-x64\
          copy README.md dist\onevox-${{ github.ref_name }}-windows-x64\
          copy config.example.toml dist\onevox-${{ github.ref_name }}-windows-x64\
          cd dist && Compress-Archive -Path onevox-${{ github.ref_name }}-windows-x64 -DestinationPath onevox-${{ github.ref_name }}-windows-x64.zip
      - uses: actions/upload-artifact@v4
        with:
          name: onevox-windows-x64
          path: dist/*.zip

  release:
    name: Create Release
    needs: [build-macos, build-linux, build-windows]
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/download-artifact@v4
        with:
          path: artifacts
      - uses: softprops/action-gh-release@v1
        with:
          files: artifacts/**/*
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Manual Release

```bash
# 1. Update version
vim Cargo.toml  # version = "0.2.0"

# 2. Commit and tag
git add Cargo.toml
git commit -m "Release v0.2.0"
git tag -a v0.2.0 -m "Release v0.2.0"

# 3. Build for your platform
./scripts/release.sh v0.2.0

# 4. Push (triggers CI)
git push origin v0.2.0

# 5. Or upload manually
gh release create v0.2.0 dist/* --title "OneVox v0.2.0"
```
