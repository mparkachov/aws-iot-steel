#!/bin/bash

# Quick Check Script - Fast pre-commit checks
# Run this for quick feedback before committing

set -e

# Colors
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

print_error() {
    echo -e "${RED}❌${NC} $1"
}

echo -e "${BLUE}⚡ AWS IoT Steel - Quick Check${NC}"
echo "Running essential checks..."
echo ""

FAILED=false

# 1. Format check
print_step "Checking code formatting..."
if cargo fmt --all -- --check; then
    print_success "Code is properly formatted"
else
    print_error "Code formatting issues found"
    echo "  Fix with: cargo fmt --all"
    FAILED=true
fi

# 2. Clippy check
print_step "Running clippy..."
if cargo clippy --workspace --all-targets -- -D warnings; then
    print_success "No clippy warnings"
else
    print_error "Clippy warnings found"
    echo "  Fix with: cargo clippy --workspace --all-targets --fix"
    FAILED=true
fi

# 3. Quick build check
print_step "Quick build check..."
if cargo check --workspace; then
    print_success "Code compiles successfully"
else
    print_error "Compilation errors found"
    FAILED=true
fi

echo ""

if [ "$FAILED" = true ]; then
    print_error "Quick check failed!"
    echo -e "${YELLOW}Run './scripts/local-ci.sh' for comprehensive testing${NC}"
    exit 1
else
    print_success "Quick check passed! ⚡"
    echo -e "${GREEN}Ready for commit${NC}"
    echo -e "${YELLOW}Run './scripts/local-ci.sh' before pushing for full CI simulation${NC}"
    exit 0
fi