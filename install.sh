#!/bin/sh

set -eu

SYSTEM_INSTALL=0
REPO="${ONEVOX_REPO:-kssgarcia/onevox}"
VERSION="${ONEVOX_VERSION:-latest}"
ASSET="${ONEVOX_RELEASE_ASSET:-}"

usage() {
  cat <<EOF
Onevox curl installer (macOS)

Usage:
  sh install.sh [--system] [--repo owner/name] [--version latest|vX.Y.Z] [--asset file.tar.gz]

Options:
  --system         Install app to /Applications (uses sudo when needed)
  --repo           GitHub repo slug (default: $REPO)
  --version        Release tag or "latest" (default: $VERSION)
  --asset          Override release asset filename
  --help           Show this help

Environment overrides:
  ONEVOX_REPO, ONEVOX_VERSION, ONEVOX_RELEASE_ASSET
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --system)
      SYSTEM_INSTALL=1
      ;;
    --repo)
      shift
      REPO="${1:-}"
      ;;
    --version)
      shift
      VERSION="${1:-}"
      ;;
    --asset)
      shift
      ASSET="${1:-}"
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
  shift
done

if [ "$(uname -s)" != "Darwin" ]; then
  echo "This installer currently supports macOS only." >&2
  exit 1
fi

# If run from a local checkout, delegate to the repo installer.
if [ -f "./scripts/install_macos.sh" ] && [ -f "./Cargo.toml" ]; then
  echo "Detected local checkout; delegating to ./scripts/install_macos.sh"
  if [ "$SYSTEM_INSTALL" -eq 1 ]; then
    exec ./scripts/install_macos.sh --system
  fi
  exec ./scripts/install_macos.sh
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required but not installed." >&2
  exit 1
fi

if ! command -v tar >/dev/null 2>&1; then
  echo "tar is required but not installed." >&2
  exit 1
fi

ARCH="$(uname -m)"
if [ -z "$ASSET" ]; then
  case "$ARCH" in
    arm64|aarch64) ASSET="onevox-macos-arm64.tar.gz" ;;
    x86_64|amd64) ASSET="onevox-macos-x86_64.tar.gz" ;;
    *) 
      echo "Unsupported architecture: $ARCH" >&2
      echo "Supported: arm64 (Apple Silicon), x86_64 (Intel)" >&2
      exit 1
      ;;
  esac
fi

if [ "$VERSION" = "latest" ]; then
  URL="https://github.com/$REPO/releases/latest/download/$ASSET"
else
  URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
fi

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

ARCHIVE="$TMP_DIR/$ASSET"
EXTRACT_DIR="$TMP_DIR/extract"
mkdir -p "$EXTRACT_DIR"

echo "Downloading: $URL"
curl -fL "$URL" -o "$ARCHIVE"

echo "Extracting package..."
tar -xzf "$ARCHIVE" -C "$EXTRACT_DIR"

APP_SRC="$(find "$EXTRACT_DIR" -type d -name "Onevox.app" | head -n 1 || true)"
BIN_SRC="$(find "$EXTRACT_DIR" -type f -name "onevox" | head -n 1 || true)"

if [ -z "$APP_SRC" ] && [ -z "$BIN_SRC" ]; then
  echo "Package did not contain Onevox.app or a onevox binary." >&2
  exit 1
fi

if [ "$SYSTEM_INSTALL" -eq 1 ]; then
  APP_INSTALL_DIR="/Applications"
else
  APP_INSTALL_DIR="$HOME/Applications"
fi

APP_PATH="$APP_INSTALL_DIR/Onevox.app"
APP_BIN="$APP_PATH/Contents/MacOS/onevox"
CLI_LINK_CANDIDATES="/usr/local/bin /opt/homebrew/bin"
CLI_LINK_PATH=""
LAUNCHD_LABEL="com.onevox.daemon"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
LAUNCHD_PLIST="$LAUNCH_AGENTS_DIR/$LAUNCHD_LABEL.plist"
LOG_DIR="$HOME/Library/Logs/onevox"

mkdir -p "$APP_INSTALL_DIR"

if [ ! -w "$APP_INSTALL_DIR" ]; then
  SUDO="sudo"
else
  SUDO=""
fi

echo "Installing app bundle to $APP_PATH"
$SUDO rm -rf "$APP_PATH"

