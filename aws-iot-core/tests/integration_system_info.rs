use aws_iot_core::*;
use steel_core::rvals::SteelVal;
use std::sync::Arc;
use tokio;

mod common;
use common::MockHAL;

#[tokio::test]
async fn test_device_info_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()).unwrap());
    
    let result = api.device_info().await;
    assert!(result.is_ok());
    
    if let Ok(SteelVal::ListV(info_list)) = result {
        assert!(!info_list.is_empty());
        println!("Device info: {:?}", info_list);
    } else {
        panic!("Expected list result from device_info");
    }
    
    println!("Device info test completed successfully");
}

#[tokio::test]
async fn test_memory_info_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()).unwrap());
    
    let result = api.memory_info().await;
    assert!(result.is_ok());
    
    if let Ok(SteelVal::ListV(info_list)) = result {
        assert!(!info_list.is_empty());
        println!("Memory info: {:?}", info_list);
    } else {
        panic!("Expected list result from memory_info");
    }
    
    println!("Memory info test completed successfully");
}

#[tokio::test]
async fn test_uptime_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()).unwrap());
    
    let result = api.uptime().await;
    assert!(result.is_ok());
    
    if let Ok(SteelVal::NumV(uptime_secs)) = result {
        assert!(uptime_secs > 0.0);
        println!("Uptime: {} seconds", uptime_secs);
    } else {
        panic!("Expected number result from uptime");
    }
    
    println!("Uptime test completed successfully");
}

#[tokio::test]
async fn test_system_info_integration_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()).unwrap());
    
    // Get all system information
    let device_result = api.device_info().await;
    let memory_result = api.memory_info().await;
    let uptime_result = api.uptime().await;
    
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
    let api = Arc::new(RustAPI::new(hal.clone()).unwrap());
    let runtime = SteelRuntime::new(api).unwrap();
    
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
    let api = Arc::new(RustAPI::new(hal.clone()).unwrap());
    
    // Simulate a monitoring scenario
    println!("Starting system monitoring scenario...");
    
    for i in 0..3 {
        println!("Monitoring iteration {}", i + 1);
        
        // Get system info
        let device_result = api.device_info().await;
        let memory_result = api.memory_info().await;
        let uptime_result = api.uptime().await;
        
        assert!(device_result.is_ok());
        assert!(memory_result.is_ok());
        assert!(uptime_result.is_ok());
        
        // Simulate some activity
        let _ = api.led_on().await;
        let _ = api.sleep(0.001).await;
        let _ = api.led_off().await;
        
        println!("Iteration {} completed", i + 1);
    }
    
    println!("System monitoring scenario completed successfully");
}