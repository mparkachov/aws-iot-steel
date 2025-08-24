#!/bin/bash
# Firmware signing script for AWS IoT Steel project
# This script signs firmware binaries and creates secure packages

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
FIRMWARE_DIR="${1:-./firmware}"
OUTPUT_DIR="${2:-./signed-artifacts}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Validate inputs
if [[ ! -d "$FIRMWARE_DIR" ]]; then
    log_error "Firmware directory not found: $FIRMWARE_DIR"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

log_info "Starting firmware signing process..."
log_info "Firmware directory: $FIRMWARE_DIR"
log_info "Output directory: $OUTPUT_DIR"

# Find firmware binary
FIRMWARE_BINARY=""
if [[ -f "$FIRMWARE_DIR/aws-iot-platform-esp32.bin" ]]; then
    FIRMWARE_BINARY="$FIRMWARE_DIR/aws-iot-platform-esp32.bin"
elif [[ -f "$FIRMWARE_DIR/aws-iot-platform-esp32" ]]; then
    FIRMWARE_BINARY="$FIRMWARE_DIR/aws-iot-platform-esp32"
else
    log_error "No firmware binary found in $FIRMWARE_DIR"
    exit 1
fi

log_info "Found firmware binary: $(basename "$FIRMWARE_BINARY")"

# Generate firmware metadata
FIRMWARE_VERSION="${GITHUB_SHA:-$(git rev-parse --short HEAD 2>/dev/null || echo 'dev')}"
FIRMWARE_SIZE=$(stat -c%s "$FIRMWARE_BINARY" 2>/dev/null || stat -f%z "$FIRMWARE_BINARY")
FIRMWARE_CHECKSUM=$(sha256sum "$FIRMWARE_BINARY" 2>/dev/null | cut -d' ' -f1 || shasum -a 256 "$FIRMWARE_BINARY" | cut -d' ' -f1)
BUILD_TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
GIT_COMMIT="${GITHUB_SHA:-$(git rev-parse HEAD 2>/dev/null || echo 'unknown')}"
GIT_BRANCH="${GITHUB_REF_NAME:-$(git branch --show-current 2>/dev/null || echo 'unknown')}"

log_info "Generating firmware metadata..."
cat > "$OUTPUT_DIR/firmware-metadata.json" << EOF
{
  "version": "$FIRMWARE_VERSION",
  "target": "esp32-c3-devkit-rust-1",
  "size_bytes": $FIRMWARE_SIZE,
  "checksum_sha256": "$FIRMWARE_CHECKSUM",
  "build_timestamp": "$BUILD_TIMESTAMP",
  "git_commit": "$GIT_COMMIT",
  "git_branch": "$GIT_BRANCH",
  "build_environment": {
    "os": "$(uname -s)",
    "arch": "$(uname -m)",
    "rust_version": "$(rustc --version 2>/dev/null || echo 'unknown')",
    "cargo_version": "$(cargo --version 2>/dev/null || echo 'unknown')"
  },
  "security": {
    "signed": true,
    "signature_algorithm": "RSA-PSS-SHA256",
    "signing_timestamp": "$BUILD_TIMESTAMP"
  }
}
EOF

log_success "Firmware metadata generated"

# Copy firmware binary to output directory
cp "$FIRMWARE_BINARY" "$OUTPUT_DIR/"
FIRMWARE_NAME=$(basename "$FIRMWARE_BINARY")

# Sign firmware
log_info "Signing firmware binary..."

if [[ -n "${FIRMWARE_SIGNING_KEY:-}" ]]; then
    # Production signing with actual private key
    log_info "Using production signing key"
    
    # Create signature using RSA-PSS with SHA-256
    echo -n "$FIRMWARE_CHECKSUM" | openssl dgst -sha256 -sigopt rsa_padding_mode:pss \
        -sigopt rsa_pss_saltlen:-1 -sign <(echo "$FIRMWARE_SIGNING_KEY" | base64 -d) \
        -out "$OUTPUT_DIR/$FIRMWARE_NAME.sig"
    
    # Verify signature
    if [[ -n "${FIRMWARE_PUBLIC_KEY:-}" ]]; then
        echo -n "$FIRMWARE_CHECKSUM" | openssl dgst -sha256 -sigopt rsa_padding_mode:pss \
            -sigopt rsa_pss_saltlen:-1 -verify <(echo "$FIRMWARE_PUBLIC_KEY" | base64 -d) \
            -signature "$OUTPUT_DIR/$FIRMWARE_NAME.sig"
        log_success "Signature verified successfully"
    fi
else
    # Development/testing mode - create placeholder signature
    log_warning "No signing key provided, creating development signature"
    
    # Create a deterministic development signature based on checksum
    echo "DEV-SIGNATURE-$(echo "$FIRMWARE_CHECKSUM" | cut -c1-16)" > "$OUTPUT_DIR/$FIRMWARE_NAME.sig"
    
    # Update metadata to reflect development signing
    jq '.security.signed = false | .security.signature_algorithm = "DEV-PLACEHOLDER"' \
        "$OUTPUT_DIR/firmware-metadata.json" > "$OUTPUT_DIR/firmware-metadata.tmp" && \
        mv "$OUTPUT_DIR/firmware-metadata.tmp" "$OUTPUT_DIR/firmware-metadata.json"
fi

log_success "Firmware signed successfully"

# Create Steel program packages
log_info "Creating Steel program packages..."

