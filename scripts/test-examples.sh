#!/bin/bash

# Test Examples Build Script
# Tests building examples with proper platform features

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

echo -e "${BLUE}🧪 Testing Examples Build${NC}"
echo "Building examples with proper platform features"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

OVERALL_SUCCESS=true

# Build platform-agnostic examples
print_step "Building platform-agnostic examples..."
if cargo build --bin basic_hal_demo --package aws-iot-examples --verbose; then
    print_success "Platform-agnostic examples built successfully"
else
    print_error "Failed to build platform-agnostic examples"
    OVERALL_SUCCESS=false
fi

echo ""

# Build platform-specific examples
print_step "Building platform-specific examples..."

if [[ "$OSTYPE" == "darwin"* ]]; then
    print_step "Building macOS-specific examples..."
    if cargo build --bin macos_system_demo --package aws-iot-examples --features macos-platform --verbose; then
        print_success "macOS examples built successfully"
    else
        print_error "Failed to build macOS examples"
        OVERALL_SUCCESS=false
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    print_step "Building Linux-specific examples..."
    print_warning "No Linux-specific examples available yet"
    # if cargo build --bin linux_system_demo --package aws-iot-examples --features linux-platform --verbose; then
    #     print_success "Linux examples built successfully"
    # else
    #     print_error "Failed to build Linux examples"
    #     OVERALL_SUCCESS=false
    # fi
else
    print_warning "Unknown platform: $OSTYPE"
    print_warning "Only building platform-agnostic examples"
fi

echo ""

# Test that examples can be listed
print_step "Listing available examples..."
echo "Available examples:"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "  • basic_hal_demo (cross-platform)"
    echo "  • macos_system_demo (macOS only)"
else
    echo "  • basic_hal_demo (cross-platform)"
fi

echo ""

# Summary
if [ "$OVERALL_SUCCESS" = true ]; then
    print_success "All examples built successfully! 🎉"
    echo ""
    echo -e "${BLUE}To run examples:${NC}"
    echo "  cargo run --bin basic_hal_demo --package aws-iot-examples"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "  cargo run --bin macos_system_demo --package aws-iot-examples --features macos-platform"
    fi
    exit 0
else
    print_error "Some examples failed to build"
    echo ""
    echo "Troubleshooting:"
    echo "• Check that all dependencies are installed"
    echo "• Run 'cargo clean' and try again"
    echo "• Check platform-specific dependencies"
    exit 1
fi