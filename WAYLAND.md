# Using Onevox on Wayland

Onevox now supports Wayland environments through manual dictation control commands. Since Wayland's security model prevents global hotkey detection, you can bind the start/stop commands to your window manager's keybindings.

## Quick Start

### 1. Start the Daemon

For manual testing:

```bash
onevox daemon --foreground
```

For automatic startup with systemd:

```bash
# Enable and start the service
systemctl --user enable onevox
systemctl --user start onevox

# Check status
systemctl --user status onevox
```

**Note**: If you installed before this fix and the daemon won't connect, reload the service file:

```bash
systemctl --user daemon-reload
systemctl --user restart onevox
```

### 2. Bind Commands to Keys

Add these bindings to your Wayland compositor configuration:

#### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```conf
# Start dictation with Super+Shift+Space
bind = SUPER_SHIFT, Space, exec, onevox start-dictation

# Stop dictation with Super+Shift+Space (same key toggles)
# Or use a different key for stop:
bind = SUPER_SHIFT, Space, exec, onevox stop-dictation
```

#### Sway

Add to `~/.config/sway/config`:

```conf
# Start dictation
bindsym $mod+Shift+Space exec onevox start-dictation

# Stop dictation (can use same key or different)
bindsym $mod+Shift+Space exec onevox stop-dictation
```

### 3. Usage

1. Press your configured keybinding to start dictation
2. Speak into your microphone
3. Press the keybinding again to stop and transcribe
4. The text will be automatically inserted into your active application

## Commands

- `onevox start-dictation` - Start recording audio for dictation
- `onevox stop-dictation` - Stop recording and transcribe the audio
- `onevox status` - Check if daemon is running and current state
- `onevox stop` - Stop the daemon

## Toggle vs Push-to-Talk

The behavior depends on your `config.toml` setting:

```toml
[hotkey]
mode = "toggle"  # Press once to start, press again to stop
# OR
mode = "push-to-talk"  # Hold to record, release to transcribe
```

When using keybindings:
- **Toggle mode**: Call `start-dictation` to begin, call `stop-dictation` to end
- **Push-to-talk mode**: Bind both commands to the same key (press/release)

## Troubleshooting

### Check daemon status
```bash
onevox status
```

### View logs
```bash
# If running in foreground
onevox daemon --foreground

# Check system logs
journalctl --user -u onevox -f
```

### Test audio capture
```bash
onevox test-audio --duration 3
```

### Verify model is loaded
```bash
onevox models downloaded
```

## Performance Notes

- The IPC commands have minimal latency (<1ms typically)
- Audio processing starts immediately when you call `start-dictation`
- Transcription happens when you call `stop-dictation`
- No performance impact compared to hotkey-based triggering

## Configuration

Edit `~/.config/onevox/config.toml` to customize:

- Model selection
- Audio device
- VAD (Voice Activity Detection) settings
- Text injection method
- And more

See `config.example.toml` for all available options.
