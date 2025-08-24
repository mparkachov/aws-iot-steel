#!/bin/bash

# ESP32 Development Setup for macOS
# Installs espup and ESP-IDF toolchain for ESP32 development

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

echo -e "${BLUE}🔧 ESP32 Development Setup for macOS${NC}"
echo "Installing ESP-IDF toolchain for ESP32-C3 development"
echo ""

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_error "This script is designed for macOS. For other platforms, see:"
    echo "https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/"
    exit 1
fi

# Check Rust installation
print_step "Checking Rust installation..."
if ! command -v rustc &> /dev/null; then
    print_error "Rust not found. Please install from https://rustup.rs/"
    exit 1
fi
print_success "Rust found: $(rustc --version)"

# Check if espup is already installed
print_step "Checking espup installation..."
if command -v espup &> /dev/null; then
    print_success "espup already installed: $(espup --version)"
else
    print_step "Installing espup..."
    print_warning "Note: espup may have dependency conflicts with current Rust version"
    
    # Try installing espup with different strategies
    if cargo install espup; then
        print_success "espup installed successfully"
    elif cargo install --git https://github.com/esp-rs/espup.git espup; then
        print_success "espup installed from git (latest version)"
    elif cargo install espup --version 0.14.0; then
        print_success "espup installed (older stable version)"
    else
        print_warning "Cargo install failed, trying pre-built binary..."
        if ./scripts/install-espup-binary.sh; then
            print_success "espup installed from pre-built binary"
        else
            print_error "Failed to install espup with all methods"
            echo ""
            echo "Manual installation options:"
            echo "1. Use ESP-IDF installer directly:"
            echo "   https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/macos-setup.html"
            echo ""
            echo "2. Skip ESP32 development for now:"
            echo "   make pre-push --skip-esp32"
            echo ""
            echo "3. Try manual binary download:"
            echo "   ./scripts/install-espup-binary.sh"
            exit 1
        fi
    fi
fi

# Check if ESP-IDF is already installed
if [ -f "$HOME/export-esp.sh" ]; then
    print_success "ESP-IDF environment already installed"
    
    # Source the environment to check version
    source "$HOME/export-esp.sh"
    if command -v idf.py &> /dev/null; then
        IDF_VERSION=$(idf.py --version 2>/dev/null | head -1 || echo "Unknown version")
        print_success "ESP-IDF version: $IDF_VERSION"
    fi
else
    print_step "Installing ESP-IDF toolchain..."
    print_warning "This may take several minutes and download ~1GB of tools..."
    
    if espup install; then
        print_success "ESP-IDF toolchain installed successfully"
    else
        print_error "Failed to install ESP-IDF toolchain"
        exit 1
    fi
fi

# Check if ldproxy is installed (needed for linking)
print_step "Checking ldproxy..."
if command -v ldproxy &> /dev/null; then
    print_success "ldproxy already installed"
else
    print_step "Installing ldproxy..."
    if cargo install ldproxy; then
        print_success "ldproxy installed successfully"
    else
        print_error "Failed to install ldproxy"
        exit 1
    fi
fi

# Source the ESP-IDF environment
print_step "Setting up ESP-IDF environment..."
if [ -f "$HOME/export-esp.sh" ]; then
    source "$HOME/export-esp.sh"
    print_success "ESP-IDF environment loaded"
else
    print_error "ESP-IDF environment script not found"
    exit 1
fi

# Test ESP32 target availability
print_step "Testing ESP32 target..."
if rustup target list --installed | grep -q "riscv32imc-esp-espidf"; then
    print_success "ESP32 target already available"
elif rustup target add riscv32imc-esp-espidf; then
    print_success "ESP32 target installed"
else
    print_warning "ESP32 target installation failed, but may be available through ESP-IDF"
fi

# Test compilation
print_step "Testing ESP32 compilation..."
if [ -d "aws-iot-platform-esp32" ]; then
    if cargo check --package aws-iot-platform-esp32 --target riscv32imc-esp-espidf; then
        print_success "ESP32 compilation test passed"
    else
        print_warning "ESP32 compilation test failed - check dependencies"
    fi
else
    print_warning "ESP32 package not found - skipping compilation test"
fi

echo ""
print_success "ESP32 development setup completed! 🎉"
echo ""
print_warning "Note: ESP-IDF target (riscv32imc-esp-espidf) requires full ESP-IDF installation"
print_warning "For now, ESP32 development uses stub implementations on macOS"
print_warning "See ESP32_TOOLCHAIN_STATUS.md for detailed status and options"
echo ""
echo -e "${BLUE}Usage:${NC}"
echo "1. For ESP-IDF development, in each new terminal session, run:"
echo "   source \$HOME/export-esp.sh"
echo ""
echo "2. Build ESP32 code for host (development):"
echo "   cargo build --package aws-iot-platform-esp32"
echo ""
echo "3. Build ESP32 firmware (requires ESP-IDF):"
echo "   cargo build --target riscv32imc-esp-espidf --package aws-iot-platform-esp32"
echo ""
echo "4. Run ESP32 tests:"
echo "   make test-esp32"
echo ""
echo -e "${BLUE}Shell Integration (optional):${NC}"
echo "Add this to your ~/.zshrc or ~/.bash_profile to auto-load ESP-IDF:"
echo "   # ESP-IDF Environment"
echo "   [ -f \$HOME/export-esp.sh ] && source \$HOME/export-esp.sh"
echo ""
echo -e "${BLUE}Troubleshooting:${NC}"
echo "• If compilation fails, try: cargo clean && cargo build"
echo "• For ESP-IDF updates: espup update"
echo "• For help: espup --help"
echo "• ESP-IDF docs: https://docs.espressif.com/projects/esp-idf/"