#!/bin/bash

# Provision Individual IoT Device
# Usage: ./provision-device.sh [device-id] [environment] [region]

set -e

# Parameters
DEVICE_ID=${1}
ENVIRONMENT=${2:-dev}
REGION=${3:-us-west-2}
PROJECT_NAME="esp32-c3-steel"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

if [ -z "${DEVICE_ID}" ]; then
    echo -e "${RED}Error: Device ID is required${NC}"
    echo "Usage: $0 <device-id> [environment] [region]"
    echo "Example: $0 device-001 dev us-west-2"
    exit 1
fi

THING_NAME="${PROJECT_NAME}-${ENVIRONMENT}-${DEVICE_ID}"
POLICY_NAME="${PROJECT_NAME}-${ENVIRONMENT}-device-policy"

echo -e "${GREEN}Provisioning IoT Device${NC}"
echo "Device ID: ${DEVICE_ID}"
echo "Thing Name: ${THING_NAME}"
echo "Environment: ${ENVIRONMENT}"
echo "Region: ${REGION}"
echo ""

# Check if AWS CLI is configured
if ! aws sts get-caller-identity > /dev/null 2>&1; then
    echo -e "${RED}Error: AWS CLI not configured or no valid credentials${NC}"
    exit 1
fi

# Get script directory for output
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CERT_DIR="${SCRIPT_DIR}/../certificates/${ENVIRONMENT}"
mkdir -p "${CERT_DIR}"

# Check if thing type exists
THING_TYPE_NAME="${PROJECT_NAME}-${ENVIRONMENT}-thing-type"
THING_TYPE_EXISTS=$(aws iot describe-thing-type \
    --thing-type-name "${THING_TYPE_NAME}" \
    --region "${REGION}" \
    --query 'thingTypeName' \
    --output text 2>/dev/null || echo "DOES_NOT_EXIST")

if [ "${THING_TYPE_EXISTS}" = "DOES_NOT_EXIST" ]; then
    echo -e "${RED}Error: Thing type ${THING_TYPE_NAME} does not exist${NC}"
    echo "Please deploy the core infrastructure first using deploy-core-infrastructure.sh"
    exit 1
fi

# Check if policy exists
POLICY_EXISTS=$(aws iot get-policy \
    --policy-name "${POLICY_NAME}" \
    --region "${REGION}" \
    --query 'policyName' \
    --output text 2>/dev/null || echo "DOES_NOT_EXIST")

if [ "${POLICY_EXISTS}" = "DOES_NOT_EXIST" ]; then
    echo -e "${RED}Error: Policy ${POLICY_NAME} does not exist${NC}"
    echo "Please deploy the core infrastructure first using deploy-core-infrastructure.sh"
    exit 1
fi

# Check if thing already exists
THING_EXISTS=$(aws iot describe-thing \
    --thing-name "${THING_NAME}" \
    --region "${REGION}" \
    --query 'thingName' \
    --output text 2>/dev/null || echo "DOES_NOT_EXIST")

if [ "${THING_EXISTS}" != "DOES_NOT_EXIST" ]; then
    echo -e "${YELLOW}Thing ${THING_NAME} already exists${NC}"
else
    # Create IoT Thing
    echo -e "${YELLOW}Creating IoT Thing: ${THING_NAME}${NC}"
    aws iot create-thing \
        --thing-name "${THING_NAME}" \
        --thing-type-name "${THING_TYPE_NAME}" \
        --attribute-payload attributes='{
            "firmware_version":"1.0.0",
            "steel_runtime_version":"0.5.0",
            "device_model":"esp32-c3-devkit-rust-1",
            "environment":"'${ENVIRONMENT}'",
            "device_id":"'${DEVICE_ID}'"
        }' \
        --region "${REGION}"
    
    echo -e "${GREEN}Thing created successfully${NC}"
fi

# Create certificate and keys
echo -e "${YELLOW}Creating certificate and keys...${NC}"
CERT_OUTPUT=$(aws iot create-keys-and-certificate \
    --set-as-active \
    --region "${REGION}" \
    --output json)

