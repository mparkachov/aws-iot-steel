#[cfg(test)]
mod tests {
    use super::super::MockHAL;
    use aws_iot_core::{PlatformHAL, LedState};
    use aws_iot_platform_macos::MacOSHAL;
    use std::time::Duration;

    #[tokio::test]
    async fn test_mock_hal_sleep() {
        let mut hal = MockHAL::new();
        hal.initialize().await.unwrap();
        
        let duration = Duration::from_secs(5);
        hal.sleep(duration).await.unwrap();
        
        let sleep_calls = hal.get_sleep_calls().await;
        assert_eq!(sleep_calls.len(), 1);
        assert_eq!(sleep_calls[0], duration);
    }

    #[tokio::test]
    async fn test_mock_hal_led_control() {
        let mut hal = MockHAL::new();
        hal.initialize().await.unwrap();
        
        // Test LED on
        hal.set_led(LedState::On).await.unwrap();
        assert_eq!(hal.get_led_state().await.unwrap(), LedState::On);
        
        // Test LED off
        hal.set_led(LedState::Off).await.unwrap();
        assert_eq!(hal.get_led_state().await.unwrap(), LedState::Off);
        
        let led_calls = hal.get_led_calls().await;
        assert_eq!(led_calls.len(), 2);
        assert_eq!(led_calls[0], LedState::On);
        assert_eq!(led_calls[1], LedState::Off);
    }

    #[tokio::test]
    async fn test_mock_hal_secure_storage() {
        let mut hal = MockHAL::new();
        hal.initialize().await.unwrap();
        
        let key = "test_key";
        let data = b"test_data";
        
        // Store data
        hal.store_secure_data(key, data).await.unwrap();
        
        // Load data
        let loaded_data = hal.load_secure_data(key).await.unwrap();
        assert_eq!(loaded_data, Some(data.to_vec()));
        
        // List keys
        let keys = hal.list_secure_keys().await.unwrap();
        assert!(keys.contains(&key.to_string()));
        
        // Delete data
        let deleted = hal.delete_secure_data(key).await.unwrap();
        assert!(deleted);
        
        // Verify deletion
        let loaded_after_delete = hal.load_secure_data(key).await.unwrap();
        assert_eq!(loaded_after_delete, None);
    }

    #[tokio::test]
    async fn test_mock_hal_device_info() {
        let mut hal = MockHAL::new();
        hal.initialize().await.unwrap();
        
        let device_info = hal.get_device_info().await.unwrap();
        assert_eq!(device_info.platform, "Mock");
        assert_eq!(device_info.device_id, "mock-device-001");
    }

    #[tokio::test]
    async fn test_mock_hal_memory_info() {
        let mut hal = MockHAL::new();
        hal.initialize().await.unwrap();
        
        let memory_info = hal.get_memory_info().await.unwrap();
        assert_eq!(memory_info.total_bytes, 1_048_576);
        assert_eq!(memory_info.free_bytes, 524_288);
        assert_eq!(memory_info.used_bytes, 524_288);
        assert_eq!(memory_info.usage_percentage(), 50.0);
    }

    #[tokio::test]
    async fn test_macos_hal_basic_operations() {
        let mut hal = MacOSHAL::new();
        hal.initialize().await.unwrap();
        
        // Test device info
        let device_info = hal.get_device_info().await.unwrap();
        assert!(device_info.platform.starts_with("macOS"));
        
        // Test LED operations
        hal.set_led(LedState::On).await.unwrap();
        assert_eq!(hal.get_led_state().await.unwrap(), LedState::On);
        
        hal.set_led(LedState::Off).await.unwrap();
        assert_eq!(hal.get_led_state().await.unwrap(), LedState::Off);
        
        // Test memory info
        let memory_info = hal.get_memory_info().await.unwrap();
        assert!(memory_info.total_bytes > 0);
        assert!(memory_info.free_bytes > 0);
        
        hal.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_macos_hal_secure_storage() {
        let mut hal = MacOSHAL::new();
        hal.initialize().await.unwrap();
        
        let key = "test_secure_key";
        let data = b"secure_test_data";
        
        // Store data
        hal.store_secure_data(key, data).await.unwrap();
        
        // Load data
        let loaded_data = hal.load_secure_data(key).await.unwrap();
        assert_eq!(loaded_data, Some(data.to_vec()));
        
        // List keys
        let keys = hal.list_secure_keys().await.unwrap();
        assert!(keys.contains(&key.to_string()));
        
        // Delete data
        let deleted = hal.delete_secure_data(key).await.unwrap();
        assert!(deleted);
        
        // Verify deletion
        let loaded_after_delete = hal.load_secure_data(key).await.unwrap();
        assert_eq!(loaded_after_delete, None);
        
        hal.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_hal_initialization_lifecycle() {
        let mut hal = MockHAL::new();
        
        // Test double initialization fails
        hal.initialize().await.unwrap();
        assert!(hal.initialize().await.is_err());
        
        // Test shutdown
        hal.shutdown().await.unwrap();
        
        // Test double shutdown fails
        assert!(hal.shutdown().await.is_err());
        
        // Test operations after shutdown fail (for some implementations)
        // Note: MockHAL doesn't enforce this, but real implementations might
    }
}