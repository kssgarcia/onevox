#!/usr/bin/env bash
# Launch the ONEVOX TUI

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TUI_DIR="$PROJECT_ROOT/tui"

echo "🖥️  ONEVOX TUI Launcher"
echo ""

# Check if Bun is installed
if ! command -v bun &> /dev/null; then
    echo "❌ Bun is not installed"
    echo ""
    echo "Please install Bun from: https://bun.sh"
    echo ""
    echo "Quick install:"
    echo "  curl -fsSL https://bun.sh/install | bash"
    echo ""
    exit 1
fi

echo "✅ Bun found: $(bun --version)"

# Check if TUI directory exists
if [ ! -d "$TUI_DIR" ]; then
    echo "❌ TUI directory not found: $TUI_DIR"
    exit 1
fi

cd "$TUI_DIR"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    bun install
    echo ""
fi

# Launch TUI
echo "🚀 Launching TUI..."
echo ""
bun run src/index.ts
