# OneVox Agent Development Guide

## Code Development Protocol - Production Standards

This document serves as the primary source of truth for coding standards, architectural decisions, and agent-specific instructions for the OneVox project.

---

## Project Overview

OneVox is a privacy-first, local speech-to-text daemon written in Rust. It provides system-wide voice dictation with:
- **Current Feature**: STT (Speech-to-Text) transcription via hotkey
- **New Feature**: STT→LLM→TTS conversational AI pipeline

### Architecture Philosophy

1. **Model-Centric Design**: Backend auto-selected based on model choice
2. **Cross-Platform First**: macOS, Linux, Windows support from day one
3. **Performance Critical**: 50-200ms latency target, minimal memory footprint
4. **Privacy First**: All processing local, no cloud dependencies
5. **Production Ready**: Robust error handling, graceful degradation, comprehensive logging

---

## Existing Codebase Patterns

### 1. Naming Conventions

- **Rust Files**: `snake_case.rs`
- **Structs/Traits**: `PascalCase` (e.g., `ModelRuntime`, `DictationEngine`)
- **Functions/Methods**: `snake_case` (e.g., `start_dictation`, `transcribe_with_model`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MIN_AUDIO_DURATION_MS`)
- **Config Fields**: `snake_case` in TOML, `snake_case` in Rust structs

### 2. Module Organization

```
src/
├── audio/           # Audio capture, processing, buffers
├── daemon/          # Daemon lifecycle, dictation engine, state
├── ipc/             # Inter-process communication
├── models/          # Model runtime abstraction, backends, registry
├── platform/        # Platform-specific implementations
├── vad/             # Voice Activity Detection
├── config.rs        # Configuration management
├── history.rs       # Transcription history
└── tui.rs           # Terminal UI launcher
```

**Key Pattern**: Feature-based modules with clear separation of concerns.

### 3. Error Handling

```rust
// Custom error type using thiserror
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Model error: {0}")]
    Model(String),
    
    #[error("Audio error: {0}")]
    Audio(String),
    
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Pattern**: Use `thiserror` for error types, always provide context.

### 4. Async Patterns

```rust
// Tokio for async runtime
use tokio::sync::{mpsc, oneshot, RwLock};
use std::sync::Arc;

// Spawn background tasks
tokio::spawn(async move {
    // Processing logic
});

// Channel communication
let (tx, mut rx) = mpsc::channel::<AudioChunk>(100);
```

**Pattern**: Tokio channels for cross-task communication, Arc<RwLock<T>> for shared state.

### 5. Logging

```rust
use tracing::{info, warn, error, debug, trace};

info!("🎤 Starting dictation");
debug!("Audio chunk: {} samples", chunk.len());
error!("Failed to load model: {}", e);
```

**Pattern**: Use tracing with emoji prefixes for visual clarity, structured logging.

### 6. Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub hotkey: HotkeyConfig,
    pub audio: AudioConfig,
    pub model: ModelConfig,
    // ... other sections
}

impl Default for Config {
    fn default() -> Self {
        // Platform-specific defaults
    }
}
```

**Pattern**: Strongly-typed config with serde, section-based organization, platform-aware defaults.

---

## STT→LLM→TTS Feature Implementation Plan

### Phase 1: Architecture & Configuration

#### 1.1 New Configuration Structures

Add to `src/config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // ... existing fields ...
    
    #[serde(default)]
    pub chat: ChatConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    /// Enable conversational AI mode
    pub enabled: bool,
    
    /// LLM model configuration
    pub llm: LlmConfig,
    
    /// TTS model configuration
    pub tts: TtsConfig,
    
    /// Hotkey for chat mode (separate from transcription hotkey)
    pub hotkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Model path/ID (e.g., "lfm2-1.2b-tool")
    pub model_path: String,
    
    /// Device: "auto", "cpu", "gpu"
    pub device: String,
    
    /// Context length
    pub context_length: usize,
    
    /// Temperature (0.0 - 2.0)
    pub temperature: f32,
    
    /// Max tokens to generate
    pub max_tokens: usize,
    
    /// System prompt
    pub system_prompt: String,
    
    /// Preload at startup
    pub preload: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    /// Model path/ID (e.g., "kokoro-82m-onnx")
    pub model_path: String,
    
    /// Device: "auto", "cpu", "gpu"
    pub device: String,
    
    /// Voice/speaker ID
    pub voice_id: String,
    
    /// Speech rate (0.5 - 2.0)
    pub speech_rate: f32,
    
    /// Preload at startup
    pub preload: bool,
}

