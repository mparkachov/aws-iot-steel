pub mod error;
pub mod hal;
pub mod iot_client;
pub mod logging;
pub mod ota_manager;
pub mod program_delivery;
pub mod rust_api;
pub mod security;
pub mod shadow_manager;
pub mod steel_runtime;
pub mod embedded_steel_runtime;
pub mod steel_program_validator;
pub mod steel_test_runner;
pub mod types;

pub use error::*;
pub use hal::*;
pub use iot_client::{IoTClient, IoTClientTrait, MockIoTClient, MessageCallback};
pub use logging::*;
pub use ota_manager::{OTAManager, OTAManagerTrait, MockOTAManager, FirmwareUpdateRequest, FirmwareUpdateStatus, FirmwareUpdateResult, FirmwareValidationResult, PreSignedUrlRequest, DownloadProgress};
pub use program_delivery::*;
pub use rust_api::{RustAPI, HardwareState as RustHardwareState, SleepStatus as RustSleepStatus};
pub use security::*;
pub use shadow_manager::{ShadowManager, ShadowUpdate as ShadowManagerUpdate};
pub use steel_runtime::{SteelRuntime, SteelRuntimeImpl, SteelRuntimeAPI, ProgramMetadata as SteelProgramMetadata};
pub use embedded_steel_runtime::{
    EmbeddedSteelRuntime, EmbeddedSteelRuntimeAPI, EmbeddedProgramHandle, 
    EmbeddedProgramMetadata, MemoryMonitor, MemoryUsageStats
};
pub use steel_program_validator::{SteelProgramValidator, ValidationResult};
pub use steel_test_runner::{SteelTestRunner, TestResults as SteelTestResults};
pub use types::*;