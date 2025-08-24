# AWS IoT Steel Dual Testing Infrastructure Makefile
# Provides convenient commands for running Rust and Steel tests

.PHONY: help test test-all test-rust test-steel test-unit test-integration examples examples-steel build clean check lint format infra-deploy infra-test infra-clean

# Default target
help:
	@echo "AWS IoT Steel Dual Testing Infrastructure"
	@echo ""
	@echo "Main Commands:"
	@echo "  make test-all       - Run all tests (Rust unit + integration + Steel)"
	@echo "  make test           - Alias for test-all"
	@echo "  make test-rust      - Run only Rust tests (unit + integration)"
	@echo "  make test-unit      - Run only Rust unit tests"
	@echo "  make test-integration - Run only Rust integration tests"
	@echo "  make test-steel     - Run only Steel tests"
	@echo "  make examples       - Run all examples"
	@echo "  make examples-steel - Run only Steel examples"
	@echo ""
	@echo "Build and Quality:"
	@echo "  make build          - Build the project"
	@echo "  make check          - Run cargo check"
	@echo "  make lint           - Run clippy linter"
	@echo "  make format         - Format code with rustfmt"
	@echo "  make clean          - Clean build artifacts"
	@echo ""
	@echo "End-to-End Validation:"
	@echo "  make validate-all   - Run comprehensive end-to-end validation"
	@echo "  make validate-e2e   - Run end-to-end system validation"
	@echo "  make validate-load  - Run load testing validation"
	@echo "  make validate-security - Run security validation"
	@echo "  make validate-production - Run production readiness validation"
	@echo "  make validate-dev   - Run quick development validation"
	@echo ""
	@echo "Infrastructure Commands:"
	@echo "  make infra-deploy   - Deploy AWS infrastructure (dev environment)"
	@echo "  make infra-test     - Test AWS infrastructure"
	@echo "  make infra-clean    - Clean up AWS infrastructure"
	@echo "  make infra-provision - Provision a new device"
	@echo ""
	@echo "Individual Steel Tests:"
	@echo "  make test-steel-led        - Run LED control Steel tests"
	@echo "  make test-steel-sleep      - Run sleep function Steel tests"
	@echo "  make test-steel-device     - Run device info Steel tests"
	@echo "  make test-steel-logging    - Run logging Steel tests"
	@echo "  make test-steel-complex    - Run complex operations Steel tests"
	@echo ""
	@echo "Individual Steel Examples:"
	@echo "  make example-blink         - Run LED blink example"
	@echo "  make example-monitor       - Run system monitor example"
	@echo "  make example-demo          - Run interactive demo example"
	@echo ""
	@echo "Development and CI:"
	@echo "  make dev-test       - Quick development test cycle"
	@echo "  make ci             - Full CI pipeline"
	@echo "  make benchmark      - Run performance benchmarks"

# Build the project
build:
	@echo "🔨 Building AWS IoT Steel project..."
	cargo build --workspace
	@echo "✅ Build completed!"

# Check the project without building
check:
	@echo "🔍 Running cargo check..."
	cargo check --workspace
	@echo "✅ Check completed!"

# Run clippy linter
lint:
	@echo "🧹 Running clippy linter..."
	cargo clippy --workspace -- -D warnings
	@echo "✅ Linting completed!"

# Format code
format:
	@echo "🎨 Formatting code..."
	cargo fmt --all
	@echo "✅ Formatting completed!"

# Run all tests (Rust + Steel) - main test target
test-all: build
	@echo "🧪 Running complete test suite (Rust + Steel)..."
	@echo ""
	@$(MAKE) test-rust
	@echo ""
	@$(MAKE) test-steel
	@echo ""
	@echo "🎉 All tests completed successfully!"

# Alias for test-all
test: test-all

# Run all Rust tests (unit + integration)
test-rust: test-unit test-integration
	@echo "✅ All Rust tests completed!"

# Run Rust unit tests
test-unit:
	@echo "🦀 Running Rust unit tests..."
	cargo test --workspace --lib
	@echo "✅ Rust unit tests completed!"

