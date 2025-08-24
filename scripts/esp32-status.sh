#!/bin/bash

# ESP32 Development Status Checker
# Checks current ESP32 toolchain status and provides recommendations

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

echo -e "${BLUE}🔍 ESP32 Development Status Check${NC}"
echo "Checking your ESP32 toolchain setup..."
echo ""

# Check espup
print_step "Checking espup installation..."
if command -v espup &> /dev/null; then
    ESPUP_VERSION=$(espup --version 2>/dev/null || echo "unknown")
    print_success "espup installed: $ESPUP_VERSION"
else
    print_error "espup not installed"
    ESPUP_MISSING=true
fi

# Check RISC-V targets
print_step "Checking RISC-V targets..."
RISCV_TARGETS=$(rustup target list --installed | grep riscv | wc -l)
if [ "$RISCV_TARGETS" -gt 0 ]; then
    print_success "RISC-V targets available: $RISCV_TARGETS"
    rustup target list --installed | grep riscv | sed 's/^/  - /'
else
    print_warning "No RISC-V targets installed"
fi

# Check ESP-IDF target
print_step "Checking ESP-IDF target..."
if rustup target list --installed | grep -q "riscv32imc-esp-espidf"; then
    print_success "ESP-IDF target available: riscv32imc-esp-espidf"
    ESP_IDF_TARGET=true
else
    print_warning "ESP-IDF target not available: riscv32imc-esp-espidf"
    ESP_IDF_TARGET=false
fi

# Check ESP-IDF environment
print_step "Checking ESP-IDF environment..."
if [ -f "$HOME/export-esp.sh" ]; then
    print_success "ESP-IDF export script exists"
    source "$HOME/export-esp.sh" 2>/dev/null || true
    if [ -n "$IDF_PATH" ]; then
        print_success "ESP-IDF environment configured: $IDF_PATH"
        ESP_IDF_ENV=true
    else
        print_warning "ESP-IDF export script exists but IDF_PATH not set"
        ESP_IDF_ENV=false
    fi
else
    print_warning "ESP-IDF export script not found"
    ESP_IDF_ENV=false
fi

# Check ESP32 platform build
print_step "Testing ESP32 platform build..."
if cargo check --package aws-iot-platform-esp32 --quiet 2>/dev/null; then
    print_success "ESP32 platform builds successfully (using stubs)"
    ESP32_BUILD=true
else
    print_error "ESP32 platform build failed"
    ESP32_BUILD=false
fi

# Check ESP32 cross-compilation
print_step "Testing ESP32 cross-compilation..."
if [ "$ESP_IDF_TARGET" = true ]; then
    if cargo check --package aws-iot-platform-esp32 --target riscv32imc-esp-espidf --quiet 2>/dev/null; then
        print_success "ESP32 cross-compilation works"
        ESP32_CROSS=true
    else
        print_error "ESP32 cross-compilation failed"
        ESP32_CROSS=false
    fi
else
    print_warning "ESP32 cross-compilation not possible (target not available)"
    ESP32_CROSS=false
fi

echo ""
echo -e "${BLUE}📊 Summary${NC}"
echo "========================================"

# Determine overall status
if [ "$ESP32_BUILD" = true ] && [ "$ESP32_CROSS" = true ]; then
    print_success "ESP32 development fully working! 🎉"
    STATUS="FULLY_WORKING"
elif [ "$ESP32_BUILD" = true ]; then
    print_warning "ESP32 development partially working (stubs only)"
    STATUS="PARTIALLY_WORKING"
else
    print_error "ESP32 development not working"
    STATUS="NOT_WORKING"
fi

echo ""
echo -e "${BLUE}🎯 Recommendations${NC}"
echo "========================================"

case $STATUS in
    "FULLY_WORKING")
        echo "✅ Your ESP32 setup is complete!"
        echo "   • You can develop ESP32 firmware locally"
        echo "   • You can flash ESP32 devices"
        echo "   • Continue with normal development workflow"
        ;;
    "PARTIALLY_WORKING")
        echo "⚠️  Your ESP32 setup works for development but not firmware builds"
        echo ""
        echo "🚀 Recommended: Continue with current setup"
        echo "   • ESP32 code compiles and runs using stubs"
        echo "   • GitHub Actions will test actual ESP32 builds"
        echo "   • Fast local development workflow"
        echo ""
        echo "💡 To enable ESP32 firmware builds:"
        echo "   1. Install ESP-IDF: https://docs.espressif.com/projects/esp-idf/"
        echo "   2. Or run: ./scripts/setup-esp32-macos.sh"
        echo "   3. Or use Docker: docker run espressif/idf:latest"
        ;;
    "NOT_WORKING")
        echo "❌ ESP32 setup needs attention"
        echo ""
        echo "🔧 Quick fixes:"
        echo "   1. Install espup: cargo install espup"
        echo "   2. Install targets: espup install"
        echo "   3. Run setup script: ./scripts/setup-esp32-macos.sh"
        echo ""
        echo "🚀 Alternative: Skip ESP32 for now"
        echo "   • Set: export SKIP_ESP32=1"
        echo "   • Continue development on macOS/Linux"
        echo "   • ESP32 still tested in CI"
        ;;
esac

echo ""
echo -e "${BLUE}📚 Resources${NC}"
echo "========================================"
echo "• ESP32 Status Report: ESP32_TOOLCHAIN_STATUS.md"
echo "• ESP32 Troubleshooting: docs/esp32-troubleshooting.md"
echo "• ESP-IDF Documentation: https://docs.espressif.com/projects/esp-idf/"
echo "• espup Repository: https://github.com/esp-rs/espup"

echo ""
echo -e "${BLUE}🧪 Quick Tests${NC}"
echo "========================================"
echo "• Test ESP32 build: cargo build --package aws-iot-platform-esp32"
echo "• Test workspace: cargo build --workspace"
echo "• Run all tests: cargo test --workspace"

if [ "$ESP_IDF_TARGET" = true ]; then
    echo "• Test ESP32 firmware: cargo build --target riscv32imc-esp-espidf --package aws-iot-platform-esp32"
fi

echo ""