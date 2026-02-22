#!/bin/bash
# Migration script for Onevox v0.1.0+
# Migrates data from old paths to new platform-appropriate paths
#
# Old paths: ~/Library/Caches/onevox, ~/Library/Application Support/onevox
# New paths: ~/Library/Caches/com.onevox.onevox, ~/Library/Application Support/com.onevox.onevox

set -e

echo "🔄 Onevox Path Migration Script"
echo "================================"
echo ""

# Detect platform
PLATFORM="$(uname -s)"
case "$PLATFORM" in
    Darwin*)
        echo "✅ Detected: macOS"
        OLD_CACHE="$HOME/Library/Caches/onevox"
        OLD_CONFIG="$HOME/Library/Application Support/onevox"
        NEW_CACHE="$HOME/Library/Caches/com.onevox.onevox"
        NEW_CONFIG="$HOME/Library/Application Support/com.onevox.onevox"
        ;;
    Linux*)
        echo "✅ Detected: Linux"
        OLD_CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/onevox"
        OLD_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/onevox"
        NEW_CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/onevox"
        NEW_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/onevox"
        echo "ℹ️  Linux paths are already correct, no migration needed"
        exit 0
        ;;
    *)
        echo "❌ Unsupported platform: $PLATFORM"
        exit 1
        ;;
esac

echo ""
echo "Old paths:"
echo "  Cache:  $OLD_CACHE"
echo "  Config: $OLD_CONFIG"
echo ""
echo "New paths:"
echo "  Cache:  $NEW_CACHE"
echo "  Config: $NEW_CONFIG"
echo ""

# Check if old paths exist
HAS_OLD_DATA=false
if [ -d "$OLD_CACHE" ] || [ -d "$OLD_CONFIG" ]; then
    HAS_OLD_DATA=true
fi

if [ "$HAS_OLD_DATA" = false ]; then
    echo "ℹ️  No legacy data found. Nothing to migrate."
    exit 0
fi

echo "📦 Found legacy data. Starting migration..."
echo ""

# Create new directories
mkdir -p "$NEW_CACHE"
mkdir -p "$NEW_CONFIG"
chmod 700 "$NEW_CACHE"
chmod 700 "$NEW_CONFIG"

# Migrate cache data
if [ -d "$OLD_CACHE" ]; then
    echo "📂 Migrating cache data..."
    
    # Migrate models
    if [ -d "$OLD_CACHE/models" ]; then
        echo "  → AI models"
        mkdir -p "$NEW_CACHE/models"
        cp -Rp "$OLD_CACHE/models"/* "$NEW_CACHE/models/" 2>/dev/null || true
    fi
    
    # Migrate debug files (if any)
    if [ -d "$OLD_CACHE/debug" ]; then
        echo "  → Debug files"
        cp -Rp "$OLD_CACHE/debug" "$NEW_CACHE/" 2>/dev/null || true
    fi
fi

# Migrate config data
if [ -d "$OLD_CONFIG" ]; then
    echo "📂 Migrating config data..."
    
    # Migrate config.toml
    if [ -f "$OLD_CONFIG/config.toml" ]; then
        echo "  → config.toml"
        cp -p "$OLD_CONFIG/config.toml" "$NEW_CONFIG/config.toml"
    fi
    
    # Migrate history.json
    if [ -f "$OLD_CONFIG/history.json" ]; then
        echo "  → history.json"
        cp -p "$OLD_CONFIG/history.json" "$NEW_CONFIG/history.json"
    fi
fi

echo ""
echo "✅ Migration complete!"
echo ""
echo "📊 Verification:"
echo ""

# Verify files
if [ -d "$NEW_CACHE/models" ]; then
    MODEL_COUNT=$(find "$NEW_CACHE/models" -name "*.bin" 2>/dev/null | wc -l)
    echo "  ✅ $MODEL_COUNT model file(s) migrated"
fi

if [ -d "$NEW_CACHE/models" ]; then
    MODEL_COUNT=$(find "$NEW_CACHE/models" -name "*.bin" | wc -l | tr -d ' ')
    echo "  ✅ $MODEL_COUNT model(s) migrated"
fi

if [ -f "$NEW_CONFIG/config.toml" ]; then
    echo "  ✅ config.toml"
fi

if [ -f "$NEW_CONFIG/history.json" ]; then
    ENTRIES=$(jq '. | length' "$NEW_CONFIG/history.json" 2>/dev/null || echo "?")
    echo "  ✅ history.json ($ENTRIES entries)"
fi

echo ""
echo "🗑️  Legacy directories (not removed automatically):"
echo "  $OLD_CACHE"
echo "  $OLD_CONFIG"
echo ""
echo "💡 To remove legacy directories after verification:"
echo "  rm -rf '$OLD_CACHE'"
echo "  rm -rf '$OLD_CONFIG'"
echo ""
echo "✨ You can now run: onevox daemon"
