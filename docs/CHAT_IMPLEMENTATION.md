# Chat Feature Implementation Checklist

> **Goal**: Add STT→LLM→TTS conversational AI pipeline to OneVox while maintaining existing transcription performance.

## Overview

This document provides a step-by-step implementation guide for adding conversational AI capabilities to OneVox. The feature will allow users to have voice conversations with an AI using a separate hotkey from the transcription feature.

### Architecture

```
User speaks → STT (Whisper) → LLM (Liquid AI) → TTS (Kokoro) → Audio playback
     ↓              ↓                ↓                ↓              ↓
  Hotkey 2      Transcribe      Generate text    Synthesize     Play audio
```

### Key Requirements

- ✅ Maintain existing STT transcription performance (50-200ms)
- ✅ Keep both features independent (dual hotkey support)
- ✅ Target <1s total latency for chat pipeline
- ✅ CPU-friendly by default, GPU optional
- ✅ Memory budget: ~2-3GB with all models loaded
- ✅ Production-ready error handling and logging

---

## Phase 1: Configuration & Foundation ⏱️ 2-3 days

### 1.1 Configuration Structures

**File**: `src/config.rs`

- [ ] Add `ChatConfig` struct with:
  - `enabled: bool`
  - `hotkey: String`
  - `llm: LlmConfig`
  - `tts: TtsConfig`
  
- [ ] Add `LlmConfig` struct with:
  - `model_path: String`
  - `device: String`
  - `context_length: usize`
  - `temperature: f32`
  - `max_tokens: usize`
  - `system_prompt: String`
  - `preload: bool`
  
- [ ] Add `TtsConfig` struct with:
  - `model_path: String`
  - `device: String`
  - `voice_id: String`
  - `speech_rate: f32`
  - `preload: bool`

- [ ] Implement `Default` for all new config structs
- [ ] Add platform-specific default hotkeys (Cmd+Shift+9 for macOS, Ctrl+Shift+Alt+Space for others)
- [ ] Update `Config::load()` and `Config::save()` to handle new fields

**Testing**:
```bash
# Test config loading with chat section
cargo test config::tests::test_chat_config_load
cargo test config::tests::test_chat_config_defaults
```

### 1.2 Model Registry Updates

**File**: `src/models/registry.rs`

- [ ] Add `ModelType` enum:
  ```rust
  pub enum ModelType {
      STT,  // Speech-to-Text
      LLM,  // Large Language Model
      TTS,  // Text-to-Speech
  }
  ```

- [ ] Update `ModelMetadata` to include:
  - `model_type: ModelType`
  - `gpu_recommended: bool`

- [ ] Add `ModelFormat::GGUF` variant

- [ ] Add LLM models to registry:
  - `lfm2-1.2b-tool` (GGUF, 1.2GB, CPU-friendly)
  - `lfm25-audio-1.5b-onnx` (ONNX, 1.5GB, GPU-recommended)

- [ ] Add TTS models to registry:
  - `kokoro-82m-onnx` (ONNX, 82MB, CPU-friendly)

- [ ] Add helper methods:
  - `list_models_by_type(model_type: ModelType) -> Vec<&ModelMetadata>`
  - `get_recommended_llm() -> &ModelMetadata`
  - `get_recommended_tts() -> &ModelMetadata`

**Testing**:
```bash
cargo test registry::tests::test_model_types
cargo test registry::tests::test_llm_models
cargo test registry::tests::test_tts_models
```

### 1.3 Update Example Config

**File**: `config.example.toml`

- [ ] Add complete `[chat]` section with documentation
- [ ] Add `[chat.llm]` subsection with all parameters
- [ ] Add `[chat.tts]` subsection with all parameters
- [ ] Document voice options for Kokoro
- [ ] Add performance notes and recommendations

---

## Phase 2: Runtime Abstractions ⏱️ 3-4 days

### 2.1 LLM Runtime Trait

**File**: `src/models/llm_runtime.rs` (new)

