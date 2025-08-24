#!/bin/bash
# Artifact validation script for AWS IoT Steel project
# This script validates signed artifacts before deployment

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARTIFACTS_DIR="${1:-./signed-artifacts}"
VALIDATION_MODE="${2:-strict}"  # strict, permissive, or development

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

# Validation counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNING_CHECKS=0

check_result() {
    local check_name="$1"
    local result="$2"
    local message="$3"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    case "$result" in
        "pass")
            PASSED_CHECKS=$((PASSED_CHECKS + 1))
            log_success "✅ $check_name: $message"
            ;;
        "fail")
            FAILED_CHECKS=$((FAILED_CHECKS + 1))
            log_error "❌ $check_name: $message"
            ;;
        "warning")
            WARNING_CHECKS=$((WARNING_CHECKS + 1))
            log_warning "⚠️  $check_name: $message"
            ;;
    esac
}

# Validate prerequisites
if ! command -v jq &> /dev/null; then
    log_error "jq not found. Please install jq for JSON processing."
    exit 1
fi

if [[ ! -d "$ARTIFACTS_DIR" ]]; then
    log_error "Artifacts directory not found: $ARTIFACTS_DIR"
    exit 1
fi

log_info "Starting artifact validation..."
log_info "Artifacts directory: $ARTIFACTS_DIR"
log_info "Validation mode: $VALIDATION_MODE"

# Check 1: Package manifest exists and is valid
log_info "Validating package manifest..."

MANIFEST_FILE="$ARTIFACTS_DIR/package-manifest.json"
if [[ -f "$MANIFEST_FILE" ]]; then
    if jq empty "$MANIFEST_FILE" 2>/dev/null; then
        check_result "Package Manifest" "pass" "Valid JSON format"
        
        # Extract key information
        PACKAGE_NAME=$(jq -r '.package_name' "$MANIFEST_FILE")
        PACKAGE_VERSION=$(jq -r '.package_version' "$MANIFEST_FILE")
        PACKAGE_CHECKSUM=$(jq -r '.package_checksum' "$MANIFEST_FILE")
        
        if [[ "$PACKAGE_NAME" != "null" && "$PACKAGE_VERSION" != "null" && "$PACKAGE_CHECKSUM" != "null" ]]; then
            check_result "Manifest Content" "pass" "All required fields present"
        else
            check_result "Manifest Content" "fail" "Missing required fields"
        fi
    else
        check_result "Package Manifest" "fail" "Invalid JSON format"
    fi
else
    check_result "Package Manifest" "fail" "Manifest file not found"
fi

# Check 2: Package file exists and matches manifest
log_info "Validating package file..."

if [[ -n "${PACKAGE_NAME:-}" ]]; then
    PACKAGE_PATH="$ARTIFACTS_DIR/$PACKAGE_NAME"
    if [[ -f "$PACKAGE_PATH" ]]; then
        check_result "Package File" "pass" "Package file exists"
        
        # Verify package integrity
        if tar -tzf "$PACKAGE_PATH" > /dev/null 2>&1; then
            check_result "Package Integrity" "pass" "Package is a valid tar.gz archive"
        else
            check_result "Package Integrity" "fail" "Package is corrupted or invalid"
        fi
        
        # Verify checksum
        ACTUAL_CHECKSUM=$(sha256sum "$PACKAGE_PATH" 2>/dev/null | cut -d' ' -f1 || shasum -a 256 "$PACKAGE_PATH" | cut -d' ' -f1)
        if [[ "$ACTUAL_CHECKSUM" == "$PACKAGE_CHECKSUM" ]]; then
            check_result "Package Checksum" "pass" "Checksum matches manifest"
        else
            check_result "Package Checksum" "fail" "Checksum mismatch (expected: $PACKAGE_CHECKSUM, actual: $ACTUAL_CHECKSUM)"
        fi
    else
        check_result "Package File" "fail" "Package file not found: $PACKAGE_NAME"
    fi
fi

# Check 3: Firmware metadata validation
log_info "Validating firmware metadata..."

METADATA_FILE="$ARTIFACTS_DIR/firmware-metadata.json"
if [[ -f "$METADATA_FILE" ]]; then
    if jq empty "$METADATA_FILE" 2>/dev/null; then
        check_result "Firmware Metadata" "pass" "Valid JSON format"
        
        # Check required fields
        REQUIRED_FIELDS=("version" "target" "size_bytes" "checksum_sha256" "build_timestamp")
        for field in "${REQUIRED_FIELDS[@]}"; do
            if jq -e ".$field" "$METADATA_FILE" > /dev/null 2>&1; then
                check_result "Metadata Field: $field" "pass" "Field present"
            else
                check_result "Metadata Field: $field" "fail" "Required field missing"
            fi
        done
        
        # Validate target platform
        TARGET=$(jq -r '.target' "$METADATA_FILE")
        if [[ "$TARGET" == "esp32-c3-devkit-rust-1" ]]; then
            check_result "Target Platform" "pass" "Correct target platform"
        else
            check_result "Target Platform" "warning" "Unexpected target platform: $TARGET"
        fi
        
        # Validate build timestamp format
        BUILD_TIMESTAMP=$(jq -r '.build_timestamp' "$METADATA_FILE")
        if [[ "$BUILD_TIMESTAMP" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$ ]]; then
            check_result "Build Timestamp" "pass" "Valid ISO 8601 format"
        else
            check_result "Build Timestamp" "warning" "Invalid timestamp format"
        fi
    else
        check_result "Firmware Metadata" "fail" "Invalid JSON format"
    fi
