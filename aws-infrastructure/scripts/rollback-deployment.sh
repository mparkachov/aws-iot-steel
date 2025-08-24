#!/bin/bash

# Deployment Rollback Script
# This script handles rollback procedures for failed deployments
# Usage: ./rollback-deployment.sh [environment] [region] [rollback-version]

set -e

# Default values
ENVIRONMENT=${1:-dev}
REGION=${2:-us-west-2}
ROLLBACK_VERSION=${3:-""}
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

log_info "Starting deployment rollback procedure..."
log_info "Environment: ${ENVIRONMENT}"
log_info "Region: ${REGION}"
log_info "Rollback Version: ${ROLLBACK_VERSION:-auto-detect}"

# Check if AWS CLI is configured
if ! aws sts get-caller-identity > /dev/null 2>&1; then
    log_error "AWS CLI not configured or no valid credentials"
    exit 1
fi

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Stack names
CORE_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-core-iot"
S3_LAMBDA_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-s3-lambda"
CODEPIPELINE_STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}-codepipeline"

# Function to get stack status
get_stack_status() {
    local stack_name=$1
    aws cloudformation describe-stacks \
        --stack-name "${stack_name}" \
        --region "${REGION}" \
        --query 'Stacks[0].StackStatus' \
        --output text 2>/dev/null || echo "DOES_NOT_EXIST"
}

# Function to check if stack is in failed state
is_stack_failed() {
    local stack_status=$1
    case "$stack_status" in
        *FAILED*|*ROLLBACK*) return 0 ;;
        *) return 1 ;;
    esac
}

# Function to rollback CloudFormation stack
rollback_stack() {
    local stack_name=$1
    local stack_status=$(get_stack_status "$stack_name")
    
    log_info "Checking stack: ${stack_name} (Status: ${stack_status})"
    
    if [ "$stack_status" = "DOES_NOT_EXIST" ]; then
        log_warning "Stack does not exist: ${stack_name}"
        return 0
    fi
    
    if is_stack_failed "$stack_status"; then
        log_warning "Stack is in failed state: ${stack_name} (${stack_status})"
        
        # Try to continue rollback if it's stuck
        if [[ "$stack_status" == *"ROLLBACK_IN_PROGRESS"* ]]; then
            log_info "Rollback already in progress for: ${stack_name}"
            return 0
        fi
        
        # Cancel update if it's in progress
        if [[ "$stack_status" == *"UPDATE_IN_PROGRESS"* ]]; then
            log_info "Cancelling update for stack: ${stack_name}"
            aws cloudformation cancel-update-stack \
                --stack-name "${stack_name}" \
                --region "${REGION}" || log_warning "Failed to cancel update"
        fi
        
        # Continue rollback
        log_info "Continuing rollback for stack: ${stack_name}"
        aws cloudformation continue-update-rollback \
            --stack-name "${stack_name}" \
            --region "${REGION}" || log_warning "Failed to continue rollback"
        
        # Wait for rollback to complete
        log_info "Waiting for rollback to complete: ${stack_name}"
        aws cloudformation wait stack-rollback-complete \
            --stack-name "${stack_name}" \
            --region "${REGION}" || log_error "Rollback failed for stack: ${stack_name}"
        
        log_success "Rollback completed for stack: ${stack_name}"
    else
        log_success "Stack is in healthy state: ${stack_name} (${stack_status})"
    fi
}

# Function to rollback OTA updates
rollback_ota_updates() {
    log_info "Checking for active OTA updates to cancel..."
    
    # List active OTA updates
    ACTIVE_OTAS=$(aws iot list-ota-updates \
        --ota-update-status IN_PROGRESS \
        --region "${REGION}" \
        --query 'otaUpdates[?contains(otaUpdateId, `'${PROJECT_NAME}'-'${ENVIRONMENT}'`)].otaUpdateId' \
        --output text)
    
    if [ -n "$ACTIVE_OTAS" ]; then
        for ota_id in $ACTIVE_OTAS; do
            log_info "Cancelling OTA update: ${ota_id}"
            aws iot cancel-ota-update \
                --ota-update-id "${ota_id}" \
                --region "${REGION}" || log_warning "Failed to cancel OTA update: ${ota_id}"
        done
    else
        log_info "No active OTA updates found"
    fi
}