- [ ] Define `LlmResponse` struct:
  - `text: String`
  - `tokens: usize`
  - `generation_time_ms: u64`
  - `tokens_per_second: f32`

- [ ] Define `ChatMessage` struct:
  - `role: MessageRole` (System/User/Assistant)
  - `content: String`

- [ ] Define `LlmConfig` struct for runtime

- [ ] Define `LlmRuntime` trait:
  - `load(&mut self, config: LlmConfig) -> Result<()>`
  - `is_loaded(&self) -> bool`
  - `generate(&mut self, messages: &[ChatMessage]) -> Result<LlmResponse>`
  - `unload(&mut self)`
  - `name(&self) -> &str`

- [ ] Add optional streaming support (placeholder for future):
  - `generate_stream(&mut self, ...) -> Result<LlmResponseStream>`

**Testing**:
```bash
cargo test models::llm_runtime::tests
```

### 2.2 TTS Runtime Trait

**File**: `src/models/tts_runtime.rs` (new)

- [ ] Define `TtsSynthesis` struct:
  - `samples: Vec<f32>`
  - `sample_rate: u32`
  - `synthesis_time_ms: u64`
  - `audio_duration_ms: u64`
  - `rtf: f32` (Real-Time Factor)

- [ ] Define `VoiceInfo` struct:
  - `id: String`
  - `name: String`
  - `language: String`
  - `gender: Option<String>`

- [ ] Define `TtsConfig` struct for runtime

- [ ] Define `TtsRuntime` trait:
  - `load(&mut self, config: TtsConfig) -> Result<()>`
  - `is_loaded(&self) -> bool`
  - `synthesize(&mut self, text: &str) -> Result<TtsSynthesis>`
  - `list_voices(&self) -> Vec<VoiceInfo>`
  - `unload(&mut self)`
  - `name(&self) -> &str`

**Testing**:
```bash
cargo test models::tts_runtime::tests
```

### 2.3 Update Module Exports

**File**: `src/models.rs`

- [ ] Add `pub mod llm_runtime;`
- [ ] Add `pub mod tts_runtime;`
- [ ] Re-export common types:
  ```rust
  pub use llm_runtime::{LlmRuntime, LlmResponse, ChatMessage, MessageRole};
  pub use tts_runtime::{TtsRuntime, TtsSynthesis, VoiceInfo};
  ```

---

## Phase 3: Backend Implementations ⏱️ 5-7 days

### 3.1 GGUF LLM Backend (Priority 1)

**File**: `src/models/llm_gguf.rs` (new)

**Dependencies**: Add to `Cargo.toml`:
```toml
[dependencies]
llama-cpp-rs = { version = "0.1", optional = true }

[features]
llm-gguf = ["llama-cpp-rs"]
chat = ["llm-gguf", "tts-onnx"]
```

**Implementation**:
- [ ] Create `LlmGguf` struct:
  - `model: Option<LlamaModel>`
  - `context: Option<LlamaContext>`
  - `config: Option<LlmConfig>`

- [ ] Implement `LlmRuntime::load()`:
  - Load GGUF model from path
  - Initialize context with configured parameters
  - Handle GPU device selection
  - Log model info (size, layers, context length)

- [ ] Implement `LlmRuntime::generate()`:
  - Format conversation history to prompt
  - Run inference with configured temperature/max_tokens
  - Decode tokens to text
  - Calculate tokens/sec metric
  - Handle generation errors gracefully

- [ ] Add prompt formatting helper:
  - Support ChatML format
  - Support Llama format
  - Auto-detect from model config

- [ ] Add token counting utilities

**Testing**:
```bash
# Unit tests
cargo test models::llm_gguf::tests --features chat

# Integration test with actual model (manual)
cargo run --features chat --bin test_llm_gguf
```

**Performance Target**: 20-50 tokens/sec on CPU for 1-2B models

### 3.2 ONNX LLM Backend (Priority 2)

**File**: `src/models/llm_onnx.rs` (new)

