#!/bin/bash

# ESP32 Cross-compilation Test Script
# Mirrors the cross-compile-esp32 job from GitHub Actions
# Tests ESP32-C3 compilation without requiring full ESP-IDF setup

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

echo -e "${BLUE}🔧 ESP32-C3 Cross-compilation Test${NC}"
echo "Testing ESP32 build without full ESP-IDF setup"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Check if ESP32 package exists
if [ ! -d "aws-iot-platform-esp32" ]; then
    print_error "ESP32 platform package not found"
    exit 1
fi

# Check Rust installation
print_step "Checking Rust installation..."
if ! command -v rustc &> /dev/null; then
    print_error "Rust not installed. Please install from https://rustup.rs/"
    exit 1
fi
print_success "Rust found: $(rustc --version)"

# Check if ESP32 target is available
print_step "Checking ESP32 target availability..."
if rustup target list | grep -q "riscv32imc-esp-espidf"; then
    if rustup target list --installed | grep -q "riscv32imc-esp-espidf"; then
        print_success "ESP32 target already installed"
    else
        print_step "Installing ESP32 target..."
        if rustup target add riscv32imc-esp-espidf; then
            print_success "ESP32 target installed"
        else
            print_warning "Failed to install ESP32 target (may require ESP-IDF toolchain)"
            print_warning "This is normal on macOS - ESP32 development requires espup"
            ESP_TARGET_AVAILABLE=false
        fi
    fi
else
    print_warning "ESP32 target not available in standard Rust toolchain"
    print_warning "ESP32 development requires espup and ESP-IDF toolchain"
    ESP_TARGET_AVAILABLE=false
fi

# Check for ESP-IDF environment
print_step "Checking ESP-IDF environment..."
if [ -f "$HOME/export-esp.sh" ]; then
    print_success "ESP-IDF environment found"
    source "$HOME/export-esp.sh"
    ESP_IDF_AVAILABLE=true
elif [ -n "$IDF_PATH" ]; then
    print_success "ESP-IDF environment variables found"
    ESP_IDF_AVAILABLE=true
else
    print_warning "ESP-IDF not found - will attempt basic compilation check only"
    ESP_IDF_AVAILABLE=false
fi

# Test basic ESP32 package structure
print_step "Validating ESP32 package structure..."
if [ -f "aws-iot-platform-esp32/Cargo.toml" ]; then
    print_success "ESP32 Cargo.toml found"
else
    print_error "ESP32 Cargo.toml not found"
    exit 1
fi

# Check ESP32 dependencies (only if target is available)
if [ "${ESP_TARGET_AVAILABLE:-true}" = true ]; then
    print_step "Checking ESP32 dependencies..."
    if cargo check --package aws-iot-platform-esp32 --target riscv32imc-esp-espidf; then
        print_success "ESP32 dependencies check passed"
    else
        if [ "$ESP_IDF_AVAILABLE" = false ]; then
            print_warning "ESP32 dependency check failed (ESP-IDF not available)"
            print_warning "This is expected without full ESP-IDF setup"
        else
            print_error "ESP32 dependency check failed"
            exit 1
        fi
    fi
else
    print_warning "Skipping ESP32 dependency check (target not available)"
fi

# Attempt cross-compilation if ESP-IDF is available and target is available
if [ "$ESP_IDF_AVAILABLE" = true ] && [ "${ESP_TARGET_AVAILABLE:-true}" = true ]; then
    print_step "Attempting ESP32-C3 cross-compilation..."
    
    if cargo build --target riscv32imc-esp-espidf --package aws-iot-platform-esp32 --verbose; then
        print_success "ESP32-C3 cross-compilation successful"
        
        # Check if binary was created
        if [ -f "target/riscv32imc-esp-espidf/debug/aws-iot-platform-esp32" ]; then
            BINARY_SIZE=$(stat -f%z "target/riscv32imc-esp-espidf/debug/aws-iot-platform-esp32" 2>/dev/null || stat -c%s "target/riscv32imc-esp-espidf/debug/aws-iot-platform-esp32" 2>/dev/null)
            print_success "ESP32 binary created (size: $(numfmt --to=iec $BINARY_SIZE))"
        fi
        
        # Try release build
        print_step "Building ESP32 release binary..."
        if cargo build --target riscv32imc-esp-espidf --package aws-iot-platform-esp32 --release; then
            print_success "ESP32 release build successful"
            
            if [ -f "target/riscv32imc-esp-espidf/release/aws-iot-platform-esp32" ]; then
                RELEASE_SIZE=$(stat -f%z "target/riscv32imc-esp-espidf/release/aws-iot-platform-esp32" 2>/dev/null || stat -c%s "target/riscv32imc-esp-espidf/release/aws-iot-platform-esp32" 2>/dev/null)
                print_success "ESP32 release binary created (size: $(numfmt --to=iec $RELEASE_SIZE))"
            fi
        else
            print_error "ESP32 release build failed"
            exit 1
        fi
    else
        print_error "ESP32-C3 cross-compilation failed"
        exit 1
    fi
else
    if [ "${ESP_TARGET_AVAILABLE:-true}" = false ]; then
        print_warning "Skipping ESP32 compilation (target not available on this platform)"
        echo ""
        echo "To enable ESP32 development on macOS:"
        echo "1. Install espup: cargo install espup"
        echo "2. Install ESP-IDF: espup install"
        echo "3. Source the environment: source \$HOME/export-esp.sh"
        echo "4. The ESP32 target will be available through ESP-IDF toolchain"
    else
        print_warning "Skipping ESP32 compilation (ESP-IDF not available)"
        echo ""
        echo "To enable full ESP32 testing:"
        echo "1. Install ESP-IDF: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/"
        echo "2. Or use espup: cargo install espup && espup install"
        echo "3. Source the environment: source \$HOME/export-esp.sh"
    fi
fi

echo ""
print_success "ESP32 cross-compilation test completed"

# Provide helpful information
echo ""
echo -e "${BLUE}💡 ESP32 Development Tips:${NC}"
echo "• Use 'cargo build --target riscv32imc-esp-espidf --package aws-iot-platform-esp32' for ESP32 builds"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "• On macOS: Run 'make setup-esp32' to install ESP-IDF toolchain"
    echo "• ESP32 target requires espup, not available in standard Rust toolchain"
else
    echo "• Install espup for easier ESP-IDF management: cargo install espup"
fi
echo "• Check ESP32 examples in aws-iot-examples/examples/"
echo "• Monitor binary size - ESP32-C3 has limited flash memory"