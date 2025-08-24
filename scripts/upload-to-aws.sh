#!/bin/bash
# AWS S3 upload script for AWS IoT Steel project
# This script securely uploads signed artifacts to S3 and triggers CodePipeline

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARTIFACTS_DIR="${1:-./signed-artifacts}"
AWS_REGION="${AWS_REGION:-us-east-1}"
DEPLOYMENT_STAGE="${DEPLOYMENT_STAGE:-development}"

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

# Validate prerequisites
if ! command -v aws &> /dev/null; then
    log_error "AWS CLI not found. Please install AWS CLI."
    exit 1
fi

if ! command -v jq &> /dev/null; then
    log_error "jq not found. Please install jq for JSON processing."
    exit 1
fi

# Validate inputs
if [[ ! -d "$ARTIFACTS_DIR" ]]; then
    log_error "Artifacts directory not found: $ARTIFACTS_DIR"
    exit 1
fi

# Check AWS credentials
if ! aws sts get-caller-identity &> /dev/null; then
    log_error "AWS credentials not configured or invalid"
    exit 1
fi

log_info "Starting AWS upload process..."
log_info "Artifacts directory: $ARTIFACTS_DIR"
log_info "AWS Region: $AWS_REGION"
log_info "Deployment stage: $DEPLOYMENT_STAGE"

# Find package manifest
MANIFEST_FILE="$ARTIFACTS_DIR/package-manifest.json"
if [[ ! -f "$MANIFEST_FILE" ]]; then
    log_error "Package manifest not found: $MANIFEST_FILE"
    exit 1
fi

# Extract package information
PACKAGE_NAME=$(jq -r '.package_name' "$MANIFEST_FILE")
PACKAGE_VERSION=$(jq -r '.package_version' "$MANIFEST_FILE")
PACKAGE_CHECKSUM=$(jq -r '.package_checksum' "$MANIFEST_FILE")

if [[ "$PACKAGE_NAME" == "null" || "$PACKAGE_VERSION" == "null" ]]; then
    log_error "Invalid package manifest"
    exit 1
fi

PACKAGE_PATH="$ARTIFACTS_DIR/$PACKAGE_NAME"
if [[ ! -f "$PACKAGE_PATH" ]]; then
    log_error "Package file not found: $PACKAGE_PATH"
    exit 1
fi

log_info "Package: $PACKAGE_NAME"
log_info "Version: $PACKAGE_VERSION"
log_info "Checksum: $PACKAGE_CHECKSUM"

# Determine S3 bucket names based on deployment stage
case "$DEPLOYMENT_STAGE" in
    "production")
        S3_ARTIFACTS_BUCKET="${S3_ARTIFACTS_BUCKET:-aws-iot-steel-artifacts-prod}"
        S3_FIRMWARE_BUCKET="${S3_FIRMWARE_BUCKET:-aws-iot-steel-firmware-prod}"
        ;;
    "staging")
        S3_ARTIFACTS_BUCKET="${S3_ARTIFACTS_BUCKET:-aws-iot-steel-artifacts-staging}"
        S3_FIRMWARE_BUCKET="${S3_FIRMWARE_BUCKET:-aws-iot-steel-firmware-staging}"
        ;;
    *)
        S3_ARTIFACTS_BUCKET="${S3_ARTIFACTS_BUCKET:-aws-iot-steel-artifacts-dev}"
        S3_FIRMWARE_BUCKET="${S3_FIRMWARE_BUCKET:-aws-iot-steel-firmware-dev}"
        ;;
esac

log_info "Target S3 buckets:"
log_info "  Artifacts: $S3_ARTIFACTS_BUCKET"
log_info "  Firmware: $S3_FIRMWARE_BUCKET"

# Verify S3 buckets exist and are accessible
for bucket in "$S3_ARTIFACTS_BUCKET" "$S3_FIRMWARE_BUCKET"; do
    if ! aws s3 ls "s3://$bucket" &> /dev/null; then
        log_error "Cannot access S3 bucket: $bucket"
        log_error "Please ensure the bucket exists and you have proper permissions"
        exit 1
    fi
    log_success "S3 bucket accessible: $bucket"
done

# Create timestamp for this deployment
TIMESTAMP=$(date -u +%Y%m%d-%H%M%S)
DEPLOYMENT_ID="deploy-$PACKAGE_VERSION-$TIMESTAMP"

log_info "Deployment ID: $DEPLOYMENT_ID"

# Upload main artifact package
log_info "Uploading artifact package to S3..."

S3_ARTIFACTS_KEY="builds/$PACKAGE_VERSION/$PACKAGE_NAME"
aws s3 cp "$PACKAGE_PATH" "s3://$S3_ARTIFACTS_BUCKET/$S3_ARTIFACTS_KEY" \
    --metadata "version=$PACKAGE_VERSION,checksum=$PACKAGE_CHECKSUM,deployment-id=$DEPLOYMENT_ID" \
    --storage-class STANDARD_IA

