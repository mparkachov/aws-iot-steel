pub mod hal;

// Conditional compilation: use actual ESP32 implementation on target, stub otherwise
#[cfg(target_arch = "riscv32")]
mod esp32_hardware;
#[cfg(target_arch = "riscv32")]
pub use esp32_hardware::ESP32HAL;

#[cfg(not(target_arch = "riscv32"))]
mod esp32_stub;
#[cfg(not(target_arch = "riscv32"))]
pub use esp32_stub::ESP32HAL;

// Hardware tests are only available on ESP32 target
#[cfg(target_arch = "riscv32")]
pub mod hardware_tests;
#[cfg(target_arch = "riscv32")]
pub use hardware_tests::{ESP32HardwareTests, run_hardware_validation};