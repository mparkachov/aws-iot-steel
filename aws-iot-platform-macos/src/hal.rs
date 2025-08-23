use async_trait::async_trait;
use aws_iot_core::{
    PlatformHAL, PlatformResult, PlatformError, LedState, 
    DeviceInfo, MemoryInfo, UptimeInfo
};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

/// macOS implementation of the Platform HAL
/// This provides simulation of ESP32 hardware for development and testing
pub struct MacOSHAL {
    led_state: Arc<Mutex<LedState>>,
    boot_time: DateTime<Utc>,
    initialized: Arc<Mutex<bool>>,
}

impl MacOSHAL {
    /// Create a new macOS HAL instance
    pub fn new() -> Self {
        Self {
            led_state: Arc::new(Mutex::new(LedState::Off)),
            boot_time: Utc::now(),
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
        tracing::info!("💤 Sleeping for {:?}", duration);
        tokio::time::sleep(duration).await;
        tracing::info!("⏰ Wake up! Sleep completed");
        Ok(())
    }
    
    async fn set_led(&self, state: LedState) -> PlatformResult<()> {
        let mut current_state = self.led_state.lock().await;
        *current_state = state;
        
        match state {
            LedState::On => tracing::info!("💡 LED turned ON"),
            LedState::Off => tracing::info!("🔌 LED turned OFF"),
        }
        
        Ok(())
    }
    
    async fn get_led_state(&self) -> PlatformResult<LedState> {
        let state = *self.led_state.lock().await;
        Ok(state)
    }
    
    async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
        Ok(DeviceInfo {
            device_id: uuid::Uuid::new_v4().to_string(),
            platform: "macOS".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            firmware_version: "1.0.0-dev".to_string(),
            hardware_revision: Some("Simulator".to_string()),
            serial_number: Some("SIM-001".to_string()),
        })
    }
    
    async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
        // Simulate memory information for macOS
        Ok(MemoryInfo {
            total_bytes: 8_589_934_592, // 8GB simulated
            free_bytes: 4_294_967_296,  // 4GB free
            used_bytes: 4_294_967_296,  // 4GB used
            largest_free_block: 2_147_483_648, // 2GB largest block
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
        // For now, we'll implement a simple file-based storage
        // In a real implementation, this would use macOS Keychain
        tracing::debug!("🔐 Storing secure data for key: {}", key);
        
        let storage_dir = std::env::temp_dir().join("aws-iot-secure");
        tokio::fs::create_dir_all(&storage_dir).await
            .map_err(|e| PlatformError::Storage(format!("Failed to create storage directory: {}", e)))?;
            
        let file_path = storage_dir.join(format!("{}.dat", key));
        tokio::fs::write(&file_path, data).await
            .map_err(|e| PlatformError::Storage(format!("Failed to write secure data: {}", e)))?;
            
        tracing::debug!("✅ Secure data stored successfully for key: {}", key);
        Ok(())
    }
    
    async fn load_secure_data(&self, key: &str) -> PlatformResult<Option<Vec<u8>>> {
        tracing::debug!("🔓 Loading secure data for key: {}", key);
        
        let storage_dir = std::env::temp_dir().join("aws-iot-secure");
        let file_path = storage_dir.join(format!("{}.dat", key));
        
        match tokio::fs::read(&file_path).await {
            Ok(data) => {
                tracing::debug!("✅ Secure data loaded successfully for key: {}", key);
                Ok(Some(data))
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!("❌ No secure data found for key: {}", key);
                Ok(None)
            },
            Err(e) => Err(PlatformError::Storage(format!("Failed to read secure data: {}", e))),
        }
    }
    
    async fn delete_secure_data(&self, key: &str) -> PlatformResult<bool> {
        tracing::debug!("🗑️ Deleting secure data for key: {}", key);
        
        let storage_dir = std::env::temp_dir().join("aws-iot-secure");
        let file_path = storage_dir.join(format!("{}.dat", key));
        
        match tokio::fs::remove_file(&file_path).await {
            Ok(()) => {
                tracing::debug!("✅ Secure data deleted successfully for key: {}", key);
                Ok(true)
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!("❌ No secure data found to delete for key: {}", key);
                Ok(false)
            },
            Err(e) => Err(PlatformError::Storage(format!("Failed to delete secure data: {}", e))),
        }
    }
    
    async fn list_secure_keys(&self) -> PlatformResult<Vec<String>> {
        tracing::debug!("📋 Listing secure storage keys");
        
        let storage_dir = std::env::temp_dir().join("aws-iot-secure");
        
        let mut keys = Vec::new();
        
        match tokio::fs::read_dir(&storage_dir).await {
            Ok(mut entries) => {
                while let Some(entry) = entries.next_entry().await
                    .map_err(|e| PlatformError::Storage(format!("Failed to read directory entry: {}", e)))? {
                    
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.ends_with(".dat") {
                            let key = file_name.trim_end_matches(".dat").to_string();
                            keys.push(key);
                        }
                    }
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Directory doesn't exist, return empty list
                tracing::debug!("📂 Secure storage directory doesn't exist, returning empty list");
            },
            Err(e) => return Err(PlatformError::Storage(format!("Failed to list secure keys: {}", e))),
        }
        
        tracing::debug!("✅ Found {} secure storage keys", keys.len());
        Ok(keys)
    }
    
    async fn initialize(&mut self) -> PlatformResult<()> {
        let mut initialized = self.initialized.lock().await;
        if *initialized {
            return Err(PlatformError::Hardware("HAL already initialized".to_string()));
        }
        
        tracing::info!("🚀 Initializing macOS HAL");
        
        // Create secure storage directory if it doesn't exist
        let storage_dir = std::env::temp_dir().join("aws-iot-secure");
        tokio::fs::create_dir_all(&storage_dir).await
            .map_err(|e| PlatformError::Hardware(format!("Failed to create storage directory: {}", e)))?;
        
        *initialized = true;
        tracing::info!("✅ macOS HAL initialized successfully");
        Ok(())
    }
    
    async fn shutdown(&mut self) -> PlatformResult<()> {
        let mut initialized = self.initialized.lock().await;
        if !*initialized {
            return Err(PlatformError::Hardware("HAL not initialized".to_string()));
        }
        
        tracing::info!("🛑 Shutting down macOS HAL");
        
        // Reset LED state
        *self.led_state.lock().await = LedState::Off;
        
        *initialized = false;
        tracing::info!("✅ macOS HAL shutdown successfully");
        Ok(())
    }
}