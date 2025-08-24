/// Hardware-specific tests for ESP32-C3-DevKit-RUST-1
/// These tests validate actual hardware integration and should be run on the target device
use crate::ESP32HAL;
use aws_iot_core::{LedState, PlatformHAL};
use log::{info, warn};
use std::time::{Duration, Instant};

/// Test suite for ESP32-C3 hardware validation
pub struct ESP32HardwareTests {
    hal: ESP32HAL,
}

impl ESP32HardwareTests {
    /// Create a new hardware test suite
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut hal = ESP32HAL::new()?;
        hal.initialize().await?;

        Ok(Self { hal })
    }

    /// Run all hardware validation tests
    pub async fn run_all_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting ESP32-C3 hardware validation tests");

        self.test_led_hardware().await?;
        self.test_sleep_accuracy().await?;
        self.test_memory_monitoring().await?;
        self.test_secure_storage_persistence().await?;
        self.test_power_management().await?;
        self.test_gpio_stability().await?;

        info!("All ESP32-C3 hardware tests completed successfully");
        Ok(())
    }

    /// Test LED hardware functionality with visual verification
    async fn test_led_hardware(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Testing LED hardware functionality");

        // Test LED blinking pattern for visual verification
        for i in 0..5 {
            info!("LED blink cycle {} of 5", i + 1);

            // Turn LED ON
            self.hal.set_led(LedState::On).await?;
            let state = self.hal.get_led_state().await?;
            assert_eq!(state, LedState::On, "LED should be ON");

            // Wait 500ms
            self.hal.sleep(Duration::from_millis(500)).await?;

            // Turn LED OFF
            self.hal.set_led(LedState::Off).await?;
            let state = self.hal.get_led_state().await?;
            assert_eq!(state, LedState::Off, "LED should be OFF");

            // Wait 500ms
            self.hal.sleep(Duration::from_millis(500)).await?;
        }

        info!("LED hardware test completed - verify LED blinked 5 times");
        Ok(())
    }

    /// Test sleep timing accuracy
    async fn test_sleep_accuracy(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Testing sleep timing accuracy");

        let test_durations = vec![
            Duration::from_millis(10),
            Duration::from_millis(100),
            Duration::from_millis(500),
            Duration::from_secs(1),
        ];

        for duration in test_durations {
            let start_time = Instant::now();
            self.hal.sleep(duration).await?;
            let elapsed = start_time.elapsed();

            let accuracy = (elapsed.as_millis() as f64 / duration.as_millis() as f64) * 100.0;
            info!(
                "Sleep accuracy for {:?}: {:.2}% (actual: {:?})",
                duration, accuracy, elapsed
            );

            // Allow 10% tolerance for timing accuracy
            assert!(
                elapsed >= duration
                    && elapsed < duration + Duration::from_millis(duration.as_millis() as u64 / 10),
                "Sleep timing should be accurate within 10%"
            );
        }

        info!("Sleep timing accuracy test completed");
        Ok(())
    }

    /// Test memory monitoring under load
    async fn test_memory_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Testing memory monitoring under load");

        // Get baseline memory info
        let baseline_memory = self.hal.get_memory_info().await?;
        info!(
            "Baseline memory: {} bytes free, {} bytes used",
            baseline_memory.free_bytes, baseline_memory.used_bytes
        );

        // Allocate some memory to test monitoring
        let mut test_vectors: Vec<Vec<u8>> = Vec::new();

        for i in 0..10 {
            // Allocate 1KB chunks
            let chunk = vec![i as u8; 1024];
            test_vectors.push(chunk);

            let current_memory = self.hal.get_memory_info().await?;
            info!(
                "After allocation {}: {} bytes free, {} bytes used",
                i + 1,
                current_memory.free_bytes,
                current_memory.used_bytes
            );

            // Verify memory usage increased
            assert!(
                current_memory.used_bytes >= baseline_memory.used_bytes,
                "Memory usage should increase with allocations"
            );
        }

        // Drop allocations
        drop(test_vectors);

        // Force garbage collection (if applicable)
        self.hal.sleep(Duration::from_millis(100)).await?;

        let final_memory = self.hal.get_memory_info().await?;
        info!(
            "Final memory: {} bytes free, {} bytes used",
            final_memory.free_bytes, final_memory.used_bytes
        );

        info!("Memory monitoring test completed");
        Ok(())
    }

    /// Test secure storage persistence across operations
    async fn test_secure_storage_persistence(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Testing secure storage persistence");

        let test_key = "persist_test";
        let test_data = b"Persistent test data for ESP32-C3";

        // Store data
        self.hal.store_secure_data(test_key, test_data).await?;
        info!("Stored test data in secure storage");

        // Verify immediate read
        let loaded_data = self.hal.load_secure_data(test_key).await?;
        assert_eq!(
            loaded_data.as_ref(),
            Some(test_data),
            "Data should be immediately readable"
        );

        // Perform some operations to test persistence
        for i in 0..5 {
            self.hal.set_led(LedState::On).await?;
            self.hal.sleep(Duration::from_millis(100)).await?;
            self.hal.set_led(LedState::Off).await?;
            self.hal.sleep(Duration::from_millis(100)).await?;

            // Verify data still exists
            let loaded_data = self.hal.load_secure_data(test_key).await?;
            assert_eq!(
                loaded_data.as_ref(),
                Some(test_data),
                "Data should persist through operations (iteration {})",
                i + 1
            );
        }

        // Clean up
        let deleted = self.hal.delete_secure_data(test_key).await?;
        assert!(deleted, "Test data should be successfully deleted");

        info!("Secure storage persistence test completed");
        Ok(())
    }

    /// Test power management features
    async fn test_power_management(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Testing power management features");

        // Get initial memory and uptime
        let initial_memory = self.hal.get_memory_info().await?;
        let initial_uptime = self.hal.get_uptime().await?;

        info!(
            "Initial state - Memory: {} bytes free, Uptime: {:?}",
            initial_memory.free_bytes, initial_uptime.uptime
        );

        // Perform sleep operations of varying lengths
        let sleep_durations = vec![
            Duration::from_millis(50),
            Duration::from_millis(200),
            Duration::from_millis(500),
        ];

        for (i, duration) in sleep_durations.iter().enumerate() {
            info!(
                "Power management test cycle {} - sleeping for {:?}",
                i + 1,
                duration
            );

            // Turn on LED before sleep
            self.hal.set_led(LedState::On).await?;

            // Sleep
            let sleep_start = Instant::now();
            self.hal.sleep(*duration).await?;
            let sleep_elapsed = sleep_start.elapsed();

            // Turn off LED after sleep
            self.hal.set_led(LedState::Off).await?;

            // Check system state after sleep
            let post_sleep_memory = self.hal.get_memory_info().await?;
            let post_sleep_uptime = self.hal.get_uptime().await?;

            info!(
                "Post-sleep state - Memory: {} bytes free, Uptime: {:?}, Sleep accuracy: {:?}",
                post_sleep_memory.free_bytes, post_sleep_uptime.uptime, sleep_elapsed
            );

            // Verify system is still responsive
            assert!(
                post_sleep_uptime.uptime > initial_uptime.uptime,
                "Uptime should increase after sleep"
            );
        }

        info!("Power management test completed");
        Ok(())
    }

    /// Test GPIO stability under rapid operations
    async fn test_gpio_stability(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Testing GPIO stability under rapid operations");

        // Perform rapid LED state changes
        let mut state = LedState::Off;

        for i in 0..100 {
            state = if state == LedState::Off {
                LedState::On
            } else {
                LedState::Off
            };

            self.hal.set_led(state).await?;
            let read_state = self.hal.get_led_state().await?;

            assert_eq!(
                read_state,
                state,
                "LED state should be consistent (iteration {})",
                i + 1
            );

            // Small delay to avoid overwhelming the GPIO
            if i % 10 == 0 {
                self.hal.sleep(Duration::from_millis(1)).await?;
                info!("GPIO stability test progress: {}/100", i + 1);
            }
        }

        // Ensure LED is off at the end
        self.hal.set_led(LedState::Off).await?;

        info!("GPIO stability test completed - performed 100 rapid state changes");
        Ok(())
    }

    /// Test device information consistency
    pub async fn test_device_info_consistency(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Testing device information consistency");

        // Get device info multiple times and verify consistency
        let mut device_infos = Vec::new();

        for i in 0..5 {
            let device_info = self.hal.get_device_info().await?;
            device_infos.push(device_info);

            info!(
                "Device info reading {}: ID={}, Platform={}",
                i + 1,
                device_infos[i].device_id,
                device_infos[i].platform
            );

            // Small delay between readings
            self.hal.sleep(Duration::from_millis(100)).await?;
        }

        // Verify all readings are consistent
        let first_info = &device_infos[0];
        for (i, info) in device_infos.iter().enumerate().skip(1) {
            assert_eq!(
                info.device_id,
                first_info.device_id,
                "Device ID should be consistent (reading {})",
                i + 1
            );
            assert_eq!(
                info.platform,
                first_info.platform,
                "Platform should be consistent (reading {})",
                i + 1
            );
            assert_eq!(
                info.firmware_version,
                first_info.firmware_version,
                "Firmware version should be consistent (reading {})",
                i + 1
            );
        }

        info!("Device information consistency test completed");
        Ok(())
    }

    /// Cleanup and shutdown the test suite
    pub async fn cleanup(mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Cleaning up ESP32-C3 hardware test suite");

        // Ensure LED is off
        if let Err(e) = self.hal.set_led(LedState::Off).await {
            warn!("Failed to turn off LED during cleanup: {:?}", e);
        }

        // Shutdown HAL
        if let Err(e) = self.hal.shutdown().await {
            warn!("Failed to shutdown HAL during cleanup: {:?}", e);
        }

        info!("ESP32-C3 hardware test suite cleanup completed");
        Ok(())
    }
}

/// Run hardware validation tests
pub async fn run_hardware_validation() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting ESP32-C3 hardware validation");

    let test_suite = ESP32HardwareTests::new().await?;

    // Run all tests
    test_suite.run_all_tests().await?;
    test_suite.test_device_info_consistency().await?;

    // Cleanup
    test_suite.cleanup().await?;

    info!("ESP32-C3 hardware validation completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Ignore by default as this requires actual hardware
    async fn test_hardware_validation() {
        env_logger::init();

        let result = run_hardware_validation().await;
        assert!(
            result.is_ok(),
            "Hardware validation should succeed: {:?}",
            result
        );
    }
}