CERT_ARN=$(echo "${CERT_OUTPUT}" | jq -r '.certificateArn')
CERT_ID=$(echo "${CERT_OUTPUT}" | jq -r '.certificateId')
CERTIFICATE_PEM=$(echo "${CERT_OUTPUT}" | jq -r '.certificatePem')
PRIVATE_KEY=$(echo "${CERT_OUTPUT}" | jq -r '.keyPair.PrivateKey')
PUBLIC_KEY=$(echo "${CERT_OUTPUT}" | jq -r '.keyPair.PublicKey')

# Save certificate and keys to files
CERT_FILE="${CERT_DIR}/${DEVICE_ID}-certificate.pem"
PRIVATE_KEY_FILE="${CERT_DIR}/${DEVICE_ID}-private.key"
PUBLIC_KEY_FILE="${CERT_DIR}/${DEVICE_ID}-public.key"

echo "${CERTIFICATE_PEM}" > "${CERT_FILE}"
echo "${PRIVATE_KEY}" > "${PRIVATE_KEY_FILE}"
echo "${PUBLIC_KEY}" > "${PUBLIC_KEY_FILE}"

# Set appropriate permissions
chmod 600 "${PRIVATE_KEY_FILE}"
chmod 644 "${CERT_FILE}" "${PUBLIC_KEY_FILE}"

echo -e "${GREEN}Certificate and keys saved:${NC}"
echo "  Certificate: ${CERT_FILE}"
echo "  Private Key: ${PRIVATE_KEY_FILE}"
echo "  Public Key: ${PUBLIC_KEY_FILE}"

# Attach policy to certificate
echo -e "${YELLOW}Attaching policy to certificate...${NC}"
aws iot attach-policy \
    --policy-name "${POLICY_NAME}" \
    --target "${CERT_ARN}" \
    --region "${REGION}"

# Attach certificate to thing
echo -e "${YELLOW}Attaching certificate to thing...${NC}"
aws iot attach-thing-principal \
    --thing-name "${THING_NAME}" \
    --principal "${CERT_ARN}" \
    --region "${REGION}"

# Download Amazon Root CA certificate
ROOT_CA_FILE="${CERT_DIR}/AmazonRootCA1.pem"
if [ ! -f "${ROOT_CA_FILE}" ]; then
    echo -e "${YELLOW}Downloading Amazon Root CA certificate...${NC}"
    curl -s https://www.amazontrust.com/repository/AmazonRootCA1.pem > "${ROOT_CA_FILE}"
    echo -e "${GREEN}Root CA certificate saved: ${ROOT_CA_FILE}${NC}"
fi

# Create device configuration file
CONFIG_FILE="${CERT_DIR}/${DEVICE_ID}-config.json"
IOT_ENDPOINT=$(aws iot describe-endpoint \
    --endpoint-type iot:Data-ATS \
    --region "${REGION}" \
    --query 'endpointAddress' \
    --output text)

cat > "${CONFIG_FILE}" << EOF
{
  "device_id": "${DEVICE_ID}",
  "thing_name": "${THING_NAME}",
  "environment": "${ENVIRONMENT}",
  "aws_region": "${REGION}",
  "iot_endpoint": "${IOT_ENDPOINT}",
  "certificate_file": "${DEVICE_ID}-certificate.pem",
  "private_key_file": "${DEVICE_ID}-private.key",
  "root_ca_file": "AmazonRootCA1.pem",
  "certificate_arn": "${CERT_ARN}",
  "certificate_id": "${CERT_ID}",
  "provisioned_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo -e "${GREEN}Device configuration saved: ${CONFIG_FILE}${NC}"

# Create summary
echo -e "\n${GREEN}Device Provisioning Complete!${NC}"
echo -e "${GREEN}================================${NC}"
echo "Device ID: ${DEVICE_ID}"
echo "Thing Name: ${THING_NAME}"
echo "Certificate ARN: ${CERT_ARN}"
echo "IoT Endpoint: ${IOT_ENDPOINT}"
echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo "1. Copy the certificate files to your device"
echo "2. Update your device configuration with the IoT endpoint"
echo "3. Test the connection using the provided certificates"
echo ""
echo -e "${YELLOW}Files created:${NC}"
echo "  ${CERT_FILE}"
echo "  ${PRIVATE_KEY_FILE}"
echo "  ${PUBLIC_KEY_FILE}"
echo "  ${ROOT_CA_FILE}"
echo "  ${CONFIG_FILE}"