**Note**: Uses existing ONNX Runtime dependency

**Implementation**:
- [ ] Create `LlmOnnx` struct:
  - `session: Option<Session>`
  - `tokenizer: Option<Tokenizer>`
  - `config: Option<LlmConfig>`

- [ ] Implement `LlmRuntime::load()`:
  - Load ONNX model and tokenizer
  - Configure execution providers (CPU/GPU)
  - Validate input/output shapes

- [ ] Implement `LlmRuntime::generate()`:
  - Tokenize input messages
  - Run ONNX inference in loop (autoregressive)
  - Decode tokens with stopping criteria
  - Calculate performance metrics

- [ ] Add ONNX-specific optimizations:
  - KV-cache management
  - Batch inference support
  - Dynamic axis handling

**Testing**:
```bash
cargo test models::llm_onnx::tests --features chat
```

**Performance Target**: 15-40 tokens/sec with ONNX optimizations

### 3.3 Kokoro TTS Backend (Priority 1)

**File**: `src/models/tts_kokoro.rs` (new)

**Note**: Uses existing ONNX Runtime dependency

**Implementation**:
- [ ] Create `TtsKokoro` struct:
  - `session: Option<Session>`
  - `config: Option<TtsConfig>`
  - `voices: Vec<VoiceInfo>`

- [ ] Initialize voice list:
  - `af_heart` - Female, American, warm
  - `af_sky` - Female, American, clear
  - `am_adam` - Male, American, deep
  - `am_michael` - Male, American, friendly
  - `bf_emma` - Female, British, elegant
  - `bm_george` - Male, British, authoritative

- [ ] Implement `TtsRuntime::load()`:
  - Load Kokoro ONNX model
  - Load phoneme vocabulary
  - Configure voice parameters
  - Validate sample rate support

- [ ] Implement `TtsRuntime::synthesize()`:
  - Preprocess text (normalization, phonemization)
  - Run ONNX inference
  - Post-process audio (denormalization, resampling)
  - Calculate RTF metric
  - Handle long text (chunking if needed)

- [ ] Add text preprocessing:
  - Number expansion (123 → "one hundred twenty-three")
  - Abbreviation expansion (Dr. → "doctor")
  - Punctuation handling for prosody

- [ ] Implement `list_voices()` to return available voices

**Testing**:
```bash
cargo test models::tts_kokoro::tests --features chat

# Manual audio quality test
cargo run --features chat --bin test_tts_kokoro -- --text "Hello, world!" --voice af_heart
```

**Performance Target**: RTF < 0.5 on CPU (faster than real-time)

### 3.4 Mock Implementations for Testing

**File**: `src/models/llm_mock.rs` (new)
**File**: `src/models/tts_mock.rs` (new)

- [ ] Create `MockLlm` for unit tests
- [ ] Create `MockTts` for unit tests
- [ ] Return predictable responses with configurable delays
- [ ] Useful for integration testing without actual models

---

## Phase 4: Audio Playback ⏱️ 2-3 days

### 4.1 Audio Player Implementation

**File**: `src/platform/audio/player.rs` (new)

**Dependencies**: Uses existing `cpal` crate

**Implementation**:
- [ ] Create `AudioPlayer` struct:
  - `device: Device`
  - `config: StreamConfig`

- [ ] Implement `AudioPlayer::new()`:
  - Get default output device
  - Configure output stream parameters
  - Validate device capabilities

- [ ] Implement `AudioPlayer::play()`:
  - Convert sample rate if needed (use `rubato` crate)
  - Create output stream
  - Feed samples to stream
  - Wait for playback completion
  - Handle buffer underruns gracefully

- [ ] Add volume control (optional):
  - `set_volume(level: f32)`
  - Apply gain to samples

- [ ] Add playback state management:
  - `stop()` - interrupt current playback
  - `is_playing() -> bool`

**Testing**:
```bash
cargo test platform::audio::player::tests

# Manual playback test
cargo run --bin test_audio_player -- --file test.wav
```

