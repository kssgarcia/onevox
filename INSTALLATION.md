# Installation

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/install.sh | sh
```

## Grant Permissions

macOS requires manual permission grants **in this order**:

### 1. Input Monitoring (for hotkey) - FIRST

```bash
open "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
```

Add `Onevox.app` and toggle ON.

### 2. Accessibility (for text injection) - SECOND

```bash
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
```

Add `Onevox.app` and toggle ON.

### 3. Restart Daemon - REQUIRED

```bash
launchctl kickstart -k gui/$(id -u)/com.onevox.daemon
```

```bash
tail -30 ~/Library/Logs/onevox/stdout.log
```

**Important**: You must restart the daemon after granting permissions!

### 4. Microphone (for audio) - APPEARS AUTOMATICALLY

The microphone permission will appear automatically when you first press the hotkey (Cmd+Shift+0). macOS will prompt you to grant it.

If it doesn't appear in System Settings, it means the daemon hasn't tried to use the microphone yet. Press the hotkey to trigger the prompt.

## Verify

```bash
onevox status
```

Test: Press Cmd+Shift+0, speak, release.

## Troubleshooting

### Hotkey not working
- Check Input Monitoring permission
- Restart daemon
- Check logs: `tail -f ~/Library/Logs/onevox/stdout.log`

### Text not appearing
- Check Accessibility permission
- Restart daemon

### No audio
- Check Microphone permission
- List devices: `onevox devices list`
- Test: `onevox test-audio --duration 3`

### Daemon not running
- Check: `launchctl print gui/$(id -u)/com.onevox.daemon`
- Logs: `tail -f ~/Library/Logs/onevox/stdout.log`
- Reinstall if needed

## Paths

- App: `~/Applications/Onevox.app`
- CLI: `/usr/local/bin/onevox`
- Config: `~/Library/Application Support/com.onevox.onevox/config.toml`
- Models: `~/Library/Application Support/com.onevox.onevox/models/`
- Logs: `~/Library/Logs/onevox/`
- LaunchAgent: `~/Library/LaunchAgents/com.onevox.daemon.plist`

## Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_macos.sh | sh
```