impl Default for ChatConfig {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        let default_hotkey = "Cmd+Shift+9";
        
        #[cfg(not(target_os = "macos"))]
        let default_hotkey = "Ctrl+Shift+Alt+Space";
        
        Self {
            enabled: false,
            llm: LlmConfig::default(),
            tts: TtsConfig::default(),
            hotkey: default_hotkey.to_string(),
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model_path: "lfm2-1.2b-tool".to_string(),
            device: "auto".to_string(),
            context_length: 2048,
            temperature: 0.7,
            max_tokens: 256,
            system_prompt: "You are a helpful AI assistant. Be concise and direct.".to_string(),
            preload: false,
        }
    }
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            model_path: "kokoro-82m-onnx".to_string(),
            device: "auto".to_string(),
            voice_id: "af_heart".to_string(),
            speech_rate: 1.0,
            preload: false,
        }
    }
}
```

#### 1.2 Model Registry Updates

Add to `src/models/registry.rs`:

```rust
// Add new model type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    STT,  // Speech-to-Text
    LLM,  // Large Language Model
    TTS,  // Text-to-Speech
}

// Update ModelMetadata
pub struct ModelMetadata {
    // ... existing fields ...
    
    /// Model type
    pub model_type: ModelType,
    
    /// Requires GPU (recommended)
    pub gpu_recommended: bool,
}

// Add LLM models to registry
impl ModelRegistry {
    pub fn new() -> Self {
        let mut models = vec![
            // ... existing STT models ...
            
            // LLM Models
            ModelMetadata {
                id: "lfm2-1.2b-tool".to_string(),
                name: "Liquid LFM2 1.2B Tool (GGUF)".to_string(),
                model_type: ModelType::LLM,
                format: ModelFormat::GGUF,
                size_bytes: 1200 * 1024 * 1024, // ~1.2 GB
                hf_repo: "LiquidAI/LFM2-1.2B-Tool-GGUF".to_string(),
                files: vec!["lfm2-1.2b-tool.gguf".to_string()],
                speed_factor: 20.0, // tokens/sec
                memory_mb: 2000,
                gpu_recommended: false,
                description: "Fast, efficient LLM optimized for tool use and conversation. Good for CPU inference.".to_string(),
            },
            
            ModelMetadata {
                id: "lfm25-audio-1.5b-onnx".to_string(),
                name: "Liquid LFM2.5 Audio 1.5B (ONNX)".to_string(),
                model_type: ModelType::LLM,
                format: ModelFormat::ONNX,
                size_bytes: 1500 * 1024 * 1024, // ~1.5 GB
                hf_repo: "LiquidAI/LFM2.5-Audio-1.5B-ONNX".to_string(),
                files: vec![
                    "model.onnx".to_string(),
                    "config.json".to_string(),
                ],
                speed_factor: 15.0,
                memory_mb: 3000,
                gpu_recommended: true,
                description: "Audio-aware LLM with better understanding of speech context. Optimized for ONNX Runtime.".to_string(),
            },
            
            // TTS Models
            ModelMetadata {
                id: "kokoro-82m-onnx".to_string(),
                name: "Kokoro TTS 82M (ONNX)".to_string(),
                model_type: ModelType::TTS,
                format: ModelFormat::ONNX,
                size_bytes: 82 * 1024 * 1024, // ~82 MB
                hf_repo: "onnx-community/Kokoro-82M-ONNX".to_string(),
                files: vec![
                    "model.onnx".to_string(),
                    "config.json".to_string(),
                    "vocab.json".to_string(),
                ],
                speed_factor: 50.0, // RTF (Real-Time Factor)
                memory_mb: 300,
                gpu_recommended: false,
                description: "Fast, natural-sounding TTS. Multiple voice options. CPU-friendly.".to_string(),
            },
        ];
        
        Self { models }
    }
}
```

### Phase 2: Model Runtime Abstraction

#### 2.1 LLM Runtime Trait

Create `src/models/llm_runtime.rs`:

```rust
//! LLM Runtime Trait
//!
//! Abstract interface for Large Language Model backends.

use crate::Result;

