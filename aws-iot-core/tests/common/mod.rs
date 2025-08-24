use async_trait::async_trait;
use aws_iot_core::*;
use chrono::Utc;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Mock HAL implementation for testing
pub struct MockHAL {
    pub led_state: Arc<Mutex<LedState>>,
    pub sleep_called: Arc<AtomicBool>,
}

impl MockHAL {
    pub fn new() -> Self {
        Self {
            led_state: Arc::new(Mutex::new(LedState::Off)),
            sleep_called: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl PlatformHAL for MockHAL {
    async fn sleep(&self, _duration: Duration) -> PlatformResult<()> {
        self.sleep_called.store(true, Ordering::SeqCst);
        // Simulate sleep without actually sleeping in tests
        tokio::time::sleep(Duration::from_millis(1)).await;
        Ok(())
    }

    async fn set_led(&self, state: LedState) -> PlatformResult<()> {
        *self.led_state.lock() = state;
        Ok(())
    }

    async fn get_led_state(&self) -> PlatformResult<LedState> {
        Ok(*self.led_state.lock())
    }

    async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
        Ok(DeviceInfo {
            device_id: "test-device".to_string(),
            platform: "test".to_string(),
            version: "1.0.0".to_string(),
            firmware_version: "1.0.0".to_string(),
            hardware_revision: Some("rev1".to_string()),
            serial_number: Some("12345".to_string()),
        })
    }

    async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
        Ok(MemoryInfo {
            total_bytes: 1024 * 1024,
            free_bytes: 512 * 1024,
            used_bytes: 512 * 1024,
            largest_free_block: 256 * 1024,
        })
    }

    async fn get_uptime(&self) -> PlatformResult<UptimeInfo> {
        Ok(UptimeInfo {
            uptime: Duration::from_secs(3600),
            boot_time: Utc::now() - chrono::Duration::seconds(3600),
        })
    }

    async fn store_secure_data(&self, _key: &str, _data: &[u8]) -> PlatformResult<()> {
        Ok(())
    }

    async fn load_secure_data(&self, _key: &str) -> PlatformResult<Option<Vec<u8>>> {
        Ok(None)
    }

    async fn delete_secure_data(&self, _key: &str) -> PlatformResult<bool> {
        Ok(false)
    }

    async fn list_secure_keys(&self) -> PlatformResult<Vec<String>> {
        Ok(vec![])
    }

    async fn initialize(&mut self) -> PlatformResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> PlatformResult<()> {
        Ok(())
    }
}
