#!/bin/bash

# Install espup using pre-built binary (workaround for compilation issues)
# This avoids the dependency conflicts when building from source

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_step() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✅${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠️${NC} $1"
}

print_error() {
    echo -e "${RED}❌${NC} $1"
}

echo -e "${BLUE}📦 Installing espup from pre-built binary${NC}"
echo "This avoids compilation issues with dependency conflicts"
echo ""

# Detect architecture
ARCH=$(uname -m)
OS=$(uname -s)

if [[ "$OS" == "Darwin" ]]; then
    if [[ "$ARCH" == "arm64" ]]; then
        BINARY_NAME="espup-aarch64-apple-darwin"
        print_step "Detected: macOS Apple Silicon (ARM64)"
    else
        BINARY_NAME="espup-x86_64-apple-darwin"
        print_step "Detected: macOS Intel (x86_64)"
    fi
elif [[ "$OS" == "Linux" ]]; then
    BINARY_NAME="espup-x86_64-unknown-linux-gnu"
    print_step "Detected: Linux x86_64"
else
    print_error "Unsupported platform: $OS $ARCH"
    exit 1
fi

# Create cargo bin directory if it doesn't exist
CARGO_BIN_DIR="$HOME/.cargo/bin"
mkdir -p "$CARGO_BIN_DIR"

# Download the latest release
print_step "Downloading espup binary..."
DOWNLOAD_URL="https://github.com/esp-rs/espup/releases/latest/download/$BINARY_NAME"

if command -v curl &> /dev/null; then
    if curl -L "$DOWNLOAD_URL" -o "$CARGO_BIN_DIR/espup"; then
        print_success "Downloaded espup binary"
    else
        print_error "Failed to download espup binary"
        exit 1
    fi
elif command -v wget &> /dev/null; then
    if wget "$DOWNLOAD_URL" -O "$CARGO_BIN_DIR/espup"; then
        print_success "Downloaded espup binary"
    else
        print_error "Failed to download espup binary"
        exit 1
    fi
else
    print_error "Neither curl nor wget found. Please install one of them."
    exit 1
fi

# Make it executable
chmod +x "$CARGO_BIN_DIR/espup"

# Verify installation
if command -v espup &> /dev/null; then
    ESPUP_VERSION=$(espup --version 2>/dev/null || echo "unknown")
    print_success "espup installed successfully: $ESPUP_VERSION"
else
    print_error "espup installation failed - binary not found in PATH"
    echo "Make sure $CARGO_BIN_DIR is in your PATH"
    exit 1
fi

echo ""
print_success "espup binary installation completed! 🎉"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "1. Install ESP-IDF: espup install"
echo "2. Source environment: source \$HOME/export-esp.sh"
echo "3. Test ESP32 build: make test-esp32"