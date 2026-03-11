# Chat Feature Quick Start Guide

> **TL;DR**: Add STT→LLM→TTS conversational AI to OneVox in 10 steps.

## Before You Start

### Prerequisites

✅ Rust 1.93+  
✅ Familiarity with OneVox codebase  
✅ Read `agent.md` for coding standards  
✅ Review `ARCHITECTURE.md` for design patterns  

### Recommended Reading Order

1. `agent.md` - Coding standards and architecture overview
2. `CHAT_IMPLEMENTATION.md` - Detailed checklist
3. This file - Quick start guide

---

## 10-Step Implementation

### Step 1: Add Configuration (30 min)

**File**: `src/config.rs`

```rust
// Add to Config struct
#[serde(default)]
pub chat: ChatConfig,

// Add new structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    pub enabled: bool,
    pub hotkey: String,
    pub llm: LlmConfig,
    pub tts: TtsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub model_path: String,
    pub device: String,
    pub temperature: f32,
    pub max_tokens: usize,
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub model_path: String,
    pub device: String,
    pub voice_id: String,
    pub speech_rate: f32,
}

// Add Default implementations
impl Default for ChatConfig { /* ... */ }
impl Default for LlmConfig { /* ... */ }
impl Default for TtsConfig { /* ... */ }
```

**Test**: `cargo test config::tests`

---

### Step 2: Update Model Registry (45 min)

**File**: `src/models/registry.rs`

```rust
// Add model type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    STT,
    LLM,
    TTS,
}

// Add to ModelMetadata
pub model_type: ModelType,

// Add models in ModelRegistry::new()
ModelMetadata {
    id: "lfm2-1.2b-tool".to_string(),
    name: "Liquid LFM2 1.2B".to_string(),
    model_type: ModelType::LLM,
    format: ModelFormat::GGUF,
    size_bytes: 1200 * 1024 * 1024,
    hf_repo: "LiquidAI/LFM2-1.2B-Tool-GGUF".to_string(),
    // ...
},

ModelMetadata {
    id: "kokoro-82m-onnx".to_string(),
    name: "Kokoro TTS 82M".to_string(),
    model_type: ModelType::TTS,
    format: ModelFormat::ONNX,
    size_bytes: 82 * 1024 * 1024,
    hf_repo: "onnx-community/Kokoro-82M-ONNX".to_string(),
    // ...
},
```

**Test**: `cargo test registry::tests`

---

### Step 3: Define Runtime Traits (1 hour)

**Create**: `src/models/llm_runtime.rs`

```rust
pub trait LlmRuntime: Send + Sync {
    fn load(&mut self, config: LlmConfig) -> Result<()>;
    fn is_loaded(&self) -> bool;
    fn generate(&mut self, messages: &[ChatMessage]) -> Result<LlmResponse>;
    fn unload(&mut self);
    fn name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub text: String,
    pub tokens: usize,
    pub generation_time_ms: u64,
    pub tokens_per_second: f32,
}

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
```

**Create**: `src/models/tts_runtime.rs`

```rust
pub trait TtsRuntime: Send + Sync {
    fn load(&mut self, config: TtsConfig) -> Result<()>;
    fn is_loaded(&self) -> bool;
    fn synthesize(&mut self, text: &str) -> Result<TtsSynthesis>;
    fn list_voices(&self) -> Vec<VoiceInfo>;
    fn unload(&mut self);
    fn name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct TtsSynthesis {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub synthesis_time_ms: u64,
    pub rtf: f32,
}
```

**Update**: `src/models.rs`

```rust
pub mod llm_runtime;
pub mod tts_runtime;

pub use llm_runtime::*;
pub use tts_runtime::*;
```

---

### Step 4: Implement LLM Backend (2-3 days)

**Create**: `src/models/llm_gguf.rs`

