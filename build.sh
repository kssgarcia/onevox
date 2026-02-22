#!/usr/bin/env bash
# OneVox Build Script
# Handles platform-specific build configuration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

echo -e "${GREEN}OneVox Build Script${NC}"
echo "Platform: $OS $ARCH"
echo ""

# Parse arguments
BUILD_TYPE="${1:-debug}"
FEATURES="${2:-}"

if [ "$BUILD_TYPE" = "release" ]; then
    BUILD_FLAG="--release"
    echo "Build type: Release (optimized)"
else
    BUILD_FLAG=""
    echo "Build type: Debug"
fi

if [ -n "$FEATURES" ]; then
    FEATURES_FLAG="--features $FEATURES"
    echo "Additional features: $FEATURES"
else
    FEATURES_FLAG=""
    echo "Features: default (whisper-cpp, overlay-indicator)"
fi
echo ""

# Platform-specific configuration
case "$OS" in
    Darwin)
        echo -e "${YELLOW}Configuring for macOS...${NC}"
        
        # Check for Xcode Command Line Tools
        if ! command -v xcrun &> /dev/null; then
            echo -e "${RED}Error: Xcode Command Line Tools not found${NC}"
            echo "Install with: xcode-select --install"
            exit 1
        fi
        
        # Set macOS-specific environment
        export CC=clang
        export CXX=clang++
        export SDKROOT=$(xcrun --show-sdk-path)
        export MACOSX_DEPLOYMENT_TARGET=13.0
        
        echo "CC: $CC"
        echo "CXX: $CXX"
        echo "SDKROOT: $SDKROOT"
        echo "MACOSX_DEPLOYMENT_TARGET: $MACOSX_DEPLOYMENT_TARGET"
        ;;
        
    Linux)
        echo -e "${YELLOW}Configuring for Linux...${NC}"
        
        # Check for build essentials
        if ! command -v gcc &> /dev/null; then
            echo -e "${RED}Error: GCC not found${NC}"
            echo "Install with: sudo apt-get install build-essential cmake"
            exit 1
        fi
        
        echo "Using system compiler"
        ;;
        
    MINGW*|MSYS*|CYGWIN*)
        echo -e "${YELLOW}Configuring for Windows...${NC}"
        echo "Using MSVC toolchain"
        ;;
        
    *)
        echo -e "${RED}Unsupported platform: $OS${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}Starting build...${NC}"
echo ""

# Build command
cargo build $BUILD_FLAG $FEATURES_FLAG

BUILD_STATUS=$?

echo ""
if [ $BUILD_STATUS -eq 0 ]; then
    echo -e "${GREEN}✅ Build successful!${NC}"
    
    if [ "$BUILD_TYPE" = "release" ]; then
        BINARY_PATH="target/release/onevox"
    else
        BINARY_PATH="target/debug/onevox"
    fi
    
    if [ -f "$BINARY_PATH" ]; then
        BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
        echo "Binary: $BINARY_PATH ($BINARY_SIZE)"
        echo ""
        echo "Run with: ./$BINARY_PATH --help"
    fi
else
    echo -e "${RED}❌ Build failed${NC}"
    exit $BUILD_STATUS
fi
