# AWS IoT Steel Dual Testing Infrastructure

This document describes the comprehensive dual testing approach for the AWS IoT Steel module, which combines Rust and Steel (Scheme) tests to ensure system reliability from both implementation and user perspectives.

## Overview

The AWS IoT Steel module uses a **dual testing infrastructure** that validates functionality at multiple levels:

1. **Rust Tests** - Test the implementation layer (HAL, APIs, runtime)
2. **Steel Tests** - Test the user-facing Steel/Scheme interface
3. **Integration Tests** - Test end-to-end system functionality
4. **Performance Tests** - Benchmark critical operations

## Test Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Dual Testing Infrastructure              │
├─────────────────────────────────────────────────────────────┤
│  Rust Tests                    │  Steel Tests               │
│  ├── Unit Tests                │  ├── API Tests             │
│  ├── Integration Tests         │  ├── Functional Tests      │
│  ├── Mock Components           │  ├── Complex Scenarios     │
│  └── Performance Benchmarks    │  └── Example Programs      │
├─────────────────────────────────────────────────────────────┤
│                    Test Aggregation & Reporting             │
│  ├── JSON Reports              │  ├── HTML Reports          │
│  ├── Markdown Reports          │  └── CI/CD Integration     │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Run All Tests
```bash
make test-all          # Run complete test suite
make test              # Alias for test-all
```

### Run Specific Test Types
```bash
make test-rust         # Rust unit + integration tests
make test-steel        # Steel test suite
make test-unit         # Rust unit tests only
make test-integration  # Rust integration tests only
```

### Run Individual Tests
```bash
make test-steel-led    # Steel LED control tests
make test-steel-sleep  # Steel sleep function tests
make examples-steel    # Steel example programs
```

## Test Categories

### 1. Rust Unit Tests (`cargo test --lib`)

**Location:** `src/` directories with `#[cfg(test)]` modules  
**Purpose:** Test individual Rust components in isolation

```rust
#[tokio::test]
async fn test_led_control_api() {
    let api = create_test_api().await;
    assert!(api.set_led(true).await.is_ok());
    assert_eq!(api.get_led_state().await.unwrap(), LedState::On);
}
```

**Coverage:**
- Hardware Abstraction Layer (HAL) operations
- Steel API bindings and validation
- Error handling and edge cases
- Concurrent operation safety
- Mock component behavior

### 2. Rust Integration Tests (`cargo test --test '*'`)

**Location:** `aws-iot-core/tests/integration_*.rs`  
**Purpose:** Test system-level interactions between components

```rust
#[tokio::test]
async fn test_steel_runtime_integration() {
    let hal = Arc::new(MockHAL::new());
    let runtime = SteelRuntime::new(hal).unwrap();
    
    let result = runtime.execute_code("(led-on)").await;
    assert!(result.is_ok());
}
```

**Coverage:**
- Steel runtime with HAL integration
- IoT client connectivity and messaging
- Security and certificate management
- Program delivery and execution
- Shadow synchronization

### 3. Steel Functional Tests

**Location:** `aws-iot-core/tests/steel/*.scm`  
**Purpose:** Test functionality from the Steel programmer's perspective

```scheme
;; test_led_control.scm
(begin
  (log-info "=== LED Control Test ===")
  (led-on)
  (if (led-state)
      (log-info "✓ LED on test: PASSED")
      (error "LED on test failed"))
  (led-off)
  (if (not (led-state))
      (log-info "✓ LED off test: PASSED")
      (error "LED off test failed")))
```

**Test Files:**
- `test_led_control.scm` - LED operations and state management
- `test_sleep_function.scm` - Sleep timing and duration validation
- `test_device_info.scm` - System information queries
- `test_logging.scm` - Logging functionality at all levels
- `test_complex_operations.scm` - Multi-component integration scenarios

### 4. Steel Example Programs

**Location:** `aws-iot-core/examples/steel/*.scm`  
**Purpose:** Demonstrate Steel capabilities and serve as integration tests