log_success "Artifact package uploaded: s3://$S3_ARTIFACTS_BUCKET/$S3_ARTIFACTS_KEY"

# Upload package manifest
log_info "Uploading package manifest..."

S3_MANIFEST_KEY="builds/$PACKAGE_VERSION/package-manifest.json"
aws s3 cp "$MANIFEST_FILE" "s3://$S3_ARTIFACTS_BUCKET/$S3_MANIFEST_KEY" \
    --content-type "application/json" \
    --metadata "version=$PACKAGE_VERSION,deployment-id=$DEPLOYMENT_ID"

log_success "Package manifest uploaded: s3://$S3_ARTIFACTS_BUCKET/$S3_MANIFEST_KEY"

# Extract and upload individual firmware components to firmware bucket
log_info "Extracting and uploading firmware components..."

TEMP_EXTRACT_DIR=$(mktemp -d)
trap "rm -rf $TEMP_EXTRACT_DIR" EXIT

cd "$TEMP_EXTRACT_DIR"
tar -xzf "$PACKAGE_PATH"

# Upload firmware binary
if [[ -f "aws-iot-platform-esp32.bin" ]]; then
    FIRMWARE_KEY="firmware/$PACKAGE_VERSION/aws-iot-platform-esp32.bin"
    aws s3 cp "aws-iot-platform-esp32.bin" "s3://$S3_FIRMWARE_BUCKET/$FIRMWARE_KEY" \
        --metadata "version=$PACKAGE_VERSION,deployment-id=$DEPLOYMENT_ID,type=firmware-binary"
    log_success "Firmware binary uploaded: s3://$S3_FIRMWARE_BUCKET/$FIRMWARE_KEY"
fi

# Upload firmware signature
if [[ -f "aws-iot-platform-esp32.bin.sig" ]]; then
    SIGNATURE_KEY="firmware/$PACKAGE_VERSION/aws-iot-platform-esp32.bin.sig"
    aws s3 cp "aws-iot-platform-esp32.bin.sig" "s3://$S3_FIRMWARE_BUCKET/$SIGNATURE_KEY" \
        --metadata "version=$PACKAGE_VERSION,deployment-id=$DEPLOYMENT_ID,type=firmware-signature"
    log_success "Firmware signature uploaded: s3://$S3_FIRMWARE_BUCKET/$SIGNATURE_KEY"
fi

# Upload firmware metadata
if [[ -f "firmware-metadata.json" ]]; then
    METADATA_KEY="firmware/$PACKAGE_VERSION/firmware-metadata.json"
    aws s3 cp "firmware-metadata.json" "s3://$S3_FIRMWARE_BUCKET/$METADATA_KEY" \
        --content-type "application/json" \
        --metadata "version=$PACKAGE_VERSION,deployment-id=$DEPLOYMENT_ID,type=firmware-metadata"
    log_success "Firmware metadata uploaded: s3://$S3_FIRMWARE_BUCKET/$METADATA_KEY"
fi

