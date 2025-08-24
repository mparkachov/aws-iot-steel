use aws_iot_core::*;
use clap::{Arg, Command};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let matches = Command::new("steel_example")
        .version("1.0")
        .about("Steel Example Runner for AWS IoT Steel module")
        .arg(
            Arg::new("example-dir")
                .short('d')
                .long("example-dir")
                .value_name("DIR")
                .help("Directory containing Steel example files")
                .default_value("aws-iot-core/examples/steel"),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Run a specific Steel example file"),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .help("List available Steel examples")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Enable verbose output"),
        )
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .action(clap::ArgAction::SetTrue)
                .help("Run examples in interactive mode with pauses"),
        )
        .get_matches();

    let example_dir = matches.get_one::<String>("example-dir").unwrap();
    let example_path = Path::new(example_dir);
    let verbose = matches.get_flag("verbose");
    let interactive = matches.get_flag("interactive");

    info!("=== Steel Example Runner Starting ===");

    if matches.get_flag("list") {
        // List available examples
        list_examples(example_path, verbose)?;
        return Ok(());
    }

    // Create mock HAL for examples
    info!("Initializing example HAL...");
    let hal = Arc::new(create_mock_hal(interactive));

    info!("Creating Steel example runner...");
    let runner = SteelTestRunner::new(hal)?;

    if let Some(example_file) = matches.get_one::<String>("file") {
        // Run a specific example file
        info!("Running Steel example: {}", example_file);

        let start_time = std::time::Instant::now();
        let path = Path::new(example_file);

        match runner.run_example_file(&path.to_string_lossy()).await {
            Ok(()) => {
                let duration = start_time.elapsed();
                info!(
                    "✓ Example completed successfully: {} (duration: {:?})",
                    example_file, duration
                );

                if verbose {
                    info!("Example execution details:");
                    info!("  File: {}", example_file);
                    info!("  Duration: {:?}", duration);
                    info!("  Status: COMPLETED");
                }
            }
            Err(e) => {
                let duration = start_time.elapsed();
                error!(
                    "✗ Example failed: {} - {} (failed after {:?})",
                    example_file, e, duration
                );

                if verbose {
                    error!("Example execution details:");
                    error!("  File: {}", example_file);
                    error!("  Duration: {:?}", duration);
                    error!("  Status: FAILED");
                    error!("  Error: {}", e);
                }

                std::process::exit(1);
            }
        }
    } else {
        // Run all examples in directory
        run_all_examples(&runner, example_path, verbose, interactive).await?;
    }

    info!("=== Steel Example Runner Completed ===");
    Ok(())
}

fn list_examples(example_dir: &Path, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Available Steel Examples ===");
    info!("Directory: {}", example_dir.display());

    if !example_dir.exists() {
        error!(
            "Example directory does not exist: {}",
            example_dir.display()
        );
        return Ok(());
    }

    let entries = fs::read_dir(example_dir)?;
    let mut examples = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("scm") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            if verbose {
                // Try to read the first few lines for description
                if let Ok(content) = fs::read_to_string(&path) {
                    let description = extract_description(&content);
                    examples.push((name.to_string(), description, path.display().to_string()));
                } else {
                    examples.push((
                        name.to_string(),
                        "Unable to read file".to_string(),
                        path.display().to_string(),
                    ));
                }
            } else {
                examples.push((name.to_string(), String::new(), path.display().to_string()));
            }
        }
    }

    if examples.is_empty() {
        warn!("No Steel example files found in {}", example_dir.display());
    } else {
        info!("Found {} Steel examples:", examples.len());

        for (i, (name, description, path)) in examples.iter().enumerate() {
            if verbose {
                info!("{}. {} - {}", i + 1, name, description);
                info!("   Path: {}", path);
            } else {
                info!("  - {}", name);
            }
        }

        if !verbose {
            info!("");
            info!("Use --verbose flag to see descriptions and paths");
        }
    }

    Ok(())
}

fn extract_description(content: &str) -> String {
    // Look for description in comments at the top of the file
    for line in content.lines().take(10) {
        let trimmed = line.trim();
        if trimmed.starts_with(";;") && trimmed.len() > 3 {
            let desc = trimmed[2..].trim();
            if !desc.is_empty() && !desc.starts_with("Steel") {
                return desc.to_string();
            }
        }
    }
    "No description available".to_string()
}