```scheme
;; system_monitor.scm - Continuous system monitoring example
;; interactive_demo.scm - Comprehensive feature demonstration
```

### 5. Performance Benchmarks

**Location:** `aws-iot-tests/src/performance_benchmarks.rs`  
**Purpose:** Measure and validate performance characteristics

```rust
#[tokio::test]
async fn benchmark_steel_execution() {
    let start = Instant::now();
    for _ in 0..1000 {
        runtime.execute_code("(+ 1 1)").await.unwrap();
    }
    let duration = start.elapsed();
    assert!(duration < Duration::from_secs(1));
}
```

## Mock Components

The testing infrastructure includes comprehensive mock implementations:

### MockHAL
- Simulates hardware operations without actual hardware
- Tracks operation history for verification
- Configurable failure modes for error testing

### MockIoTClient  
- Simulates AWS IoT connectivity
- Records published messages and subscriptions
- Supports connection failure scenarios

### MockSteelRuntime
- Simulates Steel program execution
- Tracks execution history and performance
- Configurable execution delays and failures

## Test Execution

### Command Line Tools

#### Steel Test Runner (`steel_test`)
```bash
# Run all Steel tests
cargo run --bin steel_test

# Run specific test file
cargo run --bin steel_test -- --file test_led_control.scm

# Verbose output
cargo run --bin steel_test -- --verbose

# Continue on errors
cargo run --bin steel_test -- --continue-on-error
```

#### Steel Example Runner (`steel_example`)
```bash
# Run all examples
cargo run --bin steel_example

# Run specific example
cargo run --bin steel_example -- --file system_monitor.scm

# Interactive mode
cargo run --bin steel_example -- --interactive

# List available examples
cargo run --bin steel_example -- --list
```

#### Test Aggregator (`test_aggregator`)
```bash
# Generate comprehensive test report
cargo run --bin test_aggregator

# Run tests and generate report
cargo run --bin test_aggregator -- --run-tests

# Generate HTML report
cargo run --bin test_aggregator -- --format html --output report.html
```

### Makefile Commands

The Makefile provides convenient shortcuts for all testing scenarios:

```bash
# Main test commands
make test-all              # Complete test suite
make test-rust             # All Rust tests
make test-steel            # All Steel tests
make test-unit             # Rust unit tests
make test-integration      # Rust integration tests

# Individual Steel tests
make test-steel-led        # LED control tests
make test-steel-sleep      # Sleep function tests
make test-steel-device     # Device info tests
make test-steel-logging    # Logging tests
make test-steel-complex    # Complex operations tests

# Examples
make examples-steel        # All Steel examples
make example-monitor       # System monitor example
make example-demo          # Interactive demo

# Development helpers
make dev-test              # Quick development test cycle
make smoke-test            # Fast smoke tests
make benchmark             # Performance benchmarks

# Quality and CI
make check                 # Cargo check
make lint                  # Clippy linting
make format                # Code formatting
make ci                    # Full CI pipeline
make pre-commit            # Pre-commit checks

# Reporting
make test-report           # Generate comprehensive report
make list-steel            # List available Steel tests/examples
```

## Test Result Reporting

### JSON Report Format
```json
{
  "timestamp": "2024-01-01T12:00:00Z",
  "total_suites": 3,
  "total_tests": 45,
  "total_passed": 43,
  "total_failed": 2,
  "overall_success_rate": 95.6,
  "suites": [
    {
      "name": "Rust Unit Tests",
      "total_tests": 25,
      "passed_tests": 25,
      "failed_tests": 0,
      "success_rate": 100.0
    }
  ]
}
```

### HTML Report
Generates a comprehensive HTML report with:
- Visual test result summary
- Suite-by-suite breakdown
- Pass/fail indicators
- Execution timing information

### Markdown Report
Creates documentation-friendly markdown reports suitable for:
- GitHub README integration
- CI/CD pipeline summaries
- Development team communication

