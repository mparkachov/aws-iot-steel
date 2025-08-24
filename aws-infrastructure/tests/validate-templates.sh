#!/bin/bash

# Validate CloudFormation Templates
# Usage: ./validate-templates.sh [region]

set -e

# Default values
REGION=${1:-us-west-2}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Validating CloudFormation Templates${NC}"
echo "Region: ${REGION}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_DIR="${SCRIPT_DIR}/../cloudformation"

# Check if AWS CLI is configured
if ! aws sts get-caller-identity > /dev/null 2>&1; then
    echo -e "${RED}Error: AWS CLI not configured or no valid credentials${NC}"
    exit 1
fi

# Find all CloudFormation templates
TEMPLATES=$(find "${TEMPLATE_DIR}" -name "*.yaml" -o -name "*.yml" -o -name "*.json")

if [ -z "${TEMPLATES}" ]; then
    echo -e "${RED}No CloudFormation templates found in ${TEMPLATE_DIR}${NC}"
    exit 1
fi

TOTAL_TEMPLATES=0
VALID_TEMPLATES=0
INVALID_TEMPLATES=()

# Validate each template
for template in ${TEMPLATES}; do
    TOTAL_TEMPLATES=$((TOTAL_TEMPLATES + 1))
    template_name=$(basename "${template}")
    
    echo -e "${YELLOW}Validating: ${template_name}${NC}"
    
    # Validate template syntax
    if aws cloudformation validate-template \
        --template-body file://"${template}" \
        --region "${REGION}" > /dev/null 2>&1; then
        
        echo -e "${GREEN}✅ ${template_name}: Valid${NC}"
        VALID_TEMPLATES=$((VALID_TEMPLATES + 1))
        
        # Show template summary
        DESCRIPTION=$(aws cloudformation validate-template \
            --template-body file://"${template}" \
            --region "${REGION}" \
            --query 'Description' \
            --output text 2>/dev/null || echo "No description")
        
        PARAMETERS=$(aws cloudformation validate-template \
            --template-body file://"${template}" \
            --region "${REGION}" \
            --query 'Parameters[].ParameterKey' \
            --output text 2>/dev/null || echo "None")
        
        echo "  Description: ${DESCRIPTION}"
        echo "  Parameters: ${PARAMETERS}"
        
    else
        echo -e "${RED}❌ ${template_name}: Invalid${NC}"
        INVALID_TEMPLATES+=("${template_name}")
        
        # Show validation error
        echo -e "${RED}Error details:${NC}"
        aws cloudformation validate-template \
            --template-body file://"${template}" \
            --region "${REGION}" 2>&1 | sed 's/^/  /'
    fi
    
    echo ""
done

# Summary
echo -e "${GREEN}Validation Summary${NC}"
echo "=================="
echo "Total Templates: ${TOTAL_TEMPLATES}"
echo "Valid: ${VALID_TEMPLATES}"
echo "Invalid: $((TOTAL_TEMPLATES - VALID_TEMPLATES))"

if [ ${#INVALID_TEMPLATES[@]} -gt 0 ]; then
    echo -e "\n${RED}Invalid Templates:${NC}"
    for template in "${INVALID_TEMPLATES[@]}"; do
        echo -e "  ❌ ${template}"
    done
    exit 1
else
    echo -e "\n${GREEN}✅ All templates are valid!${NC}"
    exit 0
fi