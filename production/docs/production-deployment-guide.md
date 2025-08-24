# Production Deployment Guide
## AWS IoT Steel System

### Document Information
- **Version**: 1.0.0
- **Last Updated**: 2024-01-01
- **Audience**: DevOps Engineers, System Administrators
- **Prerequisites**: AWS CLI, Rust toolchain, Docker

---

## Overview

This guide provides step-by-step instructions for deploying the AWS IoT Steel System to production environments. The system enables dynamic Steel (Scheme) program execution on ESP32-C3-DevKit-RUST-1 devices through AWS IoT Core.

### Architecture Summary
- **Devices**: ESP32-C3 modules with Rust runtime and Steel interpreter
- **Cloud**: AWS IoT Core, Lambda, S3, CloudFormation, CodePipeline
- **Languages**: Rust (system runtime), Steel/Scheme (application programs)
- **Security**: TLS 1.3, certificate-based authentication, encrypted storage

---

## Prerequisites

### Required Tools
```bash
# Install AWS CLI
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip
sudo ./aws/install

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install ESP32 toolchain (for hardware deployment)
cargo install espup
espup install
```

### AWS Account Setup
```bash
# Configure AWS credentials
aws configure

# Verify access
aws sts get-caller-identity

# Set required environment variables
export AWS_REGION=us-west-2
export DEPLOYMENT_BUCKET=aws-iot-steel-deployments-$(date +%s)
export ENVIRONMENT=production
```

### Permissions Required
- CloudFormation: Full access
- IoT Core: Full access
- S3: Full access
- Lambda: Full access
- CodePipeline: Full access
- IAM: Role and policy management

---

## Pre-Deployment Checklist

### Infrastructure Readiness
- [ ] AWS account configured with appropriate permissions
- [ ] S3 bucket for deployments created or accessible
- [ ] Domain/subdomain available for custom endpoints (optional)
- [ ] SSL certificates available (if using custom domains)
- [ ] Monitoring and alerting SNS topics configured

### Security Readiness
- [ ] Device certificates generated or CA configured
- [ ] Encryption keys generated and stored securely
- [ ] IAM roles and policies reviewed and approved
- [ ] Security groups and network ACLs configured
- [ ] Audit logging enabled

### Operational Readiness
- [ ] Monitoring dashboards configured
- [ ] Alerting rules defined and tested
- [ ] Backup procedures documented and tested
- [ ] Disaster recovery plan reviewed
- [ ] Team trained on operational procedures

---

## Deployment Steps

### Step 1: Environment Preparation

```bash
# Clone the repository
git clone https://github.com/your-org/aws-iot-steel.git
cd aws-iot-steel

# Checkout the production release
git checkout v1.0.0

# Verify the build
make ci
```

### Step 2: Configuration

```bash
# Copy and customize production configuration
cp production/config/production.toml production/config/production-custom.toml

# Edit configuration file
vim production/config/production-custom.toml

# Key settings to customize:
# - device_count: Number of devices to provision
# - aws.region: Target AWS region
# - monitoring.alerting_enabled: Enable/disable alerting
# - security settings: Certificate paths and encryption settings
```

### Step 3: Infrastructure Deployment

```bash
# Run the production deployment script
./production/scripts/deploy-production.sh \
  --environment production \
  --region us-west-2 \
  --device-count 100 \
  --bucket aws-iot-steel-deployments

# Monitor deployment progress
aws cloudformation describe-stacks \
  --stack-name aws-iot-steel-production-core \
  --region us-west-2
```

### Step 4: Validation

```bash
# Run end-to-end validation
make validate-production

# Verify specific components
make validate-e2e
make validate-security
make validate-load

# Check infrastructure health
aws cloudformation describe-stacks \
  --region us-west-2 \
  --query 'Stacks[?contains(StackName, `aws-iot-steel-production`)].{Name:StackName,Status:StackStatus}'
```

### Step 5: Device Provisioning

```bash
# Provision devices (automated)
for i in {1..100}; do
  device_id="device-$(printf "%03d" $i)"
  ./aws-infrastructure/scripts/provision-device.sh \
    "$device_id" production us-west-2
done

# Verify device provisioning
aws iot list-things --max-items 10
```

### Step 6: Monitoring Setup

