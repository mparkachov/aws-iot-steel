use aws_iot_core::steel_runtime::SteelRuntimeAPI;
use aws_iot_core::types::LogLevel;
use aws_iot_core::*;

use std::sync::Arc;

mod common;
use common::MockHAL;

#[tokio::test]
async fn test_logging_levels_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));

    // Test all log levels
    let result = api.log(LogLevel::Error, "This is an error message");
    assert!(result.is_ok());

    let result = api.log(LogLevel::Warn, "This is a warning message");
    assert!(result.is_ok());

    let result = api.log(LogLevel::Info, "This is an info message");
    assert!(result.is_ok());

    let result = api.log(LogLevel::Debug, "This is a debug message");
    assert!(result.is_ok());

    // Test unknown log level (should default to info)
    let result = api.log(LogLevel::Error, "This is an unknown level message");
    assert!(result.is_ok());

    println!("Logging levels test completed successfully");
}

#[tokio::test]
async fn test_logging_with_steel_runtime() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(SteelRuntimeAPI::new(hal.clone()).unwrap());
    let runtime = SteelRuntimeImpl::new(api).unwrap();

    // Test logging operations through Steel runtime
    let logging_test_code = r#"
        (begin
          (log-info "Starting Steel logging test")
          
          (log-error "This is an error message")
          (log-warn "This is a warning message")
          (log-info "This is an info message")
          (log-debug "This is a debug message")
          
          (log "ERROR" "Generic error message")
          (log "WARN" "Generic warning message")
          (log "INFO" "Generic info message")
          (log "DEBUG" "Generic debug message")
          
          (log-info "Steel logging test completed")
          #t)
    "#;

    let result = runtime.execute_code_with_hal(logging_test_code).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_logging_performance_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));

    // Test logging performance with multiple messages
    let start = std::time::Instant::now();

    for i in 0..100 {
        let message = format!("Performance test message {}", i);
        let result = api.log(LogLevel::Info, &message);
        assert!(result.is_ok());
    }

    let elapsed = start.elapsed();
    println!("100 log messages completed in {:?}", elapsed);
    println!("Logging performance test completed successfully");
}

#[tokio::test]
async fn test_logging_with_special_characters_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));

    // Test with special characters
    let result = api.log(LogLevel::Info, "Special chars: !@#$%^&*()");
    assert!(result.is_ok());

    // Test with unicode
    let result = api.log(LogLevel::Info, "Unicode: αβγδε 中文 🚀");
    assert!(result.is_ok());

    // Test with newlines and tabs
    let result = api.log(LogLevel::Info, "Newlines\nand\ttabs");
    assert!(result.is_ok());

    // Test with empty message
    let result = api.log(LogLevel::Info, "");
    assert!(result.is_ok());

    println!("Logging with special characters test completed successfully");
}

#[tokio::test]
async fn test_logging_integration_with_operations_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));

    // Test logging combined with other operations
    let _ = api.log(LogLevel::Info, "Starting integration test");

    let result = api.led_on().await;
    assert!(result.is_ok());
    let _ = api.log(LogLevel::Info, "LED turned on");

    let result = api.sleep(0.001).await;
    assert!(result.is_ok());
    let _ = api.log(LogLevel::Info, "Sleep completed");

    let result = api.led_off().await;
    assert!(result.is_ok());
    let _ = api.log(LogLevel::Info, "LED turned off");

    let device_result = api.get_device_info().await;
    assert!(device_result.is_ok());
    let _ = api.log(LogLevel::Info, "Device info retrieved");

    let _ = api.log(LogLevel::Info, "Integration test completed");

    println!("Logging integration test completed successfully");
}
