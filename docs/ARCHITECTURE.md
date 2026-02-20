# Architecture Documentation

## System Overview

Vox is architected as a **modular, event-driven daemon** with strict separation of concerns. The architecture prioritizes **low latency**, **high throughput**, and **platform portability**.

---

## Core Principles

1. **Zero-Copy Where Possible**: Minimize memory allocations in hot paths
2. **Async by Default**: Non-blocking I/O for all external interactions
3. **Fail-Safe Design**: Graceful degradation, never crash the daemon
4. **Platform Abstraction**: OS-specific code isolated behind traits
5. **Hot-Swappable Components**: Change models/configs without restart

---

## Module Hierarchy

```
vox/
├── daemon/          # Main daemon process and lifecycle
│   ├── mod.rs       # Daemon initialization and event loop
│   ├── lifecycle.rs # Start/stop/reload logic
│   └── state.rs     # Global state management
│
├── audio/           # Audio capture and processing
│   ├── mod.rs       # Public audio API
│   ├── capture.rs   # Microphone input (cpal-based)
│   ├── buffer.rs    # Ring buffer and chunk management
│   ├── stream.rs    # Audio stream processor
│   └── format.rs    # Audio format conversion
│
├── vad/             # Voice Activity Detection
│   ├── mod.rs       # VAD trait and dispatcher
│   ├── silero.rs    # Silero VAD implementation
│   └── webrtc.rs    # WebRTC VAD implementation
│
├── models/          # Transcription model backends
│   ├── mod.rs       # Model trait and factory
│   ├── whisper_cpp.rs  # whisper.cpp integration
│   ├── faster_whisper.rs # faster-whisper (PyO3 bridge)
│   ├── onnx.rs      # ONNX Runtime backend
│   └── candle.rs    # Candle (Rust-native) backend
│
├── platform/        # OS-specific integrations
│   ├── mod.rs       # Platform trait and dispatcher
│   ├── macos/       # macOS implementation
│   │   ├── hotkey.rs   # Carbon Events hotkey listener
│   │   └── inject.rs   # Accessibility API text injection
│   ├── linux/       # Linux implementation
│   │   ├── hotkey.rs   # evdev hotkey listener
│   │   └── inject.rs   # X11/Wayland injection
│   └── windows/     # Windows implementation (future)
│
├── ipc/             # Inter-process communication
│   ├── mod.rs       # IPC server and protocol
│   ├── server.rs    # Unix socket/named pipe server
│   ├── protocol.rs  # Message definitions
│   └── client.rs    # Client library for tools
│
├── tui/             # Terminal UI
│   ├── mod.rs       # TUI application entry
│   ├── app.rs       # Main app state and rendering
│   ├── widgets/     # Custom widgets
│   └── events.rs    # Event handling
│
├── config/          # Configuration management
│   ├── mod.rs       # Config loading and validation
│   └── schema.rs    # Config struct definitions
│
└── lib.rs           # Public library API
```

---

## Data Flow

### 1. User Activation (Hotkey Press)

```
User presses hotkey
         ↓
Platform layer detects event
         ↓
Event sent to daemon event loop
         ↓
Daemon enables audio capture
         ↓
Audio engine starts streaming
```

### 2. Audio Pipeline

```
Microphone → cpal callback
         ↓
Ring buffer (zero-copy write)
         ↓
Chunker (200ms windows)
         ↓
VAD filter (drop silence)
         ↓
Format converter (16kHz mono)
         ↓
Model input queue
```

### 3. Transcription Pipeline

```
Audio chunk from queue
         ↓
Model backend (whisper/etc)
         ↓
Raw transcript text
         ↓
Post-processor (punctuation, capitalization)
         ↓
Injection queue
```

### 4. Text Injection

```
Processed text from queue
         ↓
Platform injection layer
         ↓
Active application receives text
```

---

## Threading Model

### Thread Pool Architecture

```
Main Thread (Event Loop)
  ├── Handles IPC requests
  ├── Manages daemon state
  └── Dispatches tasks to workers

Audio Thread (Real-time priority)
  ├── Microphone capture callback
  ├── Writes to ring buffer
  └── Minimal processing (critical path)

VAD Thread
  ├── Reads from audio buffer
  ├── Runs VAD inference
  └── Writes chunks to model queue

Model Thread Pool (N threads)
  ├── Thread 1: Model inference
  ├── Thread 2: Model inference (if batching)
  └── Thread N: Model warmup/loading

Injection Thread
  ├── Reads from injection queue
  ├── Calls platform injection APIs
  └── Handles clipboard fallback

IPC Thread
  ├── Accepts client connections
  └── Routes messages to event loop
```

### Thread Communication

- **Channels**: `tokio::sync::mpsc` for most inter-thread communication
- **Lock-Free Queues**: `crossbeam::queue` for audio pipeline (hot path)
- **Shared State**: `Arc<RwLock<State>>` for daemon state (read-heavy)