if [ -n "$APP_SRC" ]; then
  $SUDO cp -R "$APP_SRC" "$APP_PATH"
else
  $SUDO mkdir -p "$APP_PATH/Contents/MacOS" "$APP_PATH/Contents/Resources"
  $SUDO cp "$BIN_SRC" "$APP_BIN"
  $SUDO chmod +x "$APP_BIN"
  $SUDO sh -c "cat > '$APP_PATH/Contents/Info.plist'" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
 "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>Onevox</string>
  <key>CFBundleDisplayName</key>
  <string>Onevox</string>
  <key>CFBundleIdentifier</key>
  <string>com.onevox.app</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleExecutable</key>
  <string>onevox</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>LSMinimumSystemVersion</key>
  <string>13.0</string>
  <key>LSUIElement</key>
  <true/>
</dict>
</plist>
EOF
fi

mkdir -p "$LAUNCH_AGENTS_DIR" "$LOG_DIR"
cat > "$LAUNCHD_PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
 "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>$LAUNCHD_LABEL</string>
  <key>ProgramArguments</key>
  <array>
    <string>$APP_BIN</string>
    <string>daemon</string>
    <string>--foreground</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>PATH</key>
    <string>/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
  </dict>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>$LOG_DIR/stdout.log</string>
  <key>StandardErrorPath</key>
  <string>$LOG_DIR/stderr.log</string>
  <key>ProcessType</key>
  <string>Interactive</string>
  <key>LimitLoadToSessionType</key>
  <array>
    <string>Aqua</string>
  </array>
</dict>
</plist>
EOF

echo "Reloading launchd agent..."
launchctl bootout "gui/$(id -u)" "$LAUNCHD_PLIST" >/dev/null 2>&1 || true
launchctl bootstrap "gui/$(id -u)" "$LAUNCHD_PLIST"
launchctl enable "gui/$(id -u)/$LAUNCHD_LABEL"
launchctl kickstart -k "gui/$(id -u)/$LAUNCHD_LABEL"

for candidate in $CLI_LINK_CANDIDATES; do
  if [ -d "$candidate" ]; then
    CLI_LINK_PATH="$candidate/onevox"
    break
  fi
done

if [ -z "$CLI_LINK_PATH" ]; then
  mkdir -p "$HOME/.local/bin"
  CLI_LINK_PATH="$HOME/.local/bin/onevox"
fi

echo "Creating CLI symlink: $CLI_LINK_PATH -> $APP_BIN"
if [ -w "$(dirname "$CLI_LINK_PATH")" ]; then
  ln -sf "$APP_BIN" "$CLI_LINK_PATH"
else
  sudo ln -sf "$APP_BIN" "$CLI_LINK_PATH"
fi

echo ""
echo "Install complete."
echo "App: $APP_PATH"
echo "CLI: $CLI_LINK_PATH"
echo "Agent: $LAUNCHD_PLIST"
echo "Logs: $LOG_DIR"
echo ""
echo "⚠️  IMPORTANT: Grant Permissions IN ORDER"
echo "For Onevox to work, you need to grant these permissions:"
echo ""
echo "1. Input Monitoring (for hotkey detection):"
echo "   open 'x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent'"
echo "   Add Onevox.app and toggle ON"
echo ""
echo "2. Accessibility (for text injection):"
echo "   open 'x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility'"
echo "   Add Onevox.app and toggle ON"
echo ""
echo "3. Restart daemon (REQUIRED after granting permissions):"
echo "   launchctl kickstart -k gui/\$(id -u)/$LAUNCHD_LABEL"
echo ""
echo "4. Microphone (for audio capture):"
echo "   Will prompt automatically when you first press Cmd+Shift+0"
echo ""
echo "💡 Quick setup:"
echo "  open 'x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent'"
echo "  open 'x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility'"
echo "  launchctl kickstart -k gui/\$(id -u)/$LAUNCHD_LABEL"
echo ""
echo "💡 To check permissions and diagnose issues:"
echo "  curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/check_permissions.sh | sh"

if ! command -v onevox >/dev/null 2>&1; then
  if [ "$CLI_LINK_PATH" = "$HOME/.local/bin/onevox" ]; then
    echo ""
    echo "Note: add ~/.local/bin to PATH to use 'onevox' directly."
    echo "  export PATH=\"$HOME/.local/bin:\$PATH\""
  fi
fi
