use aws_iot_core::{initialize_dev_logging, LedState, PlatformHAL};
use aws_iot_platform_linux::LinuxHAL;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    initialize_dev_logging()?;

    // Create and initialize HAL
    let mut hal = LinuxHAL::new();
    hal.initialize().await?;

    println!("🚀 AWS IoT Steel Module - Linux CI/CD Demo");

    // Get device information
    let device_info = hal.get_device_info().await?;
    println!(
        "📱 Device: {} ({})",
        device_info.device_id, device_info.platform
    );
    println!("📦 Version: {}", device_info.version);

    // Get memory information
    let memory_info = hal.get_memory_info().await?;
    println!(
        "💾 Memory: {:.1}% used ({} / {} bytes)",
        memory_info.usage_percentage(),
        memory_info.used_bytes,
        memory_info.total_bytes
    );

    // Get uptime
    let uptime_info = hal.get_uptime().await?;
    println!("⏱️ Uptime: {:?}", uptime_info.uptime);

    // Demonstrate LED control
    println!("\n💡 LED Control Demo:");
    hal.set_led(LedState::On).await?;
    println!("LED State: {:?}", hal.get_led_state().await?);

    hal.set_led(LedState::Off).await?;
    println!("LED State: {:?}", hal.get_led_state().await?);

    // Demonstrate sleep
    println!("\n💤 Sleep Demo:");
    println!("Sleeping for 1 second...");
    hal.sleep(Duration::from_secs(1)).await?;
    println!("Wake up!");

    // Demonstrate secure storage
    println!("\n🔐 Secure Storage Demo:");
    let key = "ci_demo_key";
    let data = b"Hello from Linux CI/CD!";

    hal.store_secure_data(key, data).await?;
    println!("Stored secure data");

    if let Some(loaded_data) = hal.load_secure_data(key).await? {
        println!(
            "Loaded secure data: {}",
            String::from_utf8_lossy(&loaded_data)
        );
    }

    let keys = hal.list_secure_keys().await?;
    println!("Secure keys: {:?}", keys);

    hal.delete_secure_data(key).await?;
    println!("Deleted secure data");

    // Shutdown HAL
    hal.shutdown().await?;
    println!("\n✅ Linux CI/CD demo completed successfully!");

    Ok(())
}
