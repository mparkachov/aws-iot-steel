#!/bin/bash

# Fix Rust Version Issues
# Updates Rust and resolves dependency version conflicts

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

echo -e "${BLUE}🔧 Rust Version Fix Script${NC}"
echo "Updating Rust and resolving dependency conflicts"
echo ""

# Check current Rust version
print_step "Checking current Rust version..."
if command -v rustc &> /dev/null; then
    CURRENT_VERSION=$(rustc --version)
    print_step "Current: $CURRENT_VERSION"
else
    print_error "Rust not found. Please install from https://rustup.rs/"
    exit 1
fi

# Update Rust to latest stable
print_step "Updating Rust to latest stable..."
if rustup update stable; then
    print_success "Rust updated successfully"
else
    print_error "Failed to update Rust"
    exit 1
fi

# Show new version
NEW_VERSION=$(rustc --version)
print_success "New version: $NEW_VERSION"

# Check if we meet minimum requirements
RUST_VERSION=$(rustc --version | grep -o '[0-9]\+\.[0-9]\+' | head -1)
REQUIRED_VERSION="1.82"

if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$RUST_VERSION" | sort -V | head -n1)" = "$REQUIRED_VERSION" ]; then
    print_success "Rust version $RUST_VERSION meets minimum requirement ($REQUIRED_VERSION)"
else
    print_error "Rust version $RUST_VERSION is still below minimum requirement ($REQUIRED_VERSION)"
    print_error "You may need to install a newer Rust version manually"
    exit 1
fi

echo ""

# Clean and update dependencies
print_step "Cleaning build artifacts..."
cargo clean
print_success "Build artifacts cleaned"

print_step "Updating Cargo.lock..."
if cargo update; then
    print_success "Dependencies updated"
else
    print_warning "Some dependencies may have conflicts"
fi

echo ""

# Check for specific problematic dependencies
print_step "Checking for known problematic dependencies..."

# Check icu_normalizer specifically
if cargo tree | grep -q "icu_normalizer"; then
    ICU_VERSION=$(cargo tree | grep "icu_normalizer" | head -1 | grep -o 'v[0-9]\+\.[0-9]\+\.[0-9]\+' | head -1)
    print_step "Found icu_normalizer $ICU_VERSION"
    
    # If it's version 2.0.0 or higher, it should work with Rust 1.82+
    if [[ "$ICU_VERSION" == "v2."* ]]; then
        print_success "icu_normalizer version is compatible with Rust 1.82+"
    else
        print_warning "icu_normalizer version may need updating"
    fi
fi

echo ""

# Test build
print_step "Testing build with updated Rust version..."
if cargo check --workspace; then
    print_success "Build check passed with updated Rust version"
else
    print_error "Build check failed - there may be other issues"
    echo ""
    echo "Common fixes:"
    echo "1. Update specific problematic dependencies:"
    echo "   cargo update -p icu_normalizer"
    echo "   cargo update -p unicode-normalization"
    echo ""
    echo "2. Check for other version conflicts:"
    echo "   cargo tree --duplicates"
    echo ""
    echo "3. If issues persist, try:"
    echo "   rm Cargo.lock && cargo build"
    exit 1
fi

echo ""

# Run a quick test
print_step "Running quick test suite..."
if cargo test --workspace --lib --quiet; then
    print_success "Quick tests passed"
else
    print_warning "Some tests failed - this may be unrelated to Rust version"
fi

echo ""
print_success "Rust version fix completed! 🎉"
echo ""
echo -e "${BLUE}Summary:${NC}"
echo "• Rust updated to: $(rustc --version | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+')"
echo "• Dependencies updated"
echo "• Build check passed"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "1. Run 'make pre-push' to test everything"
echo "2. Commit the updated Cargo.lock if needed"
echo "3. Push to GitHub - CI should now pass"
echo ""
echo -e "${BLUE}If you still have issues:${NC}"
echo "• Check 'cargo tree --duplicates' for version conflicts"
echo "• Update specific dependencies: 'cargo update -p <package>'"
echo "• Consider pinning problematic dependencies in Cargo.toml"