/// LLM generation result
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// Generated text
    pub text: String,
    
    /// Tokens generated
    pub tokens: usize,
    
    /// Generation time in milliseconds
    pub generation_time_ms: u64,
    
    /// Tokens per second
    pub tokens_per_second: f32,
}

/// LLM runtime configuration
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Path to model file
    pub model_path: String,
    
    /// Use GPU acceleration
    pub use_gpu: bool,
    
    /// Context length
    pub context_length: usize,
    
    /// Temperature (0.0 - 2.0)
    pub temperature: f32,
    
    /// Max tokens to generate
    pub max_tokens: usize,
    
    /// System prompt
    pub system_prompt: String,
}

/// Conversation message
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// LLM runtime trait
pub trait LlmRuntime: Send + Sync {
    /// Load the model
    fn load(&mut self, config: LlmConfig) -> Result<()>;
    
    /// Check if model is loaded
    fn is_loaded(&self) -> bool;
    
    /// Generate response from conversation history
    fn generate(&mut self, messages: &[ChatMessage]) -> Result<LlmResponse>;
    
    /// Generate response with streaming (optional)
    fn generate_stream(&mut self, messages: &[ChatMessage]) -> Result<LlmResponseStream> {
        // Default: non-streaming
        let response = self.generate(messages)?;
        Ok(LlmResponseStream::Complete(response))
    }
    
    /// Unload the model
    fn unload(&mut self);
    
    /// Get model name
    fn name(&self) -> &str;
}

/// Streaming response
pub enum LlmResponseStream {
    Complete(LlmResponse),
    // Future: Streaming(Box<dyn Stream<Item = String>>),
}
```

#### 2.2 TTS Runtime Trait

Create `src/models/tts_runtime.rs`:

```rust
//! TTS Runtime Trait
//!
//! Abstract interface for Text-to-Speech backends.

use crate::Result;

/// TTS synthesis result
#[derive(Debug, Clone)]
pub struct TtsSynthesis {
    /// Audio samples (f32, mono, 22050 Hz typical)
    pub samples: Vec<f32>,
    
    /// Sample rate
    pub sample_rate: u32,
    
    /// Synthesis time in milliseconds
    pub synthesis_time_ms: u64,
    
    /// Audio duration in milliseconds
    pub audio_duration_ms: u64,
    
    /// Real-time factor (< 1.0 is faster than real-time)
    pub rtf: f32,
}

/// TTS runtime configuration
#[derive(Debug, Clone)]
pub struct TtsConfig {
    /// Path to model file
    pub model_path: String,
    
    /// Use GPU acceleration
    pub use_gpu: bool,
    
    /// Voice/speaker ID
    pub voice_id: String,
    
    /// Speech rate (0.5 - 2.0)
    pub speech_rate: f32,
}

/// TTS runtime trait
pub trait TtsRuntime: Send + Sync {
    /// Load the model
    fn load(&mut self, config: TtsConfig) -> Result<()>;
    
    /// Check if model is loaded
    fn is_loaded(&self) -> bool;
    
    /// Synthesize speech from text
    fn synthesize(&mut self, text: &str) -> Result<TtsSynthesis>;
    
    /// List available voices
    fn list_voices(&self) -> Vec<VoiceInfo>;
    
    /// Unload the model
    fn unload(&mut self);
    
    /// Get model name
    fn name(&self) -> &str;
}

/// Voice information
#[derive(Debug, Clone)]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
    pub language: String,
    pub gender: Option<String>,
}
```

### Phase 3: Backend Implementations

#### 3.1 GGUF LLM Backend (llama.cpp style)

Create `src/models/llm_gguf.rs`:

```rust
//! GGUF LLM Backend
//!
//! Uses llama.cpp Rust bindings for GGUF model inference.

use super::llm_runtime::*;
use crate::Result;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct LlmGguf {
    model: Option<Arc<RwLock<LlamaModel>>>,
    config: Option<LlmConfig>,
}

impl LlmGguf {
    pub fn new() -> Self {
        Self {
            model: None,
            config: None,
        }
    }
}

impl LlmRuntime for LlmGguf {
    fn load(&mut self, config: LlmConfig) -> Result<()> {
        // Implementation: Load GGUF model using llama.cpp bindings
        // Similar pattern to whisper_cpp.rs
        todo!("Implement GGUF model loading")
    }
    
    fn is_loaded(&self) -> bool {
        self.model.is_some()
    }
    