---

## Memory Management

### Buffer Allocation Strategy

1. **Audio Ring Buffer**: Pre-allocated, fixed size (2 seconds of audio)
   - Size: `16000 Hz × 2 bytes × 2 sec = 64KB`
   - Reused continuously, no allocations in audio callback

2. **Audio Chunks**: Pooled allocations
   - Pool of 10-20 chunks (200ms each)
   - Recycled after model inference

3. **Model Buffers**: Backend-specific
   - whisper.cpp: Managed by C library
   - ONNX: Preallocated input tensors
   - Candle: Rust heap allocations

### Memory Limits

- Daemon process: **500MB idle**
- With base model loaded: **1.5GB**
- Peak during inference: **2GB**

---

## Error Handling

### Error Categories

1. **Recoverable Errors**: Log and continue
   - Audio device temporarily unavailable
   - Model inference timeout (use previous result)
   - Text injection failed (try clipboard)

2. **Fatal Errors**: Shutdown gracefully
   - Model loading failure at startup
   - IPC socket binding failure
   - Unrecoverable audio system error

### Error Propagation

```rust
type Result<T> = std::result::Result<T, VoxError>;

#[derive(Debug, thiserror::Error)]
enum VoxError {
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),
    
    #[error("Model error: {0}")]
    Model(#[from] ModelError),
    
    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),
    
    #[error("IPC error: {0}")]
    Ipc(#[from] IpcError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
}
```

---

## Configuration Architecture

### Config Layers (Priority Order)

1. **Command-line arguments** (highest priority)
2. **Environment variables** (`VOX_*`)
3. **User config file** (`~/.config/vox/config.toml`)
4. **System config file** (`/etc/vox/config.toml`)
5. **Embedded defaults** (lowest priority)

### Hot Reloading

- Watch config file with `notify` crate
- On change: validate → apply → log
- Some changes require daemon restart (models)
- Others apply immediately (hotkey, VAD threshold)

---

## Platform Abstraction

### Trait Design

```rust
#[async_trait]
trait Platform: Send + Sync {
    // Hotkey management
    async fn register_hotkey(&self, combo: KeyCombo) -> Result<HotkeyHandle>;
    async fn unregister_hotkey(&self, handle: HotkeyHandle) -> Result<()>;
    
    // Text injection
    async fn inject_text(&self, text: &str) -> Result<()>;
    async fn get_active_app(&self) -> Result<AppInfo>;
    
    // System integration
    async fn request_permissions(&self) -> Result<()>;
    fn is_accessibility_enabled(&self) -> bool;
}
```

### Platform-Specific Notes

#### macOS
- **Hotkey**: Carbon Events API or `global-hotkey` crate
- **Injection**: Accessibility API (`AXUIElementCreateSystemWide`, `AXUIElementPerformAction`)
- **Permissions**: Prompt user to enable in System Preferences
- **Clipboard**: NSPasteboard as fallback

#### Linux
- **Hotkey**: evdev or X11 keygrabbing
- **Injection**: X11 `XTestFakeKeyEvent` or Wayland `wl_keyboard`
- **Permissions**: Requires X11 server access or Wayland compositor support
- **Clipboard**: X11 clipboard via `x11-clipboard`

#### Windows (Future)
- **Hotkey**: `RegisterHotKey` Win32 API
- **Injection**: `SendInput` API
- **Permissions**: No special permissions needed
- **Clipboard**: Win32 clipboard API

---

## Model Backend Interface

### Trait Definition

```rust
#[async_trait]
trait SpeechModel: Send + Sync {
    /// Load model from disk
    async fn load(config: ModelConfig) -> Result<Self> where Self: Sized;
    
    /// Transcribe audio buffer
    async fn transcribe(&self, audio: &AudioBuffer) -> Result<Transcript>;
    
    /// Unload model and free resources
    async fn unload(&mut self) -> Result<()>;
    
    /// Get model metadata
    fn info(&self) -> ModelInfo;
    
    /// Check if GPU is available
    fn has_gpu_support(&self) -> bool;
}
```

### Backend Comparison

| Backend | Language | Latency | Accuracy | GPU | Quantization |
|---------|----------|---------|----------|-----|--------------|
| whisper.cpp | C++ | ⭐⭐⭐ | ⭐⭐⭐ | Metal, CUDA | ✅ GGML |
| faster-whisper | Python | ⭐⭐ | ⭐⭐⭐⭐ | CUDA | ✅ INT8 |
| ONNX | C++ | ⭐⭐⭐ | ⭐⭐⭐ | Multi-backend | ✅ INT8/FP16 |
| Candle | Rust | ⭐⭐ | ⭐⭐⭐ | Metal, CUDA | ✅ Custom |

### Model Loading Strategy

1. **Lazy Loading**: Load model on first use, not at daemon start
2. **Warmup**: Run dummy inference to prepare GPU
3. **Caching**: Keep last-used model in memory
4. **Preloading**: Option to preload at startup for zero-latency

