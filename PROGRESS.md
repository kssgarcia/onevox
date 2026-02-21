# Onevox Development Progress

**Last Updated**: Feb 20, 2026  
**Rust Version**: 1.93.1 (latest)  
**Edition**: 2024

---

## Phase 1: Core Infrastructure ✅ COMPLETED

### ✅ Completed

- [x] Project initialization
- [x] Documentation suite (PLAN, ARCHITECTURE, DEPENDENCIES, PERFORMANCE)
- [x] Module structure
- [x] CLI scaffolding
- [x] Fix build issues (Rust 1.93.1, edition 2024)
- [x] Latest dependencies (tokio 1.43, clap 4.5, etc.)
- [x] Clean compilation (0 warnings)
- [x] Configuration system (load/save/defaults)
- [x] Config CLI commands (show working)
- [x] Release build optimized (1.4MB binary)
- [x] **IPC Protocol** (binary message format with bincode)
- [x] **IPC Server** (Unix socket server with command handling)
- [x] **IPC Client** (CLI integration library)
- [x] **Daemon State Management** (centralized state with lifecycle tracking)
- [x] **Daemon Lifecycle** (start/stop/reload with graceful shutdown)
- [x] **Event Loop** (tokio-based with signal handling)
- [x] **CLI Integration** (`daemon`, `stop`, `status` commands)
- [x] **End-to-end testing** (daemon lifecycle verified)

### 🎉 Phase 1 Complete

All core infrastructure is now in place. The daemon can:

- Start and run in the background
- Accept IPC commands via Unix socket
- Report status (version, PID, uptime, state)
- Gracefully shutdown on SIGTERM/SIGINT or via CLI
- Handle configuration loading and display

### 📋 Todo (future phases)

- [ ] Logging setup (structured logs to file)
- [ ] Basic unit tests
- [ ] CI/CD pipeline

---

## Phase 2: Audio Pipeline ✅ COMPLETED

### ✅ Completed

- [x] Audio dependencies (cpal, hound, rubato, ringbuf, dasp)
- [x] **Device Enumeration** (list and select input devices)
- [x] **Ring Buffer** (lock-free ring buffer for zero-copy streaming)
- [x] **Microphone Capture** (real-time audio input with cpal)
- [x] **Audio Streaming** (chunked audio with configurable windows)
- [x] **Format Conversion** (multi-format support: F32, I16, U16)
- [x] **AudioEngine API** (high-level interface for audio system)
- [x] **CLI Integration** (`devices list`, `test-audio` commands)
- [x] **End-to-end testing** (verified audio capture at 48kHz)

### 🎉 Phase 2 Complete

The audio pipeline is fully functional:

- Captures audio from any input device
- Real-time streaming with configurable chunk size (default 200ms)
- Zero-copy ring buffer for efficient data transfer
- Handles multiple sample formats (F32, I16, U16)
- Tested and working with MacBook microphone

### 📋 Todo (enhancements)

- [ ] Resampling (convert 48kHz → 16kHz for model input)
- [ ] Multi-channel to mono conversion
- [ ] Audio level monitoring/visualization

---

## Phase 3: VAD Integration ✅ COMPLETED

### ✅ Completed

- [x] **VAD Detector Trait** (abstract interface for VAD backends)
- [x] **Energy-based VAD** (RMS energy detection with adaptive threshold)
- [x] **VAD Processor** (streaming detection with pre/post-roll buffering)
- [x] **State Machine** (hysteresis for smooth speech/silence transitions)
- [x] **Adaptive Threshold** (background noise tracking)
- [x] **Configuration Integration** (VAD settings in main config)
- [x] **CLI Integration** (`test-vad` command for real-time testing)
- [x] **Unit Tests** (VAD detector and processor tests)

### 🎉 Phase 3 Complete

The VAD system is fully functional:

- Energy-based VAD with configurable threshold (default 0.02)
- Adaptive background noise tracking (30-chunk window)
- Pre-roll buffering (300ms of audio before speech)
- Post-roll buffering (500ms after speech ends)
- State machine with hysteresis (min 2 chunks for speech, 3 for silence)
- Speech segment extraction with metadata
- CLI test command for real-time visualization

### 📋 Enhancements (future)

- [ ] Silero VAD backend (ML-based, more accurate)
- [ ] WebRTC VAD backend
- [ ] VAD metrics and monitoring
- [ ] Dynamic threshold adjustment

---

## Phase 4: Model Runtime ✅ COMPLETED (with caveats)

### ✅ Completed

- [x] **Model Runtime Trait** (abstract interface for STT backends)
- [x] **Model Configuration** (path, language, GPU settings, beam size)
- [x] **Transcription Types** (result objects with metadata)
- [x] **Mock Model** (testing implementation for pipeline validation)
- [x] **Streaming Transcription** (VAD segment → model integration)
- [x] **CLI Integration** (`test-transcribe` command for full pipeline)
- [x] **Unit Tests** (mock model tests)

