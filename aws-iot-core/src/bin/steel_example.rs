use aws_iot_core::*;
use clap::{Arg, Command};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let matches = Command::new("steel_example")
        .about("Steel Example Runner")
        .arg(
            Arg::new("example-dir")
                .long("example-dir")
                .value_name("DIR")
                .help("Directory containing Steel example files")
                .default_value("examples/steel")
        )
        .arg(
            Arg::new("file")
                .long("file")
                .value_name("FILE")
                .help("Run a specific Steel example file")
        )
        .arg(
            Arg::new("list")
                .long("list")
                .help("List available Steel examples")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();
    
    let example_dir = matches.get_one::<String>("example-dir").unwrap();
    let example_path = Path::new(example_dir);
    
    if matches.get_flag("list") {
        // List available examples
        list_examples(example_path)?;
        return Ok(());
    }
    
    // Create mock HAL for examples
    let hal = Arc::new(create_mock_hal());
    let runner = SteelTestRunner::new(hal)?;
    
    if let Some(example_file) = matches.get_one::<String>("file") {
        // Run a specific example file
        let path = Path::new(example_file);
        runner.run_example_file(&path.to_string_lossy()).await?;
    } else {
        // Run all examples in directory
        run_all_examples(&runner, example_path).await?;
    }
    
    Ok(())
}

fn list_examples(example_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    info!("Available Steel examples in {}:", example_dir.display());
    
    if !example_dir.exists() {
        error!("Example directory does not exist: {}", example_dir.display());
        return Ok(());
    }
    
    let entries = fs::read_dir(example_dir)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("scm") {
            let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
            info!("  - {}", name);
        }
    }
    
    Ok(())
}

async fn run_all_examples(runner: &SteelTestRunner, example_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running all Steel examples in: {}", example_dir.display());
    
    if !example_dir.exists() {
        error!("Example directory does not exist: {}", example_dir.display());
        return Ok(());
    }
    
    let entries = fs::read_dir(example_dir)?;
    let mut total = 0;
    let mut successful = 0;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("scm") {
            total += 1;
            
            match runner.run_example_file(&path.to_string_lossy()).await {
                Ok(_) => {
                    successful += 1;
                    info!("\\n{}\\n", "=".repeat(50));
                }
                Err(e) => {
                    error!("Example failed: {}", e);
                    info!("\\n{}\\n", "=".repeat(50));
                }
            }
        }
    }
    
    info!("Examples completed: {}/{} successful", successful, total);
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
            // For examples, we'll actually sleep but cap it to reasonable durations
            let sleep_duration = if duration > Duration::from_secs(2) {
                Duration::from_millis(200) // Cap long sleeps for demo purposes
            } else {
                duration
            };
            tokio::time::sleep(sleep_duration).await;
            Ok(())
        }
        
        async fn set_led(&self, state: LedState) -> PlatformResult<()> {
            *self.led_state.lock() = state;
            println!("🔆 LED set to: {:?}", state);
            Ok(())
        }
        
        async fn get_led_state(&self) -> PlatformResult<LedState> {
            Ok(*self.led_state.lock())
        }
        
        async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
            Ok(DeviceInfo {
                device_id: "steel-example-device".to_string(),
                platform: "example-platform".to_string(),
                version: "1.0.0".to_string(),
                firmware_version: "1.0.0".to_string(),
                hardware_revision: Some("example-rev1".to_string()),
                serial_number: Some("EXAMPLE12345".to_string()),
            })
        }
        
        async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
            Ok(MemoryInfo {
                total_bytes: 8192 * 1024,
                free_bytes: 4096 * 1024,
                used_bytes: 4096 * 1024,
                largest_free_block: 2048 * 1024,
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
    
    MockHAL::new()
}