# Run Rust integration tests
test-integration:
	@echo "🔗 Running Rust integration tests..."
	cargo test --workspace --test '*'
	@echo "✅ Rust integration tests completed!"

# Run all Steel tests
test-steel: build
	@echo "⚙️  Running Steel test suite..."
	cargo run --bin steel_test --package aws-iot-core
	@echo "✅ Steel tests completed!"

# Run Steel tests with verbose output
test-steel-verbose: build
	@echo "⚙️  Running Steel test suite (verbose)..."
	cargo run --bin steel_test --package aws-iot-core -- --verbose
	@echo "✅ Steel tests completed!"

# Run all examples
examples: examples-steel
	@echo "✅ All examples completed!"

# Run Steel examples
examples-steel: build
	@echo "🎯 Running Steel examples..."
	cargo run --bin steel_example --package aws-iot-core
	@echo "✅ Steel examples completed!"

# Run Steel examples with verbose output
examples-steel-verbose: build
	@echo "🎯 Running Steel examples (verbose)..."
	cargo run --bin steel_example --package aws-iot-core -- --verbose
	@echo "✅ Steel examples completed!"

# Run Steel examples interactively
examples-steel-interactive: build
	@echo "🎯 Running Steel examples (interactive)..."
	cargo run --bin steel_example --package aws-iot-core -- --interactive --verbose
	@echo "✅ Steel examples completed!"

# Individual Steel test files
test-steel-led: build
	@echo "🔆 Running Steel LED control tests..."
	cargo run --bin steel_test --package aws-iot-core -- --file aws-iot-core/tests/steel/test_led_control.scm

test-steel-sleep: build
	@echo "💤 Running Steel sleep function tests..."
	cargo run --bin steel_test --package aws-iot-core -- --file aws-iot-core/tests/steel/test_sleep_function.scm

test-steel-device: build
	@echo "📱 Running Steel device info tests..."
	cargo run --bin steel_test --package aws-iot-core -- --file aws-iot-core/tests/steel/test_device_info.scm

test-steel-logging: build
	@echo "📝 Running Steel logging tests..."
	cargo run --bin steel_test --package aws-iot-core -- --file aws-iot-core/tests/steel/test_logging.scm

test-steel-complex: build
	@echo "🔧 Running Steel complex operations tests..."
	cargo run --bin steel_test --package aws-iot-core -- --file aws-iot-core/tests/steel/test_complex_operations.scm

# Individual Steel example files
example-blink: build
	@echo "🔆 Running LED blink example..."
	cargo run --bin steel_example --package aws-iot-core -- --file aws-iot-core/examples/steel/blink_led.scm

example-monitor: build
	@echo "📊 Running system monitor example..."
	cargo run --bin steel_example --package aws-iot-core -- --file aws-iot-core/examples/steel/system_monitor.scm

example-demo: build
	@echo "🎯 Running interactive demo example..."
	cargo run --bin steel_example --package aws-iot-core -- --file aws-iot-core/examples/steel/interactive_demo.scm

example-demo-interactive: build
	@echo "🎯 Running interactive demo example (interactive mode)..."
	cargo run --bin steel_example --package aws-iot-core -- --file aws-iot-core/examples/steel/interactive_demo.scm --interactive

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "✅ Clean completed!"

# Run performance benchmarks
benchmark: build
	@echo "⚡ Running performance benchmarks..."
	cargo test --package aws-iot-tests --test '*' benchmark -- --nocapture
	@echo "✅ Benchmarks completed!"

# Development helpers
dev-test: build
	@echo "🚀 Running quick development test cycle..."
	@$(MAKE) test-unit
	@$(MAKE) test-steel-led
	@$(MAKE) test-steel-sleep
	@echo "✅ Development tests completed!"

# Quick smoke test
smoke-test: build
	@echo "💨 Running smoke tests..."
	cargo test --package aws-iot-core --lib -- --test-threads=1
	@$(MAKE) test-steel-led
	@echo "✅ Smoke tests completed!"

