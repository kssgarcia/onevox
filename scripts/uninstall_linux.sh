#!/bin/bash
set -euo pipefail

# Onevox Linux Uninstaller

INSTALL_DIR="${HOME}/.local/bin"
SERVICE_DIR="${HOME}/.config/systemd/user"
DESKTOP_DIR="${HOME}/.local/share/applications"
CONFIG_DIR="${HOME}/.config/onevox"
DATA_DIR="${HOME}/.local/share/onevox"
CACHE_DIR="${HOME}/.cache/onevox"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

echo_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Confirm uninstall
confirm_uninstall() {
    echo "This will remove Onevox and all its data."
    echo ""
    echo "The following will be deleted:"
    echo "  • Binary: $INSTALL_DIR/onevox"
    echo "  • Service: $SERVICE_DIR/onevox.service"
    echo "  • Config: $CONFIG_DIR"
    echo "  • Data: $DATA_DIR"
    echo "  • Cache: $CACHE_DIR"
    echo ""
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo_info "Uninstall cancelled"
        exit 0
    fi
}

# Stop and disable service
stop_service() {
    if systemctl --user is-active --quiet onevox.service; then
        echo_info "Stopping service..."
        systemctl --user stop onevox.service || true
    fi
    
    if systemctl --user is-enabled --quiet onevox.service; then
        echo_info "Disabling service..."
        systemctl --user disable onevox.service || true
    fi
}

# Remove files
remove_files() {
    echo_info "Removing files..."
    
    # Remove binary
    if [ -f "$INSTALL_DIR/onevox" ]; then
        rm -f "$INSTALL_DIR/onevox"
        echo_info "Removed binary"
    fi
    
    # Remove service
    if [ -f "$SERVICE_DIR/onevox.service" ]; then
        rm -f "$SERVICE_DIR/onevox.service"
        systemctl --user daemon-reload
        echo_info "Removed service"
    fi
    
    # Remove desktop entry
    if [ -f "$DESKTOP_DIR/onevox.desktop" ]; then
        rm -f "$DESKTOP_DIR/onevox.desktop"
        if command -v update-desktop-database &> /dev/null; then
            update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
        fi
        echo_info "Removed desktop entry"
    fi
    
    # Remove config
    if [ -d "$CONFIG_DIR" ]; then
        rm -rf "$CONFIG_DIR"
        echo_info "Removed config"
    fi
    
    # Remove data
    if [ -d "$DATA_DIR" ]; then
        rm -rf "$DATA_DIR"
        echo_info "Removed data"
    fi
    
    # Remove cache
    if [ -d "$CACHE_DIR" ]; then
        rm -rf "$CACHE_DIR"
        echo_info "Removed cache"
    fi
}

# Main uninstall
main() {
    echo "╔════════════════════════════════════════╗"
    echo "║   Onevox Linux Uninstaller             ║"
    echo "╚════════════════════════════════════════╝"
    echo ""
    
    confirm_uninstall
    stop_service
    remove_files
    
    echo ""
    echo_info "✅ Onevox uninstalled successfully"
    echo ""
}

main "$@"
