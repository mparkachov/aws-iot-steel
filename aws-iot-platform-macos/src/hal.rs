use async_trait::async_trait;
use aws_iot_core::{
    DeviceInfo, LedState, MemoryInfo, PlatformError, PlatformHAL, PlatformResult, UptimeInfo,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
// Removed unused imports
use crate::system_monitor::MacOSSystemMonitor;

// Note: For now using file-based storage as fallback
// TODO: Implement proper macOS Keychain integration

/// ANSI color codes for colored console output
mod colors {
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
}

/// macOS implementation of the Platform HAL
/// This provides simulation of ESP32 hardware for development and testing
pub struct MacOSHAL {
    led_state: Arc<Mutex<LedState>>,
    system_monitor: MacOSSystemMonitor,
    initialized: Arc<Mutex<bool>>,
}

impl MacOSHAL {
    /// Create a new macOS HAL instance
    pub fn new() -> Self {
        Self {
            led_state: Arc::new(Mutex::new(LedState::Off)),
            system_monitor: MacOSSystemMonitor::new(),
            initialized: Arc::new(Mutex::new(false)),
        }
    }
}

impl Default for MacOSHAL {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlatformHAL for MacOSHAL {
    async fn sleep(&self, duration: Duration) -> PlatformResult<()> {
        println!(
            "{}{}💤 SLEEP:{} Sleeping for {:?}{}",
            colors::BOLD,
            colors::BLUE,
            colors::RESET,
            duration,
            colors::RESET
        );

        let start_time = std::time::Instant::now();
        tokio::time::sleep(duration).await;
        let actual_duration = start_time.elapsed();

        println!(
            "{}{}⏰ WAKE:{} Sleep completed (actual: {:?}){}",
            colors::BOLD,
            colors::GREEN,
            colors::RESET,
            actual_duration,
            colors::RESET
        );

        tracing::info!(
            "Sleep operation completed: requested={:?}, actual={:?}",
            duration,
            actual_duration
        );
        Ok(())
    }

    async fn set_led(&self, state: LedState) -> PlatformResult<()> {
        let mut current_state = self.led_state.lock().await;
        let previous_state = *current_state;
        *current_state = state;

        match state {
            LedState::On => {
                println!(
                    "{}{}💡 LED:{} {} ON {} (was: {}){}",
                    colors::BOLD,
                    colors::YELLOW,
                    colors::RESET,
                    colors::GREEN,
                    colors::BOLD,
                    if previous_state == LedState::On {
                        "ON"
                    } else {
                        "OFF"
                    },
                    colors::RESET
                );
            }
            LedState::Off => {
                println!(
                    "{}{}🔌 LED:{} {} OFF {} (was: {}){}",
                    colors::BOLD,
                    colors::YELLOW,
                    colors::RESET,
                    colors::RED,
                    colors::BOLD,
                    if previous_state == LedState::On {
                        "ON"
                    } else {
                        "OFF"
                    },
                    colors::RESET
                );
            }
        }

        tracing::info!("LED state changed: {:?} -> {:?}", previous_state, state);
        Ok(())
    }

    async fn get_led_state(&self) -> PlatformResult<LedState> {
        let state = *self.led_state.lock().await;
        Ok(state)
    }

    async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
        self.system_monitor.get_device_info().await
    }

    async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
        self.system_monitor.get_memory_info().await
    }

    async fn get_uptime(&self) -> PlatformResult<UptimeInfo> {
        self.system_monitor.get_uptime().await
    }

    async fn store_secure_data(&self, key: &str, data: &[u8]) -> PlatformResult<()> {
        println!(
            "{}{}🔐 STORAGE:{} Storing secure data for key: '{}' ({} bytes){}",
            colors::BOLD,
            colors::MAGENTA,
            colors::RESET,
            key,
            data.len(),
            colors::RESET
        );

        // Use file-based secure storage (encrypted in production)
        let storage_dir = std::env::temp_dir().join("aws-iot-secure");
        tokio::fs::create_dir_all(&storage_dir).await.map_err(|e| {
            PlatformError::Storage(format!("Failed to create storage directory: {}", e))
        })?;

        let file_path = storage_dir.join(format!("{}.dat", key));
        tokio::fs::write(&file_path, data)
            .await
            .map_err(|e| PlatformError::Storage(format!("Failed to write secure data: {}", e)))?;

        println!(
            "{}{}✅ KEYCHAIN:{} Secure data stored successfully for key: '{}'{}",
            colors::BOLD,
            colors::GREEN,
            colors::RESET,
            key,
            colors::RESET
        );
        tracing::debug!("Secure data stored successfully for key: {}", key);
        Ok(())
    }

    async fn load_secure_data(&self, key: &str) -> PlatformResult<Option<Vec<u8>>> {
        println!(
            "{}{}🔓 STORAGE:{} Loading secure data for key: '{}'{}",
            colors::BOLD,
            colors::CYAN,
            colors::RESET,
            key,
            colors::RESET
        );

        // Use file-based secure storage (encrypted in production)
        let storage_dir = std::env::temp_dir().join("aws-iot-secure");
        let file_path = storage_dir.join(format!("{}.dat", key));

        match tokio::fs::read(&file_path).await {
            Ok(data) => {
                println!(
                    "{}{}✅ STORAGE:{} Secure data loaded successfully for key: '{}' ({} bytes){}",
                    colors::BOLD,
                    colors::GREEN,
                    colors::RESET,
                    key,
                    data.len(),
                    colors::RESET
                );
                tracing::debug!("Secure data loaded successfully for key: {}", key);
                Ok(Some(data))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!(
                    "{}{}❌ STORAGE:{} No secure data found for key: '{}'{}",
                    colors::BOLD,
                    colors::RED,
                    colors::RESET,
                    key,
                    colors::RESET
                );
                tracing::debug!("No secure data found for key: {}", key);
                Ok(None)
            }
            Err(e) => Err(PlatformError::Storage(format!(
                "Failed to read secure data: {}",
                e
            ))),
        }
    }

    async fn delete_secure_data(&self, key: &str) -> PlatformResult<bool> {
        println!(
            "{}{}🗑️ STORAGE:{} Deleting secure data for key: '{}'{}",
            colors::BOLD,
            colors::RED,
            colors::RESET,
            key,
            colors::RESET
        );

        // Use file-based secure storage (encrypted in production)
        let storage_dir = std::env::temp_dir().join("aws-iot-secure");
        let file_path = storage_dir.join(format!("{}.dat", key));

        match tokio::fs::remove_file(&file_path).await {
            Ok(()) => {
                println!(
                    "{}{}✅ STORAGE:{} Secure data deleted successfully for key: '{}'{}",
                    colors::BOLD,
                    colors::GREEN,
                    colors::RESET,
                    key,
                    colors::RESET
                );
                tracing::debug!("Secure data deleted successfully for key: {}", key);
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!(
                    "{}{}❌ STORAGE:{} No secure data found to delete for key: '{}'{}",
                    colors::BOLD,
                    colors::YELLOW,
                    colors::RESET,
                    key,
                    colors::RESET
                );
                tracing::debug!("No secure data found to delete for key: {}", key);
                Ok(false)
            }
            Err(e) => Err(PlatformError::Storage(format!(
                "Failed to delete secure data: {}",
                e
            ))),
        }
    }

    async fn list_secure_keys(&self) -> PlatformResult<Vec<String>> {
        tracing::debug!("📋 Listing secure storage keys");

        // Note: macOS Keychain doesn't provide an easy way to list all keys for a service
        // For now, we'll use a fallback file-based approach to track keys
        let storage_dir = std::env::temp_dir().join("aws-iot-secure");

        let mut keys = Vec::new();

        match tokio::fs::read_dir(&storage_dir).await {
            Ok(mut entries) => {
                while let Some(entry) = entries.next_entry().await.map_err(|e| {
                    PlatformError::Storage(format!("Failed to read directory entry: {}", e))
                })? {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.ends_with(".dat") {
                            let key = file_name.trim_end_matches(".dat").to_string();
                            keys.push(key);
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Directory doesn't exist, return empty list
                tracing::debug!("📂 Secure storage directory doesn't exist, returning empty list");
            }
            Err(e) => {
                return Err(PlatformError::Storage(format!(
                    "Failed to list secure keys: {}",
                    e
                )))
            }
        }

        tracing::debug!("✅ Found {} secure storage keys", keys.len());
        Ok(keys)
    }

    async fn initialize(&mut self) -> PlatformResult<()> {
        let mut initialized = self.initialized.lock().await;
        if *initialized {
            return Err(PlatformError::Hardware(
                "HAL already initialized".to_string(),
            ));
        }

        println!(
            "{}{}🚀 INIT:{} Initializing macOS HAL...{}",
            colors::BOLD,
            colors::BLUE,
            colors::RESET,
            colors::RESET
        );

        // Create secure storage directory
        let storage_dir = std::env::temp_dir().join("aws-iot-secure");
        tokio::fs::create_dir_all(&storage_dir).await.map_err(|e| {
            PlatformError::Hardware(format!("Failed to create storage directory: {}", e))
        })?;

        println!(
            "{}{}🔐 STORAGE:{} Secure storage initialized{}",
            colors::BOLD,
            colors::GREEN,
            colors::RESET,
            colors::RESET
        );

        // Initialize LED state
        *self.led_state.lock().await = LedState::Off;

        *initialized = true;
        println!(
            "{}{}✅ INIT:{} macOS HAL initialized successfully{}",
            colors::BOLD,
            colors::GREEN,
            colors::RESET,
            colors::RESET
        );
        tracing::info!("macOS HAL initialized successfully");
        Ok(())
    }

    async fn shutdown(&mut self) -> PlatformResult<()> {
        let mut initialized = self.initialized.lock().await;
        if !*initialized {
            return Err(PlatformError::Hardware("HAL not initialized".to_string()));
        }

        println!(
            "{}{}🛑 SHUTDOWN:{} Shutting down macOS HAL...{}",
            colors::BOLD,
            colors::RED,
            colors::RESET,
            colors::RESET
        );

        // Reset LED state
        *self.led_state.lock().await = LedState::Off;
        println!(
            "{}{}🔌 LED:{} LED turned OFF during shutdown{}",
            colors::BOLD,
            colors::YELLOW,
            colors::RESET,
            colors::RESET
        );

        *initialized = false;
        println!(
            "{}{}✅ SHUTDOWN:{} macOS HAL shutdown successfully{}",
            colors::BOLD,
            colors::GREEN,
            colors::RESET,
            colors::RESET
        );
        tracing::info!("macOS HAL shutdown successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_iot_core::{LedState, PlatformHAL};
    use chrono::Utc;
    use std::time::{Duration, Instant};
    use tokio::test;

    /// Helper function to create a test HAL instance
    async fn create_test_hal() -> MacOSHAL {
        MacOSHAL::new()
    }

    /// Helper function to create and initialize a test HAL instance
    async fn create_initialized_hal() -> MacOSHAL {
        let mut hal = create_test_hal().await;
        hal.initialize().await.expect("Failed to initialize HAL");
        hal
    }

    #[test]
    async fn test_hal_creation() {
        let hal = create_test_hal().await;

        // Verify initial state
        let led_state = hal.get_led_state().await.expect("Failed to get LED state");
        assert_eq!(led_state, LedState::Off);
    }

    #[test]
    async fn test_hal_initialization() {
        let mut hal = create_test_hal().await;

        // Test successful initialization
        let result = hal.initialize().await;
        assert!(result.is_ok(), "HAL initialization should succeed");

        // Test double initialization fails
        let result = hal.initialize().await;
        assert!(result.is_err(), "Double initialization should fail");

        // Test shutdown
        let result = hal.shutdown().await;
        assert!(result.is_ok(), "HAL shutdown should succeed");
    }

    #[test]
    async fn test_sleep_functionality() {
        let hal = create_initialized_hal().await;

        // Test short sleep
        let sleep_duration = Duration::from_millis(100);
        let start_time = Instant::now();

        let result = hal.sleep(sleep_duration).await;
        let elapsed = start_time.elapsed();

        assert!(result.is_ok(), "Sleep operation should succeed");
        assert!(
            elapsed >= sleep_duration,
            "Sleep should take at least the requested duration"
        );
        assert!(
            elapsed < sleep_duration + Duration::from_millis(50),
            "Sleep should not take much longer than requested"
        );
    }

    #[test]
    async fn test_sleep_zero_duration() {
        let hal = create_initialized_hal().await;

        let result = hal.sleep(Duration::ZERO).await;
        assert!(result.is_ok(), "Zero duration sleep should succeed");
    }

    #[test]
    async fn test_led_state_management() {
        let hal = create_initialized_hal().await;

        // Initial state should be OFF
        let initial_state = hal
            .get_led_state()
            .await
            .expect("Failed to get initial LED state");
        assert_eq!(initial_state, LedState::Off);

        // Turn LED ON
        let result = hal.set_led(LedState::On).await;
        assert!(result.is_ok(), "Setting LED to ON should succeed");

        let current_state = hal
            .get_led_state()
            .await
            .expect("Failed to get LED state after turning ON");
        assert_eq!(current_state, LedState::On);

        // Turn LED OFF
        let result = hal.set_led(LedState::Off).await;
        assert!(result.is_ok(), "Setting LED to OFF should succeed");

        let current_state = hal
            .get_led_state()
            .await
            .expect("Failed to get LED state after turning OFF");
        assert_eq!(current_state, LedState::Off);
    }

    #[test]
    async fn test_led_state_transitions() {
        let hal = create_initialized_hal().await;

        // Test multiple state transitions
        let transitions = vec![
            LedState::On,
            LedState::Off,
            LedState::On,
            LedState::On, // Same state
            LedState::Off,
            LedState::Off, // Same state
        ];

        for expected_state in transitions {
            let result = hal.set_led(expected_state).await;
            assert!(result.is_ok(), "LED state transition should succeed");

            let actual_state = hal.get_led_state().await.expect("Failed to get LED state");
            assert_eq!(
                actual_state, expected_state,
                "LED state should match expected state"
            );
        }
    }

    #[test]
    async fn test_device_info() {
        let hal = create_initialized_hal().await;

        let device_info = hal
            .get_device_info()
            .await
            .expect("Failed to get device info");

        // Verify device info structure
        assert!(
            !device_info.device_id.is_empty(),
            "Device ID should not be empty"
        );
        assert!(
            device_info.platform.contains("macOS"),
            "Platform should contain 'macOS'"
        );
        assert!(
            !device_info.version.is_empty(),
            "Version should not be empty"
        );
        assert!(
            !device_info.firmware_version.is_empty(),
            "Firmware version should not be empty"
        );
        assert!(
            device_info.hardware_revision.is_some(),
            "Hardware revision should be present"
        );
        assert!(
            device_info.serial_number.is_some(),
            "Serial number should be present"
        );

        // Verify device ID format
        assert!(
            device_info.device_id.starts_with("macos-"),
            "Device ID should start with 'macos-'"
        );
    }

    #[test]
    async fn test_memory_info() {
        let hal = create_initialized_hal().await;

        let memory_info = hal
            .get_memory_info()
            .await
            .expect("Failed to get memory info");

        // Verify memory info structure
        assert!(
            memory_info.total_bytes > 0,
            "Total memory should be greater than 0"
        );
        assert!(
            memory_info.free_bytes > 0,
            "Free memory should be greater than 0"
        );
        assert!(
            memory_info.used_bytes > 0,
            "Used memory should be greater than 0"
        );
        assert!(
            memory_info.largest_free_block > 0,
            "Largest free block should be greater than 0"
        );

        // Verify memory calculations
        assert_eq!(
            memory_info.total_bytes,
            memory_info.free_bytes + memory_info.used_bytes,
            "Total memory should equal free + used memory"
        );

        assert!(
            memory_info.largest_free_block <= memory_info.free_bytes,
            "Largest free block should not exceed free memory"
        );

        // Test usage percentage calculation
        let usage_percentage = memory_info.usage_percentage();
        assert!(
            (0.0..=100.0).contains(&usage_percentage),
            "Usage percentage should be between 0 and 100"
        );
    }

    #[test]
    async fn test_uptime_info() {
        let hal = create_initialized_hal().await;

        // Wait a small amount to ensure uptime is measurable
        tokio::time::sleep(Duration::from_millis(10)).await;

        let uptime_info = hal.get_uptime().await.expect("Failed to get uptime info");

        // Verify uptime info structure
        assert!(
            uptime_info.uptime > Duration::ZERO,
            "Uptime should be greater than zero"
        );
        assert!(
            uptime_info.boot_time <= Utc::now(),
            "Boot time should be in the past or now"
        );

        // Verify uptime is reasonable (should be small for test)
        assert!(
            uptime_info.uptime < Duration::from_secs(60),
            "Uptime should be less than 60 seconds for test"
        );
    }

    #[test]
    async fn test_secure_storage_basic_operations() {
        let hal = create_initialized_hal().await;

        let test_key = "test_key_basic";
        let test_data = b"Hello, secure world!";

        // Test storing data
        let result = hal.store_secure_data(test_key, test_data).await;
        assert!(result.is_ok(), "Storing secure data should succeed");

        // Test loading data
        let loaded_data = hal
            .load_secure_data(test_key)
            .await
            .expect("Failed to load secure data");
        assert!(loaded_data.is_some(), "Loaded data should exist");
        assert_eq!(
            loaded_data.unwrap(),
            test_data,
            "Loaded data should match stored data"
        );

        // Test deleting data
        let deleted = hal
            .delete_secure_data(test_key)
            .await
            .expect("Failed to delete secure data");
        assert!(deleted, "Data should be successfully deleted");

        // Test loading deleted data
        let loaded_data = hal
            .load_secure_data(test_key)
            .await
            .expect("Failed to load secure data after deletion");
        assert!(loaded_data.is_none(), "Deleted data should not exist");
    }

    #[test]
    async fn test_secure_storage_nonexistent_key() {
        let hal = create_initialized_hal().await;

        let nonexistent_key = "nonexistent_key_12345";

        // Test loading nonexistent data
        let loaded_data = hal
            .load_secure_data(nonexistent_key)
            .await
            .expect("Failed to load nonexistent secure data");
        assert!(loaded_data.is_none(), "Nonexistent data should return None");

        // Test deleting nonexistent data
        let deleted = hal
            .delete_secure_data(nonexistent_key)
            .await
            .expect("Failed to delete nonexistent secure data");
        assert!(!deleted, "Deleting nonexistent data should return false");
    }

    #[test]
    async fn test_secure_storage_overwrite() {
        let hal = create_initialized_hal().await;

        let test_key = "test_key_overwrite";
        let original_data = b"Original data";
        let new_data = b"New data that overwrites the original";

        // Store original data
        let result = hal.store_secure_data(test_key, original_data).await;
        assert!(
            result.is_ok(),
            "Storing original secure data should succeed"
        );

        // Overwrite with new data
        let result = hal.store_secure_data(test_key, new_data).await;
        assert!(result.is_ok(), "Overwriting secure data should succeed");

        // Verify new data is loaded
        let loaded_data = hal
            .load_secure_data(test_key)
            .await
            .expect("Failed to load overwritten secure data");
        assert!(loaded_data.is_some(), "Overwritten data should exist");
        assert_eq!(
            loaded_data.unwrap(),
            new_data,
            "Loaded data should match new data"
        );

        // Cleanup
        let _ = hal.delete_secure_data(test_key).await;
    }

    #[test]
    async fn test_secure_storage_empty_data() {
        let hal = create_initialized_hal().await;

        let test_key = "test_key_empty";
        let empty_data = b"";

        // Test storing empty data
        let result = hal.store_secure_data(test_key, empty_data).await;
        assert!(result.is_ok(), "Storing empty secure data should succeed");

        // Test loading empty data
        let loaded_data = hal
            .load_secure_data(test_key)
            .await
            .expect("Failed to load empty secure data");
        assert!(loaded_data.is_some(), "Empty data should exist");
        assert_eq!(
            loaded_data.unwrap(),
            empty_data,
            "Loaded empty data should match"
        );

        // Cleanup
        let _ = hal.delete_secure_data(test_key).await;
    }

    #[test]
    async fn test_secure_storage_large_data() {
        let hal = create_initialized_hal().await;

        let test_key = "test_key_large";
        let large_data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();

        // Test storing large data
        let result = hal.store_secure_data(test_key, &large_data).await;
        assert!(result.is_ok(), "Storing large secure data should succeed");

        // Test loading large data
        let loaded_data = hal
            .load_secure_data(test_key)
            .await
            .expect("Failed to load large secure data");
        assert!(loaded_data.is_some(), "Large data should exist");
        assert_eq!(
            loaded_data.unwrap(),
            large_data,
            "Loaded large data should match"
        );

        // Cleanup
        let _ = hal.delete_secure_data(test_key).await;
    }

    #[test]
    async fn test_secure_storage_special_characters() {
        let hal = create_initialized_hal().await;

        let test_key = "test_key_special_chars";
        let special_data = "Special chars: 🔐🚀💡⏰✅❌🛑".as_bytes();

        // Test storing data with special characters
        let result = hal.store_secure_data(test_key, special_data).await;
        assert!(
            result.is_ok(),
            "Storing secure data with special chars should succeed"
        );

        // Test loading data with special characters
        let loaded_data = hal
            .load_secure_data(test_key)
            .await
            .expect("Failed to load secure data with special chars");
        assert!(
            loaded_data.is_some(),
            "Data with special chars should exist"
        );
        assert_eq!(
            loaded_data.unwrap(),
            special_data,
            "Loaded special char data should match"
        );

        // Cleanup
        let _ = hal.delete_secure_data(test_key).await;
    }

    #[test]
    async fn test_list_secure_keys() {
        let hal = create_initialized_hal().await;

        let test_keys = vec!["list_test_1", "list_test_2", "list_test_3"];
        let test_data = b"test data for listing";

        // Store multiple keys
        for key in &test_keys {
            let result = hal.store_secure_data(key, test_data).await;
            assert!(
                result.is_ok(),
                "Storing secure data for listing should succeed"
            );
        }

        // List keys
        let listed_keys = hal
            .list_secure_keys()
            .await
            .expect("Failed to list secure keys");

        // Verify all test keys are present
        for key in &test_keys {
            assert!(
                listed_keys.contains(&key.to_string()),
                "Listed keys should contain test key: {}",
                key
            );
        }

        // Cleanup
        for key in &test_keys {
            let _ = hal.delete_secure_data(key).await;
        }
    }

    #[test]
    async fn test_shutdown_before_initialization() {
        let mut hal = create_test_hal().await;

        // Test shutdown without initialization
        let result = hal.shutdown().await;
        assert!(
            result.is_err(),
            "Shutdown without initialization should fail"
        );
    }

    #[test]
    async fn test_concurrent_led_operations() {
        let hal = Arc::new(create_initialized_hal().await);

        // Test concurrent LED operations
        let mut handles = Vec::new();

        for i in 0..10 {
            let hal_clone = Arc::clone(&hal);
            let handle = tokio::spawn(async move {
                let state = if i % 2 == 0 {
                    LedState::On
                } else {
                    LedState::Off
                };
                hal_clone.set_led(state).await
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.expect("Task should complete");
            assert!(result.is_ok(), "Concurrent LED operation should succeed");
        }

        // Verify final state is readable
        let final_state = hal.get_led_state().await;
        assert!(
            final_state.is_ok(),
            "Getting LED state after concurrent operations should succeed"
        );
    }

    #[test]
    async fn test_concurrent_secure_storage_operations() {
        let hal = Arc::new(create_initialized_hal().await);

        // Test concurrent storage operations
        let mut handles = Vec::new();

        for i in 0..5 {
            let hal_clone = Arc::clone(&hal);
            let handle = tokio::spawn(async move {
                let key = format!("concurrent_test_{}", i);
                let data = format!("data_{}", i).into_bytes();

                // Store data
                hal_clone.store_secure_data(&key, &data).await?;

                // Load data
                let loaded = hal_clone.load_secure_data(&key).await?;
                assert_eq!(loaded, Some(data));

                // Delete data
                let deleted = hal_clone.delete_secure_data(&key).await?;
                assert!(deleted);

                Ok::<(), PlatformError>(())
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.expect("Task should complete");
            assert!(
                result.is_ok(),
                "Concurrent storage operation should succeed"
            );
        }
    }
}