### ✅ Real Model Integration Working

- [x] **Whisper.cpp CLI Integration** (standalone binary approach)
- [x] **Model Download System** (registry + automatic downloads)
- [x] **Multiple Models Available** (ggml-tiny.en, ggml-base.en, etc.)
- [x] **Working Transcription** (real Whisper models producing actual text)

### 🎉 Phase 4 Complete (Fully Working)

The model runtime system is fully functional with real Whisper models:

- ModelRuntime trait allows easy backend swapping
- Whisper.cpp CLI backend avoids Rust binding build issues
- Real models downloaded and working (ggml-base.en, whisper-tiny.en)
- Transcription flow: Audio → VAD → Speech Segments → Whisper → Text
- Full pipeline tested via `test-transcribe` command with real models
- Model registry with automatic download support
- Binary cached at: ~/Library/Caches/onevox/bin/whisper-cli

### 📋 Enhancements (future)

- [ ] ONNX runtime backend for alternative model formats
- [ ] Faster-whisper backend for improved performance
- [ ] Model quantization options (int8, float16)
- [ ] Multi-model support (switch between models dynamically)

---

## Phase 5: Platform Integration ✅ COMPLETED

### ✅ Completed

- [x] **Global Hotkey System** (macOS with rdev)
- [x] **Hotkey Configuration** (parse hotkey strings like "Cmd+Shift+Space")
- [x] **HotkeyManager** (event listener and handler)
- [x] **Text Injection** (accessibility API integration with enigo)
- [x] **Permissions Handling** (check and request accessibility permissions)
- [x] **CLI Integration** (`test-hotkey` command for testing)
- [x] **Unit Tests** (hotkey parsing, injector creation, permissions checks)

### 🎉 Phase 5 Complete

The platform integration is fully functional:

- Global hotkey detection works across all applications
- Configurable hotkey combinations (Cmd+Shift+Space, etc.)
- Push-to-talk and toggle modes supported
- Text injection via accessibility API (macOS)
- Permission checks with user prompts
- Full integration with daemon lifecycle

### 📋 Enhancements (future)

- [ ] Linux support (X11/Wayland hotkeys)
- [ ] Windows support (Win32 API hotkeys)
- [ ] Alternative injection methods (clipboard, paste simulation)

---

## Phase 6: TUI ✅ COMPLETED

### ✅ Completed

- [x] **OpenTUI-based Terminal Interface** (TypeScript + Bun)
- [x] **Configuration Panel** (all settings editable with live UI)
- [x] **History Panel** (view past transcriptions with timestamps)
- [x] **Help Panel** (keyboard shortcuts overlay)
- [x] **Dark/Light Theme System** (toggle with 't' key, persists to config)
- [x] **Mouse Support** (click toggles, steppers, buttons, cards)
- [x] **Responsive Design** (works in 80x24 to 160x50+ terminals)
- [x] **Professional Polish** (Vercel-inspired monochrome design)
- [x] **Real-time Updates** (config changes write to config.toml)
- [x] **Rust Integration** (`onevox tui` command launches TUI)
- [x] **Auto-dependency Management** (checks Bun, installs deps automatically)

### 🎉 Phase 6 Complete

The TUI is production-ready:

- Full-featured terminal interface with 3 main tabs (Config, History, Help)
- All config options editable in real-time with instant validation
- Pure TypeScript implementation with OpenTUI framework
- Beautiful monochrome design (pure black/white/grays)
- Theme system with dark mode default
- Comprehensive documentation (TUI.md, TUI_INTEGRATION.md)
- Zero TypeScript errors, fully type-safe

### 📋 Enhancements (future)

- [ ] Real-time transcription monitoring panel
- [ ] Audio level visualization
- [ ] Model performance metrics dashboard
- [ ] Live daemon status updates via IPC

---

## Phase 7: Optimization ✅ COMPLETED

### ✅ Completed

- [x] **Profiling workflow established** (Criterion benchmark reports + CI artifacts)
- [x] **Benchmarks implemented** (`audio_processing`, `vad_processing`, `pipeline_e2e`)
- [x] **Performance tuning pass applied** (hot-path clone/log reductions and safer defaults)
- [x] **Benchmark CI added** (`.github/workflows/benchmark.yml`)

### 🎉 Phase 7 Complete

Optimization infrastructure is now operational:

- Repeatable Criterion benchmark suite in `benches/`
- CI execution of benchmark targets on each PR/push to main/codex branches
- Artifact upload for regression comparison (`target/criterion`)
- End-to-end mock pipeline benchmark for stable latency tracking without external model dependencies

---

## Phase 8: Packaging ⏸️ NOT STARTED

- [ ] macOS .app bundle
- [ ] launchd plist
- [ ] Installation script

---

## Current Status

**Phase**: 7 of 8 ✅ COMPLETED  
**Overall Progress**: ~93% (Phases 1-7 complete, fully functional + benchmarked)  
**Next Phase**: Phase 8 - Packaging  
**Next Task**: macOS packaging and installer workflow  
**Working Features**: Full end-to-end speech-to-text pipeline operational!