### 4.2 Platform-Specific Optimizations

- [ ] macOS: Use CoreAudio for lower latency
- [ ] Linux: Support ALSA and PulseAudio
- [ ] Windows: Use WASAPI

**File**: `src/platform/audio/mod.rs`

- [ ] Update module structure to include player
- [ ] Add platform-specific player backends if needed

---

## Phase 5: Chat Engine ⏱️ 4-5 days

### 5.1 Create Chat Engine

**File**: `src/daemon/chat.rs` (new)

**Implementation**:
- [ ] Create `ChatEngine` struct:
  - `config: ChatConfig`
  - `stt_model: Arc<RwLock<Box<dyn ModelRuntime>>>`
  - `llm_model: Arc<RwLock<Box<dyn LlmRuntime>>>`
  - `tts_model: Arc<RwLock<Box<dyn TtsRuntime>>>`
  - `audio_engine: Arc<AudioEngine>`
  - `audio_player: Arc<AudioPlayer>`
  - `conversation_history: Vec<ChatMessage>`
  - `is_active: Arc<AtomicBool>`

- [ ] Implement `ChatEngine::new()`:
  - Initialize with models and config
  - Set up system prompt in conversation history
  - Create audio engine and player instances

- [ ] Implement `ChatEngine::start_conversation()`:
  1. Check if already active (prevent concurrent conversations)
  2. Capture audio (record user speech)
  3. Transcribe with STT model
  4. Add user message to conversation history
  5. Generate LLM response
  6. Add assistant message to conversation history
  7. Synthesize speech with TTS
  8. Play audio response
  9. Log performance metrics for each step
  10. Handle errors at each stage

- [ ] Implement `ChatEngine::capture_audio()`:
  - Similar to `DictationEngine::start_dictation()`
  - Support VAD or fixed duration
  - Return audio samples as `Vec<f32>`

- [ ] Implement `ChatEngine::clear_history()`:
  - Keep system prompt
  - Clear user/assistant messages
  - Reset conversation context

- [ ] Add conversation management:
  - `get_history() -> &[ChatMessage]`
  - `set_system_prompt(prompt: String)`
  - `get_last_response() -> Option<&str>`

**Error Handling**:
- STT fails → Return error to user via TTS ("Sorry, I couldn't understand that")
- LLM fails → Return fallback message ("I'm having trouble generating a response")
- TTS fails → Log error, display text response instead
- Audio playback fails → Log error, continue

**Logging**:
```rust
info!("💬 Chat turn started");
debug!("🎤 Audio captured: {:.2}s", duration_secs);
debug!("📝 STT latency: {}ms", stt_time);
debug!("🤖 LLM tokens: {} ({} tok/s)", tokens, tok_per_sec);
debug!("🔊 TTS latency: {}ms (RTF: {:.2})", tts_time, rtf);
info!("✅ Total pipeline: {}ms", total_time);
```

**Testing**:
```bash
cargo test daemon::chat::tests --features chat

# Integration test with mock models
cargo test daemon::chat::integration_tests --features chat
```

### 5.2 Integrate into Daemon Lifecycle

**File**: `src/daemon/lifecycle.rs`

- [ ] Add `chat_engine: Option<Arc<RwLock<ChatEngine>>>` to `Lifecycle` struct

- [ ] Update `Lifecycle::start()`:
  - Check if `config.chat.enabled`
  - Load LLM and TTS models if enabled
  - Initialize `ChatEngine`
  - Store in `self.chat_engine`
  - Log chat mode initialization

- [ ] Add `Lifecycle::initialize_chat_engine()`:
  - Load LLM model based on config
  - Load TTS model based on config
  - Reuse existing STT model from dictation engine
  - Create and return `ChatEngine` instance

- [ ] Add `Lifecycle::load_llm_model()`:
  - Detect model format (GGUF vs ONNX)
  - Create appropriate backend
  - Load model with config parameters
  - Return `Box<dyn LlmRuntime>`

