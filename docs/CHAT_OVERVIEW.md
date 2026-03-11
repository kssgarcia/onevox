# Chat Feature Implementation - Executive Summary

> **Goal**: Add voice-based conversational AI to OneVox using STT→LLM→TTS pipeline

## What We're Building

A new feature that lets users have natural voice conversations with an AI assistant, triggered by a separate hotkey from the existing transcription feature.

```
User speaks → Whisper STT → Liquid AI LLM → Kokoro TTS → Audio playback
     ↓              ↓                ↓                ↓            ↓
  "Chat mode"   Transcribe      Generate text    Synthesize    Play response
```

## Key Decisions

### Model Selection

| Component | Model | Format | Size | Why |
|-----------|-------|--------|------|-----|
| **STT** | Existing Whisper | GGML/ONNX | 75MB-500MB | Already implemented, fast |
| **LLM** | Liquid LFM2 1.2B | GGUF | 1.2GB | CPU-friendly, tool-optimized |
| **TTS** | Kokoro 82M | ONNX | 82MB | Fast (RTF<0.5), natural voices |

**Alternative LLM**: LFM2.5-Audio-1.5B (ONNX, 1.5GB) for users wanting audio-aware model

### Architecture Approach

**✅ Chosen: Dual Pipeline**
- Keep existing STT transcription untouched (Cmd+Shift+0)
- Add new chat pipeline with separate hotkey (Cmd+Shift+9)
- Independent operation, no interference
- Share STT model between both modes

**❌ Rejected: Single Unified Pipeline**
- Would complicate existing fast transcription
- Users often want both modes available
- Harder to maintain performance guarantees

### Technical Stack

**Reuse Existing**:
- ✅ ONNX Runtime (already have it for TTS)
- ✅ cpal (audio I/O)
- ✅ tokio (async runtime)
- ✅ Model registry & downloader
- ✅ Configuration system
- ✅ TUI framework

**New Dependencies**:
- `llama-cpp-rs` for GGUF LLM support (~minimal)

## Implementation Strategy

### Phase 1: Foundation (Week 1)
- Add config structures for chat, LLM, TTS
- Update model registry with LLM/TTS models
- Define runtime traits (LlmRuntime, TtsRuntime)

### Phase 2: Backends (Week 2-3)
- Implement GGUF LLM backend (llama.cpp bindings)
- Implement Kokoro TTS backend (ONNX)
- Add audio playback support

### Phase 3: Integration (Week 4)
- Create ChatEngine (orchestrates STT→LLM→TTS)
- Integrate into daemon lifecycle
- Add dual hotkey support

### Phase 4: UX (Week 5)
- Add Chat configuration panel to TUI
- Model download UI
- Voice selection
- System prompt editor

### Phase 5: Polish (Week 6)
- Performance optimization
- Error handling refinement
- Documentation
- Testing on all platforms

## Performance Targets

| Component | Target | Notes |
|-----------|--------|-------|
| STT | 50-200ms | Existing, maintain |
| LLM | 20-50 tok/s | 100-500ms for response |
| TTS | RTF < 0.5 | Faster than real-time |
| **Total Pipeline** | **< 1 second** | Typical interaction |

**Memory Budget**: ~2-3GB with all models loaded

## Success Criteria

### Must Have (MVP)
- [x] STT→LLM→TTS pipeline functional
- [x] Separate hotkey for chat mode
- [x] At least one LLM backend working (GGUF or ONNX)
- [x] Kokoro TTS with multiple voices
- [x] Audio playback
- [x] TUI configuration panel
- [x] Basic error handling
- [x] Documentation

### Should Have
- [x] Both LLM backends (GGUF and ONNX)
- [x] Conversation history management
- [x] GPU acceleration support
- [x] Performance optimization
- [x] Cross-platform testing

### Nice to Have (Future)
- [ ] Streaming LLM responses
- [ ] Voice activity detection for chat
- [ ] Conversation export/import
- [ ] Custom voice training
- [ ] Multi-language support

## Risk Assessment

### Technical Risks

**1. LLM Inference Speed**
- **Risk**: Too slow for interactive use
- **Mitigation**: Start with small models (1-2B), optimize, add streaming later
- **Severity**: Medium

**2. Memory Usage**
- **Risk**: Exceeds available RAM on some systems
- **Mitigation**: Lazy loading, model unloading, quantization
- **Severity**: Medium

**3. Audio Quality**
- **Risk**: TTS sounds robotic
- **Mitigation**: Kokoro has good quality, multiple voice options
- **Severity**: Low

### User Experience Risks

**1. Complexity**
- **Risk**: Too many options, confusing setup
- **Mitigation**: Sane defaults, clear docs, guided setup
- **Severity**: Medium

**2. Latency Expectations**
- **Risk**: Users expect instant responses
- **Mitigation**: Set expectations, show progress indicators
- **Severity**: Low

## File Structure Overview

```
src/
├── config.rs                    # +ChatConfig, +LlmConfig, +TtsConfig
├── models/
│   ├── llm_runtime.rs          # NEW: LLM trait definition
│   ├── llm_gguf.rs             # NEW: GGUF LLM implementation
│   ├── llm_onnx.rs             # NEW: ONNX LLM implementation
│   ├── tts_runtime.rs          # NEW: TTS trait definition
│   ├── tts_kokoro.rs           # NEW: Kokoro TTS implementation
│   └── registry.rs             # UPDATE: Add LLM/TTS models
├── platform/audio/
│   └── player.rs               # NEW: Audio playback
└── daemon/
    ├── chat.rs                 # NEW: Chat engine
    ├── lifecycle.rs            # UPDATE: Chat initialization
    └── dictation.rs            # UPDATE: Dual hotkey support

tui/src/
├── panels/
│   └── chat.ts                 # NEW: Chat configuration panel
├── data/
│   └── config.ts               # UPDATE: Chat config types
└── app.ts                      # UPDATE: Add Chat tab

docs/
├── CHAT_OVERVIEW.md            # THIS FILE: Executive summary
├── CHAT_IMPLEMENTATION.md      # Detailed checklist
├── CHAT_QUICKSTART.md          # Quick start guide
└── CHAT_GUIDE.md               # User documentation (to be written)
```

