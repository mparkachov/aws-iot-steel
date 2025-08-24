# AWS IoT Steel Documentation

This directory contains comprehensive documentation for the AWS IoT Steel project.

## 📚 Documentation Index

### Core Documentation
- **[Main README](../README.md)** - Project overview, quick start, and basic usage
- **[Testing Guide](TESTING.md)** - Comprehensive testing documentation
- **[Changelog](CHANGELOG.md)** - Version history and changes

### Development & CI/CD
- **[GitHub Actions Setup](github-actions-setup.md)** - CI/CD pipeline configuration
- **[Artifact Management](ARTIFACT_MANAGEMENT.md)** - Build artifact handling

### Production Documentation
- **[Production Deployment](../production/README.md)** - Production deployment package
- **[Steel Programming Guide](../production/docs/steel-programming-guide.md)** - Steel language programming
- **[Production Deployment Guide](../production/docs/production-deployment-guide.md)** - Deployment procedures
- **[Disaster Recovery Plan](../production/docs/disaster-recovery-plan.md)** - Recovery procedures

### Infrastructure
- **[AWS Infrastructure](../aws-infrastructure/README.md)** - CloudFormation templates and AWS setup

### Platform-Specific
- **[ESP32 Platform](../aws-iot-platform-esp32/README.md)** - ESP32-C3-DevKit-RUST-1 specific documentation

## 🏗️ Architecture Overview

The AWS IoT Steel system consists of:

1. **Core Library** (`aws-iot-core`) - Main interfaces and Steel runtime
2. **Platform Implementations** - macOS, Linux, and ESP32 HAL implementations  
3. **Test Suite** (`aws-iot-tests`) - Comprehensive dual testing (Rust + Steel)
4. **AWS Infrastructure** - CloudFormation templates and deployment scripts
5. **Production Package** - Ready-to-deploy production configuration

## 🚀 Quick Navigation

### For Developers
- [Getting Started](../README.md#getting-started)
- [Testing](TESTING.md)
- [Contributing](../README.md#contributing)

### For DevOps Engineers
- [CI/CD Setup](github-actions-setup.md)
- [Production Deployment](../production/README.md)
- [Infrastructure](../aws-infrastructure/README.md)

### For Steel Programmers
- [Steel Programming Guide](../production/docs/steel-programming-guide.md)
- [Steel Examples](../examples/steel/)
- [Steel API Documentation](../aws-iot-core/src/steel_api_documentation.rs)

## 📖 Additional Resources

- **[Project Specifications](.kiro/specs/aws-iot-steel/)** - Detailed requirements and design
- **[Examples](../examples/)** - Code examples and demonstrations
- **[Scripts](../scripts/)** - Utility scripts for development and deployment