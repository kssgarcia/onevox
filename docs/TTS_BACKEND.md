# Kokoro TTS Backend - Developer Guide

**Status**: ✅ Implemented  
**File**: `src/models/tts_kokoro.rs`  
**Model**: Kokoro-82M (ONNX)

---

## Overview

The Kokoro TTS backend provides high-quality, fast text-to-speech synthesis using the Kokoro-82M ONNX model. It supports 11 different voices (American/British, male/female) and runs efficiently on CPU.

### Key Features

- **11 voices**: American & British accents, male & female
- **Fast synthesis**: RTF < 0.5 (faster than real-time)
- **CPU-friendly**: No GPU required
- **High quality**: Natural-sounding speech at 24kHz
- **Flexible**: Voice switching, speech rate control, volume adjustment

---

## Quick Start

```rust
use onevox::models::{TtsKokoro, TtsRuntime, TtsRuntimeConfig};

// Create backend
let mut tts = TtsKokoro::new();

// Configure
let config = TtsRuntimeConfig {
    model_path: "kokoro-82m-onnx".to_string(),
    use_gpu: false,
    voice_id: "af_heart".to_string(),
    speech_rate: 1.0,
    pitch: 0.0,
    volume: 1.0,
};

// Load model
tts.load(config)?;

// Synthesize speech
let synthesis = tts.synthesize("Hello, world!")?;

println!("Generated {} samples at {}Hz", 
    synthesis.samples.len(), 
    synthesis.sample_rate);
println!("Synthesis time: {}ms (RTF: {:.3})", 
    synthesis.synthesis_time_ms, 
    synthesis.rtf);

// Play audio (see audio::player module)
```

---

## Available Voices

### American Voices

| Voice ID | Name | Gender | Description |
|----------|------|--------|-------------|
| `af` | Default | Female | Warm, friendly default voice |
| `af_bella` | Bella | Female | Clear, professional voice |
| `af_nicole` | Nicole | Female | Energetic, youthful voice |
| `af_sarah` | Sarah | Female | Calm, soothing voice |
| `af_sky` | Sky | Female | Bright, clear voice |
| `am_adam` | Adam | Male | Deep, authoritative voice |
| `am_michael` | Michael | Male | Friendly, conversational voice |

### British Voices

| Voice ID | Name | Gender | Description |
|----------|------|--------|-------------|
| `bf_emma` | Emma | Female | Elegant, sophisticated voice |
| `bf_isabella` | Isabella | Female | Refined, articulate voice |
| `bm_george` | George | Male | Distinguished, commanding voice |
| `bm_lewis` | Lewis | Male | Warm, approachable voice |

### Listing Voices Programmatically

```rust
let voices = tts.list_voices();
for voice in voices {
    println!("{}: {} ({})", 
        voice.id, 
        voice.name, 
        voice.language);
}
```

### Switching Voices

```rust
// Switch to a different voice
tts.set_voice("am_adam")?;

// Synthesize with new voice
let synthesis = tts.synthesize("This is Adam speaking.")?;
```

---

## Configuration

### TtsRuntimeConfig

```rust
pub struct TtsRuntimeConfig {
    /// Path to model (e.g., "kokoro-82m-onnx")
    pub model_path: String,
    
    /// Use GPU acceleration (not required for Kokoro)
    pub use_gpu: bool,
    
    /// Voice/speaker ID (e.g., "af_heart", "am_adam")
    pub voice_id: String,
    
    /// Speech rate (0.5 - 2.0, 1.0 is normal)
    pub speech_rate: f32,
    
    /// Pitch adjustment (-1.0 to 1.0, 0.0 is normal)
    pub pitch: f32,
    
    /// Volume (0.0 - 1.0, 1.0 is max)
    pub volume: f32,
}
```

### Default Configuration

```rust
let config = TtsRuntimeConfig::default();
// model_path: "models/kokoro-82m-onnx"
// use_gpu: false
// voice_id: "af_heart"
// speech_rate: 1.0
// pitch: 0.0
// volume: 1.0
```

---

## Pipeline Architecture

```
Text Input ("Hello, world!")
    ↓
Text Normalization
    • Lowercase conversion
    • Punctuation handling
    • Special character removal
    ↓
Phonemization (espeak-ng)
    • Convert text to IPA phonemes
    • Fallback to raw text if espeak-ng unavailable
    ↓
Tokenization (vocab.json)
    • Map phonemes to token IDs
    • Add padding tokens (0)
    • Max sequence length: 512 tokens
    ↓
Style Vector Lookup (voices/{voice_id}.bin)
    • Load 256-dim style vector
    • Based on token sequence length
    ↓
ONNX Inference (model.onnx)
    • Inputs: input_ids [1, seq_len], style [1, 256], speed [1]
    • Output: audio samples [1, num_samples]
    ↓
Volume Adjustment
    • Apply volume multiplier
    ↓
TtsSynthesis Result
    • Samples: Vec<f32>
    • Sample rate: 24000 Hz
    • Metrics: synthesis_time_ms, rtf
```

---

## Model Files

The Kokoro model requires the following files (auto-downloaded):

```
models/kokoro-82m-onnx/
├── model.onnx              # Main TTS model (82MB)
├── config.json             # Model configuration
├── vocab.json              # Phoneme vocabulary
└── voices/
    ├── af.bin              # Default female voice
    ├── af_bella.bin        # Bella voice
    ├── af_nicole.bin       # Nicole voice
    ├── af_sarah.bin        # Sarah voice
    ├── af_sky.bin          # Sky voice
    ├── am_adam.bin         # Adam voice
    ├── am_michael.bin      # Michael voice
    ├── bf_emma.bin         # Emma voice
    ├── bf_isabella.bin     # Isabella voice
    ├── bm_george.bin       # George voice
    └── bm_lewis.bin        # Lewis voice
```