## Configuration Example

```toml
[chat]
enabled = true
hotkey = "Cmd+Shift+9"

[chat.llm]
model_path = "lfm2-1.2b-tool"
device = "auto"
temperature = 0.7
max_tokens = 256
system_prompt = "You are a helpful AI assistant. Be concise and direct."

[chat.tts]
model_path = "kokoro-82m-onnx"
device = "auto"
voice_id = "af_heart"
speech_rate = 1.0
```

## Usage Flow

1. **Setup** (one-time):
   ```bash
   # Download models
   onevox models download lfm2-1.2b-tool
   onevox models download kokoro-82m-onnx
   
   # Configure via TUI
   onevox tui  # Enable chat, select voice, customize prompt
   ```

2. **Daily Use**:
   - Press Cmd+Shift+0 → Speak → Get transcription (existing)
   - Press Cmd+Shift+9 → Speak → Get AI response (new)

3. **Conversation**:
   - User: "What's the weather like today?"
   - AI: [Generates response]
   - AI: [Speaks response back]
   - Context maintained for follow-up questions

## Code Quality Standards

Following existing OneVox patterns:

- ✅ Use `ModelRuntime` trait pattern
- ✅ Same error handling approach (`thiserror`)
- ✅ Structured logging with emojis
- ✅ Arc<RwLock<T>> for shared state
- ✅ Tokio channels for async communication
- ✅ Platform-agnostic implementations
- ✅ Comprehensive testing
- ✅ Clear documentation

## Next Steps

### Immediate
1. Review `agent.md` for complete coding standards
2. Read `CHAT_IMPLEMENTATION.md` for detailed checklist
3. Follow `CHAT_QUICKSTART.md` for step-by-step guide

### Development
1. Start with Phase 1 (Configuration & Foundation)
2. Test each component independently
3. Integrate incrementally
4. Profile performance early
5. Test on all platforms

### Documentation
1. Keep implementation docs updated
2. Write user guide as features complete
3. Add troubleshooting section
4. Create video demos

## Timeline

**Optimistic**: 4 weeks (with parallelization)  
**Realistic**: 6 weeks (accounting for testing, bugs)  
**Conservative**: 8 weeks (including polish and docs)

**First Usable Version**: ~3 weeks (MVP with GGUF LLM only)  
**Production Ready**: ~6 weeks (both backends, full testing)

## Dependencies

### Core
- `llama-cpp-rs` - GGUF LLM support (new)
- `ort` - ONNX Runtime (existing, reuse)
- `cpal` - Audio I/O (existing, reuse)

### Build Size Impact
- Binary: +5-10MB (llama.cpp bindings)
- Runtime: +2-3GB when models loaded
- Disk: +1-2GB for models

## Testing Strategy

### Unit Tests
- Config parsing and validation
- Runtime trait implementations
- Audio processing and playback
- Error handling

### Integration Tests
- Full STT→LLM→TTS pipeline
- Hotkey routing
- Model switching
- Conversation management

### Performance Tests
- Latency benchmarks
- Memory profiling
- Concurrent operation
- GPU vs CPU comparison

### Manual Tests
- Cross-platform validation
- Audio quality assessment
- User experience flow
- Edge cases and errors

## Success Metrics

### Technical
- ✅ Pipeline latency < 1 second
- ✅ Memory usage < 3GB
- ✅ No interference with transcription
- ✅ All tests passing on macOS/Linux/Windows

### User Experience
- ✅ Clear setup process (< 5 minutes)
- ✅ Natural conversation flow
- ✅ Good audio quality
- ✅ Reliable error handling

### Code Quality
- ✅ Follows existing patterns
- ✅ Well-documented
- ✅ Comprehensive tests
- ✅ Maintainable architecture

## Resources

- **agent.md** - Complete development guide with coding standards
- **CHAT_IMPLEMENTATION.md** - Detailed phase-by-phase checklist
- **CHAT_QUICKSTART.md** - Quick 10-step implementation guide
- **ARCHITECTURE.md** - System design and patterns (existing)
- **src/models/whisper_cpp.rs** - Reference model implementation
- **src/models/onnx_runtime.rs** - Reference ONNX implementation

## Questions?

### For Users
- See `CHAT_GUIDE.md` (to be written) for usage instructions
- Check `README.md` for general OneVox information

### For Developers
- Read `agent.md` first for coding standards
- Follow `CHAT_QUICKSTART.md` for implementation steps
- Refer to `CHAT_IMPLEMENTATION.md` for detailed checklist

### For Contributors
- Review `CONTRIBUTING.md` for contribution guidelines
- Check `DEVELOPMENT.md` for build instructions
- See existing model implementations as examples

---

**Status**: Planning Phase  
**Last Updated**: 2024-01-XX  
**Next Milestone**: Phase 1 - Configuration & Foundation

**Ready to start?** 🚀 Begin with `CHAT_QUICKSTART.md` Step 1!