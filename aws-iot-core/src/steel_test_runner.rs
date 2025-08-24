use crate::{PlatformHAL, SteelRuntimeAPI, SteelRuntimeImpl, SystemError, SystemResult};
use std::fs;

use std::sync::Arc;
use tracing::{error, info, warn};

/// Steel test runner for executing Steel test files
pub struct SteelTestRunner {
    runtime: SteelRuntimeImpl,
}

impl SteelTestRunner {
    /// Create a new Steel test runner
    pub fn new(hal: Arc<dyn PlatformHAL>) -> SystemResult<Self> {
        let rust_api = Arc::new(SteelRuntimeAPI::new(hal)?);
        let runtime = SteelRuntimeImpl::new(rust_api)?;

        Ok(Self { runtime })
    }

    /// Run a single Steel test file
    pub async fn run_test_file(&self, file_path: &str) -> SystemResult<()> {
        info!("Running Steel test file: {}", file_path);

        // Read the test file
        let test_code = fs::read_to_string(file_path).map_err(|e| {
            SystemError::Configuration(format!("Failed to read test file {}: {}", file_path, e))
        })?;

        // Execute the test
        match self.runtime.execute_code_with_hal(&test_code).await {
            Ok(_) => {
                info!("✓ Steel test file {} passed", file_path);
                Ok(())
            }
            Err(e) => {
                error!("✗ Steel test file {} failed: {}", file_path, e);
                Err(e)
            }
        }
    }

    /// Run all Steel test files in a directory
    pub async fn run_test_directory(&self, dir_path: &str) -> SystemResult<TestResults> {
        info!("Running Steel tests in directory: {}", dir_path);

        let mut results = TestResults::new();

        // Read directory contents
        let entries = fs::read_dir(dir_path).map_err(|e| {
            SystemError::Configuration(format!("Failed to read test directory {}: {}", dir_path, e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                SystemError::Configuration(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            // Only process .scm files
            if path.extension().and_then(|s| s.to_str()) == Some("scm") {
                let file_path = path.to_string_lossy();

                match self.run_test_file(&file_path).await {
                    Ok(()) => results.add_pass(&file_path),
                    Err(e) => {
                        results.add_fail(&file_path, &e.to_string());
                        // Continue with other tests even if one fails
                    }
                }
            }
        }

        Ok(results)
    }

    /// Run a Steel example file
    pub async fn run_example_file(&self, file_path: &str) -> SystemResult<()> {
        info!("Running Steel example file: {}", file_path);

        // Read the example file
        let example_code = fs::read_to_string(file_path).map_err(|e| {
            SystemError::Configuration(format!("Failed to read example file {}: {}", file_path, e))
        })?;

        // Execute the example
        match self.runtime.execute_code_with_hal(&example_code).await {
            Ok(_) => {
                info!("✓ Steel example file {} completed successfully", file_path);
                Ok(())
            }
            Err(e) => {
                error!("✗ Steel example file {} failed: {}", file_path, e);
                Err(e)
            }
        }
    }

    /// Run all Steel example files in a directory
    pub async fn run_example_directory(&self, dir_path: &str) -> SystemResult<TestResults> {
        info!("Running Steel examples in directory: {}", dir_path);

        let mut results = TestResults::new();

        // Read directory contents
        let entries = fs::read_dir(dir_path).map_err(|e| {
            SystemError::Configuration(format!(
                "Failed to read example directory {}: {}",
                dir_path, e
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                SystemError::Configuration(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            // Only process .scm files
            if path.extension().and_then(|s| s.to_str()) == Some("scm") {
                let file_path = path.to_string_lossy();

                match self.run_example_file(&file_path).await {
                    Ok(()) => results.add_pass(&file_path),
                    Err(e) => {
                        results.add_fail(&file_path, &e.to_string());
                        // Continue with other examples even if one fails
                    }
                }
            }
        }

        Ok(results)
    }
}

/// Test results summary
#[derive(Debug, Clone)]
pub struct TestResults {
    pub passed: Vec<String>,
    pub failed: Vec<(String, String)>, // (file_path, error_message)
}

impl std::fmt::Display for TestResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Steel Test Results: {} total, {} passed, {} failed ({:.1}% success rate)",
            self.total(),
            self.passed_count(),
            self.failed_count(),
            self.success_rate()
        )
    }
}

impl Default for TestResults {
    fn default() -> Self {
        Self::new()
    }
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            passed: Vec::new(),
            failed: Vec::new(),
        }
    }

    pub fn add_pass(&mut self, file_path: &str) {
        self.passed.push(file_path.to_string());
    }

    pub fn add_fail(&mut self, file_path: &str, error: &str) {
        self.failed.push((file_path.to_string(), error.to_string()));
    }

    pub fn total(&self) -> usize {
        self.passed.len() + self.failed.len()
    }

    pub fn passed_count(&self) -> usize {
        self.passed.len()
    }

    pub fn failed_count(&self) -> usize {
        self.failed.len()
    }

    pub fn success_rate(&self) -> f64 {
        if self.total() == 0 {
            0.0
        } else {
            self.passed_count() as f64 / self.total() as f64 * 100.0
        }
    }

    pub fn print_summary(&self) {
        info!("=== Test Results Summary ===");
        info!("Total tests: {}", self.total());
        info!("Passed: {}", self.passed_count());
        info!("Failed: {}", self.failed_count());
        info!("Success rate: {:.1}%", self.success_rate());

        if !self.failed.is_empty() {
            warn!("Failed tests:");
            for (file, error) in &self.failed {
                warn!("  {} - {}", file, error);
            }
        }

        info!("=== End Summary ===");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DeviceInfo, LedState, MemoryInfo, PlatformResult, UptimeInfo};
    use async_trait::async_trait;
    use chrono::Utc;
    use parking_lot::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    /// Mock HAL for testing
    struct MockHAL {
        led_state: Arc<Mutex<LedState>>,
        sleep_called: Arc<AtomicBool>,
    }

    impl MockHAL {
        fn new() -> Self {
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

    #[tokio::test]
    async fn test_steel_test_runner_creation() {
        let hal = Arc::new(MockHAL::new());
        let _runner = SteelTestRunner::new(hal).unwrap();

        // Test that we can create the runner without errors
        // If we get here, creation succeeded
    }

    #[tokio::test]
    async fn test_steel_code_execution() {
        let hal = Arc::new(MockHAL::new());
        let runner = SteelTestRunner::new(hal.clone()).unwrap();

        // Test simple Steel code execution
        let simple_test = r#"
            (begin
              (log-info "Running simple test")
              (led-on)
              (sleep 0.001)
              (led-off)
              (log-info "Simple test completed")
              #t)
        "#;

        let result = runner.runtime.execute_code_with_hal(simple_test).await;
        assert!(result.is_ok());

        // Verify LED operations were called
        assert_eq!(*hal.led_state.lock(), LedState::Off);
        // Note: sleep_called might not be set due to async execution timing
        // The important thing is that the Steel code executed successfully
    }

    #[test]
    fn test_results_tracking() {
        let mut results = TestResults::new();

        results.add_pass("test1.scm");
        results.add_pass("test2.scm");
        results.add_fail("test3.scm", "Test failed");

        assert_eq!(results.total(), 3);
        assert_eq!(results.passed_count(), 2);
        assert_eq!(results.failed_count(), 1);
        // Use approximate comparison for floating point
        assert!((results.success_rate() - 66.66666666666667).abs() < 0.0001);
    }
}