- [ ] Add `Lifecycle::load_tts_model()`:
  - Create Kokoro backend
  - Load model with config parameters
  - Return `Box<dyn TtsRuntime>`

- [ ] Update `Lifecycle::stop()`:
  - Unload LLM and TTS models
  - Clean up chat engine resources

**Testing**:
```bash
cargo test daemon::lifecycle::tests::test_chat_initialization --features chat
```

### 5.3 Hotkey Management

**Option A**: Extend existing hotkey handler in `DictationEngine`

**File**: `src/daemon/dictation.rs`

- [ ] Add `chat_hotkey: Option<String>` to `DictationEngine`
- [ ] Add `chat_engine: Option<Arc<RwLock<ChatEngine>>>` to `DictationEngine`

- [ ] Update `handle_hotkey_event()`:
  ```rust
  if event.key == self.config.hotkey.trigger {
      // Existing transcription logic
      self.start_dictation().await?;
  } else if Some(&event.key) == self.chat_hotkey.as_ref() {
      // New chat logic
      if let Some(chat) = &self.chat_engine {
          chat.write().start_conversation().await?;
      }
  }
  ```

**Option B**: Create separate `InputManager` (cleaner, recommended)

**File**: `src/daemon/input_manager.rs` (new)

- [ ] Create `InputManager` struct:
  - `dictation_engine: Arc<RwLock<DictationEngine>>`
  - `chat_engine: Option<Arc<RwLock<ChatEngine>>>`
  - `transcription_hotkey: String`
  - `chat_hotkey: Option<String>`

- [ ] Implement hotkey routing:
  ```rust
  pub async fn handle_hotkey(&mut self, key: &str) -> Result<()> {
      match self.classify_hotkey(key) {
          HotkeyAction::Transcription => {
              self.dictation_engine.write().start_dictation().await?;
          }
          HotkeyAction::Chat => {
              if let Some(chat) = &self.chat_engine {
                  chat.write().start_conversation().await?;
              }
          }
          HotkeyAction::Unknown => {
              warn!("Unknown hotkey: {}", key);
          }
      }
      Ok(())
  }
  ```

- [ ] Update `Lifecycle` to use `InputManager`

**Testing**:
```bash
cargo test daemon::input_manager::tests --features chat
```

---

## Phase 6: TUI Integration ⏱️ 3-4 days

### 6.1 Update TUI Configuration Types

**File**: `tui/src/data/config.ts`

- [ ] Add `ChatConfig` interface:
  ```typescript
  export interface ChatConfig {
    enabled: boolean
    hotkey: string
    llm: LlmConfig
    tts: TtsConfig
  }
  ```

- [ ] Add `LlmConfig` interface

- [ ] Add `TtsConfig` interface

- [ ] Update `VoxConfig` to include `chat: ChatConfig`

- [ ] Update `DEFAULT_CONFIG` with chat defaults

- [ ] Update TOML parser to handle chat section

### 6.2 Create Chat Configuration Panel

**File**: `tui/src/panels/chat.ts` (new)

**Implementation**:
- [ ] Create panel layout with sections:
  1. General (enabled toggle, hotkey)
  2. LLM Settings (model, device, temperature, context length)
  3. TTS Settings (model, device, voice, speech rate)
  4. System Prompt (text area)

- [ ] Add model selectors:
  - LLM model dropdown (fetch from registry)
  - TTS model dropdown (fetch from registry)
  - Device selector (auto/cpu/gpu)

- [ ] Add voice selector:
  - Fetch available voices from TTS model
  - Display voice names with descriptions

- [ ] Add parameter controls:
  - Temperature stepper (0.1 - 2.0)
  - Context length stepper (512, 1024, 2048, 4096)
  - Speech rate stepper (0.5 - 2.0)
  - Max tokens stepper (64, 128, 256, 512)

- [ ] Add system prompt editor:
  - Multi-line text area
  - Character count
  - Preset prompts dropdown (helpful, concise, creative, etc.)