STEEL_PROGRAMS_DIR="$PROJECT_ROOT/aws-iot-core/examples/steel"
STEEL_PACKAGE_DIR="$OUTPUT_DIR/steel-programs"
mkdir -p "$STEEL_PACKAGE_DIR"

if [[ -d "$STEEL_PROGRAMS_DIR" ]]; then
    for steel_file in "$STEEL_PROGRAMS_DIR"/*.scm; do
        if [[ -f "$steel_file" ]]; then
            PROGRAM_NAME=$(basename "$steel_file" .scm)
            PROGRAM_SIZE=$(stat -c%s "$steel_file" 2>/dev/null || stat -f%z "$steel_file")
            PROGRAM_CHECKSUM=$(sha256sum "$steel_file" 2>/dev/null | cut -d' ' -f1 || shasum -a 256 "$steel_file" | cut -d' ' -f1)
            
            # Create Steel program package
            cat > "$STEEL_PACKAGE_DIR/$PROGRAM_NAME.json" << EOF
{
  "program_id": "steel-$PROGRAM_NAME-$FIRMWARE_VERSION",
  "name": "$PROGRAM_NAME",
  "version": "$FIRMWARE_VERSION",
  "description": "Steel program: $PROGRAM_NAME",
  "author": "AWS IoT Steel Team",
  "created_at": "$BUILD_TIMESTAMP",
  "steel_code": $(jq -Rs . < "$steel_file"),
  "checksum_sha256": "$PROGRAM_CHECKSUM",
  "size_bytes": $PROGRAM_SIZE,
  "metadata": {
    "memory_requirement": 32768,
    "execution_timeout": 3600,
    "auto_restart": false,
    "priority": "normal"
  }
}
EOF
            
            log_info "Packaged Steel program: $PROGRAM_NAME"
        fi
    done
    
    log_success "Steel programs packaged successfully"
else
    log_warning "Steel programs directory not found: $STEEL_PROGRAMS_DIR"
fi

# Create comprehensive artifact package
log_info "Creating final artifact package..."

PACKAGE_NAME="aws-iot-steel-artifacts-$FIRMWARE_VERSION.tar.gz"
cd "$OUTPUT_DIR"

tar -czf "$PACKAGE_NAME" \
    "$FIRMWARE_NAME" \
    "$FIRMWARE_NAME.sig" \
    "firmware-metadata.json" \
    $(find steel-programs -name "*.json" 2>/dev/null || true)

# Generate package manifest
PACKAGE_SIZE=$(stat -c%s "$PACKAGE_NAME" 2>/dev/null || stat -f%z "$PACKAGE_NAME")
PACKAGE_CHECKSUM=$(sha256sum "$PACKAGE_NAME" 2>/dev/null | cut -d' ' -f1 || shasum -a 256 "$PACKAGE_NAME" | cut -d' ' -f1)

cat > "package-manifest.json" << EOF
{
  "package_name": "$PACKAGE_NAME",
  "package_version": "$FIRMWARE_VERSION",
  "package_size": $PACKAGE_SIZE,
  "package_checksum": "$PACKAGE_CHECKSUM",
  "created_at": "$BUILD_TIMESTAMP",
  "contents": {
    "firmware_binary": "$FIRMWARE_NAME",
    "firmware_signature": "$FIRMWARE_NAME.sig",
    "firmware_metadata": "firmware-metadata.json",
    "steel_programs": $(find steel-programs -name "*.json" 2>/dev/null | jq -R . | jq -s . || echo '[]')
  },
  "deployment_info": {
    "target_platform": "esp32-c3-devkit-rust-1",
    "aws_region": "${AWS_REGION:-us-east-1}",
    "deployment_stage": "${DEPLOYMENT_STAGE:-development}"
  }
}
EOF

cd - > /dev/null

log_success "Artifact package created: $PACKAGE_NAME"
if command -v numfmt &> /dev/null; then
    log_info "Package size: $(numfmt --to=iec $PACKAGE_SIZE)"
else
    log_info "Package size: $PACKAGE_SIZE bytes"
fi
log_info "Package checksum: $PACKAGE_CHECKSUM"

# Validation
log_info "Validating artifact package..."

# Check package integrity
if tar -tzf "$OUTPUT_DIR/$PACKAGE_NAME" > /dev/null 2>&1; then
    log_success "Package integrity verified"
else
    log_error "Package integrity check failed"
    exit 1
fi

# Check required files
REQUIRED_FILES=("$FIRMWARE_NAME" "$FIRMWARE_NAME.sig" "firmware-metadata.json")
for file in "${REQUIRED_FILES[@]}"; do
    if tar -tzf "$OUTPUT_DIR/$PACKAGE_NAME" | grep -q "^$file$"; then
        log_success "Required file present: $file"
    else
        log_error "Required file missing: $file"
        exit 1
    fi
done

log_success "All validations passed"

# Summary
echo
log_success "=== FIRMWARE SIGNING COMPLETE ==="
echo "📦 Package: $PACKAGE_NAME"
echo "🔢 Version: $FIRMWARE_VERSION"
if command -v numfmt &> /dev/null; then
    echo "📏 Size: $(numfmt --to=iec $PACKAGE_SIZE)"
else
    echo "📏 Size: $PACKAGE_SIZE bytes"
fi
echo "🔐 Checksum: $PACKAGE_CHECKSUM"
echo "📁 Output: $OUTPUT_DIR"
echo

log_info "Artifacts ready for deployment to AWS S3"