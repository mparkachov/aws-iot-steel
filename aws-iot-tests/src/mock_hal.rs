use async_trait::async_trait;
use aws_iot_core::{
    PlatformHAL, PlatformResult, PlatformError, LedState, 
    DeviceInfo, MemoryInfo, UptimeInfo
};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Mock implementation of PlatformHAL for testing
#[derive(Debug)]
pub struct MockHAL {
    led_state: Arc<Mutex<LedState>>,
    secure_storage: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    boot_time: DateTime<Utc>,
    initialized: Arc<Mutex<bool>>,
    sleep_calls: Arc<Mutex<Vec<Duration>>>,
    led_calls: Arc<Mutex<Vec<LedState>>>,
}

impl MockHAL {
    pub fn new() -> Self {
        Self {
            led_state: Arc::new(Mutex::new(LedState::Off)),
            secure_storage: Arc::new(Mutex::new(HashMap::new())),
            boot_time: Utc::now(),
            initialized: Arc::new(Mutex::new(false)),
            sleep_calls: Arc::new(Mutex::new(Vec::new())),
            led_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get the history of sleep calls for testing
    pub async fn get_sleep_calls(&self) -> Vec<Duration> {
        self.sleep_calls.lock().await.clone()
    }
    
    /// Get the history of LED calls for testing
    pub async fn get_led_calls(&self) -> Vec<LedState> {
        self.led_calls.lock().await.clone()
    }
    
    /// Clear all call history
    pub async fn clear_call_history(&self) {
        self.sleep_calls.lock().await.clear();
        self.led_calls.lock().await.clear();
    }
}

impl Default for MockHAL {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlatformHAL for MockHAL {
    async fn sleep(&self, duration: Duration) -> PlatformResult<()> {
        self.sleep_calls.lock().await.push(duration);
        // Don't actually sleep in tests, just record the call
        Ok(())
    }
    
    async fn set_led(&self, state: LedState) -> PlatformResult<()> {
        self.led_calls.lock().await.push(state);
        *self.led_state.lock().await = state;
        Ok(())
    }
    
    async fn get_led_state(&self) -> PlatformResult<LedState> {
        Ok(*self.led_state.lock().await)
    }
    
    async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
        Ok(DeviceInfo {
            device_id: "mock-device-001".to_string(),
            platform: "Mock".to_string(),
            version: "0.1.0".to_string(),
            firmware_version: "1.0.0-test".to_string(),
            hardware_revision: Some("Mock-Rev1".to_string()),
            serial_number: Some("MOCK-001".to_string()),
        })
    }
    
    async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
        Ok(MemoryInfo {
            total_bytes: 1_048_576, // 1MB
            free_bytes: 524_288,    // 512KB
            used_bytes: 524_288,    // 512KB
            largest_free_block: 262_144, // 256KB
        })
    }
    
    async fn get_uptime(&self) -> PlatformResult<UptimeInfo> {
        let now = Utc::now();
        let uptime = now.signed_duration_since(self.boot_time)
            .to_std()
            .map_err(|e| PlatformError::DeviceInfo(format!("Invalid uptime calculation: {}", e)))?;
            
        Ok(UptimeInfo {
            uptime,
            boot_time: self.boot_time,
        })
    }
    
    async fn store_secure_data(&self, key: &str, data: &[u8]) -> PlatformResult<()> {
        self.secure_storage.lock().await.insert(key.to_string(), data.to_vec());
        Ok(())
    }
    
    async fn load_secure_data(&self, key: &str) -> PlatformResult<Option<Vec<u8>>> {
        Ok(self.secure_storage.lock().await.get(key).cloned())
    }
    
    async fn delete_secure_data(&self, key: &str) -> PlatformResult<bool> {
        Ok(self.secure_storage.lock().await.remove(key).is_some())
    }
    
    async fn list_secure_keys(&self) -> PlatformResult<Vec<String>> {
        Ok(self.secure_storage.lock().await.keys().cloned().collect())
    }
    
    async fn initialize(&mut self) -> PlatformResult<()> {
        let mut initialized = self.initialized.lock().await;
        if *initialized {
            return Err(PlatformError::Hardware("HAL already initialized".to_string()));
        }
        *initialized = true;
        Ok(())
    }
    
    async fn shutdown(&mut self) -> PlatformResult<()> {
        let mut initialized = self.initialized.lock().await;
        if !*initialized {
            return Err(PlatformError::Hardware("HAL not initialized".to_string()));
        }
        *initialized = false;
        Ok(())
    }
}