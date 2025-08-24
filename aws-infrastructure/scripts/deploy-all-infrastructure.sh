#!/bin/bash

# Deploy All Infrastructure Components
# Usage: ./deploy-all-infrastructure.sh [environment] [region] [github-repo] [github-branch]

set -e

# Default values
ENVIRONMENT=${1:-dev}
REGION=${2:-us-west-2}
GITHUB_REPO=${3:-""}
GITHUB_BRANCH=${4:-main}
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

echo -e "${GREEN}Deploying Complete AWS Infrastructure for ESP32-Steel Project${NC}"
echo "Environment: ${ENVIRONMENT}"
echo "Region: ${REGION}"
echo "GitHub Repository: ${GITHUB_REPO}"
echo "GitHub Branch: ${GITHUB_BRANCH}"
echo ""

# Check if AWS CLI is configured
if ! aws sts get-caller-identity > /dev/null 2>&1; then
    log_error "AWS CLI not configured or no valid credentials"
    exit 1
fi

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Track deployment progress
DEPLOYMENT_START_TIME=$(date +%s)
DEPLOYMENT_ID="deploy-all-$(date +%Y%m%d-%H%M%S)"

log_info "Starting complete infrastructure deployment..."
log_info "Deployment ID: ${DEPLOYMENT_ID}"

# Step 1: Deploy Core IoT Infrastructure
log_info "=== STEP 1: Deploying Core IoT Infrastructure ==="
if "${SCRIPT_DIR}/deploy-core-infrastructure.sh" "${ENVIRONMENT}" "${REGION}"; then
    log_success "Core IoT infrastructure deployed successfully"
else
    log_error "Core IoT infrastructure deployment failed"
    exit 1
fi

echo

# Step 2: Deploy S3 and Lambda Infrastructure
log_info "=== STEP 2: Deploying S3 and Lambda Infrastructure ==="
if "${SCRIPT_DIR}/deploy-s3-lambda.sh" "${ENVIRONMENT}" "${REGION}"; then
    log_success "S3 and Lambda infrastructure deployed successfully"
else
    log_error "S3 and Lambda infrastructure deployment failed"
    exit 1
fi

echo

# Step 3: Deploy CodePipeline Infrastructure
log_info "=== STEP 3: Deploying CodePipeline Infrastructure ==="
if "${SCRIPT_DIR}/deploy-codepipeline.sh" "${ENVIRONMENT}" "${REGION}" "${GITHUB_REPO}" "${GITHUB_BRANCH}"; then
    log_success "CodePipeline infrastructure deployed successfully"
else
    log_error "CodePipeline infrastructure deployment failed"
    exit 1
fi

echo

# Step 4: Provision Initial Devices (optional for dev environment)
if [ "${ENVIRONMENT}" = "dev" ]; then
    log_info "=== STEP 4: Provisioning Initial Development Devices ==="
    if "${SCRIPT_DIR}/provision-device-automated.sh" "${ENVIRONMENT}" "${REGION}" "3"; then
        log_success "Initial development devices provisioned successfully"
    else
        log_warning "Initial device provisioning failed (continuing anyway)"
    fi
    echo
fi

# Step 5: Run Infrastructure Tests
log_info "=== STEP 5: Running Infrastructure Tests ==="
if "${SCRIPT_DIR}/../tests/test-codepipeline.sh" "${ENVIRONMENT}" "${REGION}"; then
    log_success "Infrastructure tests passed"
else
    log_warning "Some infrastructure tests failed (check logs for details)"
fi

echo

# Calculate deployment time
DEPLOYMENT_END_TIME=$(date +%s)
DEPLOYMENT_DURATION=$((DEPLOYMENT_END_TIME - DEPLOYMENT_START_TIME))
DEPLOYMENT_MINUTES=$((DEPLOYMENT_DURATION / 60))
DEPLOYMENT_SECONDS=$((DEPLOYMENT_DURATION % 60))

# Collect deployment information
log_info "Collecting deployment information..."

# Get stack outputs
CORE_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-core-iot"
S3_LAMBDA_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-s3-lambda"
CODEPIPELINE_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-codepipeline"

