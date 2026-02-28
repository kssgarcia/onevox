#!/usr/bin/env bash
# OneVox Build Script
# Handles platform-specific build configuration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

patch_linux_interpreter() {
    local binary_path="$1"

    # Only relevant on Linux for ELF binaries.
    if [ "$(uname -s)" != "Linux" ] || [ ! -f "$binary_path" ]; then
        return
    fi

    if ! command -v readelf >/dev/null 2>&1 || ! command -v patchelf >/dev/null 2>&1; then
        return
    fi

    local current_interp=""
    current_interp="$(readelf -l "$binary_path" 2>/dev/null | sed -n 's/.*Requesting program interpreter: \(.*\)]/\1/p' | head -n1)"

    # Nix-linked binaries can fail to resolve transitive system libs outside nix env.
    if [[ "$current_interp" == /nix/store/* ]]; then
        local system_interp=""
        if [ -x /lib64/ld-linux-x86-64.so.2 ]; then
            system_interp="/lib64/ld-linux-x86-64.so.2"
        elif [ -x /usr/lib64/ld-linux-x86-64.so.2 ]; then
            system_interp="/usr/lib64/ld-linux-x86-64.so.2"
        fi

        if [ -n "$system_interp" ]; then
            echo -e "${YELLOW}Patching Linux ELF interpreter:${NC} $current_interp -> $system_interp"
            patchelf --set-interpreter "$system_interp" "$binary_path"
        else
            echo -e "${YELLOW}Warning:${NC} binary uses Nix interpreter ($current_interp), but no system ld-linux was found to patch."
        fi
    fi
}

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
        patch_linux_interpreter "$BINARY_PATH"

        BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
        echo "Binary: $BINARY_PATH ($BINARY_SIZE)"
        echo ""
        echo "Run with: ./$BINARY_PATH --help"
    fi
else
    echo -e "${RED}❌ Build failed${NC}"
    exit $BUILD_STATUS
fi
