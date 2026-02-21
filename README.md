# Onevox

Local speech-to-text daemon for macOS. Press a hotkey, speak, and your words appear in any app.

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/install.sh | sh
```

Then grant permissions (required by macOS):

```bash
# 1. Input Monitoring (for hotkey)
open "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
# Add Onevox.app and toggle ON

# 2. Accessibility (for text injection)
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
# Add Onevox.app and toggle ON

# 3. Restart daemon (REQUIRED!)
launchctl kickstart -k gui/$(id -u)/com.onevox.daemon

# 4. Microphone permission will prompt automatically when you press the hotkey
```

## Usage

Press **Cmd+Shift+0**, speak, release. Text appears.

## Commands

```bash
onevox daemon          # Start daemon
onevox stop            # Stop daemon
onevox status          # Check status
onevox tui             # Open terminal UI
onevox devices list    # List audio devices
onevox models list     # List available models
onevox models download <model-id>  # Download a model
onevox history list    # View transcription history
onevox config show     # Show configuration
```

## Configuration

Config file: `~/Library/Application Support/com.onevox.onevox/config.toml`

```bash
# Edit config
nano ~/Library/Application\ Support/com.onevox.onevox/config.toml

# Or use onevox command
onevox config show
```

Key settings:
- `hotkey.trigger` - Change hotkey (default: "Cmd+Shift+0")
- `hotkey.mode` - "push-to-talk" or "toggle"
- `model.model_path` - Path to Whisper model
- `audio.device` - Audio input device

## Models

Models location: `~/Library/Application Support/com.onevox.onevox/models/`

```bash
# List available models
onevox models list

# Download a model
onevox models download whisper-base.en

# Models are stored in:
~/Library/Application Support/com.onevox.onevox/models/whisper-base.en/
```

Recommended: `whisper-base.en` (good balance of speed and accuracy)

## Logs

```bash
# View logs
tail -f ~/Library/Logs/onevox/stdout.log

# Log location
~/Library/Logs/onevox/
```

## Build from Source

```bash
# Clone repo
git clone https://github.com/kssgarcia/onevox.git
cd onevox

# Build
cargo build --release

# Run
./target/release/onevox daemon --foreground

# Or install locally
./scripts/install_macos.sh
```

## Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_macos.sh | sh
```

## Troubleshooting

**Hotkey not working?**
- Grant Input Monitoring permission
- Restart: `launchctl kickstart -k gui/$(id -u)/com.onevox.daemon`

**Text not appearing?**
- Grant Accessibility permission
- Restart daemon

**No audio?**
- Grant Microphone permission
- Check device: `onevox devices list`

**Check status:**
```bash
onevox status
tail -f ~/Library/Logs/onevox/stdout.log
```

## Requirements

- macOS 13.0+
- Apple Silicon or Intel
- ~500MB disk space for models

## License

MIT

## Documentation

- [QUICKREF.md](QUICKREF.md) - Quick reference card
- [INSTALLATION.md](INSTALLATION.md) - Detailed installation and troubleshooting
- [DEVELOPMENT.md](DEVELOPMENT.md) - Build, test, and development guide