## Continuous Integration

### CI Pipeline Steps
1. **Code Quality Checks**
   ```bash
   make check lint format
   ```

2. **Rust Test Execution**
   ```bash
   make test-unit test-integration
   ```

3. **Steel Test Execution**
   ```bash
   make test-steel
   ```

4. **Performance Validation**
   ```bash
   make benchmark
   ```

5. **Report Generation**
   ```bash
   make test-report
   ```

### Exit Codes
- `0` - All tests passed
- `1` - One or more tests failed
- `2` - Test execution error

## Writing New Tests

### Rust Test Guidelines

1. **Use descriptive test names**
   ```rust
   #[tokio::test]
   async fn test_led_state_consistency_after_rapid_toggling() {
       // Test implementation
   }
   ```

2. **Test both success and failure cases**
   ```rust
   #[tokio::test]
   async fn test_invalid_sleep_duration_returns_error() {
       let api = create_test_api().await;
       let result = api.sleep(-1.0).await;
       assert!(matches!(result, Err(APIError::InvalidParameter(_))));
   }
   ```

3. **Use appropriate mock configurations**
   ```rust
   let mut mock_hal = MockHAL::new();
   mock_hal.set_should_fail_led_operation(true);
   ```

### Steel Test Guidelines

1. **Use clear test structure**
   ```scheme
   (begin
     (log-info "=== Test Name ===")
     ;; Setup
     ;; Test operations
     ;; Assertions
     (log-info "=== Test Completed ===")
     #t)
   ```

2. **Include comprehensive error checking**
   ```scheme
   (let ((result (led-on)))
     (if result
         (log-info "✓ Test passed")
         (begin
           (log-error "✗ Test failed")
           (error "LED operation failed"))))
   ```

3. **Test edge cases and error conditions**
   ```scheme
   ;; Test invalid parameters
   (let ((result (sleep -1)))
     (if (error? result)
         (log-info "✓ Error handling works")
         (error "Should have failed with negative sleep")))
   ```

## Performance Expectations

### Rust Tests
- Unit tests: < 100ms per test
- Integration tests: < 1s per test
- Total Rust test suite: < 30s

### Steel Tests
- Individual Steel tests: < 5s per test
- Complex integration tests: < 30s per test
- Total Steel test suite: < 2 minutes

### Examples
- Simple examples: < 10s
- Complex examples: < 60s
- Interactive examples: Variable (user-dependent)

## Troubleshooting

### Common Issues

1. **Steel test timeouts**
   - Check for infinite loops in Steel code
   - Verify mock HAL sleep durations are reasonable
   - Use `--verbose` flag for detailed execution logs

2. **Mock component state issues**
   - Ensure proper cleanup between tests
   - Use `clear_test_data()` methods
   - Check for shared state between concurrent tests

3. **Platform-specific test failures**
   - Verify platform-specific HAL implementations
   - Check timing-sensitive tests for platform differences
   - Use appropriate tolerance in timing assertions

### Debug Commands
```bash
# Verbose Steel test execution
make test-steel-verbose

# Run single test with full output
cargo run --bin steel_test -- --file test_led_control.scm --verbose

# Check test environment
make list-steel
make help-steel-test
```

## Best Practices

1. **Test Independence** - Each test should be independent and not rely on other tests
2. **Deterministic Results** - Tests should produce consistent results across runs
3. **Comprehensive Coverage** - Test both happy path and error conditions
4. **Performance Awareness** - Keep tests fast to encourage frequent execution
5. **Clear Documentation** - Include comments explaining complex test scenarios
6. **Mock Realism** - Ensure mocks behave realistically to catch real issues

## Future Enhancements

- **Property-based testing** for Steel programs
- **Fuzzing integration** for robust error handling
- **Hardware-in-the-loop testing** with real ESP32 devices
- **Load testing** for concurrent Steel program execution
- **Memory usage profiling** for embedded deployment validation