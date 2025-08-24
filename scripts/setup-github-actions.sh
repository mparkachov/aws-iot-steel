#!/bin/bash

# Setup script for GitHub Actions CI/CD pipeline
# This script helps configure the necessary AWS resources and GitHub secrets

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
GITHUB_REPO=""
AWS_REGION="us-west-2"
AWS_ACCOUNT_ID=""
PROJECT_NAME="aws-iot-steel"

print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

check_prerequisites() {
    print_header "Checking Prerequisites"
    
    # Check if AWS CLI is installed
    if ! command -v aws &> /dev/null; then
        print_error "AWS CLI is not installed. Please install it first."
        exit 1
    fi
    
    # Check if GitHub CLI is installed
    if ! command -v gh &> /dev/null; then
        print_error "GitHub CLI is not installed. Please install it first."
        exit 1
    fi
    
    # Check if jq is installed
    if ! command -v jq &> /dev/null; then
        print_error "jq is not installed. Please install it first."
        exit 1
    fi
    
    print_success "All prerequisites are installed"
}

get_configuration() {
    print_header "Configuration"
    
    # Get GitHub repository
    if [ -z "$GITHUB_REPO" ]; then
        read -p "Enter GitHub repository (owner/repo): " GITHUB_REPO
    fi
    
    # Get AWS account ID
    if [ -z "$AWS_ACCOUNT_ID" ]; then
        AWS_ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
        print_success "Detected AWS Account ID: $AWS_ACCOUNT_ID"
    fi
    
    # Confirm AWS region
    read -p "AWS Region [$AWS_REGION]: " input_region
    AWS_REGION=${input_region:-$AWS_REGION}
    
    print_success "Configuration complete"
    echo "  Repository: $GITHUB_REPO"
    echo "  AWS Account: $AWS_ACCOUNT_ID"
    echo "  AWS Region: $AWS_REGION"
}

create_oidc_provider() {
    print_header "Creating OIDC Identity Provider"
    
    # Check if OIDC provider already exists
    if aws iam get-open-id-connect-provider --open-id-connect-provider-arn "arn:aws:iam::$AWS_ACCOUNT_ID:oidc-provider/token.actions.githubusercontent.com" &> /dev/null; then
        print_warning "OIDC provider already exists"
        return
    fi
    
    # Create OIDC provider
    aws iam create-open-id-connect-provider \
        --url https://token.actions.githubusercontent.com \
        --client-id-list sts.amazonaws.com \
        --thumbprint-list 6938fd4d98bab03faadb97b34396831e3780aea1 \
        --tags Key=Project,Value=$PROJECT_NAME Key=Purpose,Value=GitHubActions
    
    print_success "OIDC provider created"
}

create_github_actions_role() {
    print_header "Creating GitHub Actions IAM Role"
    
    local role_name="GitHubActions-${PROJECT_NAME}"
    local trust_policy_file="/tmp/github-actions-trust-policy.json"
    local permissions_policy_file="/tmp/github-actions-permissions-policy.json"
    
    # Create trust policy
    cat > "$trust_policy_file" << EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "Federated": "arn:aws:iam::$AWS_ACCOUNT_ID:oidc-provider/token.actions.githubusercontent.com"
      },
      "Action": "sts:AssumeRoleWithWebIdentity",
      "Condition": {
        "StringEquals": {
          "token.actions.githubusercontent.com:aud": "sts.amazonaws.com"
        },
        "StringLike": {
          "token.actions.githubusercontent.com:sub": [
            "repo:$GITHUB_REPO:ref:refs/heads/main",
            "repo:$GITHUB_REPO:ref:refs/heads/develop"
          ]
        }
      }
    }
  ]
}
EOF
    
    # Create permissions policy
    cat > "$permissions_policy_file" << EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:PutObject",
        "s3:PutObjectAcl",
        "s3:GetObject"
      ],
      "Resource": [
        "arn:aws:s3:::${PROJECT_NAME}-build-artifacts-*/*"
      ]
    },
    {
      "Effect": "Allow",
      "Action": [
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::${PROJECT_NAME}-build-artifacts-*"
      ]
    }
  ]
}
EOF
    
    # Create IAM role
    if aws iam get-role --role-name "$role_name" &> /dev/null; then
        print_warning "IAM role $role_name already exists, updating trust policy"
        aws iam update-assume-role-policy --role-name "$role_name" --policy-document "file://$trust_policy_file"
    else
        aws iam create-role \
            --role-name "$role_name" \
            --assume-role-policy-document "file://$trust_policy_file" \
            --tags Key=Project,Value=$PROJECT_NAME Key=Purpose,Value=GitHubActions
        print_success "IAM role $role_name created"
    fi
    
    # Attach permissions policy
    local policy_name="GitHubActions-${PROJECT_NAME}-Policy"
    
    # Create or update the policy
    if aws iam get-policy --policy-arn "arn:aws:iam::$AWS_ACCOUNT_ID:policy/$policy_name" &> /dev/null; then
        print_warning "Policy $policy_name already exists, creating new version"
        aws iam create-policy-version \
            --policy-arn "arn:aws:iam::$AWS_ACCOUNT_ID:policy/$policy_name" \
            --policy-document "file://$permissions_policy_file" \
            --set-as-default
    else
        aws iam create-policy \
            --policy-name "$policy_name" \
            --policy-document "file://$permissions_policy_file" \
            --tags Key=Project,Value=$PROJECT_NAME Key=Purpose,Value=GitHubActions
        print_success "IAM policy $policy_name created"
    fi
    
    # Attach policy to role
    aws iam attach-role-policy \
        --role-name "$role_name" \
        --policy-arn "arn:aws:iam::$AWS_ACCOUNT_ID:policy/$policy_name"
    
    print_success "IAM role and policy configured"
    
    # Clean up temporary files
    rm -f "$trust_policy_file" "$permissions_policy_file"
    
    # Export role ARN for later use
    export GITHUB_ACTIONS_ROLE_ARN="arn:aws:iam::$AWS_ACCOUNT_ID:role/$role_name"
    echo "Role ARN: $GITHUB_ACTIONS_ROLE_ARN"
}