### ✅ What's Working NOW

- ✅ Background daemon with lifecycle management
- ✅ Audio capture from any input device
- ✅ Voice Activity Detection (energy-based)
- ✅ **Real Whisper transcription** (ggml-base.en and whisper-tiny.en models)
- ✅ Global hotkey detection (macOS)
- ✅ Text injection into any application
- ✅ Professional TUI for configuration and monitoring
- ✅ Model download and management system
- ✅ IPC protocol for CLI ↔ daemon communication
- ✅ Configuration system with TOML persistence

### 🚀 Ready to Use

You can actually use Onevox for real dictation right now:

```bash
# Start the daemon
onevox daemon --foreground

# In another terminal, open the TUI
onevox tui

# Or test the full pipeline
onevox test-transcribe --duration 10

# Check which models you have
onevox models downloaded
```

---

## Build Info

```bash
Rust: 1.93.1 (01f6ddf75 2026-02-11)
Cargo: 1.93.1 (083ac5135 2025-12-15)
Edition: 2024
Debug Binary: ~4MB
Release Binary: 1.6MB (optimized with audio + VAD + models)
Compilation: ✅ Clean (0 warnings, clippy passed)
Tests: ✅ All tests passing (16 unit tests)
Commands: daemon, stop, status, config, tui, devices, models, test-audio, test-vad, test-transcribe, test-hotkey
Models: ✅ ggml-base.en (141.1 MB), whisper-tiny.en (147.5 MB)
Binary: ~/Library/Caches/onevox/bin/whisper-cli (825KB)
```

---

## Phase 4 Implementation Summary

### New Modules Added

1. **`models/runtime.rs`** - ModelRuntime trait and types (130 lines)
2. **`models/mock.rs`** - Mock model for testing (120 lines)
3. **`models/whisper_cpp_cli.rs`** - Whisper.cpp CLI backend (298 lines) ✅ WORKING
4. **`models/registry.rs`** - Model registry and downloads (350+ lines)
5. **`models/downloader.rs`** - Model download system (200+ lines)
6. **`models/tokenizer.rs`** - Whisper tokenizer utilities (150+ lines)
7. **`models.rs`** - Model module exports (updated)

### Key Features Implemented

- ✅ ModelRuntime trait for backend abstraction
- ✅ Transcription result types with metadata
- ✅ ModelConfig for configuration management
- ✅ **Whisper.cpp CLI backend** (real working transcription)
- ✅ **Model registry system** (download any Whisper GGML model)
- ✅ **Model downloader** (automatic caching to ~/.onevox/models/)
- ✅ Integration with VAD segments
- ✅ CLI commands (`models list`, `models download`, `test-transcribe`)
- ✅ Full pipeline: Audio → VAD → Segments → Whisper → Real Text
- ✅ Multiple models: ggml-tiny.en, ggml-base.en, ggml-small.en, etc.

### Documentation Added

- ✅ `docs/WHISPER_INTEGRATION.md` - Integration details
- ✅ Model registry in `src/models/registry.rs`
- ✅ CLI backend in `src/models/whisper_cpp_cli.rs`

---

## Phase 3 Implementation Summary

### New Modules Added

1. **`vad/detector.rs`** - VadDetector trait and VadDecision enum (26 lines)
2. **`vad/energy.rs`** - Energy-based VAD implementation (191 lines)
3. **`vad/processor.rs`** - VAD processor with buffering (287 lines)
4. **`vad.rs`** - VAD module exports (updated)

### Key Features Implemented

- ✅ VadDetector trait for backend abstraction
- ✅ Energy-based VAD with RMS calculation
- ✅ Adaptive threshold with background noise tracking
- ✅ State machine with hysteresis for smooth transitions
- ✅ Pre-roll buffer (configurable, default 300ms)
- ✅ Post-roll buffer (configurable, default 500ms)
- ✅ Speech segment extraction with metadata
- ✅ Configuration integration (VadConfig)
- ✅ CLI test command (`test-vad`)
- ✅ Unit tests for VAD components

---

## Phase 2 Implementation Summary

### New Modules Added

1. **`audio/devices.rs`** - Device enumeration and management (117 lines)
2. **`audio/buffer.rs`** - Ring buffer and audio chunks (144 lines)
3. **`audio/capture.rs`** - Real-time microphone capture (232 lines)
4. **`audio.rs`** - AudioEngine public API (updated)

### Key Features Implemented

- ✅ Cross-platform audio capture with cpal
- ✅ Lock-free ring buffer (ringbuf 0.4)
- ✅ Device enumeration and selection
- ✅ Configurable chunk duration (default 200ms)
- ✅ Multi-format sample support (F32, I16, U16)
- ✅ Async chunk streaming with tokio channels
- ✅ Graceful start/stop with proper cleanup
- ✅ CLI test command for audio verification
