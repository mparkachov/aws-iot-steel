#!/bin/bash

# Cleanup AWS Infrastructure
# Usage: ./cleanup-infrastructure.sh [environment] [region]

set -e

# Default values
ENVIRONMENT=${1:-dev}
REGION=${2:-us-west-2}
PROJECT_NAME="esp32-steel"
CORE_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-core-iot"
S3_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-s3-lambda"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${RED}WARNING: This will delete ALL AWS infrastructure for ${ENVIRONMENT} environment${NC}"
echo "Environment: ${ENVIRONMENT}"
echo "Region: ${REGION}"
echo ""

# Confirmation prompt
read -p "Are you sure you want to proceed? (yes/no): " CONFIRM
if [ "${CONFIRM}" != "yes" ]; then
    echo "Cleanup cancelled"
    exit 0
fi

# Check if AWS CLI is configured
if ! aws sts get-caller-identity > /dev/null 2>&1; then
    echo -e "${RED}Error: AWS CLI not configured or no valid credentials${NC}"
    exit 1
fi

# Function to delete stack if it exists
delete_stack_if_exists() {
    local stack_name=$1
    local stack_exists=$(aws cloudformation describe-stacks \
        --stack-name "${stack_name}" \
        --region "${REGION}" \
        --query 'Stacks[0].StackStatus' \
        --output text 2>/dev/null || echo "DOES_NOT_EXIST")
    
    if [ "${stack_exists}" != "DOES_NOT_EXIST" ]; then
        echo -e "${YELLOW}Deleting stack: ${stack_name}${NC}"
        aws cloudformation delete-stack \
            --stack-name "${stack_name}" \
            --region "${REGION}"
        
        echo -e "${YELLOW}Waiting for stack deletion to complete...${NC}"
        aws cloudformation wait stack-delete-complete \
            --stack-name "${stack_name}" \
            --region "${REGION}"
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}Stack ${stack_name} deleted successfully${NC}"
        else
            echo -e "${RED}Failed to delete stack ${stack_name}${NC}"
        fi
    else
        echo -e "${YELLOW}Stack ${stack_name} does not exist${NC}"
    fi
}

# Delete S3 and Lambda stack first (due to dependencies)
delete_stack_if_exists "${S3_STACK_NAME}"

# Delete core IoT stack
delete_stack_if_exists "${CORE_STACK_NAME}"

# Clean up output files
OUTPUT_DIR="$(dirname "${BASH_SOURCE[0]}")/../outputs"
if [ -d "${OUTPUT_DIR}" ]; then
    echo -e "${YELLOW}Cleaning up output files...${NC}"
    rm -f "${OUTPUT_DIR}/${ENVIRONMENT}-"*.json
    echo -e "${GREEN}Output files cleaned up${NC}"
fi

echo -e "\n${GREEN}Infrastructure cleanup complete!${NC}"