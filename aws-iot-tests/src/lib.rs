//! AWS IoT Tests
//!
//! Comprehensive testing suite for the AWS IoT Steel system including:
//! - Unit tests for individual components
//! - Integration tests for system interactions
//! - Mock implementations for testing
//! - Property-based testing for Steel programs
//! - Performance benchmarks
//! - End-to-end validation tests
//! - Load testing and performance validation
//! - Security validation and penetration testing

pub mod error_handling_tests;
pub mod hal_tests;
pub mod integration_tests;
pub mod iot_integration_tests;
pub mod mock_hal;
pub mod mock_iot_client;
pub mod mock_steel_runtime;
pub mod property_based_tests;
pub mod steel_api_tests;
pub mod steel_runtime_tests;

// End-to-end validation modules
pub mod end_to_end_validation;
pub mod load_testing;
pub mod security_validation;

pub use iot_integration_tests::{IoTIntegrationTests, IoTTestConfig};
pub use mock_hal::MockHAL;
pub use mock_iot_client::MockIoTClient;
pub use mock_steel_runtime::MockSteelRuntime;

// Re-export validation components
pub use end_to_end_validation::EndToEndValidator;
pub use load_testing::{LoadTestConfig, LoadTestResults, LoadTester};
pub use security_validation::{SecurityTestResults, SecurityValidator};
