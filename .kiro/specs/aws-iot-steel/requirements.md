# Requirements Document

## Introduction

This project implements an ESP32-S3 embedded module that combines Rust for system-level operations with Steel (Scheme) for scripting capabilities. The module connects to AWS IoT Core using shadow functionality to receive and execute commands, with over-the-air update capabilities. The system is designed for extensible command implementation and includes comprehensive testing and CI/CD infrastructure.

## Requirements

### Requirement 1

**User Story:** As a developer, I want to create a cross-platform Rust application that can simulate ESP32 commands on macOS for development and testing, so that I can develop and test the system without requiring physical hardware.

#### Acceptance Criteria

1. WHEN the application is compiled and run on macOS THEN the system SHALL simulate sleep and LED commands by printing to stdout
2. WHEN a sleep command is executed THEN the system SHALL print the sleep duration and simulate the waiting period
3. WHEN an LED command is executed THEN the system SHALL print the LED state change (on/off)
4. WHEN the application starts THEN the system SHALL initialize both Rust and Steel runtime environments

### Requirement 2

**User Story:** As a developer, I want to integrate Steel (Scheme) scripting capabilities with Rust, so that commands can be executed through a Scheme interface while maintaining system-level control in Rust.

#### Acceptance Criteria

1. WHEN Steel runtime is initialized THEN the system SHALL expose sleep and LED commands as Scheme functions
2. WHEN `(sleep 5)` is called in Steel THEN the system SHALL execute a 5-second sleep operation
3. WHEN `(led "on")` or `(led "off")` is called in Steel THEN the system SHALL toggle the LED state accordingly
4. WHEN Steel programs are executed THEN the system SHALL handle errors gracefully and return appropriate status codes
5. WHEN new commands are added THEN the system SHALL automatically expose them to the Steel API without requiring Steel-specific code changes

### Requirement 3

**User Story:** As a system operator, I want the module to connect to AWS IoT Core and use shadow functionality, so that I can remotely control and monitor the device state.

#### Acceptance Criteria

1. WHEN the module starts THEN the system SHALL establish a secure connection to AWS IoT Core using device certificates
2. WHEN the module connects to AWS IoT THEN the system SHALL subscribe to shadow update topics
3. WHEN a shadow update contains Steel program code THEN the system SHALL download and execute the program
4. WHEN commands are executed THEN the system SHALL update the device shadow with current state information
5. WHEN the connection is lost THEN the system SHALL attempt to reconnect with exponential backoff

### Requirement 4

**User Story:** As a system operator, I want the device shadow to reflect the current module state, so that I can monitor sleep status, LED state, and command execution progress remotely.

#### Acceptance Criteria

1. WHEN the module is sleeping THEN the shadow SHALL report sleep status as "sleeping" and include wake-up time
2. WHEN the module is awake THEN the shadow SHALL report sleep status as "awake"
3. WHEN the LED state changes THEN the shadow SHALL immediately reflect the new LED status (on/off)
4. WHEN a command is executing THEN the shadow SHALL report command execution status and progress
5. WHEN an error occurs THEN the shadow SHALL report error details and timestamp

### Requirement 5

**User Story:** As a developer, I want comprehensive unit and integration tests, so that I can ensure system reliability and catch regressions during development.

#### Acceptance Criteria

1. WHEN `cargo test` is executed THEN all unit tests SHALL pass and cover sleep, LED, and Steel integration functionality
2. WHEN integration tests run THEN the system SHALL start a test process, connect to AWS IoT, send Steel programs, and verify expected outcomes
3. WHEN testing command extensibility THEN tests SHALL verify that new commands can be added without breaking existing functionality
4. WHEN testing error conditions THEN tests SHALL verify proper error handling for network failures, invalid commands, and Steel runtime errors
5. WHEN testing AWS IoT integration THEN tests SHALL use mock or test AWS IoT endpoints to avoid production dependencies

### Requirement 6

**User Story:** As a DevOps engineer, I want AWS infrastructure defined as CloudFormation templates, so that I can deploy and manage the IoT infrastructure consistently and reproducibly.

