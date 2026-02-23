#!/bin/bash
set -euo pipefail

# Onevox Linux Installer
# Supports: Ubuntu, Debian, Fedora, Arch Linux

INSTALL_DIR="${HOME}/.local/bin"
SERVICE_DIR="${HOME}/.config/systemd/user"
DESKTOP_DIR="${HOME}/.local/share/applications"
CONFIG_DIR="${HOME}/.config/onevox"
REPO="${ONEVOX_REPO:-kssgarcia/onevox}"
VERSION="${ONEVOX_VERSION:-latest}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

echo_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64) ASSET="onevox-linux-x86_64.tar.gz" ;;
    aarch64|arm64) ASSET="onevox-linux-arm64.tar.gz" ;;
    *) 
        echo_error "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Detect distribution
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$ID
else
    DISTRO="unknown"
fi

echo_info "Detected: $DISTRO on $ARCH"

# Check dependencies
check_dependencies() {
    local missing_deps=()
    
    # Check for required commands
    for cmd in curl tar; do
        if ! command -v $cmd &> /dev/null; then
            missing_deps+=($cmd)
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo_error "Missing dependencies: ${missing_deps[*]}"
        echo_info "Install them with:"
        case "$DISTRO" in
            ubuntu|debian)
                echo "  sudo apt-get install ${missing_deps[*]}"
                ;;
            fedora|rhel|centos)
                echo "  sudo dnf install ${missing_deps[*]}"
                ;;
            arch|manjaro)
                echo "  sudo pacman -S ${missing_deps[*]}"
                ;;
        esac
        exit 1
    fi
}

# Download and extract
download_onevox() {
    echo_info "Downloading Onevox $VERSION..." >&2
    
    local TMP_DIR=$(mktemp -d)
    local EXTRACT_DIR="$TMP_DIR/extract"
    mkdir -p "$EXTRACT_DIR"
    
    if [ "$VERSION" = "latest" ]; then
        URL="https://github.com/$REPO/releases/latest/download/$ASSET"
    else
        URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
    fi
    
    echo_info "Downloading from: $URL" >&2
    curl -fsSL "$URL" -o "$TMP_DIR/$ASSET"
    
    echo_info "Extracting..." >&2
    tar -xzf "$TMP_DIR/$ASSET" -C "$EXTRACT_DIR"
    
    # Find the binary
    BINARY=$(find "$EXTRACT_DIR" -name "onevox" -type f 2>/dev/null | head -n 1 || true)
    
    if [ -z "$BINARY" ]; then
        echo_error "Binary not found in archive" >&2
        echo_error "All files in archive:" >&2
        find "$EXTRACT_DIR" -type f >&2
        rm -rf "$TMP_DIR"
        exit 1
    fi
    
    # Make sure it's executable
    chmod +x "$BINARY"
    
    echo "$BINARY"
}

# Install binary
install_binary() {
    local binary=$1
    
    echo_info "Installing binary to $INSTALL_DIR..."
    mkdir -p "$INSTALL_DIR"
    install -m 755 "$binary" "$INSTALL_DIR/onevox"
    
    # Add to PATH if not already there
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo_warn "$INSTALL_DIR is not in PATH"
        echo_info "Add this to your ~/.bashrc or ~/.zshrc:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
}

# Install systemd service
install_service() {
    echo_info "Installing systemd user service..."
    mkdir -p "$SERVICE_DIR"
    
    cat > "$SERVICE_DIR/onevox.service" <<'EOF'
[Unit]
Description=Onevox Speech-to-Text Daemon
Documentation=https://github.com/kssgarcia/onevox
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.local/bin/onevox daemon --foreground
Restart=on-failure
RestartSec=5s
StandardOutput=journal
StandardError=journal

# Security hardening
PrivateTmp=true
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=%h/.local/share/onevox %h/.cache/onevox %h/.config/onevox

# Resource limits
MemoryMax=1G
CPUQuota=50%

[Install]
WantedBy=default.target
EOF

    # Reload systemd
    systemctl --user daemon-reload
    
    # Enable service
    systemctl --user enable onevox.service
    
    echo_info "Service installed and enabled"
}

