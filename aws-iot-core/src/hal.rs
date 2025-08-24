use crate::{DeviceInfo, LedState, MemoryInfo, PlatformResult, UptimeInfo};
use async_trait::async_trait;
use std::time::Duration;

/// Hardware Abstraction Layer trait that provides unified interface
/// for both ESP32 and macOS platforms
#[async_trait]
pub trait PlatformHAL: Send + Sync {
    /// Sleep for the specified duration
    ///
    /// # Arguments
    /// * `duration` - The duration to sleep
    ///
    /// # Returns
    /// * `Ok(())` if sleep completed successfully
    /// * `Err(PlatformError)` if sleep operation failed
    async fn sleep(&self, duration: Duration) -> PlatformResult<()>;

    /// Set the LED state (on/off)
    ///
    /// # Arguments
    /// * `state` - The desired LED state
    ///
    /// # Returns
    /// * `Ok(())` if LED state was set successfully
    /// * `Err(PlatformError)` if LED operation failed
    async fn set_led(&self, state: LedState) -> PlatformResult<()>;

    /// Get the current LED state
    ///
    /// # Returns
    /// * `Ok(LedState)` with current LED state
    /// * `Err(PlatformError)` if LED state cannot be determined
    async fn get_led_state(&self) -> PlatformResult<LedState>;

    /// Get device information
    ///
    /// # Returns
    /// * `Ok(DeviceInfo)` with device details
    /// * `Err(PlatformError)` if device info cannot be retrieved
    async fn get_device_info(&self) -> PlatformResult<DeviceInfo>;

    /// Get memory usage information
    ///
    /// # Returns
    /// * `Ok(MemoryInfo)` with memory statistics
    /// * `Err(PlatformError)` if memory info cannot be retrieved
    async fn get_memory_info(&self) -> PlatformResult<MemoryInfo>;

    /// Get system uptime information
    ///
    /// # Returns
    /// * `Ok(UptimeInfo)` with uptime details
    /// * `Err(PlatformError)` if uptime cannot be determined
    async fn get_uptime(&self) -> PlatformResult<UptimeInfo>;

    /// Store data securely (encrypted at rest)
    ///
    /// # Arguments
    /// * `key` - The key to store the data under
    /// * `data` - The data to store securely
    ///
    /// # Returns
    /// * `Ok(())` if data was stored successfully
    /// * `Err(PlatformError)` if storage operation failed
    async fn store_secure_data(&self, key: &str, data: &[u8]) -> PlatformResult<()>;

    /// Load data from secure storage
    ///
    /// # Arguments
    /// * `key` - The key to load data for
    ///
    /// # Returns
    /// * `Ok(Some(Vec<u8>))` if data was found and loaded
    /// * `Ok(None)` if no data exists for the key
    /// * `Err(PlatformError)` if storage operation failed
    async fn load_secure_data(&self, key: &str) -> PlatformResult<Option<Vec<u8>>>;

    /// Delete data from secure storage
    ///
    /// # Arguments
    /// * `key` - The key to delete data for
    ///
    /// # Returns
    /// * `Ok(true)` if data was found and deleted
    /// * `Ok(false)` if no data existed for the key
    /// * `Err(PlatformError)` if storage operation failed
    async fn delete_secure_data(&self, key: &str) -> PlatformResult<bool>;

    /// List all keys in secure storage
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` with all available keys
    /// * `Err(PlatformError)` if storage operation failed
    async fn list_secure_keys(&self) -> PlatformResult<Vec<String>>;

    /// Initialize the platform HAL
    /// This method should be called once before using other HAL methods
    ///
    /// # Returns
    /// * `Ok(())` if initialization was successful
    /// * `Err(PlatformError)` if initialization failed
    async fn initialize(&mut self) -> PlatformResult<()>;

    /// Shutdown the platform HAL
    /// This method should be called when the HAL is no longer needed
    ///
    /// # Returns
    /// * `Ok(())` if shutdown was successful
    /// * `Err(PlatformError)` if shutdown failed
    async fn shutdown(&mut self) -> PlatformResult<()>;
}
