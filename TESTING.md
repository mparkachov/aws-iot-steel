# AWS IoT Steel Testing Guide

This document describes the comprehensive testing system for the AWS IoT Steel project, which includes both Rust and Steel (Scheme) tests.

## Overview

The testing system provides dual-language testing capabilities:

1. **Rust Tests**: Traditional Rust integration tests using the `#[tokio::test]` framework
2. **Steel Tests**: Tests written in Steel (Scheme) that exercise the same functionality
3. **Examples**: Demonstration programs in Steel showing real-world usage

## Test Structure

```
aws-iot-core/
├── tests/
│   ├── steel/                    # Steel test files
│   │   ├── test_led.scm         # LED functionality tests
│   │   ├── test_sleep.scm       # Sleep functionality tests
│   │   ├── test_system_info.scm # System information tests
│   │   ├── test_logging.scm     # Logging functionality tests
│   │   └── test_integration.scm # Integration tests
│   ├── integration_led.rs       # Rust LED tests
│   ├── integration_sleep.rs     # Rust sleep tests
│   ├── integration_system_info.rs # Rust system info tests
│   ├── integration_logging.rs   # Rust logging tests
│   └── common/                  # Shared test utilities
│       └── mod.rs
├── examples/
│   └── steel/                   # Steel example programs
│       ├── blink_led.scm       # LED blinking example
│       ├── system_monitor.scm  # System monitoring example
│       └── interactive_demo.scm # Interactive demonstration
└── src/
    ├── bin/
    │   └── test_runner.rs      # Steel test runner binary
    └── steel_test_runner.rs    # Steel test runner library
```

## Running Tests

### Using Make (Recommended)

The project includes a Makefile with convenient commands:

```bash
# Run all tests (Rust + Steel)
make test

# Run only Rust tests
make test-rust

# Run only Steel tests
make test-steel

# Run all examples
make examples

# Run specific Steel tests
make test-steel-led
make test-steel-sleep
make test-steel-system
make test-steel-logging
make test-steel-integration

# Run specific examples
make example-blink
make example-monitor
make example-demo
```

### Using Cargo Directly

```bash
# Build the project
cargo build --workspace

# Run Rust tests
cargo test --package aws-iot-core --test integration_led
cargo test --package aws-iot-core --test integration_sleep
cargo test --package aws-iot-core --test integration_system_info
cargo test --package aws-iot-core --test integration_logging

# Run Steel tests
cargo run --package aws-iot-core --bin test_runner steel-tests

# Run Steel examples
cargo run --package aws-iot-core --bin test_runner steel-examples

# Run specific Steel test file
cargo run --package aws-iot-core --bin test_runner steel-test aws-iot-core/tests/steel/test_led.scm

# Run specific Steel example file
cargo run --package aws-iot-core --bin test_runner steel-example aws-iot-core/examples/steel/blink_led.scm
```

## Test Categories

### 1. LED Tests

**Rust**: `tests/integration_led.rs`
**Steel**: `tests/steel/test_led.scm`

Tests LED control functionality:
- Basic LED on/off operations
- LED state querying
- LED blinking sequences
- Error handling

### 2. Sleep Tests

**Rust**: `tests/integration_sleep.rs`
**Steel**: `tests/steel/test_sleep.scm`

Tests sleep functionality:
- Basic sleep operations
- Zero duration sleep
- Error handling for negative durations
- Timing accuracy
- Multiple sleep sequences

### 3. System Information Tests

**Rust**: `tests/integration_system_info.rs`
**Steel**: `tests/steel/test_system_info.scm`

Tests system information retrieval:
- Device information
- Memory usage statistics
- System uptime
- Integration scenarios

### 4. Logging Tests

**Rust**: `tests/integration_logging.rs`
**Steel**: `tests/steel/test_logging.scm`

Tests logging functionality:
- Different log levels (error, warn, info, debug)
- Message formatting
- Performance with multiple messages
- Special character handling

