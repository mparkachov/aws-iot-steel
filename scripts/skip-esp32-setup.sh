#!/bin/bash

# Skip ESP32 Setup - Configure local CI to skip ESP32 checks
# For users who don't need ESP32 development right now

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

echo -e "${BLUE}⏭️  Skip ESP32 Setup${NC}"
echo "Configuring local CI to skip ESP32 checks"
echo ""

# Create a local configuration file to skip ESP32
CONFIG_DIR=".kiro/local-ci"
CONFIG_FILE="$CONFIG_DIR/config.sh"

print_step "Creating local CI configuration..."
mkdir -p "$CONFIG_DIR"

cat > "$CONFIG_FILE" << 'EOF'
#!/bin/bash
# Local CI Configuration
# This file is sourced by local CI scripts

# Skip ESP32 checks by default
SKIP_ESP32=true

# You can override this by setting environment variables:
# SKIP_ESP32=false make pre-push
EOF

print_success "Local CI configuration created"

# Update .gitignore to ignore local CI config
if [ -f ".gitignore" ]; then
    if ! grep -q ".kiro/local-ci/" ".gitignore"; then
        echo "" >> .gitignore
        echo "# Local CI configuration" >> .gitignore
        echo ".kiro/local-ci/" >> .gitignore
        print_success "Added local CI config to .gitignore"
    fi
fi

echo ""
print_success "ESP32 setup skipped! 🎉"
echo ""
echo -e "${BLUE}What this means:${NC}"
echo "• Local CI scripts will skip ESP32 compilation checks"
echo "• GitHub Actions will still test ESP32 (that's good!)"
echo "• You can focus on other platform development"
echo ""
echo -e "${BLUE}Available commands:${NC}"
echo "  make pre-push       - Run checks (ESP32 skipped)"
echo "  make pre-push-quick - Quick checks"
echo "  make ci-local       - Full CI (ESP32 skipped)"
echo ""
echo -e "${BLUE}To enable ESP32 later:${NC}"
echo "1. Delete: rm -rf .kiro/local-ci/"
echo "2. Run: make setup-esp32"
echo ""
echo -e "${BLUE}To temporarily enable ESP32:${NC}"
echo "  SKIP_ESP32=false make pre-push"