#!/bin/bash

# Deploy S3 and Lambda Infrastructure
# Usage: ./deploy-s3-lambda.sh [environment] [region]

set -e

# Default values
ENVIRONMENT=${1:-dev}
REGION=${2:-us-west-2}
PROJECT_NAME="esp32-steel"
STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-s3-lambda"
CORE_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-core-iot"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Deploying S3 and Lambda Infrastructure${NC}"
echo "Environment: ${ENVIRONMENT}"
echo "Region: ${REGION}"
echo "Stack Name: ${STACK_NAME}"
echo "Core Stack: ${CORE_STACK_NAME}"
echo ""

# Check if AWS CLI is configured
if ! aws sts get-caller-identity > /dev/null 2>&1; then
    echo -e "${RED}Error: AWS CLI not configured or no valid credentials${NC}"
    exit 1
fi

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_PATH="${SCRIPT_DIR}/../cloudformation/s3-lambda-infrastructure.yaml"

# Check if core infrastructure exists
CORE_STACK_EXISTS=$(aws cloudformation describe-stacks \
    --stack-name "${CORE_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].StackStatus' \
    --output text 2>/dev/null || echo "DOES_NOT_EXIST")

if [ "${CORE_STACK_EXISTS}" = "DOES_NOT_EXIST" ]; then
    echo -e "${RED}Error: Core IoT infrastructure stack does not exist${NC}"
    echo "Please deploy the core infrastructure first using deploy-core-infrastructure.sh"
    exit 1
fi

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
        ParameterKey=CoreStackName,ParameterValue="${CORE_STACK_NAME}" \
    --capabilities CAPABILITY_NAMED_IAM \
    --region "${REGION}" \
    --tags \
        Key=Project,Value="${PROJECT_NAME}" \
        Key=Environment,Value="${ENVIRONMENT}" \
        Key=ManagedBy,Value=CloudFormation \
        Key=Component,Value=S3Lambda

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
    OUTPUT_FILE="${SCRIPT_DIR}/../outputs/${ENVIRONMENT}-s3-lambda-outputs.json"
    mkdir -p "$(dirname "${OUTPUT_FILE}")"
    aws cloudformation describe-stacks \
        --stack-name "${STACK_NAME}" \
        --region "${REGION}" \
        --query 'Stacks[0].Outputs' \
        --output json > "${OUTPUT_FILE}"
    
    echo -e "\n${GREEN}Outputs saved to: ${OUTPUT_FILE}${NC}"
    
    # Test Lambda function
    echo -e "\n${YELLOW}Testing Lambda function...${NC}"
    FUNCTION_NAME=$(aws cloudformation describe-stacks \
        --stack-name "${STACK_NAME}" \
        --region "${REGION}" \
        --query 'Stacks[0].Outputs[?OutputKey==`URLGeneratorFunctionName`].OutputValue' \
        --output text)
    
    if [ -n "${FUNCTION_NAME}" ]; then
        TEST_PAYLOAD='{"device_id":"'${PROJECT_NAME}'-'${ENVIRONMENT}'-test-001","request_type":"firmware","resource_id":"1.0.0","request_id":"test-123"}'
        
        echo -e "${YELLOW}Invoking Lambda function with test payload...${NC}"
        aws lambda invoke \
            --function-name "${FUNCTION_NAME}" \
            --payload "${TEST_PAYLOAD}" \
            --region "${REGION}" \
            /tmp/lambda-response.json > /dev/null
        
        LAMBDA_STATUS=$?
        if [ ${LAMBDA_STATUS} -eq 0 ]; then
            echo -e "${GREEN}Lambda function test completed${NC}"
            echo -e "${YELLOW}Response:${NC}"
            cat /tmp/lambda-response.json | jq .
            rm -f /tmp/lambda-response.json
        else
            echo -e "${YELLOW}Lambda function test failed (expected for missing test objects)${NC}"
        fi
    fi
    
else
    echo -e "${RED}Stack operation failed${NC}"
    
    # Show stack events for debugging
    echo -e "\n${RED}Recent stack events:${NC}"
    aws cloudformation describe-stack-events \
        --stack-name "${STACK_NAME}" \
        --region "${REGION}" \
        --query 'StackEvents[0:10].[Timestamp,ResourceStatus,ResourceType,LogicalResourceId,ResourceStatusReason]' \
        --output table
    
    exit 1
fi

echo -e "\n${GREEN}S3 and Lambda Infrastructure deployment complete!${NC}"
echo -e "\n${YELLOW}Next Steps:${NC}"
echo "1. Upload firmware files to the firmware bucket"
echo "2. Upload Steel programs to the programs bucket"
echo "3. Test device download requests"
echo ""
echo -e "${YELLOW}Bucket Information:${NC}"
FIRMWARE_BUCKET=$(aws cloudformation describe-stacks \
    --stack-name "${STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`FirmwareBucketName`].OutputValue' \
    --output text)

PROGRAMS_BUCKET=$(aws cloudformation describe-stacks \
    --stack-name "${STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`SteelProgramsBucketName`].OutputValue' \
    --output text)

echo "  Firmware Bucket: ${FIRMWARE_BUCKET}"
echo "  Programs Bucket: ${PROGRAMS_BUCKET}"