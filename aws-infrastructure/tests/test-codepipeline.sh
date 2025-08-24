#!/bin/bash

# Test script for CodePipeline infrastructure
# Usage: ./test-codepipeline.sh [environment] [region]

set -e

# Default values
ENVIRONMENT=${1:-dev}
REGION=${2:-us-west-2}
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

# Test counters
TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    log_info "Running test: $test_name"
    
    if eval "$test_command"; then
        log_success "✅ PASSED: $test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        log_error "❌ FAILED: $test_name"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

log_info "Starting CodePipeline infrastructure tests..."
log_info "Environment: ${ENVIRONMENT}"
log_info "Region: ${REGION}"
echo

# Stack names
CORE_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-core-iot"
S3_LAMBDA_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-s3-lambda"
CODEPIPELINE_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-codepipeline"

# Test 1: Verify all required stacks exist
run_test "Core IoT stack exists" \
    "aws cloudformation describe-stacks --stack-name '$CORE_STACK_NAME' --region '$REGION' --query 'Stacks[0].StackStatus' --output text | grep -E '(CREATE_COMPLETE|UPDATE_COMPLETE)'"

run_test "S3 Lambda stack exists" \
    "aws cloudformation describe-stacks --stack-name '$S3_LAMBDA_STACK_NAME' --region '$REGION' --query 'Stacks[0].StackStatus' --output text | grep -E '(CREATE_COMPLETE|UPDATE_COMPLETE)'"

run_test "CodePipeline stack exists" \
    "aws cloudformation describe-stacks --stack-name '$CODEPIPELINE_STACK_NAME' --region '$REGION' --query 'Stacks[0].StackStatus' --output text | grep -E '(CREATE_COMPLETE|UPDATE_COMPLETE)'"

# Get stack outputs for further testing
log_info "Retrieving stack outputs..."

PIPELINE_NAME=$(aws cloudformation describe-stacks \
    --stack-name "$CODEPIPELINE_STACK_NAME" \
    --region "$REGION" \
    --query 'Stacks[0].Outputs[?OutputKey==`CodePipelineName`].OutputValue' \
    --output text 2>/dev/null || echo "")

ARTIFACTS_BUCKET=$(aws cloudformation describe-stacks \
    --stack-name "$S3_LAMBDA_STACK_NAME" \
    --region "$REGION" \
    --query 'Stacks[0].Outputs[?OutputKey==`FirmwareBucketName`].OutputValue' \
    --output text 2>/dev/null || echo "")

FIRMWARE_BUCKET=$(aws cloudformation describe-stacks \
    --stack-name "$S3_LAMBDA_STACK_NAME" \
    --region "$REGION" \
    --query 'Stacks[0].Outputs[?OutputKey==`FirmwareBucketName`].OutputValue' \
    --output text 2>/dev/null || echo "")

STEEL_PROGRAMS_BUCKET=$(aws cloudformation describe-stacks \
    --stack-name "$S3_LAMBDA_STACK_NAME" \
    --region "$REGION" \
    --query 'Stacks[0].Outputs[?OutputKey==`SteelProgramsBucketName`].OutputValue' \
    --output text 2>/dev/null || echo "")

INFRASTRUCTURE_PROJECT=$(aws cloudformation describe-stacks \
    --stack-name "$CODEPIPELINE_STACK_NAME" \
    --region "$REGION" \
    --query 'Stacks[0].Outputs[?OutputKey==`CodeBuildInfrastructureProjectName`].OutputValue' \
    --output text 2>/dev/null || echo "")

STEEL_PROGRAMS_PROJECT=$(aws cloudformation describe-stacks \
    --stack-name "$CODEPIPELINE_STACK_NAME" \
    --region "$REGION" \
    --query 'Stacks[0].Outputs[?OutputKey==`CodeBuildSteelProgramsProjectName`].OutputValue' \
    --output text 2>/dev/null || echo "")

# Test 2: Verify CodePipeline exists and is accessible
run_test "CodePipeline exists" \
    "aws codepipeline get-pipeline --name '$PIPELINE_NAME' --region '$REGION' > /dev/null"

