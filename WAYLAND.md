# Wayland Setup

Wayland's security model prevents global hotkey detection. Use manual keybindings instead.

## Setup

**1. Start the daemon:**
```bash
systemctl --user enable --now onevox
```

**2. Bind commands to your compositor:**

### Hyprland

Add to `~/.config/hypr/keybidings.conf`:
```conf
bind = SUPER_SHIFT, Space, exec, onevox start-dictation
bind = SUPER_SHIFT, Space, exec, onevox stop-dictation
```

### Sway

Add to `~/.config/sway/config`:
```conf
bindsym $mod+Shift+Space exec onevox start-dictation
bindsym $mod+Shift+Space exec onevox stop-dictation
```

## Usage

1. Press your keybinding to start recording
2. Speak
3. Press again to stop and transcribe
4. Text appears in active application

## Commands

```bash
onevox start-dictation  # Start recording
onevox stop-dictation   # Stop and transcribe
onevox status           # Check daemon status
```

## Configuration

Edit `~/.config/onevox/config.toml`:

```toml
[hotkey]
mode = "toggle"  # Press once to start, again to stop
# OR
mode = "push-to-talk"  # Bind both commands to same key
```

## Troubleshooting

```bash
# Check status
onevox status

# View logs
journalctl --user -u onevox -f

# Test audio
onevox test-audio --duration 3
```