### 5. Integration Tests

**Steel**: `tests/steel/test_integration.scm`

Tests complex scenarios combining multiple features:
- LED control with system monitoring
- Error recovery scenarios
- Performance testing
- Real-world usage patterns

## Example Programs

### 1. LED Blink Example

**File**: `examples/steel/blink_led.scm`

Demonstrates basic LED control with configurable timing and repetition.

### 2. System Monitor Example

**File**: `examples/steel/system_monitor.scm`

Shows how to create a system monitoring application that:
- Displays system information in human-readable format
- Monitors system state over time
- Provides visual feedback via LED

### 3. Interactive Demo

**File**: `examples/steel/interactive_demo.scm`

Comprehensive demonstration including:
- Various LED patterns (fast blink, slow pulse, Morse code)
- System monitoring capabilities
- Error handling demonstrations
- Performance testing

## Writing New Tests

### Rust Tests

1. Create a new file in `tests/` directory
2. Use the common MockHAL from `tests/common/mod.rs`
3. Follow the existing test patterns
4. Use `#[tokio::test]` for async tests

Example:
```rust
use aws_iot_core::*;
use std::sync::Arc;
use tokio;

mod common;
use common::MockHAL;

#[tokio::test]
async fn test_new_functionality() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()).unwrap());
    
    // Your test code here
    let result = api.some_function().await;
    assert!(result.is_ok());
}
```

### Steel Tests

1. Create a new `.scm` file in `tests/steel/` directory
2. Use the Steel testing patterns
3. Include proper error handling and logging
4. Follow the existing test structure

Example:
```scheme
;; Steel test for new functionality

(define (test-new-feature)
  "Test description"
  (begin
    (log-info "Starting new feature test")
    
    ;; Test implementation
    (let ((result (some-function)))
      (if (expected-condition? result)
          (log-info "✓ Test passed")
          (error "Test failed")))
    
    (log-info "New feature test completed")
    #t))

(define (run-new-tests)
  "Run all new tests"
  (begin
    (log-info "=== Running New Feature Tests ===")
    (test-new-feature)
    (log-info "=== All new tests passed ===")
    #t))

;; Run the tests
(run-new-tests)
```

## Test Runner Architecture

The Steel test runner (`SteelTestRunner`) provides:

- **File Execution**: Run individual Steel test/example files
- **Directory Scanning**: Automatically discover and run all `.scm` files in a directory
- **Result Tracking**: Collect pass/fail results with detailed error messages
- **Summary Reporting**: Generate comprehensive test result summaries

## Mock HAL

Both Rust and Steel tests use a mock Hardware Abstraction Layer (HAL) that:

- Simulates hardware operations without real hardware
- Tracks operation calls for verification
- Provides consistent test data
- Enables fast, reliable testing

## Continuous Integration

The testing system is designed for CI/CD integration:

```bash
# CI command that runs all tests
make ci
```

This command:
1. Builds the entire workspace
2. Runs all Rust tests
3. Runs all Steel tests
4. Reports overall success/failure

## Performance Considerations

- Steel tests include performance benchmarks
- Mock HAL uses minimal delays for realistic timing
- Test runner provides execution time reporting
- Large test suites can be run in parallel (Rust tests)

## Troubleshooting

### Common Issues

1. **Steel syntax errors**: Check parentheses balancing and string literals
2. **File not found**: Ensure file paths are relative to workspace root
3. **Test timeouts**: Check for infinite loops in Steel code
4. **Mock HAL state**: Reset HAL state between tests if needed

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug make test-steel
```

### Individual Test Debugging

Run single tests for focused debugging:
```bash
make test-steel-led
cargo test --package aws-iot-core --test integration_led -- --nocapture
```

## Future Enhancements

- Property-based testing integration
- Performance regression testing
- Hardware-in-the-loop testing support
- Test coverage reporting for Steel code
- Automated test generation from specifications