- [ ] Add preload toggles for LLM and TTS

- [ ] Implement focus management (same pattern as config.ts)

- [ ] Add save/dirty state handling

**Testing**:
- [ ] Manual TUI navigation test
- [ ] Config save/load test
- [ ] Model selector population test

### 6.3 Update Main App

**File**: `tui/src/app.ts`

- [ ] Add "Chat" tab to TabSelect options:
  ```typescript
  options: [
    { name: "History", description: "Transcription history" },
    { name: "Config", description: "Settings and configuration" },
    { name: "Chat", description: "Conversational AI settings" },
  ]
  ```

- [ ] Add chat panel to tab switching logic:
  ```typescript
  if (index === 2) {
    showChat()
    if (chatPanel) chatPanel.focusFirst()
  }
  ```

- [ ] Create `showChat()` function:
  - Clear current content
  - Create and mount chat panel
  - Set focus mode

- [ ] Update status bar hints for chat tab

### 6.4 Model Download UI (Future Enhancement)

**File**: `tui/src/panels/models.ts` (new, optional)

- [ ] Create dedicated model management tab
- [ ] Show installed models by type (STT/LLM/TTS)
- [ ] Add download/delete actions
- [ ] Show model info (size, format, performance)
- [ ] Progress bars for downloads

---

## Phase 7: Testing & Validation ⏱️ 3-4 days

### 7.1 Unit Tests

**Files**: `src/models/*/tests.rs`

- [ ] LLM runtime trait tests
- [ ] TTS runtime trait tests
- [ ] Chat engine tests with mock models
- [ ] Config serialization/deserialization tests
- [ ] Audio player tests

**Run**:
```bash
cargo test --features chat
```

### 7.2 Integration Tests

**File**: `tests/chat_integration.rs` (new)

- [ ] Test full STT→LLM→TTS pipeline with mock models
- [ ] Test error handling at each stage
- [ ] Test conversation history management
- [ ] Test hotkey routing
- [ ] Test concurrent transcription + chat

**Run**:
```bash
cargo test --test chat_integration --features chat
```

### 7.3 Performance Tests

**File**: `benches/chat_pipeline.rs` (new)

- [ ] Benchmark LLM inference (tokens/sec)
- [ ] Benchmark TTS synthesis (RTF)
- [ ] Benchmark full pipeline latency
- [ ] Benchmark memory usage
- [ ] Compare CPU vs GPU performance

**Run**:
```bash
cargo bench --bench chat_pipeline --features chat
```

### 7.4 Manual Testing Checklist

**Basic Functionality**:
- [ ] Enable chat in config
- [ ] Download LLM and TTS models
- [ ] Start daemon
- [ ] Press chat hotkey → speak → hear response
- [ ] Verify conversation continuity (multi-turn)
- [ ] Clear conversation history

**Transcription Independence**:
- [ ] Use transcription hotkey → verify normal transcription still works
- [ ] Use chat hotkey → verify chat works
- [ ] Alternate between both → verify no interference

**Error Scenarios**:
- [ ] Chat with missing LLM model → graceful error
- [ ] Chat with missing TTS model → graceful error
- [ ] Interrupt during audio capture → clean cancellation
- [ ] Network disconnected (all local) → works fine
- [ ] Low memory → graceful degradation

**Performance**:
- [ ] Measure end-to-end latency (target: <1s)
- [ ] Check memory usage (target: <3GB total)
- [ ] Verify CPU usage is reasonable
- [ ] Test on all platforms (macOS, Linux, Windows)

**TUI**:
- [ ] Navigate to Chat tab
- [ ] Change LLM model → save → restart daemon
- [ ] Change TTS voice → save → restart daemon
- [ ] Edit system prompt → verify reflected in conversation
- [ ] Toggle chat enabled/disabled

---

## Phase 8: Documentation ⏱️ 2-3 days

### 8.1 User Documentation

**File**: `docs/CHAT_GUIDE.md` (new)

