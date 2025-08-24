use serde::{Deserialize, Serialize};
use std::time::Duration;
use chrono::{DateTime, Utc};

/// LED state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedState {
    On,
    Off,
}

impl From<bool> for LedState {
    fn from(value: bool) -> Self {
        if value { LedState::On } else { LedState::Off }
    }
}

impl From<LedState> for bool {
    fn from(state: LedState) -> Self {
        matches!(state, LedState::On)
    }
}

/// Device information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub platform: String,
    pub version: String,
    pub firmware_version: String,
    pub hardware_revision: Option<String>,
    pub serial_number: Option<String>,
}

/// Memory information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub largest_free_block: u64,
}

impl MemoryInfo {
    pub fn usage_percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.used_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }
}

/// System uptime information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeInfo {
    pub uptime: Duration,
    pub boot_time: DateTime<Utc>,
}

/// Secure storage key-value pair
#[derive(Debug, Clone)]
pub struct SecureData {
    pub key: String,
    pub data: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Configuration for logging levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Trace => write!(f, "TRACE"),
        }
    }
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

/// IoT connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

/// Steel value types - our own Send + Sync version
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SteelValue {
    NumV(f64),
    StringV(String),
    BoolV(bool),
    ListV(Vec<SteelValue>),
    Null,
}

/// IoT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoTConfig {
    pub device_id: String,
    pub thing_name: String,
    pub endpoint: String,
    pub region: String,
    pub certificate_path: Option<String>,
    pub private_key_path: Option<String>,
    pub ca_cert_path: Option<String>,
    pub client_id: Option<String>,
    pub keep_alive_secs: u16,
    pub clean_session: bool,
}

impl Default for IoTConfig {
    fn default() -> Self {
        Self {
            device_id: "default-device".to_string(),
            thing_name: "default-thing".to_string(),
            endpoint: "".to_string(),
            region: "us-east-1".to_string(),
            certificate_path: None,
            private_key_path: None,
            ca_cert_path: None,
            client_id: None,
            keep_alive_secs: 60,
            clean_session: true,
        }
    }
}

/// MQTT message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: u8,
    pub retain: bool,
    pub timestamp: DateTime<Utc>,
}

impl MqttMessage {
    pub fn new(topic: String, payload: Vec<u8>) -> Self {
        Self {
            topic,
            payload,
            qos: 0,
            retain: false,
            timestamp: Utc::now(),
        }
    }
    
    pub fn payload_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.payload.clone())
    }
}

/// Device shadow state representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    pub runtime_status: RuntimeStatus,
    pub hardware_state: HardwareState,
    pub system_info: SystemInfo,
    pub timestamp: DateTime<Utc>,
}

/// Runtime execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeStatus {
    Idle,
    ExecutingProgram { 
        program_id: String, 
        started_at: DateTime<Utc> 
    },
    Error { 
        message: String, 
        timestamp: DateTime<Utc> 
    },
}

/// Hardware state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareState {
    pub led_status: bool,
    pub sleep_status: SleepStatus,
    pub memory_usage: MemoryInfo,
}

/// Sleep status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SleepStatus {
    Awake,
    Sleeping { wake_time: DateTime<Utc> },
}

/// System information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub firmware_version: String,
    pub platform: String,
    pub uptime_seconds: u64,
    pub steel_runtime_version: String,
}

/// Program message for Steel code delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramMessage {
    pub program_id: String,
    pub program_name: String,
    pub steel_code: String,
    pub version: String,
    pub checksum: String,
    pub auto_start: bool,
    pub metadata: Option<ProgramMetadata>,
}

/// Program metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramMetadata {
    pub description: Option<String>,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
    pub memory_requirement: Option<u64>,
    pub execution_timeout: Option<u64>,
}

/// Program execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramResult {
    pub program_id: String,
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// MQTT subscription information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MQTTSubscription {
    pub topic: String,
    pub qos: u8,
}

/// Shadow update message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowUpdate {
    pub state: DeviceState,
    pub version: u64,
    pub timestamp: DateTime<Utc>,
}

/// Program handle for managing loaded Steel programs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProgramHandle {
    pub id: String,
}

impl ProgramHandle {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

/// Program information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramInfo {
    pub handle: ProgramHandle,
    pub name: Option<String>,
    pub loaded_at: DateTime<Utc>,
    pub execution_count: u64,
}

/// Execution context for Steel programs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            device_id: "default-device".to_string(),
            timestamp: Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub total_programs_loaded: u64,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: f64,
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self {
            total_programs_loaded: 0,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time_ms: 0.0,
        }
    }
}

/// Test results structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub passed: Vec<String>,
    pub failed: Vec<(String, String)>,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            passed: Vec::new(),
            failed: Vec::new(),
        }
    }
    
    pub fn total(&self) -> usize {
        self.passed.len() + self.failed.len()
    }
    
    pub fn passed_count(&self) -> usize {
        self.passed.len()
    }
    
    pub fn failed_count(&self) -> usize {
        self.failed.len()
    }
    
    pub fn success_rate(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            (self.passed_count() as f64 / total as f64) * 100.0
        }
    }
}

impl Default for TestResults {
    fn default() -> Self {
        Self::new()
    }
}

// Add Default implementations for missing types
impl Default for DeviceState {
    fn default() -> Self {
        Self {
            runtime_status: RuntimeStatus::Idle,
            hardware_state: HardwareState::default(),
            system_info: SystemInfo::default(),
            timestamp: Utc::now(),
        }
    }
}

impl Default for HardwareState {
    fn default() -> Self {
        Self {
            led_status: false,
            sleep_status: SleepStatus::Awake,
            memory_usage: MemoryInfo {
                total_bytes: 0,
                free_bytes: 0,
                used_bytes: 0,
                largest_free_block: 0,
            },
        }
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            firmware_version: "1.0.0".to_string(),
            platform: "unknown".to_string(),
            uptime_seconds: 0,
            steel_runtime_version: "0.7.0".to_string(),
        }
    }
}