```bash
# Deploy CloudWatch dashboard
aws cloudwatch put-dashboard \
  --dashboard-name "AWS-IoT-Steel-Production" \
  --dashboard-body file://production/monitoring/cloudwatch-dashboard.json

# Deploy CloudWatch alarms
aws cloudformation deploy \
  --template-file production/monitoring/cloudwatch-alarms.yaml \
  --stack-name aws-iot-steel-production-alarms \
  --parameter-overrides \
    Environment=production \
    SNSTopicArn=arn:aws:sns:us-west-2:123456789012:aws-iot-steel-alerts \
    DeviceCount=100
```

---

## Post-Deployment Configuration

### Device Configuration

```bash
# Generate device configuration files
./production/scripts/generate-device-configs.sh \
  --environment production \
  --output-dir ./device-configs

# Example device configuration (device-001.toml)
[device]
device_id = "device-001"
thing_name = "aws-iot-steel-production-device-001"

[aws]
region = "us-west-2"
iot_endpoint = "a1b2c3d4e5f6g7-ats.iot.us-west-2.amazonaws.com"
certificate_path = "/etc/aws-iot-steel/certs/device-001.pem.crt"
private_key_path = "/etc/aws-iot-steel/certs/device-001.pem.key"
```

### Steel Program Deployment

```bash
# Deploy sample Steel programs
aws s3 cp examples/steel/ \
  s3://aws-iot-steel-programs-production/examples/ \
  --recursive

# Test program delivery
./production/scripts/test-program-delivery.sh \
  --device-id device-001 \
  --program examples/steel/blink_led.scm
```

### Monitoring Configuration

```bash
# Configure custom metrics
aws logs create-log-group \
  --log-group-name /aws/iot/steel-runtime

# Set up log retention
aws logs put-retention-policy \
  --log-group-name /aws/iot/steel-runtime \
  --retention-in-days 30
```

---

## Operational Procedures

### Daily Operations

#### Health Checks
```bash
# Check system health
aws cloudwatch get-metric-statistics \
  --namespace AWS/IoT \
  --metric-name Connect.Success \
  --start-time $(date -u -d '1 hour ago' +%Y-%m-%dT%H:%M:%S) \
  --end-time $(date -u +%Y-%m-%dT%H:%M:%S) \
  --period 3600 \
  --statistics Sum

# Verify device connectivity
./production/scripts/check-device-health.sh
```

#### Log Monitoring
```bash
# Check for errors in the last hour
aws logs filter-log-events \
  --log-group-name /aws/iot/logsV2 \
  --start-time $(date -d '1 hour ago' +%s)000 \
  --filter-pattern "ERROR"
```

### Weekly Operations

#### Performance Review
```bash
# Generate performance report
./production/scripts/generate-performance-report.sh \
  --start-date $(date -d '7 days ago' +%Y-%m-%d) \
  --end-date $(date +%Y-%m-%d)
```

#### Security Audit
```bash
# Check certificate expiration
./production/scripts/check-certificate-expiration.sh

# Review access logs
aws s3api get-bucket-logging \
  --bucket aws-iot-steel-programs-production
```

### Monthly Operations

#### Capacity Planning
```bash
# Analyze usage trends
./production/scripts/analyze-usage-trends.sh \
  --period 30days

# Review resource utilization
aws cloudwatch get-metric-statistics \
  --namespace Custom/SteelRuntime \
  --metric-name MemoryUsage \
  --start-time $(date -u -d '30 days ago' +%Y-%m-%dT%H:%M:%S) \
  --end-time $(date -u +%Y-%m-%dT%H:%M:%S) \
  --period 86400 \
  --statistics Average,Maximum
```

#### Backup Verification
```bash
# Verify backup integrity
./production/scripts/verify-backups.sh

# Test disaster recovery procedures
./production/scripts/test-dr-procedures.sh --dry-run
```

---

## Troubleshooting

### Common Issues

#### Device Connection Failures
```bash
# Check device certificates
aws iot describe-certificate --certificate-id CERTIFICATE_ID

# Verify IoT policies
aws iot get-policy --policy-name DevicePolicy

# Check CloudWatch logs
aws logs filter-log-events \
  --log-group-name /aws/iot/logsV2 \
  --filter-pattern "{ $.eventType = \"CONNECT\" && $.errorMessage EXISTS }"
```

#### Steel Program Execution Failures
```bash
# Check program validation logs
aws logs filter-log-events \
  --log-group-name /aws/lambda/steel-program-validator \
  --filter-pattern "ERROR"

# Verify program syntax
cargo run --bin steel_test -- --file path/to/program.scm
```

