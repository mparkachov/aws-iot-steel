use aws_iot_core::*;
use std::env;
use std::sync::Arc;
use tokio;
use tracing::{info, error};
use tracing_subscriber;

/// Mock HAL for test runner
struct TestHAL {
    led_state: parking_lot::Mutex<LedState>,
    sleep_called: std::sync::atomic::AtomicBool,
}

impl TestHAL {
    fn new() -> Self {
        Self {
            led_state: parking_lot::Mutex::new(LedState::Off),
            sleep_called: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

#[async_trait::async_trait]
impl PlatformHAL for TestHAL {
    async fn sleep(&self, _duration: std::time::Duration) -> PlatformResult<()> {
        self.sleep_called.store(true, std::sync::atomic::Ordering::SeqCst);
        tokio::time::sleep(std::time::Duration::from_millis(10)).await; // Short delay for realism
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
            device_id: "test-runner-device".to_string(),
            platform: "test-runner".to_string(),
            version: "1.0.0".to_string(),
            firmware_version: "1.0.0".to_string(),
            hardware_revision: Some("test-rev1".to_string()),
            serial_number: Some("TEST12345".to_string()),
        })
    }

    async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
        Ok(MemoryInfo {
            total_bytes: 2 * 1024 * 1024, // 2MB
            free_bytes: 1024 * 1024,      // 1MB
            used_bytes: 1024 * 1024,      // 1MB
            largest_free_block: 512 * 1024, // 512KB
        })
    }

    async fn get_uptime(&self) -> PlatformResult<UptimeInfo> {
        Ok(UptimeInfo {
            uptime: std::time::Duration::from_secs(7200), // 2 hours
            boot_time: chrono::Utc::now() - chrono::Duration::seconds(7200),
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

async fn run_steel_tests() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Running Steel Tests ===");
    
    let hal = Arc::new(TestHAL::new());
    let runner = SteelTestRunner::new(hal)?;
    
    // Run Steel tests
    let test_dir = "aws-iot-core/tests/steel";
    match runner.run_test_directory(test_dir).await {
        Ok(results) => {
            results.print_summary();
            if results.failed_count() > 0 {
                error!("Some Steel tests failed");
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Failed to run Steel tests: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

async fn run_steel_examples() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Running Steel Examples ===");
    
    let hal = Arc::new(TestHAL::new());
    let runner = SteelTestRunner::new(hal)?;
    
    // Run Steel examples
    let example_dir = "aws-iot-core/examples/steel";
    match runner.run_example_directory(example_dir).await {
        Ok(results) => {
            results.print_summary();
            if results.failed_count() > 0 {
                error!("Some Steel examples failed");
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Failed to run Steel examples: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

async fn run_single_steel_file(file_path: &str, is_example: bool) -> Result<(), Box<dyn std::error::Error>> {
    let hal = Arc::new(TestHAL::new());
    let runner = SteelTestRunner::new(hal)?;
    
    if is_example {
        info!("Running Steel example: {}", file_path);
        runner.run_example_file(file_path).await?;
    } else {
        info!("Running Steel test: {}", file_path);
        runner.run_test_file(file_path).await?;
    }
    
    Ok(())
}

fn print_usage() {
    println!("Steel Test Runner");
    println!("Usage:");
    println!("  test_runner steel-tests          - Run all Steel tests");
    println!("  test_runner steel-examples       - Run all Steel examples");
    println!("  test_runner steel-test <file>    - Run specific Steel test file");
    println!("  test_runner steel-example <file> - Run specific Steel example file");
    println!("  test_runner --help               - Show this help");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    match args[1].as_str() {
        "steel-tests" => {
            run_steel_tests().await?;
        }
        "steel-examples" => {
            run_steel_examples().await?;
        }
        "steel-test" => {
            if args.len() < 3 {
                error!("Please specify a test file path");
                std::process::exit(1);
            }
            run_single_steel_file(&args[2], false).await?;
        }
        "steel-example" => {
            if args.len() < 3 {
                error!("Please specify an example file path");
                std::process::exit(1);
            }
            run_single_steel_file(&args[2], true).await?;
        }
        "--help" | "-h" => {
            print_usage();
        }
        _ => {
            error!("Unknown command: {}", args[1]);
            print_usage();
            std::process::exit(1);
        }
    }
    
    Ok(())
}