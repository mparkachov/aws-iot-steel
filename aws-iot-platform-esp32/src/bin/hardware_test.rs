/// ESP32-C3 Hardware Test Runner
/// This binary runs comprehensive hardware validation tests on the ESP32-C3-DevKit-RUST-1
use log::{info, warn};

#[cfg(target_arch = "riscv32")]
use aws_iot_platform_esp32::run_hardware_validation;

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async_main());
}

async fn async_main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("ESP32-C3 Hardware Test Runner starting...");

    #[cfg(target_arch = "riscv32")]
    {
        info!("Target: ESP32-C3-DevKit-RUST-1 (RISC-V)");
        info!("Testing: GPIO, Sleep, Memory, Secure Storage, Power Management");

        match run_hardware_validation().await {
            Ok(()) => {
                info!("✅ All hardware tests passed successfully!");
                info!("ESP32-C3-DevKit-RUST-1 hardware validation complete");
            }
            Err(e) => {
                error!("❌ Hardware test failed: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(not(target_arch = "riscv32"))]
    {
        warn!("Target: {} (Non-ESP32)", std::env::consts::ARCH);
        warn!("Hardware tests are only available on ESP32-C3 target (riscv32imc-esp-espidf)");
        warn!("This is a stub binary for cross-compilation compatibility");
        warn!("To run actual hardware tests, compile and flash to ESP32-C3-DevKit-RUST-1:");
        warn!("  cargo build --bin hardware_test --target riscv32imc-esp-espidf");
        warn!("  espflash flash --monitor target/riscv32imc-esp-espidf/debug/hardware_test");

        info!("Stub execution completed - no actual hardware testing performed");
    }
}
