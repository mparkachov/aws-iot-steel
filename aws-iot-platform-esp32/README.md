# ESP32-C3-DevKit-RUST-1 Platform Implementation

This crate provides the ESP32-C3-DevKit-RUST-1 hardware abstraction layer (HAL) implementation for the AWS IoT Steel module.

## Hardware Requirements

- **ESP32-C3-DevKit-RUST-1** development board
- USB-C cable for programming and power
- Built-in LED on GPIO2 (used for testing)

## Development Setup

### Prerequisites

1. **Install Rust ESP toolchain:**
   ```bash
   # Install espup
   cargo install espup
   
   # Install ESP toolchain
   espup install
   
   # Source the environment
   source ~/export-esp.sh
   ```

2. **Install additional tools:**
   ```bash
   # Install espflash for flashing
   cargo install espflash
   
   # Install ldproxy for linking
   cargo install ldproxy
   ```

3. **Verify ESP32-C3 target:**
   ```bash
   rustup target list | grep riscv32imc-esp-espidf
   ```

### Building

1. **Build for ESP32-C3:**
   ```bash
   cargo build --target riscv32imc-esp-espidf
   ```

2. **Build hardware test binary:**
   ```bash
   cargo build --bin hardware_test --target riscv32imc-esp-espidf
   ```

3. **Build with optimizations:**
   ```bash
   cargo build --release --target riscv32imc-esp-espidf
   ```

### Flashing and Testing

1. **Connect ESP32-C3-DevKit-RUST-1** via USB-C

2. **Flash hardware test:**
   ```bash
   espflash flash --monitor target/riscv32imc-esp-espidf/debug/hardware_test
   ```

3. **Flash with automatic monitoring:**
   ```bash
   cargo run --bin hardware_test --target riscv32imc-esp-espidf
   ```

4. **Monitor serial output:**
   ```bash
   espflash monitor
   ```

## Hardware Features Implemented

### GPIO Control
- **LED Control**: Built-in LED on GPIO2
- **State Management**: LED on/off with state tracking
- **Error Handling**: GPIO operation error detection

### Sleep and Power Management
- **FreeRTOS Sleep**: Accurate sleep timing using FreeRTOS delays
- **Power Optimization**: Configurable power management
- **Wake-up Handling**: Proper wake-up from sleep states

### Memory Management
- **Heap Monitoring**: Real-time heap usage tracking
- **Memory Statistics**: Total, free, used, and largest free block
- **Memory Safety**: Heap poisoning and overflow detection

### Secure Storage
- **NVS Integration**: Non-Volatile Storage for persistent data
- **Encryption**: Hardware-encrypted storage at rest
- **Key Management**: Secure key-value storage operations
- **Data Validation**: Input validation and size limits

### System Information
- **Device Identification**: MAC address-based device ID
- **Chip Information**: ESP32-C3 model and revision detection
- **Uptime Tracking**: System uptime using FreeRTOS ticks
- **Version Information**: Firmware and ESP-IDF version reporting

## Hardware Tests

The hardware test suite validates all ESP32-C3 functionality:

### Test Categories

1. **LED Hardware Test**
   - Visual LED blinking verification
   - GPIO state consistency
   - Rapid state change stability

2. **Sleep Accuracy Test**
   - Timing precision validation
   - Multiple duration testing
   - Power management verification

3. **Memory Monitoring Test**
   - Heap allocation tracking
   - Memory usage validation
   - Garbage collection verification

4. **Secure Storage Test**
   - NVS persistence validation
   - Data integrity verification
   - Key management testing

5. **Power Management Test**
   - Sleep/wake cycle testing
   - System state preservation
   - Power optimization validation

6. **GPIO Stability Test**
   - Rapid operation testing
   - State consistency validation
   - Hardware reliability verification

### Running Hardware Tests

```bash
# Flash and run all hardware tests
cargo run --bin hardware_test --target riscv32imc-esp-espidf

# Run specific test (modify source)
# Tests are organized in hardware_tests.rs module
```

### Expected Test Output

```
ESP32-C3 Hardware Test Runner starting...
Target: ESP32-C3-DevKit-RUST-1
Testing: GPIO, Sleep, Memory, Secure Storage, Power Management

[INFO] Starting ESP32-C3 hardware validation tests
[INFO] Testing LED hardware functionality
[INFO] LED blink cycle 1 of 5
[INFO] ESP32-C3 LED turned ON
[INFO] ESP32-C3 LED turned OFF
...
[INFO] All ESP32-C3 hardware tests completed successfully
✅ All hardware tests passed successfully!
```

