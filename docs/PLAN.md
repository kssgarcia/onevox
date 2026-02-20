# Onevox - Development Plan

## Project Overview

**Onevox** is a cross-platform, privacy-first, local speech-to-text daemon designed for ultra-fast transcription optimized for agentic workflows. It provides a system-wide speech inference layer with minimal latency.

### Key Objectives

1. **Performance**: Sub-100ms transcription latency for real-time dictation
2. **Privacy**: 100% local processing, no cloud dependencies
3. **Flexibility**: Support multiple model backends (Whisper, Faster-Whisper, ONNX, GGUF)
4. **Integration**: Seamless injection into any application via global hotkey
5. **Reliability**: Production-grade daemon with monitoring and recovery

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     User Application                         │
│              (Any text field, any app)                       │
└───────────────────────┬─────────────────────────────────────┘
                        │ (receives text)
                        │
┌───────────────────────▼─────────────────────────────────────┐
│                  Platform Layer                              │
│  • Global Hotkey Listener                                    │
│  • Text Injection (macOS/Linux/Windows)                      │
│  • System Tray Integration                                   │
└───────────────────────┬─────────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────────┐
│                   Daemon Core (speechd)                      │
│  • Event Loop                                                │
│  • State Management                                          │
│  • IPC Server (Unix socket / Named pipe)                     │
│  • Configuration Manager                                     │
└─────┬──────────────┬──────────────┬────────────────┬────────┘
      │              │              │                │
      │              │              │                │
┌─────▼──────┐ ┌────▼─────┐ ┌──────▼──────┐ ┌──────▼──────┐
│   Audio    │ │   VAD    │ │   Model     │ │     TUI     │
│   Engine   │ │  Engine  │ │   Runtime   │ │   Monitor   │
└────────────┘ └──────────┘ └─────────────┘ └─────────────┘
     │              │              │
     │              │              │