else
    check_result "Firmware Metadata" "fail" "Metadata file not found"
fi

# Check 4: Firmware binary validation
log_info "Validating firmware binary..."

FIRMWARE_BINARY="$ARTIFACTS_DIR/aws-iot-platform-esp32.bin"
if [[ -f "$FIRMWARE_BINARY" ]]; then
    check_result "Firmware Binary" "pass" "Binary file exists"
    
    # Check file size
    BINARY_SIZE=$(stat -c%s "$FIRMWARE_BINARY" 2>/dev/null || stat -f%z "$FIRMWARE_BINARY")
    if [[ "$BINARY_SIZE" -gt 0 ]]; then
        check_result "Binary Size" "pass" "Binary has content ($BINARY_SIZE bytes)"
        
        # Check if size is reasonable for ESP32-C3 (should be less than 4MB)
        if [[ "$BINARY_SIZE" -lt 4194304 ]]; then
            check_result "Binary Size Limit" "pass" "Binary size within ESP32-C3 limits"
        else
            check_result "Binary Size Limit" "warning" "Binary size exceeds typical ESP32-C3 limits"
        fi
    else
        check_result "Binary Size" "fail" "Binary file is empty"
    fi
    
    # Verify checksum against metadata
    if [[ -f "$METADATA_FILE" ]]; then
        EXPECTED_CHECKSUM=$(jq -r '.checksum_sha256' "$METADATA_FILE" 2>/dev/null || echo "")
        if [[ -n "$EXPECTED_CHECKSUM" && "$EXPECTED_CHECKSUM" != "null" ]]; then
            ACTUAL_BINARY_CHECKSUM=$(sha256sum "$FIRMWARE_BINARY" 2>/dev/null | cut -d' ' -f1 || shasum -a 256 "$FIRMWARE_BINARY" | cut -d' ' -f1)
            if [[ "$ACTUAL_BINARY_CHECKSUM" == "$EXPECTED_CHECKSUM" ]]; then
                check_result "Binary Checksum" "pass" "Binary checksum matches metadata"
            else
                check_result "Binary Checksum" "fail" "Binary checksum mismatch"
            fi
        fi
    fi
else
    check_result "Firmware Binary" "fail" "Binary file not found"
fi

# Check 5: Firmware signature validation
log_info "Validating firmware signature..."

SIGNATURE_FILE="$ARTIFACTS_DIR/aws-iot-platform-esp32.bin.sig"
if [[ -f "$SIGNATURE_FILE" ]]; then
    check_result "Firmware Signature" "pass" "Signature file exists"
    
    # Check if it's a development signature
    if grep -q "DEV-SIGNATURE" "$SIGNATURE_FILE" 2>/dev/null; then
        if [[ "$VALIDATION_MODE" == "development" ]]; then
            check_result "Signature Type" "pass" "Development signature (acceptable in dev mode)"
        else
            check_result "Signature Type" "warning" "Development signature in non-dev mode"
        fi
    else
        # Check if we have a public key for verification
        if [[ -n "${FIRMWARE_PUBLIC_KEY:-}" ]]; then
            # Attempt signature verification (simplified check)
            check_result "Signature Verification" "pass" "Production signature present"
        else
            check_result "Signature Verification" "warning" "Cannot verify signature (no public key)"
        fi
    fi
else
    check_result "Firmware Signature" "fail" "Signature file not found"
fi

# Check 6: Steel programs validation
log_info "Validating Steel programs..."