async fn run_all_examples(
    runner: &SteelTestRunner,
    example_dir: &Path,
    verbose: bool,
    interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running all Steel examples in: {}", example_dir.display());

    if !example_dir.exists() {
        error!(
            "Example directory does not exist: {}",
            example_dir.display()
        );
        return Ok(());
    }

    let entries = fs::read_dir(example_dir)?;
    let mut examples = Vec::new();

    // Collect all example files
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("scm") {
            examples.push(path);
        }
    }

    if examples.is_empty() {
        warn!("No Steel example files found in {}", example_dir.display());
        return Ok(());
    }

    // Sort examples for consistent execution order
    examples.sort();

    let mut total = 0;
    let mut successful = 0;
    let start_time = std::time::Instant::now();

    info!("Found {} Steel examples to run", examples.len());

    for (i, path) in examples.iter().enumerate() {
        total += 1;
        let example_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        info!("");
        info!(
            "=== Running Example {}/{}: {} ===",
            i + 1,
            examples.len(),
            example_name
        );

        if interactive {
            info!("Press Enter to continue or Ctrl+C to exit...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
        }

        let example_start = std::time::Instant::now();

        match runner.run_example_file(&path.to_string_lossy()).await {
            Ok(_) => {
                let duration = example_start.elapsed();
                successful += 1;
                info!(
                    "✓ Example '{}' completed successfully (duration: {:?})",
                    example_name, duration
                );

                if verbose {
                    info!("Example details:");
                    info!("  Name: {}", example_name);
                    info!("  Path: {}", path.display());
                    info!("  Duration: {:?}", duration);
                    info!("  Status: COMPLETED");
                }
            }
            Err(e) => {
                let duration = example_start.elapsed();
                error!(
                    "✗ Example '{}' failed: {} (failed after {:?})",
                    example_name, e, duration
                );

                if verbose {
                    error!("Example details:");
                    error!("  Name: {}", example_name);
                    error!("  Path: {}", path.display());
                    error!("  Duration: {:?}", duration);
                    error!("  Status: FAILED");
                    error!("  Error: {}", e);
                }
            }
        }

        if interactive && i < examples.len() - 1 {
            info!("Waiting before next example...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    let total_duration = start_time.elapsed();

    info!("");
    info!("=== Steel Examples Summary ===");
    info!("Total examples: {}", total);
    info!("Successful: {} ✓", successful);
    info!("Failed: {} ✗", total - successful);
    info!(
        "Success rate: {:.1}%",
        (successful as f64 / total as f64) * 100.0
    );
    info!("Total execution time: {:?}", total_duration);

    if verbose && total > 0 {
        let avg_time = total_duration.as_secs_f64() / total as f64;
        info!("Average example time: {:.3}s", avg_time);
    }

    if successful == total {
        info!("🎉 All Steel examples completed successfully!");
    } else {
        warn!("⚠️  Some Steel examples failed. Check the output above for details.");
    }

    Ok(())
}

fn create_mock_hal(interactive: bool) -> impl PlatformHAL {
    use async_trait::async_trait;
    use chrono::Utc;
    use parking_lot::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    struct MockHAL {
        led_state: Arc<Mutex<LedState>>,
        sleep_called: Arc<AtomicBool>,
        interactive: bool,
    }

    impl MockHAL {
        fn new(interactive: bool) -> Self {
            Self {
                led_state: Arc::new(Mutex::new(LedState::Off)),
                sleep_called: Arc::new(AtomicBool::new(false)),
                interactive,
            }
        }
    }

    #[async_trait]
    impl PlatformHAL for MockHAL {
        async fn sleep(&self, duration: Duration) -> PlatformResult<()> {
            self.sleep_called.store(true, Ordering::SeqCst);

            // For examples, we'll actually sleep but cap it to reasonable durations
            let sleep_duration = if self.interactive {
                // In interactive mode, use actual durations for better demo experience
                if duration > Duration::from_secs(3) {
                    Duration::from_secs(1) // Cap very long sleeps
                } else {
                    duration
                }
            } else {
                // In non-interactive mode, speed up for faster execution
                if duration > Duration::from_secs(1) {
                    Duration::from_millis(100) // Cap long sleeps for demo purposes
                } else {
                    duration.min(Duration::from_millis(500))
                }
            };

            if sleep_duration > Duration::from_millis(50) {
                println!("💤 Sleeping for {:?}...", sleep_duration);
            }

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

    MockHAL::new(interactive)
}
