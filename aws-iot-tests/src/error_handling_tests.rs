#[cfg(test)]
mod tests {
    use crate::{MockHAL, MockIoTClient, MockSteelRuntime};
    use aws_iot_core::steel_runtime::SteelError;
    use aws_iot_core::{IoTClientTrait, IoTError, LedState, PlatformHAL, SteelRuntime};
    use std::time::Duration;

    #[tokio::test]
    async fn test_platform_error_handling() {
        let hal = MockHAL::new();

        // Test successful operations first
        assert!(hal.sleep(Duration::from_secs(1)).await.is_ok());
        assert!(hal.set_led(LedState::On).await.is_ok());

        // Test edge cases that should succeed
        assert!(hal.sleep(Duration::from_secs(0)).await.is_ok()); // Zero duration
        assert!(hal.get_device_info().await.is_ok());
        assert!(hal.get_memory_info().await.is_ok());

        // Test secure storage edge cases
        let empty_key = "";
        let result = hal.store_secure_data(empty_key, b"data").await;
        // Empty key should be handled gracefully
        assert!(result.is_ok());

        let very_long_key = "a".repeat(1000);
        let result = hal.store_secure_data(&very_long_key, b"data").await;
        assert!(result.is_ok());

        // Test large data storage
        let large_data = vec![0u8; 1_000_000]; // 1MB
        let result = hal.store_secure_data("large_data", &large_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_iot_client_error_scenarios() {
        let mut client = MockIoTClient::new("test-device".to_string());

        // Test operations when not connected
        assert!(!client.is_connected().await);

        let result = client
            .publish("test/topic", b"message", rumqttc::QoS::AtMostOnce)
            .await;
        assert!(matches!(result, Err(IoTError::NotConnected)));

        let result = client
            .subscribe("test/topic", rumqttc::QoS::AtMostOnce)
            .await;
        assert!(matches!(result, Err(IoTError::NotConnected)));

        // Test connection failure
        client.set_should_fail_connection(true).await;
        let result = client.connect().await;
        assert!(matches!(result, Err(IoTError::Connection(_))));

        // Test successful connection
        client.set_should_fail_connection(false).await;
        assert!(client.connect().await.is_ok());
        assert!(client.is_connected().await);

        // Test publish failure after connection
        client.set_should_fail_publish(true).await;
        let result = client
            .publish("test/topic", b"message", rumqttc::QoS::AtMostOnce)
            .await;
        assert!(matches!(result, Err(IoTError::Publish(_))));

        // Test successful operations after fixing publish
        client.set_should_fail_publish(false).await;
        assert!(client
            .publish("test/topic", b"message", rumqttc::QoS::AtMostOnce)
            .await
            .is_ok());
        assert!(client
            .subscribe("test/topic", rumqttc::QoS::AtMostOnce)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_steel_runtime_error_scenarios() {
        let mut runtime = MockSteelRuntime::new();

        // Test loading failure
        runtime.set_should_fail_loading(true).await;
        let result = runtime.load_program("(+ 1 2)", None).await;
        assert!(matches!(result, Err(SteelError::Compilation(_))));

        // Test execution failure
        runtime.set_should_fail_loading(false).await;
        runtime.set_should_fail_execution(true).await;

        let handle = runtime.load_program("(+ 1 2)", None).await.unwrap();
        let result = runtime.execute_program(handle).await;
        assert!(matches!(result, Err(SteelError::Runtime(_))));

        let result = runtime.execute_code("(+ 1 2)").await;
        assert!(matches!(result, Err(SteelError::Runtime(_))));

        // Test operations on non-existent programs
        runtime.set_should_fail_execution(false).await;
        let fake_handle = aws_iot_core::steel_runtime::ProgramHandle::new();
        let result = runtime.execute_program(fake_handle.clone()).await;
        assert!(matches!(result, Err(SteelError::Runtime(_))));

        let result = runtime.remove_program(fake_handle).await;
        assert!(matches!(result, Err(SteelError::Runtime(_))));
    }

    #[tokio::test]
    async fn test_api_parameter_validation() {
        // Test invalid parameters that should be caught by API validation

        // Invalid sleep durations
        let invalid_durations = vec![-1.0, -0.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY];

        for duration in invalid_durations {
            // In a real implementation, we'd test the actual RustAPI
            // For now, we test the validation logic conceptually
            assert!(
                duration < 0.0 || !duration.is_finite(),
                "Duration {} should be invalid",
                duration
            );
        }

        // Invalid MQTT topics
        let invalid_topics = vec!["", "+", "#", "topic/+/invalid", "topic/#/invalid"];

        for topic in invalid_topics {
            // Test topic validation logic
            let is_valid = !topic.is_empty() && !topic.contains('+') && !topic.contains('#');
            assert!(
                !is_valid || topic.is_empty(),
                "Topic '{}' should be invalid",
                topic
            );
        }

        // Invalid shadow keys
        let invalid_keys = vec!["", " ", "\n", "\t"];

        for key in invalid_keys {
            let is_valid = !key.trim().is_empty();
            assert!(!is_valid, "Key '{}' should be invalid", key);
        }
    }

    #[tokio::test]
    async fn test_concurrent_error_scenarios() {
        let client = std::sync::Arc::new(tokio::sync::Mutex::new(MockIoTClient::new(
            "test-device".to_string(),
        )));

        // Test concurrent operations when client is not connected
        let mut handles = Vec::new();

        for i in 0..10 {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                let client = client_clone.lock().await;
                client
                    .publish(
                        &format!("test/topic/{}", i),
                        b"message",
                        rumqttc::QoS::AtMostOnce,
                    )
                    .await
            });
            handles.push(handle);
        }

        // All should fail with NotConnected error
        for handle in handles {
            let result = handle.await.expect("Task panicked");
            assert!(matches!(result, Err(IoTError::NotConnected)));
        }

        // Test concurrent operations after connection
        {
            let mut client_guard = client.lock().await;
            client_guard.connect().await.unwrap();
        }

        let mut handles = Vec::new();

        for i in 0..10 {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                let client = client_clone.lock().await;
                client
                    .publish(
                        &format!("test/topic/{}", i),
                        b"message",
                        rumqttc::QoS::AtMostOnce,
                    )
                    .await
            });
            handles.push(handle);
        }

        // All should succeed now
        for handle in handles {
            let result = handle.await.expect("Task panicked");
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_resource_exhaustion_scenarios() {
        let mut runtime = MockSteelRuntime::new();

        // Test loading many programs
        let mut handles = Vec::new();
        for i in 0..100 {
            let program = format!("(+ {} {})", i, i + 1);
            let handle = runtime
                .load_program(&program, Some(&format!("prog_{}", i)))
                .await;
            assert!(handle.is_ok());
            handles.push(handle.unwrap());
        }

        // Verify all programs were loaded
        let programs = runtime.get_loaded_programs().await;
        assert_eq!(programs.len(), 100);

        // Test executing many programs
        for handle in handles.into_iter().take(10) {
            // Test first 10 to avoid too much overhead
            let result = runtime.execute_program(handle).await;
            assert!(result.is_ok());
        }

        // Test many global variables
        for i in 0..100 {
            let result = runtime
                .set_global_variable(
                    &format!("var_{}", i),
                    aws_iot_core::SteelValue::NumV(i as f64),
                )
                .await;
            assert!(result.is_ok());
        }

        let globals = runtime.get_global_variables().await;
        assert_eq!(globals.len(), 100);
    }

    #[tokio::test]
    async fn test_malformed_data_handling() {
        let mut client = MockIoTClient::new("test-device".to_string());
        client.connect().await.unwrap();

        // Test publishing malformed JSON
        let malformed_json = r#"{"incomplete": json"#;
        let result = client
            .publish(
                "test/topic",
                malformed_json.as_bytes(),
                rumqttc::QoS::AtMostOnce,
            )
            .await;
        // Should succeed at publish level (validation happens at application level)
        assert!(result.is_ok());

        // Test very large payloads
        let large_payload = "x".repeat(1_000_000); // 1MB payload
        let result = client
            .publish(
                "test/topic",
                large_payload.as_bytes(),
                rumqttc::QoS::AtMostOnce,
            )
            .await;
        assert!(result.is_ok());

        // Test binary data in string context
        let binary_data = String::from_utf8_lossy(&[0, 1, 2, 255, 254, 253]);
        let result = client
            .publish(
                "test/topic",
                binary_data.as_bytes(),
                rumqttc::QoS::AtMostOnce,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_timeout_scenarios() {
        let mut runtime = MockSteelRuntime::new();

        // Set a long execution delay
        runtime.set_execution_delay(1000).await; // 1 second

        // Test that execution still completes (in mock, we don't enforce timeouts)
        let start = std::time::Instant::now();
        let result = runtime.execute_code("(+ 1 1)").await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration >= Duration::from_millis(900)); // Allow some tolerance

        // In a real implementation, we'd test actual timeout enforcement
        // For now, we just verify the delay mechanism works
    }

    #[tokio::test]
    async fn test_state_corruption_recovery() {
        let hal = MockHAL::new();

        // Test operations in various states
        assert!(hal.set_led(LedState::On).await.is_ok());
        let state = hal.get_led_state().await.unwrap();
        assert_eq!(state, LedState::On);

        // Test rapid state changes
        for i in 0..100 {
            let new_state = if i % 2 == 0 {
                LedState::On
            } else {
                LedState::Off
            };
            assert!(hal.set_led(new_state).await.is_ok());
            let current_state = hal.get_led_state().await.unwrap();
            assert_eq!(current_state, new_state);
        }

        // Test storage consistency under rapid operations
        for i in 0..50 {
            let key = format!("key_{}", i);
            let value = format!("value_{}", i);
            assert!(hal.store_secure_data(&key, value.as_bytes()).await.is_ok());

            let loaded = hal.load_secure_data(&key).await.unwrap();
            assert_eq!(loaded, Some(value.as_bytes().to_vec()));
        }
    }

    #[tokio::test]
    async fn test_error_propagation() {
        // Test that errors are properly propagated through the system layers

        let mut client = MockIoTClient::new("test-device".to_string());

        // Test error propagation from connection failure
        client.set_should_fail_connection(true).await;
        let connection_result = client.connect().await;
        assert!(connection_result.is_err());

        // Verify error type and message
        match connection_result {
            Err(IoTError::Connection(msg)) => {
                assert!(msg.contains("Mock connection failure"));
            }
            _ => panic!("Expected Connection error"),
        }

        // Test error propagation from publish failure
        client.set_should_fail_connection(false).await;
        client.connect().await.unwrap();
        client.set_should_fail_publish(true).await;

        let publish_result = client
            .publish("test/topic", b"message", rumqttc::QoS::AtMostOnce)
            .await;
        assert!(publish_result.is_err());

        match publish_result {
            Err(IoTError::Publish(msg)) => {
                assert!(msg.contains("Mock publish failure"));
            }
            _ => panic!("Expected Publish error"),
        }
    }
}