    fn generate(&mut self, messages: &[ChatMessage]) -> Result<LlmResponse> {
        // Implementation: Format prompt, run inference, decode tokens
        todo!("Implement text generation")
    }
    
    fn unload(&mut self) {
        self.model = None;
    }
    
    fn name(&self) -> &str {
        "llm-gguf"
    }
}
```

#### 3.2 ONNX LLM Backend

Create `src/models/llm_onnx.rs`:

```rust
//! ONNX LLM Backend
//!
//! Uses ONNX Runtime for model inference (LFM2.5-Audio-1.5B-ONNX).

use super::llm_runtime::*;
use crate::Result;
use ort::{Session, Value};

pub struct LlmOnnx {
    session: Option<Session>,
    config: Option<LlmConfig>,
}

impl LlmOnnx {
    pub fn new() -> Self {
        Self {
            session: None,
            config: None,
        }
    }
}

impl LlmRuntime for LlmOnnx {
    fn load(&mut self, config: LlmConfig) -> Result<()> {
        // Implementation: Load ONNX model
        // Similar pattern to onnx_runtime.rs
        todo!("Implement ONNX LLM loading")
    }
    
    fn is_loaded(&self) -> bool {
        self.session.is_some()
    }
    
    fn generate(&mut self, messages: &[ChatMessage]) -> Result<LlmResponse> {
        // Implementation: Run ONNX inference
        todo!("Implement ONNX generation")
    }
    
    fn unload(&mut self) {
        self.session = None;
    }
    
    fn name(&self) -> &str {
        "llm-onnx"
    }
}
```

#### 3.3 Kokoro TTS Backend

Create `src/models/tts_kokoro.rs`:

```rust
//! Kokoro TTS Backend
//!
//! ONNX-based TTS using Kokoro-82M model.

use super::tts_runtime::*;
use crate::Result;
use ort::{Session, Value};

pub struct TtsKokoro {
    session: Option<Session>,
    config: Option<TtsConfig>,
    voices: Vec<VoiceInfo>,
}

impl TtsKokoro {
    pub fn new() -> Self {
        Self {
            session: None,
            config: None,
            voices: Self::initialize_voices(),
        }
    }
    
    fn initialize_voices() -> Vec<VoiceInfo> {
        // Kokoro voice presets
        vec![
            VoiceInfo {
                id: "af_heart".to_string(),
                name: "Heart (Female, American)".to_string(),
                language: "en-US".to_string(),
                gender: Some("female".to_string()),
            },
            VoiceInfo {
                id: "af_sky".to_string(),
                name: "Sky (Female, American)".to_string(),
                language: "en-US".to_string(),
                gender: Some("female".to_string()),
            },
            VoiceInfo {
                id: "am_adam".to_string(),
                name: "Adam (Male, American)".to_string(),
                language: "en-US".to_string(),
                gender: Some("male".to_string()),
            },
            // Add more voices
        ]
    }
}

impl TtsRuntime for TtsKokoro {
    fn load(&mut self, config: TtsConfig) -> Result<()> {
        // Implementation: Load Kokoro ONNX model
        todo!("Implement Kokoro TTS loading")
    }
    
    fn is_loaded(&self) -> bool {
        self.session.is_some()
    }
    
    fn synthesize(&mut self, text: &str) -> Result<TtsSynthesis> {
        // Implementation: Run TTS synthesis
        todo!("Implement TTS synthesis")
    }
    
    fn list_voices(&self) -> Vec<VoiceInfo> {
        self.voices.clone()
    }
    
    fn unload(&mut self) {
        self.session = None;
    }
    
    fn name(&self) -> &str {
        "tts-kokoro"
    }
}
```

### Phase 4: Chat Engine

#### 4.1 Create Chat Engine

Create `src/daemon/chat.rs`:

```rust
//! Chat Engine
//!
//! Manages STT→LLM→TTS conversational pipeline.

use crate::audio::{AudioEngine, CaptureConfig};
use crate::config::ChatConfig;
use crate::models::{LlmRuntime, TtsRuntime, ModelRuntime, ChatMessage, MessageRole};
use crate::platform::audio::AudioPlayer;
use crate::Result;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

pub struct ChatEngine {
    config: ChatConfig,
    
