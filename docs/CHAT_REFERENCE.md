# Chat Feature - Quick Reference

**OneVox Chat**: Talk to an AI assistant with voice input and audio responses.

---

## 🚀 Quick Start

### 1. Enable Chat in Config
```toml
# ~/.config/onevox/config.toml

[chat]
enabled = true
hotkey = "Cmd+Shift+9"  # macOS: Cmd+Shift+9, Linux/Win: Ctrl+Shift+9
```

### 2. Download Models
```bash
# Download LLM model (1.2 GB)
onevox models download lfm2-1.2b-tool

# Download TTS model (82 MB)
onevox models download kokoro-82m-onnx
```

### 3. Start Daemon
```bash
onevox daemon --foreground
```

### 4. Start Chatting
```bash
# Option 1: CLI command
onevox start-chat

# Option 2: Press hotkey (Cmd+Shift+9)
# Hold to talk, release to process
```

---

## 📋 Commands

### Chat Control
```bash
onevox start-chat          # Start chat session
onevox stop-chat           # Stop chat session
onevox clear-chat-history  # Clear conversation history
```

### Status & Info
```bash
onevox status              # Check daemon and chat status
onevox info                # Show system info and GPU capabilities
```

### Models
```bash
onevox models list         # List available models
onevox models downloaded   # Show downloaded models
onevox models download ID  # Download a model
onevox models info ID      # Show model information
```

---

## ⌨️ Hotkeys

| Action | macOS | Linux/Windows |
|--------|-------|---------------|
| Dictation | Cmd+Shift+0 | Ctrl+Shift+0 |
| Chat | Cmd+Shift+9 | Ctrl+Shift+9 |

**Usage**: Hold hotkey while talking, release when done. Wait for AI response.

---

## 🎙️ How It Works

```
You speak (hold hotkey)
    ↓
STT: Speech → Text (Whisper)
    ↓
LLM: Text → AI Response (Liquid LFM2)
    ↓
TTS: Text → Speech (Kokoro)
    ↓
Audio plays through speakers
```

---

## 📊 Status Display

```bash
$ onevox status
📊 Onevox Daemon Status

  Version:     0.1.0
  PID:         12345
  State:       Idle
  Uptime:      120s
  Model:       whisper-base.en
  Dictating:   No
  Chat:        Ready ✅
  Memory:      1234 MB
  CPU:         5.2%
```

**Chat Status**:
- `Disabled` - Chat not enabled in config
- `Enabled (loading models...)` - Models are loading
- `Ready ✅` - Chat ready, press hotkey to start
- `Active 💬` - Currently in chat session

---

## ⚙️ Configuration

### Minimal Config
```toml
[chat]
enabled = true
hotkey = "Cmd+Shift+9"

[chat.llm]
model_path = "~/.local/share/onevox/models/lfm2-1.2b-tool"

[chat.tts]
model_path = "~/.local/share/onevox/models/kokoro-82m-onnx"
```

### Advanced Config
```toml
[chat]
enabled = true
hotkey = "Cmd+Shift+9"
system_prompt = "You are a helpful assistant."

[chat.llm]
model_path = "~/.local/share/onevox/models/lfm2-1.2b-tool"
temperature = 0.7
max_tokens = 150
top_p = 0.9
top_k = 40
repetition_penalty = 1.1
context_window = 2048

[chat.tts]
model_path = "~/.local/share/onevox/models/kokoro-82m-onnx"
voice = "af_heart"
pitch = 1.0
volume = 1.0
speech_rate = 1.0
```

### Available TTS Voices
- `af_heart` - Female, warm voice (default)
- `af_sky` - Female, clear voice
- `am_adam` - Male, professional voice
- `am_michael` - Male, friendly voice
- `bf_emma` - Female, British accent
- `bf_isabella` - Female, British accent
- `bm_george` - Male, British accent
- `bm_lewis` - Male, British accent

---

## 🔧 Troubleshooting

### Chat Not Starting
```bash
# Check status
onevox status

# Check logs
journalctl --user -u onevox -f  # Linux
log show --predicate 'process == "onevox"' --info  # macOS
```

**Common Issues**:
- Models not downloaded → `onevox models downloaded`
- Chat not enabled in config → Check `[chat] enabled = true`
- Daemon not running → `onevox daemon --foreground`

### No Audio Output
- Check system audio settings
- Verify volume in config (`[chat.tts] volume = 1.0`)
- Test with: `onevox test-audio`

### Poor Performance
```bash
# Check system info
onevox info

# Enable GPU acceleration (if available)
# Rebuild with: cargo build --release --features metal  # macOS
#           or: cargo build --release --features cuda   # Linux/Win
```

**Performance Tips**:
- Use smaller LLM model for faster responses
- Reduce `max_tokens` in config
- Increase `temperature` for more creative (but slower) responses
- Use GPU acceleration if available

### Model Loading Errors
```bash
# Verify model exists
ls -lh ~/.local/share/onevox/models/

# Re-download if corrupted
onevox models remove lfm2-1.2b-tool
onevox models download lfm2-1.2b-tool
```

---

## 🎯 Use Cases

### Quick Q&A
"What's the weather like today?"  
"Define 'serendipity'"  
"How do I restart a service in Linux?"

### Code Help
"Explain what this regex does: `^\d{3}-\d{2}-\d{4}$`"  
"Write a Python function to reverse a string"

### Writing Assistance
"Rewrite this email to be more formal: ..."  
"Suggest 5 titles for a blog post about..."

### Learning
"Explain quantum computing in simple terms"  
"What's the difference between RAM and ROM?"

---

## 📖 Related Documentation

- [Chat Implementation Guide](CHAT_IMPLEMENTATION.md)
- [Chat Progress Report](CHAT_PROGRESS.md)
- [LLM Backend Documentation](LLM_BACKEND.md)
- [TTS Backend Documentation](TTS_BACKEND.md)
- [Main README](../README.md)

---

## 💡 Tips

1. **Keep prompts short**: The AI works best with concise questions
2. **Clear history regularly**: Use `onevox clear-chat-history` to start fresh
3. **Adjust temperature**: Lower = more focused, higher = more creative
4. **Use system prompt**: Customize AI personality in config
5. **GPU acceleration**: Significantly faster on Metal (Mac) or CUDA (Linux/Win)

---

## 🆘 Getting Help

- GitHub Issues: https://github.com/yourusername/onevox/issues
- Documentation: https://onevox.dev/docs
- Community: https://discord.gg/onevox

---

**Last Updated**: March 10, 2026  
**Version**: 0.1.0 (Chat Feature Beta)
