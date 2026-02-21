#!/usr/bin/env bash

set -euo pipefail

APP_NAME="Onevox.app"
USER_APP_PATH="$HOME/Applications/$APP_NAME"
SYSTEM_APP_PATH="/Applications/$APP_NAME"
LAUNCHD_LABEL="com.onevox.daemon"

echo "🔍 Onevox Permission Checker"
echo "=============================="
echo ""

# Find app location
APP_PATH=""
if [[ -d "$USER_APP_PATH" ]]; then
  APP_PATH="$USER_APP_PATH"
elif [[ -d "$SYSTEM_APP_PATH" ]]; then
  APP_PATH="$SYSTEM_APP_PATH"
else
  echo "❌ Onevox.app not found"
  echo "   Expected locations:"
  echo "   - $USER_APP_PATH"
  echo "   - $SYSTEM_APP_PATH"
  exit 1
fi

echo "✅ App found: $APP_PATH"
echo ""

# Check if daemon is running
echo "📡 Daemon Status:"
if launchctl print "gui/$(id -u)/$LAUNCHD_LABEL" >/dev/null 2>&1; then
  echo "   ✅ Daemon is loaded"
  
  # Check if it's running
  if launchctl print "gui/$(id -u)/$LAUNCHD_LABEL" | grep -q "state = running"; then
    echo "   ✅ Daemon is running"
  else
    echo "   ⚠️  Daemon is loaded but not running"
    echo "   Try: launchctl kickstart -k gui/\$(id -u)/$LAUNCHD_LABEL"
  fi
else
  echo "   ❌ Daemon is not loaded"
  echo "   Try reinstalling: curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/install.sh | sh"
fi
echo ""

# Check CLI
echo "🔧 CLI Status:"
if command -v onevox >/dev/null 2>&1; then
  CLI_PATH=$(which onevox)
  echo "   ✅ CLI found: $CLI_PATH"
  
  # Check if it's a symlink to the app
  if [[ -L "$CLI_PATH" ]]; then
    TARGET=$(readlink "$CLI_PATH")
    echo "   ✅ Symlink target: $TARGET"
  fi
else
  echo "   ❌ CLI not found in PATH"
  echo "   Try reinstalling or add to PATH"
fi
echo ""

# Check permissions (macOS 13+)
echo "🔐 Permission Status:"
echo ""

# Note: Checking permissions programmatically is complex on macOS
# We'll provide instructions instead

echo "Please manually verify these permissions in System Settings:"
echo ""
echo "1. Input Monitoring:"
echo "   System Settings → Privacy & Security → Input Monitoring"
echo "   Verify: $APP_PATH is listed and enabled"
echo ""
echo "2. Accessibility:"
echo "   System Settings → Privacy & Security → Accessibility"
echo "   Verify: $APP_PATH is listed and enabled"
echo ""
echo "3. Microphone:"
echo "   System Settings → Privacy & Security → Microphone"
echo "   Verify: Onevox is listed and enabled"
echo ""

# Check logs
echo "📋 Recent Logs:"
LOG_DIR="$HOME/Library/Logs/onevox"
if [[ -d "$LOG_DIR" ]]; then
  echo "   Log directory: $LOG_DIR"
  
  if [[ -f "$LOG_DIR/stderr.log" ]]; then
    echo ""
    echo "   Last 10 lines of stderr.log:"
    echo "   ─────────────────────────────"
    tail -n 10 "$LOG_DIR/stderr.log" | sed 's/^/   /'
  fi
else
  echo "   ⚠️  Log directory not found: $LOG_DIR"
fi
echo ""

# Check config
echo "⚙️  Configuration:"
CONFIG_DIR="$HOME/Library/Application Support/com.onevox.onevox"
CONFIG_FILE="$CONFIG_DIR/config.toml"

if [[ -f "$CONFIG_FILE" ]]; then
  echo "   ✅ Config found: $CONFIG_FILE"
  
  # Extract hotkey
  if grep -q "trigger" "$CONFIG_FILE"; then
    HOTKEY=$(grep "trigger" "$CONFIG_FILE" | cut -d'"' -f2)
    echo "   Hotkey: $HOTKEY"
  fi
else
  echo "   ⚠️  Config not found, using defaults"
  echo "   Default hotkey: Cmd+Shift+0"
fi
echo ""

# Recommendations
echo "💡 Troubleshooting Steps:"
echo ""
echo "If hotkey doesn't work:"
echo "  1. Grant Input Monitoring permission (see above)"
echo "  2. Restart daemon: onevox stop && launchctl kickstart -k gui/\$(id -u)/$LAUNCHD_LABEL"
echo "  3. Check logs: tail -f $LOG_DIR/stderr.log"
echo ""
echo "If text doesn't inject:"
echo "  1. Grant Accessibility permission (see above)"
echo "  2. Restart daemon"
echo ""
echo "If no audio captured:"
echo "  1. Grant Microphone permission (see above)"
echo "  2. Test: onevox test-audio --duration 3"
echo "  3. List devices: onevox devices list"
echo ""
echo "For more help, see: https://github.com/kssgarcia/onevox/blob/main/INSTALLATION.md"