    // Models
    stt_model: Arc<RwLock<Box<dyn ModelRuntime>>>,
    llm_model: Arc<RwLock<Box<dyn LlmRuntime>>>,
    tts_model: Arc<RwLock<Box<dyn TtsRuntime>>>,
    
    // Audio
    audio_engine: Arc<AudioEngine>,
    audio_player: Arc<AudioPlayer>,
    
    // Conversation state
    conversation_history: Vec<ChatMessage>,
    
    // State
    is_active: Arc<std::sync::atomic::AtomicBool>,
}

impl ChatEngine {
    pub fn new(
        config: ChatConfig,
        stt_model: Arc<RwLock<Box<dyn ModelRuntime>>>,
        llm_model: Arc<RwLock<Box<dyn LlmRuntime>>>,
        tts_model: Arc<RwLock<Box<dyn TtsRuntime>>>,
    ) -> Self {
        let audio_engine = Arc::new(AudioEngine::new());
        let audio_player = Arc::new(AudioPlayer::new());
        
        // Initialize with system prompt
        let conversation_history = vec![
            ChatMessage {
                role: MessageRole::System,
                content: config.llm.system_prompt.clone(),
            }
        ];
        
        Self {
            config,
            stt_model,
            llm_model,
            tts_model,
            audio_engine,
            audio_player,
            conversation_history,
            is_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    
    pub async fn start_conversation(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;
        
        if self.is_active.load(Ordering::SeqCst) {
            return Ok(());
        }
        
        info!("💬 Starting conversation mode");
        self.is_active.store(true, Ordering::SeqCst);
        
        // 1. Capture audio (STT)
        info!("🎤 Listening...");
        let audio_samples = self.capture_audio().await?;
        
        // 2. Transcribe
        info!("📝 Transcribing...");
        let transcription = {
            let mut model = self.stt_model.write();
            model.transcribe(&audio_samples, 16000)?
        };
        
        info!("User: {}", transcription.text);
        
        // Add user message to history
        self.conversation_history.push(ChatMessage {
            role: MessageRole::User,
            content: transcription.text.clone(),
        });
        
        // 3. Generate LLM response
        info!("🤖 Thinking...");
        let llm_response = {
            let mut model = self.llm_model.write();
            model.generate(&self.conversation_history)?
        };
        
        info!("Assistant: {}", llm_response.text);
        
        // Add assistant message to history
        self.conversation_history.push(ChatMessage {
            role: MessageRole::Assistant,
            content: llm_response.text.clone(),
        });
        
        // 4. Synthesize speech (TTS)
        info!("🔊 Synthesizing speech...");
        let tts_synthesis = {
            let mut model = self.tts_model.write();
            model.synthesize(&llm_response.text)?
        };
        
        // 5. Play audio
        info!("▶️  Playing response...");
        self.audio_player.play(&tts_synthesis.samples, tts_synthesis.sample_rate).await?;
        
        info!("✅ Conversation turn complete");
        self.is_active.store(false, Ordering::SeqCst);
        
        Ok(())
    }
    
    async fn capture_audio(&self) -> Result<Vec<f32>> {
        // Similar to dictation.rs audio capture
        // But with timeout or explicit stop signal
        todo!("Implement audio capture for chat")
    }
    
    pub fn clear_history(&mut self) {
        let system_prompt = self.conversation_history[0].clone();
        self.conversation_history = vec![system_prompt];
        info!("🗑️  Conversation history cleared");
    }
}
```

#### 4.2 Integrate into Daemon

Update `src/daemon/lifecycle.rs`:

```rust
pub struct Lifecycle {
    // ... existing fields ...
    
    chat_engine: Option<Arc<RwLock<ChatEngine>>>,
}

impl Lifecycle {
    pub async fn start(&mut self) -> Result<()> {
        // ... existing startup code ...
        
        // Initialize chat engine if enabled
        if self.config.chat.enabled {
            info!("💬 Initializing chat mode");
            let chat_engine = self.initialize_chat_engine().await?;
            self.chat_engine = Some(Arc::new(RwLock::new(chat_engine)));
        }
        
        // ... rest of startup ...
    }
    
    async fn initialize_chat_engine(&self) -> Result<ChatEngine> {
        // Load models
        let stt_model = /* existing STT model */;
        let llm_model = self.load_llm_model().await?;
        let tts_model = self.load_tts_model().await?;
        
        Ok(ChatEngine::new(
            self.config.chat.clone(),
            stt_model,
            llm_model,
            tts_model,
        ))
    }
}
```

### Phase 5: TUI Integration

#### 5.1 Add Chat Tab

Update `tui/src/app.ts`:

```typescript
// Add chat tab to options
options: [
  { name: "History", description: "Transcription history" },
  { name: "Config", description: "Settings and configuration" },
  { name: "Chat", description: "Conversational AI settings" },  // NEW
]
```

#### 5.2 Create Chat Panel

Create `tui/src/panels/chat.ts`:

```typescript
/**
 * Chat configuration panel
 */

export function createChatPanel(
  renderer: CliRenderer,
  state: AppState,
  callbacks: {
    onDirty: () => void
    onSaved: () => void
    onStatusMessage: (msg: string) => void
    onEscape: () => void
  }
): ConfigPanelInstance {
  const config = state.config
  const theme = state.theme
  
  // Root container
  const root = new BoxRenderable(/* ... */)
  
  // Chat enabled toggle
  const chatEnabled = createToggleField({
    label: "Enable Conversational AI",
    value: config.chat.enabled,
    onChange: (val) => {
      config.chat.enabled = val
      callbacks.onDirty()
    }
  })
  
  // LLM model selector
  const llmModels = ["lfm2-1.2b-tool", "lfm25-audio-1.5b-onnx"]
  const llmModelField = createSelectField({
    label: "LLM Model",
    options: llmModels.map(id => ({ name: id })),
    selectedIndex: llmModels.indexOf(config.chat.llm.model_path),
    onChange: (idx) => {
      config.chat.llm.model_path = llmModels[idx]
      callbacks.onDirty()
    }
  })
  
  // LLM device selector
  const llmDeviceField = createSelectField({
    label: "LLM Device",
    options: [
      { name: "auto", description: "Auto-detect" },
      { name: "cpu", description: "CPU only" },
      { name: "gpu", description: "GPU acceleration" },
    ],
    selectedIndex: ["auto", "cpu", "gpu"].indexOf(config.chat.llm.device),
    onChange: (idx) => {
      config.chat.llm.device = ["auto", "cpu", "gpu"][idx]
      callbacks.onDirty()
    }
  })
  
  // TTS model selector
  const ttsModels = ["kokoro-82m-onnx"]
  const ttsModelField = createSelectField({
    label: "TTS Model",
    options: ttsModels.map(id => ({ name: id })),
    selectedIndex: ttsModels.indexOf(config.chat.tts.model_path),
    onChange: (idx) => {
      config.chat.tts.model_path = ttsModels[idx]
      callbacks.onDirty()
    }
  })
  
  // Voice selector (dynamically loaded from TTS model)
  const voiceField = createSelectField({
    label: "Voice",
    options: [
      { name: "af_heart", description: "Heart (Female)" },
      { name: "af_sky", description: "Sky (Female)" },
      { name: "am_adam", description: "Adam (Male)" },
    ],
    selectedIndex: 0,
    onChange: (idx) => {
      config.chat.tts.voice_id = ["af_heart", "af_sky", "am_adam"][idx]
      callbacks.onDirty()
    }
  })
  
  // Temperature slider
  const tempValues = [0.1, 0.3, 0.5, 0.7, 0.9, 1.0, 1.2, 1.5]
  const tempStepper = createStepperField({
    label: "Temperature",
    values: tempValues.map(v => v.toString()),
    selectedIndex: tempValues.indexOf(config.chat.llm.temperature),
    onChange: (idx) => {
      config.chat.llm.temperature = tempValues[idx]
      callbacks.onDirty()
    }
  })
  
  // System prompt editor (text area)
  const systemPromptField = createTextAreaField({
    label: "System Prompt",
    value: config.chat.llm.system_prompt,
    onChange: (val) => {
      config.chat.llm.system_prompt = val
      callbacks.onDirty()
    }
  })
  
  // Hotkey capture
  const chatHotkeyField = createKeyCaptureField({
    label: "Chat Hotkey",
    value: config.chat.hotkey,
    onChange: (val) => {
      config.chat.hotkey = val
      callbacks.onDirty()
    }
  })
  
  // Assemble sections...
  
  return {
    root,
    focusFirst: () => { /* ... */ },
    // ... other methods
  }
}
```

### Phase 6: Hotkey Management

#### 6.1 Dual Hotkey Support

Update `src/daemon/dictation.rs` (or create `src/daemon/input_manager.rs`):

```rust
pub struct InputManager {
    transcription_hotkey: String,
    chat_hotkey: Option<String>,
    mode: InputMode,
}

enum InputMode {
    Transcription,
    Chat,
}

impl InputManager {
    pub fn handle_hotkey(&mut self, hotkey: &str) -> HotkeyAction {
        if hotkey == self.transcription_hotkey {
            HotkeyAction::Transcription
        } else if Some(hotkey.to_string()) == self.chat_hotkey {
            HotkeyAction::Chat
        } else {
            HotkeyAction::Unknown
        }
    }
}

pub enum HotkeyAction {
    Transcription,
    Chat,
    Unknown,
}
```

### Phase 7: Audio Playback

#### 7.1 Create Audio Player

Create `src/platform/audio/player.rs`:

```rust
//! Audio Playback
//!
//! Cross-platform audio output for TTS.

use crate::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};

pub struct AudioPlayer {
    device: Device,
    config: StreamConfig,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| crate::Error::Audio("No output device".to_string()))?;
        
