use aws_iot_core::*;
use clap::{Arg, Command};
use std::path::Path;
use std::sync::Arc;
use tokio;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let matches = Command::new("steel_test")
        .about("Steel Test Runner")
        .arg(
            Arg::new("test-dir")
                .long("test-dir")
                .value_name("DIR")
                .help("Directory containing Steel test files")
                .default_value("tests/steel")
        )
        .arg(
            Arg::new("file")
                .long("file")
                .value_name("FILE")
                .help("Run a specific Steel test file")
        )
        .get_matches();
    
    // Create mock HAL for testing
    let hal = Arc::new(create_mock_hal());
    let runner = SteelTestRunner::new(hal)?;
    
    if let Some(test_file) = matches.get_one::<String>("file") {
        // Run a specific test file
        let path = Path::new(test_file);
        runner.run_test_file(&path.to_string_lossy()).await?;
    } else {
        // Run all tests in directory
        let test_dir = matches.get_one::<String>("test-dir").unwrap();
        
        match runner.run_test_directory(test_dir).await {
            Ok(results) => {
                info!("{}", results);
                if results.failed_count() > 0 {
                    std::process::exit(1);
                }
            }
            Err(e) => {
                error!("Failed to run tests: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}

fn create_mock_hal() -> impl PlatformHAL {
    use async_trait::async_trait;
    use chrono::Utc;
    use parking_lot::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    
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
        async fn sleep(&self, duration: Duration) -> PlatformResult<()> {
            self.sleep_called.store(true, Ordering::SeqCst);
            // Simulate sleep without actually sleeping for long periods in tests
            let sleep_duration = if duration > Duration::from_millis(100) {
                Duration::from_millis(10) // Cap sleep duration for tests
            } else {
                duration
            };
            tokio::time::sleep(sleep_duration).await;
            Ok(())
        }
        
        async fn set_led(&self, state: LedState) -> PlatformResult<()> {
            *self.led_state.lock() = state;
            println!("LED set to: {:?}", state);
            Ok(())
        }
        
        async fn get_led_state(&self) -> PlatformResult<LedState> {
            Ok(*self.led_state.lock())
        }
        
        async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
            Ok(DeviceInfo {
                device_id: "steel-test-device".to_string(),
                platform: "test".to_string(),
                version: "1.0.0".to_string(),
                firmware_version: "1.0.0".to_string(),
                hardware_revision: Some("test-rev1".to_string()),
                serial_number: Some("STEEL12345".to_string()),
            })
        }
        
        async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
            Ok(MemoryInfo {
                total_bytes: 4096 * 1024,
                free_bytes: 2048 * 1024,
                used_bytes: 2048 * 1024,
                largest_free_block: 1024 * 1024,
            })
        }
        
        async fn get_uptime(&self) -> PlatformResult<UptimeInfo> {
            Ok(UptimeInfo {
                uptime: Duration::from_secs(1800),
                boot_time: Utc::now() - chrono::Duration::seconds(1800),
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
    
    MockHAL::new()
}