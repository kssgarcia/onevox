# Installation

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/install.sh | sh
```

## Grant Permissions

macOS requires manual permission grants:

### 1. Input Monitoring (for hotkey)

```bash
open "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
```

Add `Onevox.app` and toggle ON.

### 2. Accessibility (for text injection)

```bash
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
```

Add `Onevox.app` and toggle ON.

### 3. Microphone (for audio)

```bash
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"
```

Toggle Onevox ON (appears after first use).

### 4. Restart Daemon

```bash
launchctl kickstart -k gui/$(id -u)/com.onevox.daemon
```

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