# Test 3: Verify CodeBuild projects exist
run_test "Infrastructure CodeBuild project exists" \
    "aws codebuild batch-get-projects --names '$INFRASTRUCTURE_PROJECT' --region '$REGION' --query 'projects[0].name' --output text | grep -q '$INFRASTRUCTURE_PROJECT'"

run_test "Steel Programs CodeBuild project exists" \
    "aws codebuild batch-get-projects --names '$STEEL_PROGRAMS_PROJECT' --region '$REGION' --query 'projects[0].name' --output text | grep -q '$STEEL_PROGRAMS_PROJECT'"

# Test 4: Verify S3 buckets are accessible
run_test "Artifacts bucket accessible" \
    "aws s3 ls 's3://$ARTIFACTS_BUCKET' --region '$REGION' > /dev/null"

run_test "Firmware bucket accessible" \
    "aws s3 ls 's3://$FIRMWARE_BUCKET' --region '$REGION' > /dev/null"

run_test "Steel Programs bucket accessible" \
    "aws s3 ls 's3://$STEEL_PROGRAMS_BUCKET' --region '$REGION' > /dev/null"

# Test 5: Test CodeBuild project permissions
log_info "Testing CodeBuild project permissions..."

run_test "Infrastructure project can access S3" \
    "aws codebuild start-build --project-name '$INFRASTRUCTURE_PROJECT' --region '$REGION' --source-version 'main' --buildspec-override 'version: 0.2
phases:
  build:
    commands:
      - aws s3 ls s3://$ARTIFACTS_BUCKET --region $REGION
      - echo \"S3 access test completed\"' > /dev/null && sleep 5"

# Test 6: Create test deployment trigger
log_info "Creating test deployment trigger..."

TEST_DEPLOYMENT_ID="test-$(date +%Y%m%d-%H%M%S)"
TEST_TRIGGER_PAYLOAD=$(cat << EOF
{
  "deployment_id": "$TEST_DEPLOYMENT_ID",
  "package_version": "test-1.0.0",
  "package_checksum": "test-checksum-123",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "stage": "$ENVIRONMENT",
  "test_deployment": true,
  "artifacts": {
    "package_s3_key": "test/package.tar.gz",
    "manifest_s3_key": "test/manifest.json",
    "firmware_s3_key": "test/firmware.bin"
  },
  "git_info": {
    "commit": "test-commit",
    "branch": "test-branch",
    "repository": "test/repo"
  },
  "build_info": {
    "build_number": "1",
    "build_id": "test-build",
    "workflow": "test-workflow"
  }
}
EOF
)

TEST_TRIGGER_KEY="triggers/$ENVIRONMENT/test-deployment-$TEST_DEPLOYMENT_ID.json"

run_test "Upload test trigger to S3" \
    "echo '$TEST_TRIGGER_PAYLOAD' | aws s3 cp - 's3://$ARTIFACTS_BUCKET/$TEST_TRIGGER_KEY' --content-type 'application/json' --region '$REGION'"

# Test 7: Verify pipeline can be triggered
log_info "Testing pipeline trigger..."

run_test "Pipeline can be started manually" \
    "aws codepipeline start-pipeline-execution --name '$PIPELINE_NAME' --region '$REGION' > /dev/null"

# Wait a moment for the pipeline to start
sleep 10

# Test 8: Check pipeline execution status
run_test "Pipeline execution started" \
    "aws codepipeline list-pipeline-executions --pipeline-name '$PIPELINE_NAME' --region '$REGION' --query 'pipelineExecutionSummaries[0].status' --output text | grep -E '(InProgress|Succeeded)'"

# Test 9: Verify CloudWatch logs are being created
log_info "Checking CloudWatch logs..."

LOG_GROUP_NAME="/aws/codebuild/${PROJECT_NAME}-${ENVIRONMENT}"

run_test "CodeBuild log group exists" \
    "aws logs describe-log-groups --log-group-name-prefix '$LOG_GROUP_NAME' --region '$REGION' --query 'logGroups[0].logGroupName' --output text | grep -q '$LOG_GROUP_NAME'"

# Test 10: Test IAM permissions
log_info "Testing IAM permissions..."

CODEBUILD_ROLE_NAME="${PROJECT_NAME}-${ENVIRONMENT}-codebuild-infrastructure-role"

run_test "CodeBuild IAM role exists" \
    "aws iam get-role --role-name '$CODEBUILD_ROLE_NAME' > /dev/null"