## Configuration

### SDK Configuration (`sdkconfig.defaults`)

Key configurations for ESP32-C3-DevKit-RUST-1:
- **CPU Frequency**: 160MHz
- **Memory**: 8KB main task stack
- **NVS Encryption**: Enabled for secure storage
- **Power Management**: Enabled with DFS
- **Logging**: Info level with debug capability
- **Security**: Heap poisoning and stack overflow detection

### Cargo Configuration (`.cargo/config.toml`)

- **Target**: `riscv32imc-esp-espidf`
- **Linker**: `ldproxy`
- **Build Standard**: Custom std with panic_abort
- **ESP-IDF Version**: v5.1.2

## Memory Usage

### Flash Memory
- **Application**: ~200KB (debug), ~100KB (release)
- **ESP-IDF**: ~1MB
- **Available**: ~3MB remaining

### RAM Usage
- **Static**: ~50KB
- **Heap**: ~200KB available
- **Stack**: 8KB main task + additional async tasks

## Security Features

### Hardware Security
- **Secure Boot**: Configurable (disabled by default)
- **Flash Encryption**: Configurable (disabled by default)
- **NVS Encryption**: Enabled by default
- **Hardware RNG**: Used for cryptographic operations

### Secure Storage
- **Encryption**: AES-256 encryption at rest
- **Key Protection**: Hardware-protected keys
- **Access Control**: Namespace-based isolation
- **Data Integrity**: Checksum validation

## Troubleshooting

### Common Issues

1. **Build Errors**
   ```bash
   # Ensure ESP toolchain is sourced
   source ~/export-esp.sh
   
   # Clean and rebuild
   cargo clean
   cargo build --target riscv32imc-esp-espidf
   ```

2. **Flash Errors**
   ```bash
   # Check USB connection
   ls /dev/ttyUSB* # Linux
   ls /dev/cu.usbserial* # macOS
   
   # Reset ESP32-C3 and try again
   espflash flash --monitor target/riscv32imc-esp-espidf/debug/hardware_test
   ```

3. **Runtime Errors**
   ```bash
   # Monitor serial output for detailed logs
   espflash monitor
   
   # Check power supply (USB-C should provide sufficient power)
   ```

### Debug Information

Enable detailed logging:
```bash
export RUST_LOG=debug
cargo run --bin hardware_test --target riscv32imc-esp-espidf
```

## Integration with AWS IoT Steel

This HAL implementation integrates with the main AWS IoT Steel system:

1. **HAL Trait**: Implements `PlatformHAL` from `aws-iot-core`
2. **Error Handling**: Uses `PlatformError` for consistent error reporting
3. **Async Support**: Full async/await compatibility
4. **Type Safety**: Strong typing for all hardware operations

### Usage in Steel Runtime

```rust
use aws_iot_platform_esp32::ESP32HAL;
use aws_iot_core::PlatformHAL;

// Create and initialize ESP32-C3 HAL
let mut hal = ESP32HAL::new()?;
hal.initialize().await?;

// Use with Steel runtime
let steel_runtime = SteelRuntime::new(Arc::new(hal))?;
```

## Performance Characteristics

### Operation Timings (Typical)
- **LED State Change**: <1ms
- **Sleep (100ms)**: 100ms ±1ms
- **Memory Info**: <1ms
- **Secure Storage Write**: 5-20ms
- **Secure Storage Read**: 1-5ms
- **Device Info**: <1ms

### Resource Usage
- **CPU**: Minimal overhead, mostly I/O bound
- **Memory**: ~2KB static allocation
- **Flash**: ~50KB code size
- **Power**: Optimized for low-power operation

## Future Enhancements

### Planned Features
1. **WiFi Integration**: AWS IoT connectivity
2. **OTA Updates**: Over-the-air firmware updates
3. **Sensor Support**: Additional GPIO and I2C sensors
4. **Power Optimization**: Deep sleep modes
5. **Security Hardening**: Secure boot and flash encryption

### Hardware Expansion
- **I2C Sensors**: Temperature, humidity, accelerometer
- **SPI Devices**: External flash, displays
- **UART Communication**: Additional serial interfaces
- **PWM Output**: Motor control, servo control