# Function to rollback to previous firmware version
rollback_firmware_version() {
    if [ -z "$ROLLBACK_VERSION" ]; then
        log_info "Auto-detecting previous firmware version..."
        
        # Get S3 bucket name
        FIRMWARE_BUCKET=$(aws cloudformation describe-stacks \
            --stack-name "${S3_LAMBDA_STACK_NAME}" \
            --region "${REGION}" \
            --query 'Stacks[0].Outputs[?OutputKey==`FirmwareBucketName`].OutputValue' \
            --output text 2>/dev/null || echo "")
        
        if [ -z "$FIRMWARE_BUCKET" ]; then
            log_warning "Could not determine firmware bucket name"
            return 1
        fi
        
        # List firmware versions and get the second most recent
        ROLLBACK_VERSION=$(aws s3 ls "s3://${FIRMWARE_BUCKET}/firmware/" \
            --region "${REGION}" \
            | grep "PRE" \
            | awk '{print $2}' \
            | sed 's|/||' \
            | sort -V \
            | tail -n 2 \
            | head -n 1)
        
        if [ -z "$ROLLBACK_VERSION" ]; then
            log_warning "Could not auto-detect rollback version"
            return 1
        fi
        
        log_info "Auto-detected rollback version: ${ROLLBACK_VERSION}"
    fi
    
    log_info "Rolling back firmware to version: ${ROLLBACK_VERSION}"
    
    # Update device shadows to request firmware rollback
    aws iot list-things \
        --thing-type-name "${PROJECT_NAME}-${ENVIRONMENT}-thing-type" \
        --region "${REGION}" \
        --query 'things[].thingName' \
        --output text | tr '\t' '\n' | while read thing_name; do
        
        if [ -n "$thing_name" ]; then
            log_info "Updating shadow for rollback: ${thing_name}"
            
            ROLLBACK_SHADOW=$(cat << EOF
{
  "state": {
    "desired": {
      "firmware_update": {
        "version": "${ROLLBACK_VERSION}",
        "request_id": "rollback-$(date +%s)",
        "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "rollback": true,
        "reason": "deployment_rollback"
      }
    }
  }
}
EOF
            )
            
            aws iot-data update-thing-shadow \
                --thing-name "$thing_name" \
                --payload "$ROLLBACK_SHADOW" \
                --region "${REGION}" \
                /tmp/rollback-shadow-response.json || log_warning "Shadow update failed for $thing_name"
        fi
    done
    
    log_success "Firmware rollback initiated for all devices"
}

# Function to rollback Steel programs
rollback_steel_programs() {
    log_info "Rolling back Steel programs..."
    
    # Get Steel programs bucket
    STEEL_PROGRAMS_BUCKET=$(aws cloudformation describe-stacks \
        --stack-name "${S3_LAMBDA_STACK_NAME}" \
        --region "${REGION}" \
        --query 'Stacks[0].Outputs[?OutputKey==`SteelProgramsBucketName`].OutputValue' \
        --output text 2>/dev/null || echo "")
    
    if [ -z "$STEEL_PROGRAMS_BUCKET" ]; then
        log_warning "Could not determine Steel programs bucket name"
        return 1
    fi
    
    # Broadcast rollback message to all devices
    ROLLBACK_MESSAGE=$(cat << EOF
{
  "action": "rollback",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "reason": "deployment_rollback",
  "instructions": "Stop current programs and revert to previous versions"
}
EOF
    )
    
    aws iot-data publish \
        --topic "steel-programs/broadcast/rollback" \
        --payload "$ROLLBACK_MESSAGE" \
        --region "${REGION}" || log_warning "Failed to broadcast rollback message"
    
    log_success "Steel programs rollback message broadcasted"
}

