# Installation

## macOS

```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/install.sh | sh
```

**Grant Permissions (Required):**

1. Input Monitoring: `open "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"`
2. Accessibility: `open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"`
3. Restart daemon: `launchctl kickstart -k gui/$(id -u)/com.onevox.daemon`
4. Microphone permission will prompt automatically on first use

**Test:** Press `Cmd+Shift+0`, speak, release.

**Paths:**
- Config: `~/Library/Application Support/com.onevox.onevox/config.toml`
- Models: `~/Library/Caches/com.onevox.onevox/models/`
- Logs: `~/Library/Logs/onevox/stdout.log`

**Service Management:**
```bash
# Start daemon
launchctl start com.onevox.daemon

# Stop daemon
launchctl stop com.onevox.daemon

# Restart daemon (use after permission changes)
launchctl kickstart -k gui/$(id -u)/com.onevox.daemon

# Check if running
launchctl list | grep onevox

# Unload service
launchctl unload ~/Library/LaunchAgents/com.onevox.daemon.plist

# Load service
launchctl load ~/Library/LaunchAgents/com.onevox.daemon.plist
```

**View Logs:**
```bash
# Tail logs (follow)
tail -f ~/Library/Logs/onevox/stdout.log

# View last 50 lines
tail -50 ~/Library/Logs/onevox/stdout.log

# View errors only
grep -i error ~/Library/Logs/onevox/stdout.log
```

**Useful Commands:**
```bash
# Check status
onevox status

# List audio devices
onevox devices list

# Test audio capture
onevox test-audio --duration 3

# View configuration
onevox config show

# Download model
onevox models download whisper-base.en

# View history
onevox history list
```

---

## Linux

```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/install_linux.sh | bash

# Add user to required groups
sudo usermod -aG audio,input $USER
# Log out and back in

# Start service
systemctl --user enable --now onevox
```

**Test:** Press `Ctrl+Shift+Space`, speak, release.

**Paths:**
- Config: `~/.config/onevox/config.toml`
- Models: `~/.cache/onevox/models/`
- Logs: `~/.local/share/onevox/logs/onevox.log`

**Service Management:**
```bash
# Start daemon
systemctl --user start onevox

# Stop daemon
systemctl --user stop onevox

# Restart daemon
systemctl --user restart onevox

# Enable auto-start on boot
systemctl --user enable onevox

# Disable auto-start
systemctl --user disable onevox

# Check status
systemctl --user status onevox

# Reload service file (after editing)
systemctl --user daemon-reload
systemctl --user restart onevox
```

**View Logs:**
```bash
# Follow logs in real-time
journalctl --user -u onevox -f

# View last 50 lines
journalctl --user -u onevox -n 50

# View logs since boot
journalctl --user -u onevox -b

# View logs from today
journalctl --user -u onevox --since today

# View errors only
journalctl --user -u onevox -p err

# Alternative: direct log file
tail -f ~/.local/share/onevox/logs/onevox.log
```

**Useful Commands:**
```bash
# Check status
onevox status

# List audio devices
onevox devices list

# Test audio capture
onevox test-audio --duration 3

# View configuration
onevox config show

# Download model
onevox models download whisper-base.en

# View history
onevox history list

# Check group membership
groups | grep -E 'audio|input'

# Test PulseAudio
pactl list sources short

# Test ALSA
arecord -l
```

**Wayland:** See [WAYLAND.md](WAYLAND.md) for manual keybinding setup.

---

## Windows

Download installer from [Releases](https://github.com/kssgarcia/onevox/releases) and run it.

**Test:** Press `Ctrl+Shift+Space`, speak, release.

**Paths:**
- Config: `%APPDATA%\onevox\onevox\config\config.toml`
- Models: `%LOCALAPPDATA%\onevox\onevox\cache\models\`
- Logs: `%APPDATA%\onevox\onevox\data\logs\onevox.log`

**Service Management:**
```powershell
# Start service
Start-Service Onevox

# Stop service
Stop-Service Onevox

# Restart service
Restart-Service Onevox

# Check status
Get-Service Onevox

# Set to start automatically
Set-Service -Name Onevox -StartupType Automatic

# Set to manual start
Set-Service -Name Onevox -StartupType Manual

# View service details
Get-Service Onevox | Format-List *
```

**View Logs:**
```powershell
# Follow logs in real-time
Get-Content "$env:APPDATA\onevox\onevox\data\logs\onevox.log" -Wait

# View last 50 lines
Get-Content "$env:APPDATA\onevox\onevox\data\logs\onevox.log" -Tail 50

# Search for errors
Select-String -Path "$env:APPDATA\onevox\onevox\data\logs\onevox.log" -Pattern "error" -CaseSensitive:$false

# View event log
Get-EventLog -LogName Application -Source Onevox -Newest 20
```

**Useful Commands:**
```powershell
# Check status
onevox status

# List audio devices
onevox devices list

# Test audio capture
onevox test-audio --duration 3

# View configuration
onevox config show

# Download model
onevox models download whisper-base.en

# View history
onevox history list

# Open microphone settings
start ms-settings:privacy-microphone
```

---

## Troubleshooting

**Hotkey not working?**
- macOS: Restart daemon after granting permissions
- Linux: Ensure you're in `input` group and logged out/in
- Windows: Check no other app uses the same hotkey

**No audio?**
- Run `onevox devices list` to verify microphone
- Linux: Ensure you're in `audio` group
- Test: `onevox test-audio --duration 3`

**Text not appearing?**
- macOS: Grant Accessibility permission
- Check logs for errors

**Check status:** `onevox status`

---

## Build from Source

```bash
git clone https://github.com/kssgarcia/onevox.git
cd onevox
cargo build --release
```

**Platform dependencies:**
- macOS: Xcode Command Line Tools
- Linux: `build-essential pkg-config libasound2-dev libpulse-dev`
- Windows: Visual Studio Build Tools

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed build instructions.
