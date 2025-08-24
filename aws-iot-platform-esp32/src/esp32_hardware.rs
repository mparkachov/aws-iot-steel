/// ESP32-C3-DevKit-RUST-1 Hardware Implementation
/// This module provides the actual ESP32-C3 hardware integration
use async_trait::async_trait;
use aws_iot_core::{
    DeviceInfo, LedState, MemoryInfo, PlatformError, PlatformHAL, PlatformResult, UptimeInfo,
};
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use embedded_svc::storage::RawStorage;
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{Gpio2, Output, PinDriver},
    nvs::{EspNvs, EspNvsPartition, NvsDefault},
    peripherals::Peripherals,
    task::block_on,
};
use esp_idf_sys as _;
use heapless::String as HeaplessString;

/// GPIO pin for the built-in LED on ESP32-C3-DevKit-RUST-1
const LED_GPIO_PIN: u8 = 2;

/// NVS namespace for secure storage
const SECURE_STORAGE_NAMESPACE: &str = "aws_iot_secure";

/// Maximum key length for NVS storage
const MAX_KEY_LENGTH: usize = 15;

/// Maximum data size for single NVS entry (bytes)
const MAX_DATA_SIZE: usize = 4000;

/// ESP32-C3-DevKit-RUST-1 implementation of the Platform HAL
/// Provides actual hardware integration with ESP32-C3 peripherals
pub struct ESP32HAL {
    led_pin: Arc<Mutex<Option<PinDriver<'static, Gpio2, Output>>>>,
    led_state: Arc<Mutex<LedState>>,
    nvs: Arc<Mutex<Option<EspNvs<NvsDefault>>>>,
    boot_time: DateTime<Utc>,
    initialized: Arc<Mutex<bool>>,
}

impl ESP32HAL {
    /// Create a new ESP32-C3 HAL instance
    pub fn new() -> PlatformResult<Self> {
        log::info!("Creating ESP32-C3 HAL instance");

        Ok(Self {
            led_pin: Arc::new(Mutex::new(None)),
            led_state: Arc::new(Mutex::new(LedState::Off)),
            nvs: Arc::new(Mutex::new(None)),
            boot_time: Utc::now(),
            initialized: Arc::new(Mutex::new(false)),
        })
    }

    /// Get ESP32-C3 chip information
    fn get_chip_info() -> PlatformResult<(String, String)> {
        unsafe {
            let chip_info = esp_idf_sys::esp_chip_info_t::default();
            esp_idf_sys::esp_chip_info(&chip_info as *const _ as *mut _);

            let model = match chip_info.model {
                esp_idf_sys::esp_chip_model_t_CHIP_ESP32C3 => "ESP32-C3",
                _ => "Unknown ESP32",
            };

            let revision = format!("Rev {}", chip_info.revision);
            Ok((model.to_string(), revision))
        }
    }