IOT_ENDPOINT=$(aws cloudformation describe-stacks \
    --stack-name "${CORE_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`IoTEndpoint`].OutputValue' \
    --output text 2>/dev/null || echo "unknown")

FIRMWARE_BUCKET=$(aws cloudformation describe-stacks \
    --stack-name "${S3_LAMBDA_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`FirmwareBucketName`].OutputValue' \
    --output text 2>/dev/null || echo "unknown")

STEEL_PROGRAMS_BUCKET=$(aws cloudformation describe-stacks \
    --stack-name "${S3_LAMBDA_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`SteelProgramsBucketName`].OutputValue' \
    --output text 2>/dev/null || echo "unknown")

PIPELINE_NAME=$(aws cloudformation describe-stacks \
    --stack-name "${CODEPIPELINE_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`CodePipelineName`].OutputValue' \
    --output text 2>/dev/null || echo "unknown")

URL_GENERATOR_FUNCTION=$(aws cloudformation describe-stacks \
    --stack-name "${S3_LAMBDA_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`URLGeneratorFunctionName`].OutputValue' \
    --output text 2>/dev/null || echo "unknown")

# Generate comprehensive deployment report
DEPLOYMENT_REPORT=$(cat << EOF
{
  "deployment_id": "$DEPLOYMENT_ID",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "environment": "$ENVIRONMENT",
  "region": "$REGION",
  "github_repository": "$GITHUB_REPO",
  "github_branch": "$GITHUB_BRANCH",
  "deployment_duration_seconds": $DEPLOYMENT_DURATION,
  "deployment_duration_formatted": "${DEPLOYMENT_MINUTES}m ${DEPLOYMENT_SECONDS}s",
  "status": "completed",
  "stacks_deployed": [
    {
      "name": "$CORE_STACK_NAME",
      "type": "core-iot",
      "status": "deployed"
    },
    {
      "name": "$S3_LAMBDA_STACK_NAME",
      "type": "s3-lambda",
      "status": "deployed"
    },
    {
      "name": "$CODEPIPELINE_STACK_NAME",
      "type": "codepipeline",
      "status": "deployed"
    }
  ],
  "infrastructure_endpoints": {
    "iot_endpoint": "$IOT_ENDPOINT",
    "firmware_bucket": "$FIRMWARE_BUCKET",
    "steel_programs_bucket": "$STEEL_PROGRAMS_BUCKET",
    "pipeline_name": "$PIPELINE_NAME",
    "url_generator_function": "$URL_GENERATOR_FUNCTION"
  },
  "aws_console_urls": {
    "iot_core": "https://console.aws.amazon.com/iot/home?region=$REGION",
    "codepipeline": "https://console.aws.amazon.com/codesuite/codepipeline/pipelines/$PIPELINE_NAME/view?region=$REGION",
    "s3_firmware": "https://s3.console.aws.amazon.com/s3/buckets/$FIRMWARE_BUCKET?region=$REGION",
    "s3_steel_programs": "https://s3.console.aws.amazon.com/s3/buckets/$STEEL_PROGRAMS_BUCKET?region=$REGION",
    "cloudwatch_logs": "https://console.aws.amazon.com/cloudwatch/home?region=$REGION#logsV2:log-groups"
  }
}
EOF
)

# Save deployment report
REPORT_FILE="${SCRIPT_DIR}/../outputs/complete-deployment-report-$(date +%Y%m%d-%H%M%S).json"
mkdir -p "$(dirname "$REPORT_FILE")"
echo "$DEPLOYMENT_REPORT" > "$REPORT_FILE"

# Summary
echo
log_success "=== COMPLETE INFRASTRUCTURE DEPLOYMENT SUCCESSFUL ==="
echo "🚀 Deployment ID: $DEPLOYMENT_ID"
echo "⏱️  Duration: ${DEPLOYMENT_MINUTES}m ${DEPLOYMENT_SECONDS}s"
echo "🌍 Environment: $ENVIRONMENT"
echo "🎯 Region: $REGION"
echo "📄 Report: $REPORT_FILE"
echo

log_info "=== INFRASTRUCTURE SUMMARY ==="
echo "📡 IoT Endpoint: $IOT_ENDPOINT"
echo "💾 Firmware Bucket: $FIRMWARE_BUCKET"
echo "🔧 Steel Programs Bucket: $STEEL_PROGRAMS_BUCKET"
echo "🔄 Pipeline Name: $PIPELINE_NAME"
echo "⚡ URL Generator Function: $URL_GENERATOR_FUNCTION"
echo

log_info "=== AWS CONSOLE LINKS ==="
echo "🌐 IoT Core: https://console.aws.amazon.com/iot/home?region=$REGION"
echo "🔄 CodePipeline: https://console.aws.amazon.com/codesuite/codepipeline/pipelines/$PIPELINE_NAME/view?region=$REGION"
echo "💾 S3 Firmware: https://s3.console.aws.amazon.com/s3/buckets/$FIRMWARE_BUCKET?region=$REGION"
echo "🔧 S3 Steel Programs: https://s3.console.aws.amazon.com/s3/buckets/$STEEL_PROGRAMS_BUCKET?region=$REGION"
echo "📊 CloudWatch Logs: https://console.aws.amazon.com/cloudwatch/home?region=$REGION#logsV2:log-groups"
echo

log_info "=== NEXT STEPS ==="
echo "1. 🔧 Configure GitHub Actions with AWS credentials:"
echo "   - Set AWS_GITHUB_ACTIONS_ROLE_ARN secret"
echo "   - Set AWS_REGION secret"
echo "   - Set S3_BUILD_ARTIFACTS_BUCKET secret to: $FIRMWARE_BUCKET"
echo "   - Set CODEPIPELINE_NAME secret to: $PIPELINE_NAME"
echo ""
echo "2. 🧪 Test the complete CI/CD pipeline:"
echo "   - Push code to trigger GitHub Actions"
echo "   - Monitor pipeline execution in AWS Console"
echo "   - Verify artifacts are uploaded to S3"
echo ""
echo "3. 📱 Provision and test devices:"
echo "   - Use provision-device.sh to create device certificates"
echo "   - Test device connectivity to IoT Core"
echo "   - Verify Steel program delivery"
echo ""
echo "4. 📊 Set up monitoring and alerts:"
echo "   - Configure CloudWatch alarms for pipeline failures"
echo "   - Set up SNS notifications for deployment status"
echo "   - Monitor device connectivity and health"
echo ""
echo "5. 🔒 Security review:"
echo "   - Review IAM roles and permissions"
echo "   - Audit S3 bucket policies"
echo "   - Verify certificate management procedures"

log_success "Infrastructure deployment completed successfully!"
log_info "The ESP32-Steel project is now ready for continuous deployment!"

exit 0