# Upload Steel programs
if [[ -d "steel-programs" ]]; then
    log_info "Uploading Steel programs..."
    
    for steel_program in steel-programs/*.json; do
        if [[ -f "$steel_program" ]]; then
            PROGRAM_NAME=$(basename "$steel_program")
            PROGRAM_KEY="steel-programs/$PACKAGE_VERSION/$PROGRAM_NAME"
            
            aws s3 cp "$steel_program" "s3://$S3_FIRMWARE_BUCKET/$PROGRAM_KEY" \
                --content-type "application/json" \
                --metadata "version=$PACKAGE_VERSION,deployment-id=$DEPLOYMENT_ID,type=steel-program"
            
            log_success "Steel program uploaded: s3://$S3_FIRMWARE_BUCKET/$PROGRAM_KEY"
        fi
    done
fi

cd - > /dev/null

# Create deployment trigger for CodePipeline
log_info "Creating CodePipeline trigger..."

TRIGGER_PAYLOAD=$(cat << EOF
{
  "deployment_id": "$DEPLOYMENT_ID",
  "package_version": "$PACKAGE_VERSION",
  "package_checksum": "$PACKAGE_CHECKSUM",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "stage": "$DEPLOYMENT_STAGE",
  "artifacts": {
    "package_s3_key": "$S3_ARTIFACTS_KEY",
    "manifest_s3_key": "$S3_MANIFEST_KEY",
    "firmware_s3_key": "firmware/$PACKAGE_VERSION/aws-iot-platform-esp32.bin"
  },
  "git_info": {
    "commit": "${GITHUB_SHA:-$(git rev-parse HEAD 2>/dev/null || echo 'unknown')}",
    "branch": "${GITHUB_REF_NAME:-$(git branch --show-current 2>/dev/null || echo 'unknown')}",
    "repository": "${GITHUB_REPOSITORY:-unknown}"
  },
  "build_info": {
    "build_number": "${GITHUB_RUN_NUMBER:-0}",
    "build_id": "${GITHUB_RUN_ID:-unknown}",
    "workflow": "${GITHUB_WORKFLOW:-manual}"
  }
}
EOF
)

TRIGGER_KEY="triggers/$DEPLOYMENT_STAGE/deployment-$DEPLOYMENT_ID.json"
echo "$TRIGGER_PAYLOAD" | aws s3 cp - "s3://$S3_ARTIFACTS_BUCKET/$TRIGGER_KEY" \
    --content-type "application/json" \
    --metadata "deployment-id=$DEPLOYMENT_ID,stage=$DEPLOYMENT_STAGE,version=$PACKAGE_VERSION"

log_success "CodePipeline trigger created: s3://$S3_ARTIFACTS_BUCKET/$TRIGGER_KEY"

# Create latest version pointer for easy access
log_info "Updating latest version pointer..."

LATEST_POINTER=$(cat << EOF
{
  "latest_version": "$PACKAGE_VERSION",
  "deployment_id": "$DEPLOYMENT_ID",
  "updated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "stage": "$DEPLOYMENT_STAGE",
  "artifacts_s3_key": "$S3_ARTIFACTS_KEY",
  "firmware_s3_key": "firmware/$PACKAGE_VERSION/aws-iot-platform-esp32.bin"
}
EOF
)

LATEST_KEY="latest/$DEPLOYMENT_STAGE/version.json"
echo "$LATEST_POINTER" | aws s3 cp - "s3://$S3_ARTIFACTS_BUCKET/$LATEST_KEY" \
    --content-type "application/json" \
    --metadata "version=$PACKAGE_VERSION,deployment-id=$DEPLOYMENT_ID,stage=$DEPLOYMENT_STAGE"

log_success "Latest version pointer updated: s3://$S3_ARTIFACTS_BUCKET/$LATEST_KEY"

# Generate pre-signed URLs for verification (optional)
if [[ "${GENERATE_PRESIGNED_URLS:-false}" == "true" ]]; then
    log_info "Generating pre-signed URLs for verification..."
    
    PRESIGNED_PACKAGE_URL=$(aws s3 presign "s3://$S3_ARTIFACTS_BUCKET/$S3_ARTIFACTS_KEY" --expires-in 3600)
    PRESIGNED_MANIFEST_URL=$(aws s3 presign "s3://$S3_ARTIFACTS_BUCKET/$S3_MANIFEST_KEY" --expires-in 3600)
    
    echo "Pre-signed URLs (valid for 1 hour):"
    echo "Package: $PRESIGNED_PACKAGE_URL"
    echo "Manifest: $PRESIGNED_MANIFEST_URL"
fi

# Verify uploads
log_info "Verifying uploads..."

# Check package upload
if aws s3 ls "s3://$S3_ARTIFACTS_BUCKET/$S3_ARTIFACTS_KEY" &> /dev/null; then
    log_success "Package upload verified"
else
    log_error "Package upload verification failed"
    exit 1
fi

# Check trigger file
if aws s3 ls "s3://$S3_ARTIFACTS_BUCKET/$TRIGGER_KEY" &> /dev/null; then
    log_success "Trigger file verified"
else
    log_error "Trigger file verification failed"
    exit 1
fi

# Summary
echo
log_success "=== AWS UPLOAD COMPLETE ==="
echo "🚀 Deployment ID: $DEPLOYMENT_ID"
echo "📦 Package Version: $PACKAGE_VERSION"
echo "🌍 AWS Region: $AWS_REGION"
echo "🎯 Deployment Stage: $DEPLOYMENT_STAGE"
echo "📁 Artifacts Bucket: $S3_ARTIFACTS_BUCKET"
echo "💾 Firmware Bucket: $S3_FIRMWARE_BUCKET"
echo "🔗 Trigger Key: $TRIGGER_KEY"
echo

log_info "CodePipeline should be triggered automatically"
log_info "Monitor deployment progress in AWS Console"

# Optional: Trigger CodePipeline directly if pipeline name is provided
if [[ -n "${CODEPIPELINE_NAME:-}" ]]; then
    log_info "Triggering CodePipeline directly: $CODEPIPELINE_NAME"
    
    aws codepipeline start-pipeline-execution \
        --name "$CODEPIPELINE_NAME" \
        --region "$AWS_REGION" || log_warning "Failed to trigger CodePipeline directly"
fi

log_success "Upload process completed successfully!"