        let config = device.default_output_config()
            .map_err(|e| crate::Error::Audio(format!("Output config error: {}", e)))?
            .into();
        
        Ok(Self { device, config })
    }
    
    pub async fn play(&self, samples: &[f32], sample_rate: u32) -> Result<()> {
        // Implementation: Create output stream and play samples
        // Handle sample rate conversion if needed
        // Wait for playback completion
        todo!("Implement audio playback")
    }
}
```

### Phase 8: Dependencies

#### 8.1 Update Cargo.toml

```toml
[dependencies]
# ... existing dependencies ...

# LLM support (GGUF models)
llama-cpp-rs = { version = "0.1", optional = true }
llama-cpp-sys = { version = "0.1", optional = true }

[features]
default = ["whisper-cpp", "onnx", "overlay-indicator"]

# ... existing features ...

# Chat features
chat = ["llm-gguf", "tts-onnx"]
llm-gguf = ["llama-cpp-rs", "llama-cpp-sys"]
tts-onnx = []  # Uses existing ONNX Runtime
```

---

## Implementation Guidelines

### Performance Requirements

1. **Latency Targets**:
   - STT: 50-200ms (existing)
   - LLM: 100-500ms for response generation (20-50 tokens/sec)
   - TTS: 50-200ms (RTF < 0.5)
   - Total pipeline: < 1 second for typical interaction

2. **Memory Budget**:
   - STT model: ~100-500MB (existing)
   - LLM model: ~1-2GB
   - TTS model: ~100-300MB
   - Total runtime: ~2-3GB with all models loaded

3. **CPU/GPU Strategy**:
   - STT: CPU default, GPU optional (existing)
   - LLM: CPU for small models (1-2B), GPU recommended for larger
   - TTS: CPU sufficient (Kokoro is fast)

### Error Handling

1. **Graceful Degradation**:
   - If LLM fails, return error message via TTS
   - If TTS fails, fall back to text display
   - If STT fails, retry or prompt user

2. **Model Loading**:
   - Lazy loading by default
   - Preload option for low first-use latency
   - Clear error messages for missing models

3. **Resource Management**:
   - Automatic model unloading on low memory
   - Configurable model caching
   - GPU memory monitoring

### Testing Strategy

1. **Unit Tests**:
   - Each runtime implementation
   - Config parsing and validation
   - Audio capture and playback

2. **Integration Tests**:
   - Full STT→LLM→TTS pipeline
   - Hotkey handling
   - Model switching

3. **Performance Tests**:
   - Latency benchmarks
   - Memory usage profiling
   - Concurrent operation stress tests

### Logging & Debugging

```rust
// Use structured logging throughout
info!("💬 Chat turn started");
debug!("STT latency: {}ms", transcription.processing_time_ms);
debug!("LLM tokens/sec: {}", llm_response.tokens_per_second);
debug!("TTS RTF: {:.2}", tts_synthesis.rtf);
info!("✅ Full pipeline: {}ms", total_time);
```

---

## Migration Path

### Phase 1: Foundation (Week 1-2)
- [ ] Add config structures
- [ ] Update model registry
- [ ] Create runtime traits

### Phase 2: Backends (Week 3-4)
- [ ] Implement LLM GGUF backend
- [ ] Implement TTS Kokoro backend
- [ ] Basic integration tests

### Phase 3: Integration (Week 5-6)
- [ ] Create ChatEngine
- [ ] Integrate into daemon lifecycle
- [ ] Dual hotkey support

### Phase 4: TUI & UX (Week 7-8)
- [ ] Add Chat configuration panel
- [ ] Model download UI
- [ ] Voice selection UI

### Phase 5: Polish (Week 9-10)
- [ ] Performance optimization
- [ ] Error handling refinement
- [ ] Documentation
- [ ] End-to-end testing

---

## Code Quality Standards

### 1. Always Use Existing Patterns

- Follow naming conventions exactly
- Use same error handling approach
- Match logging style (emojis + structured logs)
- Replicate config structure patterns

### 2. Performance First

- Profile before optimizing
- Minimize allocations in hot paths
- Use `Arc` and `RwLock` for shared state
- Prefer channels over polling

### 3. Cross-Platform

- Test on macOS, Linux, Windows
- Use platform-agnostic crates
- Provide platform-specific fallbacks

### 4. Documentation

- Document all public APIs
- Include examples in doc comments
- Keep ARCHITECTURE.md updated
- Add inline comments for complex logic

### 5. Error Context

```rust
// Bad
model.load(path)?;

