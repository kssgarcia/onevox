# Vox - Local Speech-to-Text Daemon

> **⚡ Ultra-fast, privacy-first speech recognition for your entire operating system**

Vox is a cross-platform background daemon that provides system-wide speech-to-text capabilities. Press a global hotkey, speak, and watch your words appear in any application—all processed locally with zero network latency.

---

## 🎯 Vision

This is not just another dictation app. Vox is:

> **A Local Speech Inference Layer for the Operating System**

It's designed for developers, power users, and anyone who values:
- **Privacy**: 100% local processing, no cloud dependencies
- **Speed**: Sub-350ms latency for real-time dictation
- **Flexibility**: Multiple model backends (Whisper, ONNX, etc.)
- **Integration**: Works seamlessly in any text field, any application

---

## ✨ Features (Planned)

- ✅ **Global Hotkey**: System-wide push-to-talk in any application
- ✅ **Background Daemon**: Runs silently, always ready
- ✅ **Real-time Transcription**: Low-latency streaming inference
- ✅ **Voice Activity Detection**: Intelligent silence trimming
- ✅ **Multiple Model Support**: Whisper, Faster-Whisper, ONNX, GGUF
- ✅ **GPU Acceleration**: Metal (macOS), CUDA (Linux/Windows)
- ✅ **Terminal UI**: Monitor and configure via TUI
- ✅ **Cross-Platform**: macOS, Linux, Windows (future)

---

## 🚀 Quick Start

> **Note**: Vox is currently in active development. This is a planning/documentation phase.

### Prerequisites

**macOS**:
```bash
xcode-select --install
brew install cmake portaudio
```

**Linux (Ubuntu/Debian)**:
```bash
sudo apt-get install build-essential cmake pkg-config \
    libasound2-dev libx11-dev portaudio19-dev
```

### Installation (Future)

```bash
# Install from source
cargo install --git https://github.com/yourusername/vox

# Or via Homebrew (macOS)
brew install vox

# Or download binary
curl -L https://github.com/yourusername/vox/releases/latest/download/vox-macos.tar.gz | tar xz
```

### Usage (Planned)

```bash
# Start daemon
vox daemon start

# Check status
vox status

# Configure
vox config set hotkey "Cmd+Shift+Space"
vox config set model whisper-tiny

# Open TUI monitor
vox tui

# Stop daemon
vox daemon stop
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│      User Application (Any App)         │
└──────────────┬──────────────────────────┘
               │ (text injection)
┌──────────────▼──────────────────────────┐
│         Platform Layer                  │
│  • Hotkey Listener                      │
│  • Text Injection                       │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│         Daemon Core (speechd)           │
│  • Event Loop                           │
│  • IPC Server                           │
│  • State Management                     │
└─┬────────┬──────────┬───────────┬───────┘
  │        │          │           │
┌─▼──┐  ┌─▼──┐  ┌────▼────┐  ┌──▼──┐
│Audio│  │VAD │  │  Model  │  │ TUI │
└────┘  └────┘  └─────────┘  └─────┘
```

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed design.

---

## 📋 Project Status

### Current Phase: **Planning & Documentation** ✍️

- [x] Project initialization
- [x] Architecture design
- [x] Technology stack selection
- [ ] Core infrastructure (Phase 1)
- [ ] Audio pipeline (Phase 2)
- [ ] VAD integration (Phase 3)
- [ ] Model runtime (Phase 4)
- [ ] Platform integration (Phase 5)
- [ ] TUI interface (Phase 6)
- [ ] Performance optimization (Phase 7)
- [ ] Packaging & distribution (Phase 8)

See [PLAN.md](docs/PLAN.md) for the complete development roadmap.

---

## 🛠️ Technology Stack

- **Language**: Rust (2021 edition)
- **Audio**: `cpal` for cross-platform capture
- **VAD**: Silero VAD (ONNX) or WebRTC VAD
- **Models**: whisper.cpp, Faster-Whisper, ONNX Runtime, Candle
- **Platform**: `global-hotkey`, Accessibility APIs, X11/Wayland
- **TUI**: `ratatui` + `crossterm`
- **Async**: `tokio`

See [DEPENDENCIES.md](docs/DEPENDENCIES.md) for full dependency list.

---

## 📊 Performance Targets

| Metric | Target | Hardware |
|--------|--------|----------|
| End-to-end latency (1sec audio) | <350ms | M1 Pro, Tiny model |
| Model inference (tiny) | <100ms | M1 Pro, Metal |
| Hotkey activation | <10ms | Any |
| Memory usage (idle) | <500MB | Any |
| Memory usage (active) | <1.5GB | With base model |

See [PERFORMANCE.md](docs/PERFORMANCE.md) for benchmarks and optimization guide.

---

## 🧪 Development

### Build from Source

```bash
# Clone repository
git clone https://github.com/yourusername/vox.git
cd vox

# Build
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Development Commands

```bash
# Run daemon in development mode
cargo run -- daemon --dev

# Run with debug logging
RUST_LOG=debug cargo run -- daemon

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open
```

---

## 📚 Documentation

- [**Development Plan**](docs/PLAN.md) - Roadmap and milestones
- [**Architecture**](docs/ARCHITECTURE.md) - System design deep-dive
- [**Dependencies**](docs/DEPENDENCIES.md) - Technology stack
- [**Performance**](docs/PERFORMANCE.md) - Benchmarks and optimization

---

## 🤝 Contributing

Contributions are welcome! This project is in early development, so there are many opportunities to contribute.

### How to Contribute

1. Check the [development plan](docs/PLAN.md) for current focus areas
2. Pick an issue or propose a new feature
3. Fork the repository
4. Create a feature branch (`git checkout -b feature/amazing-feature`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Development Guidelines

- Follow Rust best practices and idioms
- Write tests for new functionality
- Update documentation as needed
- Run `cargo fmt` and `cargo clippy` before committing
- Keep commits atomic and well-described

---

## 📜 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [OpenAI Whisper](https://github.com/openai/whisper) - The foundation for accurate transcription
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) - Fast C++ implementation
- [Silero VAD](https://github.com/snakers4/silero-vad) - Excellent voice activity detection
- The Rust community for amazing libraries and tools

---

## 🔮 Roadmap

### Version 0.1.0 (MVP) - Q2 2026
- ✅ Core daemon infrastructure
- ✅ macOS support
- ✅ whisper.cpp integration (tiny/base models)
- ✅ Basic TUI

### Version 0.2.0 - Q3 2026
- ✅ Linux support
- ✅ Multiple model backends (ONNX, Candle)
- ✅ Advanced VAD
- ✅ Performance optimizations

### Version 1.0.0 - Q4 2026
- ✅ Windows support
- ✅ Plugin system
- ✅ Multi-language support
- ✅ Production-ready stability

---

## 📞 Contact & Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/vox/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/vox/discussions)
- **Email**: your.email@example.com

---

## ⚠️ Current Status

**This project is in the planning phase.** The documentation represents the intended architecture and features. Active development will begin soon.

Star ⭐ this repository to follow the progress!

---

**Built with ❤️ by developers who value privacy and performance**
