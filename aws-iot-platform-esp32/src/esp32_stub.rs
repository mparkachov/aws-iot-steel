/// ESP32-C3 Stub Implementation
/// This module provides a stub implementation for non-ESP32 targets to enable cross-compilation

use async_trait::async_trait;
use aws_iot_core::{
    PlatformHAL, PlatformResult, PlatformError, LedState, 
    DeviceInfo, MemoryInfo, UptimeInfo
};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};

/// Stub implementation of ESP32HAL for non-ESP32 targets
/// This allows the code to compile on development machines without ESP32 toolchain
pub struct ESP32HAL {
    led_state: Arc<Mutex<LedState>>,
    boot_time: DateTime<Utc>,
    initialized: Arc<Mutex<bool>>,
}

impl ESP32HAL {
    /// Create a new ESP32HAL stub instance
    pub fn new() -> PlatformResult<Self> {
        log::info!("Creating ESP32HAL stub instance (non-ESP32 target)");
        
        Ok(Self {
            led_state: Arc::new(Mutex::new(LedState::Off)),
            boot_time: Utc::now(),
            initialized: Arc::new(Mutex::new(false)),
        })
    }
}

impl Default for ESP32HAL {
    fn default() -> Self {
        Self::new().expect("Failed to create ESP32HAL stub")
    }
}

#[async_trait]
impl PlatformHAL for ESP32HAL {
    async fn sleep(&self, duration: Duration) -> PlatformResult<()> {
        log::warn!("ESP32HAL stub: sleep({:?}) - not implemented on this target", duration);
        Err(PlatformError::Unsupported(
            "ESP32HAL sleep not available on non-ESP32 targets".to_string()
        ))
    }
    
    async fn set_led(&self, state: LedState) -> PlatformResult<()> {
        log::warn!("ESP32HAL stub: set_led({:?}) - not implemented on this target", state);
        
        // Update internal state for consistency
        let mut led_state_guard = self.led_state.lock()
            .map_err(|_| PlatformError::Hardware("Failed to lock LED state".to_string()))?;
        *led_state_guard = state;
        
        Err(PlatformError::Unsupported(
            "ESP32HAL LED control not available on non-ESP32 targets".to_string()
        ))
    }
    
    async fn get_led_state(&self) -> PlatformResult<LedState> {
        let led_state_guard = self.led_state.lock()
            .map_err(|_| PlatformError::Hardware("Failed to lock LED state".to_string()))?;
        
        Ok(*led_state_guard)
    }
    
    async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
        log::warn!("ESP32HAL stub: get_device_info() - returning stub data");
        
        Ok(DeviceInfo {
            device_id: "esp32c3-stub-000000000000".to_string(),
            platform: "ESP32-C3 Stub (Non-target)".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            firmware_version: "1.0.0-stub".to_string(),
            hardware_revision: Some("Stub Rev 1".to_string()),
            serial_number: Some("stub-000000000000".to_string()),
        })
    }
    
    async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
        log::warn!("ESP32HAL stub: get_memory_info() - returning stub data");
        
        Ok(MemoryInfo {
            total_bytes: 256 * 1024, // 256KB stub
            free_bytes: 200 * 1024,  // 200KB free stub
            used_bytes: 56 * 1024,   // 56KB used stub
            largest_free_block: 150 * 1024, // 150KB largest block stub
        })
    }
    
    async fn get_uptime(&self) -> PlatformResult<UptimeInfo> {
        let uptime = Utc::now().signed_duration_since(self.boot_time);
        let uptime_duration = Duration::from_secs(uptime.num_seconds().max(0) as u64);
        
        Ok(UptimeInfo {
            uptime: uptime_duration,
            boot_time: self.boot_time,
        })
    }
    
    async fn store_secure_data(&self, key: &str, data: &[u8]) -> PlatformResult<()> {
        log::warn!("ESP32HAL stub: store_secure_data('{}', {} bytes) - not implemented on this target", 
                  key, data.len());
        Err(PlatformError::Unsupported(
            "ESP32HAL secure storage not available on non-ESP32 targets".to_string()
        ))
    }
    
    async fn load_secure_data(&self, key: &str) -> PlatformResult<Option<Vec<u8>>> {
        log::warn!("ESP32HAL stub: load_secure_data('{}') - not implemented on this target", key);
        Err(PlatformError::Unsupported(
            "ESP32HAL secure storage not available on non-ESP32 targets".to_string()
        ))
    }
    
    async fn delete_secure_data(&self, key: &str) -> PlatformResult<bool> {
        log::warn!("ESP32HAL stub: delete_secure_data('{}') - not implemented on this target", key);
        Err(PlatformError::Unsupported(
            "ESP32HAL secure storage not available on non-ESP32 targets".to_string()
        ))
    }
    
    async fn list_secure_keys(&self) -> PlatformResult<Vec<String>> {
        log::warn!("ESP32HAL stub: list_secure_keys() - not implemented on this target");
        Err(PlatformError::Unsupported(
            "ESP32HAL secure storage not available on non-ESP32 targets".to_string()
        ))
    }
    
    async fn initialize(&mut self) -> PlatformResult<()> {
        let mut initialized_guard = self.initialized.lock()
            .map_err(|_| PlatformError::Hardware("Failed to lock initialization state".to_string()))?;
        
        if *initialized_guard {
            return Err(PlatformError::Hardware("ESP32HAL stub already initialized".to_string()));
        }
        
        log::warn!("ESP32HAL stub: initialize() - stub initialization on non-ESP32 target");
        
        *initialized_guard = true;
        Ok(())
    }
    
    async fn shutdown(&mut self) -> PlatformResult<()> {
        let mut initialized_guard = self.initialized.lock()
            .map_err(|_| PlatformError::Hardware("Failed to lock initialization state".to_string()))?;
        
        if !*initialized_guard {
            return Err(PlatformError::Hardware("ESP32HAL stub not initialized".to_string()));
        }
        
        log::warn!("ESP32HAL stub: shutdown() - stub shutdown on non-ESP32 target");
        
        *initialized_guard = false;
        Ok(())
    }
}