#!/usr/bin/env bash

set -euo pipefail

APP_PATH="$HOME/Applications/Onevox.app"
if [[ ! -d "$APP_PATH" ]]; then
  APP_PATH="/Applications/Onevox.app"
fi

if [[ ! -d "$APP_PATH" ]]; then
  echo "❌ Onevox.app not found"
  exit 1
fi

echo "🔐 Onevox Permission Setup Helper"
echo "=================================="
echo ""
echo "This script will help you grant the required permissions to Onevox."
echo ""
echo "App location: $APP_PATH"
echo ""

# Step 1: Reset permissions
echo "Step 1: Resetting permissions..."
tccutil reset Accessibility com.onevox.app 2>/dev/null || true
tccutil reset ListenEvent com.onevox.app 2>/dev/null || true
echo "✅ Permissions reset"
echo ""

# Step 2: Stop launchd daemon
echo "Step 2: Stopping launchd daemon..."
launchctl bootout "gui/$(id -u)" "$HOME/Library/LaunchAgents/com.onevox.daemon.plist" 2>/dev/null || true
sleep 1
echo "✅ Daemon stopped"
echo ""

# Step 3: Run daemon in foreground
echo "Step 3: Starting daemon in foreground..."
echo ""
echo "⚠️  IMPORTANT: macOS will now prompt you for permissions!"
echo ""
echo "When you see the prompts:"
echo "  1. Click 'Open System Settings' or 'OK'"
echo "  2. In System Settings, find Onevox.app in the list"
echo "  3. Toggle it ON"
echo "  4. Come back to this terminal"
echo ""
echo "The daemon will run for 30 seconds to trigger permission prompts..."
echo "Press Cmd+Shift+0 to test the hotkey!"
echo ""
read -p "Press ENTER to continue..."
echo ""

# Run daemon in foreground for 30 seconds
timeout 30 "$APP_PATH/Contents/MacOS/onevox" daemon --foreground 2>&1 &
DAEMON_PID=$!

echo "✅ Daemon running (PID: $DAEMON_PID)"
echo ""
echo "👉 Now:"
echo "  1. Grant permissions when prompted"
echo "  2. Press Cmd+Shift+0 to test the hotkey"
echo "  3. Wait for this script to finish..."
echo ""

# Wait for daemon or timeout
wait $DAEMON_PID 2>/dev/null || true

echo ""
echo "Step 4: Starting launchd daemon..."
launchctl bootstrap "gui/$(id -u)" "$HOME/Library/LaunchAgents/com.onevox.daemon.plist" 2>/dev/null || true
launchctl kickstart -k "gui/$(id -u)/com.onevox.daemon"
sleep 2
echo "✅ Daemon started via launchd"
echo ""

# Step 5: Verify
echo "Step 5: Verifying..."
echo ""

if onevox status >/dev/null 2>&1; then
  echo "✅ Daemon is running!"
  onevox status
  echo ""
  echo "🎉 Setup complete!"
  echo ""
  echo "Try pressing Cmd+Shift+0 in any text editor to test dictation."
else
  echo "❌ Daemon is not responding"
  echo ""
  echo "Check logs: tail -f ~/Library/Logs/onevox/stdout.log"
fi