```rust
use super::llm_runtime::*;
use llama_cpp_rs::{LlamaModel, LlamaContext};

pub struct LlmGguf {
    model: Option<LlamaModel>,
    context: Option<LlamaContext>,
    config: Option<LlmConfig>,
}

impl LlmRuntime for LlmGguf {
    fn load(&mut self, config: LlmConfig) -> Result<()> {
        info!("🤖 Loading GGUF LLM: {}", config.model_path);
        
        // Load model from path
        let model = LlamaModel::load_from_file(&config.model_path, /* params */)?;
        
        // Create context
        let context = model.create_context(/* params */)?;
        
        self.model = Some(model);
        self.context = Some(context);
        self.config = Some(config);
        
        info!("✅ LLM loaded successfully");
        Ok(())
    }
    
    fn generate(&mut self, messages: &[ChatMessage]) -> Result<LlmResponse> {
        let start = Instant::now();
        
        // Format prompt from messages
        let prompt = self.format_chat_prompt(messages)?;
        
        // Run inference
        let tokens = self.context.as_mut().unwrap().generate(
            &prompt,
            self.config.as_ref().unwrap().max_tokens,
        )?;
        
        // Decode tokens to text
        let text = self.decode_tokens(&tokens)?;
        
        let elapsed = start.elapsed().as_millis() as u64;
        let tokens_per_sec = tokens.len() as f32 / (elapsed as f32 / 1000.0);
        
        Ok(LlmResponse {
            text,
            tokens: tokens.len(),
            generation_time_ms: elapsed,
            tokens_per_second: tokens_per_sec,
        })
    }
    
    // ... other trait methods
}
```

**Update**: `Cargo.toml`

```toml
[dependencies]
llama-cpp-rs = { version = "0.1", optional = true }

[features]
chat = ["llm-gguf", "tts-onnx"]
llm-gguf = ["llama-cpp-rs"]
```

**Test**: `cargo test --features chat models::llm_gguf::tests`

---

### Step 5: Implement TTS Backend (1-2 days)

**Create**: `src/models/tts_kokoro.rs`

```rust
use super::tts_runtime::*;
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
            voices: vec![
                VoiceInfo {
                    id: "af_heart".to_string(),
                    name: "Heart (Female)".to_string(),
                    language: "en-US".to_string(),
                    gender: Some("female".to_string()),
                },
                // ... more voices
            ],
        }
    }
}

impl TtsRuntime for TtsKokoro {
    fn load(&mut self, config: TtsConfig) -> Result<()> {
        info!("🔊 Loading Kokoro TTS: {}", config.model_path);
        
        // Load ONNX model
        let session = Session::builder()?
            .with_model_from_file(&config.model_path)?;
        
        self.session = Some(session);
        self.config = Some(config);
        
        info!("✅ TTS loaded successfully");
        Ok(())
    }
    
    fn synthesize(&mut self, text: &str) -> Result<TtsSynthesis> {
        let start = Instant::now();
        
        // Preprocess text
        let phonemes = self.text_to_phonemes(text)?;
        
        // Run ONNX inference
        let outputs = self.session.as_ref().unwrap().run(vec![
            Value::from_array(phonemes)?,
        ])?;
        
        // Extract audio samples
        let samples: Vec<f32> = outputs[0].extract_tensor()?;
        
        let elapsed = start.elapsed().as_millis() as u64;
        let duration_ms = (samples.len() as f32 / 22050.0 * 1000.0) as u64;
        let rtf = elapsed as f32 / duration_ms as f32;
        
        Ok(TtsSynthesis {
            samples,
            sample_rate: 22050,
            synthesis_time_ms: elapsed,
            audio_duration_ms: duration_ms,
            rtf,
        })
    }
    
    fn list_voices(&self) -> Vec<VoiceInfo> {
        self.voices.clone()
    }
    
    // ... other trait methods
}
```

**Test**: `cargo test --features chat models::tts_kokoro::tests`

---

### Step 6: Add Audio Playback (1 day)

**Create**: `src/platform/audio/player.rs`

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioPlayer {
    device: Device,
    config: StreamConfig,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| Error::Audio("No output device".to_string()))?;
        
        let config = device.default_output_config()?.into();
        
        Ok(Self { device, config })
    }
    
    pub async fn play(&self, samples: &[f32], sample_rate: u32) -> Result<()> {
        info!("▶️  Playing audio: {:.2}s", samples.len() as f32 / sample_rate as f32);
        
        // Resample if needed
        let resampled = if sample_rate != self.config.sample_rate.0 {
            self.resample(samples, sample_rate, self.config.sample_rate.0)?
        } else {
            samples.to_vec()
        };
        
        // Create output stream
        let stream = self.device.build_output_stream(
            &self.config,
            move |data: &mut [f32], _: &_| {
                // Write samples to output buffer
                data.copy_from_slice(&resampled);
            },
            |err| error!("Audio playback error: {}", err),
        )?;
        
        stream.play()?;
        
        // Wait for completion
        tokio::time::sleep(Duration::from_millis(duration_ms)).await;
        
        Ok(())
    }
}
```

**Update**: `src/platform/audio/mod.rs`

```rust
pub mod player;
pub use player::AudioPlayer;
```

---

### Step 7: Create Chat Engine (2-3 days)

**Create**: `src/daemon/chat.rs`

```rust
pub struct ChatEngine {
    config: ChatConfig,
    stt_model: Arc<RwLock<Box<dyn ModelRuntime>>>,
    llm_model: Arc<RwLock<Box<dyn LlmRuntime>>>,
    tts_model: Arc<RwLock<Box<dyn TtsRuntime>>>,
    audio_engine: Arc<AudioEngine>,
    audio_player: Arc<AudioPlayer>,
    conversation_history: Vec<ChatMessage>,
    is_active: Arc<AtomicBool>,
}