# Test result aggregation and reporting
test-report: build
	@echo "📊 Generating comprehensive test report..."
	@cargo run --bin test_aggregator --package aws-iot-core -- --run-tests --output test-results.json
	@echo "📊 Test report generated! Check test-results.json for details."

# Generate HTML test report
test-report-html: build
	@echo "📊 Generating HTML test report..."
	@cargo run --bin test_aggregator --package aws-iot-core -- --run-tests --format html --output test-results.html
	@echo "📊 HTML test report generated! Open test-results.html in your browser."

# Generate Markdown test report
test-report-md: build
	@echo "📊 Generating Markdown test report..."
	@cargo run --bin test_aggregator --package aws-iot-core -- --run-tests --format markdown --output test-results.md
	@echo "📊 Markdown test report generated! Check test-results.md."

# Continuous integration target
ci: check lint test-all
	@echo "🎉 CI pipeline completed successfully!"

# Pre-commit checks
pre-commit: format lint check test-all
	@echo "✅ Pre-commit checks passed!"

# Watch mode for development (requires cargo-watch)
watch:
	@echo "👀 Starting watch mode for tests..."
	@command -v cargo-watch >/dev/null 2>&1 || { echo "cargo-watch not found. Install with: cargo install cargo-watch"; exit 1; }
	cargo watch -x "test --workspace --lib" -x "run --bin steel_test"

# List all available Steel tests and examples
list-steel:
	@echo "📋 Available Steel tests and examples:"
	@echo ""
	@echo "Steel Tests:"
	@cargo run --bin steel_test --package aws-iot-core -- --list || true
	@echo ""
	@echo "Steel Examples:"
	@cargo run --bin steel_example --package aws-iot-core -- --list || true

# Help for Steel test runner
help-steel-test:
	@echo "⚙️  Steel Test Runner Help:"
	@cargo run --bin steel_test --package aws-iot-core -- --help

# Help for Steel example runner
help-steel-example:
	@echo "🎯 Steel Example Runner Help:"
	@cargo run --bin steel_example --package aws-iot-core -- --help

# Integration tests with AWS IoT
test-integration-iot: build
	@echo "🌐 Running AWS IoT integration tests..."
	@cargo run --bin integration_test_runner --package aws-iot-tests -- --test-type iot

# Load tests
test-load: build
	@echo "⚡ Running load tests..."
	@cargo run --bin integration_test_runner --package aws-iot-tests -- --test-type load

# Load tests with different configurations
test-load-light: build
	@echo "⚡ Running light load tests..."
	@cargo run --bin integration_test_runner --package aws-iot-tests -- --test-type load --load-config light

test-load-heavy: build
	@echo "⚡ Running heavy load tests..."
	@cargo run --bin integration_test_runner --package aws-iot-tests -- --test-type load --load-config heavy

# Quick integration tests
test-integration-quick: build
	@echo "🚀 Running quick integration tests..."
	@cargo run --bin integration_test_runner --package aws-iot-tests -- --test-type quick

# Full integration test suite
test-integration-full: build
	@echo "🎯 Running full integration test suite..."
	@cargo run --bin integration_test_runner --package aws-iot-tests -- --test-type all --verbose

# Integration tests with output file
test-integration-report: build
	@echo "📊 Running integration tests with report generation..."
	@cargo run --bin integration_test_runner --package aws-iot-tests -- --test-type all --output integration-results.json --verbose
	@echo "📊 Integration test report saved to integration-results.json"

# AWS Infrastructure Management
infra-deploy:
	@echo "🏗️  Deploying AWS infrastructure (dev environment)..."
	@./aws-infrastructure/scripts/deploy-core-infrastructure.sh dev us-west-2
	@./aws-infrastructure/scripts/deploy-s3-lambda.sh dev us-west-2
	@echo "✅ Infrastructure deployment completed!"