create_s3_buckets() {
    print_header "Creating S3 Buckets"
    
    local bucket_name="${PROJECT_NAME}-build-artifacts-${AWS_REGION}"
    
    # Create S3 bucket
    if aws s3 ls "s3://$bucket_name" &> /dev/null; then
        print_warning "S3 bucket $bucket_name already exists"
    else
        if [ "$AWS_REGION" = "us-east-1" ]; then
            aws s3 mb "s3://$bucket_name"
        else
            aws s3 mb "s3://$bucket_name" --region "$AWS_REGION"
        fi
        print_success "S3 bucket $bucket_name created"
    fi
    
    # Configure bucket versioning
    aws s3api put-bucket-versioning \
        --bucket "$bucket_name" \
        --versioning-configuration Status=Enabled
    
    # Configure bucket encryption
    aws s3api put-bucket-encryption \
        --bucket "$bucket_name" \
        --server-side-encryption-configuration '{
            "Rules": [
                {
                    "ApplyServerSideEncryptionByDefault": {
                        "SSEAlgorithm": "AES256"
                    }
                }
            ]
        }'
    
    # Block public access
    aws s3api put-public-access-block \
        --bucket "$bucket_name" \
        --public-access-block-configuration \
        BlockPublicAcls=true,IgnorePublicAcls=true,BlockPublicPolicy=true,RestrictPublicBuckets=true
    
    print_success "S3 bucket configured with security settings"
    
    # Export bucket name for later use
    export S3_BUILD_ARTIFACTS_BUCKET="$bucket_name"
}

configure_github_secrets() {
    print_header "Configuring GitHub Secrets"
    
    # Check if user is authenticated with GitHub CLI
    if ! gh auth status &> /dev/null; then
        print_error "Please authenticate with GitHub CLI first: gh auth login"
        exit 1
    fi
    
    # Set GitHub secrets
    echo "$GITHUB_ACTIONS_ROLE_ARN" | gh secret set AWS_GITHUB_ACTIONS_ROLE_ARN --repo "$GITHUB_REPO"
    echo "$AWS_REGION" | gh secret set AWS_REGION --repo "$GITHUB_REPO"
    echo "$S3_BUILD_ARTIFACTS_BUCKET" | gh secret set S3_BUILD_ARTIFACTS_BUCKET --repo "$GITHUB_REPO"
    
    print_success "GitHub secrets configured"
}

test_configuration() {
    print_header "Testing Configuration"
    
    # Test AWS CLI access
    if aws sts get-caller-identity &> /dev/null; then
        print_success "AWS CLI access confirmed"
    else
        print_error "AWS CLI access failed"
        exit 1
    fi
    
    # Test S3 bucket access
    if aws s3 ls "s3://$S3_BUILD_ARTIFACTS_BUCKET" &> /dev/null; then
        print_success "S3 bucket access confirmed"
    else
        print_error "S3 bucket access failed"
        exit 1
    fi
    
    # Test GitHub CLI access
    if gh repo view "$GITHUB_REPO" &> /dev/null; then
        print_success "GitHub repository access confirmed"
    else
        print_error "GitHub repository access failed"
        exit 1
    fi
    
    print_success "All tests passed"
}

print_summary() {
    print_header "Setup Complete"
    
    echo -e "${GREEN}GitHub Actions CI/CD pipeline has been configured successfully!${NC}"
    echo ""
    echo "Configuration Summary:"
    echo "  Repository: $GITHUB_REPO"
    echo "  AWS Account: $AWS_ACCOUNT_ID"
    echo "  AWS Region: $AWS_REGION"
    echo "  IAM Role: $GITHUB_ACTIONS_ROLE_ARN"
    echo "  S3 Bucket: $S3_BUILD_ARTIFACTS_BUCKET"
    echo ""
    echo "Next Steps:"
    echo "1. Push your code to trigger the first workflow run"
    echo "2. Monitor the workflow in the GitHub Actions tab"
    echo "3. Set up AWS CodePipeline for the deployment phase"
    echo ""
    echo "For more information, see .github/README.md"
}

main() {
    echo -e "${BLUE}AWS IoT Steel - GitHub Actions Setup${NC}"
    echo ""
    
    check_prerequisites
    get_configuration
    create_oidc_provider
    create_github_actions_role
    create_s3_buckets
    configure_github_secrets
    test_configuration
    print_summary
}

# Run main function
main "$@"