#### High Resource Usage
```bash
# Check memory usage
aws cloudwatch get-metric-statistics \
  --namespace Custom/SteelRuntime \
  --metric-name MemoryUsage \
  --start-time $(date -u -d '1 hour ago' +%Y-%m-%dT%H:%M:%S) \
  --end-time $(date -u +%Y-%m-%dT%H:%M:%S) \
  --period 300 \
  --statistics Average,Maximum

# Identify resource-intensive programs
./production/scripts/identify-resource-intensive-programs.sh
```

### Emergency Procedures

#### Service Degradation
1. Check AWS Service Health Dashboard
2. Review CloudWatch alarms
3. Implement temporary workarounds
4. Escalate to AWS Support if needed

#### Security Incident
1. Isolate affected devices
2. Revoke compromised certificates
3. Review access logs
4. Follow incident response procedures

---

## Scaling Considerations

### Horizontal Scaling

#### Adding More Devices
```bash
# Provision additional devices
./production/scripts/provision-devices.sh \
  --start-id 101 \
  --count 50 \
  --environment production

# Update monitoring thresholds
aws cloudformation update-stack \
  --stack-name aws-iot-steel-production-alarms \
  --use-previous-template \
  --parameters ParameterKey=DeviceCount,ParameterValue=150
```

#### Multi-Region Deployment
```bash
# Deploy to additional region
./production/scripts/deploy-production.sh \
  --environment production \
  --region eu-west-1 \
  --device-count 50

# Configure cross-region replication
aws s3api put-bucket-replication \
  --bucket aws-iot-steel-programs-production \
  --replication-configuration file://replication-config.json
```

### Vertical Scaling

#### Lambda Function Optimization
```bash
# Increase memory allocation
aws lambda update-function-configuration \
  --function-name steel-program-validator \
  --memory-size 512

# Configure provisioned concurrency
aws lambda put-provisioned-concurrency-config \
  --function-name steel-program-validator \
  --provisioned-concurrency-config ProvisionedConcurrencyCount=10
```

---

## Maintenance Windows

### Planned Maintenance

#### Monthly Updates
- **Schedule**: First Sunday of each month, 2:00 AM UTC
- **Duration**: 2 hours
- **Scope**: Security patches, minor updates

#### Quarterly Upgrades
- **Schedule**: First Sunday of each quarter, 2:00 AM UTC
- **Duration**: 4 hours
- **Scope**: Major version updates, infrastructure changes

### Maintenance Procedures

#### Pre-Maintenance
```bash
# Create system snapshot
./production/scripts/create-system-snapshot.sh

# Notify stakeholders
./production/scripts/send-maintenance-notification.sh \
  --start-time "2024-01-07T02:00:00Z" \
  --duration "2 hours"
```

#### During Maintenance
```bash
# Apply updates
./production/scripts/apply-updates.sh \
  --environment production \
  --maintenance-window

# Monitor system health
./production/scripts/monitor-maintenance.sh
```

#### Post-Maintenance
```bash
# Validate system functionality
make validate-production

# Send completion notification
./production/scripts/send-maintenance-completion.sh
```

---

## Security Best Practices

### Certificate Management
- Rotate device certificates annually
- Use AWS IoT Device Management for certificate lifecycle
- Monitor certificate expiration dates
- Implement automated certificate renewal where possible

### Access Control
- Follow principle of least privilege
- Regularly review IAM policies and roles
- Use AWS CloudTrail for audit logging
- Implement multi-factor authentication for administrative access

### Data Protection
- Encrypt data in transit and at rest
- Use AWS KMS for key management
- Implement secure key rotation
- Regular security assessments and penetration testing

---

## Support and Escalation

### Internal Support
- **Level 1**: DevOps team (24/7 on-call rotation)
- **Level 2**: Engineering team (business hours)
- **Level 3**: Architecture team (escalation only)

### External Support
- **AWS Support**: Business or Enterprise plan recommended
- **Community**: GitHub issues and discussions
- **Documentation**: Internal wiki and runbooks

### Escalation Procedures
1. **P1 (Critical)**: Immediate escalation to on-call engineer
2. **P2 (High)**: Escalation within 2 hours
3. **P3 (Medium)**: Escalation within 8 hours
4. **P4 (Low)**: Escalation within 24 hours

---

## Appendices

### Appendix A: Configuration Reference
See `production/config/` directory for complete configuration examples.

### Appendix B: API Reference
See `docs/api-reference.md` for Steel API documentation.

### Appendix C: Troubleshooting Scripts
See `production/scripts/troubleshooting/` directory for diagnostic tools.

### Appendix D: Performance Benchmarks
See `docs/performance-benchmarks.md` for baseline performance metrics.

---

*This document is maintained by the DevOps team and updated with each major release.*