#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="Onevox.app"
LAUNCHD_LABEL="com.onevox.daemon"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
LAUNCHD_PLIST="$LAUNCH_AGENTS_DIR/$LAUNCHD_LABEL.plist"
LOG_DIR="$HOME/Library/Logs/onevox"
APP_INSTALL_DIR="$HOME/Applications"
CLI_LINK_CANDIDATES=(/usr/local/bin /opt/homebrew/bin)
CLI_LINK_PATH=""

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This installer only supports macOS."
  exit 1
fi

if [[ "${1:-}" == "--system" ]]; then
  APP_INSTALL_DIR="/Applications"
fi

APP_PATH="$APP_INSTALL_DIR/$APP_NAME"
APP_BIN="$APP_PATH/Contents/MacOS/onevox"

echo "Packaging Onevox .app..."
"$ROOT_DIR/scripts/package_macos_app.sh" "$ROOT_DIR/dist"

mkdir -p "$APP_INSTALL_DIR"

echo "Installing app bundle to: $APP_PATH"
if [[ -w "$APP_INSTALL_DIR" ]]; then
  rm -rf "$APP_PATH"
  cp -R "$ROOT_DIR/dist/$APP_NAME" "$APP_PATH"
else
  sudo rm -rf "$APP_PATH"
  sudo cp -R "$ROOT_DIR/dist/$APP_NAME" "$APP_PATH"
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

for candidate in "${CLI_LINK_CANDIDATES[@]}"; do
  if [[ -d "$candidate" ]]; then
    CLI_LINK_PATH="$candidate/onevox"
    break
  fi
done

if [[ -z "$CLI_LINK_PATH" ]]; then
  mkdir -p "$HOME/.local/bin"
  CLI_LINK_PATH="$HOME/.local/bin/onevox"
fi

echo "Creating CLI symlink: $CLI_LINK_PATH -> $APP_BIN"
if [[ -w "$(dirname "$CLI_LINK_PATH")" ]]; then
  ln -sf "$APP_BIN" "$CLI_LINK_PATH"
else
  sudo ln -sf "$APP_BIN" "$CLI_LINK_PATH"
fi

echo ""
echo "Installation complete."
echo "App: $APP_PATH"
echo "CLI: $CLI_LINK_PATH"
echo "Agent: $LAUNCHD_PLIST"
echo "Logs: $LOG_DIR"
echo ""
echo "⚠️  IMPORTANT: Grant Permissions IN ORDER"
echo "For Onevox to work, you need to grant these permissions:"
echo ""
echo "1. Microphone (CRITICAL - for audio capture):"
echo "   open 'x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone'"
echo "   Add Onevox.app and toggle ON"
echo ""
echo "2. Input Monitoring (for hotkey detection):"
echo "   open 'x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent'"
echo "   Add Onevox.app and toggle ON"
echo ""
echo "3. Accessibility (for text injection):"
echo "   open 'x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility'"
echo "   Add Onevox.app and toggle ON"
echo ""
echo "4. Restart daemon (REQUIRED after granting permissions):"
echo "   launchctl kickstart -k gui/$(id -u)/$LAUNCHD_LABEL"
echo ""
echo "💡 Quick setup:"
echo "  open 'x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone'"
echo "  open 'x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent'"
echo "  open 'x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility'"
echo "  launchctl kickstart -k gui/$(id -u)/$LAUNCHD_LABEL"
echo ""
echo "💡 To check permissions and diagnose issues:"
echo "  curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/check_permissions.sh | sh"

if ! command -v onevox >/dev/null 2>&1 && [[ "$CLI_LINK_PATH" == "$HOME/.local/bin/onevox" ]]; then
  echo ""
  echo "Note: add ~/.local/bin to PATH to use 'onevox' directly."
  echo "  export PATH=\"$HOME/.local/bin:\$PATH\""
fi
