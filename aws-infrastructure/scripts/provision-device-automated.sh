#!/bin/bash

# Automated Device Provisioning Script for CodeBuild
# This script provisions devices automatically during deployment
# Usage: ./provision-device-automated.sh [environment] [region] [device-count]

set -e

# Default values
ENVIRONMENT=${1:-dev}
REGION=${2:-us-west-2}
DEVICE_COUNT=${3:-1}
PROJECT_NAME="esp32-steel"

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

log_info "Starting automated device provisioning..."
log_info "Environment: ${ENVIRONMENT}"
log_info "Region: ${REGION}"
log_info "Device Count: ${DEVICE_COUNT}"

# Check if AWS CLI is configured
if ! aws sts get-caller-identity > /dev/null 2>&1; then
    log_error "AWS CLI not configured or no valid credentials"
    exit 1
fi

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if core infrastructure exists
CORE_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-core-iot"
STACK_EXISTS=$(aws cloudformation describe-stacks \
    --stack-name "${CORE_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].StackStatus' \
    --output text 2>/dev/null || echo "DOES_NOT_EXIST")

if [ "${STACK_EXISTS}" = "DOES_NOT_EXIST" ]; then
    log_error "Core IoT infrastructure stack does not exist: ${CORE_STACK_NAME}"
    exit 1
fi

# Get stack outputs
log_info "Retrieving infrastructure information..."

THING_TYPE_NAME=$(aws cloudformation describe-stacks \
    --stack-name "${CORE_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`ThingTypeName`].OutputValue' \
    --output text)

DEVICE_POLICY_NAME=$(aws cloudformation describe-stacks \
    --stack-name "${CORE_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`DevicePolicyName`].OutputValue' \
    --output text)

IOT_ENDPOINT=$(aws cloudformation describe-stacks \
    --stack-name "${CORE_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`IoTEndpoint`].OutputValue' \
    --output text)

if [ -z "$THING_TYPE_NAME" ] || [ -z "$DEVICE_POLICY_NAME" ] || [ -z "$IOT_ENDPOINT" ]; then
    log_error "Could not retrieve required infrastructure information"
    exit 1
fi

log_success "Infrastructure information retrieved:"
log_info "  Thing Type: ${THING_TYPE_NAME}"
log_info "  Device Policy: ${DEVICE_POLICY_NAME}"
log_info "  IoT Endpoint: ${IOT_ENDPOINT}"

# Create certificates directory
CERT_DIR="${SCRIPT_DIR}/../certificates/${ENVIRONMENT}"
mkdir -p "${CERT_DIR}"

# Download Amazon Root CA if not exists
if [ ! -f "${CERT_DIR}/AmazonRootCA1.pem" ]; then
    log_info "Downloading Amazon Root CA certificate..."
    curl -s https://www.amazontrust.com/repository/AmazonRootCA1.pem -o "${CERT_DIR}/AmazonRootCA1.pem"
    log_success "Amazon Root CA certificate downloaded"
fi

# Function to provision a single device
provision_device() {
    local device_number=$1
    local device_id="${PROJECT_NAME}-${ENVIRONMENT}-auto-$(printf "%03d" $device_number)"
    
    log_info "Provisioning device: ${device_id}"
    
    # Check if device already exists
    EXISTING_THING=$(aws iot describe-thing \
        --thing-name "${device_id}" \
        --region "${REGION}" \
        --query 'thingName' \
        --output text 2>/dev/null || echo "")
    
    if [ -n "$EXISTING_THING" ]; then
        log_warning "Device already exists: ${device_id}"
        return 0
    fi
    
    # Create certificate and keys
    log_info "Creating certificate for device: ${device_id}"
    
    CERT_RESPONSE=$(aws iot create-keys-and-certificate \
        --set-as-active \
        --region "${REGION}" \
        --output json)
    
    if [ $? -ne 0 ]; then
        log_error "Failed to create certificate for device: ${device_id}"
        return 1
    fi
    
    # Extract certificate information
    CERTIFICATE_ARN=$(echo "$CERT_RESPONSE" | jq -r '.certificateArn')
    CERTIFICATE_ID=$(echo "$CERT_RESPONSE" | jq -r '.certificateId')
    CERTIFICATE_PEM=$(echo "$CERT_RESPONSE" | jq -r '.certificatePem')
    PRIVATE_KEY=$(echo "$CERT_RESPONSE" | jq -r '.keyPair.PrivateKey')
    PUBLIC_KEY=$(echo "$CERT_RESPONSE" | jq -r '.keyPair.PublicKey')
    
    # Save certificate files
    echo "$CERTIFICATE_PEM" > "${CERT_DIR}/${device_id}-certificate.pem"
    echo "$PRIVATE_KEY" > "${CERT_DIR}/${device_id}-private.key"
    echo "$PUBLIC_KEY" > "${CERT_DIR}/${device_id}-public.key"
    
    # Set proper permissions for private key
    chmod 600 "${CERT_DIR}/${device_id}-private.key"
    
    log_success "Certificate created and saved for device: ${device_id}"
    
    # Create IoT Thing
    log_info "Creating IoT Thing: ${device_id}"
    
    aws iot create-thing \
        --thing-name "${device_id}" \
        --thing-type-name "${THING_TYPE_NAME}" \
        --attribute-payload "attributes={firmware_version=1.0.0,steel_runtime_version=0.5.0,device_model=esp32-c3-devkit-rust-1,environment=${ENVIRONMENT},provisioned_by=codebuild,provisioned_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)}" \
        --region "${REGION}" > /dev/null
    
    if [ $? -eq 0 ]; then
        log_success "IoT Thing created: ${device_id}"
    else
        log_error "Failed to create IoT Thing: ${device_id}"
        return 1
    fi
    
    # Attach policy to certificate
    log_info "Attaching policy to certificate for device: ${device_id}"
    
    aws iot attach-policy \
        --policy-name "${DEVICE_POLICY_NAME}" \
        --target "${CERTIFICATE_ARN}" \
        --region "${REGION}"
    
    if [ $? -eq 0 ]; then
        log_success "Policy attached to certificate for device: ${device_id}"
    else
        log_error "Failed to attach policy to certificate for device: ${device_id}"
        return 1
    fi
    
    # Attach certificate to thing
    log_info "Attaching certificate to thing for device: ${device_id}"
    
    aws iot attach-thing-principal \
        --thing-name "${device_id}" \
        --principal "${CERTIFICATE_ARN}" \
        --region "${REGION}"
    
    if [ $? -eq 0 ]; then
        log_success "Certificate attached to thing for device: ${device_id}"
    else
        log_error "Failed to attach certificate to thing for device: ${device_id}"
        return 1
    fi
    
    # Create device configuration file
    log_info "Creating configuration file for device: ${device_id}"
    
    DEVICE_CONFIG=$(cat << EOF
{
  "device_id": "${device_id}",
  "thing_name": "${device_id}",
  "thing_type": "${THING_TYPE_NAME}",
  "certificate_id": "${CERTIFICATE_ID}",
  "certificate_arn": "${CERTIFICATE_ARN}",
  "iot_endpoint": "${IOT_ENDPOINT}",
  "aws_region": "${REGION}",
  "environment": "${ENVIRONMENT}",
  "project_name": "${PROJECT_NAME}",
  "provisioned_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "provisioned_by": "codebuild-automation",
  "mqtt_topics": {
    "steel_programs_load": "steel-programs/${device_id}/load",
    "steel_programs_execute": "steel-programs/${device_id}/execute",
    "steel_programs_status": "steel-programs/${device_id}/status",
    "downloads_request": "downloads/${device_id}/request",
    "downloads_firmware_url": "downloads/${device_id}/firmware-url",
    "downloads_program_url": "downloads/${device_id}/program-url",
    "shadow_update": "\$aws/things/${device_id}/shadow/update",
    "shadow_update_delta": "\$aws/things/${device_id}/shadow/update/delta"
  },
  "files": {
    "certificate": "${device_id}-certificate.pem",
    "private_key": "${device_id}-private.key",
    "public_key": "${device_id}-public.key",
    "root_ca": "AmazonRootCA1.pem"
  }
}
EOF
    )
    
    echo "$DEVICE_CONFIG" > "${CERT_DIR}/${device_id}-config.json"
    log_success "Configuration file created for device: ${device_id}"
    
    # Initialize device shadow
    log_info "Initializing device shadow for device: ${device_id}"
    
    INITIAL_SHADOW=$(cat << EOF
{
  "state": {
    "desired": {
      "welcome": true,
      "firmware_version": "1.0.0",
      "steel_runtime_version": "0.5.0"
    },
    "reported": {
      "device_info": {
        "device_id": "${device_id}",
        "provisioned_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "environment": "${ENVIRONMENT}",
        "status": "provisioned"
      }
    }
  }
}
EOF
    )
    
    aws iot-data update-thing-shadow \
        --thing-name "${device_id}" \
        --payload "$INITIAL_SHADOW" \
        --region "${REGION}" \
        /tmp/shadow-response-${device_id}.json > /dev/null 2>&1 || log_warning "Failed to initialize shadow for device: ${device_id}"
    
    log_success "Device provisioning completed: ${device_id}"
    
    return 0
}

# Provision devices
log_info "Starting device provisioning for ${DEVICE_COUNT} devices..."

PROVISIONED_COUNT=0
FAILED_COUNT=0

for i in $(seq 1 $DEVICE_COUNT); do
    if provision_device $i; then
        PROVISIONED_COUNT=$((PROVISIONED_COUNT + 1))
    else
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
done

# Generate provisioning report
PROVISIONING_REPORT=$(cat << EOF
{
  "provisioning_id": "auto-$(date +%Y%m%d-%H%M%S)",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "environment": "${ENVIRONMENT}",
  "region": "${REGION}",
  "requested_devices": ${DEVICE_COUNT},
  "provisioned_devices": ${PROVISIONED_COUNT},
  "failed_devices": ${FAILED_COUNT},
  "success_rate": $(echo "scale=2; ${PROVISIONED_COUNT} * 100 / ${DEVICE_COUNT}" | bc -l)%,
  "certificate_directory": "${CERT_DIR}",
  "thing_type": "${THING_TYPE_NAME}",
  "device_policy": "${DEVICE_POLICY_NAME}",
  "iot_endpoint": "${IOT_ENDPOINT}"
}
EOF
)

REPORT_FILE="${CERT_DIR}/provisioning-report-$(date +%Y%m%d-%H%M%S).json"
echo "$PROVISIONING_REPORT" > "$REPORT_FILE"

# Summary
echo
log_success "=== DEVICE PROVISIONING COMPLETE ==="
echo "📊 Requested Devices: ${DEVICE_COUNT}"
echo "✅ Provisioned Successfully: ${PROVISIONED_COUNT}"
echo "❌ Failed: ${FAILED_COUNT}"
echo "📁 Certificates Directory: ${CERT_DIR}"
echo "📄 Report File: ${REPORT_FILE}"
echo

if [ $FAILED_COUNT -eq 0 ]; then
    log_success "All devices provisioned successfully!"
    exit 0
else
    log_warning "Some devices failed to provision. Check the logs above for details."
    exit 1
fi