# Function to stop CodePipeline execution
stop_pipeline_execution() {
    log_info "Checking for active CodePipeline executions..."
    
    PIPELINE_NAME="${PROJECT_NAME}-${ENVIRONMENT}-deployment-pipeline"
    
    # Get active executions
    ACTIVE_EXECUTIONS=$(aws codepipeline list-pipeline-executions \
        --pipeline-name "${PIPELINE_NAME}" \
        --region "${REGION}" \
        --query 'pipelineExecutionSummaries[?status==`InProgress`].pipelineExecutionId' \
        --output text 2>/dev/null || echo "")
    
    if [ -n "$ACTIVE_EXECUTIONS" ]; then
        for execution_id in $ACTIVE_EXECUTIONS; do
            log_info "Stopping pipeline execution: ${execution_id}"
            aws codepipeline stop-pipeline-execution \
                --pipeline-name "${PIPELINE_NAME}" \
                --pipeline-execution-id "${execution_id}" \
                --abandon \
                --reason "Deployment rollback initiated" \
                --region "${REGION}" || log_warning "Failed to stop execution: ${execution_id}"
        done
    else
        log_info "No active pipeline executions found"
    fi
}

# Main rollback procedure
log_info "=== STARTING DEPLOYMENT ROLLBACK ==="

# Step 1: Stop active pipeline executions
stop_pipeline_execution

# Step 2: Cancel active OTA updates
rollback_ota_updates

# Step 3: Rollback CloudFormation stacks
log_info "Rolling back CloudFormation stacks..."
rollback_stack "$CODEPIPELINE_STACK_NAME"
rollback_stack "$S3_LAMBDA_STACK_NAME"
rollback_stack "$CORE_STACK_NAME"

# Step 4: Rollback firmware version
rollback_firmware_version

# Step 5: Rollback Steel programs
rollback_steel_programs

# Generate rollback report
ROLLBACK_REPORT=$(cat << EOF
{
  "rollback_id": "rollback-$(date +%Y%m%d-%H%M%S)",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "environment": "${ENVIRONMENT}",
  "region": "${REGION}",
  "rollback_version": "${ROLLBACK_VERSION}",
  "actions_performed": [
    "stopped_pipeline_executions",
    "cancelled_ota_updates",
    "rolled_back_cloudformation_stacks",
    "initiated_firmware_rollback",
    "broadcasted_steel_programs_rollback"
  ],
  "status": "completed"
}
EOF
)

REPORT_FILE="${SCRIPT_DIR}/../outputs/rollback-report-$(date +%Y%m%d-%H%M%S).json"
mkdir -p "$(dirname "$REPORT_FILE")"
echo "$ROLLBACK_REPORT" > "$REPORT_FILE"

# Summary
echo
log_success "=== DEPLOYMENT ROLLBACK COMPLETE ==="
echo "🔄 Rollback ID: rollback-$(date +%Y%m%d-%H%M%S)"
echo "🌍 Environment: ${ENVIRONMENT}"
echo "🎯 Region: ${REGION}"
echo "📦 Rollback Version: ${ROLLBACK_VERSION}"
echo "📄 Report File: ${REPORT_FILE}"
echo

log_success "Rollback procedure completed successfully!"
log_info "Monitor device status and verify rollback completion"
log_info "Check CloudWatch logs for any issues"

echo -e "\n${YELLOW}Next Steps:${NC}"
echo "1. Monitor device shadows for rollback confirmation"
echo "2. Verify firmware versions on devices"
echo "3. Check Steel program execution status"
echo "4. Review CloudWatch logs for errors"
echo "5. Investigate root cause of deployment failure"