use aws_iot_core::steel_test_runner::TestResults;
use aws_iot_core::{
    DeviceInfo, LedState, MemoryInfo, PlatformHAL, PlatformResult, SteelTestRunner, UptimeInfo,
};
use clap::{Arg, Command};
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let matches = Command::new("steel_test")
        .version("1.0")
        .about("Steel Test Runner for AWS IoT Steel module")
        .arg(
            Arg::new("test-dir")
                .short('d')
                .long("test-dir")
                .value_name("DIR")
                .help("Directory containing Steel test files")
                .default_value("aws-iot-core/tests/steel"),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Run a specific Steel test file"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Enable verbose output"),
        )
        .arg(
            Arg::new("continue-on-error")
                .short('c')
                .long("continue-on-error")
                .action(clap::ArgAction::SetTrue)
                .help("Continue running tests even if some fail"),
        )
        .get_matches();

    let verbose = matches.get_flag("verbose");
    let continue_on_error = matches.get_flag("continue-on-error");

    info!("=== Steel Test Runner Starting ===");

    // Create mock HAL for testing
    info!("Initializing test HAL...");
    let hal = Arc::new(create_mock_hal());

    info!("Creating Steel test runner...");
    let runner = SteelTestRunner::new(hal)?;

    if let Some(test_file) = matches.get_one::<String>("file") {
        // Run a specific test file
        info!("Running single test file: {}", test_file);

        let start_time = std::time::Instant::now();
        let path = Path::new(test_file);

        match runner.run_test_file(&path.to_string_lossy()).await {
            Ok(()) => {
                let duration = start_time.elapsed();
                info!(
                    "✓ Test file passed: {} (completed in {:?})",
                    test_file, duration
                );

                if verbose {
                    info!("Test execution details:");
                    info!("  File: {}", test_file);
                    info!("  Duration: {:?}", duration);
                    info!("  Status: PASSED");
                }

                std::process::exit(0);
            }
            Err(e) => {
                let duration = start_time.elapsed();
                error!(
                    "✗ Test file failed: {} - {} (failed after {:?})",
                    test_file, e, duration
                );

                if verbose {
                    error!("Test execution details:");
                    error!("  File: {}", test_file);
                    error!("  Duration: {:?}", duration);
                    error!("  Status: FAILED");
                    error!("  Error: {}", e);
                }

                std::process::exit(1);
            }
        }
    } else {
        // Run all tests in directory
        let test_dir = matches.get_one::<String>("test-dir").unwrap();
        info!("Running all Steel tests in directory: {}", test_dir);

        let start_time = std::time::Instant::now();
        match runner.run_test_directory(test_dir).await {
            Ok(results) => {
                let total_duration = start_time.elapsed();

                // Print detailed results
                print_detailed_results(&results, total_duration, verbose);

                if results.failed_count() > 0 {
                    if continue_on_error {
                        warn!("Some tests failed, but continuing due to --continue-on-error flag");
                        std::process::exit(0);
                    } else {
                        error!("Some tests failed!");
                        std::process::exit(1);
                    }
                } else {
                    info!("🎉 All tests passed!");
                    std::process::exit(0);
                }
            }
            Err(e) => {
                let total_duration = start_time.elapsed();
                error!(
                    "Failed to run test directory: {} (failed after {:?})",
                    e, total_duration
                );
                std::process::exit(1);
            }
        }
    }
}

fn print_detailed_results(
    results: &TestResults,
    total_duration: std::time::Duration,
    verbose: bool,
) {
    info!("");
    info!("=== Steel Test Results ===");
    info!("Total execution time: {:?}", total_duration);
    info!("Tests run: {}", results.total());
    info!("Passed: {} ✓", results.passed_count());
    info!("Failed: {} ✗", results.failed_count());
    info!("Success rate: {:.1}%", results.success_rate());

    if verbose && !results.passed.is_empty() {
        info!("");
        info!("Passed tests:");
        for test in &results.passed {
            info!("  ✓ {}", test);
        }
    }

    if !results.failed.is_empty() {
        info!("");
        warn!("Failed tests:");
        for (test, error) in &results.failed {
            error!("  ✗ {} - {}", test, error);
        }
    }

    if verbose {
        info!("");
        info!("Performance metrics:");
        if results.total() > 0 {
            let avg_time = total_duration.as_secs_f64() / results.total() as f64;
            info!("  Average test time: {:.3}s", avg_time);
            info!(
                "  Tests per second: {:.2}",
                results.total() as f64 / total_duration.as_secs_f64()
            );
        }
    }

    info!("");
    if results.failed_count() == 0 {
        info!("🎉 All Steel tests completed successfully!");
    } else {
        warn!("⚠️  Some Steel tests failed. Check the output above for details.");
    }
    info!("=== End Steel Test Results ===");
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
