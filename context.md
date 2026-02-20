# Project Summary — Cross-Platform Local Speech-to-Text Daemon (Pro Version)

## Vision

Build a **cross-platform, privacy-first, local speech-to-text engine** that runs as a background daemon and allows users to dictate text into **any text input field in any application** using a global hotkey.

This is not just a dictation app.

It is a:

> **Local Speech Inference Layer for the Operating System**

The system must support multiple transcription models (Whisper, Faster-Whisper, HuggingFace models, ONNX, GGUF, etc.), allow model switching, and provide a terminal-based TUI for configuration and monitoring.

---

# Core Requirements

## 1. Cross-Platform Support

- macOS (initial target)
- Linux
- Windows (later phase)

---

## 2. Background Daemon

A background service (`speechd`) that:

- Runs on system startup
- Listens for global hotkeys
- Captures microphone input
- Performs streaming transcription
- Injects transcribed text into the active application
- Exposes IPC for control

---

## 3. Global Push-to-Talk

- System-wide hotkey (works in any app)
- Press-and-hold or toggle mode
- Configurable keybinding
- Low latency activation

---

## 4. Audio Engine

- Real-time microphone capture
- Streaming audio chunks
- Voice Activity Detection (VAD)
- Buffering and chunk management
- Device selection support

---

## 5. Model Runtime Abstraction

The system must support multiple transcription backends through a unified interface:

```rust
trait SpeechModel {
    fn load(config: ModelConfig) -> Result<Self>;
    fn transcribe(audio: AudioBuffer) -> Result<Transcript>;
    fn unload(&mut self);
}
```
