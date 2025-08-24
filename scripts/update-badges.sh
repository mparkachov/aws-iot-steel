#!/bin/bash

# Script to update GitHub badges with the correct repository URL
# Usage: ./scripts/update-badges.sh <github-username> <repository-name>

if [ $# -ne 2 ]; then
    echo "Usage: $0 <github-username> <repository-name>"
    echo "Example: $0 myusername aws-iot-steel"
    exit 1
fi

USERNAME=$1
REPO_NAME=$2

echo "Updating badges for repository: $USERNAME/$REPO_NAME"

# Update README.md badges
sed -i.bak "s/your-org/$USERNAME/g" README.md
sed -i.bak "s/aws-iot-steel/$REPO_NAME/g" README.md

# Update Cargo.toml repository URL
sed -i.bak "s/your-org/$USERNAME/g" Cargo.toml
sed -i.bak "s/aws-iot-steel/$REPO_NAME/g" Cargo.toml

echo "✅ Updated badges and repository URLs"
echo "📝 Please review the changes and commit them"
echo ""
echo "Files updated:"
echo "- README.md"
echo "- Cargo.toml"
echo ""
echo "Backup files created with .bak extension"