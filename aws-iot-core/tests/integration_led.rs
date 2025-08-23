use aws_iot_core::*;
use steel_core::rvals::SteelVal;
use std::sync::Arc;
use tokio;

mod common;
use common::MockHAL;

#[tokio::test]
async fn test_led_basic_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));
    
    // Test LED on
    let result = api.led_on().await;
    assert!(result.is_ok());
    assert_eq!(*hal.led_state.lock(), LedState::On);
    
    // Test LED off
    let result = api.led_off().await;
    assert!(result.is_ok());
    assert_eq!(*hal.led_state.lock(), LedState::Off);
    
    // Test LED state query
    let result = api.led_state().await;
    assert!(result.is_ok());
    if let Ok(SteelVal::BoolV(state)) = result {
        assert!(!state); // Should be false since we turned it off
    } else {
        panic!("Expected boolean result from led_state");
    }
}

#[tokio::test]
async fn test_led_sequence_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));
    
    // Blink LED 3 times
    for i in 0..3 {
        println!("Blink {}", i + 1);
        
        // Turn on
        let result = api.led_on().await;
        assert!(result.is_ok());
        assert_eq!(*hal.led_state.lock(), LedState::On);
        
        // Short delay
        let result = api.sleep(0.001).await;
        assert!(result.is_ok());
        
        // Turn off
        let result = api.led_off().await;
        assert!(result.is_ok());
        assert_eq!(*hal.led_state.lock(), LedState::Off);
        
        // Short delay
        let result = api.sleep(0.001).await;
        assert!(result.is_ok());
    }
    
    println!("LED sequence test completed successfully");
}

#[tokio::test]
async fn test_led_with_steel_runtime() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));
    let runtime = SteelRuntime::new(api).unwrap();
    
    // Test LED operations through Steel runtime
    let led_test_code = r#"
        (begin
          (log-info "Starting Steel LED test")
          (led-on)
          (let ((state (led-state)))
            (if state
                (log-info "LED is on as expected")
                (error "LED should be on")))
          (led-off)
          (let ((state (led-state)))
            (if (not state)
                (log-info "LED is off as expected")
                (error "LED should be off")))
          (log-info "Steel LED test completed")
          #t)
    "#;
    
    let result = runtime.execute_code_with_hal(led_test_code).await;
    if let Err(e) = &result {
        println!("Steel LED test error: {}", e);
    }
    assert!(result.is_ok());
    
    // Note: This test verifies Steel program execution logic
    // HAL integration is tested separately in unit tests
}

#[tokio::test]
async fn test_led_error_conditions_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));
    
    // Test that LED operations work normally
    let result = api.led_on().await;
    assert!(result.is_ok());
    
    let result = api.led_off().await;
    assert!(result.is_ok());
    
    let result = api.led_state().await;
    assert!(result.is_ok());
    
    println!("LED error conditions test completed successfully");
}