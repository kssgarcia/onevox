<div align="center">

<img src="logo.svg" alt="OneVox Logo" width="200"/>

# OneVox

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.93%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)](https://github.com/kssgarcia/onevox)
[![GitHub release](https://img.shields.io/github/v/release/kssgarcia/onevox)](https://github.com/kssgarcia/onevox/releases)
[![GitHub stars](https://img.shields.io/github/stars/kssgarcia/onevox?style=social)](https://github.com/kssgarcia/onevox/stargazers)

**Privacy-first local speech-to-text for developers.**

Press a hotkey, speak, and your words appear instantly in any application. All processing happens locally on your machine—no cloud, no data collection, no subscriptions.

[Installation](#installation) • [Features](#features) • [Documentation](#documentation) • [Contributing](#contributing)

</div>

---

## Features

- **100% Local** - All processing on your machine, zero cloud dependencies
- **Cross-Platform** - macOS, Linux, and Windows support
- **Fast** - Native whisper.cpp integration, 50-200ms transcription latency
- **System-Wide** - Works in any application
- **Privacy-First** - Your voice data never leaves your device
- **Open Source** - MIT licensed, fully auditable

## Installation

### Quick Install

**macOS (Apple Silicon M1/M2/M3/M4)**
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/install.sh | sh
```
> **Note:** Pre-built binaries include Metal GPU acceleration by default. Intel Macs are not supported in pre-built releases—build from source instead.

**Linux**
```bash
# CPU-only (default, works on all systems)
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/install.sh | sh

# For GPU acceleration (NVIDIA, AMD, Intel), build from source
# See INSTALLATION.md for detailed build instructions
```

**Linux Post-Install**
```bash
# Add user to required groups (run once, then log out and back in)
sudo usermod -aG audio,input $USER

# Start service
systemctl --user enable --now onevox
```

**Windows**
```powershell
$onevoxDir = "$env:LOCALAPPDATA\onevox"
$asset = "onevox-windows-x86_64.zip"
Invoke-WebRequest -Uri "https://github.com/kssgarcia/onevox/releases/latest/download/$asset" -OutFile $asset
Expand-Archive -Path $asset -DestinationPath $onevoxDir -Force
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$onevoxDir", [EnvironmentVariableTarget]::User)
$env:Path += ";$onevoxDir"
& "$onevoxDir\onevox.exe" --version
```

### Release Artifacts

Each release provides platform-specific binaries:

| Platform | Artifact | GPU Support | Notes |
|----------|----------|-------------|-------|
| **macOS** | `onevox-macos-arm64.tar.gz` | ✅ Metal (built-in) | Apple Silicon only (M1/M2/M3/M4) |
| **Linux** | `onevox-linux-x86_64.tar.gz` | ❌ CPU only | Default, works on all systems |
| **Windows** | `onevox-windows-x86_64.zip` | ❌ CPU only | GPU support via custom build |

> **GPU on Linux/Windows:** Pre-built binaries are CPU-only. For GPU acceleration, build from source with `--features cuda` (NVIDIA) or `--features vulkan` (AMD/Intel). See [INSTALLATION.md](INSTALLATION.md) for detailed instructions.

See [INSTALLATION.md](INSTALLATION.md) for detailed setup instructions, troubleshooting, and service management.

### GPU Acceleration

OneVox supports GPU acceleration for significantly faster transcription (2-4x speedup):

**macOS Apple Silicon (M1/M2/M3/M4)**
```bash
# Metal GPU is included in pre-built binaries (onevox-macos-arm64.tar.gz)
# No additional setup required - just enable in config
```

**macOS Intel (Custom Build)**
```bash
# Intel Macs require building from source with Metal support
cargo build --release --features metal
```

**Linux with NVIDIA GPU**
```bash
# Build from source with CUDA support
# Requires: NVIDIA GPU + CUDA Toolkit 11.0+ installed
cargo build --release --features cuda
```

**Linux/Windows with AMD/Intel GPU**
```bash
cargo build --release --features vulkan
```

**Configuration:**
- GPU acceleration is **disabled by default** for maximum compatibility
- Enable via TUI: `onevox tui` → Model Settings → Device: "gpu"
- Or edit config: `~/.config/onevox/config.toml` → `[model] device = "gpu"`
- Check GPU status: `onevox info`

**Automatic Fallback:**
- If GPU is unavailable or fails, OneVox automatically falls back to CPU
- No configuration changes needed - it just works
- Performance: ~50-200ms with GPU, ~200-500ms with CPU (for base/small models)

### Build Variants

OneVox supports multiple model backends with automatic detection:

**Default (Recommended): whisper.cpp**
- Fast, stable, production-ready
- Native C++ integration
- Multilingual support (99+ languages) or English-only models
- GPU acceleration: Metal (macOS), CUDA, Vulkan
- ~100MB memory usage
- 50-200ms latency

```bash
# Build default (includes whisper.cpp + ONNX on ARM64 macOS/Linux/Windows)
cargo build --release
```

**Experimental: ONNX Runtime**
- Alternative models (Parakeet CTC, etc.)
- INT8 quantization for faster inference
- ~250MB memory usage
- Included by default on ARM64 macOS, Linux, Windows
- **Note**: Not available on x86_64 (Intel) macOS due to ONNX Runtime limitations

```bash
# Build with ONNX (ARM64 macOS, Linux, Windows)
cargo build --release

# Build without ONNX (x86_64 macOS or if you prefer whisper.cpp only)
cargo build --release --no-default-features --features whisper-cpp,overlay-indicator
```

Backend selection is automatic based on model choice (see Configuration below).

For pre-built binaries, see the [Releases](https://github.com/kssgarcia/onevox/releases) page.

### Uninstall

**macOS**
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_macos.sh | bash
```

**Linux**
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_linux.sh | bash
```

**Windows**
```powershell
# Download and run the uninstall script
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_windows.ps1" -OutFile uninstall_windows.ps1
.\uninstall_windows.ps1

# To keep your config file
.\uninstall_windows.ps1 -KeepConfig

# To skip confirmation prompt
.\uninstall_windows.ps1 -Force
```

The uninstaller removes:
- All binaries and executables
- Service/daemon registrations
- Application data and cache
- Configuration files (unless `-KeepConfig` is used on Windows)

## Quick Start

1. Install OneVox using the command above
2. Grant required permissions (installer will guide you)
3. Press the hotkey: `Cmd+Shift+0` (macOS) or `Ctrl+Shift+Space` (Linux/Windows)
4. Speak
5. Release the hotkey
6. Your text appears instantly

## Usage

```bash
# Check system info and GPU capabilities
onevox info

# Check daemon status
onevox status

# Open terminal UI
onevox tui

# Manage models
onevox models list
onevox models download whisper-base.en

# View history
onevox history list

# Configuration
onevox config show
```

For service management and advanced usage, see [QUICKREF.md](QUICKREF.md).

## Configuration

OneVox is highly configurable. Edit your config file to customize:

```bash
onevox config show  # View current configuration
```

**Key settings:**
- Hotkey combination and mode (push-to-talk vs toggle)
- Audio device and quality
- Model selection (auto-detects backend and language)
- Voice Activity Detection (VAD)
- Text post-processing
- GPU acceleration

**Config locations:**
- macOS: `~/Library/Application Support/com.onevox.onevox/config.toml`
- Linux: `~/.config/onevox/config.toml`
- Windows: `%APPDATA%\onevox\onevox\config\config.toml`

See [QUICKREF.md](QUICKREF.md#configuration) for all configuration options and examples, or check [config.example.toml](config.example.toml) for detailed comments.

## Architecture

OneVox uses a model-centric architecture where the backend is automatically selected based on your model choice:

### Whisper.cpp Backend (Default)
- Automatic selection for GGML models (ggml-tiny, ggml-base, ggml-small, etc.)
- Native C++ bindings for maximum performance
- Single self-contained binary
- GPU acceleration (Metal, CUDA, Vulkan)
- 50-200ms transcription latency
- ~100MB memory usage
- Supports both English-only (.en models) and multilingual models (99+ languages)

### ONNX Runtime Backend (Experimental)
- Automatic selection for ONNX models (parakeet, *.onnx files)
- Alternative models with INT8 quantization
- CPU-optimized inference
- ~250MB memory usage
- Included by default in all builds

**Model Selection:**
```toml
# config.toml
[model]
# Backend is auto-detected from model_path
model_path = "ggml-base.en"      # Uses whisper.cpp, English-only
# model_path = "ggml-base"       # Uses whisper.cpp, multilingual (auto-detect language)
# model_path = "parakeet-ctc-0.6b"  # Uses ONNX Runtime (included by default)
device = "auto"                   # or "cpu", "gpu"
preload = true
```

**Available Models:**
- `ggml-tiny.en`, `ggml-tiny` - Fastest, ~75MB
- `ggml-base.en`, `ggml-base` - Recommended, ~142MB
- `ggml-small.en`, `ggml-small` - Better accuracy, ~466MB
- `ggml-medium.en`, `ggml-medium` - High accuracy, ~1.5GB
- `ggml-large-v2`, `ggml-large-v3`, `ggml-large-v3-turbo` - Best accuracy, ~1.6-2.9GB
- `parakeet-ctc-0.6b` - ONNX, multilingual, 100+ languages

Models with `.en` suffix are English-only. Multilingual models auto-detect the spoken language.

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed technical information.

## Development

```bash
git clone https://github.com/kssgarcia/onevox.git
cd onevox
cargo build --release
```

See [DEVELOPMENT.md](DEVELOPMENT.md) for build instructions, testing, and contribution guidelines.

## Troubleshooting

**Check status:**
```bash
onevox status
```

**Common issues:**
- Hotkey not working → Check permissions (see [INSTALLATION.md](INSTALLATION.md))
- No audio → Run `onevox devices list` to verify your microphone
- Text not appearing → Verify accessibility permissions

For detailed troubleshooting, see [INSTALLATION.md](INSTALLATION.md).

## System Requirements

- **macOS:** 13.0+ (Apple Silicon or Intel)
- **Linux:** X11 or Wayland, PulseAudio/ALSA
- **Windows:** 10 (1809+) or 11
- **Disk:** ~500MB for models
- **RAM:** ~200MB runtime

## Documentation

- [INSTALLATION.md](INSTALLATION.md) - Detailed installation and platform-specific setup
- [QUICKREF.md](QUICKREF.md) - Command reference and common tasks
- [DEVELOPMENT.md](DEVELOPMENT.md) - Building, testing, and contributing
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical design and architecture
- [WAYLAND.md](WAYLAND.md) - Wayland-specific configuration

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:
- Setting up your development environment
- Code style and standards
- Pull request process
- Areas where we need help

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

Built with [whisper.cpp](https://github.com/ggerganov/whisper.cpp) and powered by OpenAI's Whisper models.