    /// Get ESP32-C3 MAC address as device ID
    fn get_mac_address() -> PlatformResult<String> {
        let mut mac = [0u8; 6];
        unsafe {
            let result = esp_idf_sys::esp_read_mac(
                mac.as_mut_ptr(),
                esp_idf_sys::esp_mac_type_t_ESP_MAC_WIFI_STA,
            );
            if result != esp_idf_sys::ESP_OK {
                return Err(PlatformError::Hardware(
                    "Failed to read MAC address".to_string(),
                ));
            }
        }

        Ok(format!(
            "esp32c3-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        ))
    }

    /// Get ESP32-C3 memory information from heap
    fn get_esp32_memory_info() -> PlatformResult<MemoryInfo> {
        unsafe {
            let free_heap = esp_idf_sys::esp_get_free_heap_size() as u64;
            let min_free_heap = esp_idf_sys::esp_get_minimum_free_heap_size() as u64;

            // Get total heap size (approximate)
            let total_heap = free_heap + (256 * 1024 - min_free_heap); // Rough estimate
            let used_heap = total_heap - free_heap;

            // Get largest free block
            let largest_free =
                esp_idf_sys::heap_caps_get_largest_free_block(esp_idf_sys::MALLOC_CAP_DEFAULT)
                    as u64;

            Ok(MemoryInfo {
                total_bytes: total_heap,
                free_bytes: free_heap,
                used_bytes: used_heap,
                largest_free_block: largest_free,
            })
        }
    }

    /// Get ESP32-C3 uptime using system tick count
    fn get_esp32_uptime(&self) -> PlatformResult<UptimeInfo> {
        unsafe {
            let tick_count = esp_idf_sys::xTaskGetTickCount();
            let tick_rate = esp_idf_sys::configTICK_RATE_HZ;
            let uptime_secs = tick_count / tick_rate;
            let uptime = Duration::from_secs(uptime_secs as u64);

            Ok(UptimeInfo {
                uptime,
                boot_time: self.boot_time,
            })
        }
    }

    /// Initialize NVS (Non-Volatile Storage) for secure data
    fn init_nvs(&self) -> PlatformResult<EspNvs<NvsDefault>> {
        log::info!("Initializing NVS for secure storage");

        // Initialize NVS partition
        let nvs_default_partition = EspNvsPartition::<NvsDefault>::take().map_err(|e| {
            PlatformError::Storage(format!("Failed to take NVS partition: {:?}", e))
        })?;

        // Open NVS namespace for secure storage
        let nvs =
            EspNvs::new(nvs_default_partition, SECURE_STORAGE_NAMESPACE, true).map_err(|e| {
                PlatformError::Storage(format!("Failed to open NVS namespace: {:?}", e))
            })?;

        log::info!("NVS initialized successfully");
        Ok(nvs)
    }

    /// Validate storage key format for NVS compatibility
    fn validate_storage_key(key: &str) -> PlatformResult<()> {
        if key.is_empty() {
            return Err(PlatformError::Storage(
                "Storage key cannot be empty".to_string(),
            ));
        }

        if key.len() > MAX_KEY_LENGTH {
            return Err(PlatformError::Storage(format!(
                "Storage key too long: {} > {}",
                key.len(),
                MAX_KEY_LENGTH
            )));
        }

        // NVS keys must be valid C identifiers
        if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(PlatformError::Storage(
                "Storage key must contain only alphanumeric characters and underscores".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for ESP32HAL {
    fn default() -> Self {
        Self::new().expect("Failed to create ESP32HAL")
    }
}

#[async_trait]
impl PlatformHAL for ESP32HAL {
    async fn sleep(&self, duration: Duration) -> PlatformResult<()> {
        log::info!("ESP32-C3 sleeping for {:?}", duration);

        // Convert duration to milliseconds for FreeRTOS
        let duration_ms = duration.as_millis() as u32;

        if duration_ms == 0 {
            // For zero duration, just yield to other tasks
            FreeRtos::delay_ms(0);
        } else {
            // Use FreeRTOS delay for actual sleep
            FreeRtos::delay_ms(duration_ms);
        }

        log::info!("ESP32-C3 wake up after {:?}", duration);
        Ok(())
    }

    async fn set_led(&self, state: LedState) -> PlatformResult<()> {
        log::info!("ESP32-C3 setting LED to {:?}", state);

        let mut led_pin_guard = self
            .led_pin
            .lock()
            .map_err(|_| PlatformError::Hardware("Failed to lock LED pin".to_string()))?;

        let mut led_state_guard = self
            .led_state
            .lock()
            .map_err(|_| PlatformError::Hardware("Failed to lock LED state".to_string()))?;

        if let Some(ref mut led_pin) = *led_pin_guard {
            match state {
                LedState::On => {
                    led_pin.set_high().map_err(|e| {
                        PlatformError::Led(format!("Failed to turn LED on: {:?}", e))
                    })?;
                    log::info!("ESP32-C3 LED turned ON");
                }
                LedState::Off => {
                    led_pin.set_low().map_err(|e| {
                        PlatformError::Led(format!("Failed to turn LED off: {:?}", e))
                    })?;
                    log::info!("ESP32-C3 LED turned OFF");
                }
            }

            *led_state_guard = state;
            Ok(())
        } else {
            Err(PlatformError::Hardware(
                "LED pin not initialized".to_string(),
            ))
        }
    }

    async fn get_led_state(&self) -> PlatformResult<LedState> {
        let led_state_guard = self
            .led_state
            .lock()
            .map_err(|_| PlatformError::Hardware("Failed to lock LED state".to_string()))?;

        Ok(*led_state_guard)
    }

    async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
        log::debug!("Getting ESP32-C3 device information");

        let device_id = Self::get_mac_address()?;
        let (chip_model, hardware_revision) = Self::get_chip_info()?;

        // Get IDF version
        let idf_version = unsafe {
            let version_ptr = esp_idf_sys::esp_get_idf_version();
            let version_cstr = std::ffi::CStr::from_ptr(version_ptr);
            version_cstr.to_string_lossy().to_string()
        };

        Ok(DeviceInfo {
            device_id,
            platform: format!("{} (ESP-IDF {})", chip_model, idf_version),
            version: env!("CARGO_PKG_VERSION").to_string(),
            firmware_version: "1.0.0".to_string(),
            hardware_revision: Some(hardware_revision),
            serial_number: Some(Self::get_mac_address()?),
        })
    }

    async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
        Self::get_esp32_memory_info()
    }

    async fn get_uptime(&self) -> PlatformResult<UptimeInfo> {
        self.get_esp32_uptime()
    }

    async fn store_secure_data(&self, key: &str, data: &[u8]) -> PlatformResult<()> {
        Self::validate_storage_key(key)?;

        if data.len() > MAX_DATA_SIZE {
            return Err(PlatformError::Storage(format!(
                "Data too large: {} > {}",
                data.len(),
                MAX_DATA_SIZE
            )));
        }

        log::info!(
            "Storing secure data for key '{}' ({} bytes)",
            key,
            data.len()
        );

        let nvs_guard = self
            .nvs
            .lock()
            .map_err(|_| PlatformError::Storage("Failed to lock NVS".to_string()))?;

        if let Some(ref nvs) = *nvs_guard {
            // Store data as blob in NVS
            nvs.set_blob(key, data)
                .map_err(|e| PlatformError::Storage(format!("Failed to store data: {:?}", e)))?;

            log::info!("Secure data stored successfully for key '{}'", key);
            Ok(())
        } else {
            Err(PlatformError::Storage("NVS not initialized".to_string()))
        }
    }

    async fn load_secure_data(&self, key: &str) -> PlatformResult<Option<Vec<u8>>> {
        Self::validate_storage_key(key)?;

        log::debug!("Loading secure data for key '{}'", key);

        let nvs_guard = self
            .nvs
            .lock()
            .map_err(|_| PlatformError::Storage("Failed to lock NVS".to_string()))?;

        if let Some(ref nvs) = *nvs_guard {
            // First, get the size of the data
            match nvs.blob_len(key) {
                Ok(Some(size)) => {
                    // Allocate buffer and read data
                    let mut buffer = vec![0u8; size];
                    match nvs.get_blob(key, &mut buffer) {
                        Ok(actual_size) => {
                            buffer.truncate(actual_size);
                            log::debug!("Loaded {} bytes for key '{}'", actual_size, key);
                            Ok(Some(buffer))
                        }
                        Err(e) => Err(PlatformError::Storage(format!(
                            "Failed to read data: {:?}",
                            e
                        ))),
                    }
                }
                Ok(None) => {
                    log::debug!("No data found for key '{}'", key);
                    Ok(None)
                }
                Err(e) => Err(PlatformError::Storage(format!(
                    "Failed to get data size: {:?}",
                    e
                ))),
            }
        } else {
            Err(PlatformError::Storage("NVS not initialized".to_string()))
        }
    }

    async fn delete_secure_data(&self, key: &str) -> PlatformResult<bool> {
        Self::validate_storage_key(key)?;

        log::info!("Deleting secure data for key '{}'", key);

        let nvs_guard = self
            .nvs
            .lock()
            .map_err(|_| PlatformError::Storage("Failed to lock NVS".to_string()))?;

        if let Some(ref nvs) = *nvs_guard {
            match nvs.remove(key) {
                Ok(()) => {
                    log::info!("Secure data deleted successfully for key '{}'", key);
                    Ok(true)
                }
                Err(embedded_svc::storage::StorageError::NotFound) => {
                    log::debug!("No data found to delete for key '{}'", key);
                    Ok(false)
                }
                Err(e) => Err(PlatformError::Storage(format!(
                    "Failed to delete data: {:?}",
                    e
                ))),
            }
        } else {
            Err(PlatformError::Storage("NVS not initialized".to_string()))
        }
    }

    async fn list_secure_keys(&self) -> PlatformResult<Vec<String>> {
        log::debug!("Listing secure storage keys");

        let nvs_guard = self
            .nvs
            .lock()
            .map_err(|_| PlatformError::Storage("Failed to lock NVS".to_string()))?;

        if let Some(ref nvs) = *nvs_guard {
            // NVS doesn't provide a direct way to list keys, so we'll use a workaround
            // by trying to iterate through possible keys or maintaining a key index
            // For now, return empty list as NVS key enumeration is complex
            log::warn!("NVS key listing not fully implemented - returning empty list");
            Ok(Vec::new())
        } else {
            Err(PlatformError::Storage("NVS not initialized".to_string()))
        }
    }

    async fn initialize(&mut self) -> PlatformResult<()> {
        let mut initialized_guard = self.initialized.lock().map_err(|_| {
            PlatformError::Hardware("Failed to lock initialization state".to_string())
        })?;

        if *initialized_guard {
            return Err(PlatformError::Hardware(
                "ESP32-C3 HAL already initialized".to_string(),
            ));
        }

        log::info!("Initializing ESP32-C3 HAL...");

        // Initialize ESP-IDF
        esp_idf_sys::link_patches();

        // Take peripherals
        let peripherals = Peripherals::take().map_err(|_| {
            PlatformError::Hardware("Failed to take ESP32-C3 peripherals".to_string())
        })?;

        // Initialize LED GPIO pin
        log::info!("Initializing LED on GPIO{}", LED_GPIO_PIN);
        let led_pin = PinDriver::output(peripherals.pins.gpio2).map_err(|e| {
            PlatformError::Hardware(format!("Failed to initialize LED pin: {:?}", e))
        })?;

        // Store LED pin driver
        {
            let mut led_pin_guard = self
                .led_pin
                .lock()
                .map_err(|_| PlatformError::Hardware("Failed to lock LED pin".to_string()))?;
            *led_pin_guard = Some(led_pin);
        }

        // Initialize LED to OFF state
        self.set_led(LedState::Off).await?;

        // Initialize NVS for secure storage
        let nvs = self.init_nvs()?;
        {
            let mut nvs_guard = self
                .nvs
                .lock()
                .map_err(|_| PlatformError::Storage("Failed to lock NVS".to_string()))?;
            *nvs_guard = Some(nvs);
        }

        *initialized_guard = true;
        log::info!("ESP32-C3 HAL initialized successfully");
        Ok(())
    }

    async fn shutdown(&mut self) -> PlatformResult<()> {
        let mut initialized_guard = self.initialized.lock().map_err(|_| {
            PlatformError::Hardware("Failed to lock initialization state".to_string())
        })?;

        if !*initialized_guard {
            return Err(PlatformError::Hardware(
                "ESP32-C3 HAL not initialized".to_string(),
            ));
        }

        log::info!("Shutting down ESP32-C3 HAL...");

        // Turn off LED
        if let Err(e) = self.set_led(LedState::Off).await {
            log::warn!("Failed to turn off LED during shutdown: {:?}", e);
        }

        // Clear LED pin driver
        {
            let mut led_pin_guard = self
                .led_pin
                .lock()
                .map_err(|_| PlatformError::Hardware("Failed to lock LED pin".to_string()))?;
            *led_pin_guard = None;
        }

        // Clear NVS
        {
            let mut nvs_guard = self
                .nvs
                .lock()
                .map_err(|_| PlatformError::Storage("Failed to lock NVS".to_string()))?;
            *nvs_guard = None;
        }

        *initialized_guard = false;
        log::info!("ESP32-C3 HAL shutdown successfully");
        Ok(())
    }
}
