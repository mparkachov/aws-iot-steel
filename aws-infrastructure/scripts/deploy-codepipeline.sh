#!/bin/bash

# Deploy CodePipeline Infrastructure
# Usage: ./deploy-codepipeline.sh [environment] [region] [github-repo] [github-branch]

set -e

# Default values
ENVIRONMENT=${1:-dev}
REGION=${2:-us-west-2}
GITHUB_REPO=${3:-""}
GITHUB_BRANCH=${4:-main}
PROJECT_NAME="esp32-steel"
STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-codepipeline"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Deploying CodePipeline Infrastructure${NC}"
echo "Environment: ${ENVIRONMENT}"
echo "Region: ${REGION}"
echo "Stack Name: ${STACK_NAME}"
echo "GitHub Repository: ${GITHUB_REPO}"
echo "GitHub Branch: ${GITHUB_BRANCH}"
echo ""

# Check if AWS CLI is configured
if ! aws sts get-caller-identity > /dev/null 2>&1; then
    echo -e "${RED}Error: AWS CLI not configured or no valid credentials${NC}"
    exit 1
fi

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_PATH="${SCRIPT_DIR}/../cloudformation/codepipeline-infrastructure.yaml"

# Check if required stacks exist
CORE_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-core-iot"
S3_LAMBDA_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-s3-lambda"

for required_stack in "$CORE_STACK_NAME" "$S3_LAMBDA_STACK_NAME"; do
    STACK_EXISTS=$(aws cloudformation describe-stacks \
        --stack-name "${required_stack}" \
        --region "${REGION}" \
        --query 'Stacks[0].StackStatus' \
        --output text 2>/dev/null || echo "DOES_NOT_EXIST")

    if [ "${STACK_EXISTS}" = "DOES_NOT_EXIST" ]; then
        echo -e "${RED}Error: Required stack does not exist: ${required_stack}${NC}"
        echo "Please deploy the required infrastructure first:"
        echo "  ./deploy-core-infrastructure.sh ${ENVIRONMENT} ${REGION}"
        echo "  ./deploy-s3-lambda.sh ${ENVIRONMENT} ${REGION}"
        exit 1
    fi
done

# Get S3 bucket names from existing stacks
echo -e "${YELLOW}Retrieving S3 bucket names from existing stacks...${NC}"

