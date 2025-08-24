use aws_iot_core::steel_runtime::SteelRuntimeAPI;
use aws_iot_core::*;

use std::sync::Arc;
use std::time::Instant;

mod common;
use common::MockHAL;

#[tokio::test]
async fn test_sleep_basic_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));

    // Test short sleep
    let start = Instant::now();
    let result = api.sleep(0.001).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    assert!(hal.sleep_called.load(std::sync::atomic::Ordering::SeqCst));
    println!("Short sleep completed in {:?}", elapsed);

    // Test very small sleep (should work)
    hal.sleep_called
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let result = api.sleep(0.001).await;
    assert!(result.is_ok());
    assert!(hal.sleep_called.load(std::sync::atomic::Ordering::SeqCst));

    println!("Sleep basic test completed successfully");
}

#[tokio::test]
async fn test_sleep_error_handling_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));

    // Test negative sleep (should fail)
    let result = api.sleep(-1.0).await;
    assert!(result.is_err());

    // Verify error message
    if let Err(e) = result {
        assert!(e.to_string().contains("positive"));
    }

    // Test zero sleep (should also fail)
    let result = api.sleep(0.0).await;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("positive"));
    }

    println!("Sleep error handling test completed successfully");
}

#[tokio::test]
async fn test_sleep_timing_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));

    // Test multiple short sleeps
    let start = Instant::now();
    for i in 0..5 {
        let result = api.sleep(0.001).await;
        assert!(result.is_ok());
        println!("Sleep {} completed", i + 1);
    }
    let elapsed = start.elapsed();

    println!("Multiple sleeps completed in {:?}", elapsed);
    println!("Sleep timing test completed successfully");
}

#[tokio::test]
async fn test_sleep_with_steel_runtime() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(SteelRuntimeAPI::new(hal.clone()).unwrap());
    let runtime = SteelRuntimeImpl::new(api).unwrap();

    // Test sleep operations through Steel runtime
    let sleep_test_code = r#"
        (begin
          (log-info "Starting Steel sleep test")
          (sleep 0.001)
          (log-info "First sleep completed")
          (sleep 0)
          (log-info "Zero sleep completed")
          (log-info "Steel sleep test completed")
          #t)
    "#;

    let result = runtime.execute_code_with_hal(sleep_test_code).await;
    assert!(result.is_ok());

    // Verify sleep was called
    assert!(hal.sleep_called.load(std::sync::atomic::Ordering::SeqCst));
}

#[tokio::test]
async fn test_sleep_with_led_integration_rust() {
    let hal = Arc::new(MockHAL::new());
    let api = Arc::new(RustAPI::new(hal.clone()));

    // Test sleep combined with LED operations
    let result = api.led_on().await;
    assert!(result.is_ok());

    let result = api.sleep(0.001).await;
    assert!(result.is_ok());

    let result = api.led_off().await;
    assert!(result.is_ok());

    let result = api.sleep(0.001).await;
    assert!(result.is_ok());

    // Verify final state
    assert_eq!(*hal.led_state.lock(), LedState::Off);
    assert!(hal.sleep_called.load(std::sync::atomic::Ordering::SeqCst));

    println!("Sleep with LED integration test completed successfully");
}
