# AWS IoT Steel Project Specification

This directory contains the complete specification for the AWS IoT Steel project, including requirements, design, and implementation tasks.

## Document Structure

### Requirements Documents

The project requirements are split into two focused documents:

- **[functional-requirements.md](functional-requirements.md)**: Device functionality and system behavior requirements (11 requirements)
  - Cross-platform development support (macOS, Linux, ESP32)
  - Steel (Scheme) scripting integration
  - AWS IoT Core connectivity and shadow functionality
  - Security and certificate management
  - Testing and validation requirements
  - Linux platform enhanced system monitoring

- **[build-requirements.md](build-requirements.md)**: CI/CD, build, and deployment requirements (10 requirements)
  - Hybrid GitHub Actions + AWS CodePipeline CI/CD
  - Cross-platform compilation support
  - Code quality and security checks
  - Automated testing across platforms
  - Secure artifact management and signing
  - AWS infrastructure deployment
  - Build monitoring and reporting

### Design Document

- **[design.md](design.md)**: Comprehensive system design including:
  - Architecture overview with multi-platform support
  - Component interfaces and implementations
  - Data models and AWS integration
  - Security model and error handling
  - Testing strategy and platform-specific details

### Implementation Plan

- **[tasks.md](tasks.md)**: Detailed implementation tasks organized by major components:
  - ✅ Project structure and core interfaces
  - ✅ Cross-platform simulation (macOS, Linux, ESP32)
  - ✅ Steel runtime integration
  - ✅ Comprehensive Rust API layer
  - ✅ AWS IoT integration
  - ✅ Security and certificate management
  - ✅ Dual testing suite (Rust + Steel)
  - ✅ AWS infrastructure with CloudFormation
  - ✅ Over-the-air update system
  - ✅ ESP32-C3-DevKit-RUST-1 platform
  - 🔄 Hybrid CI/CD pipeline (in progress)
  - ⏳ Development tools and documentation
  - ⏳ Final integration and validation

## Key Features

### Multi-Platform Support
- **ESP32-C3-DevKit-RUST-1**: Production hardware platform
- **macOS**: Development platform with keychain integration
- **Linux**: CI/CD platform with enhanced system monitoring via /proc filesystem

### Steel (Scheme) Integration
- Dynamic program delivery via AWS IoT MQTT
- Comprehensive Rust API exposed to Steel
- Dual testing approach (Rust + Steel tests)
- Program packaging and distribution system

### Security Model
- Cryptographic firmware signing (RSA-PSS-SHA256)
- Secure certificate management
- TLS 1.3 communications
- Hardware security features on ESP32

### CI/CD Architecture
- **GitHub Actions**: Cross-compilation, testing, artifact signing
- **AWS CodePipeline**: Infrastructure deployment, OTA distribution
- Secure artifact management with comprehensive validation

## Current Status

The project is in active development with most core functionality complete:

- ✅ **Core Platform Implementation**: All three platforms (ESP32, macOS, Linux) implemented
- ✅ **Steel Runtime**: Full integration with comprehensive API
- ✅ **AWS Integration**: IoT Core, shadows, program delivery
- ✅ **Security**: Certificate management, secure communications
- ✅ **Testing**: Dual test suites with 44 validation checks
- ✅ **Artifact Management**: Secure signing, packaging, validation
- 🔄 **CI/CD Pipeline**: GitHub Actions complete, AWS CodePipeline in progress

## Getting Started

1. **Development Setup**: Use macOS or Linux for development
2. **Testing**: Run `make test-all` for comprehensive testing
3. **Building**: Use GitHub Actions for cross-compilation and artifact creation
4. **Deployment**: AWS CodePipeline handles infrastructure and OTA updates

For detailed implementation guidance, see the individual specification documents and the comprehensive [ARTIFACT_MANAGEMENT.md](../../../ARTIFACT_MANAGEMENT.md) documentation.