infra-deploy-prod:
	@echo "🏗️  Deploying AWS infrastructure (production environment)..."
	@./aws-infrastructure/scripts/deploy-core-infrastructure.sh prod us-west-2
	@./aws-infrastructure/scripts/deploy-s3-lambda.sh prod us-west-2
	@echo "✅ Production infrastructure deployment completed!"

infra-test:
	@echo "🧪 Testing AWS infrastructure..."
	@./aws-infrastructure/tests/run-all-tests.sh dev us-west-2
	@echo "✅ Infrastructure tests completed!"

infra-test-prod:
	@echo "🧪 Testing AWS infrastructure (production)..."
	@./aws-infrastructure/tests/run-all-tests.sh prod us-west-2
	@echo "✅ Production infrastructure tests completed!"

infra-clean:
	@echo "🧹 Cleaning up AWS infrastructure (dev environment)..."
	@./aws-infrastructure/scripts/cleanup-infrastructure.sh dev us-west-2
	@echo "✅ Infrastructure cleanup completed!"

infra-clean-prod:
	@echo "🧹 Cleaning up AWS infrastructure (production environment)..."
	@./aws-infrastructure/scripts/cleanup-infrastructure.sh prod us-west-2
	@echo "✅ Production infrastructure cleanup completed!"

infra-provision:
	@echo "📱 Provisioning new device..."
	@read -p "Enter device ID (e.g., device-001): " DEVICE_ID; \
	./aws-infrastructure/scripts/provision-device.sh $$DEVICE_ID dev us-west-2
	@echo "✅ Device provisioning completed!"

infra-validate:
	@echo "✅ Validating CloudFormation templates..."
	@./aws-infrastructure/tests/validate-templates.sh us-west-2
	@echo "✅ Template validation completed!"

# Infrastructure development helpers
infra-dev: infra-validate infra-deploy infra-test
	@echo "🚀 Infrastructure development cycle completed!"

# End-to-End Validation Tests
validate-e2e: build
	@echo "🔄 Running end-to-end validation tests..."
	@cargo run --bin end_to_end_validator --package aws-iot-tests -- --test-suite e2e --verbose
	@echo "✅ End-to-end validation completed!"

validate-load: build
	@echo "⚡ Running load testing validation..."
	@cargo run --bin end_to_end_validator --package aws-iot-tests -- --test-suite load --verbose
	@echo "✅ Load testing validation completed!"

validate-security: build
	@echo "🔒 Running security validation..."
	@cargo run --bin end_to_end_validator --package aws-iot-tests -- --test-suite security --verbose
	@echo "✅ Security validation completed!"

validate-all: build
	@echo "🎯 Running comprehensive end-to-end validation..."
	@cargo run --bin end_to_end_validator --package aws-iot-tests -- --test-suite all --verbose
	@echo "✅ Comprehensive validation completed!"

# Load testing with custom parameters
validate-load-heavy: build
	@echo "⚡ Running heavy load testing..."
	@cargo run --bin end_to_end_validator --package aws-iot-tests -- --test-suite load \
		--concurrent-programs 100 --messages-per-program 200 --test-duration 600 --verbose
	@echo "✅ Heavy load testing completed!"

validate-load-light: build
	@echo "⚡ Running light load testing..."
	@cargo run --bin end_to_end_validator --package aws-iot-tests -- --test-suite load \
		--concurrent-programs 10 --messages-per-program 50 --test-duration 120 --verbose
	@echo "✅ Light load testing completed!"

# Production readiness validation
validate-production: build
	@echo "🏭 Running production readiness validation..."
	@$(MAKE) validate-all
	@$(MAKE) validate-load-heavy
	@$(MAKE) infra-test
	@echo "🎉 Production readiness validation completed!"

# Quick validation for development
validate-dev: build
	@echo "🚀 Running development validation..."
	@cargo run --bin end_to_end_validator --package aws-iot-tests -- --test-suite e2e
	@$(MAKE) validate-load-light
	@echo "✅ Development validation completed!"

# Full deployment pipeline
deploy-all: ci validate-production infra-deploy infra-test
	@echo "🎉 Full deployment pipeline completed successfully!"