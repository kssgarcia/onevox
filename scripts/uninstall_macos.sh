#!/usr/bin/env bash

set -euo pipefail

APP_NAME="Onevox.app"
LAUNCHD_LABEL="com.onevox.daemon"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
LAUNCHD_PLIST="$LAUNCH_AGENTS_DIR/$LAUNCHD_LABEL.plist"
USER_APP_PATH="$HOME/Applications/$APP_NAME"
SYSTEM_APP_PATH="/Applications/$APP_NAME"
CLI_LINKS=(
  "/usr/local/bin/onevox"
  "/opt/homebrew/bin/onevox"
  "$HOME/.local/bin/onevox"
)

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This uninstaller only supports macOS."
  exit 1
fi

echo "Stopping launchd agent..."
launchctl bootout "gui/$(id -u)" "$LAUNCHD_PLIST" >/dev/null 2>&1 || true

if [[ -f "$LAUNCHD_PLIST" ]]; then
  rm -f "$LAUNCHD_PLIST"
  echo "Removed: $LAUNCHD_PLIST"
fi

if [[ -d "$USER_APP_PATH" ]]; then
  rm -rf "$USER_APP_PATH"
  echo "Removed: $USER_APP_PATH"
fi

if [[ -d "$SYSTEM_APP_PATH" ]]; then
  if [[ -w "/Applications" ]]; then
    rm -rf "$SYSTEM_APP_PATH"
  else
    sudo rm -rf "$SYSTEM_APP_PATH"
  fi
  echo "Removed: $SYSTEM_APP_PATH"
fi

for link in "${CLI_LINKS[@]}"; do
  if [[ -L "$link" || -f "$link" ]]; then
    if [[ -w "$(dirname "$link")" ]]; then
      rm -f "$link"
    else
      sudo rm -f "$link"
    fi
    echo "Removed: $link"
  fi
done

echo "Uninstall complete."
