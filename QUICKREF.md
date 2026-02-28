# Quick Reference

## Commands

```bash
onevox status              # Check daemon status
onevox tui                 # Terminal UI
onevox daemon              # Start daemon (foreground)
onevox stop                # Stop daemon

onevox devices list        # List audio devices
onevox models list         # Available models
onevox models download <id>  # Download model
onevox history list        # Transcription history
onevox config show         # Show configuration
```

## Service Management

**macOS:**
```bash
launchctl kickstart -k gui/$(id -u)/com.onevox.daemon  # Restart
launchctl stop com.onevox.daemon                        # Stop
```

**Linux:**
```bash
systemctl --user start onevox    # Start
systemctl --user stop onevox     # Stop
systemctl --user status onevox   # Status
```

**Windows:**
```powershell
# Register (one-time, run as Administrator)
sc.exe create Onevox binPath= "\"$env:LOCALAPPDATA\onevox\onevox.exe\" daemon --foreground" start= auto

sc.exe start Onevox     # Start
sc.exe stop Onevox      # Stop
sc.exe query Onevox     # Status
sc.exe stop Onevox; sc.exe start Onevox   # Restart
```

## Paths

**macOS:**
- Config: `~/Library/Application Support/com.onevox.onevox/config.toml`
- Models: `~/Library/Caches/com.onevox.onevox/models/`
- Logs: `~/Library/Logs/onevox/stdout.log`

**Linux:**
- Config: `~/.config/onevox/config.toml`
- Models: `~/.cache/onevox/models/`
- Logs: `journalctl --user -u onevox -f`

**Windows:**
- Config: `%APPDATA%\onevox\onevox\config\config.toml`
- Models: `%LOCALAPPDATA%\onevox\onevox\cache\models\`
- Logs: `%APPDATA%\onevox\onevox\data\logs\onevox.log`

## Configuration

**Location:**
- macOS: `~/Library/Application Support/com.onevox.onevox/config.toml`
- Linux: `~/.config/onevox/config.toml`
- Windows: `%APPDATA%\onevox\onevox\config\config.toml`

**View/Edit:**
```bash
onevox config show         # View current config
onevox config init         # Create default config
onevox reload-config       # Reload and restart daemon with new config
```

> **Note:** `reload-config` automatically restarts the daemon using:
> - macOS: `launchctl kickstart` (launchd)
> - Linux: `systemctl --user restart onevox` (systemd)
> - Windows: Manual restart required

### Essential Settings

**Hotkey:**
```toml
[hotkey]
trigger = "Cmd+Shift+0"  # macOS default
# trigger = "Ctrl+Shift+Space"  # Linux/Windows default
mode = "push-to-talk"  # or "toggle"
```

**Audio Device:**
```bash
# List available devices
onevox devices list

# Set in config
[audio]
device = "default"  # or specific device name
sample_rate = 16000
```

**Model:**
```toml
[model]
# Model identifier (backend auto-detected from path)
model_path = "ggml-base.en"         # English-only, ~142MB
# model_path = "ggml-base"          # Multilingual (99+ languages)
# model_path = "parakeet-ctc-0.6b"  # ONNX (included by default)