// Good
model.load(path)
    .map_err(|e| Error::Model(format!("Failed to load LLM from {}: {}", path, e)))?;
```

---

## Security Considerations

1. **Model Files**:
   - Verify checksums after download
   - Validate file formats before loading
   - Sandboxed model execution (future)

2. **User Data**:
   - Conversation history stored locally only
   - Optional history clearing
   - No telemetry by default

3. **Permissions**:
   - Request only necessary permissions
   - Clear permission requirements in docs
   - Graceful degradation without permissions

---

## Future Enhancements

1. **Advanced Features**:
   - Conversation memory with vector DB
   - Multi-turn context optimization
   - Voice cloning support
   - Real-time translation mode

2. **Model Ecosystem**:
   - Support more LLM formats (PyTorch, CoreML)
   - Plugin system for custom models
   - Model quantization tools

3. **Performance**:
   - Speculative decoding for LLMs
   - Parallel STT+LLM processing
   - Edge device optimization

---

## Questions & Decisions

### Model Selection Rationale

**LLM Choice: LFM2-1.2B-Tool vs LFM2.5-Audio-1.5B**
- Start with LFM2-1.2B-Tool (GGUF) for broader compatibility
- LFM2.5-Audio-1.5B (ONNX) as advanced option for users with ONNX support
- Both are small enough for CPU inference

**TTS Choice: Kokoro-82M**
- Fast (RTF < 0.5 on CPU)
- Good quality, natural voices
- ONNX format (reuse existing ONNX Runtime)
- Multiple voice options

### Architecture Decisions

**Q: Single hotkey or dual hotkey?**
A: Dual hotkey - keeps transcription and chat separate and intuitive.

**Q: Model preloading?**
A: Optional, default OFF to save memory. Power users can enable.

**Q: Conversation persistence?**
A: In-memory only by default. Optional save/load in future.

**Q: Streaming LLM responses?**
A: Phase 2 feature. Start with complete responses for simplicity.

---

## Contact & Support

For questions about this implementation:
1. Check existing code patterns first
2. Review ARCHITECTURE.md for design philosophy
3. Consult this agent.md for chat-specific guidance
4. Test changes on all platforms before committing

**Priority**: Maintain existing functionality. The STT pipeline must remain fast and reliable.

---

## Appendix: Example Config

```toml
# config.toml with chat enabled

[daemon]
auto_start = true
log_level = "info"

[hotkey]
trigger = "Cmd+Shift+0"
mode = "push-to-talk"

[model]
model_path = "ggml-base.en"
device = "auto"
preload = true

# NEW: Chat configuration
[chat]
enabled = true
hotkey = "Cmd+Shift+9"

[chat.llm]
model_path = "lfm2-1.2b-tool"
device = "auto"
context_length = 2048
temperature = 0.7
max_tokens = 256
system_prompt = "You are a helpful AI assistant. Be concise and direct."
preload = false

[chat.tts]
model_path = "kokoro-82m-onnx"
device = "auto"
voice_id = "af_heart"
speech_rate = 1.0
preload = false
```

---

**End of Agent Development Guide**