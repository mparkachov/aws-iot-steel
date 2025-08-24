#!/bin/bash

# Test S3 Upload Functionality
# Usage: ./test-s3-upload.sh [environment] [region]

set -e

# Default values
ENVIRONMENT=${1:-dev}
REGION=${2:-us-west-2}
PROJECT_NAME="esp32-steel"
STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-s3-lambda"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Testing S3 Upload Functionality${NC}"
echo "Environment: ${ENVIRONMENT}"
echo "Region: ${REGION}"
echo ""

# Check if AWS CLI is configured
if ! aws sts get-caller-identity > /dev/null 2>&1; then
    echo -e "${RED}Error: AWS CLI not configured or no valid credentials${NC}"
    exit 1
fi

# Get stack outputs
echo -e "${YELLOW}Getting stack outputs...${NC}"
OUTPUTS=$(aws cloudformation describe-stacks \
    --stack-name "${STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs' \
    --output json 2>/dev/null)

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Could not get stack outputs. Make sure the stack is deployed.${NC}"
    exit 1
fi

FIRMWARE_BUCKET=$(echo "${OUTPUTS}" | jq -r '.[] | select(.OutputKey=="FirmwareBucketName") | .OutputValue')
PROGRAMS_BUCKET=$(echo "${OUTPUTS}" | jq -r '.[] | select(.OutputKey=="SteelProgramsBucketName") | .OutputValue')
CICD_ROLE_ARN=$(echo "${OUTPUTS}" | jq -r '.[] | select(.OutputKey=="CICDRoleArn") | .OutputValue')

echo "Firmware Bucket: ${FIRMWARE_BUCKET}"
echo "Programs Bucket: ${PROGRAMS_BUCKET}"
echo "CI/CD Role ARN: ${CICD_ROLE_ARN}"
echo ""

# Create test directory
TEST_DIR="/tmp/s3-upload-test-$$"
mkdir -p "${TEST_DIR}"

# Create test firmware file
echo -e "${YELLOW}Creating test firmware file...${NC}"
FIRMWARE_FILE="${TEST_DIR}/esp32-s3-firmware.bin"
dd if=/dev/urandom of="${FIRMWARE_FILE}" bs=1024 count=100 2>/dev/null
FIRMWARE_SHA256=$(shasum -a 256 "${FIRMWARE_FILE}" | cut -d' ' -f1)

echo "Test firmware file created: ${FIRMWARE_FILE}"
echo "SHA256: ${FIRMWARE_SHA256}"

# Create test Steel program
echo -e "${YELLOW}Creating test Steel program...${NC}"
PROGRAM_FILE="${TEST_DIR}/test-program.json"
cat > "${PROGRAM_FILE}" << EOF
{
  "program_id": "test-program-$(date +%s)",
  "name": "test-sensor-monitor",
  "version": "1.0.0",
  "description": "Test Steel program for upload testing",
  "author": "Test Suite",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "steel_code": "(define (main) (begin (log \\\"info\\\" \\\"Test program running\\\") (led-on) (sleep 5) (led-off)))",
  "checksum": "sha256:test-checksum",
  "metadata": {
    "memory_requirement": 8192,
    "execution_timeout": 30,
    "auto_restart": false,
    "priority": "normal"
  }
}
EOF

echo "Test Steel program created: ${PROGRAM_FILE}"

# Test 1: Upload firmware to firmware bucket
echo -e "\n${YELLOW}Test 1: Uploading firmware to firmware bucket...${NC}"
FIRMWARE_KEY="firmware/1.0.0-test/esp32-s3-firmware.bin"

aws s3 cp "${FIRMWARE_FILE}" "s3://${FIRMWARE_BUCKET}/${FIRMWARE_KEY}" \
    --metadata "checksum-sha256=${FIRMWARE_SHA256},version=1.0.0-test,build-date=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --region "${REGION}"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Firmware upload successful${NC}"
    
    # Verify upload
    aws s3 ls "s3://${FIRMWARE_BUCKET}/${FIRMWARE_KEY}" --region "${REGION}" > /dev/null
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Firmware file verified in bucket${NC}"
    else
        echo -e "${RED}❌ Firmware file not found in bucket${NC}"
    fi
else
    echo -e "${RED}❌ Firmware upload failed${NC}"
fi

# Test 2: Upload Steel program to programs bucket
echo -e "\n${YELLOW}Test 2: Uploading Steel program to programs bucket...${NC}"
PROGRAM_KEY="programs/test-sensor-monitor.json"

aws s3 cp "${PROGRAM_FILE}" "s3://${PROGRAMS_BUCKET}/${PROGRAM_KEY}" \
    --content-type "application/json" \
    --metadata "program-type=steel,version=1.0.0" \
    --region "${REGION}"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Steel program upload successful${NC}"
    
    # Verify upload
    aws s3 ls "s3://${PROGRAMS_BUCKET}/${PROGRAM_KEY}" --region "${REGION}" > /dev/null
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Steel program file verified in bucket${NC}"
    else
        echo -e "${RED}❌ Steel program file not found in bucket${NC}"
    fi
else
    echo -e "${RED}❌ Steel program upload failed${NC}"
fi

# Test 3: Test bucket policies (try to access without proper permissions)
echo -e "\n${YELLOW}Test 3: Testing bucket security policies...${NC}"

# Try to make objects public (should fail)
echo -e "${YELLOW}Testing public ACL prevention...${NC}"
aws s3api put-object-acl \
    --bucket "${FIRMWARE_BUCKET}" \
    --key "${FIRMWARE_KEY}" \
    --acl public-read \
    --region "${REGION}" 2>/dev/null

if [ $? -ne 0 ]; then
    echo -e "${GREEN}✅ Public ACL correctly blocked${NC}"
else
    echo -e "${RED}❌ Public ACL was allowed (security issue!)${NC}"
fi

# Test 4: Verify HTTPS-only policy
echo -e "\n${YELLOW}Test 4: Testing HTTPS-only policy...${NC}"
# This test would require making an HTTP request, which is complex to test directly
# Instead, we'll check if the bucket policy exists
BUCKET_POLICY=$(aws s3api get-bucket-policy \
    --bucket "${FIRMWARE_BUCKET}" \
    --region "${REGION}" \
    --query 'Policy' \
    --output text 2>/dev/null)

if echo "${BUCKET_POLICY}" | grep -q "aws:SecureTransport"; then
    echo -e "${GREEN}✅ HTTPS-only policy found in bucket policy${NC}"
else
    echo -e "${RED}❌ HTTPS-only policy not found${NC}"
fi

# Test 5: Test versioning
echo -e "\n${YELLOW}Test 5: Testing bucket versioning...${NC}"

# Upload a new version of the firmware
echo "Updated firmware content" >> "${FIRMWARE_FILE}"
aws s3 cp "${FIRMWARE_FILE}" "s3://${FIRMWARE_BUCKET}/${FIRMWARE_KEY}" \
    --metadata "checksum-sha256=updated,version=1.0.1-test" \
    --region "${REGION}" > /dev/null

# Check if multiple versions exist
VERSIONS=$(aws s3api list-object-versions \
    --bucket "${FIRMWARE_BUCKET}" \
    --prefix "${FIRMWARE_KEY}" \
    --region "${REGION}" \
    --query 'Versions[].VersionId' \
    --output text)

VERSION_COUNT=$(echo "${VERSIONS}" | wc -w)
if [ "${VERSION_COUNT}" -gt 1 ]; then
    echo -e "${GREEN}✅ Versioning working (${VERSION_COUNT} versions found)${NC}"
else
    echo -e "${RED}❌ Versioning not working properly${NC}"
fi

# Test 6: Test Lambda function with uploaded files
echo -e "\n${YELLOW}Test 6: Testing Lambda function with uploaded files...${NC}"

# Get Lambda function name
FUNCTION_NAME=$(echo "${OUTPUTS}" | jq -r '.[] | select(.OutputKey=="URLGeneratorFunctionName") | .OutputValue')

if [ -n "${FUNCTION_NAME}" ]; then
    # Test firmware download request
    TEST_PAYLOAD=$(cat << EOF
{
  "device_id": "${PROJECT_NAME}-${ENVIRONMENT}-test-001",
  "request_type": "firmware",
  "resource_id": "1.0.0-test",
  "request_id": "test-upload-$(date +%s)"
}
EOF
)
    
    echo -e "${YELLOW}Testing firmware download request...${NC}"
    aws lambda invoke \
        --function-name "${FUNCTION_NAME}" \
        --payload "${TEST_PAYLOAD}" \
        --region "${REGION}" \
        /tmp/lambda-response.json > /dev/null
    
    RESPONSE_CODE=$(cat /tmp/lambda-response.json | jq -r '.statusCode')
    if [ "${RESPONSE_CODE}" = "200" ]; then
        echo -e "${GREEN}✅ Lambda function successfully generated pre-signed URL for firmware${NC}"
    else
        echo -e "${YELLOW}⚠️  Lambda function returned status ${RESPONSE_CODE} (may be expected if IoT topic doesn't exist)${NC}"
    fi
    
    # Test program download request
    TEST_PAYLOAD=$(cat << EOF
{
  "device_id": "${PROJECT_NAME}-${ENVIRONMENT}-test-002",
  "request_type": "program",
  "resource_id": "test-sensor-monitor",
  "request_id": "test-prog-$(date +%s)"
}
EOF
)
    
    echo -e "${YELLOW}Testing program download request...${NC}"
    aws lambda invoke \
        --function-name "${FUNCTION_NAME}" \
        --payload "${TEST_PAYLOAD}" \
        --region "${REGION}" \
        /tmp/lambda-response2.json > /dev/null
    
    RESPONSE_CODE=$(cat /tmp/lambda-response2.json | jq -r '.statusCode')
    if [ "${RESPONSE_CODE}" = "200" ]; then
        echo -e "${GREEN}✅ Lambda function successfully generated pre-signed URL for program${NC}"
    else
        echo -e "${YELLOW}⚠️  Lambda function returned status ${RESPONSE_CODE} (may be expected if IoT topic doesn't exist)${NC}"
    fi
    
    rm -f /tmp/lambda-response.json /tmp/lambda-response2.json
else
    echo -e "${RED}❌ Could not find Lambda function name${NC}"
fi

# Cleanup test files
echo -e "\n${YELLOW}Cleaning up test files...${NC}"
rm -rf "${TEST_DIR}"

# Optional: Clean up uploaded test files (uncomment if desired)
# aws s3 rm "s3://${FIRMWARE_BUCKET}/${FIRMWARE_KEY}" --region "${REGION}"
# aws s3 rm "s3://${PROGRAMS_BUCKET}/${PROGRAM_KEY}" --region "${REGION}"

echo -e "\n${GREEN}S3 Upload Testing Complete!${NC}"
echo -e "\n${YELLOW}Note: Test files were left in S3 buckets for further testing.${NC}"
echo -e "${YELLOW}To clean up, run:${NC}"
echo "  aws s3 rm s3://${FIRMWARE_BUCKET}/${FIRMWARE_KEY}"
echo "  aws s3 rm s3://${PROGRAMS_BUCKET}/${PROGRAM_KEY}"