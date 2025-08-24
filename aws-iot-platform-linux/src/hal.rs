use aws_iot_core::{
    DeviceInfo, LedState, MemoryInfo, PlatformError, PlatformHAL, PlatformResult, UptimeInfo,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use sysinfo::System;
use tokio::time::sleep;
use tracing::{debug, info};

/// Linux HAL implementation for CI/CD and development
pub struct LinuxHAL {
    initialized: bool,
    led_state: Arc<Mutex<bool>>,
    secure_storage: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    system: Arc<Mutex<System>>,
}

impl LinuxHAL {
    pub fn new() -> Self {
        Self {
            initialized: false,
            led_state: Arc::new(Mutex::new(false)),
            secure_storage: Arc::new(Mutex::new(HashMap::new())),
            system: Arc::new(Mutex::new(System::new_all())),
        }
    }

    fn ensure_initialized(&self) -> PlatformResult<()> {
        if !self.initialized {
            return Err(PlatformError::Hardware("HAL not initialized".to_string()));
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl PlatformHAL for LinuxHAL {
    async fn initialize(&mut self) -> PlatformResult<()> {
        info!("Initializing Linux HAL for CI/CD environment");

        // Initialize system information
        {
            let mut system = self.system.lock().unwrap();
            system.refresh_all();
        }

        self.initialized = true;
        info!("Linux HAL initialized successfully");
        Ok(())
    }

    async fn shutdown(&mut self) -> PlatformResult<()> {
        self.ensure_initialized()?;

        info!("Shutting down Linux HAL");
        self.initialized = false;
        info!("Linux HAL shutdown successfully");
        Ok(())
    }

    async fn sleep(&self, duration: Duration) -> PlatformResult<()> {
        self.ensure_initialized()?;

        debug!("💤 Sleeping for {:?} (simulated on Linux)", duration);
        sleep(duration).await;
        debug!("⏰ Wake up from sleep");
        Ok(())
    }

    async fn set_led(&self, state: LedState) -> PlatformResult<()> {
        self.ensure_initialized()?;

        let new_state = matches!(state, LedState::On);
        let old_state = {
            let mut led_state = self.led_state.lock().unwrap();
            let old = *led_state;
            *led_state = new_state;
            old
        };

        let state_str = if new_state { "ON" } else { "OFF" };
        let color_code = if new_state { "\x1b[32m" } else { "\x1b[31m" };
        let reset_code = "\x1b[0m";

        info!(
            "💡 LED state changed: {} -> {} {}●{}",
            if old_state { "ON" } else { "OFF" },
            state_str,
            color_code,
            reset_code
        );

        Ok(())
    }

    async fn get_led_state(&self) -> PlatformResult<LedState> {
        self.ensure_initialized()?;

        let state = *self.led_state.lock().unwrap();
        Ok(if state { LedState::On } else { LedState::Off })
    }

    async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
        self.ensure_initialized()?;

        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "linux-ci-runner".to_string());

        let device_id = format!(
            "linux-{}",
            hostname
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>()
                .to_lowercase()
        );

        Ok(DeviceInfo {
            device_id,
            platform: "Linux (CI/CD)".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            firmware_version: env!("CARGO_PKG_VERSION").to_string(),
            hardware_revision: Some("ci-runner".to_string()),
            serial_number: Some("CI-001".to_string()),
        })
    }

    async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
        self.ensure_initialized()?;

        let mut system = self.system.lock().unwrap();
        system.refresh_memory();

        let total_bytes = system.total_memory() * 1024; // sysinfo returns KB
        let free_bytes = system.available_memory() * 1024;
        let used_bytes = total_bytes - free_bytes;

        // Get largest free block (approximation)
        let largest_free_block = free_bytes / 2; // Conservative estimate

        Ok(MemoryInfo {
            total_bytes,
            free_bytes,
            used_bytes,
            largest_free_block,
        })
    }

    async fn get_uptime(&self) -> PlatformResult<UptimeInfo> {
        self.ensure_initialized()?;

        let mut system = self.system.lock().unwrap();
        system.refresh_all();

        let uptime_seconds = System::uptime();
        let uptime = Duration::from_secs(uptime_seconds);

        // Calculate approximate boot time
        let boot_time = Utc::now() - chrono::Duration::seconds(uptime_seconds as i64);

        Ok(UptimeInfo { uptime, boot_time })
    }

    async fn store_secure_data(&self, key: &str, data: &[u8]) -> PlatformResult<()> {
        self.ensure_initialized()?;

        debug!(
            "🔐 Storing secure data for key: {} ({} bytes)",
            key,
            data.len()
        );

        let mut storage = self.secure_storage.lock().unwrap();
        storage.insert(key.to_string(), data.to_vec());

        debug!("✅ Secure data stored successfully");
        Ok(())
    }

    async fn load_secure_data(&self, key: &str) -> PlatformResult<Option<Vec<u8>>> {
        self.ensure_initialized()?;

        debug!("🔓 Loading secure data for key: {}", key);

        let storage = self.secure_storage.lock().unwrap();
        let data = storage.get(key).cloned();

        if data.is_some() {
            debug!("✅ Secure data loaded successfully");
        } else {
            debug!("❌ Secure data not found for key: {}", key);
        }

        Ok(data)
    }

    async fn delete_secure_data(&self, key: &str) -> PlatformResult<bool> {
        self.ensure_initialized()?;

        debug!("🗑️ Deleting secure data for key: {}", key);

        let mut storage = self.secure_storage.lock().unwrap();
        let existed = storage.remove(key).is_some();

        if existed {
            debug!("✅ Secure data deleted successfully");
        } else {
            debug!("❌ Secure data not found for key: {}", key);
        }

        Ok(existed)
    }

    async fn list_secure_keys(&self) -> PlatformResult<Vec<String>> {
        self.ensure_initialized()?;

        let storage = self.secure_storage.lock().unwrap();
        let keys: Vec<String> = storage.keys().cloned().collect();

        debug!("📋 Listed {} secure keys", keys.len());
        Ok(keys)
    }
}

impl Default for LinuxHAL {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_hal_creation() {
        let hal = LinuxHAL::new();
        assert!(!hal.initialized);
    }

    #[tokio::test]
    async fn test_hal_initialization() {
        let mut hal = LinuxHAL::new();
        assert!(hal.initialize().await.is_ok());
        assert!(hal.initialized);
        assert!(hal.shutdown().await.is_ok());
        assert!(!hal.initialized);
    }

    #[tokio::test]
    async fn test_led_state_management() {
        let mut hal = LinuxHAL::new();
        hal.initialize().await.unwrap();

        // Test initial state
        let initial_state = hal.get_led_state().await.unwrap();
        assert!(matches!(initial_state, LedState::Off));

        // Test setting LED on
        hal.set_led(LedState::On).await.unwrap();
        let state = hal.get_led_state().await.unwrap();
        assert!(matches!(state, LedState::On));

        // Test setting LED off
        hal.set_led(LedState::Off).await.unwrap();
        let state = hal.get_led_state().await.unwrap();
        assert!(matches!(state, LedState::Off));

        hal.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_sleep_functionality() {
        let mut hal = LinuxHAL::new();
        hal.initialize().await.unwrap();

        let start = std::time::Instant::now();
        hal.sleep(Duration::from_millis(100)).await.unwrap();
        let elapsed = start.elapsed();

        // Allow some tolerance for timing
        assert!(elapsed >= Duration::from_millis(90));
        assert!(elapsed <= Duration::from_millis(200));

        hal.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_device_info() {
        let mut hal = LinuxHAL::new();
        hal.initialize().await.unwrap();

        let device_info = hal.get_device_info().await.unwrap();
        assert!(device_info.device_id.starts_with("linux-"));
        assert_eq!(device_info.platform, "Linux (CI/CD)");
        assert!(!device_info.version.is_empty());

        hal.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_memory_info() {
        let mut hal = LinuxHAL::new();
        hal.initialize().await.unwrap();

        let memory_info = hal.get_memory_info().await.unwrap();
        assert!(memory_info.total_bytes > 0);
        assert!(memory_info.free_bytes > 0);
        assert!(memory_info.used_bytes > 0);
        assert!(memory_info.largest_free_block > 0);
        assert_eq!(
            memory_info.total_bytes,
            memory_info.free_bytes + memory_info.used_bytes
        );

        hal.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_uptime_info() {
        let mut hal = LinuxHAL::new();
        hal.initialize().await.unwrap();

        let uptime_info = hal.get_uptime().await.unwrap();
        assert!(uptime_info.uptime.as_secs() > 0);
        assert!(uptime_info.boot_time < Utc::now());

        hal.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_secure_storage_basic_operations() {
        let mut hal = LinuxHAL::new();
        hal.initialize().await.unwrap();

        let key = "test_key";
        let data = b"test_data";

        // Test storing data
        hal.store_secure_data(key, data).await.unwrap();

        // Test loading data
        let loaded_data = hal.load_secure_data(key).await.unwrap();
        assert_eq!(loaded_data, Some(data.to_vec()));

        // Test listing keys
        let keys = hal.list_secure_keys().await.unwrap();
        assert!(keys.contains(&key.to_string()));

        // Test deleting data
        let deleted = hal.delete_secure_data(key).await.unwrap();
        assert!(deleted);

        // Test loading deleted data
        let loaded_data = hal.load_secure_data(key).await.unwrap();
        assert_eq!(loaded_data, None);

        hal.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_secure_storage_nonexistent_key() {
        let mut hal = LinuxHAL::new();
        hal.initialize().await.unwrap();

        let result = hal.load_secure_data("nonexistent_key").await.unwrap();
        assert_eq!(result, None);

        let deleted = hal.delete_secure_data("nonexistent_key").await.unwrap();
        assert!(!deleted);

        hal.shutdown().await.unwrap();
    }
}
