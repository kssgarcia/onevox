# Installation

## macOS

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/install.sh | sh
```

### Grant Permissions

macOS requires manual permission grants **in this order**:

#### 1. Input Monitoring (for hotkey) - FIRST

```bash
open "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
```

Add `Onevox.app` and toggle ON.

#### 2. Accessibility (for text injection) - SECOND

```bash
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
```

Add `Onevox.app` and toggle ON.

#### 3. Restart Daemon - REQUIRED

```bash
launchctl kickstart -k gui/$(id -u)/com.onevox.daemon
```

```bash
tail -30 ~/Library/Logs/onevox/stdout.log
```

**Important**: You must restart the daemon after granting permissions!

#### 4. Microphone (for audio) - APPEARS AUTOMATICALLY

The microphone permission will appear automatically when you first press the hotkey (Cmd+Shift+0). macOS will prompt you to grant it.

### Verify

```bash
onevox status
```

Test: Press **Cmd+Shift+0**, speak, release.

### Paths

- App: `~/Applications/Onevox.app`
- CLI: `/usr/local/bin/onevox`
- Config: `~/Library/Application Support/com.onevox.onevox/config.toml`
- Models: `~/Library/Caches/com.onevox.onevox/models/`
- Logs: `~/Library/Logs/onevox/`
- LaunchAgent: `~/Library/LaunchAgents/com.onevox.daemon.plist`

### Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_macos.sh | sh
```

---

## Linux

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/install_linux.sh | bash
```

### Setup

#### 1. Add User to Groups

```bash
# For audio access
sudo usermod -aG audio $USER

# For hotkey access (optional but recommended)
sudo usermod -aG input $USER

# Log out and back in for changes to take effect
```

#### 2. Start Service

```bash
# Enable and start
systemctl --user enable --now onevox

# Check status
systemctl --user status onevox
```

### Verify

```bash
onevox status
```

Test: Press **Ctrl+Shift+Space**, speak, release.

### Desktop Environment Notes

**GNOME (Wayland):**
```bash
# May need extension for global hotkeys
sudo apt install gnome-shell-extension-appindicator
```

**KDE Plasma:**
Works out of the box on both X11 and Wayland.

**XFCE:**
Works on X11. Configure hotkeys in Settings → Keyboard if needed.

**i3/Sway:**
Add to your config:
```
exec --no-startup-id systemctl --user start onevox
```

### Paths

- Binary: `~/.local/bin/onevox`
- Config: `~/.config/onevox/config.toml`
- Models: `~/.cache/onevox/models/`
- Data: `~/.local/share/onevox/`
- Service: `~/.config/systemd/user/onevox.service`
- Desktop: `~/.local/share/applications/onevox.desktop`

### Logs

```bash
# View logs
journalctl --user -u onevox -f

# Or
tail -f ~/.local/share/onevox/logs/onevox.log
```

### Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_linux.sh | bash
```

---

## Windows

### Quick Install

1. Download installer from [Releases](https://github.com/kssgarcia/onevox/releases)
2. Run `onevox-windows-x86_64.msi`
3. Follow installation wizard

### Manual Installation

```powershell
# Download and extract
Invoke-WebRequest -Uri "https://github.com/kssgarcia/onevox/releases/latest/download/onevox-windows-x86_64.zip" -OutFile "onevox.zip"
Expand-Archive onevox.zip -DestinationPath "$env:ProgramFiles\Onevox"

# Add to PATH
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:ProgramFiles\Onevox", "User")
```

### Setup

#### 1. Grant Microphone Permission

1. Open Settings → Privacy → Microphone
2. Enable "Allow apps to access your microphone"
3. Scroll down and enable for "Onevox"

Or run:
```powershell
start ms-settings:privacy-microphone
```

#### 2. Start Service

```powershell
Start-Service Onevox
```

### Verify

```powershell
onevox status
```

Test: Press **Ctrl+Shift+Space**, speak, release.

### Paths

- Binary: `C:\Program Files\Onevox\onevox.exe`
- Config: `%APPDATA%\onevox\onevox\config\config.toml`
- Models: `%LOCALAPPDATA%\onevox\onevox\cache\models\`
- Data: `%APPDATA%\onevox\onevox\data\`
- Logs: `%APPDATA%\onevox\onevox\data\logs\`

### Logs

```powershell
Get-Content "$env:APPDATA\onevox\onevox\data\logs\onevox.log" -Wait
```

### Uninstall

Use "Add or Remove Programs" in Windows Settings.

---

## Troubleshooting

### macOS

**Hotkey not working?**
- Check Input Monitoring permission
- Restart daemon: `launchctl kickstart -k gui/$(id -u)/com.onevox.daemon`
- Check logs: `tail -f ~/Library/Logs/onevox/stdout.log`

**Text not appearing?**
- Check Accessibility permission
- Restart daemon

**No audio?**
- Check Microphone permission
- List devices: `onevox devices list`
- Test: `onevox test-audio --duration 3`

### Linux

**Hotkey not working?**
- Check if in input group: `groups | grep input`
- Add to group: `sudo usermod -aG input $USER` (log out/in)
- Check for conflicting hotkeys in your DE
- Try: `sudo onevox daemon --foreground`

**No audio?**
- Check if in audio group: `groups | grep audio`
- Add to group: `sudo usermod -aG audio $USER` (log out/in)
- List devices: `onevox devices list`
- Check PulseAudio: `pactl list sources short`
- Check ALSA: `arecord -l`

**Wayland issues?**
- Some compositors have limited global hotkey support
- Try X11 session as fallback
- Check compositor-specific documentation

**Service not starting?**
```bash
systemctl --user status onevox
journalctl --user -u onevox -n 50
```

### Windows

**Hotkey not working?**
- Check Windows Defender isn't blocking
- Ensure no other app uses the same hotkey
- Try running as Administrator
- Check for conflicting software

**No audio?**
- Check microphone permissions in Settings → Privacy
- Ensure microphone is set as default device
- Check Windows Sound settings
- Test: `onevox test-audio --duration 3`

**Service not starting?**
```powershell
Get-Service Onevox
Get-EventLog -LogName Application -Source Onevox -Newest 10
```

---

## Build from Source

### All Platforms

```bash
# Clone repository
git clone https://github.com/kssgarcia/onevox.git
cd onevox

# Build release
cargo build --release

# Binary location
./target/release/onevox
```

### Platform-Specific Dependencies

**macOS:**
```bash
# Xcode Command Line Tools
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install build-essential pkg-config libasound2-dev libpulse-dev
```

**Linux (Fedora):**
```bash
sudo dnf install gcc pkg-config alsa-lib-devel pulseaudio-libs-devel
```

**Linux (Arch):**
```bash
sudo pacman -S base-devel alsa-lib pulseaudio
```

**Windows:**
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
- Install [Rust](https://rustup.rs/)

### Install Locally

**macOS:**
```bash
./scripts/install_macos.sh
```

**Linux:**
```bash
./scripts/install_linux.sh
```

**Windows:**
```powershell
.\scripts\install_windows.ps1
```
