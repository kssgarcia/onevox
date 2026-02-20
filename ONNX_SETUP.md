# Vox - ONNX Runtime Setup Guide

## ✅ Current Status

The daemon is now successfully running with Whisper ONNX model support! The system automatically detects the ONNX Runtime library on macOS.

## Quick Start

### 1. Install ONNX Runtime (if not already installed)

```bash
brew install onnxruntime
```

### 2. Download a Whisper Model

```bash
# List available models
./target/release/vox models list

# Download the tiny English model (144 MB)
./target/release/vox models download whisper-tiny.en

# Or download a larger model for better accuracy
./target/release/vox models download whisper-base.en
```

### 3. Start the Daemon

```bash
# Build the release binary
cargo build --release

# Start the daemon (auto-detects ONNX Runtime)
./target/release/vox daemon
```

The daemon will automatically:
- Detect ONNX Runtime at `/opt/homebrew/lib/libonnxruntime.dylib`
- Load the Whisper ONNX model (whisper-tiny.en)
- Register the global hotkey (`Option+Delete`)
- Start listening for dictation commands

### 4. Test Dictation

1. Open any text editor or application
2. Click in a text input field
3. Press and hold `Option+Delete`
4. Speak into your microphone
5. Release `Option+Delete`
6. The transcribed text should appear!

**Current Behavior**: The system will inject placeholder text showing the model is loaded and the audio duration processed:
```
[Whisper whisper-tiny.en model loaded! Processed 2.3s of audio. Full inference pipeline coming soon...]
```

## Daemon Logs

The daemon is running and shows these successful initialization messages:

```
✅ Whisper ONNX model loaded successfully
✅ Dictation engine initialized
✅ Hotkey registered: Option+Delete
✅ Hotkey listener started
```

## What's Working

✅ ONNX Runtime auto-detection  
✅ Whisper ONNX model loading (encoder + decoder)  
✅ Global hotkey detection (`Option+Delete`)  
✅ Audio capture from microphone  
✅ Voice Activity Detection (VAD)  
✅ Text injection into applications  

## What's Next (Full Transcription)

The current implementation loads the models but returns placeholder transcriptions. To get real speech-to-text, we need to implement:

1. **Audio preprocessing** - Convert raw audio to mel spectrogram
2. **Encoder inference** - Run mel spectrogram through ONNX encoder
3. **Decoder loop** - Autoregressive token generation
4. **Token-to-text** - Convert tokens back to readable text

## Configuration

The configuration file is located at:
```
~/Library/Application Support/vox/config.toml
```

Current model settings:
```toml
[model]
backend = "onnx"
model_path = "whisper-tiny.en"
device = "auto"
language = "en"
task = "transcribe"
preload = true
```

## Troubleshooting

### "ONNX Runtime library not found"

If you see this error, install ONNX Runtime:
```bash
brew install onnxruntime
```

Or manually set the library path:
```bash
export ORT_DYLIB_PATH=/opt/homebrew/lib/libonnxruntime.dylib
./target/release/vox daemon
```

### "Model not found"

Download the model first:
```bash
./target/release/vox models download whisper-tiny.en
```

### Check daemon status

```bash
# View recent logs
tail -f /tmp/vox-test.log

# Check if daemon is running
pgrep -f "vox daemon"
```

## Model Management Commands

```bash
# List all available models
./target/release/vox models list

# Show downloaded models
./target/release/vox models downloaded

# Download a specific model
./target/release/vox models download whisper-base.en

# Show model details
./target/release/vox models info whisper-tiny.en

# Remove a downloaded model
./target/release/vox models remove whisper-tiny.en
```

## Available Models

- `whisper-tiny.en` - 144 MB (fastest, good for testing)
- `whisper-base.en` - 290 MB (better accuracy)
- `whisper-small.en` - 967 MB (good balance)
- `whisper-medium.en` - 3.1 GB (highest accuracy)

## Architecture

```
User presses hotkey
    ↓
Audio captured from microphone (cpal)
    ↓
Voice Activity Detection (Energy VAD)
    ↓
Whisper ONNX Model (encoder + decoder)
    ↓
Text transcription (placeholder for now)
    ↓
Text injection via Accessibility API (enigo)
    ↓
Text appears in active application
```

## Files Modified

- `src/models/whisper_onnx.rs` - ONNX backend with auto-detection
- `src/daemon/dictation.rs` - Uses WhisperOnnx instead of MockModel
- `Cargo.toml` - Added ONNX dependencies
- `config.toml` - Updated to use ONNX backend

## Performance

Model loading takes approximately 30-35 seconds on the first run. Subsequent runs are faster as the model is cached in memory.

---

**Status**: ✅ Daemon running successfully with ONNX model loaded!