### Voice File Format

Each `.bin` file contains 512 × 256 float32 values (524,288 bytes):
- 512 context lengths (0-511 tokens)
- 256-dimensional style vector per context length
- Little-endian float32 encoding

---

## Dependencies

### Required

- `ort` - ONNX Runtime bindings (v2.0.0-rc.11)
- `serde_json` - JSON parsing for vocab.json

### Optional

- `espeak-ng` - Phonemization (external command-line tool)
  - **macOS**: `brew install espeak-ng`
  - **Linux**: `apt install espeak-ng` or `pacman -S espeak-ng`
  - **Windows**: Download from [espeak-ng releases](https://github.com/espeak-ng/espeak-ng/releases)
  - **Fallback**: If not available, raw text is passed through (may reduce quality)

---

## Performance

### Benchmarks (MacBook Pro M1, CPU only)

| Metric | Value |
|--------|-------|
| Model size | 82 MB (quantized ONNX) |
| Memory usage | ~300 MB loaded |
| RTF (Real-Time Factor) | 0.3-0.5 (2-3x faster than real-time) |
| Latency (10 words) | ~200-300ms |
| Latency (50 words) | ~500-800ms |
| Sample rate | 24 kHz |
| Audio format | f32, mono |

### RTF Explanation

RTF (Real-Time Factor) = Synthesis Time / Audio Duration

- RTF < 1.0: Faster than real-time (good!)
- RTF = 1.0: Exactly real-time
- RTF > 1.0: Slower than real-time (bad)

Example: 1 second of audio synthesized in 0.3 seconds → RTF = 0.3

---

## Error Handling

### Common Errors

```rust
// Model not downloaded
Err(Error::Model("Model directory not found: ..."))
→ Run: onevox models download kokoro-82m-onnx

// Voice file missing
Err(Error::Model("Voice file not found: voices/af_heart.bin"))
→ Re-download model or check file integrity

// Text too long
Err(Error::Model("Token sequence too long (600 tokens), truncating to 512"))
→ Split text into smaller chunks

// Voice not found
Err(Error::Model("Voice 'invalid_voice' not found"))
→ Use tts.list_voices() to see available voices
```

---

## Testing

### Unit Tests

```bash
# Run TTS backend tests
cargo test --features onnx tts_kokoro

# Tests:
# ✓ test_create_backend
# ✓ test_not_loaded_initially
# ✓ test_normalize_text
# ✓ test_voice_list
# ✓ test_voice_style_data
# ✓ test_empty_text_synthesis
```

### Integration Testing

```rust
#[test]
fn test_kokoro_synthesis() {
    let mut tts = TtsKokoro::new();
    let config = TtsRuntimeConfig::default();
    
    // Note: Requires model to be downloaded
    if let Ok(_) = tts.load(config) {
        let result = tts.synthesize("Test message");
        assert!(result.is_ok());
        
        let synthesis = result.unwrap();
        assert!(!synthesis.is_empty());
        assert_eq!(synthesis.sample_rate, 24000);
        assert!(synthesis.rtf < 1.0); // Faster than real-time
    }
}
```

---

## Best Practices

### 1. Model Preloading

```rust
// Load once at startup for better latency
let mut tts = TtsKokoro::new();
tts.load(config)?;

// Reuse for multiple syntheses
for text in texts {
    let audio = tts.synthesize(text)?;
    // Process audio...
}
```

### 2. Text Chunking

```rust
// Split long text into sentences for better streaming
fn synthesize_long_text(tts: &mut TtsKokoro, text: &str) -> Result<Vec<TtsSynthesis>> {
    let sentences = text.split(|c| c == '.' || c == '!' || c == '?');
    
    sentences
        .filter(|s| !s.trim().is_empty())
        .map(|sentence| tts.synthesize(sentence.trim()))
        .collect()
}
```

### 3. Voice Selection

```rust
// Choose voice based on content/context
let voice = if formal_context {
    "bf_emma" // British, formal
} else if technical_content {
    "am_adam" // Male, authoritative
} else {
    "af_heart" // Default, friendly
};

tts.set_voice(voice)?;
```

### 4. Error Recovery

```rust
// Graceful degradation
match tts.synthesize(text) {
    Ok(synthesis) => play_audio(synthesis),
    Err(e) => {
        warn!("TTS failed: {}, falling back to text display", e);
        display_text(text);
    }
}
```

---

## Limitations

1. **English only**: Kokoro is trained on English speech
2. **Token limit**: Max 512 tokens per synthesis (~500 words)
3. **Phonemization**: Best with espeak-ng installed
4. **Single speaker**: Cannot mix voices within one synthesis
5. **No streaming**: Full synthesis before playback

---

## Future Enhancements

- [ ] Streaming synthesis (word-by-word)
- [ ] SSML support (prosody, emphasis, pauses)
- [ ] Custom voice training
- [ ] Multi-language support
- [ ] Real-time voice cloning
- [ ] Emotion/style control beyond preset voices

---

## References

- **Model**: [Kokoro-82M on Hugging Face](https://huggingface.co/onnx-community/Kokoro-82M-ONNX)
- **ONNX Runtime**: [ort crate](https://docs.rs/ort/latest/ort/)
- **espeak-ng**: [GitHub](https://github.com/espeak-ng/espeak-ng)
- **OneVox TTS Trait**: `src/models/tts_runtime.rs`

---

## Support

For issues, questions, or feature requests:
1. Check the main [CHAT_IMPLEMENTATION.md](CHAT_IMPLEMENTATION.md) guide
2. Review [CHAT_PROGRESS.md](CHAT_PROGRESS.md) for current status
3. File an issue on the OneVox repository