#!/bin/bash
# Production Deployment Script for AWS IoT Steel System
# This script deploys the complete system to production environment

set -euo pipefail

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PRODUCTION_DIR="${PROJECT_ROOT}/production"

# Default values
ENVIRONMENT="production"
AWS_REGION="${AWS_REGION:-us-west-2}"
DEPLOYMENT_BUCKET="${DEPLOYMENT_BUCKET:-aws-iot-steel-deployments}"
STACK_NAME_PREFIX="aws-iot-steel"
DEVICE_COUNT="${DEVICE_COUNT:-1}"
DRY_RUN="${DRY_RUN:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
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

# Usage function
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Deploy AWS IoT Steel System to Production

OPTIONS:
    -e, --environment ENVIRONMENT    Deployment environment (production|staging) [default: production]
    -r, --region REGION             AWS region [default: us-west-2]
    -b, --bucket BUCKET             S3 deployment bucket [default: aws-iot-steel-deployments]
    -s, --stack-prefix PREFIX       CloudFormation stack name prefix [default: aws-iot-steel]
    -d, --device-count COUNT        Number of devices to provision [default: 1]
    -n, --dry-run                   Perform a dry run without making changes
    -h, --help                      Show this help message

EXAMPLES:
    # Deploy to production with default settings
    $0

    # Deploy to staging environment
    $0 --environment staging

    # Deploy with custom region and device count
    $0 --region eu-west-1 --device-count 10

    # Perform a dry run
    $0 --dry-run

ENVIRONMENT VARIABLES:
    AWS_REGION                      AWS region (overridden by --region)
    DEPLOYMENT_BUCKET               S3 deployment bucket (overridden by --bucket)
    DEVICE_COUNT                    Number of devices to provision (overridden by --device-count)
    DRY_RUN                         Perform dry run (overridden by --dry-run)

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -e|--environment)
                ENVIRONMENT="$2"
                shift 2
                ;;
            -r|--region)
                AWS_REGION="$2"
                shift 2
                ;;
            -b|--bucket)
                DEPLOYMENT_BUCKET="$2"
                shift 2
                ;;
            -s|--stack-prefix)
                STACK_NAME_PREFIX="$2"
                shift 2
                ;;
            -d|--device-count)
                DEVICE_COUNT="$2"
                shift 2
                ;;
            -n|--dry-run)
                DRY_RUN="true"
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
}

# Validate prerequisites
validate_prerequisites() {
    log_info "Validating prerequisites..."

    # Check required tools
    local required_tools=("aws" "cargo" "jq" "openssl")
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            log_error "Required tool '$tool' is not installed"
            exit 1
        fi
    done

    # Check AWS credentials
    if ! aws sts get-caller-identity &> /dev/null; then
        log_error "AWS credentials not configured or invalid"
        exit 1
    fi

    # Check AWS region
    if ! aws ec2 describe-regions --region-names "$AWS_REGION" &> /dev/null; then
        log_error "Invalid AWS region: $AWS_REGION"
        exit 1
    fi

    # Validate environment
    if [[ "$ENVIRONMENT" != "production" && "$ENVIRONMENT" != "staging" ]]; then
        log_error "Invalid environment: $ENVIRONMENT (must be 'production' or 'staging')"
        exit 1
    fi

    # Check configuration files
    local config_file="${PRODUCTION_DIR}/config/${ENVIRONMENT}.toml"
    if [[ ! -f "$config_file" ]]; then
        log_error "Configuration file not found: $config_file"
        exit 1
    fi

    log_success "Prerequisites validated"
}

# Build the project
build_project() {
    log_info "Building AWS IoT Steel project..."

    cd "$PROJECT_ROOT"

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would build project with: cargo build --release --workspace"
        return
    fi

    # Build for all targets
    cargo build --release --workspace

    # Build ESP32 target if toolchain is available
    if rustup target list --installed | grep -q "riscv32imc-esp-espidf"; then
        log_info "Building ESP32 target..."
        cargo build --release --target riscv32imc-esp-espidf --package aws-iot-platform-esp32
    else
        log_warning "ESP32 toolchain not installed, skipping ESP32 build"
    fi

    log_success "Project built successfully"
}

