#[cfg(test)]
mod tests {
    use aws_iot_core::RustAPI;
    use crate::MockHAL;
    use std::sync::Arc;
    use std::time::Duration;

    fn create_test_api() -> RustAPI {
        let hal = Arc::new(MockHAL::new());
        RustAPI::new(hal)
    }

    #[tokio::test]
    async fn test_sleep_api_binding() {
        let api = create_test_api();
        
        // Test valid sleep duration
        let result = api.sleep(2.5).await;
        assert!(result.is_ok());
        
        // Test zero duration (should fail)
        let result = api.sleep(0.0).await;
        assert!(result.is_err());
        
        // Test negative duration (should fail)
        let result = api.sleep(-1.0).await;
        assert!(result.is_err());
        
        // Test very large duration (should fail)
        let result = api.sleep(86400.0 * 365.0).await; // 1 year
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_led_api_binding() {
        let api = create_test_api();
        
        // Test LED on
        let result = api.set_led(true).await;
        assert!(result.is_ok());
        
        // Test LED off
        let result = api.set_led(false).await;
        assert!(result.is_ok());
        
        // Test LED state query
        let state = api.get_led_state().await;
        assert!(state.is_ok());
    }

    #[tokio::test]
    async fn test_device_info_api() {
        let api = create_test_api();
        
        let device_info = api.get_device_info().await;
        assert!(device_info.is_ok());
        
        let info = device_info.unwrap();
        assert_eq!(info.device_id, "mock-device-001");
        assert_eq!(info.platform, "Mock");
        assert!(!info.version.is_empty());
    }

    #[tokio::test]
    async fn test_memory_info_api() {
        let api = create_test_api();
        
        let memory_info = api.get_memory_info().await;
        assert!(memory_info.is_ok());
        
        let info = memory_info.unwrap();
        assert!(info.total_bytes > 0);
        assert!(info.free_bytes <= info.total_bytes);
        assert_eq!(info.used_bytes + info.free_bytes, info.total_bytes);
    }

    #[tokio::test]
    async fn test_uptime_api() {
        let api = create_test_api();
        
        let uptime_info = api.get_uptime().await;
        assert!(uptime_info.is_ok());
        
        let uptime_duration = uptime_info.unwrap();
        assert!(uptime_duration >= Duration::from_secs(0));
    }

    #[tokio::test]
    async fn test_secure_storage_api() {
        let api = create_test_api();
        
        let key = "test_key";
        let value = "test_value";
        
        // Test store
        let result = api.store_data(key, value).await;
        assert!(result.is_ok());
        
        // Test load
        let loaded = api.load_data(key).await;
        assert!(loaded.is_ok());
        assert_eq!(loaded.unwrap(), Some(value.to_string()));
        
        // Test load non-existent key
        let missing = api.load_data("non_existent").await;
        assert!(missing.is_ok());
        assert_eq!(missing.unwrap(), None);
        
        // Test delete
        let deleted = api.delete_data(key).await;
        assert!(deleted.is_ok());
        assert!(deleted.unwrap());
        
        // Test delete non-existent key
        let not_deleted = api.delete_data("non_existent").await;
        assert!(not_deleted.is_ok());
        assert!(!not_deleted.unwrap());
    }

    #[tokio::test]
    async fn test_logging_api() {
        let api = create_test_api();
        
        // Test different log levels
        api.log(aws_iot_core::LogLevel::Info, "Test info message").unwrap();
        api.log(aws_iot_core::LogLevel::Warn, "Test warning message").unwrap();
        api.log(aws_iot_core::LogLevel::Error, "Test error message").unwrap();
        api.log(aws_iot_core::LogLevel::Debug, "Test debug message").unwrap();
        
        // Test empty message
        api.log(aws_iot_core::LogLevel::Info, "").unwrap();
        
        // Test long message
        let long_message = "a".repeat(1000);
        api.log(aws_iot_core::LogLevel::Info, &long_message).unwrap();
    }

    #[tokio::test]
    async fn test_mqtt_publish_api() {
        let api = create_test_api();
        
        // Test valid publish
        let result = api.publish_mqtt("test/topic", "test message").await;
        assert!(result.is_ok());
        
        // Test empty topic (should fail)
        let result = api.publish_mqtt("", "test message").await;
        assert!(result.is_err());
        
        // Test empty payload (should be ok)
        let result = api.publish_mqtt("test/topic", "").await;
        assert!(result.is_ok());
        
        // Test invalid topic characters
        let result = api.publish_mqtt("test/topic/with/+/wildcard", "message").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_shadow_update_api() {
        let api = create_test_api();
        
        // Test valid shadow update
        let value = serde_json::json!({"temperature": 25.5, "humidity": 60});
        let result = api.update_shadow("sensor_data", value).await;
        assert!(result.is_ok());
        
        // Test empty key (should fail)
        let value = serde_json::json!({"test": "value"});
        let result = api.update_shadow("", value).await;
        assert!(result.is_err());
        
        // Test null value
        let result = api.update_shadow("null_test", serde_json::Value::Null).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_timer_api() {
        let api = create_test_api();
        
        // Note: This is a simplified test since we can't easily test actual timer callbacks
        // In a real implementation, we'd need a more sophisticated mock system
        
        // Test timer creation with valid duration
        let timer_name = "test_timer";
        let duration = 1.0; // 1 second
        let callback_code = "(log-info \"Timer fired\")";
        
        // For now, just test that the API accepts the call
        // In a full implementation, this would test actual timer functionality
        let result = api.set_timer(timer_name, duration, callback_code);
        // We expect this to work in the mock implementation
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_api_error_handling() {
        let api = create_test_api();
        
        // Test invalid sleep duration
        let result = api.sleep(-1.0).await;
        assert!(result.is_err());
        
        // Test invalid MQTT topic
        let result = api.publish_mqtt("", "message").await;
        assert!(result.is_err());
        
        // Test invalid shadow key
        let result = api.update_shadow("", serde_json::Value::Null).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_api_calls() {
        let api = Arc::new(create_test_api());
        
        let mut handles = Vec::new();
        
        // Test concurrent sleep calls
        for _i in 0..10 {
            let api_clone = api.clone();
            let handle = tokio::spawn(async move {
                api_clone.sleep(0.1).await
            });
            handles.push(handle);
        }
        
        // Test concurrent LED calls
        for i in 0..10 {
            let api_clone = api.clone();
            let state = i % 2 == 0;
            let handle = tokio::spawn(async move {
                api_clone.set_led(state).await
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.expect("Task panicked");
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_api_state_consistency() {
        let api = create_test_api();
        
        // Test LED state consistency
        api.set_led(true).await.expect("Failed to set LED on");
        let state = api.get_led_state().await.expect("Failed to get LED state");
        assert!(state);
        
        api.set_led(false).await.expect("Failed to set LED off");
        let state = api.get_led_state().await.expect("Failed to get LED state");
        assert!(!state);
        
        // Test storage consistency
        let key = "consistency_test";
        let value1 = "value1";
        let value2 = "value2";
        
        api.store_data(key, value1).await.expect("Failed to store data");
        let loaded = api.load_data(key).await.expect("Failed to load data");
        assert_eq!(loaded, Some(value1.to_string()));
        
        api.store_data(key, value2).await.expect("Failed to update data");
        let loaded = api.load_data(key).await.expect("Failed to load updated data");
        assert_eq!(loaded, Some(value2.to_string()));
        
        api.delete_data(key).await.expect("Failed to delete data");
        let loaded = api.load_data(key).await.expect("Failed to load deleted data");
        assert_eq!(loaded, None);
    }
}