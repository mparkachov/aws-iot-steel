#!/bin/bash

# Pre-push Script - Essential checks before pushing to GitHub
# Runs a subset of CI checks for faster feedback

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

# Load local configuration if it exists
if [ -f ".kiro/local-ci/config.sh" ]; then
    source ".kiro/local-ci/config.sh"
fi

# Parse command line arguments
SKIP_TESTS=${SKIP_TESTS:-false}
SKIP_ESP32=${SKIP_ESP32:-false}
QUICK_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-esp32)
            SKIP_ESP32=true
            shift
            ;;
        --quick)
            QUICK_MODE=true
            SKIP_TESTS=true
            SKIP_ESP32=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --skip-tests    Skip running tests (faster)"
            echo "  --skip-esp32    Skip ESP32 cross-compilation check"
            echo "  --quick         Quick mode (skip tests and ESP32)"
            echo "  -h, --help      Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}🚀 Pre-push Checks${NC}"
if [ "$QUICK_MODE" = true ]; then
    echo "Running in quick mode"
elif [ "$SKIP_TESTS" = true ] || [ "$SKIP_ESP32" = true ]; then
    echo "Running with some checks skipped"
else
    echo "Running essential checks before push"
fi
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Check Rust version
print_step "Checking Rust version..."
RUST_VERSION=$(rustc --version | grep -o '[0-9]\+\.[0-9]\+' | head -1)
REQUIRED_VERSION="1.82"

if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$RUST_VERSION" | sort -V | head -n1)" = "$REQUIRED_VERSION" ]; then
    print_success "Rust version $RUST_VERSION meets minimum requirement ($REQUIRED_VERSION)"
else
    print_error "Rust version $RUST_VERSION is below minimum requirement ($REQUIRED_VERSION)"
    print_error "Please update Rust: rustup update stable"
    OVERALL_SUCCESS=false
fi

OVERALL_SUCCESS=true

# 1. Code Quality (always run)
echo -e "${BLUE}📋 Code Quality${NC}"
echo "------------------------"

print_step "Checking code formatting..."
if cargo fmt --all -- --check; then
    print_success "Code formatting is correct"
else
    print_error "Code formatting issues found"
    print_step "Auto-fixing formatting..."
    cargo fmt --all
    print_success "Code formatted - please review changes"
fi

print_step "Running clippy..."
if cargo clippy --workspace --all-targets --all-features -- -D warnings; then
    print_success "Clippy checks passed"
else
    print_error "Clippy found issues"
    OVERALL_SUCCESS=false
fi

echo ""

# 2. Build Check (always run)
echo -e "${BLUE}🔨 Build Check${NC}"
echo "------------------------"

print_step "Building workspace..."
if cargo build --workspace --lib && cargo build --bin basic_hal_demo --package aws-iot-examples; then
    print_success "Build successful"
else
    print_error "Build failed"
    OVERALL_SUCCESS=false
fi

echo ""

# 3. Tests (unless skipped)
if [ "$SKIP_TESTS" = false ]; then
    echo -e "${BLUE}🧪 Tests${NC}"
    echo "------------------------"
    
    print_step "Running unit tests..."
    if cargo test --workspace --lib; then
        print_success "Unit tests passed"
    else
        print_error "Unit tests failed"
        OVERALL_SUCCESS=false
    fi
    
    print_step "Running integration tests..."
    if cargo test --workspace --test '*'; then
        print_success "Integration tests passed"
    else
        print_error "Integration tests failed"
        OVERALL_SUCCESS=false
    fi
    
    echo ""
else
    print_warning "Skipping tests (use without --skip-tests to run)"
    echo ""
fi

# 4. ESP32 Check (unless skipped)
if [ "$SKIP_ESP32" = false ]; then
    echo -e "${BLUE}🔧 ESP32 Check${NC}"
    echo "------------------------"
    
    if [ -f "scripts/test-esp32-build.sh" ]; then
        if ./scripts/test-esp32-build.sh; then
            print_success "ESP32 check passed"
        else
            print_warning "ESP32 check had issues (not blocking)"
        fi
    else
        print_warning "ESP32 test script not found"
    fi
    
    echo ""
else
    print_warning "Skipping ESP32 check (use without --skip-esp32 to run)"
    echo ""
fi

# 5. Security Audit (if available)
echo -e "${BLUE}🔒 Security${NC}"
echo "------------------------"

if command -v cargo-audit &> /dev/null; then
    print_step "Running security audit..."
    if cargo audit; then
        print_success "Security audit passed"
    else
        print_warning "Security audit found issues (review recommended)"
    fi
else
    print_warning "cargo-audit not installed (install with: cargo install cargo-audit)"
fi

echo ""

# 6. Final Summary
echo -e "${BLUE}📊 Summary${NC}"
echo "========================"

if [ "$OVERALL_SUCCESS" = true ]; then
    print_success "Pre-push checks passed! 🎉"
    echo -e "${GREEN}Your code is ready to push${NC}"
    
    # Show what will happen in CI
    echo ""
    echo -e "${BLUE}What happens next in CI:${NC}"
    echo "• GitHub Actions will run the full test suite"
    echo "• ESP32 cross-compilation will be tested"
    echo "• Security audits will be performed"
    if [ "$SKIP_TESTS" = true ]; then
        echo "• Tests will run (skipped locally)"
    fi
    if [ "$SKIP_ESP32" = true ]; then
        echo "• ESP32 compilation will be tested (skipped locally)"
    fi
    
    exit 0
else
    print_error "Some checks failed!"
    echo -e "${RED}Please fix the issues above before pushing${NC}"
    echo ""
    echo "Quick fixes:"
    echo "  • Format code: cargo fmt --all"
    echo "  • Fix clippy: cargo clippy --workspace --all-targets --fix"
    echo "  • Run tests: cargo test --workspace"
    echo ""
    exit 1
fi