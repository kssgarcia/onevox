# Onevox - Local Speech-to-Text Daemon

> **⚡ Ultra-fast, privacy-first speech recognition for your entire operating system**

Onevox is a cross-platform background daemon that provides system-wide speech-to-text capabilities. Press a global hotkey, speak, and watch your words appear in any application—all processed locally with zero network latency.

---

## 🎯 Vision

This is not just another dictation app. Onevox is:

> **A Local Speech Inference Layer for the Operating System**

It's designed for developers, power users, and anyone who values:
- **Privacy**: 100% local processing, no cloud dependencies
- **Speed**: Sub-350ms latency for real-time dictation
- **Flexibility**: Multiple model backends (Whisper, ONNX, etc.)
- **Integration**: Works seamlessly in any text field, any application

---

## ✨ Features

- ✅ **Global Hotkey**: System-wide push-to-talk in any application (macOS)
- ✅ **Background Daemon**: Runs silently with full lifecycle management
- ✅ **Real Whisper Transcription**: Working with ggml-base.en and whisper-tiny.en
- ✅ **Voice Activity Detection**: Energy-based VAD with adaptive threshold
- ✅ **Model Management**: Download and manage multiple Whisper models
- ✅ **Text Injection**: Automatic text insertion via accessibility API
- ✅ **Terminal UI**: Professional TUI with dark/light themes
- ✅ **History Tracking**: Full transcription history with export support
- ✅ **Configuration System**: Full TOML-based config with hot-reload
- 🚧 **Cross-Platform**: macOS complete, Linux/Windows in progress

### 🎉 Currently Working

The app is **fully functional** on macOS! You can:
- Start the daemon and have it listen for hotkeys
- Speak and get real-time transcription from Whisper models
- Inject transcribed text into any application
- Configure everything via the beautiful terminal UI
- Manage models (download, list, remove)

---

## 🚀 Quick Start

### Prerequisites

**Rust 1.93+ Required**:
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version  # Should show 1.93 or higher
```

**macOS**:
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian)**:
```bash
sudo apt-get install build-essential cmake pkg-config
```

### Build & Run

```bash
# Clone repository
git clone https://github.com/yourusername/onevox.git
cd onevox

# Check if it compiles (fast!)
cargo check

# Run the program
cargo run -- --help

# Try the config system
cargo run -- config show

# Build optimized binary
cargo build --release

# Binary is at: ./target/release/onevox
./target/release/onevox --version
```

to add command `onevox` (mac):

``` bash
sudo ln -sf /Users/kevinsepulveda/Documents/onevox/target/release/onevox /usr/local/bin/onevox
onevox --help
```

### For TypeScript/Node Developers

If you're coming from TypeScript/pnpm, see **[CHEATSHEET.md](CHEATSHEET.md)** for command equivalents!

```bash
cargo run       # = pnpm dev
cargo build     # = pnpm install + build
cargo test      # = pnpm test
cargo fmt       # = pnpm format
```
    libasound2-dev libx11-dev portaudio19-dev
```

### Installation (Future)

```bash
# Install from source
cargo install --git https://github.com/yourusername/onevox

# Or via Homebrew (macOS)
brew install onevox

# Or download binary
curl -L https://github.com/yourusername/onevox/releases/latest/download/onevox-macos.tar.gz | tar xz
```

### Usage

```bash
# Start daemon in foreground (for testing)
onevox daemon --foreground

# Start daemon in background
onevox daemon

# Check status
onevox status

# Configure
onevox config show
onevox config init  # Create default config file

# Download and manage models
onevox models list       # See all available models
onevox models download ggml-base.en  # Download a model
onevox models downloaded # See what you have

# Test the pipeline
onevox test-audio --duration 5        # Test mic capture
onevox test-vad --duration 10         # Test voice detection
onevox test-transcribe --duration 10  # Test full transcription

# Open TUI monitor
onevox tui

# Stop daemon
onevox stop

# View transcription history
onevox history list           # Show recent transcriptions
onevox history list --limit 5 # Show last 5 entries
onevox history delete <id>    # Delete a specific entry
onevox history clear          # Clear all history (with confirmation)
onevox history export         # Export history to text file
```

### Quick Test (Real Transcription)

```bash
# 1. Make sure you have a model
cargo run -- models downloaded

# 2. If no models, download one (141 MB)
cargo run -- models download ggml-base.en

# 3. Test real transcription (speak into your mic!)
cargo run -- test-transcribe --duration 10

# 4. Open the TUI to configure settings
cargo run -- tui
```

---

## 🖥️ Terminal User Interface (TUI)

Onevox includes a **production-ready** terminal interface built with **OpenTUI** and **TypeScript/Bun**.

### Features

- 🎨 **Dark/Light themes** - Pure monochrome Vercel-inspired design (toggle with `t`)
- ⚙️ **Interactive configuration** - All daemon settings editable in real-time
- 📜 **History viewer** - Browse past transcriptions with timestamps
- 🎤 **Device selection** - Visual audio device picker
- ⌨️ **Full keyboard + mouse support** - Click or use keyboard shortcuts
- 💾 **Instant persistence** - Config changes save immediately to TOML

### Quick Start

