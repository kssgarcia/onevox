#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="Onevox.app"
DEFAULT_OUTPUT_DIR="$ROOT_DIR/dist"
OUTPUT_DIR="${OUTPUT_DIR:-$DEFAULT_OUTPUT_DIR}"
APP_DIR="$OUTPUT_DIR/$APP_NAME"
APP_BIN="$APP_DIR/Contents/MacOS/onevox"
APP_PLIST="$APP_DIR/Contents/Info.plist"

if [[ "${1:-}" == "--help" ]]; then
  cat <<EOF
Usage: $0 [output_dir]

Builds a macOS .app bundle at:
  <output_dir>/Onevox.app

Defaults:
  output_dir = $DEFAULT_OUTPUT_DIR
EOF
  exit 0
fi

if [[ $# -ge 1 ]]; then
  OUTPUT_DIR="$1"
  APP_DIR="$OUTPUT_DIR/$APP_NAME"
  APP_BIN="$APP_DIR/Contents/MacOS/onevox"
  APP_PLIST="$APP_DIR/Contents/Info.plist"
fi

echo "Building release binary..."
cd "$ROOT_DIR"
cargo build --release --locked

echo "Creating app bundle at: $APP_DIR"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"

cp "$ROOT_DIR/target/release/onevox" "$APP_BIN"
chmod +x "$APP_BIN"
cp "$ROOT_DIR/packaging/macos/Info.plist" "$APP_PLIST"

echo "Bundling TUI resources..."
mkdir -p "$APP_DIR/Contents/Resources/tui"
rsync -a --delete \
  --exclude "node_modules" \
  --exclude ".DS_Store" \
  "$ROOT_DIR/tui/" "$APP_DIR/Contents/Resources/tui/"

echo "App bundle created: $APP_DIR"
