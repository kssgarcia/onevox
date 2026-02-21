# Development

## Build

```bash
cargo build --release
```

Binary: `./target/release/onevox`

## Run

```bash
# Run daemon in foreground
./target/release/onevox daemon --foreground

# Or install locally
./scripts/install_macos.sh
```

## Test

```bash
# Test hotkey
./target/release/onevox test-hotkey --hotkey "Cmd+Shift+0"

# Test audio
./target/release/onevox test-audio --duration 3

# Test VAD
./target/release/onevox test-vad --duration 10

# Test full pipeline
./target/release/onevox test-transcribe --duration 10
```

## Package

```bash
# Create app bundle
./scripts/package_macos_app.sh

# Output: dist/Onevox.app
```

## Release

```bash
# Build release
cargo build --release --locked

# Package
./scripts/package_macos_app.sh

# Create tarball
cd dist
tar -czf onevox-macos-arm64.tar.gz Onevox.app

# Upload to GitHub releases
```

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
│   ├── whisper_cpp_cli.rs  # whisper.cpp CLI wrapper
│   ├── registry.rs      # Model metadata
│   └── downloader.rs    # HuggingFace downloader
├── platform/            # Platform-specific
│   ├── hotkey.rs        # Global hotkey (handy-keys)
│   ├── injector.rs      # Text injection (enigo)
│   ├── paths.rs         # macOS paths
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
- `handy-keys` - Global hotkey detection
- `cpal` - Cross-platform audio
- `enigo` - Text injection
- `tokio` - Async runtime
- `serde` - Serialization
- `toml` - Config parsing
- `bincode` - IPC protocol

## Configuration

Default config: `config.example.toml`

User config: `~/Library/Application Support/com.onevox.onevox/config.toml`

## Models

Models stored in: `~/Library/Application Support/com.onevox.onevox/models/`

Downloaded from HuggingFace: `ggerganov/whisper.cpp`

## Logs

Daemon logs: `~/Library/Logs/onevox/stdout.log`

Set log level: `RUST_LOG=debug onevox daemon --foreground`
