# AWS IoT Steel Test Runner Makefile
# Provides convenient commands for running Rust and Steel tests

.PHONY: help test test-rust test-steel examples examples-steel build clean

# Default target
help:
	@echo "AWS IoT Steel Test Runner"
	@echo ""
	@echo "Available commands:"
	@echo "  make test           - Run all tests (Rust + Steel)"
	@echo "  make test-rust      - Run only Rust integration tests"
	@echo "  make test-steel     - Run only Steel tests"
	@echo "  make examples       - Run all examples"
	@echo "  make examples-steel - Run only Steel examples"
	@echo "  make build          - Build the project"
	@echo "  make clean          - Clean build artifacts"
	@echo ""
	@echo "Individual Steel test files:"
	@echo "  make test-steel-led        - Run LED Steel tests"
	@echo "  make test-steel-sleep      - Run sleep Steel tests"
	@echo "  make test-steel-system     - Run system info Steel tests"
	@echo "  make test-steel-logging    - Run logging Steel tests"
	@echo "  make test-steel-integration - Run integration Steel tests"
	@echo ""
	@echo "Individual Steel example files:"
	@echo "  make example-blink         - Run LED blink example"
	@echo "  make example-monitor       - Run system monitor example"
	@echo "  make example-demo          - Run interactive demo example"

# Build the project
build:
	@echo "Building AWS IoT Steel project..."
	cargo build --workspace

# Run all tests (Rust + Steel)
test: test-rust test-steel
	@echo "All tests completed!"

# Run Rust integration tests
test-rust:
	@echo "Running Rust integration tests..."
	cargo test --package aws-iot-core --test integration_led
	cargo test --package aws-iot-core --test integration_sleep
	cargo test --package aws-iot-core --test integration_system_info
	cargo test --package aws-iot-core --test integration_logging
	@echo "Rust tests completed!"

# Run all Steel tests
test-steel: build
	@echo "Running all Steel tests..."
	cargo run --package aws-iot-core --bin test_runner steel-tests

# Run all examples
examples: examples-steel
	@echo "All examples completed!"

# Run Steel examples
examples-steel: build
	@echo "Running Steel examples..."
	cargo run --package aws-iot-core --bin test_runner steel-examples

# Individual Steel test files
test-steel-led: build
	@echo "Running Steel LED tests..."
	cargo run --package aws-iot-core --bin test_runner steel-test aws-iot-core/tests/steel/test_led.scm

test-steel-sleep: build
	@echo "Running Steel sleep tests..."
	cargo run --package aws-iot-core --bin test_runner steel-test aws-iot-core/tests/steel/test_sleep.scm

test-steel-system: build
	@echo "Running Steel system info tests..."
	cargo run --package aws-iot-core --bin test_runner steel-test aws-iot-core/tests/steel/test_system_info.scm

test-steel-logging: build
	@echo "Running Steel logging tests..."
	cargo run --package aws-iot-core --bin test_runner steel-test aws-iot-core/tests/steel/test_logging.scm

test-steel-integration: build
	@echo "Running Steel integration tests..."
	cargo run --package aws-iot-core --bin test_runner steel-test aws-iot-core/tests/steel/test_integration.scm

# Individual Steel example files
example-blink: build
	@echo "Running LED blink example..."
	cargo run --package aws-iot-core --bin test_runner steel-example aws-iot-core/examples/steel/blink_led.scm

example-monitor: build
	@echo "Running system monitor example..."
	cargo run --package aws-iot-core --bin test_runner steel-example aws-iot-core/examples/steel/system_monitor.scm

example-demo: build
	@echo "Running interactive demo example..."
	cargo run --package aws-iot-core --bin test_runner steel-example aws-iot-core/examples/steel/interactive_demo.scm

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Development helpers
dev-test: build
	@echo "Running quick development tests..."
	cargo test --package aws-iot-core --lib steel_runtime
	make test-steel-led

# Continuous integration target
ci: build test
	@echo "CI pipeline completed successfully!"