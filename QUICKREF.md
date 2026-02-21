# Quick Reference

## Install

### macOS
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/install.sh | sh
```

### Linux
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/install_linux.sh | bash
```

### Windows
Download from releases and run installer (or build from source)

## Permissions

### macOS
```bash
# Input Monitoring
open "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"

# Accessibility
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"

# Microphone
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"

# Restart daemon
launchctl kickstart -k gui/$(id -u)/com.onevox.daemon
```

### Linux
```bash
# Check permissions
./scripts/check_permissions.sh

# Restart daemon
systemctl --user restart onevox
```

### Windows
Run as Administrator for global hotkey access

## Commands

```bash
onevox daemon              # Start
onevox stop                # Stop
onevox status              # Status
onevox tui                 # Terminal UI
onevox devices list        # Audio devices
onevox models list         # Available models
onevox models download <id>  # Download model
onevox history list        # History
onevox config show         # Config
```

## Paths

### macOS
```bash
# Config
~/Library/Application Support/com.onevox.onevox/config.toml

# Models
~/Library/Application Support/com.onevox.onevox/models/

# Logs
~/Library/Logs/onevox/stdout.log

# App
~/Applications/Onevox.app

# LaunchAgent
~/Library/LaunchAgents/com.onevox.daemon.plist
```

### Linux
```bash
# Config
~/.config/onevox/config.toml

# Models
~/.local/share/onevox/models/

# Logs
~/.local/share/onevox/logs/onevox.log
# Or view with: journalctl --user -u onevox

# Binary
~/.local/bin/onevox

# Systemd service
~/.config/systemd/user/onevox.service
```

### Windows
```powershell
# Config
%APPDATA%\onevox\config.toml

# Models
%APPDATA%\onevox\models\

# Logs
%APPDATA%\onevox\logs\onevox.log

# Binary
%LOCALAPPDATA%\Programs\onevox\onevox.exe
```

## Config

```toml
[hotkey]
# Platform-specific defaults:
# macOS: "Cmd+Shift+0"
# Linux/Windows: "Ctrl+Shift+Space"
trigger = "Cmd+Shift+0"
mode = "push-to-talk"  # or "toggle"

[model]
model_path = "ggml-base.en.bin"
language = "en"

[audio]
device = "default"
sample_rate = 16000
```

## Logs

### macOS
```bash
# View logs
tail -f ~/Library/Logs/onevox/stdout.log

# Debug mode
RUST_LOG=debug onevox daemon --foreground
```

### Linux
```bash
# View logs
tail -f ~/.local/share/onevox/logs/onevox.log

# Or use journalctl
journalctl --user -u onevox -f

# Debug mode
RUST_LOG=debug onevox daemon --foreground
```

### Windows
```powershell
# View logs
Get-Content -Wait %APPDATA%\onevox\logs\onevox.log

# Debug mode
$env:RUST_LOG="debug"; onevox daemon --foreground
```

## Troubleshooting

```bash
# Check status
onevox status

# Test hotkey (use platform-specific combo)
# macOS
onevox test-hotkey --hotkey "Cmd+Shift+0"
# Linux/Windows
onevox test-hotkey --hotkey "Ctrl+Shift+Space"

# Test audio
onevox test-audio --duration 3

# List devices
onevox devices list

# View logs (see Logs section above for platform-specific commands)
```

## Uninstall

### macOS
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_macos.sh | sh
```

### Linux
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_linux.sh | bash
# Or if installed locally:
./scripts/uninstall_linux.sh
```

### Windows
Use Windows "Add or Remove Programs" or delete manually from `%LOCALAPPDATA%\Programs\onevox\`