- [ ] Feature overview and use cases
- [ ] Getting started (enable, download models)
- [ ] Hotkey setup and usage
- [ ] Model selection guide (LLM and TTS)
- [ ] Voice selection guide
- [ ] System prompt customization
- [ ] Troubleshooting common issues
- [ ] Performance tuning tips

### 8.2 Update Existing Docs

**File**: `README.md`

- [ ] Add chat feature to features list
- [ ] Update quick start section
- [ ] Add chat hotkey to usage examples
- [ ] Update system requirements (memory, models)

**File**: `ARCHITECTURE.md`

- [ ] Document chat pipeline architecture
- [ ] Add LLM and TTS runtime sections
- [ ] Update model registry documentation
- [ ] Add performance characteristics

**File**: `INSTALLATION.md`

- [ ] Add chat feature setup instructions
- [ ] Document model download process
- [ ] Add troubleshooting for chat-specific issues

### 8.3 Developer Documentation

**File**: `DEVELOPMENT.md`

- [ ] Add chat feature development guide
- [ ] Document new traits and implementations
- [ ] Add examples for extending with new models
- [ ] Update build instructions for chat feature

**File**: `CONTRIBUTING.md`

- [ ] Add guidelines for chat-related contributions
- [ ] Document testing requirements for chat PRs
- [ ] Add model integration guidelines

---

## Phase 9: Polish & Optimization ⏱️ 2-3 days

### 9.1 Performance Optimization

- [ ] Profile LLM inference bottlenecks
- [ ] Optimize prompt formatting
- [ ] Reduce memory allocations in hot paths
- [ ] Implement model caching strategies
- [ ] Add GPU memory monitoring and warnings

### 9.2 Error Handling Refinement

- [ ] Add detailed error messages with recovery hints
- [ ] Implement retry logic for transient failures
- [ ] Add fallback strategies (text display if TTS fails)
- [ ] Log errors with structured context

### 9.3 User Experience

- [ ] Add visual/audio indicators for chat mode
- [ ] Improve hotkey registration feedback
- [ ] Add conversation turn counter
- [ ] Implement "thinking" indicator during LLM generation
- [ ] Add audio playback progress indicator

### 9.4 Security & Privacy

- [ ] Verify all processing stays local
- [ ] Add conversation history encryption (optional)
- [ ] Implement conversation auto-clear on daemon shutdown
- [ ] Add option to disable conversation logging

---

## Phase 10: Release Preparation ⏱️ 1-2 days

### 10.1 Pre-Release Checklist

- [ ] All tests passing on all platforms
- [ ] Documentation complete and reviewed
- [ ] Example config updated
- [ ] Changelog updated
- [ ] Version bumped in Cargo.toml
- [ ] Pre-built binaries tested

### 10.2 Beta Testing

- [ ] Internal testing (all platforms)
- [ ] Community beta testing (optional)
- [ ] Gather performance feedback
- [ ] Collect UX feedback
- [ ] Address critical issues

### 10.3 Release

- [ ] Tag release in git
- [ ] Build and upload binaries
- [ ] Update GitHub release notes
- [ ] Announce on social media / forums
- [ ] Update documentation website

---

## Dependencies Summary

### New Dependencies

**Cargo.toml additions**:
```toml
[dependencies]
# LLM support (GGUF models via llama.cpp)
llama-cpp-rs = { version = "0.1", optional = true }

# Note: ONNX Runtime already included for TTS
# Note: cpal already included for audio playback

[features]
default = ["whisper-cpp", "onnx", "overlay-indicator"]
chat = ["llm-gguf", "tts-onnx"]
llm-gguf = ["llama-cpp-rs"]
tts-onnx = []  # Uses existing ONNX Runtime
```

### Existing Dependencies Used

- ✅ `ort` - ONNX Runtime (for TTS and optional LLM)
- ✅ `cpal` - Audio playback
- ✅ `rubato` - Audio resampling
- ✅ `tokio` - Async runtime
- ✅ `tracing` - Logging
- ✅ `serde` - Config serialization

