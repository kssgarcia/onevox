#!/usr/bin/env bash

set -euo pipefail

echo "🔐 Onevox Permission Setup"
echo "=========================="
echo ""
echo "This script will open System Settings to help you grant permissions."
echo ""

APP_PATH="$HOME/Applications/Onevox.app"
if [[ ! -d "$APP_PATH" ]]; then
  APP_PATH="/Applications/Onevox.app"
fi

if [[ ! -d "$APP_PATH" ]]; then
  echo "❌ Onevox.app not found. Please install Onevox first."
  exit 1
fi

echo "App location: $APP_PATH"
echo ""

# Step 1: Input Monitoring
echo "Step 1: Input Monitoring Permission"
echo "-----------------------------------"
echo "This permission allows Onevox to detect the Cmd+Shift+0 hotkey."
echo ""
echo "Opening System Settings → Privacy & Security → Input Monitoring..."
open "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
echo ""
echo "In the window that opened:"
echo "  1. Find 'Onevox' in the list (or click + to add it)"
echo "  2. Toggle it ON (blue/green)"
echo ""
read -p "Press ENTER when done..."
echo ""

# Step 2: Accessibility
echo "Step 2: Accessibility Permission"
echo "--------------------------------"
echo "This permission allows Onevox to inject transcribed text into apps."
echo ""
echo "Opening System Settings → Privacy & Security → Accessibility..."
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
echo ""
echo "In the window that opened:"
echo "  1. Click the lock icon and enter your password"
echo "  2. Find 'Onevox' in the list (or click + to add it)"
echo "  3. Toggle it ON (blue/green)"
echo ""
read -p "Press ENTER when done..."
echo ""

# Step 3: Microphone
echo "Step 3: Microphone Permission"
echo "-----------------------------"
echo "This permission allows Onevox to capture your voice."
echo ""
echo "Note: macOS will prompt you automatically when you first use the hotkey."
echo "      You can also grant it now in System Settings."
echo ""
read -p "Open Microphone settings now? (y/N): " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
  open "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"
  echo ""
  echo "In the window that opened:"
  echo "  1. Find 'Onevox' in the list"
  echo "  2. Toggle it ON (blue/green)"
  echo ""
  read -p "Press ENTER when done..."
fi
echo ""

# Step 4: Restart daemon
echo "Step 4: Restart Daemon"
echo "---------------------"
echo "Restarting the Onevox daemon to apply permissions..."
launchctl kickstart -k "gui/$(id -u)/com.onevox.daemon"
sleep 2
echo "✅ Daemon restarted"
echo ""

# Step 5: Verify
echo "Step 5: Verify"
echo "-------------"
if onevox status >/dev/null 2>&1; then
  echo "✅ Daemon is running!"
  echo ""
  onevox status
  echo ""
  echo "🎉 Setup complete!"
  echo ""
  echo "Try it now:"
  echo "  1. Open any text editor (TextEdit, Notes, etc.)"
  echo "  2. Press and hold Cmd+Shift+0"
  echo "  3. Speak clearly"
  echo "  4. Release the hotkey"
  echo "  5. Your transcribed text should appear!"
  echo ""
  echo "If it doesn't work, check logs:"
  echo "  tail -f ~/Library/Logs/onevox/stdout.log"
else
  echo "❌ Daemon is not responding"
  echo ""
  echo "Try running the diagnostic:"
  echo "  curl -fsSL https://raw.githubusercontent.com/kssgarcia/onevox/main/scripts/check_permissions.sh | sh"
fi