┌────▼──────────────▼──────────────▼──────────────────────────┐
│            Transcription Pipeline                            │
│  Audio → VAD → Chunking → Model → Post-process → Inject     │
└──────────────────────────────────────────────────────────────┘
```

---

## Development Phases

### Phase 1: Core Infrastructure (Weeks 1-2)

**Goal**: Establish foundational architecture and module structure

#### Deliverables

- [ ] Project structure and build system
- [ ] Core daemon lifecycle (start/stop/reload)
- [ ] Configuration system (TOML-based)
- [ ] Logging and telemetry infrastructure
- [ ] Basic IPC protocol definition
- [ ] Error handling framework

#### Key Modules

- `daemon/`: Main daemon process
- `ipc/`: Inter-process communication
- `platform/`: OS-specific abstractions

---

### Phase 2: Audio Pipeline (Weeks 3-4)

**Goal**: Real-time audio capture with minimal latency

#### Deliverables

- [ ] Cross-platform microphone capture (via `cpal` or `rodio`)
- [ ] Audio buffer management and ring buffers
- [ ] Streaming chunk processor
- [ ] Audio format conversion (to 16kHz mono PCM)
- [ ] Device enumeration and selection
- [ ] Audio level monitoring

#### Performance Targets

- Capture latency: <10ms
- Buffer size: 100-200ms chunks
- Format: 16kHz, 16-bit, mono

#### Key Modules

- `audio/capture.rs`: Microphone input
- `audio/buffer.rs`: Ring buffer implementation
- `audio/stream.rs`: Streaming processor

---

### Phase 3: VAD Integration (Week 5)

**Goal**: Intelligent voice activity detection to reduce inference load

#### Deliverables

- [ ] Integrate Silero VAD or WebRTC VAD
- [ ] Streaming VAD with sliding window
- [ ] Configurable sensitivity thresholds
- [ ] Silence trimming and chunk boundary detection
- [ ] Pre-roll buffering (capture audio before VAD trigger)

#### Performance Targets

- VAD latency: <5ms per chunk
- Detection accuracy: >95%
- False positive rate: <1%

#### Key Modules

- `vad/detector.rs`: VAD core
- `vad/silero.rs`: Silero backend
- `vad/webrtc.rs`: WebRTC backend

---

### Phase 4: Model Runtime (Weeks 6-8)

**Goal**: Unified abstraction for multiple transcription backends

#### Deliverables

- [ ] Model trait abstraction
- [ ] Whisper.cpp integration (GGML/GGUF)
- [ ] Faster-Whisper integration (CTranslate2)
- [ ] ONNX Runtime integration
- [ ] Model loading and warm-up
- [ ] Model pooling/caching for hot-swapping
- [ ] GPU acceleration support (Metal/CUDA/ROCm)

#### Supported Backends

1. **whisper.cpp** (GGML) - Fast, quantized models
2. **faster-whisper** (via Python bridge) - Best accuracy
3. **ONNX Runtime** - Cross-platform optimization
4. **Candle** (Rust-native) - Full Rust stack

#### Performance Targets

- Model load time: <2s
- Inference latency (tiny): <100ms per second of audio
- Inference latency (base): <300ms per second of audio
- Memory usage: <500MB (tiny), <1.5GB (base)

#### Key Modules

- `models/trait.rs`: Model abstraction
- `models/whisper_cpp.rs`: whisper.cpp backend
- `models/faster_whisper.rs`: Python bridge
- `models/onnx.rs`: ONNX backend
- `models/candle.rs`: Candle backend

---

### Phase 5: Platform Integration (Weeks 9-10)

**Goal**: Global hotkey and text injection across platforms

#### Deliverables

- [ ] macOS: Accessibility API for text injection
- [ ] macOS: Carbon Events for global hotkey
- [ ] Linux: X11/Wayland text injection
- [ ] Linux: evdev/libinput global hotkey
- [ ] Windows: SendInput API (future phase)
- [ ] Clipboard fallback mechanism
- [ ] Permission handling and user consent

#### Key Modules

- `platform/macos/`: macOS-specific code
- `platform/linux/`: Linux-specific code
- `platform/windows/`: Windows-specific code (stub)
- `platform/mod.rs`: Unified API

---

### Phase 6: TUI Interface (Week 11)

**Goal**: Terminal-based monitoring and configuration

#### Deliverables

- [ ] Real-time transcription display
- [ ] Audio level meter
- [ ] Model status and switching
- [ ] Configuration editor
- [ ] Performance metrics dashboard
- [ ] Log viewer

#### Technology

- `ratatui` (TUI framework)
- `crossterm` (terminal backend)

#### Key Modules

- `tui/app.rs`: Main TUI application
- `tui/widgets/`: Custom widgets
- `tui/events.rs`: Event handling

---

### Phase 7: Optimization (Week 12)

**Goal**: Performance tuning for production use

#### Focus Areas

- [ ] Profile critical paths (flamegraph)
- [ ] Optimize memory allocations
- [ ] Thread pool tuning
- [ ] Model quantization evaluation
- [ ] Batch processing optimization
- [ ] Cache warming strategies

#### Benchmarks

- End-to-end latency benchmark suite
- Memory usage profiling
- CPU usage under load
- Concurrent dictation stress test

---

### Phase 8: Packaging & Distribution (Week 13-14)

**Goal**: Easy installation and system integration

#### Deliverables

- [ ] macOS: .app bundle + launchd plist
- [ ] Linux: systemd service + .deb/.rpm packages
- [ ] Homebrew formula
- [ ] Installation scripts
- [ ] Auto-update mechanism
- [ ] Uninstallation cleanup

---

## Technology Stack

### Core Language

- **Rust** (2021 edition, stable toolchain)

### Key Dependencies

#### Audio Processing

- `cpal` - Cross-platform audio I/O
- `hound` - WAV encoding/decoding
- `rubato` - Sample rate conversion

#### Model Inference

- `whisper-rs` - Rust bindings for whisper.cpp
- `candle-core` - Rust ML framework
- `ort` - ONNX Runtime bindings
- `pyo3` - Python interop (for faster-whisper)

#### Platform Integration

- `global-hotkey` - Cross-platform hotkey handling
- `enigo` - Cross-platform text injection
- `accessibility-sys` (macOS) - Accessibility APIs
- `x11-clipboard` (Linux) - X11 integration

#### Daemon & IPC

- `tokio` - Async runtime
- `serde` - Serialization
- `bincode` / `rmp-serde` - Binary IPC protocol
- `interprocess` - Unix sockets / Named pipes

#### TUI

- `ratatui` - Terminal UI framework
- `crossterm` - Terminal backend

#### Configuration & Logging

- `toml` - Configuration format
- `tracing` - Structured logging
- `tracing-subscriber` - Log output

#### Utilities

- `anyhow` / `thiserror` - Error handling
- `clap` - CLI argument parsing
- `dirs` - Platform directories

---

## Configuration Design

### Config File Location

- macOS: `~/Library/Application Support/onevox/config.toml`
- Linux: `~/.config/onevox/config.toml`
- Windows: `%APPDATA%\onevox\config.toml`

### Example Configuration

```toml
[daemon]
auto_start = true
log_level = "info"

[hotkey]
trigger = "Cmd+Shift+Space"  # macOS
mode = "push-to-talk"  # or "toggle"

[audio]
device = "default"  # or specific device name
sample_rate = 16000
chunk_duration_ms = 200

[vad]
enabled = true
backend = "silero"  # or "webrtc"
threshold = 0.5
pre_roll_ms = 300
post_roll_ms = 500

