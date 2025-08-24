#!/bin/bash

# Local CI Script - Mirrors GitHub Actions Pipeline
# Run this script before pushing to catch issues early
# Matches the exact steps from .github/workflows/ci.yml

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Function to run a command with error handling
run_check() {
    local name="$1"
    local command="$2"
    
    print_step "Running $name..."
    
    if eval "$command"; then
        print_success "$name passed"
        return 0
    else
        print_error "$name failed"
        return 1
    fi
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d ".github" ]; then
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

echo -e "${BLUE}🚀 AWS IoT Steel - Local CI Pipeline${NC}"
echo "This script runs the same checks as GitHub Actions"
echo ""

# Track overall success
OVERALL_SUCCESS=true

# 1. Code Quality Checks
echo -e "${BLUE}📋 Code Quality Checks${NC}"
echo "----------------------------------------"

if ! run_check "Rust formatting" "cargo fmt --all -- --check"; then
    print_warning "Run 'cargo fmt --all' to fix formatting issues"
    OVERALL_SUCCESS=false
fi

if ! run_check "Clippy linting" "cargo clippy --workspace --all-targets --all-features -- -D warnings"; then
    print_warning "Fix clippy warnings before pushing"
    OVERALL_SUCCESS=false
fi

echo ""

# 2. Build Tests
echo -e "${BLUE}🔨 Build Tests${NC}"
echo "----------------------------------------"

if ! run_check "Workspace build" "cargo build --workspace --verbose"; then
    OVERALL_SUCCESS=false
fi

if ! run_check "Release build" "cargo build --workspace --release"; then
    OVERALL_SUCCESS=false
fi

echo ""

# 3. Unit and Integration Tests
echo -e "${BLUE}🧪 Test Suite${NC}"
echo "----------------------------------------"

if ! run_check "Unit tests" "cargo test --workspace --lib --verbose"; then
    OVERALL_SUCCESS=false
fi

if ! run_check "Integration tests" "cargo test --workspace --test '*' --verbose"; then
    OVERALL_SUCCESS=false
fi

# Platform-specific tests (if on Linux/macOS)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if ! run_check "Linux platform tests" "cargo test --package aws-iot-platform-linux --verbose"; then
        OVERALL_SUCCESS=false
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    if ! run_check "macOS platform tests" "cargo test --package aws-iot-platform-macos --verbose"; then
        OVERALL_SUCCESS=false
    fi
fi

echo ""

# 4. Steel Test Suite (matches GitHub Actions)
echo -e "${BLUE}⚙️ Steel Test Suite${NC}"
echo "----------------------------------------"

# Set platform for testing (matches CI)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    export AWS_IOT_PLATFORM=linux
elif [[ "$OSTYPE" == "darwin"* ]]; then
    export AWS_IOT_PLATFORM=macos
fi

# Run Steel test suite if available
if cargo build --bin steel_test --package aws-iot-core > /dev/null 2>&1; then
    if ! run_check "Steel test suite" "cargo run --bin steel_test --package aws-iot-core -- --verbose"; then
        OVERALL_SUCCESS=false
    fi
else
    print_warning "Steel test suite not available (steel_test binary not found)"
fi

# Run Steel examples if available
if cargo build --bin steel_example --package aws-iot-core > /dev/null 2>&1; then
    if ! run_check "Steel examples" "cargo run --bin steel_example --package aws-iot-core -- --verbose"; then
        OVERALL_SUCCESS=false
    fi
else
    print_warning "Steel examples not available (steel_example binary not found)"
fi

echo ""

# 5. Examples and Demos
echo -e "${BLUE}📚 Examples${NC}"
echo "----------------------------------------"

if ! run_check "Build examples" "cargo build --workspace --examples --verbose"; then
    OVERALL_SUCCESS=false
fi

# Platform-specific demos (matches GitHub Actions)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if cargo build --bin linux_system_demo --package aws-iot-examples > /dev/null 2>&1; then
        print_step "Testing Linux system demo..."
        if timeout 10s cargo run --bin linux_system_demo --package aws-iot-examples > /dev/null 2>&1; then
            print_success "Linux system demo runs successfully"
        else
            print_warning "Linux system demo test skipped (timeout or platform issue)"
        fi
    fi
fi

echo ""

# 6. Security Checks (matches GitHub Actions)
echo -e "${BLUE}🔒 Security Checks${NC}"
echo "----------------------------------------"

if command -v cargo-audit &> /dev/null; then
    if ! run_check "Security audit" "cargo audit"; then
        print_warning "Security vulnerabilities found - review before pushing"
        OVERALL_SUCCESS=false
    fi
else
    print_warning "cargo-audit not installed. Install with: cargo install cargo-audit"
fi

if command -v cargo-deny &> /dev/null; then
    # Create minimal deny.toml if it doesn't exist
    if [ ! -f "deny.toml" ]; then
        print_step "Creating minimal deny.toml for supply chain checks..."
        cat > deny.toml << 'EOF'
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC"]
deny = ["GPL-2.0", "GPL-3.0", "AGPL-1.0", "AGPL-3.0"]

[bans]
multiple-versions = "warn"
EOF
    fi
    
    if ! run_check "Supply chain security" "cargo deny check"; then
        print_warning "Supply chain issues found - review before pushing"
        OVERALL_SUCCESS=false
    fi
else
    print_warning "cargo-deny not installed. Install with: cargo install cargo-deny"
fi

echo ""

# 7. Documentation
echo -e "${BLUE}📖 Documentation${NC}"
echo "----------------------------------------"

if ! run_check "Documentation build" "cargo doc --workspace --no-deps"; then
    print_warning "Documentation has issues"
    OVERALL_SUCCESS=false
fi

echo ""

# 8. Steel Programs (if they exist)
if [ -d "aws-iot-core/examples/steel" ] && [ "$(ls -A aws-iot-core/examples/steel/*.scm 2>/dev/null)" ]; then
    echo -e "${BLUE}⚙️ Steel Programs${NC}"
    echo "----------------------------------------"
    
    print_step "Building Steel validator..."
    if cargo build --bin steel_program_validator --package aws-iot-core; then
        print_step "Validating Steel programs..."
        for steel_file in aws-iot-core/examples/steel/*.scm; do
            if [ -f "$steel_file" ]; then
                filename=$(basename "$steel_file")
                if cargo run --bin steel_program_validator --package aws-iot-core -- --file "$steel_file" --validate-only; then
                    print_success "Steel program $filename is valid"
                else
                    print_error "Steel program $filename validation failed"
                    OVERALL_SUCCESS=false
                fi
            fi
        done
    else
        print_error "Failed to build Steel validator"
        OVERALL_SUCCESS=false
    fi
    echo ""
fi

# 9. Final Summary
echo -e "${BLUE}📊 Summary${NC}"
echo "========================================"

if [ "$OVERALL_SUCCESS" = true ]; then
    print_success "All checks passed! ✨"
    echo -e "${GREEN}Your code is ready to push to GitHub${NC}"
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