impl ChatEngine {
    pub async fn start_conversation(&mut self) -> Result<()> {
        info!("💬 Chat turn started");
        
        // 1. Capture audio
        let audio = self.capture_audio().await?;
        
        // 2. Transcribe (STT)
        let transcript = self.stt_model.write().transcribe(&audio, 16000)?;
        info!("User: {}", transcript.text);
        
        self.conversation_history.push(ChatMessage {
            role: MessageRole::User,
            content: transcript.text,
        });
        
        // 3. Generate response (LLM)
        let response = self.llm_model.write().generate(&self.conversation_history)?;
        info!("Assistant: {}", response.text);
        
        self.conversation_history.push(ChatMessage {
            role: MessageRole::Assistant,
            content: response.text.clone(),
        });
        
        // 4. Synthesize speech (TTS)
        let audio = self.tts_model.write().synthesize(&response.text)?;
        
        // 5. Play audio
        self.audio_player.play(&audio.samples, audio.sample_rate).await?;
        
        info!("✅ Chat turn complete");
        Ok(())
    }
}
```

**Update**: `src/daemon/mod.rs`

```rust
pub mod chat;
pub use chat::ChatEngine;
```

---

### Step 8: Integrate into Daemon (1-2 days)

**Update**: `src/daemon/lifecycle.rs`

```rust
pub struct Lifecycle {
    // ... existing fields
    chat_engine: Option<Arc<RwLock<ChatEngine>>>,
}

impl Lifecycle {
    pub async fn start(&mut self) -> Result<()> {
        // ... existing startup code
        
        // Initialize chat if enabled
        if self.config.chat.enabled {
            info!("💬 Initializing chat mode");
            
            let llm_model = self.load_llm_model().await?;
            let tts_model = self.load_tts_model().await?;
            
            let chat_engine = ChatEngine::new(
                self.config.chat.clone(),
                self.stt_model.clone(),
                llm_model,
                tts_model,
            );
            
            self.chat_engine = Some(Arc::new(RwLock::new(chat_engine)));
        }
        
        // ... rest of startup
    }
}
```

**Update**: `src/daemon/dictation.rs`

```rust
impl DictationEngine {
    async fn handle_hotkey_event(&mut self, event: HotkeyEvent) -> Result<()> {
        if event.key == self.config.hotkey.trigger {
            // Existing transcription
            self.start_dictation().await?;
        } else if event.key == self.config.chat.hotkey {
            // New chat
            if let Some(chat) = &self.chat_engine {
                chat.write().start_conversation().await?;
            }
        }
        Ok(())
    }
}
```

---

### Step 9: Add TUI Panel (2-3 days)

**Create**: `tui/src/panels/chat.ts`

```typescript
export function createChatPanel(
  renderer: CliRenderer,
  state: AppState,
  callbacks: ConfigPanelCallbacks
) {
  // Create sections
  const llmSection = createSection("LLM Settings")
  const ttsSection = createSection("TTS Settings")
  
  // Add fields
  const llmModelField = createSelectField({
    label: "LLM Model",
    options: ["lfm2-1.2b-tool", "lfm25-audio-1.5b-onnx"],
    onChange: (idx) => {
      state.config.chat.llm.model_path = options[idx]
      callbacks.onDirty()
    }
  })
  
  const voiceField = createSelectField({
    label: "Voice",
    options: ["af_heart", "af_sky", "am_adam"],
    onChange: (idx) => {
      state.config.chat.tts.voice_id = options[idx]
      callbacks.onDirty()
    }
  })
  
  // ... more fields
  
  return { root, focusFirst, save, /* ... */ }
}
```

**Update**: `tui/src/app.ts`

```typescript
const tabs = new TabSelectRenderable(renderer, {
  options: [
    { name: "History" },
    { name: "Config" },
    { name: "Chat" },  // NEW
  ],
})

