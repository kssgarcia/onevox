# Quick Reference

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/install.sh | sh
```

## Permissions

```bash
# Input Monitoring
open "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"

# Accessibility
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"

# Microphone
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"

# Restart
launchctl kickstart -k gui/$(id -u)/com.onevox.daemon
```

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

## Config

```toml
[hotkey]
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

```bash
# View logs
tail -f ~/Library/Logs/onevox/stdout.log

# Debug mode
RUST_LOG=debug onevox daemon --foreground
```

## Troubleshooting

```bash
# Check status
onevox status

# Test hotkey
onevox test-hotkey --hotkey "Cmd+Shift+0"

# Test audio
onevox test-audio --duration 3

# List devices
onevox devices list

# View logs
tail -f ~/Library/Logs/onevox/stdout.log
```

## Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_macos.sh | sh
```
