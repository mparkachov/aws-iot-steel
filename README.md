# AWS IoT Steel

A cross-platform Rust application that combines Rust for system-level operations with Steel (Scheme) for scripting capabilities. The module connects to AWS IoT Core and supports over-the-air updates.

## Project Structure

This project uses a Cargo workspace with separate crates for different concerns:

```
aws-iot-steel/
├── Cargo.toml                     # Workspace configuration
├── README.md                      # This file
├── LICENSE                        # MIT License
├── aws-iot-core/                 # Core interfaces and types
│   ├── src/
│   │   ├── lib.rs                # Main library exports
│   │   ├── error.rs              # Error types and handling
│   │   ├── hal.rs                # Hardware Abstraction Layer trait
│   │   ├── logging.rs            # Logging framework setup
│   │   └── types.rs              # Common data types
│   └── Cargo.toml
├── aws-iot-platform-macos/       # macOS simulation platform
│   ├── src/
│   │   ├── lib.rs
│   │   └── hal.rs                # macOS HAL implementation
│   └── Cargo.toml
├── aws-iot-platform-esp32/       # ESP32-S3 hardware platform
│   ├── src/
│   │   ├── lib.rs
│   │   └── hal.rs                # ESP32 HAL implementation (stub)
│   └── Cargo.toml
├── aws-iot-tests/                # Comprehensive test suite
│   ├── src/
│   │   ├── lib.rs
│   │   ├── hal_tests.rs          # HAL unit tests
│   │   ├── integration_tests.rs  # Integration tests
│   │   └── mock_hal.rs           # Mock HAL for testing
│   └── Cargo.toml
└── examples/                     # Example applications
    ├── Cargo.toml
    └── basic_hal_demo.rs         # Basic HAL demonstration
```

## Core Components

### Hardware Abstraction Layer (HAL)

The `PlatformHAL` trait provides a unified interface for hardware operations:

- **Sleep operations**: `sleep(duration)` with configurable duration
- **LED control**: `set_led(state)` and `get_led_state()` for status indication
- **Device information**: `get_device_info()`, `get_memory_info()`, `get_uptime()`
- **Secure storage**: `store_secure_data()`, `load_secure_data()`, `delete_secure_data()`
- **Lifecycle management**: `initialize()` and `shutdown()`

### Error Handling

Comprehensive error types with proper error propagation:

- `SystemError`: Top-level system errors
- `PlatformError`: Hardware and platform-specific errors  
- `SecurityError`: Security and cryptography errors

### Logging Framework

Configurable logging with multiple levels and formats:

- Support for different log levels (Error, Warn, Info, Debug, Trace)
- Multiple output formats (Pretty, JSON, Compact)
- Development and production configurations

## Platform Implementations

### macOS Simulator (`aws-iot-platform-macos`)

Provides simulation of ESP32 hardware for development:

- Sleep simulation using `tokio::time::sleep`
- LED state simulation with console output
- File-based secure storage (will use Keychain in future)
- System information gathering using macOS APIs

### ESP32-S3 Hardware (`aws-iot-platform-esp32`)

Will provide actual hardware integration:

- Real sleep functionality with power management
- GPIO-based LED control
- Secure element integration for certificate storage
- ESP-IDF API integration

## Getting Started

### Prerequisites

- Rust 1.70+ with Cargo
- For macOS development: macOS 10.15+
- For ESP32 development: ESP-IDF toolchain (future)

### Building

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p aws-iot-core

# Run tests
cargo test

# Run example
cd examples && cargo run --bin basic_hal_demo
```

### Running the Demo

```bash
cd examples
cargo run --bin basic_hal_demo
```

This will demonstrate:
- HAL initialization and device information
- LED control operations
- Sleep functionality
- Secure storage operations
- Memory and uptime reporting

## Testing

The project includes comprehensive testing:

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p aws-iot-core
cargo test -p aws-iot-tests

# Run tests with logging
RUST_LOG=debug cargo test
```

### Test Coverage

- **Unit tests**: Individual component testing with mocks
- **Integration tests**: Cross-component functionality
- **HAL tests**: Platform abstraction layer validation
- **Mock implementations**: For testing without hardware

## Development Workflow

1. **Core Development**: Implement interfaces in `aws-iot-core`
2. **Platform Implementation**: Add platform-specific code in platform crates
3. **Testing**: Use `aws-iot-tests` for validation
4. **Examples**: Create demonstrations in `examples/`

## Requirements Satisfied

This implementation satisfies the following requirements:

- **Requirement 1.1**: Cross-platform Rust application with macOS simulation ✅
- **Requirement 1.4**: Device information and system monitoring ✅  
- **Requirement 8.1**: Extensible command architecture with HAL trait ✅
- **Requirement 8.2**: Automatic API exposure through trait system ✅

## Next Steps

The project structure is now ready for:

1. Steel (Scheme) runtime integration
2. AWS IoT Core connectivity
3. Over-the-air update system
4. ESP32-S3 hardware implementation
5. CI/CD pipeline setup

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.