// Add to tab switching
if (index === 2) {
  showChat()
}
```

---

### Step 10: Test & Document (2-3 days)

**Write Tests**:

```rust
// tests/chat_integration.rs
#[tokio::test]
async fn test_full_chat_pipeline() {
    let config = load_test_config();
    let chat_engine = ChatEngine::new_with_mocks(config);
    
    chat_engine.start_conversation().await.unwrap();
    
    assert!(chat_engine.conversation_history.len() > 1);
}
```

**Manual Testing**:

1. Enable chat in config
2. Download models: `onevox models download lfm2-1.2b-tool`
3. Start daemon: `onevox daemon start`
4. Press chat hotkey → speak → hear response
5. Verify transcription still works independently

**Documentation**:

- Update `README.md` with chat feature
- Create `docs/CHAT_GUIDE.md` with user guide
- Update `ARCHITECTURE.md` with chat pipeline

---

## Quick Reference

### File Structure

```
src/
├── config.rs              # +ChatConfig, +LlmConfig, +TtsConfig
├── models/
│   ├── llm_runtime.rs     # NEW: LLM trait
│   ├── llm_gguf.rs        # NEW: GGUF implementation
│   ├── tts_runtime.rs     # NEW: TTS trait
│   ├── tts_kokoro.rs      # NEW: Kokoro implementation
│   └── registry.rs        # +ModelType, +LLM/TTS models
├── platform/audio/
│   └── player.rs          # NEW: Audio playback
└── daemon/
    ├── chat.rs            # NEW: Chat engine
    └── lifecycle.rs       # +chat initialization

tui/src/
├── panels/
│   └── chat.ts            # NEW: Chat config panel
├── data/
│   └── config.ts          # +ChatConfig types
└── app.ts                 # +Chat tab
```

### Key Commands

```bash
# Build with chat
cargo build --release --features chat

# Test chat
cargo test --features chat

# Run with chat enabled
onevox daemon start  # After enabling in config

# Download models
onevox models download lfm2-1.2b-tool
onevox models download kokoro-82m-onnx

# Open TUI
onevox tui  # Navigate to Chat tab
```

### Performance Targets

- **STT**: 50-200ms (existing)
- **LLM**: 20-50 tokens/sec (100-500ms for typical response)
- **TTS**: RTF < 0.5 (faster than real-time)
- **Total**: <1 second end-to-end

### Memory Budget

- **STT model**: ~100-500MB
- **LLM model**: ~1-2GB
- **TTS model**: ~100-300MB
- **Total runtime**: ~2-3GB

---

## Troubleshooting

### Common Issues

**"Model not found"**
- Download model: `onevox models download <model-id>`
- Check path in config: `~/.config/onevox/config.toml`

**"Out of memory"**
- Use smaller LLM model (lfm2-1.2b-tool instead of 1.5b)
- Disable preload in config
- Enable GPU acceleration

**"Audio not playing"**
- Check output device: `onevox devices list`
- Verify sample rate compatibility
- Check system audio settings

**"Slow generation"**
- Enable GPU: `device = "gpu"` in config
- Use smaller model
- Reduce max_tokens
- Lower temperature for faster sampling

---

## Next Steps

1. ✅ Review `agent.md` for coding standards
2. ✅ Read `CHAT_IMPLEMENTATION.md` for detailed checklist
3. ✅ Start with Step 1 (Configuration)
4. 🔄 Test each step before moving forward
5. 📝 Document as you implement
6. 🚀 Deploy and gather feedback

---

## Resources

- **agent.md** - Complete development guide and standards
- **CHAT_IMPLEMENTATION.md** - Detailed implementation checklist
- **ARCHITECTURE.md** - System design and patterns
- **config.example.toml** - Configuration examples
- **src/models/whisper_cpp.rs** - Reference model implementation
- **src/models/onnx_runtime.rs** - Reference ONNX implementation

---

**Questions?** Check the detailed implementation guide in `CHAT_IMPLEMENTATION.md` or review existing model implementations.

**Ready to start?** Begin with Step 1: Configuration! 🚀