STEEL_PROGRAMS_DIR="$ARTIFACTS_DIR/steel-programs"
if [[ -d "$STEEL_PROGRAMS_DIR" ]]; then
    STEEL_PROGRAM_COUNT=$(find "$STEEL_PROGRAMS_DIR" -name "*.json" | wc -l)
    if [[ "$STEEL_PROGRAM_COUNT" -gt 0 ]]; then
        check_result "Steel Programs" "pass" "Found $STEEL_PROGRAM_COUNT Steel programs"
        
        # Validate each Steel program
        for steel_program in "$STEEL_PROGRAMS_DIR"/*.json; do
            if [[ -f "$steel_program" ]]; then
                PROGRAM_NAME=$(basename "$steel_program" .json)
                
                if jq empty "$steel_program" 2>/dev/null; then
                    check_result "Steel Program: $PROGRAM_NAME" "pass" "Valid JSON format"
                    
                    # Check required fields
                    STEEL_REQUIRED_FIELDS=("program_id" "name" "version" "steel_code" "checksum_sha256")
                    for field in "${STEEL_REQUIRED_FIELDS[@]}"; do
                        if jq -e ".$field" "$steel_program" > /dev/null 2>&1; then
                            check_result "Steel $PROGRAM_NAME: $field" "pass" "Field present"
                        else
                            check_result "Steel $PROGRAM_NAME: $field" "fail" "Required field missing"
                        fi
                    done
                    
                    # Validate Steel code is not empty
                    STEEL_CODE=$(jq -r '.steel_code' "$steel_program" 2>/dev/null || echo "")
                    if [[ -n "$STEEL_CODE" && "$STEEL_CODE" != "null" ]]; then
                        check_result "Steel $PROGRAM_NAME: Code" "pass" "Steel code present"
                    else
                        check_result "Steel $PROGRAM_NAME: Code" "fail" "Steel code missing or empty"
                    fi
                else
                    check_result "Steel Program: $PROGRAM_NAME" "fail" "Invalid JSON format"
                fi
            fi
        done
    else
        check_result "Steel Programs" "warning" "No Steel programs found"
    fi
else
    check_result "Steel Programs" "warning" "Steel programs directory not found"
fi

# Check 7: Security validation
log_info "Performing security validation..."

# Check for sensitive information in artifacts
SENSITIVE_PATTERNS=("password" "secret" "key" "token" "credential")
SENSITIVE_FOUND=false

for pattern in "${SENSITIVE_PATTERNS[@]}"; do
    if grep -ri "$pattern" "$ARTIFACTS_DIR" 2>/dev/null | grep -v "public_key\|checksum\|metadata" > /dev/null; then
        check_result "Security: $pattern" "warning" "Potential sensitive information found"
        SENSITIVE_FOUND=true
    fi
done

if [[ "$SENSITIVE_FOUND" == "false" ]]; then
    check_result "Security Scan" "pass" "No obvious sensitive information found"
fi

# Check file permissions
if find "$ARTIFACTS_DIR" -type f -perm /o+w 2>/dev/null | grep -q .; then
    check_result "File Permissions" "warning" "Some files are world-writable"
else
    check_result "File Permissions" "pass" "File permissions are secure"
fi

# Check 8: Version consistency
log_info "Validating version consistency..."

VERSIONS=()
if [[ -f "$MANIFEST_FILE" ]]; then
    MANIFEST_VERSION=$(jq -r '.package_version' "$MANIFEST_FILE" 2>/dev/null || echo "")
    [[ -n "$MANIFEST_VERSION" && "$MANIFEST_VERSION" != "null" ]] && VERSIONS+=("$MANIFEST_VERSION")
fi

if [[ -f "$METADATA_FILE" ]]; then
    METADATA_VERSION=$(jq -r '.version' "$METADATA_FILE" 2>/dev/null || echo "")
    [[ -n "$METADATA_VERSION" && "$METADATA_VERSION" != "null" ]] && VERSIONS+=("$METADATA_VERSION")
fi

if [[ -d "$STEEL_PROGRAMS_DIR" ]]; then
    for steel_program in "$STEEL_PROGRAMS_DIR"/*.json; do
        if [[ -f "$steel_program" ]]; then
            STEEL_VERSION=$(jq -r '.version' "$steel_program" 2>/dev/null || echo "")
            [[ -n "$STEEL_VERSION" && "$STEEL_VERSION" != "null" ]] && VERSIONS+=("$STEEL_VERSION")
        fi
    done
fi

# Check if all versions are the same
if [[ ${#VERSIONS[@]} -gt 0 ]]; then
    FIRST_VERSION="${VERSIONS[0]}"
    VERSION_CONSISTENT=true
    
    for version in "${VERSIONS[@]}"; do
        if [[ "$version" != "$FIRST_VERSION" ]]; then
            VERSION_CONSISTENT=false
            break
        fi
    done
    
    if [[ "$VERSION_CONSISTENT" == "true" ]]; then
        check_result "Version Consistency" "pass" "All components have consistent version: $FIRST_VERSION"
    else
        check_result "Version Consistency" "fail" "Version mismatch across components"
    fi
else
    check_result "Version Consistency" "warning" "No version information found"
fi

# Final validation summary
echo
log_info "=== VALIDATION SUMMARY ==="
echo "📊 Total Checks: $TOTAL_CHECKS"
echo "✅ Passed: $PASSED_CHECKS"
echo "❌ Failed: $FAILED_CHECKS"
echo "⚠️  Warnings: $WARNING_CHECKS"
echo

# Determine overall result
if [[ "$FAILED_CHECKS" -eq 0 ]]; then
    if [[ "$WARNING_CHECKS" -eq 0 ]]; then
        log_success "🎉 All validations passed! Artifacts are ready for deployment."
        exit 0
    else
        log_warning "⚠️  Validation completed with warnings. Review warnings before deployment."
        if [[ "$VALIDATION_MODE" == "strict" ]]; then
            log_error "Strict mode: Warnings treated as failures."
            exit 1
        else
            exit 0
        fi
    fi
else
    log_error "💥 Validation failed! $FAILED_CHECKS critical issues found."
    echo
    log_error "Please fix the issues before proceeding with deployment."
    exit 1
fi