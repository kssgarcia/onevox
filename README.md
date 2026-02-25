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

**macOS**
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/install.sh | sh
```

**Linux**
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/install_linux.sh | bash

# Add user to required groups
sudo usermod -aG audio,input $USER
# Log out and back in

# Start service
systemctl --user start onevox
systemctl --user enable onevox
```

**Windows**
```powershell
# Download from releases
# https://github.com/kssgarcia/onevox/releases
```

See [INSTALLATION.md](INSTALLATION.md) for detailed setup instructions and troubleshooting.

## Quick Start

1. Install OneVox using the command above
2. Grant required permissions (installer will guide you)
3. Press the hotkey: `Cmd+Shift+0` (macOS) or `Ctrl+Shift+Space` (Linux/Windows)
4. Speak
5. Release the hotkey
6. Your text appears instantly

## Usage

```bash
# Check status
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
- Model selection (tiny, base, small, medium, large)
- Voice Activity Detection (VAD)
- Text post-processing
- GPU acceleration

**Config locations:**
- macOS: `~/Library/Application Support/com.onevox.onevox/config.toml`
- Linux: `~/.config/onevox/config.toml`
- Windows: `%APPDATA%\onevox\onevox\config\config.toml`

See [QUICKREF.md](QUICKREF.md#configuration) for all configuration options and examples, or check [config.example.toml](config.example.toml) for detailed comments.

## Architecture

OneVox uses native whisper.cpp bindings for maximum performance and reliability:

- Native Rust + whisper.cpp (no Python, no ONNX Runtime)
- Single self-contained binary
- GPU acceleration support (Metal, CUDA, Vulkan)
- 50-200ms transcription latency
- Cross-platform stability

See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details.

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