---

## Estimated Timeline

| Phase | Duration | Parallel Work Possible |
|-------|----------|------------------------|
| Phase 1: Config & Foundation | 2-3 days | ✅ Yes (config + registry) |
| Phase 2: Runtime Abstractions | 3-4 days | ✅ Yes (LLM + TTS traits) |
| Phase 3: Backend Implementations | 5-7 days | ✅ Yes (GGUF, ONNX, TTS) |
| Phase 4: Audio Playback | 2-3 days | ⚠️ Depends on Phase 2 |
| Phase 5: Chat Engine | 4-5 days | ⚠️ Depends on Phase 3+4 |
| Phase 6: TUI Integration | 3-4 days | ✅ Yes (parallel with Phase 5) |
| Phase 7: Testing & Validation | 3-4 days | ⚠️ Depends on Phase 5+6 |
| Phase 8: Documentation | 2-3 days | ✅ Yes (parallel with testing) |
| Phase 9: Polish & Optimization | 2-3 days | ⚠️ After testing |
| Phase 10: Release Preparation | 1-2 days | Final stage |

**Total Sequential**: ~27-38 days (~6-8 weeks)  
**Total with Parallelization**: ~20-28 days (~4-6 weeks)

---

## Success Criteria

### Must Have (MVP)
- ✅ STT→LLM→TTS pipeline functional
- ✅ Separate hotkey for chat mode
- ✅ At least one LLM backend (GGUF or ONNX)
- ✅ Kokoro TTS working with multiple voices
- ✅ Audio playback functional
- ✅ TUI configuration panel
- ✅ Basic error handling
- ✅ Documentation

### Should Have
- ✅ Both LLM backends (GGUF and ONNX)
- ✅ Conversation history management
- ✅ GPU acceleration support
- ✅ Performance optimization
- ✅ Comprehensive testing
- ✅ Cross-platform validation

### Nice to Have (Future)
- Streaming LLM responses
- Voice activity detection for chat
- Custom voice training
- Multi-language support
- Conversation export/import
- Advanced prompt templates

---

## Risk Mitigation

### Technical Risks

1. **LLM Inference Speed**
   - Risk: Too slow for interactive use
   - Mitigation: Start with small models (1-2B), optimize inference, add streaming

2. **Memory Usage**
   - Risk: Exceeds available RAM
   - Mitigation: Lazy loading, model unloading, quantization

3. **Audio Quality**
   - Risk: TTS sounds robotic or unclear
   - Mitigation: Multiple voice options, quality testing, user feedback

4. **Cross-Platform Issues**
   - Risk: Feature breaks on some platforms
   - Mitigation: Early testing on all platforms, CI/CD for all targets

### User Experience Risks

1. **Complexity**
   - Risk: Too many options, confusing setup
   - Mitigation: Sane defaults, clear documentation, guided setup

2. **Latency**
   - Risk: Users expect instant responses
   - Mitigation: Set expectations, show progress indicators, optimize

3. **Model Management**
   - Risk: Difficult to download/manage models
   - Mitigation: Built-in downloader, clear model info, auto-detection

---

## Next Steps

1. **Review this plan** with the team/maintainer
2. **Set up development environment** with required dependencies
3. **Create feature branch**: `git checkout -b feature/chat-pipeline`
4. **Start with Phase 1**: Configuration and foundation
5. **Iterate and test** each phase before moving forward
6. **Get feedback early** from beta testers
7. **Document as you go** to maintain clarity

---

## Getting Help

- Check `agent.md` for coding standards and patterns
- Review existing model implementations (`whisper_cpp.rs`, `onnx_runtime.rs`)
- Test incrementally - don't wait until the end
- Ask for code review after each major phase
- Profile performance early and often

---

**Last Updated**: 2024-01-XX  
**Status**: Planning Phase  
**Next Milestone**: Phase 1 - Configuration & Foundation