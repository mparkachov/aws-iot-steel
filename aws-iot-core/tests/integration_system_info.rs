use aws_iot_core::*;
use aws_iot_core::steel_runtime::SteelRuntimeAPI;

use std::sync::Arc;

mod common;
use common::MockHAL;

#[tokio::test]
async fn test_device_info_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));
    
    let result = api.get_device_info().await;
    assert!(result.is_ok());
    
    if let Ok(device_info) = result {
        println!("Device info: {:?}", device_info);
    } else {
        panic!("Expected device info result");
    }
    
    println!("Device info test completed successfully");
}

#[tokio::test]
async fn test_memory_info_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));
    
    let result = api.get_memory_info().await;
    assert!(result.is_ok());
    
    if let Ok(memory_info) = result {
        println!("Memory info: {:?}", memory_info);
    } else {
        panic!("Expected memory info result");
    }
    
    println!("Memory info test completed successfully");
}

#[tokio::test]
async fn test_uptime_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));
    
    let result = api.get_uptime().await;
    assert!(result.is_ok());
    
    if let Ok(uptime_duration) = result {
        let uptime_secs = uptime_duration.as_secs_f64();
        assert!(uptime_secs >= 0.0);
        println!("Uptime: {} seconds", uptime_secs);
    } else {
        panic!("Expected number result from uptime");
    }
    
    println!("Uptime test completed successfully");
}

#[tokio::test]
async fn test_system_info_integration_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));
    
    // Get all system information
    let device_result = api.get_device_info().await;
    let memory_result = api.get_memory_info().await;
    let uptime_result = api.get_uptime().await;
    
    assert!(device_result.is_ok());
    assert!(memory_result.is_ok());
    assert!(uptime_result.is_ok());
    
    println!("=== System Information Summary ===");
    println!("Device: {:?}", device_result.unwrap());
    println!("Memory: {:?}", memory_result.unwrap());
    println!("Uptime: {:?}", uptime_result.unwrap());
    println!("=== End Summary ===");
    
    println!("System info integration test completed successfully");
}

#[tokio::test]
async fn test_system_info_with_steel_runtime() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(SteelRuntimeAPI::new(hal.clone()).unwrap());
    let runtime = SteelRuntimeImpl::new(api).unwrap();
    
    // Test system info operations through Steel runtime
    let system_info_test_code = r#"
        (begin
          (log-info "Starting Steel system info test")
          
          (let ((device (device-info)))
            (log-info "Device info retrieved")
            (if (list? device)
                (log-info "Device info is a list as expected")
                (error "Device info should be a list")))
          
          (let ((memory (memory-info)))
            (log-info "Memory info retrieved")
            (if (list? memory)
                (log-info "Memory info is a list as expected")
                (error "Memory info should be a list")))
          
          (let ((up (uptime)))
            (log-info "Uptime retrieved")
            (if (number? up)
                (log-info "Uptime is a number as expected")
                (error "Uptime should be a number")))
          
          (log-info "Steel system info test completed")
          #t)
    "#;
    
    let result = runtime.execute_code_with_hal(system_info_test_code).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_system_monitoring_scenario_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));
    
    // Simulate a monitoring scenario
    println!("Starting system monitoring scenario...");
    
    for i in 0..3 {
        println!("Monitoring iteration {}", i + 1);
        
        // Get system info
        let device_result = api.get_device_info().await;
        let memory_result = api.get_memory_info().await;
        let uptime_result = api.get_uptime().await;
        
        assert!(device_result.is_ok());
        assert!(memory_result.is_ok());
        assert!(uptime_result.is_ok());
        
        // Simulate some activity
        let _ = api.set_led(true).await;
        let _ = api.sleep(0.001).await;
        let _ = api.set_led(false).await;
        
        println!("Iteration {} completed", i + 1);
    }
    
    println!("System monitoring scenario completed successfully");
}