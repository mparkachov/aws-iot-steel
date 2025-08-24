#!/bin/bash

# Run All Infrastructure Tests
# Usage: ./run-all-tests.sh [environment] [region]

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

echo -e "${BLUE}Running All Infrastructure Tests${NC}"
echo -e "${BLUE}================================${NC}"
echo "Environment: ${ENVIRONMENT}"
echo "Region: ${REGION}"
echo "Project: ${PROJECT_NAME}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

# Check AWS CLI
if ! command -v aws &> /dev/null; then
    echo -e "${RED}❌ AWS CLI not found${NC}"
    exit 1
fi

# Check jq
if ! command -v jq &> /dev/null; then
    echo -e "${RED}❌ jq not found (required for JSON parsing)${NC}"
    exit 1
fi

# Check Python
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Python 3 not found${NC}"
    exit 1
fi

# Check AWS credentials
if ! aws sts get-caller-identity > /dev/null 2>&1; then
    echo -e "${RED}❌ AWS CLI not configured or no valid credentials${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All prerequisites met${NC}"

# Check if stacks exist
echo -e "\n${YELLOW}Checking infrastructure deployment...${NC}"

CORE_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-core-iot"
S3_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-s3-lambda"

CORE_STACK_EXISTS=$(aws cloudformation describe-stacks \
    --stack-name "${CORE_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].StackStatus' \
    --output text 2>/dev/null || echo "DOES_NOT_EXIST")

S3_STACK_EXISTS=$(aws cloudformation describe-stacks \
    --stack-name "${S3_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].StackStatus' \
    --output text 2>/dev/null || echo "DOES_NOT_EXIST")

if [ "${CORE_STACK_EXISTS}" = "DOES_NOT_EXIST" ]; then
    echo -e "${RED}❌ Core IoT stack not found: ${CORE_STACK_NAME}${NC}"
    echo "Please deploy the core infrastructure first"
    exit 1
fi

if [ "${S3_STACK_EXISTS}" = "DOES_NOT_EXIST" ]; then
    echo -e "${RED}❌ S3 Lambda stack not found: ${S3_STACK_NAME}${NC}"
    echo "Please deploy the S3 Lambda infrastructure first"
    exit 1
fi

echo -e "${GREEN}✅ Core IoT stack: ${CORE_STACK_EXISTS}${NC}"
echo -e "${GREEN}✅ S3 Lambda stack: ${S3_STACK_EXISTS}${NC}"

# Initialize test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=()

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "\n${BLUE}Running: ${test_name}${NC}"
    echo -e "${BLUE}$(printf '=%.0s' {1..50})${NC}"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "${test_command}"; then
        echo -e "${GREEN}✅ ${test_name}: PASSED${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}❌ ${test_name}: FAILED${NC}"
        FAILED_TESTS+=("${test_name}")
        return 1
    fi
}

# Test 1: S3 Upload Functionality
run_test "S3 Upload Functionality" "${SCRIPT_DIR}/test-s3-upload.sh ${ENVIRONMENT} ${REGION}"

# Test 2: Lambda Function and S3 Security
run_test "Lambda Function and S3 Security" "python3 ${SCRIPT_DIR}/test-lambda-function.py --environment ${ENVIRONMENT} --region ${REGION} --project-name ${PROJECT_NAME}"

# Test 3: CloudFormation Template Validation
run_test "CloudFormation Template Validation" "${SCRIPT_DIR}/validate-templates.sh"

# Test 4: IAM Role Permissions (basic check)
echo -e "\n${BLUE}Running: IAM Role Permissions Check${NC}"
echo -e "${BLUE}$(printf '=%.0s' {1..50})${NC}"

TOTAL_TESTS=$((TOTAL_TESTS + 1))

# Get CI/CD role ARN
CICD_ROLE_ARN=$(aws cloudformation describe-stacks \
    --stack-name "${S3_STACK_NAME}" \
    --region "${REGION}" \
    --query 'Stacks[0].Outputs[?OutputKey==`CICDRoleArn`].OutputValue' \
    --output text)

if [ -n "${CICD_ROLE_ARN}" ]; then
    echo "CI/CD Role ARN: ${CICD_ROLE_ARN}"
    
    # Check if role exists and has policies
    ROLE_NAME=$(echo "${CICD_ROLE_ARN}" | cut -d'/' -f2)
    POLICIES=$(aws iam list-attached-role-policies --role-name "${ROLE_NAME}" --query 'AttachedPolicies[].PolicyName' --output text)
    INLINE_POLICIES=$(aws iam list-role-policies --role-name "${ROLE_NAME}" --query 'PolicyNames' --output text)
    
    if [ -n "${POLICIES}" ] || [ -n "${INLINE_POLICIES}" ]; then
        echo -e "${GREEN}✅ IAM Role Permissions Check: PASSED${NC}"
        echo "Attached Policies: ${POLICIES}"
        echo "Inline Policies: ${INLINE_POLICIES}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ IAM Role Permissions Check: FAILED (No policies found)${NC}"
        FAILED_TESTS+=("IAM Role Permissions Check")
    fi
else
    echo -e "${RED}❌ IAM Role Permissions Check: FAILED (Role ARN not found)${NC}"
    FAILED_TESTS+=("IAM Role Permissions Check")
fi

# Test 5: IoT Rule Validation
echo -e "\n${BLUE}Running: IoT Rule Validation${NC}"
echo -e "${BLUE}$(printf '=%.0s' {1..50})${NC}"

TOTAL_TESTS=$((TOTAL_TESTS + 1))

RULE_NAME="${PROJECT_NAME}_${ENVIRONMENT}_url_generator_rule"
RULE_EXISTS=$(aws iot get-topic-rule --rule-name "${RULE_NAME}" --region "${REGION}" --query 'rule.ruleName' --output text 2>/dev/null || echo "DOES_NOT_EXIST")

if [ "${RULE_EXISTS}" != "DOES_NOT_EXIST" ]; then
    echo -e "${GREEN}✅ IoT Rule Validation: PASSED${NC}"
    echo "Rule Name: ${RULE_NAME}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}❌ IoT Rule Validation: FAILED (Rule not found)${NC}"
    FAILED_TESTS+=("IoT Rule Validation")
fi

# Generate test report
echo -e "\n${BLUE}$(printf '=%.0s' {1..60})${NC}"
echo -e "${BLUE}TEST REPORT${NC}"
echo -e "${BLUE}$(printf '=%.0s' {1..60})${NC}"

echo -e "\nEnvironment: ${ENVIRONMENT}"
echo -e "Region: ${REGION}"
echo -e "Timestamp: $(date)"
echo -e "\nResults:"
echo -e "  Total Tests: ${TOTAL_TESTS}"
echo -e "  Passed: ${GREEN}${PASSED_TESTS}${NC}"
echo -e "  Failed: ${RED}$((TOTAL_TESTS - PASSED_TESTS))${NC}"

if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
    echo -e "\n${RED}Failed Tests:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo -e "  ❌ ${test}"
    done
fi

# Success/failure summary
echo -e "\n$(printf '=%.0s' {1..60})"
if [ ${PASSED_TESTS} -eq ${TOTAL_TESTS} ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED! Infrastructure is working correctly.${NC}"
    
    echo -e "\n${YELLOW}Next Steps:${NC}"
    echo "1. Provision devices using provision-device.sh"
    echo "2. Upload firmware and Steel programs to S3 buckets"
    echo "3. Test end-to-end device connectivity"
    
    exit 0
else
    echo -e "${RED}❌ SOME TESTS FAILED! Please review the failures above.${NC}"
    
    echo -e "\n${YELLOW}Troubleshooting:${NC}"
    echo "1. Check CloudFormation stack events for deployment issues"
    echo "2. Verify IAM permissions for your AWS credentials"
    echo "3. Check CloudWatch logs for Lambda function errors"
    echo "4. Ensure all required AWS services are available in your region"
    
    exit 1
fi