---

## IPC Protocol Design

### Transport Layer
- **Unix Domain Socket** (macOS/Linux): `/tmp/vox.sock`
- **Named Pipe** (Windows): `\\.\pipe\vox`

### Message Format

```rust
// Binary encoding with bincode
#[derive(Serialize, Deserialize)]
struct Message {
    id: u64,           // Request ID for correlation
    payload: Payload,  // Command or response
}

#[derive(Serialize, Deserialize)]
enum Payload {
    Request(DaemonCommand),
    Response(DaemonResponse),
    Event(DaemonEvent),  // Unsolicited events (logs, metrics)
}
```

### Security

- Socket permissions: `0600` (owner read/write only)
- Process authentication: Verify client UID matches daemon UID
- No remote access: Only local Unix socket

---

## Observability

### Logging

- **Framework**: `tracing` with structured logging
- **Levels**: TRACE, DEBUG, INFO, WARN, ERROR
- **Outputs**:
  - File: `~/.vox/logs/vox.log` (rotated daily)
  - Syslog: macOS Console.app, Linux journald
  - TUI: Real-time log viewer

### Metrics

```rust
struct Metrics {
    // Performance
    transcription_latency_ms: Histogram,
    audio_capture_latency_ms: Histogram,
    vad_processing_time_ms: Histogram,
    
    // Throughput
    audio_chunks_processed: Counter,
    transcriptions_completed: Counter,
    text_injections: Counter,
    
    // Errors
    model_errors: Counter,
    audio_errors: Counter,
    injection_errors: Counter,
    
    // Resources
    memory_usage_mb: Gauge,
    cpu_usage_percent: Gauge,
}
```

### Tracing

- Span: Each transcription request
- Context: Audio length, model used, latency breakdown
- Export: OpenTelemetry-compatible (future)

---

## Security Considerations

1. **No Network Access**: Daemon never makes network requests
2. **Model Validation**: Verify model file checksums
3. **Config Validation**: Sanitize all user inputs
4. **Permission Boundaries**: Request only necessary permissions
5. **IPC Authentication**: Verify client identity
6. **Secrets**: Never log sensitive data

---

## Performance Optimizations

### Critical Path Optimizations

1. **Audio Callback**: Lock-free ring buffer write (<1μs)
2. **VAD**: SIMD-optimized energy calculation
3. **Model Inference**: Batch multiple chunks if possible
4. **Text Injection**: Async dispatch, don't block pipeline

### Memory Optimizations

1. **Object Pooling**: Reuse audio buffers and chunks
2. **Lazy Allocation**: Allocate model tensors only when needed
3. **Memory Mapping**: mmap model files for faster loading
4. **Quantization**: Use INT8 models to reduce memory footprint

### CPU Optimizations

1. **Thread Pinning**: Pin audio thread to dedicated core
2. **SIMD**: Use AVX2/NEON for audio processing
3. **Model Quantization**: INT8/FP16 inference
4. **Batching**: Process multiple chunks together

---

## Testing Architecture

### Unit Tests
- Each module has `#[cfg(test)]` section
- Mock all external dependencies (audio, platform, models)

### Integration Tests
- `tests/` directory with full pipeline tests
- Use test audio files (fixtures)
- Mock model for deterministic output

### Benchmarks
- `benches/` directory with `criterion.rs` benchmarks
- Measure: audio capture, VAD, model inference, injection
- Track regression in CI

### System Tests
- End-to-end tests in real applications
- Manual testing checklist
- Performance validation suite

---

## Deployment Architecture

### Installation Layout

```
/Applications/Vox.app/              (macOS)
  ├── Contents/
  │   ├── MacOS/vox                 (daemon binary)
  │   ├── Resources/models/         (bundled models)
  │   └── Info.plist

~/Library/Application Support/vox/  (user data)
  ├── config.toml
  ├── models/                       (user models)
  └── logs/

~/Library/LaunchAgents/             (autostart)
  └── com.vox.daemon.plist
```

### Autostart Configuration

**macOS (launchd)**:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" ...>
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.vox.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Applications/Vox.app/Contents/MacOS/vox</string>
        <string>daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

**Linux (systemd)**:
```ini
[Unit]
Description=Vox Speech-to-Text Daemon
After=sound.target

[Service]
Type=simple
ExecStart=/usr/bin/vox daemon
Restart=on-failure

[Install]
WantedBy=default.target
```

---

## Future Architecture Considerations

1. **Plugin System**: Dynamic loading of custom models/processors
2. **Distributed Inference**: Offload to local server for heavy models
3. **Multi-User Support**: Daemon per user vs shared daemon
4. **Cloud Sync**: Optional config/model sync (privacy-preserving)
5. **Model Marketplace**: Download quantized models in-app

---

**Last Updated**: February 20, 2026  
**Version**: 1.0
