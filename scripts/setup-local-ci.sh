#!/bin/bash

# Setup Local CI Environment
# Installs all tools needed to run local CI checks that mirror GitHub Actions

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

echo -e "${BLUE}🛠️  AWS IoT Steel - Local CI Setup${NC}"
echo "Setting up tools to run GitHub Actions pipeline locally"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Check Rust installation
print_step "Checking Rust installation..."
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    print_success "Rust found: $RUST_VERSION"
else
    print_error "Rust not found. Please install from https://rustup.rs/"
    exit 1
fi

# Install Rust components
print_step "Installing Rust components..."
rustup component add rustfmt clippy
print_success "Rust components installed"

# Check ESP32 target availability (for cross-compilation testing)
print_step "Checking ESP32 target availability..."
if rustup target list | grep -q "riscv32imc-esp-espidf"; then
    if rustup target add riscv32imc-esp-espidf; then
        print_success "ESP32 target installed"
    else
        print_warning "Failed to install ESP32 target (may require ESP-IDF toolchain)"
    fi
else
    print_warning "ESP32 target not available in standard Rust toolchain"
    print_warning "For ESP32 development, install espup: cargo install espup && espup install"
fi

# Install development tools
print_step "Installing Cargo development tools..."

tools=(
    "cargo-audit:Security vulnerability scanner"
    "cargo-deny:Dependency policy enforcement"
    "cargo-outdated:Check for outdated dependencies"
    "cargo-license:License compliance checking"
    "cargo-watch:File watching for development"
)

for tool_info in "${tools[@]}"; do
    tool=$(echo $tool_info | cut -d: -f1)
    desc=$(echo $tool_info | cut -d: -f2)
    
    if command -v $tool &> /dev/null; then
        print_success "$tool already installed ($desc)"
    else
        print_step "Installing $tool ($desc)..."
        if cargo install $tool; then
            print_success "$tool installed"
        else
            print_warning "Failed to install $tool (optional)"
        fi
    fi
done

# Setup Git hooks
print_step "Setting up Git hooks..."
if [ -d ".git" ]; then
    # Create pre-commit hook
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Auto-generated pre-commit hook for AWS IoT Steel
echo "🔍 Running pre-commit checks..."

# Run quick pre-push checks
if [ -f "scripts/pre-push.sh" ]; then
    if ./scripts/pre-push.sh --quick; then
        echo "✅ Pre-commit checks passed"
        exit 0
    else
        echo "❌ Pre-commit checks failed"
        echo "Run 'make pre-push' to see detailed issues"
        exit 1
    fi
else
    echo "⚠️  Pre-push script not found, skipping checks"
    exit 0
fi
EOF
    chmod +x .git/hooks/pre-commit
    print_success "Git pre-commit hook installed"
    
    # Create pre-push hook
    cat > .git/hooks/pre-push << 'EOF'
#!/bin/bash
# Auto-generated pre-push hook for AWS IoT Steel
echo "🚀 Running pre-push validation..."

# Run full pre-push checks
if [ -f "scripts/pre-push.sh" ]; then
    if ./scripts/pre-push.sh; then
        echo "✅ Pre-push validation passed"
        exit 0
    else
        echo "❌ Pre-push validation failed"
        echo ""
        echo "To skip this check (not recommended):"
        echo "  git push --no-verify"
        echo ""
        echo "To run full CI locally:"
        echo "  make ci-local"
        exit 1
    fi
else
    echo "⚠️  Pre-push script not found, skipping checks"
    exit 0
fi
EOF
    chmod +x .git/hooks/pre-push
    print_success "Git pre-push hook installed"
else
    print_warning "Not a Git repository, skipping Git hooks"
fi

# Create deny.toml for supply chain security
if [ ! -f "deny.toml" ]; then
    print_step "Creating deny.toml for supply chain security..."
    cat > deny.toml << 'EOF'
# Cargo deny configuration for AWS IoT Steel
# This file defines policies for dependency management and security

[advisories]
# Deny crates with security vulnerabilities
vulnerability = "deny"
# Warn about unmaintained crates
unmaintained = "warn"
# Deny yanked crates
yanked = "deny"
# Ignore specific advisories (use sparingly)
ignore = [
    # Example: "RUSTSEC-2020-0001",
]

[licenses]
# Deny unlicensed crates
unlicensed = "deny"
# Allow these licenses
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
]
# Deny these licenses
deny = [
    "GPL-2.0",
    "GPL-3.0",
    "AGPL-1.0",
    "AGPL-3.0",
]
# Confidence threshold for license detection
confidence-threshold = 0.8

[bans]
# Warn about multiple versions of the same crate
multiple-versions = "warn"
# Allow multiple versions for these crates (common in embedded development)
multiple-versions-include-dev = true
skip = [
    # Example: { name = "winapi", version = "0.2" },
]
skip-tree = [
    # Example: { name = "windows-sys", version = "0.45" },
]

[sources]
# Deny crates from unknown registries
unknown-registry = "deny"
# Deny crates from Git repositories (use sparingly)
unknown-git = "deny"
# Allow these Git repositories
allow-git = [
    # Example: "https://github.com/rust-embedded/embedded-hal",
]
EOF
    print_success "deny.toml created"
fi

# Test the setup
print_step "Testing local CI setup..."
echo ""

# Test formatting
print_step "Testing code formatting..."
if cargo fmt --all -- --check; then
    print_success "Code formatting test passed"
else
    print_warning "Code formatting needs attention (run 'cargo fmt --all')"
fi

# Test clippy
print_step "Testing clippy..."
if cargo clippy --workspace --all-targets -- -D warnings; then
    print_success "Clippy test passed"
else
    print_warning "Clippy found issues (run 'cargo clippy --workspace --all-targets --fix')"
fi

# Test build
print_step "Testing build..."
if cargo build --workspace; then
    print_success "Build test passed"
else
    print_error "Build test failed"
fi

# Test security audit (if available)
if command -v cargo-audit &> /dev/null; then
    print_step "Testing security audit..."
    if cargo audit; then
        print_success "Security audit test passed"
    else
        print_warning "Security audit found issues"
    fi
fi

echo ""
print_success "Local CI setup completed! 🎉"
echo ""
echo -e "${BLUE}Available commands:${NC}"
echo "  make ci-local       - Run full local CI (mirrors GitHub Actions)"
echo "  make pre-push       - Run essential pre-push checks"
echo "  make pre-push-quick - Run quick pre-push checks"
echo "  make test-esp32     - Test ESP32 cross-compilation"
echo ""
echo -e "${BLUE}Git hooks installed:${NC}"
echo "  pre-commit  - Runs quick checks before each commit"
echo "  pre-push    - Runs full validation before pushing"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "1. Run 'make pre-push' to test your current code"
echo "2. Run 'make ci-local' to run the full CI pipeline"
echo "3. Make changes and commit - hooks will run automatically"
echo "4. Push to GitHub - the same checks will run in CI"