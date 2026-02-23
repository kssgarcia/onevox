# Onevox

Local speech-to-text daemon for **macOS**, **Linux**, and **Windows**. Press a hotkey, speak, and your words appear in any app.

## Quick Install

### Pre-built Releases

Download the latest release for your platform from [GitHub Releases](https://github.com/kssgarcia/onevox/releases):

- **macOS ARM64** (Apple Silicon): `onevox-macos-arm64.tar.gz`
- **macOS x86_64** (Intel): `onevox-macos-x86_64.tar.gz`
- **Linux x86_64**: `onevox-v{version}-linux-x86_64.tar.gz`

Or use the automated installers below:

### macOS
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

### Linux
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/install_linux.sh | bash
```

Then setup groups and start service:
```bash
# Add user to required groups
sudo usermod -aG audio,input $USER
# Log out and back in

# Start service
systemctl --user start onevox
systemctl --user enable onevox
```

### Windows
Download the installer from [Releases](https://github.com/kssgarcia/onevox/releases) and run it.
Grant microphone permission when prompted.

## Usage

Press the hotkey, speak, release. Text appears.

**Default Hotkeys:**
- macOS: `Cmd+Shift+0`
- Linux: `Ctrl+Shift+Space`
- Windows: `Ctrl+Shift+Space`

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

### Platform-Specific Service Management

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
journalctl --user -u onevox -f   # View logs
```

**Windows:**
```powershell
Start-Service Onevox    # Start
Stop-Service Onevox     # Stop
Get-Service Onevox      # Status
```

## Configuration

### Config File Locations

- **macOS:** `~/Library/Application Support/com.onevox.onevox/config.toml`
- **Linux:** `~/.config/onevox/config.toml`
- **Windows:** `%APPDATA%\onevox\onevox\config\config.toml`

```bash
# View config
onevox config show

# Initialize default config
onevox config init
```

Key settings:
- `hotkey.trigger` - Change hotkey (platform-specific defaults)
- `hotkey.mode` - "push-to-talk" or "toggle"
- `model.model_path` - Path to Whisper model
- `audio.device` - Audio input device

## Models

### Model Locations

- **macOS:** `~/Library/Caches/com.onevox.onevox/models/`
- **Linux:** `~/.cache/onevox/models/`
- **Windows:** `%LOCALAPPDATA%\onevox\onevox\cache\models\`

```bash
# List available models
onevox models list

# Download a model
onevox models download whisper-base.en

# Check downloaded models
onevox models downloaded
```

Recommended: `whisper-base.en` (good balance of speed and accuracy)

## Logs

### Log Locations

**macOS:**
```bash
tail -f ~/Library/Logs/onevox/stdout.log
```

**Linux:**
```bash
journalctl --user -u onevox -f
# or
tail -f ~/.local/share/onevox/logs/onevox.log
```

**Windows:**
```powershell
Get-Content "$env:APPDATA\onevox\onevox\data\logs\onevox.log" -Wait
```

## Build from Source

```bash
# Clone repo
git clone https://github.com/kssgarcia/onevox.git
cd onevox

# Build with default features (native whisper.cpp)
cargo build --release

# Or with GPU acceleration
cargo build --release --features metal      # macOS
cargo build --release --features cuda       # Linux/Windows NVIDIA
cargo build --release --features vulkan     # Cross-platform GPU
cargo build --release --features openblas   # CPU optimization

# Run
./target/release/onevox daemon --foreground

# Or install locally
./scripts/install_macos.sh
```

### Backend Architecture

OneVox uses **native whisper.cpp bindings** as the primary backend for maximum stability and performance:

- ✅ No subprocess overhead
- ✅ No Python or external runtime dependencies
- ✅ Cross-platform stability (Linux, macOS, Windows)
- ✅ GPU acceleration support (Metal, CUDA, Vulkan, OpenBLAS)
- ✅ Deterministic performance
- ✅ Easy distribution

See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details.

## Uninstall

**macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_macos.sh | sh
```

**Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_linux.sh | bash
```

**Windows:**
```
Use "Add or Remove Programs" in Windows Settings
```

## Troubleshooting

### macOS
**Hotkey not working?**
- Grant Input Monitoring permission
- Restart: `launchctl kickstart -k gui/$(id -u)/com.onevox.daemon`

**Text not appearing?**
- Grant Accessibility permission
- Restart daemon

**No audio?**
- Grant Microphone permission
- Check device: `onevox devices list`

### Linux
**Hotkey not working?**
- Add user to input group: `sudo usermod -aG input $USER`
- Log out and back in
- Check for conflicting hotkeys in your DE

**No audio?**
- Add user to audio group: `sudo usermod -aG audio $USER`
- Check devices: `onevox devices list`
- Verify PulseAudio/ALSA: `pactl list sources short`

**Wayland issues?**
- Some compositors have limited global hotkey support
- Try X11 session as fallback

### Windows
**Hotkey not working?**
- Check Windows Defender isn't blocking
- Ensure no other app uses the same hotkey
- Try running as Administrator

**No audio?**
- Check microphone permissions in Settings → Privacy
- Ensure microphone is set as default device

**Check status (all platforms):**
```bash
onevox status
```

## Requirements

### macOS
- macOS 13.0 or later
- Apple Silicon or Intel
- ~500MB disk space for models

### Linux
- X11 or Wayland display server
- PulseAudio or ALSA
- systemd (optional, for service management)
- ~500MB disk space for models

### Windows
- Windows 10 version 1809 or later
- Windows 11 recommended
- ~500MB disk space for models

## License

MIT

## Documentation

- [QUICKREF.md](QUICKREF.md) - Quick reference card
- [INSTALLATION.md](INSTALLATION.md) - Detailed installation and troubleshooting
- [DEVELOPMENT.md](DEVELOPMENT.md) - Build, test, and development guide
- [ARCHITECTURE.md](ARCHITECTURE.md) - Model pipeline architecture and design decisions
- [MIGRATION.md](MIGRATION.md) - Migration guide for backend refactoring
