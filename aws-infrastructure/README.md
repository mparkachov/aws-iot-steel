# AWS Infrastructure for ESP32-Steel Project

This directory contains CloudFormation templates and deployment scripts for the ESP32-Steel project's AWS infrastructure.

## Overview

The infrastructure is designed with security and least-privilege access in mind, supporting:

- **Core IoT Infrastructure**: Thing types, policies, and logging
- **Secure S3 Storage**: Encrypted buckets for firmware and Steel programs
- **Lambda Functions**: Pre-signed URL generation for secure downloads
- **IAM Roles**: Separate roles for CI/CD and device operations

## Directory Structure

```
aws-infrastructure/
├── cloudformation/
│   ├── core-iot-infrastructure.yaml    # Core IoT resources
│   └── s3-lambda-infrastructure.yaml   # S3 buckets and Lambda functions
├── scripts/
│   ├── deploy-core-infrastructure.sh   # Deploy core IoT infrastructure
│   ├── deploy-s3-lambda.sh            # Deploy S3 and Lambda resources
│   ├── provision-device.sh             # Provision individual devices
│   └── cleanup-infrastructure.sh       # Clean up all resources
├── outputs/                            # Stack outputs (auto-generated)
├── certificates/                       # Device certificates (auto-generated)
└── README.md                          # This file
```

## Prerequisites

1. **AWS CLI**: Installed and configured with appropriate credentials
2. **jq**: JSON processor for parsing AWS CLI outputs
3. **curl**: For downloading Amazon Root CA certificates

### AWS Permissions Required

Your AWS credentials need the following permissions:
- CloudFormation: Full access for stack management
- IoT: Full access for thing, policy, and certificate management
- IAM: Create and manage roles and policies
- S3: Create and manage buckets
- Lambda: Create and manage functions
- CloudWatch Logs: Create and manage log groups

## Quick Start

### 1. Deploy Core IoT Infrastructure

```bash
# Deploy to development environment
./scripts/deploy-core-infrastructure.sh dev us-west-2

# Deploy to production environment
./scripts/deploy-core-infrastructure.sh prod us-west-2
```

### 2. Deploy S3 and Lambda Infrastructure

```bash
# Deploy S3 buckets and Lambda functions
./scripts/deploy-s3-lambda.sh dev us-west-2
```

### 3. Provision a Device

```bash
# Provision a new device
./scripts/provision-device.sh device-001 dev us-west-2
```

## Infrastructure Components

### Core IoT Infrastructure

**Resources Created:**
- IoT Thing Type for ESP32-Steel devices
- IoT Policy with minimal required permissions
- CloudWatch Log Groups for monitoring
- Development certificate and thing (dev environment only)

**Security Features:**
- Device-specific topic permissions
- Shadow access limited to device's own shadow
- Separate log groups for different types of logs

### S3 and Lambda Infrastructure

**Resources Created:**
- Encrypted S3 buckets for firmware and Steel programs
- Lambda function for pre-signed URL generation
- IAM roles with least-privilege access
- Bucket policies denying public access

**Security Features:**
- All S3 buckets are private with encryption at rest
- Pre-signed URLs with short expiration (15 minutes)
- Separate IAM roles for CI/CD and devices
- HTTPS-only bucket policies

## Device Provisioning

The `provision-device.sh` script creates:

1. **IoT Thing** with appropriate attributes
2. **Certificate and Keys** for device authentication
3. **Policy Attachment** for device permissions
4. **Configuration File** with connection details

### Generated Files

For each device, the following files are created in `certificates/{environment}/`:

- `{device-id}-certificate.pem`: Device certificate
- `{device-id}-private.key`: Private key (600 permissions)
- `{device-id}-public.key`: Public key
- `{device-id}-config.json`: Device configuration
- `AmazonRootCA1.pem`: Amazon Root CA certificate

## Environment Management

### Supported Environments

- **dev**: Development environment with relaxed policies
- **staging**: Staging environment for testing
- **prod**: Production environment with strict security

### Environment Isolation

Each environment creates separate:
- CloudFormation stacks
- IoT resources (things, policies, certificates)
- S3 buckets
- CloudWatch log groups

## Security Considerations

### Certificate Management

- **Development**: Certificates created automatically by CloudFormation
- **Production**: Use AWS IoT Device Management or custom provisioning
- **Storage**: Private keys have restricted file permissions (600)
- **Rotation**: Implement certificate rotation for production use

### Access Control

- **Device Policy**: Minimal permissions for device operations only
- **CI/CD Role**: Upload permissions to S3, no device access
- **Lambda Role**: Read-only access to S3 for URL generation

### Network Security

- **TLS 1.3**: All IoT communications use TLS 1.3
- **HTTPS Only**: S3 bucket policies enforce HTTPS
- **Private Buckets**: No public access allowed

## Monitoring and Logging

### CloudWatch Log Groups

- `/aws/iot/{project}-{env}/devices`: Device operational logs
- `/aws/iot/{project}-{env}/steel-programs`: Steel program execution logs
- `/aws/iot/{project}-{env}/system-monitoring`: System monitoring logs

### Retention Policies

- Device logs: 30 days
- Steel program logs: 14 days
- System monitoring: 7 days

## Troubleshooting

### Common Issues

1. **Stack Creation Fails**
   - Check AWS credentials and permissions
   - Verify template syntax with `aws cloudformation validate-template`
   - Check CloudFormation events in AWS Console

2. **Device Provisioning Fails**
   - Ensure core infrastructure is deployed first
   - Check if thing type and policy exist
   - Verify AWS CLI configuration

3. **Certificate Issues**
   - Ensure certificates directory has proper permissions
   - Check if certificate is active in IoT Console
   - Verify policy attachment

### Useful Commands

```bash
# Check stack status
aws cloudformation describe-stacks --stack-name esp32-steel-dev-core-iot

# List IoT things
aws iot list-things --thing-type-name esp32-steel-dev-thing-type

# Check certificate status
aws iot describe-certificate --certificate-id <certificate-id>

# View CloudWatch logs
aws logs describe-log-groups --log-group-name-prefix "/aws/iot/esp32-steel"
```

## Cleanup

To remove all infrastructure:

```bash
# Clean up all resources for an environment
./scripts/cleanup-infrastructure.sh dev us-west-2
```

**Warning**: This will permanently delete all resources and data. Use with caution.

## Cost Optimization

### Free Tier Usage

- IoT Core: 2.25 million messages/month free
- Lambda: 1 million requests/month free
- CloudWatch Logs: 5GB ingestion/month free

### Cost Monitoring

- Set up CloudWatch billing alarms
- Monitor S3 storage usage
- Review IoT message usage regularly

## Next Steps

1. **Deploy Infrastructure**: Use the deployment scripts
2. **Provision Devices**: Create certificates for your devices
3. **Test Connectivity**: Verify devices can connect to IoT Core
4. **Implement Monitoring**: Set up CloudWatch dashboards
5. **Security Review**: Audit permissions and policies