ARTIFACTS_BUCKET=$(aws cloudformation describe-stacks \
    --stack-name "${S3_LAMBDA_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`FirmwareBucketName`].OutputValue' \
    --output text)

FIRMWARE_BUCKET=$(aws cloudformation describe-stacks \
    --stack-name "${S3_LAMBDA_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`FirmwareBucketName`].OutputValue' \
    --output text)

STEEL_PROGRAMS_BUCKET=$(aws cloudformation describe-stacks \
    --stack-name "${S3_LAMBDA_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`SteelProgramsBucketName`].OutputValue' \
    --output text)

if [ -z "$ARTIFACTS_BUCKET" ] || [ -z "$FIRMWARE_BUCKET" ] || [ -z "$STEEL_PROGRAMS_BUCKET" ]; then
    echo -e "${RED}Error: Could not retrieve S3 bucket names from existing stacks${NC}"
    exit 1
fi

echo "Artifacts Bucket: ${ARTIFACTS_BUCKET}"
echo "Firmware Bucket: ${FIRMWARE_BUCKET}"
echo "Steel Programs Bucket: ${STEEL_PROGRAMS_BUCKET}"

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

# Prepare parameters
PARAMETERS=(
    "ParameterKey=Environment,ParameterValue=${ENVIRONMENT}"
    "ParameterKey=ProjectName,ParameterValue=${PROJECT_NAME}"
    "ParameterKey=ArtifactsBucketName,ParameterValue=${ARTIFACTS_BUCKET}"
    "ParameterKey=FirmwareBucketName,ParameterValue=${FIRMWARE_BUCKET}"
    "ParameterKey=SteelProgramsBucketName,ParameterValue=${STEEL_PROGRAMS_BUCKET}"
    "ParameterKey=GitHubBranch,ParameterValue=${GITHUB_BRANCH}"
)

if [ -n "$GITHUB_REPO" ]; then
    PARAMETERS+=("ParameterKey=GitHubRepository,ParameterValue=${GITHUB_REPO}")
fi

# Deploy stack
aws cloudformation ${OPERATION} \
    --stack-name "${STACK_NAME}" \
    --template-body file://"${TEMPLATE_PATH}" \
    --parameters "${PARAMETERS[@]}" \
    --capabilities CAPABILITY_NAMED_IAM \
    --region "${REGION}" \
    --tags \
        Key=Project,Value="${PROJECT_NAME}" \
        Key=Environment,Value="${ENVIRONMENT}" \
        Key=ManagedBy,Value=CloudFormation \
        Key=Component,Value=CodePipeline

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
    OUTPUT_FILE="${SCRIPT_DIR}/../outputs/${ENVIRONMENT}-codepipeline-outputs.json"
    mkdir -p "$(dirname "${OUTPUT_FILE}")"
    aws cloudformation describe-stacks \
        --stack-name "${STACK_NAME}" \
        --region "${REGION}" \
        --query 'Stacks[0].Outputs' \
        --output json > "${OUTPUT_FILE}"
    
    echo -e "\n${GREEN}Outputs saved to: ${OUTPUT_FILE}${NC}"
    
    # Get CodePipeline name for testing
    PIPELINE_NAME=$(aws cloudformation describe-stacks \
        --stack-name "${STACK_NAME}" \
        --region "${REGION}" \
        --query 'Stacks[0].Outputs[?OutputKey==`CodePipelineName`].OutputValue' \
        --output text)
    
    if [ -n "${PIPELINE_NAME}" ]; then
        echo -e "\n${YELLOW}CodePipeline Information:${NC}"
        echo "Pipeline Name: ${PIPELINE_NAME}"
        echo "Pipeline URL: https://console.aws.amazon.com/codesuite/codepipeline/pipelines/${PIPELINE_NAME}/view?region=${REGION}"
        
        # Test pipeline trigger (optional)
        echo -e "\n${YELLOW}Testing pipeline trigger...${NC}"
        TEST_TRIGGER_PAYLOAD=$(cat << EOF
{
  "deployment_id": "test-$(date +%Y%m%d-%H%M%S)",
  "package_version": "test-1.0.0",
  "package_checksum": "test-checksum",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "stage": "${ENVIRONMENT}",
  "test_deployment": true
}
EOF
        )
        
        TEST_TRIGGER_KEY="triggers/${ENVIRONMENT}/test-deployment-$(date +%s).json"
        echo "$TEST_TRIGGER_PAYLOAD" | aws s3 cp - "s3://${ARTIFACTS_BUCKET}/${TEST_TRIGGER_KEY}" \
            --content-type "application/json" \
            --metadata "test=true,environment=${ENVIRONMENT}"
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}Test trigger uploaded successfully${NC}"
            echo "Monitor pipeline execution in AWS Console"
        else
            echo -e "${YELLOW}Test trigger upload failed${NC}"
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

echo -e "\n${GREEN}CodePipeline Infrastructure deployment complete!${NC}"
echo -e "\n${YELLOW}Next Steps:${NC}"
echo "1. Configure GitHub Actions to upload artifacts to S3"
echo "2. Test the pipeline by uploading a deployment trigger"
echo "3. Monitor pipeline execution in AWS Console"
echo "4. Set up CloudWatch alarms for pipeline failures"
echo ""
echo -e "${YELLOW}Pipeline Configuration:${NC}"
echo "  Pipeline Name: ${PIPELINE_NAME}"
echo "  Trigger Bucket: ${ARTIFACTS_BUCKET}"
echo "  Trigger Prefix: triggers/${ENVIRONMENT}/"
echo ""
echo -e "${YELLOW}To trigger the pipeline manually:${NC}"
echo "  aws s3 cp deployment-trigger.json s3://${ARTIFACTS_BUCKET}/triggers/${ENVIRONMENT}/manual-trigger-\$(date +%s).json"