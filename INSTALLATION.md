# Installation

## Quick Install

### Universal Installer (Recommended)

**macOS & Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/install.sh | sh
```

This single command detects your platform and installs the appropriate version.

**Linux Post-Install:**
```bash
# Add user to required groups (required for audio and hotkeys)
sudo usermod -aG audio,input $USER
# Log out and back in for group changes to take effect

# Start and enable service
systemctl --user enable --now onevox
```

**Windows:**
```powershell
$onevoxDir = "$env:LOCALAPPDATA\onevox"
$asset = "onevox-windows-x86_64.zip"
Invoke-WebRequest -Uri "https://github.com/kssgarcia/onevox/releases/latest/download/$asset" -OutFile $asset
Expand-Archive -Path $asset -DestinationPath $onevoxDir -Force
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$onevoxDir", [EnvironmentVariableTarget]::User)
$env:Path += ";$onevoxDir"
& "$onevoxDir\onevox.exe" --version
```

---

## Build Variants

OneVox is available in two build configurations:

### Default: whisper.cpp (Recommended)
- **Best for**: Most users, production use
- **Pros**: Fast, stable, single binary, GPU acceleration
- **Memory**: ~100MB
- **Latency**: 50-200ms
- **Models**: Whisper GGML models (tiny, base, small, medium, large)

**Installation**: Standard installer provides this by default

### Experimental: ONNX Runtime
- **Best for**: Multilingual use cases, research
- **Pros**: 25+ languages, CTC models, INT8 quantization
- **Memory**: ~250MB
- **Latency**: Varies by model
- **Models**: Parakeet, custom ONNX models
- **Platform Support**: ARM64 macOS, Linux (x86_64 macOS not supported by ONNX Runtime)

**Installation**: Build from source with ONNX feature (see [Build from Source](#build-from-source) below)

---

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

### Quick Install
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/install.sh | sh

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

**Service Management with SCM (run PowerShell as Administrator):**
```powershell
# Register service (one-time)
sc.exe create Onevox binPath= "\"$env:LOCALAPPDATA\onevox\onevox.exe\" daemon --foreground" start= auto

# Start / Stop / Restart
sc.exe start Onevox
sc.exe stop Onevox
sc.exe stop Onevox; sc.exe start Onevox

# Status
sc.exe query Onevox

# Remove service
sc.exe delete Onevox
```

**Auto-start:** The `start= auto` flag above configures startup with SCM.

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

### Prerequisites

**All platforms:**
- Rust 1.93+ ([rustup.rs](https://rustup.rs))
- Git

**macOS:**
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install build-essential pkg-config cmake libasound2-dev libpulse-dev
```

**Linux (Fedora):**
```bash
sudo dnf install gcc pkg-config cmake alsa-lib-devel pulseaudio-libs-devel
```

**Linux (Arch):**
```bash
sudo pacman -S base-devel cmake alsa-lib pulseaudio
```

**Windows:**
- Visual Studio Build Tools with C++ support

### Build Default (whisper.cpp + ONNX)

```bash
git clone https://github.com/kssgarcia/onevox.git
cd onevox

# macOS ARM64 (M1/M2/M3) - includes ONNX by default
CC=clang CXX=clang++ SDKROOT=$(xcrun --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=13.0 \
  cargo build --release

# macOS x86_64 (Intel) - ONNX not available, use whisper.cpp only
CC=clang CXX=clang++ SDKROOT=$(xcrun --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=13.0 \
  cargo build --release --no-default-features --features whisper-cpp,overlay-indicator

# Linux - includes ONNX by default
cargo build --release

# Windows - includes ONNX by default
cargo build --release

# Install locally
./target/release/onevox --version
```

**Note**: ONNX Runtime does not provide prebuilt binaries for x86_64 (Intel) macOS. Use whisper.cpp models on Intel Macs.

### Build with Only whisper.cpp

If you want to disable ONNX and use only whisper.cpp:

```bash
# All platforms
cargo build --release --no-default-features --features whisper-cpp,overlay-indicator
```

### Configure Model

After building, edit your config file to select a model:

```toml
[model]
# Backend is auto-detected from model_path
model_path = "ggml-base.en"         # English-only (whisper.cpp)
# model_path = "ggml-base"          # Multilingual (whisper.cpp, 99+ languages)
# model_path = "parakeet-ctc-0.6b"  # ONNX model (included by default)

device = "auto"  # auto, cpu, gpu
preload = true
```

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed build instructions and troubleshooting.

---

## Uninstall

### macOS

**Quick Uninstall:**
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_macos.sh | bash
```

**Manual Uninstall:**
```bash
# Stop launchd service
launchctl bootout "gui/$(id -u)" ~/Library/LaunchAgents/com.onevox.daemon.plist

# Remove service file
rm -f ~/Library/LaunchAgents/com.onevox.daemon.plist

# Remove application
rm -rf ~/Applications/Onevox.app
rm -rf /Applications/Onevox.app  # If installed system-wide

# Remove CLI symlinks
rm -f /usr/local/bin/onevox
rm -f /opt/homebrew/bin/onevox
rm -f ~/.local/bin/onevox

# Remove config and data
rm -rf ~/Library/Application\ Support/com.onevox.onevox
rm -rf ~/Library/Caches/com.onevox.onevox
rm -rf ~/Library/Logs/onevox
```

---

### Linux

**Quick Uninstall:**
```bash
curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_linux.sh | bash
```

**Manual Uninstall:**
```bash
# Stop and disable service
systemctl --user stop onevox.service
systemctl --user disable onevox.service

# Remove service file
rm -f ~/.config/systemd/user/onevox.service
systemctl --user daemon-reload

# Remove binary
rm -f ~/.local/bin/onevox

# Remove desktop entry
rm -f ~/.local/share/applications/onevox.desktop
update-desktop-database ~/.local/share/applications 2>/dev/null || true

# Remove config and data
rm -rf ~/.config/onevox
rm -rf ~/.local/share/onevox
rm -rf ~/.cache/onevox
```

---

### Windows

**Quick Uninstall:**
```powershell
# Download and run uninstall script
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/uninstall_windows.ps1" -OutFile uninstall_windows.ps1
.\uninstall_windows.ps1
```

**Options:**
```powershell
# Keep configuration files
.\uninstall_windows.ps1 -KeepConfig

# Skip confirmation prompt
.\uninstall_windows.ps1 -Force

# Both options
.\uninstall_windows.ps1 -KeepConfig -Force
```

**Manual Uninstall:**
```powershell
# Stop and remove service (run as Administrator)
sc.exe stop Onevox
sc.exe delete Onevox

# Remove from PATH (requires reopening PowerShell after)
$userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
$newPath = ($userPath -split ';' | Where-Object { $_ -ne "$env:LOCALAPPDATA\onevox" }) -join ';'
[Environment]::SetEnvironmentVariable("Path", $newPath, [EnvironmentVariableTarget]::User)

# Remove files and directories
Remove-Item -Path "$env:LOCALAPPDATA\onevox" -Recurse -Force
Remove-Item -Path "$env:APPDATA\onevox" -Recurse -Force
```

---

## What Gets Removed

The uninstaller removes the following on all platforms:

**Binaries:**
- Application executable
- CLI tools and symlinks

**Services:**
- System service/daemon registration
- LaunchAgent (macOS) / systemd service (Linux) / Windows Service

**Data:**
- Configuration files
- Downloaded models (can be large, ~100MB-3GB)
- Transcription history
- Application logs
- Cache files

**Note:** On Windows, use the `-KeepConfig` flag to preserve your configuration and downloaded models if you plan to reinstall later.