**Prerequisites:** Install [Bun](https://bun.sh)
```bash
curl -fsSL https://bun.sh/install | bash
```

**Launch TUI:**
```bash
# Method 1: Via Rust CLI (recommended - auto-installs dependencies)
onevox tui

# Method 2: Direct launch
cd tui && bun install && bun start

# Method 3: Helper script
./scripts/run-tui.sh
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Switch tabs (Config ↔ History ↔ Help) |
| `t` | Toggle dark/light theme |
| `Ctrl+S` | Save configuration |
| `?` | Show help overlay |
| `q` / `Ctrl+C` | Quit |

**Config Panel:**
- `Tab` / `Shift+Tab` - Navigate fields
- `Space` - Toggle switches
- `←` / `→` - Adjust steppers
- Click any control with mouse!

**History Panel:**
- `↑` / `↓` or `j` / `k` - Navigate entries
- `c` - Copy transcription to clipboard
- `e` - Export entry to file
- `dd` or `x` - Delete entry (Vim-style)
- `D` (Shift+d) - Clear all history
- `Enter` - Expand full details

### Documentation

- **Full Guide:** [docs/TUI_INTEGRATION.md](docs/TUI_INTEGRATION.md)
- **Architecture:** [docs/TUI.md](docs/TUI.md)
- **Quick Start:** [tui/README.md](tui/README.md)

---

## 📜 History Management

Onevox automatically tracks all your transcriptions with timestamps, model information, and performance metrics. History is stored locally in JSON format.

### Features

- **Automatic Tracking**: Every transcription is logged with metadata
- **Persistent Storage**: History survives daemon restarts
- **Configurable Limits**: Set maximum entries to manage disk space
- **Privacy Controls**: Easy deletion or complete clearing
- **Export Support**: Export history for backup or analysis

### CLI Commands

```bash
# View recent transcriptions
onevox history list

# Limit output
onevox history list --limit 10

# Delete a specific entry
onevox history delete 42

# Clear all history (with confirmation)
onevox history clear

# Force clear without confirmation
onevox history clear --yes

# Export to text file
onevox history export
onevox history export --output my-transcriptions.txt
```

### Storage Location

History is stored in platform-specific locations:

- **macOS**: `~/Library/Application Support/onevox/history.json`
- **Linux**: `~/.local/share/onevox/history.json`
- **Windows**: `%APPDATA%\onevox\history.json`

### Configuration

Edit your `config.toml` to customize history behavior:

```toml
[history]
# Enable/disable history tracking
enabled = true

# Maximum entries to keep (oldest are auto-removed)
max_entries = 1000

# Auto-save after each transcription (vs. only on shutdown)
auto_save = true
```

### History Format

Each entry contains:
- **ID**: Unique identifier
- **Timestamp**: When the transcription occurred
- **Text**: The transcribed content
- **Model**: Which model was used
- **Duration**: Processing time in milliseconds
- **Confidence**: Transcription confidence score (if available)

### TUI Integration

The Terminal UI includes a dedicated **History Panel** where you can:
- Browse all past transcriptions
- View detailed metadata
- Copy transcriptions to clipboard
- Delete individual entries
- All with full keyboard and mouse support

Press `Tab` in the TUI to navigate to the History panel.

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

### Current Phase: **Phase 6/8 Complete - Fully Functional!** 🎉

**What's Working:**
- ✅ **Phase 1**: Core infrastructure (daemon, IPC, config)
- ✅ **Phase 2**: Audio pipeline (capture, streaming, device selection)
- ✅ **Phase 3**: Voice Activity Detection (energy-based VAD)
- ✅ **Phase 4**: Model runtime (Whisper.cpp CLI, real transcription)
- ✅ **Phase 5**: Platform integration (global hotkeys, text injection - macOS)
- ✅ **Phase 6**: Terminal UI (production-ready TUI with themes)
- 🚧 **Phase 7**: Performance optimization (next up)
- 🚧 **Phase 8**: Packaging & distribution (planned)

**You can use Onevox for real dictation on macOS right now!**

See [PROGRESS.md](PROGRESS.md) for detailed implementation status.

---

## 🛠️ Technology Stack

### Core (Rust)
- **Language**: Rust 1.93+ (Edition 2024)
- **Audio**: `cpal` for cross-platform capture
- **VAD**: Energy-based VAD with adaptive thresholding
- **Models**: Whisper.cpp CLI (ggml models), ONNX Runtime (planned)
- **Platform**: `rdev` hotkeys (macOS), `enigo` text injection
- **Async**: `tokio` runtime
- **IPC**: Unix domain sockets with `bincode`

### Terminal UI (TypeScript)
- **Framework**: OpenTUI (flexbox-based TUI framework)
- **Runtime**: Bun (fast TypeScript runtime)
- **Config**: TOML parsing
- **Styling**: Light theme, borderless design

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
git clone https://github.com/yourusername/onevox.git
cd onevox

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

- **Issues**: [GitHub Issues](https://github.com/yourusername/onevox/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/onevox/discussions)
- **Email**: your.email@example.com

---

## ⚠️ Current Status

**This project is in the planning phase.** The documentation represents the intended architecture and features. Active development will begin soon.

Star ⭐ this repository to follow the progress!

---

**Built with ❤️ by developers who value privacy and performance**