device = "auto"  # auto, cpu, gpu
preload = true   # Load model at startup
```

**Available Models:**
- **English-only**: `ggml-tiny.en` (75MB), `ggml-base.en` (142MB), `ggml-small.en` (466MB), `ggml-medium.en` (1.5GB)
- **Multilingual**: `ggml-tiny` (75MB), `ggml-base` (142MB), `ggml-small` (466MB), `ggml-medium` (1.5GB), `ggml-large-v2/v3` (2.9GB), `ggml-large-v3-turbo` (1.6GB)
- **ONNX**: `parakeet-ctc-0.6b` (multilingual, INT8, included by default)

Multilingual models automatically detect the spoken language. Backend is auto-selected based on model name.

### All Configuration Options

**[daemon]** - Daemon behavior
```toml
auto_start = true           # Start on system boot
log_level = "info"          # trace, debug, info, warn, error
log_retention_days = 7      # Log rotation
```

**[hotkey]** - Global hotkey settings
```toml
trigger = "Cmd+Shift+0"     # Hotkey combination
mode = "push-to-talk"       # push-to-talk or toggle
min_hold_duration_ms = 100  # Prevent accidental triggers
```

**[audio]** - Audio capture
```toml
device = "default"          # Audio input device
sample_rate = 16000         # Sample rate (Hz)
chunk_duration_ms = 200     # Processing chunk size
buffer_duration_sec = 2     # Buffer size
```

**[vad]** - Voice Activity Detection
```toml
enabled = false             # Auto-detect speech/silence
backend = "energy"          # energy, silero, webrtc
threshold = 0.001           # Detection sensitivity (0.0-1.0)
pre_roll_ms = 300          # Capture before speech starts
post_roll_ms = 500         # Continue after speech ends
```

**[model]** - Transcription model
```toml
model_path = "ggml-base.en" # Model identifier (backend auto-detected)
device = "auto"             # auto, cpu, gpu
preload = true              # Load model at startup
```

**[post_processing]** - Text processing
```toml
auto_punctuation = true     # Add punctuation
auto_capitalize = true      # Capitalize sentences
remove_filler_words = false # Remove um, uh, etc.

[post_processing.replacements]
"onevox" = "OneVox"        # Custom word replacements
```

**[injection]** - Text insertion
```toml
method = "accessibility"    # accessibility, clipboard, paste
paste_delay_ms = 50        # Delay before pasting
focus_settle_ms = 80       # Wait for focus
typing_speed = 100         # Chars/sec for paste method
```

**[ui]** - User interface
```toml
recording_overlay = true    # Show recording indicator
```

**[tui]** - Terminal UI
```toml
enabled = true              # Enable TUI
refresh_rate = 10          # Update frequency (Hz)
theme = "dark"             # dark or light
```

**[history]** - Transcription history
```toml
enabled = true              # Track history
max_entries = 1000         # Maximum entries
auto_save = true           # Save after each transcription
```

**[advanced]** - Advanced settings
```toml
max_concurrent_transcriptions = 1  # Concurrent jobs
chunk_queue_size = 10              # Audio buffer queue
inference_timeout_sec = 30         # Timeout
auto_restart = true                # Restart on errors
```

### Common Configurations

**Fast & Accurate (Recommended):**
```toml
[model]
model_path = "ggml-base.en"
device = "auto"

[vad]
enabled = false  # Manual control

[hotkey]
mode = "push-to-talk"
```

**Automatic Speech Detection:**
```toml
[vad]
enabled = true
threshold = 0.001
post_roll_ms = 500

[hotkey]
mode = "push-to-talk"
```

**Toggle Mode (Press once to start/stop):**
```toml
[hotkey]
mode = "toggle"

[vad]
enabled = false
```

**Maximum Accuracy (Slower):**
```toml
[model]
model_path = "ggml-medium.en"  # or ggml-large-v3
device = "gpu"
```

**Wayland Manual Control:**
```toml
[vad]
enabled = false  # Use start-dictation/stop-dictation commands

[hotkey]
mode = "toggle"
```

## Troubleshooting

```bash
# Check status
onevox status

# Test audio
onevox test-audio --duration 3

# List devices
onevox devices list

# View logs
tail -f ~/Library/Logs/onevox/stdout.log  # macOS
journalctl --user -u onevox -f             # Linux
```

**Common Issues:**
- Hotkey not working → Check permissions, restart daemon
- No audio → Verify device with `onevox devices list`
- Text not appearing → Check accessibility permissions (macOS)

## Debug Mode

```bash
# macOS/Linux
RUST_LOG=debug onevox daemon --foreground

# Windows
$env:RUST_LOG="debug"; onevox daemon --foreground
```
