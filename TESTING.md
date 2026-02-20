# Testing Onevox - Quick Start Guide

## Default Hotkey: `Cmd + Shift + 1`

---

## Quick Test (5 minutes)

### 0. Create Config (Optional)

If you want to customize settings before starting:

```bash
# Create default config file
./target/release/onevox config init

# View the config
cat ~/.config/onevox/config.toml

# Edit if needed (change hotkey, audio device, etc.)
nano ~/.config/onevox/config.toml
```

### 1. Grant Accessibility Permissions

**macOS**: System Settings → Privacy & Security → Accessibility → Enable your Terminal app

### 2. Start the Daemon

```bash
./target/release/onevox daemon
```

**Expected logs**:
```
🚀 Starting Onevox daemon v0.1.0
✅ IPC server started at ...
✅ Onevox daemon is ready
📡 Starting event loop
✅ Dictation engine initialized
✅ Hotkey registered: Cmd+Shift+1
Dictation engine event loop started
```

### 3. Test Hotkey

1. Open **TextEdit** or **Notes**
2. Click in a text field
3. **Press and hold**: `Cmd + Shift + 1`
4. **Speak a few words** (e.g., "hello testing one two three")
5. **Release** the hotkey

**What should happen**:
- Daemon logs show: `Hotkey pressed - starting dictation`
- After you stop speaking and release: Mock text appears in your editor
- Example: `Mock transcription: 1234ms`

### 4. Check Status (in another terminal)

```bash
./target/release/onevox status
```

### 5. Stop the Daemon

```bash
./target/release/onevox stop
```

---

## Supported Hotkey Combinations

You can customize the hotkey in config. Supported keys:

### Modifiers
- `Cmd` (or `Command`, `Super`)
- `Shift`
- `Alt` (or `Option`)
- `Ctrl` (or `Control`)

### Keys
- **Letters**: `A-Z`
- **Numbers**: `0-9` ✅
- **Function keys**: `F1-F12`
- **Special**: `Space`, `Enter`, `Tab`, `Escape`, `Backspace`, `Delete`
- **Arrows**: `Left`, `Right`, `Up`, `Down`

### Examples
- `Cmd+Shift+1` (default)
- `Cmd+Shift+2`
- `Cmd+Alt+D`
- `Ctrl+Shift+Space`
- `Cmd+Shift+F1`

### Change Hotkey

Edit `~/.config/onevox/config.toml`:

```toml
[hotkey]
trigger = "Cmd+Shift+2"  # Change this
mode = "push-to-talk"
```

Or create a config file if it doesn't exist:

```bash
# Easy way:
./target/release/onevox config init

# Then edit:
nano ~/.config/onevox/config.toml
```

---

## Troubleshooting

### "Failed to register hotkey"
- Grant accessibility permissions (see step 1)
- Try a different hotkey combination
- Check if another app is using that hotkey

### "No text appears"
- Ensure you clicked in a text field
- Check daemon logs for errors
- Try TextEdit or Notes.app first (most compatible)
- Verify accessibility permissions are granted

### "Daemon won't start"
- Check if already running: `./target/release/onevox status`
- Kill existing process: `pkill -f "onevox daemon"`
- Remove stale socket: `rm -f /tmp/onevox.sock`

### "Audio device not found"
```bash
./target/release/onevox devices list
# Update config with correct device name
```

---

## Current Limitations

- ✅ **Works**: Hotkey detection, audio capture, VAD, text injection
- ⚠️ **Mock**: Transcription is fake (`"Mock transcription: Xms"`) 
- 🚧 **Not Ready**: Real Whisper.cpp model (build blocked)

See `docs/WHISPER_INTEGRATION.md` for model status.

---

## Expected Behavior

**Normal flow**:
1. Press `Cmd+Shift+1` → Logs: "Hotkey pressed - starting dictation"
2. Audio capture starts → Logs: "🎤 Starting dictation"
3. Speak for 2-3 seconds
4. VAD detects speech → Logs: "🎯 Speech segment detected"
5. Model transcribes → Logs: "📝 Transcription: Mock transcription: ..."
6. Text injector types → Logs: "✅ Text injected successfully"
7. Release hotkey → Logs: "Hotkey released - stopping dictation"
8. Text appears in your focused app

**Success**: Mock text appears in your text editor!

---

## Next Steps

Once basic testing works:
1. Test with different applications (Safari, Chrome, Slack, etc.)
2. Test multiple dictation sessions in a row
3. Test different audio devices
4. Adjust VAD sensitivity if needed
5. Report any bugs or unexpected behavior

---

**Need help?** Check the full testing guide in the main conversation or create an issue.
