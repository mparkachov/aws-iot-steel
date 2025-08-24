#!/bin/bash

# Deploy Core IoT Infrastructure
# Usage: ./deploy-core-infrastructure.sh [environment] [region]

set -e

# Default values
ENVIRONMENT=${1:-dev}
REGION=${2:-us-west-2}
PROJECT_NAME="esp32-steel"
STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-core-iot"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Deploying Core IoT Infrastructure${NC}"
echo "Environment: ${ENVIRONMENT}"
echo "Region: ${REGION}"
echo "Stack Name: ${STACK_NAME}"
echo ""

# Check if AWS CLI is configured
if ! aws sts get-caller-identity > /dev/null 2>&1; then
    echo -e "${RED}Error: AWS CLI not configured or no valid credentials${NC}"
    exit 1
fi

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_PATH="${SCRIPT_DIR}/../cloudformation/core-iot-infrastructure.yaml"

# Validate template
echo -e "${YELLOW}Validating CloudFormation template...${NC}"
aws cloudformation validate-template \
    --template-body file://"${TEMPLATE_PATH}" \
    --region "${REGION}"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}Template validation successful${NC}"
else
    echo -e "${RED}Template validation failed${NC}"
    exit 1
fi

# Check if stack exists
STACK_EXISTS=$(aws cloudformation describe-stacks \
    --stack-name "${STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].StackStatus' \
    --output text 2>/dev/null || echo "DOES_NOT_EXIST")

if [ "${STACK_EXISTS}" = "DOES_NOT_EXIST" ]; then
    echo -e "${YELLOW}Creating new stack...${NC}"
    OPERATION="create-stack"
else
    echo -e "${YELLOW}Updating existing stack...${NC}"
    OPERATION="update-stack"
fi

# Deploy stack
aws cloudformation ${OPERATION} \
    --stack-name "${STACK_NAME}" \
    --template-body file://"${TEMPLATE_PATH}" \
    --parameters \
        ParameterKey=Environment,ParameterValue="${ENVIRONMENT}" \
        ParameterKey=ProjectName,ParameterValue="${PROJECT_NAME}" \
    --capabilities CAPABILITY_NAMED_IAM \
    --region "${REGION}" \
    --tags \
        Key=Project,Value="${PROJECT_NAME}" \
        Key=Environment,Value="${ENVIRONMENT}" \
        Key=ManagedBy,Value=CloudFormation

if [ $? -eq 0 ]; then
    echo -e "${GREEN}Stack deployment initiated successfully${NC}"
else
    echo -e "${RED}Stack deployment failed${NC}"
    exit 1
fi

# Wait for stack operation to complete
echo -e "${YELLOW}Waiting for stack operation to complete...${NC}"
if [ "${OPERATION}" = "create-stack" ]; then
    aws cloudformation wait stack-create-complete \
        --stack-name "${STACK_NAME}" \
        --region "${REGION}"
else
    aws cloudformation wait stack-update-complete \
        --stack-name "${STACK_NAME}" \
        --region "${REGION}"
fi

if [ $? -eq 0 ]; then
    echo -e "${GREEN}Stack operation completed successfully${NC}"
    
    # Display stack outputs
    echo -e "\n${GREEN}Stack Outputs:${NC}"
    aws cloudformation describe-stacks \
        --stack-name "${STACK_NAME}" \
        --region "${REGION}" \
        --query 'Stacks[0].Outputs[*].[OutputKey,OutputValue]' \
        --output table
    
    # Save outputs to file for other scripts
    OUTPUT_FILE="${SCRIPT_DIR}/../outputs/${ENVIRONMENT}-core-iot-outputs.json"
    mkdir -p "$(dirname "${OUTPUT_FILE}")"
    aws cloudformation describe-stacks \
        --stack-name "${STACK_NAME}" \
        --region "${REGION}" \
        --query 'Stacks[0].Outputs' \
        --output json > "${OUTPUT_FILE}"
    
    echo -e "\n${GREEN}Outputs saved to: ${OUTPUT_FILE}${NC}"
else
    echo -e "${RED}Stack operation failed${NC}"
    exit 1
fi

echo -e "\n${GREEN}Core IoT Infrastructure deployment complete!${NC}"