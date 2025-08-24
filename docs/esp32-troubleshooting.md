# ESP32 Development Troubleshooting Guide

This guide helps resolve common ESP32 development issues, especially on macOS.

## Quick Solutions

### Option 1: Skip ESP32 Development (Recommended for now)
```bash
make skip-esp32
make pre-push  # Will now skip ESP32 checks
```

### Option 2: Install espup Binary (Avoids compilation issues)
```bash
./scripts/install-espup-binary.sh
espup install
source $HOME/export-esp.sh
```

### Option 3: Full ESP32 Setup (May have issues)
```bash
make setup-esp32  # May fail due to dependency conflicts
```

## Common Issues and Solutions

### 1. espup Compilation Fails (indicatif dependency conflict)

**Error:**
```
error[E0308]: mismatched types
expected `indicatif::multi::MultiProgress`, found `MultiProgress`
```

**Solutions:**

**A. Use pre-built binary (Recommended):**
```bash
./scripts/install-espup-binary.sh
```

**B. Try older version:**
```bash
cargo install espup --version 0.14.0
```

**C. Install from git:**
```bash
cargo install --git https://github.com/esp-rs/espup.git espup
```

### 2. ESP32 Target Not Available

**Error:**
```
error: toolchain 'stable-aarch64-apple-darwin' does not support target 'riscv32imc-esp-espidf'
```

**Explanation:**
- This is normal on macOS
- ESP32 target requires ESP-IDF toolchain, not standard Rust
- The target becomes available after installing espup and ESP-IDF

**Solution:**
```bash
# Install espup first
./scripts/install-espup-binary.sh

# Install ESP-IDF
espup install

# Source environment (adds ESP32 target)
source $HOME/export-esp.sh

# Now ESP32 target should be available
cargo build --target riscv32imc-esp-espidf --package aws-iot-platform-esp32
```

### 3. ESP-IDF Installation Fails

**Error:**
```
Failed to install ESP-IDF toolchain
```

**Solutions:**

**A. Manual ESP-IDF installation:**
1. Download ESP-IDF installer from [Espressif](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/macos-setup.html)
2. Follow the official installation guide
3. Source the environment: `source ~/esp/esp-idf/export.sh`

**B. Use Docker (Alternative):**
```bash
# Use ESP-IDF Docker container
docker run --rm -v $PWD:/project -w /project espressif/idf:latest \
  idf.py build
```

### 4. Permission Issues

**Error:**
```
Permission denied when accessing /dev/ttyUSB0
```

**Solution (macOS):**
```bash
# Add user to dialout group (Linux)
sudo usermod -a -G dialout $USER

# On macOS, check USB permissions
ls -la /dev/tty.*
```

### 5. Build Fails After ESP-IDF Installation

**Error:**
```
linker `riscv32-esp-elf-gcc` not found
```

**Solution:**
```bash
# Make sure ESP-IDF environment is sourced
source $HOME/export-esp.sh

# Verify tools are available
which riscv32-esp-elf-gcc

# If not found, reinstall ESP-IDF
espup uninstall
espup install
```

## Platform-Specific Notes

### macOS (Apple Silicon)
- ESP32 development requires espup, not standard Rust toolchain
- Use pre-built binaries to avoid compilation issues
- ESP-IDF installation downloads ~1GB of tools

### macOS (Intel)
- Similar to Apple Silicon
- May have better compatibility with some tools

### Linux
- ESP32 target available through standard rustup
- May still need ESP-IDF for full functionality

## Development Workflow Options

### Option 1: Skip ESP32 Locally, Test in CI
```bash
make skip-esp32           # Configure to skip ESP32
make pre-push            # Fast local checks
git push                 # ESP32 tested in GitHub Actions
```

**Pros:**
- Fast local development
- No ESP32 setup required
- CI still validates ESP32

**Cons:**
- Can't test ESP32 changes locally

### Option 2: Full ESP32 Development
```bash
./scripts/install-espup-binary.sh  # Install espup
espup install                      # Install ESP-IDF
source $HOME/export-esp.sh         # Load environment
make test-esp32                    # Test ESP32 build
```

**Pros:**
- Full ESP32 development capability
- Test changes locally

**Cons:**
- Complex setup
- Large download (~1GB)
- Potential compatibility issues

### Option 3: Docker-Based Development
```bash
# Use ESP-IDF Docker container for ESP32 work
docker run --rm -v $PWD:/project -w /project espressif/idf:latest \
  cargo build --target riscv32imc-esp-espidf --package aws-iot-platform-esp32
```

**Pros:**
- Isolated environment
- No local ESP-IDF installation
- Consistent across platforms

**Cons:**
- Requires Docker
- Slower than native development

## Verification Commands

After setup, verify your ESP32 environment:

```bash
# Check espup installation
espup --version

# Check ESP-IDF environment
source $HOME/export-esp.sh
echo $IDF_PATH

# Check ESP32 target
rustup target list --installed | grep esp

# Test ESP32 build
cargo check --target riscv32imc-esp-espidf --package aws-iot-platform-esp32

# Run ESP32 tests
make test-esp32
```

## Getting Help

If you're still having issues:

1. **Check ESP-IDF documentation:** https://docs.espressif.com/projects/esp-idf/
2. **Check espup repository:** https://github.com/esp-rs/espup
3. **Skip ESP32 for now:** `make skip-esp32`
4. **Use GitHub Actions for ESP32 testing:** Your CI will still validate ESP32 builds

## Recommended Approach

For most developers, we recommend:

1. **Start with:** `make skip-esp32`
2. **Develop on:** macOS/Linux platforms
3. **Let CI handle:** ESP32 validation
4. **Setup ESP32 later:** When you need to work on ESP32-specific features

This keeps your development workflow fast and simple while still ensuring ESP32 compatibility through CI.