#[cfg(target_os = "macos")]
use aws_iot_core::{LedState, PlatformHAL};
#[cfg(target_os = "macos")]
use aws_iot_platform_macos::{MacOSHAL, MacOSSystemMonitor};
#[cfg(target_os = "macos")]
use std::time::Duration;

#[cfg(target_os = "macos")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for colored output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 AWS IoT Steel - macOS System Demonstration");
    println!("==============================================\n");

    // Create and initialize HAL
    let mut hal = MacOSHAL::new();
    hal.initialize().await?;

    // Demonstrate device information
    println!("📱 Device Information:");
    let device_info = hal.get_device_info().await?;
    println!("   Device ID: {}", device_info.device_id);
    println!("   Platform: {}", device_info.platform);
    println!("   Version: {}", device_info.version);
    println!("   Firmware: {}", device_info.firmware_version);
    if let Some(hw) = &device_info.hardware_revision {
        println!("   Hardware: {}", hw);
    }
    if let Some(serial) = &device_info.serial_number {
        println!("   Serial: {}", serial);
    }
    println!();

    // Demonstrate memory monitoring
    println!("💾 Memory Information:");
    let memory_info = hal.get_memory_info().await?;
    println!(
        "   Total: {:.2} GB",
        memory_info.total_bytes as f64 / 1_073_741_824.0
    );
    println!(
        "   Free: {:.2} GB",
        memory_info.free_bytes as f64 / 1_073_741_824.0
    );
    println!(
        "   Used: {:.2} GB",
        memory_info.used_bytes as f64 / 1_073_741_824.0
    );
    println!("   Usage: {:.1}%", memory_info.usage_percentage());
    println!(
        "   Largest Free Block: {:.2} GB",
        memory_info.largest_free_block as f64 / 1_073_741_824.0
    );
    println!();

    // Demonstrate uptime monitoring
    println!("⏰ Uptime Information:");
    let uptime_info = hal.get_uptime().await?;
    println!("   Uptime: {:?}", uptime_info.uptime);
    println!(
        "   Boot Time: {}",
        uptime_info.boot_time.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!();

    // Demonstrate enhanced system monitoring
    let system_monitor = MacOSSystemMonitor::new();

    println!("🖥️ CPU Information:");
    let cpu_info = system_monitor.get_cpu_info().await?;
    println!("   Model: {}", cpu_info.model);
    println!("   Cores: {}", cpu_info.cores);
    println!("   Frequency: {} MHz", cpu_info.frequency_mhz);
    println!();

    println!("💿 Disk Information:");
    let disk_info = system_monitor.get_disk_info().await?;
    println!(
        "   Total: {:.2} GB",
        disk_info.total_bytes as f64 / 1_000_000_000.0
    );
    println!(
        "   Free: {:.2} GB",
        disk_info.free_bytes as f64 / 1_000_000_000.0
    );
    println!(
        "   Used: {:.2} GB",
        disk_info.used_bytes as f64 / 1_000_000_000.0
    );
    println!("   Usage: {:.1}%", disk_info.usage_percentage());
    println!();

    // Demonstrate LED simulation with colored output
    println!("💡 LED Control Demonstration:");
    println!("   Testing LED state changes...");

    hal.set_led(LedState::On).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    hal.set_led(LedState::Off).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    hal.set_led(LedState::On).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    hal.set_led(LedState::Off).await?;
    println!();

    // Demonstrate sleep functionality
    println!("😴 Sleep Demonstration:");
    println!("   Testing sleep functionality...");
    hal.sleep(Duration::from_millis(1000)).await?;
    println!();

    // Demonstrate secure storage
    println!("🔐 Secure Storage Demonstration:");
    let test_key = "demo_key";
    let test_data = b"Hello, secure world! This is a test of the secure storage system.";

    println!("   Storing secure data...");
    hal.store_secure_data(test_key, test_data).await?;

    println!("   Loading secure data...");
    if let Some(loaded_data) = hal.load_secure_data(test_key).await? {
        println!(
            "   ✅ Data loaded successfully: {} bytes",
            loaded_data.len()
        );
        println!("   Content: {}", String::from_utf8_lossy(&loaded_data));
    }

    println!("   Listing secure keys...");
    let keys = hal.list_secure_keys().await?;
    println!("   Found {} keys: {:?}", keys.len(), keys);

    println!("   Cleaning up...");
    let deleted = hal.delete_secure_data(test_key).await?;
    println!("   Data deleted: {}", deleted);
    println!();

    // Shutdown HAL
    println!("🛑 Shutting down HAL...");
    hal.shutdown().await?;

    println!("✅ Demonstration completed successfully!");

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("❌ This example is only available on macOS");
    eprintln!("💡 Try running on macOS or use the basic_hal_demo example instead");
    std::process::exit(1);
}