[model]
backend = "whisper_cpp"  # or "faster_whisper", "onnx", "candle"
model_path = "~/.onevox/models/ggml-base.en.bin"
device = "auto"  # "cpu", "cuda", "metal", "auto"
language = "en"
task = "transcribe"  # or "translate"

[model.whisper_cpp]
n_threads = 4
use_gpu = true

[post_processing]
auto_punctuation = true
auto_capitalize = true
remove_filler_words = false

[injection]
method = "accessibility"  # or "clipboard", "paste"
paste_delay_ms = 50
```

---

## IPC Protocol

### Communication Method

- **Unix Domain Sockets** (macOS/Linux)
- **Named Pipes** (Windows)

### Message Format

Binary protocol using `bincode` or MessagePack

### Commands

```rust
enum DaemonCommand {
    // Control
    Start,
    Stop,
    Reload,
    GetStatus,

    // Configuration
    SetConfig(Config),
    GetConfig,

    // Model Management
    LoadModel { backend: String, path: String },
    UnloadModel,
    ListModels,

    // Transcription
    StartDictation,
    StopDictation,

    // Monitoring
    GetMetrics,
    StreamLogs,
}

enum DaemonResponse {
    Success,
    Error(String),
    Status(DaemonStatus),
    Config(Config),
    Models(Vec<ModelInfo>),
    Metrics(Metrics),
}
```

---

## Performance Targets

### Latency Budget (Push-to-Talk → Text Injection)

| Stage                   | Target     | Critical Path |
| ----------------------- | ---------- | ------------- |
| Hotkey detection        | <5ms       | ✓             |
| Audio capture start     | <10ms      | ✓             |
| VAD detection           | <5ms/chunk | ✓             |
| Audio buffering         | 100-200ms  | ✓             |
| Model inference (tiny)  | <100ms/sec | ✓             |
| Post-processing         | <10ms      | ✓             |
| Text injection          | <20ms      | ✓             |
| **Total (1 sec audio)** | **<350ms** |               |

### Resource Limits

- **Memory**: <500MB idle, <1.5GB during inference
- **CPU**: <5% idle, burst to 100% during inference
- **Disk**: <200MB (excluding models)

---

## Testing Strategy

### Unit Tests

- Each module with >80% coverage
- Mock platform APIs for testing

### Integration Tests

- End-to-end pipeline tests
- Model loading/inference tests
- IPC communication tests

### Benchmarks

- `criterion.rs` for micro-benchmarks
- End-to-end latency benchmarks
- Memory profiling with `valgrind`/`heaptrack`

### Manual Testing

- Dogfooding: Use onevox daily for development
- Test in various applications (browsers, IDEs, terminals)
- Multi-language testing

---

## Documentation Requirements

1. **README.md**: Quick start, installation, features
2. **ARCHITECTURE.md**: System design deep-dive
3. **API.md**: IPC protocol documentation
4. **CONFIGURATION.md**: All config options explained
5. **MODELS.md**: Supported models, quantization guide
6. **CONTRIBUTING.md**: Development setup, guidelines
7. **TROUBLESHOOTING.md**: Common issues and solutions

---

## Risks and Mitigations

| Risk                                | Impact | Mitigation                                            |
| ----------------------------------- | ------ | ----------------------------------------------------- |
| Model inference too slow            | High   | Use quantized models, GPU acceleration, model pooling |
| Global hotkey conflicts             | Medium | Allow custom keybindings, detect conflicts            |
| Accessibility permissions denial    | High   | Clear onboarding, fallback to clipboard               |
| Audio device compatibility          | Medium | Support multiple audio backends, device enumeration   |
| Platform API changes                | Low    | Abstract platform layer, version checks               |
| Memory leaks in long-running daemon | High   | Rigorous testing, memory profilers, leak sanitizers   |

---

## Success Metrics

1. **Latency**: <350ms for 1-second audio (tiny model)
2. **Accuracy**: >95% WER on conversational English
3. **Stability**: 99.9% uptime over 30 days
4. **Resource**: <1.5GB memory, <10% CPU idle
5. **Compatibility**: Works on macOS 12+, Ubuntu 20.04+

---

## Future Enhancements (Post-MVP)

- [ ] Multi-language support
- [ ] Custom wake word detection
- [ ] Voice commands for system control
- [ ] Cloud model fallback (optional)
- [ ] Speaker diarization
- [ ] Real-time translation
- [ ] Plugin system for custom post-processing
- [ ] Web dashboard for monitoring
- [ ] Mobile companion app

---

## Next Steps

1. Review and approve this plan
2. Set up development environment
3. Create initial project structure
4. Begin Phase 1: Core Infrastructure
5. Establish CI/CD pipeline (GitHub Actions)

---

**Last Updated**: February 20, 2026  
**Version**: 1.0  
**Status**: Planning Phase