# Create desktop entry
install_desktop_entry() {
    echo_info "Creating desktop entry..."
    mkdir -p "$DESKTOP_DIR"
    
    cat > "$DESKTOP_DIR/onevox.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=Onevox
Comment=Local Speech-to-Text
Exec=onevox tui
Icon=audio-input-microphone
Terminal=true
Categories=Utility;Audio;Accessibility;
Keywords=speech;transcription;dictation;voice;
EOF

    # Update desktop database if available
    if command -v update-desktop-database &> /dev/null; then
        update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    fi
}

# Create initial config
create_config() {
    echo_info "Creating initial configuration..."
    mkdir -p "$CONFIG_DIR"
    
    if [ ! -f "$CONFIG_DIR/config.toml" ]; then
        "$INSTALL_DIR/onevox" config init || true
    fi
}

# Check audio group membership
check_audio_group() {
    if ! groups | grep -q audio; then
        echo_warn "User is not in 'audio' group"
        echo_info "Add yourself to the audio group:"
        echo "  sudo usermod -aG audio $USER"
        echo "  Then log out and back in"
    fi
}

# Check input group membership (for hotkeys)
check_input_group() {
    if ! groups | grep -q input; then
        echo_warn "User is not in 'input' group (needed for hotkeys)"
        echo_info "Add yourself to the input group:"
        echo "  sudo usermod -aG input $USER"
        echo "  Then log out and back in"
    fi
}

# Print post-install instructions
print_instructions() {
    echo ""
    echo_info "✅ Onevox installed successfully!"
    echo ""
    echo "📍 Installation locations:"
    echo "  Binary:  $INSTALL_DIR/onevox"
    echo "  Service: $SERVICE_DIR/onevox.service"
    echo "  Config:  $CONFIG_DIR/config.toml"
    echo ""
    echo "🚀 Quick start:"
    echo "  1. Start the service:"
    echo "     systemctl --user start onevox"
    echo ""
    echo "  2. Check status:"
    echo "     systemctl --user status onevox"
    echo "     onevox status"
    echo ""
    echo "  3. View logs:"
    echo "     journalctl --user -u onevox -f"
    echo ""
    echo "  4. Open TUI:"
    echo "     onevox tui"
    echo ""
    echo "⚙️  Configuration:"
    echo "  Edit: $CONFIG_DIR/config.toml"
    echo "  Default hotkey: Ctrl+Shift+Space (Linux)"
    echo ""
    echo "📦 Download a model:"
    echo "  onevox models list"
    echo "  onevox models download whisper-base.en"
    echo ""
    
    # Display environment-specific notes
    if [ "$XDG_SESSION_TYPE" = "wayland" ]; then
        echo "🔔 Wayland detected:"
        echo "  Some compositors may require additional configuration"
        echo "  for global hotkeys and text injection."
        echo ""
    fi
    
    # Check for missing group memberships
    if ! groups | grep -q audio || ! groups | grep -q input; then
        echo "⚠️  Group membership:"
        check_audio_group
        check_input_group
        echo ""
    fi
    
    echo "📚 Documentation:"
    echo "  https://github.com/$REPO"
    echo ""
}

# Main installation
main() {
    echo "╔════════════════════════════════════════╗"
    echo "║   Onevox Linux Installer               ║"
    echo "╚════════════════════════════════════════╝"
    echo ""
    
    check_dependencies
    
    BINARY=$(download_onevox)
    install_binary "$BINARY"
    
    # Clean up temp directory
    TMP_DIR=$(dirname "$(dirname "$BINARY")")
    rm -rf "$TMP_DIR"
    
    install_service
    install_desktop_entry
    create_config
    
    print_instructions
}

main "$@"