# Package artifacts
package_artifacts() {
    log_info "Packaging deployment artifacts..."

    local package_dir="${PROJECT_ROOT}/target/deployment-${ENVIRONMENT}"
    local timestamp=$(date +%Y%m%d-%H%M%S)
    local version=$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would create package in: $package_dir"
        return
    fi

    # Create package directory
    rm -rf "$package_dir"
    mkdir -p "$package_dir"/{binaries,config,scripts,docs,certificates}

    # Copy binaries
    cp "${PROJECT_ROOT}/target/release/steel_runtime" "$package_dir/binaries/" 2>/dev/null || true
    cp "${PROJECT_ROOT}/target/release/steel_test" "$package_dir/binaries/" 2>/dev/null || true
    cp "${PROJECT_ROOT}/target/release/steel_example" "$package_dir/binaries/" 2>/dev/null || true
    cp "${PROJECT_ROOT}/target/release/end_to_end_validator" "$package_dir/binaries/" 2>/dev/null || true

    # Copy ESP32 binaries if available
    if [[ -d "${PROJECT_ROOT}/target/riscv32imc-esp-espidf/release" ]]; then
        mkdir -p "$package_dir/binaries/esp32"
        cp "${PROJECT_ROOT}/target/riscv32imc-esp-espidf/release"/*.bin "$package_dir/binaries/esp32/" 2>/dev/null || true
    fi

    # Copy configuration files
    cp -r "${PRODUCTION_DIR}/config"/* "$package_dir/config/"

    # Copy deployment scripts
    cp -r "${PRODUCTION_DIR}/scripts"/* "$package_dir/scripts/"
    chmod +x "$package_dir/scripts"/*.sh

    # Copy CloudFormation templates
    cp -r "${PROJECT_ROOT}/aws-infrastructure/cloudformation"/* "$package_dir/scripts/"

    # Copy documentation
    cp "${PROJECT_ROOT}/README.md" "$package_dir/docs/"
    cp "${PROJECT_ROOT}/CHANGELOG.md" "$package_dir/docs/"
    cp -r "${PRODUCTION_DIR}/docs"/* "$package_dir/docs/" 2>/dev/null || true

    # Create deployment manifest
    cat > "$package_dir/deployment-manifest.json" << EOF
{
    "version": "$version",
    "environment": "$ENVIRONMENT",
    "timestamp": "$timestamp",
    "region": "$AWS_REGION",
    "stack_prefix": "$STACK_NAME_PREFIX",
    "device_count": $DEVICE_COUNT,
    "artifacts": {
        "binaries": $(find "$package_dir/binaries" -type f -name "*" | jq -R . | jq -s .),
        "config_files": $(find "$package_dir/config" -type f -name "*.toml" | jq -R . | jq -s .),
        "scripts": $(find "$package_dir/scripts" -type f -name "*.sh" | jq -R . | jq -s .)
    }
}
EOF

    # Create deployment archive
    cd "$(dirname "$package_dir")"
    tar -czf "aws-iot-steel-${ENVIRONMENT}-${version}-${timestamp}.tar.gz" "$(basename "$package_dir")"

    log_success "Artifacts packaged in: $package_dir"
    log_success "Deployment archive created: aws-iot-steel-${ENVIRONMENT}-${version}-${timestamp}.tar.gz"
}

# Deploy AWS infrastructure
deploy_infrastructure() {
    log_info "Deploying AWS infrastructure..."

    local stack_name="${STACK_NAME_PREFIX}-${ENVIRONMENT}"
    local template_dir="${PROJECT_ROOT}/aws-infrastructure/cloudformation"

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would deploy CloudFormation stacks:"
        log_info "  - Core infrastructure: ${stack_name}-core"
        log_info "  - S3 and Lambda: ${stack_name}-s3-lambda"
        log_info "  - CodePipeline: ${stack_name}-codepipeline"
        return
    fi

    # Deploy core infrastructure
    log_info "Deploying core IoT infrastructure..."
    aws cloudformation deploy \
        --template-file "${template_dir}/core-iot-infrastructure.yaml" \
        --stack-name "${stack_name}-core" \
        --parameter-overrides \
            Environment="$ENVIRONMENT" \
            DeviceCount="$DEVICE_COUNT" \
        --capabilities CAPABILITY_IAM \
        --region "$AWS_REGION"

    # Deploy S3 and Lambda infrastructure
    log_info "Deploying S3 and Lambda infrastructure..."
    aws cloudformation deploy \
        --template-file "${template_dir}/s3-lambda-infrastructure.yaml" \
        --stack-name "${stack_name}-s3-lambda" \
        --parameter-overrides \
            Environment="$ENVIRONMENT" \
            DeploymentBucket="$DEPLOYMENT_BUCKET" \
        --capabilities CAPABILITY_IAM \
        --region "$AWS_REGION"

    # Deploy CodePipeline infrastructure
    log_info "Deploying CodePipeline infrastructure..."
    aws cloudformation deploy \
        --template-file "${template_dir}/codepipeline-infrastructure.yaml" \
        --stack-name "${stack_name}-codepipeline" \
        --parameter-overrides \
            Environment="$ENVIRONMENT" \
            SourceBucket="$DEPLOYMENT_BUCKET" \
        --capabilities CAPABILITY_IAM \
        --region "$AWS_REGION"

    log_success "AWS infrastructure deployed successfully"
}

# Provision devices
provision_devices() {
    log_info "Provisioning $DEVICE_COUNT device(s)..."

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would provision $DEVICE_COUNT devices"
        return
    fi

    for ((i=1; i<=DEVICE_COUNT; i++)); do
        local device_id="device-$(printf "%03d" $i)"
        local thing_name="${STACK_NAME_PREFIX}-${ENVIRONMENT}-${device_id}"

        log_info "Provisioning device: $device_id"

        # Run device provisioning script
        "${PROJECT_ROOT}/aws-infrastructure/scripts/provision-device.sh" \
            "$device_id" \
            "$ENVIRONMENT" \
            "$AWS_REGION"
    done

    log_success "All devices provisioned successfully"
}

# Upload artifacts to S3
upload_artifacts() {
    log_info "Uploading artifacts to S3..."

    local package_dir="${PROJECT_ROOT}/target/deployment-${ENVIRONMENT}"
    local s3_prefix="deployments/${ENVIRONMENT}/$(date +%Y/%m/%d)"

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would upload artifacts to: s3://${DEPLOYMENT_BUCKET}/${s3_prefix}/"
        return
    fi

    # Create S3 bucket if it doesn't exist
    if ! aws s3 ls "s3://$DEPLOYMENT_BUCKET" &> /dev/null; then
        log_info "Creating S3 bucket: $DEPLOYMENT_BUCKET"
        aws s3 mb "s3://$DEPLOYMENT_BUCKET" --region "$AWS_REGION"
    fi

    # Upload deployment package
    aws s3 sync "$package_dir" "s3://${DEPLOYMENT_BUCKET}/${s3_prefix}/" \
        --delete \
        --region "$AWS_REGION"

    # Upload deployment archive
    local archive_name="aws-iot-steel-${ENVIRONMENT}-$(date +%Y%m%d-%H%M%S).tar.gz"
    if [[ -f "${PROJECT_ROOT}/target/${archive_name}" ]]; then
        aws s3 cp "${PROJECT_ROOT}/target/${archive_name}" \
            "s3://${DEPLOYMENT_BUCKET}/archives/${archive_name}" \
            --region "$AWS_REGION"
    fi

    log_success "Artifacts uploaded to S3"
}

# Validate deployment
validate_deployment() {
    log_info "Validating deployment..."

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would validate deployment"
        return
    fi

    # Check CloudFormation stacks
    local stack_name="${STACK_NAME_PREFIX}-${ENVIRONMENT}"
    local stacks=("${stack_name}-core" "${stack_name}-s3-lambda" "${stack_name}-codepipeline")

    for stack in "${stacks[@]}"; do
        local status=$(aws cloudformation describe-stacks \
            --stack-name "$stack" \
            --region "$AWS_REGION" \
            --query 'Stacks[0].StackStatus' \
            --output text 2>/dev/null || echo "NOT_FOUND")

        if [[ "$status" == "CREATE_COMPLETE" || "$status" == "UPDATE_COMPLETE" ]]; then
            log_success "Stack $stack is healthy"
        else
            log_error "Stack $stack is in state: $status"
            exit 1
        fi
    done

    # Run end-to-end validation tests
    log_info "Running end-to-end validation tests..."
    cd "$PROJECT_ROOT"
    cargo run --bin end_to_end_validator --package aws-iot-tests -- \
        --test-suite e2e \
        --verbose

    log_success "Deployment validation completed"
}

# Generate deployment report
generate_report() {
    log_info "Generating deployment report..."

    local report_file="${PROJECT_ROOT}/target/deployment-report-${ENVIRONMENT}-$(date +%Y%m%d-%H%M%S).json"
    local version=$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would generate deployment report: $report_file"
        return
    fi

    # Get stack outputs
    local stack_name="${STACK_NAME_PREFIX}-${ENVIRONMENT}"
    local iot_endpoint=$(aws cloudformation describe-stacks \
        --stack-name "${stack_name}-core" \
        --region "$AWS_REGION" \
        --query 'Stacks[0].Outputs[?OutputKey==`IoTEndpoint`].OutputValue' \
        --output text 2>/dev/null || echo "unknown")

    # Create deployment report
    cat > "$report_file" << EOF
{
    "deployment": {
        "version": "$version",
        "environment": "$ENVIRONMENT",
        "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "region": "$AWS_REGION",
        "stack_prefix": "$STACK_NAME_PREFIX",
        "device_count": $DEVICE_COUNT,
        "dry_run": $DRY_RUN
    },
    "infrastructure": {
        "iot_endpoint": "$iot_endpoint",
        "deployment_bucket": "$DEPLOYMENT_BUCKET",
        "stacks": [
            "${stack_name}-core",
            "${stack_name}-s3-lambda",
            "${stack_name}-codepipeline"
        ]
    },
    "artifacts": {
        "binaries_built": true,
        "configuration_deployed": true,
        "documentation_included": true
    },
    "validation": {
        "infrastructure_validated": true,
        "end_to_end_tests_passed": true
    }
}
EOF

    log_success "Deployment report generated: $report_file"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up temporary files..."
    # Add cleanup logic here if needed
}

# Main deployment function
main() {
    log_info "Starting AWS IoT Steel Production Deployment"
    log_info "Environment: $ENVIRONMENT"
    log_info "Region: $AWS_REGION"
    log_info "Device Count: $DEVICE_COUNT"
    log_info "Dry Run: $DRY_RUN"
    echo

    # Set trap for cleanup
    trap cleanup EXIT

    # Execute deployment steps
    validate_prerequisites
    build_project
    package_artifacts
    deploy_infrastructure
    provision_devices
    upload_artifacts
    validate_deployment
    generate_report

    log_success "🎉 Production deployment completed successfully!"
    log_info "Check the deployment report for detailed information"
}

# Parse arguments and run main function
parse_args "$@"
main