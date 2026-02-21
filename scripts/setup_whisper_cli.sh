#!/usr/bin/env bash
# Setup script for whisper-cli binary
# This script downloads and builds whisper.cpp, then installs the CLI binary

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Onevox Whisper CLI Setup ===${NC}"
echo ""

# Determine cache directory based on platform
if [[ "$OSTYPE" == "darwin"* ]]; then
    CACHE_DIR="$HOME/Library/Caches/com.onevox.onevox"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/onevox"
else
    echo -e "${RED}Unsupported platform: $OSTYPE${NC}"
    exit 1
fi

BIN_DIR="$CACHE_DIR/bin"
WHISPER_CLI_PATH="$BIN_DIR/whisper-cli"

# Check if whisper-cli already exists
if [ -f "$WHISPER_CLI_PATH" ]; then
    echo -e "${YELLOW}whisper-cli already exists at: $WHISPER_CLI_PATH${NC}"
    read -p "Do you want to reinstall? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Setup cancelled."
        exit 0
    fi
fi

# Create bin directory
echo "Creating bin directory: $BIN_DIR"
mkdir -p "$BIN_DIR"

# Clone whisper.cpp to /tmp
echo ""
echo -e "${GREEN}Cloning whisper.cpp repository...${NC}"
if [ -d "/tmp/whisper.cpp" ]; then
    echo "Removing existing /tmp/whisper.cpp..."
    rm -rf /tmp/whisper.cpp
fi

git clone https://github.com/ggerganov/whisper.cpp /tmp/whisper.cpp
cd /tmp/whisper.cpp

# Build whisper.cpp with CMake
echo ""
echo -e "${GREEN}Building whisper.cpp with CMake...${NC}"
echo "This may take a few minutes..."

if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Detected macOS - enabling Metal acceleration"
    cmake -B build \
        -DCMAKE_C_COMPILER=$(xcrun -find clang) \
        -DCMAKE_CXX_COMPILER=$(xcrun -find clang++) \
        -DGGML_METAL=ON
    
    cmake --build build --config Release -j$(sysctl -n hw.logicalcpu)
else
    cmake -B build
    cmake --build build --config Release -j$(nproc)
fi

# Check if build was successful
if [ ! -f "./build/bin/whisper-cli" ]; then
    echo -e "${RED}Build failed: whisper-cli binary not found${NC}"
    exit 1
fi

# Copy binary to cache directory
echo ""
echo -e "${GREEN}Installing whisper-cli...${NC}"
cp ./build/bin/whisper-cli "$WHISPER_CLI_PATH"
chmod +x "$WHISPER_CLI_PATH"

# Verify installation
if [ -f "$WHISPER_CLI_PATH" ]; then
    echo ""
    echo -e "${GREEN}✅ Setup complete!${NC}"
    echo ""
    echo "whisper-cli installed at: $WHISPER_CLI_PATH"
    echo ""
    echo "Next steps:"
    echo "1. Download a model: onevox models download ggml-base.en"
    echo "2. Start the daemon: onevox daemon"
    echo ""
else
    echo -e "${RED}Installation failed${NC}"
    exit 1
fi