#### Acceptance Criteria

1. WHEN CloudFormation templates are deployed THEN the system SHALL create AWS IoT Core things, certificates, and policies
2. WHEN infrastructure is deployed THEN the system SHALL configure IoT device shadows with appropriate permissions
3. WHEN using AWS CLI THEN the infrastructure SHALL be deployable with a single command
4. WHEN infrastructure is updated THEN the system SHALL support incremental updates without data loss
5. WHEN infrastructure is torn down THEN the system SHALL clean up all created resources

### Requirement 7

**User Story:** As a developer, I want over-the-air update capabilities, so that I can deploy firmware updates to ESP32 modules remotely without physical access.

#### Acceptance Criteria

1. WHEN a firmware update is available THEN the module SHALL detect and download the update securely
2. WHEN downloading updates THEN the system SHALL verify cryptographic signatures before installation
3. WHEN an update is applied THEN the system SHALL maintain rollback capability in case of failure
4. WHEN update process fails THEN the system SHALL revert to the previous working firmware version
5. WHEN update is successful THEN the system SHALL report new firmware version to the device shadow

### Requirement 8

**User Story:** As a developer, I want the system architecture to support easy addition of new commands, so that I can extend functionality without modifying core system components.

#### Acceptance Criteria

1. WHEN a new command is implemented THEN it SHALL follow a standard trait interface
2. WHEN commands are registered THEN the system SHALL automatically expose them to both Steel and IoT shadow interfaces
3. WHEN adding commands THEN no changes SHALL be required to Steel runtime or AWS IoT integration code
4. WHEN commands have different parameter types THEN the system SHALL handle serialization/deserialization automatically
5. WHEN commands have async operations THEN the system SHALL support both synchronous and asynchronous command execution

### Requirement 9

**User Story:** As a security-conscious developer, I want secure key management and credential handling, so that device certificates and AWS credentials are protected and never exposed in the codebase.

#### Acceptance Criteria

1. WHEN the ESP32 module stores IoT certificates THEN the system SHALL use the device's secure element or encrypted flash storage
2. WHEN AWS IoT device certificates are generated THEN they SHALL never be committed to version control
3. WHEN local development uses AWS credentials THEN the system SHALL read credentials from AWS CLI configuration or environment variables only
4. WHEN the application accesses AWS services THEN it SHALL use separate IAM roles with minimal required permissions for CI/CD and device IoT operations
5. WHEN CI/CD pipeline accesses AWS THEN it SHALL use a dedicated IAM role with permissions limited to: CloudFormation stack management, IoT device provisioning, S3 firmware upload, and OTA job creation
6. WHEN IoT devices connect to AWS THEN they SHALL use a separate IAM role with permissions limited to: IoT Core publish/subscribe on device-specific topics, shadow read/write access, and OTA firmware download
7. WHEN certificates are provisioned THEN the system SHALL support secure certificate injection during manufacturing or initial setup
8. WHEN storing sensitive data THEN the system SHALL encrypt all credentials at rest using hardware security features
9. WHEN transmitting data THEN the system SHALL use TLS 1.3 or higher for all AWS IoT communications

### Requirement 10

**User Story:** As a developer, I want CI/CD pipeline integration, so that cross-compilation for ESP32-S3 and deployment can be automated.

#### Acceptance Criteria

1. WHEN code is committed THEN the CI/CD pipeline SHALL automatically cross-compile for ESP32-S3 target
2. WHEN cross-compilation succeeds THEN the system SHALL run all tests including integration tests
3. WHEN tests pass THEN the pipeline SHALL build firmware images ready for OTA deployment
4. WHEN building for ESP32 THEN the system SHALL use esp-rs toolchain and verify compatibility with ESP32-S3 hardware
5. WHEN deployment is triggered THEN the system SHALL update AWS IoT with new firmware versions for OTA distribution
6. WHEN CI/CD handles credentials THEN the system SHALL use secure secret management and never log sensitive information