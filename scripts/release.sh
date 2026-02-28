#!/usr/bin/env bash
# OneVox Release Script
# Creates platform-specific release artifacts

set -e

VERSION=${1:-$(git describe --tags --always)}
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Building OneVox $VERSION for $PLATFORM-$ARCH${NC}"
echo ""

# Create dist directory
mkdir -p dist

case "$PLATFORM" in
    darwin)
        echo "🍎 Building for macOS..."
        echo ""
        
        # Build with proper environment
        CC=clang CXX=clang++ SDKROOT=$(xcrun --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=13.0 \
          cargo build --release --locked
        
        echo ""
        echo "📦 Creating app bundle..."
        ./scripts/package_macos_app.sh
        
        echo ""
        echo "💿 Creating DMG..."
        DMG_NAME="Onevox-${VERSION}-macos-${ARCH}.dmg"
        hdiutil create -volname "OneVox" -srcfolder dist/Onevox.app \
          -ov -format UDZO "dist/${DMG_NAME}"
        
        echo ""
        echo -e "${GREEN}✅ Created: dist/${DMG_NAME}${NC}"
        
        # Also create tarball
        echo ""
        echo "📦 Creating tarball..."
        cd dist
        tar -czf "onevox-macos-${ARCH}.tar.gz" Onevox.app
        cd ..
        
        echo -e "${GREEN}✅ Created: dist/onevox-macos-${ARCH}.tar.gz${NC}"
        ;;
        
    linux)
        echo "🐧 Building for Linux..."
        echo ""
        
        cargo build --release --locked
        
        echo ""
        echo "🔧 Stripping binary..."
        strip target/release/onevox
        
        echo ""
        echo "📦 Creating tarball..."
        RELEASE_DIR="onevox-linux-${ARCH}"
        mkdir -p "dist/${RELEASE_DIR}"
        cp target/release/onevox "dist/${RELEASE_DIR}/"
        cp scripts/install_linux.sh "dist/${RELEASE_DIR}/"
        cp scripts/uninstall_linux.sh "dist/${RELEASE_DIR}/"
        cp README.md "dist/${RELEASE_DIR}/"
        cp config.example.toml "dist/${RELEASE_DIR}/"
        
        echo ""
        echo "📦 Bundling TUI resources..."
        if [ -d "tui" ]; then
            cp -r tui "dist/${RELEASE_DIR}/"
            # Remove node_modules if it exists
            rm -rf "dist/${RELEASE_DIR}/tui/node_modules"
            # Remove .DS_Store files if they exist
            find "dist/${RELEASE_DIR}/tui" -name ".DS_Store" -delete 2>/dev/null || true
            echo "✅ TUI resources bundled"
        else
            echo "⚠️  Warning: tui directory not found, skipping"
        fi
        
        cd dist
        tar -czf "${RELEASE_DIR}.tar.gz" "${RELEASE_DIR}"
        cd ..
        
        echo ""
        echo -e "${GREEN}✅ Created: dist/${RELEASE_DIR}.tar.gz${NC}"
        
        # Check if we can create deb package
        if command -v cargo-deb &> /dev/null; then
            echo ""
            echo "📦 Creating Debian package..."
            cargo deb
            cp target/debian/onevox_*.deb dist/ 2>/dev/null || true
            echo -e "${GREEN}✅ Created Debian package in dist/${NC}"
        else
            echo ""
            echo -e "${YELLOW}ℹ️  Install cargo-deb to create .deb packages: cargo install cargo-deb${NC}"
        fi
        ;;
        
    mingw*|msys*|cygwin*)
        echo "🪟 Building for Windows..."
        echo ""
        
        cargo build --release --locked
        
        echo ""
        echo "📦 Creating ZIP archive..."
        RELEASE_DIR="onevox-${VERSION}-windows-x64"
        mkdir -p "dist/${RELEASE_DIR}"
        cp target/release/onevox.exe "dist/${RELEASE_DIR}/"
        cp README.md "dist/${RELEASE_DIR}/"
        cp config.example.toml "dist/${RELEASE_DIR}/"
        if [ -d "tui" ]; then
            cp -r tui "dist/${RELEASE_DIR}/"
            rm -rf "dist/${RELEASE_DIR}/tui/node_modules"
            find "dist/${RELEASE_DIR}/tui" -name ".DS_Store" -delete 2>/dev/null || true
        fi
        
        cd dist
        if command -v zip &> /dev/null; then
            zip -r "${RELEASE_DIR}.zip" "${RELEASE_DIR}"
        else
            # Fallback to PowerShell on Windows
            powershell -Command "Compress-Archive -Path ${RELEASE_DIR}/* -DestinationPath ${RELEASE_DIR}.zip"
        fi
        cd ..
        
        echo ""
        echo -e "${GREEN}✅ Created: dist/${RELEASE_DIR}.zip${NC}"
        ;;
        
    *)
        echo "❌ Unsupported platform: $PLATFORM"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}📦 Release artifacts:${NC}"
ls -lh dist/ | grep -E "${VERSION}|Onevox"

echo ""
echo -e "${GREEN}🎉 Release build complete!${NC}"
echo ""
echo "Next steps:"
echo "  1. Test the release artifact"
echo "  2. Create GitHub release: gh release create ${VERSION} dist/* --title 'OneVox ${VERSION}'"
echo "  3. Or upload manually to: https://github.com/kssgarcia/onevox/releases/new"