run_test "CodeBuild role has CloudFormation permissions" \
    "aws iam list-attached-role-policies --role-name '$CODEBUILD_ROLE_NAME' --query 'AttachedPolicies[?contains(PolicyName, \`CloudFormation\`) || contains(PolicyArn, \`cloudformation\`)].PolicyName' --output text | grep -q '.'"

# Test 11: Test Lambda function integration
log_info "Testing Lambda function integration..."

LAMBDA_FUNCTION_NAME="${PROJECT_NAME}-${ENVIRONMENT}-url-generator"

run_test "URL Generator Lambda function exists" \
    "aws lambda get-function --function-name '$LAMBDA_FUNCTION_NAME' --region '$REGION' > /dev/null"

# Test 12: Test IoT integration
log_info "Testing IoT integration..."

THING_TYPE_NAME="${PROJECT_NAME}-${ENVIRONMENT}-thing-type"

run_test "IoT Thing Type exists" \
    "aws iot describe-thing-type --thing-type-name '$THING_TYPE_NAME' --region '$REGION' > /dev/null"

# Test 13: Cleanup test resources
log_info "Cleaning up test resources..."

run_test "Remove test trigger file" \
    "aws s3 rm 's3://$ARTIFACTS_BUCKET/$TEST_TRIGGER_KEY' --region '$REGION'"

# Generate test report
TEST_REPORT=$(cat << EOF
{
  "test_run_id": "codepipeline-test-$(date +%Y%m%d-%H%M%S)",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "environment": "$ENVIRONMENT",
  "region": "$REGION",
  "tests_total": $TESTS_TOTAL,
  "tests_passed": $TESTS_PASSED,
  "tests_failed": $TESTS_FAILED,
  "success_rate": $(echo "scale=2; $TESTS_PASSED * 100 / $TESTS_TOTAL" | bc -l)%,
  "pipeline_name": "$PIPELINE_NAME",
  "infrastructure_project": "$INFRASTRUCTURE_PROJECT",
  "steel_programs_project": "$STEEL_PROGRAMS_PROJECT",
  "test_status": "$([ $TESTS_FAILED -eq 0 ] && echo 'PASSED' || echo 'FAILED')"
}
EOF
)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_FILE="${SCRIPT_DIR}/../outputs/codepipeline-test-report-$(date +%Y%m%d-%H%M%S).json"
mkdir -p "$(dirname "$REPORT_FILE")"
echo "$TEST_REPORT" > "$REPORT_FILE"

# Summary
echo
echo "=== CODEPIPELINE INFRASTRUCTURE TEST RESULTS ==="
echo "🧪 Total Tests: $TESTS_TOTAL"
echo "✅ Passed: $TESTS_PASSED"
echo "❌ Failed: $TESTS_FAILED"
echo "📊 Success Rate: $(echo "scale=2; $TESTS_PASSED * 100 / $TESTS_TOTAL" | bc -l)%"
echo "📄 Report File: $REPORT_FILE"
echo

if [ $TESTS_FAILED -eq 0 ]; then
    log_success "All tests passed! CodePipeline infrastructure is working correctly."
    echo
    log_info "Pipeline Information:"
    echo "  Pipeline Name: $PIPELINE_NAME"
    echo "  Pipeline URL: https://console.aws.amazon.com/codesuite/codepipeline/pipelines/$PIPELINE_NAME/view?region=$REGION"
    echo "  Infrastructure Project: $INFRASTRUCTURE_PROJECT"
    echo "  Steel Programs Project: $STEEL_PROGRAMS_PROJECT"
    echo
    log_info "Next Steps:"
    echo "1. Test end-to-end deployment with real artifacts"
    echo "2. Monitor pipeline executions in CloudWatch"
    echo "3. Set up CloudWatch alarms for failures"
    echo "4. Configure notifications for deployment status"
    exit 0
else
    log_error "Some tests failed. Please review the output above and fix any issues."
    echo
    log_info "Common Issues:"
    echo "1. Check IAM permissions for CodeBuild roles"
    echo "2. Verify S3 bucket policies allow required access"
    echo "3. Ensure CloudFormation stacks are in healthy state"
    echo "4. Check CloudWatch logs for detailed error messages"
    exit 1
fi