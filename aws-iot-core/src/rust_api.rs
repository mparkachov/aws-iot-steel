use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Timelike};
use parking_lot::RwLock;
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    PlatformHAL, SystemResult, SystemError, PlatformError,
    LedState, DeviceInfo, MemoryInfo, LogLevel
};

/// Rust API layer that provides hardware and system APIs to Steel programs
#[derive(Clone)]
pub struct RustAPI {
    hal: Arc<dyn PlatformHAL>,
    hardware_state: Arc<RwLock<HardwareState>>,
    sensor_config: Arc<RwLock<SensorConfig>>,
    device_config: Arc<RwLock<DeviceConfig>>,
    timer_manager: Arc<TimerManager>,
}

/// Current hardware state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareState {
    pub led_state: LedState,
    pub sleep_status: SleepStatus,
    pub last_sensor_reading: Option<SensorData>,
    pub last_updated: DateTime<Utc>,
}

/// Sleep status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SleepStatus {
    Awake,
    Sleeping { 
        started_at: DateTime<Utc>,
        duration: Duration,
        wake_time: DateTime<Utc>,
    },
}

/// Sensor data with configurable simulation values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    pub temperature: f64,
    pub humidity: f64,
    pub pressure: f64,
    pub battery_level: f64,
    pub light_level: f64,
    pub timestamp: DateTime<Utc>,
}

/// Configuration for sensor data simulation
#[derive(Debug, Clone)]
pub struct SensorConfig {
    pub temperature_range: (f64, f64),
    pub humidity_range: (f64, f64),
    pub pressure_range: (f64, f64),
    pub battery_drain_rate: f64, // percentage per hour
    pub light_variation: f64,
    pub update_interval: Duration,
}

/// Device configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub device_id: String,
    pub log_level: LogLevel,
    pub sensor_update_interval: u64, // seconds
    pub max_storage_keys: usize,
    pub enable_debug_mode: bool,
    pub heartbeat_interval: u64, // seconds
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            device_id: "unknown".to_string(),
            log_level: LogLevel::Info,
            sensor_update_interval: 30,
            max_storage_keys: 1000,
            enable_debug_mode: false,
            heartbeat_interval: 60,
        }
    }
}

/// Handle for a timer
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimerHandle(Uuid);

impl Default for TimerHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerHandle {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn id(&self) -> &Uuid {
        &self.0
    }
}

/// Handle for a scheduled task
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskHandle(Uuid);

impl Default for TaskHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskHandle {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn id(&self) -> &Uuid {
        &self.0
    }
}

/// Timer information
#[derive(Debug, Clone)]
pub struct TimerInfo {
    pub handle: TimerHandle,
    pub name: String,
    pub duration: Duration,
    pub created_at: DateTime<Utc>,
    pub fire_time: DateTime<Utc>,
    pub is_active: bool,
}

/// Scheduled task information
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub handle: TaskHandle,
    pub name: String,
    pub cron_expression: String,
    pub created_at: DateTime<Utc>,
    pub next_run: Option<DateTime<Utc>>,
    pub last_run: Option<DateTime<Utc>>,
    pub run_count: u64,
    pub is_active: bool,
}

/// Steel callback function representation
#[derive(Debug, Clone)]
pub struct SteelCallback {
    pub name: String,
    pub code: String,
}

/// Event data for callbacks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

/// Timer event message
#[derive(Debug, Clone)]
pub enum TimerEvent {
    TimerFired {
        handle: TimerHandle,
        name: String,
        callback: SteelCallback,
    },
    TaskScheduled {
        handle: TaskHandle,
        name: String,
        callback: SteelCallback,
    },
    CancelTimer {
        handle: TimerHandle,
    },
    CancelTask {
        handle: TaskHandle,
    },
}

/// Timer and scheduling manager
pub struct TimerManager {
    timers: Arc<RwLock<HashMap<TimerHandle, TimerInfo>>>,
    tasks: Arc<RwLock<HashMap<TaskHandle, TaskInfo>>>,
    event_sender: mpsc::UnboundedSender<TimerEvent>,
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<TimerEvent>>>>,
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Self {
            timers: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            event_sender: sender,
            event_receiver: Arc::new(RwLock::new(Some(receiver))),
        }
    }
    
    /// Set a timer with a Steel callback
    pub fn set_timer(&self, name: String, duration: Duration, callback: SteelCallback) -> SystemResult<TimerHandle> {
        let handle = TimerHandle::new();
        let now = Utc::now();
        let fire_time = now + chrono::Duration::from_std(duration).unwrap();
        
        let timer_info = TimerInfo {
            handle: handle.clone(),
            name: name.clone(),
            duration,
            created_at: now,
            fire_time,
            is_active: true,
        };
        
        // Store timer info
        self.timers.write().insert(handle.clone(), timer_info);
        
        // Schedule the timer
        let event_sender = self.event_sender.clone();
        let timer_handle = handle.clone();
        let timer_name = name.clone();
        let timer_callback = callback.clone();
        
        tokio::spawn(async move {
            tokio::time::sleep(duration).await;
            
            let _ = event_sender.send(TimerEvent::TimerFired {
                handle: timer_handle,
                name: timer_name,
                callback: timer_callback,
            });
        });
        
        Ok(handle)
    }
    
    /// Cancel a timer
    pub fn cancel_timer(&self, handle: TimerHandle) -> SystemResult<bool> {
        let mut timers = self.timers.write();
        if let Some(timer) = timers.get_mut(&handle) {
            timer.is_active = false;
            
            // Send cancel event
            let _ = self.event_sender.send(TimerEvent::CancelTimer { handle });
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Schedule a recurring task (simplified cron-style)
    pub fn schedule_task(&self, name: String, cron_expr: String, callback: SteelCallback) -> SystemResult<TaskHandle> {
        // For this implementation, we'll support simple interval-based scheduling
        // Parse simple expressions like "*/30 * * * *" (every 30 seconds)
        let interval = self.parse_simple_cron(&cron_expr)?;
        
        let handle = TaskHandle::new();
        let now = Utc::now();
        let next_run = now + chrono::Duration::from_std(interval).unwrap();
        
        let task_info = TaskInfo {
            handle: handle.clone(),
            name: name.clone(),
            cron_expression: cron_expr,
            created_at: now,
            next_run: Some(next_run),
            last_run: None,
            run_count: 0,
            is_active: true,
        };
        
        // Store task info
        self.tasks.write().insert(handle.clone(), task_info);
        
        // Schedule the recurring task
        let event_sender = self.event_sender.clone();
        let task_handle = handle.clone();
        let task_name = name.clone();
        let task_callback = callback.clone();
        let tasks_ref = Arc::clone(&self.tasks);
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            interval_timer.tick().await; // Skip first immediate tick
            
            loop {
                interval_timer.tick().await;
                
                // Check if task is still active
                let is_active = {
                    let tasks = tasks_ref.read();
                    tasks.get(&task_handle).map(|t| t.is_active).unwrap_or(false)
                };
                
                if !is_active {
                    break;
                }
                
                // Update task info
                {
                    let mut tasks = tasks_ref.write();
                    if let Some(task) = tasks.get_mut(&task_handle) {
                        task.last_run = Some(Utc::now());
                        task.run_count += 1;
                        task.next_run = Some(Utc::now() + chrono::Duration::from_std(interval).unwrap());
                    }
                }
                
                // Send task event
                let _ = event_sender.send(TimerEvent::TaskScheduled {
                    handle: task_handle.clone(),
                    name: task_name.clone(),
                    callback: task_callback.clone(),
                });
            }
        });
        
        Ok(handle)
    }
    
    /// Cancel a scheduled task
    pub fn cancel_task(&self, handle: TaskHandle) -> SystemResult<bool> {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(&handle) {
            task.is_active = false;
            
            // Send cancel event
            let _ = self.event_sender.send(TimerEvent::CancelTask { handle });
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// List active timers
    pub fn list_timers(&self) -> Vec<TimerInfo> {
        self.timers.read().values()
            .filter(|t| t.is_active)
            .cloned()
            .collect()
    }
    
    /// List active tasks
    pub fn list_tasks(&self) -> Vec<TaskInfo> {
        self.tasks.read().values()
            .filter(|t| t.is_active)
            .cloned()
            .collect()
    }
    
    /// Get timer info
    pub fn get_timer_info(&self, handle: &TimerHandle) -> Option<TimerInfo> {
        self.timers.read().get(handle).cloned()
    }
    
    /// Get task info
    pub fn get_task_info(&self, handle: &TaskHandle) -> Option<TaskInfo> {
        self.tasks.read().get(handle).cloned()
    }
    
    /// Take the event receiver (can only be called once)
    pub fn take_event_receiver(&self) -> Option<mpsc::UnboundedReceiver<TimerEvent>> {
        self.event_receiver.write().take()
    }
    
    /// Parse simple cron expressions (simplified implementation)
    fn parse_simple_cron(&self, cron_expr: &str) -> SystemResult<Duration> {
        // Support simple patterns like:
        // "*/30 * * * *" - every 30 seconds
        // "0 */5 * * *" - every 5 minutes
        // "0 0 */2 * *" - every 2 hours
        
        let parts: Vec<&str> = cron_expr.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(SystemError::Configuration(
                "Cron expression must have 5 parts: second minute hour day month".to_string()
            ));
        }
        
        // Simple parsing for */N patterns
        if parts[0].starts_with("*/") {
            // Every N seconds
            let seconds: u64 = parts[0][2..].parse()
                .map_err(|_| SystemError::Configuration("Invalid seconds in cron expression".to_string()))?;
            return Ok(Duration::from_secs(seconds));
        }
        
        if parts[1].starts_with("*/") && parts[0] == "0" {
            // Every N minutes
            let minutes: u64 = parts[1][2..].parse()
                .map_err(|_| SystemError::Configuration("Invalid minutes in cron expression".to_string()))?;
            return Ok(Duration::from_secs(minutes * 60));
        }
        
        if parts[2].starts_with("*/") && parts[0] == "0" && parts[1] == "0" {
            // Every N hours
            let hours: u64 = parts[2][2..].parse()
                .map_err(|_| SystemError::Configuration("Invalid hours in cron expression".to_string()))?;
            return Ok(Duration::from_secs(hours * 3600));
        }
        
        // Default to 60 seconds for unsupported patterns
        Err(SystemError::Configuration(
            "Unsupported cron expression. Use patterns like '*/30 * * * *' for every 30 seconds".to_string()
        ))
    }
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            temperature_range: (18.0, 28.0),
            humidity_range: (40.0, 80.0),
            pressure_range: (1000.0, 1020.0),
            battery_drain_rate: 0.5,
            light_variation: 0.2,
            update_interval: Duration::from_secs(30),
        }
    }
}

impl Default for HardwareState {
    fn default() -> Self {
        Self {
            led_state: LedState::Off,
            sleep_status: SleepStatus::Awake,
            last_sensor_reading: None,
            last_updated: Utc::now(),
        }
    }
}

impl RustAPI {
    /// Create a new RustAPI instance
    pub fn new(hal: Arc<dyn PlatformHAL>) -> Self {
        Self {
            hal,
            hardware_state: Arc::new(RwLock::new(HardwareState::default())),
            sensor_config: Arc::new(RwLock::new(SensorConfig::default())),
            device_config: Arc::new(RwLock::new(DeviceConfig::default())),
            timer_manager: Arc::new(TimerManager::new()),
        }
    }

    // ========== Hardware Control APIs ==========

    /// Sleep for the specified duration with parameter validation
    /// 
    /// # Arguments
    /// * `duration_secs` - Duration to sleep in seconds (must be positive and <= 3600)
    /// 
    /// # Returns
    /// * `Ok(())` if sleep completed successfully
    /// * `Err(SystemError)` if duration is invalid or sleep operation failed
    pub async fn sleep(&self, duration_secs: f64) -> SystemResult<()> {
        // Validate duration parameter
        if duration_secs <= 0.0 {
            return Err(SystemError::Platform(PlatformError::Sleep(
                "Duration must be positive".to_string()
            )));
        }
        
        if duration_secs > 3600.0 {
            return Err(SystemError::Platform(PlatformError::Sleep(
                "Duration cannot exceed 1 hour (3600 seconds)".to_string()
            )));
        }

        let duration = Duration::from_secs_f64(duration_secs);
        let start_time = Utc::now();
        let wake_time = start_time + chrono::Duration::from_std(duration).unwrap();

        // Update hardware state to sleeping
        {
            let mut state = self.hardware_state.write();
            state.sleep_status = SleepStatus::Sleeping {
                started_at: start_time,
                duration,
                wake_time,
            };
            state.last_updated = start_time;
        }

        // Perform the actual sleep operation
        let result = self.hal.sleep(duration).await;

        // Update hardware state back to awake
        {
            let mut state = self.hardware_state.write();
            state.sleep_status = SleepStatus::Awake;
            state.last_updated = Utc::now();
        }

        result.map_err(SystemError::from)
    }

    /// Set LED state with state management and status reporting
    /// 
    /// # Arguments
    /// * `state` - Desired LED state (true for on, false for off)
    /// 
    /// # Returns
    /// * `Ok(())` if LED state was set successfully
    /// * `Err(SystemError)` if LED operation failed
    pub async fn set_led(&self, state: bool) -> SystemResult<()> {
        let led_state = LedState::from(state);
        
        // Set the LED state via HAL
        let result = self.hal.set_led(led_state).await;
        
        if result.is_ok() {
            // Update hardware state tracking
            let mut hw_state = self.hardware_state.write();
            hw_state.led_state = led_state;
            hw_state.last_updated = Utc::now();
        }
        
        result.map_err(SystemError::from)
    }

    /// Get current LED state
    /// 
    /// # Returns
    /// * `Ok(bool)` with current LED state (true for on, false for off)
    /// * `Err(SystemError)` if LED state cannot be determined
    pub async fn get_led_state(&self) -> SystemResult<bool> {
        let state = self.hal.get_led_state().await?;
        Ok(state.into())
    }

    /// Turn LED on (convenience method)
    /// 
    /// # Returns
    /// * `Ok(())` if LED was successfully turned on
    /// * `Err(SystemError)` if LED operation failed
    pub async fn led_on(&self) -> SystemResult<()> {
        self.set_led(true).await
    }

    /// Turn LED off (convenience method)
    /// 
    /// # Returns
    /// * `Ok(())` if LED was successfully turned off
    /// * `Err(SystemError)` if LED operation failed
    pub async fn led_off(&self) -> SystemResult<()> {
        self.set_led(false).await
    }

    /// Get LED state (convenience method, alias for get_led_state)
    /// 
    /// # Returns
    /// * `Ok(bool)` with current LED state (true for on, false for off)
    /// * `Err(SystemError)` if LED state cannot be determined
    pub async fn led_state(&self) -> SystemResult<bool> {
        self.get_led_state().await
    }

    /// Generate simulated sensor data with configurable values
    /// 
    /// # Returns
    /// * `Ok(SensorData)` with current sensor readings
    /// * `Err(SystemError)` if sensor data generation failed
    pub async fn get_sensor_data(&self) -> SystemResult<SensorData> {
        let config = self.sensor_config.read().clone();
        let now = Utc::now();
        
        // Generate realistic sensor data with some randomness
        let temperature = self.generate_sensor_value(
            config.temperature_range.0,
            config.temperature_range.1,
            0.5
        );
        
        let humidity = self.generate_sensor_value(
            config.humidity_range.0,
            config.humidity_range.1,
            2.0
        );
        
        let pressure = self.generate_sensor_value(
            config.pressure_range.0,
            config.pressure_range.1,
            1.0
        );
        
        // Simulate battery drain over time
        let battery_level = self.calculate_battery_level(&config, now);
        
        let light_level = self.generate_light_level(&config, now);
        
        let sensor_data = SensorData {
            temperature,
            humidity,
            pressure,
            battery_level,
            light_level,
            timestamp: now,
        };
        
        // Update hardware state with latest sensor reading
        {
            let mut state = self.hardware_state.write();
            state.last_sensor_reading = Some(sensor_data.clone());
            state.last_updated = now;
        }
        
        Ok(sensor_data)
    }

    /// Configure sensor simulation parameters
    /// 
    /// # Arguments
    /// * `config` - New sensor configuration
    pub fn configure_sensors(&self, config: SensorConfig) {
        *self.sensor_config.write() = config;
    }

    /// Get current hardware state for reporting
    /// 
    /// # Returns
    /// * `HardwareState` with current hardware status
    pub fn get_hardware_state(&self) -> HardwareState {
        self.hardware_state.read().clone()
    }

    // ========== System and Storage APIs ==========

    /// Store data securely with encryption at rest
    /// 
    /// # Arguments
    /// * `key` - The key to store the data under
    /// * `value` - The value to store (will be encrypted)
    /// 
    /// # Returns
    /// * `Ok(())` if data was stored successfully
    /// * `Err(SystemError)` if storage operation failed
    pub async fn store_data(&self, key: &str, value: &str) -> SystemResult<()> {
        // Validate key
        if key.is_empty() {
            return Err(SystemError::Platform(PlatformError::Storage(
                "Key cannot be empty".to_string()
            )));
        }
        
        if key.len() > 256 {
            return Err(SystemError::Platform(PlatformError::Storage(
                "Key too long (max 256 characters)".to_string()
            )));
        }
        
        // Validate value size (max 64KB)
        if value.len() > 65536 {
            return Err(SystemError::Platform(PlatformError::Storage(
                "Value too large (max 64KB)".to_string()
            )));
        }
        
        // Store the data via HAL (HAL handles encryption)
        self.hal.store_secure_data(key, value.as_bytes()).await
            .map_err(SystemError::from)
    }

    /// Load data from secure storage
    /// 
    /// # Arguments
    /// * `key` - The key to load data for
    /// 
    /// # Returns
    /// * `Ok(Some(String))` if data was found and loaded
    /// * `Ok(None)` if no data exists for the key
    /// * `Err(SystemError)` if storage operation failed
    pub async fn load_data(&self, key: &str) -> SystemResult<Option<String>> {
        if key.is_empty() {
            return Err(SystemError::Platform(PlatformError::Storage(
                "Key cannot be empty".to_string()
            )));
        }
        
        match self.hal.load_secure_data(key).await? {
            Some(data) => {
                let value = String::from_utf8(data)
                    .map_err(|e| SystemError::Platform(PlatformError::Storage(
                        format!("Invalid UTF-8 data: {}", e)
                    )))?;
                Ok(Some(value))
            }
            None => Ok(None)
        }
    }

    /// Delete data from secure storage
    /// 
    /// # Arguments
    /// * `key` - The key to delete data for
    /// 
    /// # Returns
    /// * `Ok(true)` if data was found and deleted
    /// * `Ok(false)` if no data existed for the key
    /// * `Err(SystemError)` if storage operation failed
    pub async fn delete_data(&self, key: &str) -> SystemResult<bool> {
        if key.is_empty() {
            return Err(SystemError::Platform(PlatformError::Storage(
                "Key cannot be empty".to_string()
            )));
        }
        
        self.hal.delete_secure_data(key).await
            .map_err(SystemError::from)
    }

    /// List all keys in secure storage
    /// 
    /// # Returns
    /// * `Ok(Vec<String>)` with all available keys
    /// * `Err(SystemError)` if storage operation failed
    pub async fn list_storage_keys(&self) -> SystemResult<Vec<String>> {
        self.hal.list_secure_keys().await
            .map_err(SystemError::from)
    }

    /// Get device information
    /// 
    /// # Returns
    /// * `Ok(DeviceInfo)` with device details
    /// * `Err(SystemError)` if device info cannot be retrieved
    pub async fn get_device_info(&self) -> SystemResult<DeviceInfo> {
        self.hal.get_device_info().await
            .map_err(SystemError::from)
    }

    /// Get memory usage information
    /// 
    /// # Returns
    /// * `Ok(MemoryInfo)` with memory statistics
    /// * `Err(SystemError)` if memory info cannot be retrieved
    pub async fn get_memory_info(&self) -> SystemResult<MemoryInfo> {
        self.hal.get_memory_info().await
            .map_err(SystemError::from)
    }

    /// Get system uptime
    /// 
    /// # Returns
    /// * `Ok(Duration)` with system uptime
    /// * `Err(SystemError)` if uptime cannot be determined
    pub async fn get_uptime(&self) -> SystemResult<Duration> {
        let uptime_info = self.hal.get_uptime().await?;
        Ok(uptime_info.uptime)
    }

    /// Log a message with structured output
    /// 
    /// # Arguments
    /// * `level` - Log level (error, warn, info, debug, trace)
    /// * `message` - The message to log
    /// * `context` - Optional context data as key-value pairs
    /// 
    /// # Returns
    /// * `Ok(())` always (logging should not fail the application)
    pub fn log_structured(&self, level: LogLevel, message: &str, context: Option<&[(&str, &str)]>) -> SystemResult<()> {
        let formatted_message = if let Some(ctx) = context {
            let mut fields = Vec::new();
            for (key, value) in ctx {
                fields.push(format!("{}={}", key, value));
            }
            let context_str = fields.join(" ");
            format!("RustAPI: {} [{}]", message, context_str)
        } else {
            format!("RustAPI: {}", message)
        };
        
        match level {
            LogLevel::Error => tracing::error!("{}", formatted_message),
            LogLevel::Warn => tracing::warn!("{}", formatted_message),
            LogLevel::Info => tracing::info!("{}", formatted_message),
            LogLevel::Debug => tracing::debug!("{}", formatted_message),
            LogLevel::Trace => tracing::trace!("{}", formatted_message),
        }
        
        Ok(())
    }

    /// Log a simple message
    /// 
    /// # Arguments
    /// * `level` - Log level (error, warn, info, debug, trace)
    /// * `message` - The message to log
    pub fn log(&self, level: LogLevel, message: &str) -> SystemResult<()> {
        self.log_structured(level, message, None)
    }

    // ========== Configuration Management APIs ==========

    /// Get current device configuration
    /// 
    /// # Returns
    /// * `DeviceConfig` with current configuration settings
    pub fn get_device_config(&self) -> DeviceConfig {
        self.device_config.read().clone()
    }

    /// Update device configuration
    /// 
    /// # Arguments
    /// * `config` - New device configuration
    /// 
    /// # Returns
    /// * `Ok(())` if configuration was updated successfully
    /// * `Err(SystemError)` if configuration is invalid
    pub fn update_device_config(&self, config: DeviceConfig) -> SystemResult<()> {
        // Validate configuration
        if config.device_id.is_empty() {
            return Err(SystemError::Configuration("Device ID cannot be empty".to_string()));
        }
        
        if config.sensor_update_interval == 0 {
            return Err(SystemError::Configuration("Sensor update interval must be positive".to_string()));
        }
        
        if config.heartbeat_interval == 0 {
            return Err(SystemError::Configuration("Heartbeat interval must be positive".to_string()));
        }
        
        if config.max_storage_keys == 0 {
            return Err(SystemError::Configuration("Max storage keys must be positive".to_string()));
        }
        
        *self.device_config.write() = config;
        Ok(())
    }

    /// Save device configuration to persistent storage
    /// 
    /// # Returns
    /// * `Ok(())` if configuration was saved successfully
    /// * `Err(SystemError)` if save operation failed
    pub async fn save_device_config(&self) -> SystemResult<()> {
        let config = self.device_config.read().clone();
        let config_json = serde_json::to_string(&config)
            .map_err(SystemError::Serialization)?;
        
        self.store_data("device_config", &config_json).await
    }

    /// Load device configuration from persistent storage
    /// 
    /// # Returns
    /// * `Ok(())` if configuration was loaded successfully
    /// * `Err(SystemError)` if load operation failed
    pub async fn load_device_config(&self) -> SystemResult<()> {
        match self.load_data("device_config").await? {
            Some(config_json) => {
                let config: DeviceConfig = serde_json::from_str(&config_json)
                    .map_err(SystemError::Serialization)?;
                
                self.update_device_config(config)?;
                Ok(())
            }
            None => {
                // No saved configuration, use defaults
                self.log(LogLevel::Info, "No saved configuration found, using defaults")?;
                Ok(())
            }
        }
    }

    /// Reset device configuration to defaults
    /// 
    /// # Returns
    /// * `Ok(())` always
    pub fn reset_device_config(&self) -> SystemResult<()> {
        *self.device_config.write() = DeviceConfig::default();
        Ok(())
    }

    // ========== Timer and Scheduling APIs ==========

    /// Set a timer with Steel callback support
    /// 
    /// # Arguments
    /// * `name` - Name for the timer (for identification)
    /// * `duration_secs` - Duration in seconds after which the timer fires
    /// * `callback_code` - Steel code to execute when timer fires
    /// 
    /// # Returns
    /// * `Ok(TimerHandle)` if timer was set successfully
    /// * `Err(SystemError)` if timer setup failed
    pub fn set_timer(&self, name: &str, duration_secs: f64, callback_code: &str) -> SystemResult<TimerHandle> {
        if duration_secs <= 0.0 {
            return Err(SystemError::Configuration("Timer duration must be positive".to_string()));
        }
        
        if duration_secs > 86400.0 {
            return Err(SystemError::Configuration("Timer duration cannot exceed 24 hours".to_string()));
        }
        
        if name.is_empty() {
            return Err(SystemError::Configuration("Timer name cannot be empty".to_string()));
        }
        
        if callback_code.trim().is_empty() {
            return Err(SystemError::Configuration("Callback code cannot be empty".to_string()));
        }
        
        let duration = Duration::from_secs_f64(duration_secs);
        let callback = SteelCallback {
            name: format!("timer_{}", name),
            code: callback_code.to_string(),
        };
        
        self.timer_manager.set_timer(name.to_string(), duration, callback)
    }

    /// Cancel a timer
    /// 
    /// # Arguments
    /// * `handle` - Handle of the timer to cancel
    /// 
    /// # Returns
    /// * `Ok(true)` if timer was found and cancelled
    /// * `Ok(false)` if timer was not found
    /// * `Err(SystemError)` if cancellation failed
    pub fn cancel_timer(&self, handle: TimerHandle) -> SystemResult<bool> {
        self.timer_manager.cancel_timer(handle)
    }

    /// Schedule a recurring task with cron-style expression
    /// 
    /// # Arguments
    /// * `name` - Name for the task (for identification)
    /// * `cron_expr` - Cron expression (simplified: "*/N * * * *" for every N seconds)
    /// * `callback_code` - Steel code to execute on each run
    /// 
    /// # Returns
    /// * `Ok(TaskHandle)` if task was scheduled successfully
    /// * `Err(SystemError)` if scheduling failed
    pub fn schedule_task(&self, name: &str, cron_expr: &str, callback_code: &str) -> SystemResult<TaskHandle> {
        if name.is_empty() {
            return Err(SystemError::Configuration("Task name cannot be empty".to_string()));
        }
        
        if cron_expr.trim().is_empty() {
            return Err(SystemError::Configuration("Cron expression cannot be empty".to_string()));
        }
        
        if callback_code.trim().is_empty() {
            return Err(SystemError::Configuration("Callback code cannot be empty".to_string()));
        }
        
        let callback = SteelCallback {
            name: format!("task_{}", name),
            code: callback_code.to_string(),
        };
        
        self.timer_manager.schedule_task(name.to_string(), cron_expr.to_string(), callback)
    }

    /// Cancel a scheduled task
    /// 
    /// # Arguments
    /// * `handle` - Handle of the task to cancel
    /// 
    /// # Returns
    /// * `Ok(true)` if task was found and cancelled
    /// * `Ok(false)` if task was not found
    /// * `Err(SystemError)` if cancellation failed
    pub fn cancel_task(&self, handle: TaskHandle) -> SystemResult<bool> {
        self.timer_manager.cancel_task(handle)
    }

    /// List all active timers
    /// 
    /// # Returns
    /// * `Vec<TimerInfo>` with information about all active timers
    pub fn list_timers(&self) -> Vec<TimerInfo> {
        self.timer_manager.list_timers()
    }

    /// List all active scheduled tasks
    /// 
    /// # Returns
    /// * `Vec<TaskInfo>` with information about all active tasks
    pub fn list_tasks(&self) -> Vec<TaskInfo> {
        self.timer_manager.list_tasks()
    }

    /// Get information about a specific timer
    /// 
    /// # Arguments
    /// * `handle` - Handle of the timer
    /// 
    /// # Returns
    /// * `Some(TimerInfo)` if timer exists
    /// * `None` if timer not found
    pub fn get_timer_info(&self, handle: &TimerHandle) -> Option<TimerInfo> {
        self.timer_manager.get_timer_info(handle)
    }

    /// Get information about a specific task
    /// 
    /// # Arguments
    /// * `handle` - Handle of the task
    /// 
    /// # Returns
    /// * `Some(TaskInfo)` if task exists
    /// * `None` if task not found
    pub fn get_task_info(&self, handle: &TaskHandle) -> Option<TaskInfo> {
        self.timer_manager.get_task_info(handle)
    }

    /// Get the timer event receiver for processing timer/task events
    /// This should be called once to set up event processing
    /// 
    /// # Returns
    /// * `Some(Receiver)` if receiver is available
    /// * `None` if receiver has already been taken
    pub fn take_timer_event_receiver(&self) -> Option<mpsc::UnboundedReceiver<TimerEvent>> {
        self.timer_manager.take_event_receiver()
    }

    /// Create an event system for Steel callbacks
    /// 
    /// # Arguments
    /// * `event_type` - Type of event to emit
    /// * `data` - Event data as JSON value
    /// 
    /// # Returns
    /// * `EventData` structure for the event
    pub fn create_event(&self, event_type: &str, data: serde_json::Value) -> EventData {
        EventData {
            event_type: event_type.to_string(),
            timestamp: Utc::now(),
            data,
        }
    }

    // ========== Private Helper Methods ==========

    /// Generate a sensor value within a range with some randomness
    fn generate_sensor_value(&self, min: f64, max: f64, variation: f64) -> f64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // Use current time as seed for pseudo-randomness
        let mut hasher = DefaultHasher::new();
        Utc::now().timestamp_millis().hash(&mut hasher);
        let seed = hasher.finish();
        
        // Simple linear congruential generator for deterministic randomness
        let random = ((seed.wrapping_mul(1103515245).wrapping_add(12345)) >> 16) as f64 / 32768.0;
        
        let base = min + (max - min) * 0.5; // Middle of range
        let offset = (random - 0.5) * variation;
        
        (base + offset).clamp(min, max)
    }

    /// Calculate battery level with drain over time
    fn calculate_battery_level(&self, config: &SensorConfig, _now: DateTime<Utc>) -> f64 {
        // For simulation, assume we started with 100% battery at system start
        // and drain at the configured rate
        let hours_elapsed = 1.0; // Simplified - in real implementation would track actual uptime
        let drain = config.battery_drain_rate * hours_elapsed;
        (100.0 - drain).clamp(0.0, 100.0)
    }

    /// Generate light level based on time of day simulation
    fn generate_light_level(&self, config: &SensorConfig, now: DateTime<Utc>) -> f64 {
        // Simulate day/night cycle based on hour of day
        let hour = now.hour() as f64;
        let base_light = if (6.0..=18.0).contains(&hour) {
            // Daytime: higher light levels
            50.0 + 40.0 * ((hour - 12.0).abs() / 6.0).cos()
        } else {
            // Nighttime: lower light levels
            5.0 + 10.0 * (hour / 24.0 * 2.0 * std::f64::consts::PI).sin().abs()
        };
        
        // Add some variation
        let variation = self.generate_sensor_value(-config.light_variation, config.light_variation, 1.0);
        (base_light + variation).clamp(0.0, 100.0)
    }

    // ========== IoT Communication APIs ==========

    /// Publish a message to an MQTT topic
    /// 
    /// # Arguments
    /// * `topic` - MQTT topic to publish to
    /// * `message` - Message content to publish
    /// 
    /// # Returns
    /// * `Ok(())` if message was published successfully
    /// * `Err(APIError)` if topic is invalid or publish failed
    pub async fn publish_mqtt(&self, topic: &str, message: &str) -> Result<(), crate::APIError> {
        use crate::APIError;
        
        if topic.is_empty() {
            return Err(APIError::InvalidParameter("Topic cannot be empty".to_string()));
        }
        
        if topic.contains('+') || topic.contains('#') {
            return Err(APIError::InvalidParameter("Topic cannot contain wildcards".to_string()));
        }
        
        // In a real implementation, this would use an IoT client
        // For now, we'll just validate and return success
        tracing::info!("Publishing to topic '{}': {}", topic, message);
        Ok(())
    }

    /// Update device shadow with a key-value pair
    /// 
    /// # Arguments
    /// * `key` - Shadow property key
    /// * `value` - Shadow property value as JSON
    /// 
    /// # Returns
    /// * `Ok(())` if shadow was updated successfully
    /// * `Err(APIError)` if key is invalid or update failed
    pub async fn update_shadow(&self, key: &str, value: serde_json::Value) -> Result<(), crate::APIError> {
        use crate::APIError;
        
        if key.is_empty() {
            return Err(APIError::InvalidParameter("Shadow key cannot be empty".to_string()));
        }
        
        // In a real implementation, this would use an IoT client to update the device shadow
        // For now, we'll just validate and return success
        tracing::info!("Updating shadow key '{}' with value: {}", key, value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PlatformError, PlatformResult, DeviceInfo, MemoryInfo, UptimeInfo};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicBool, Ordering};

    // Mock HAL for testing
    struct MockHAL {
        sleep_should_fail: AtomicBool,
        led_should_fail: AtomicBool,
        current_led_state: Arc<RwLock<LedState>>,
    }

    impl MockHAL {
        fn new() -> Self {
            Self {
                sleep_should_fail: AtomicBool::new(false),
                led_should_fail: AtomicBool::new(false),
                current_led_state: Arc::new(RwLock::new(LedState::Off)),
            }
        }

        fn set_sleep_should_fail(&self, should_fail: bool) {
            self.sleep_should_fail.store(should_fail, Ordering::Relaxed);
        }

        fn set_led_should_fail(&self, should_fail: bool) {
            self.led_should_fail.store(should_fail, Ordering::Relaxed);
        }
    }

    #[async_trait]
    impl PlatformHAL for MockHAL {
        async fn sleep(&self, _duration: Duration) -> PlatformResult<()> {
            if self.sleep_should_fail.load(Ordering::Relaxed) {
                return Err(PlatformError::Sleep("Mock sleep failure".to_string()));
            }
            
            // Simulate sleep with a very short actual delay for testing
            tokio::time::sleep(Duration::from_millis(1)).await;
            Ok(())
        }

        async fn set_led(&self, state: LedState) -> PlatformResult<()> {
            if self.led_should_fail.load(Ordering::Relaxed) {
                return Err(PlatformError::Led("Mock LED failure".to_string()));
            }
            
            *self.current_led_state.write() = state;
            Ok(())
        }

        async fn get_led_state(&self) -> PlatformResult<LedState> {
            Ok(*self.current_led_state.read())
        }

        async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
            Ok(DeviceInfo {
                device_id: "mock-device".to_string(),
                platform: "mock".to_string(),
                version: "1.0.0".to_string(),
                firmware_version: "1.0.0".to_string(),
                hardware_revision: Some("rev1".to_string()),
                serial_number: Some("12345".to_string()),
            })
        }

        async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
            Ok(MemoryInfo {
                total_bytes: 1024 * 1024,
                free_bytes: 512 * 1024,
                used_bytes: 512 * 1024,
                largest_free_block: 256 * 1024,
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

    #[tokio::test]
    async fn test_sleep_with_valid_duration() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        let result = api.sleep(1.0).await;
        assert!(result.is_ok());

        // Check that hardware state was updated
        let state = api.get_hardware_state();
        assert!(matches!(state.sleep_status, SleepStatus::Awake));
    }

    #[tokio::test]
    async fn test_sleep_with_invalid_duration() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test negative duration
        let result = api.sleep(-1.0).await;
        assert!(result.is_err());

        // Test duration too long
        let result = api.sleep(3601.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sleep_hal_failure() {
        let mock_hal = Arc::new(MockHAL::new());
        mock_hal.set_sleep_should_fail(true);
        let api = RustAPI::new(mock_hal);

        let result = api.sleep(1.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_led_control() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test turning LED on
        let result = api.set_led(true).await;
        assert!(result.is_ok());

        let led_state = api.get_led_state().await.unwrap();
        assert!(led_state);

        // Check hardware state tracking
        let hw_state = api.get_hardware_state();
        assert_eq!(hw_state.led_state, LedState::On);

        // Test turning LED off
        let result = api.set_led(false).await;
        assert!(result.is_ok());

        let led_state = api.get_led_state().await.unwrap();
        assert!(!led_state);
    }

    #[tokio::test]
    async fn test_led_hal_failure() {
        let mock_hal = Arc::new(MockHAL::new());
        mock_hal.set_led_should_fail(true);
        let api = RustAPI::new(mock_hal);

        let result = api.set_led(true).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sensor_data_generation() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        let sensor_data = api.get_sensor_data().await.unwrap();

        // Verify sensor data is within expected ranges
        assert!(sensor_data.temperature >= 18.0 && sensor_data.temperature <= 28.0);
        assert!(sensor_data.humidity >= 40.0 && sensor_data.humidity <= 80.0);
        assert!(sensor_data.pressure >= 1000.0 && sensor_data.pressure <= 1020.0);
        assert!(sensor_data.battery_level >= 0.0 && sensor_data.battery_level <= 100.0);
        assert!(sensor_data.light_level >= 0.0 && sensor_data.light_level <= 100.0);

        // Check that hardware state was updated
        let hw_state = api.get_hardware_state();
        assert!(hw_state.last_sensor_reading.is_some());
    }

    #[tokio::test]
    async fn test_sensor_configuration() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Configure custom sensor ranges
        let custom_config = SensorConfig {
            temperature_range: (20.0, 25.0),
            humidity_range: (50.0, 70.0),
            pressure_range: (1010.0, 1015.0),
            battery_drain_rate: 1.0,
            light_variation: 0.1,
            update_interval: Duration::from_secs(60),
        };

        api.configure_sensors(custom_config);

        let sensor_data = api.get_sensor_data().await.unwrap();

        // Verify sensor data respects new configuration
        assert!(sensor_data.temperature >= 20.0 && sensor_data.temperature <= 25.0);
        assert!(sensor_data.humidity >= 50.0 && sensor_data.humidity <= 70.0);
        assert!(sensor_data.pressure >= 1010.0 && sensor_data.pressure <= 1015.0);
    }

    #[tokio::test]
    async fn test_hardware_state_tracking() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Initial state should be default
        let initial_state = api.get_hardware_state();
        assert_eq!(initial_state.led_state, LedState::Off);
        assert!(matches!(initial_state.sleep_status, SleepStatus::Awake));
        assert!(initial_state.last_sensor_reading.is_none());

        // Change LED state and verify tracking
        api.set_led(true).await.unwrap();
        let updated_state = api.get_hardware_state();
        assert_eq!(updated_state.led_state, LedState::On);

        // Generate sensor data and verify tracking
        api.get_sensor_data().await.unwrap();
        let sensor_state = api.get_hardware_state();
        assert!(sensor_state.last_sensor_reading.is_some());
    }

    #[tokio::test]
    async fn test_secure_data_storage() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test storing data
        let result = api.store_data("test_key", "test_value").await;
        assert!(result.is_ok());

        // Test loading data (mock returns None, so we expect None)
        let loaded = api.load_data("test_key").await.unwrap();
        assert_eq!(loaded, None); // Mock HAL returns None

        // Test deleting data
        let deleted = api.delete_data("test_key").await.unwrap();
        assert!(!deleted); // Mock HAL returns false

        // Test listing keys
        let keys = api.list_storage_keys().await.unwrap();
        assert!(keys.is_empty()); // Mock HAL returns empty vec
    }

    #[tokio::test]
    async fn test_storage_validation() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test empty key validation
        let result = api.store_data("", "value").await;
        assert!(result.is_err());

        let result = api.load_data("").await;
        assert!(result.is_err());

        let result = api.delete_data("").await;
        assert!(result.is_err());

        // Test key too long
        let long_key = "a".repeat(300);
        let result = api.store_data(&long_key, "value").await;
        assert!(result.is_err());

        // Test value too large
        let large_value = "a".repeat(70000);
        let result = api.store_data("key", &large_value).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_system_info_apis() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test device info
        let device_info = api.get_device_info().await.unwrap();
        assert_eq!(device_info.device_id, "mock-device");
        assert_eq!(device_info.platform, "mock");

        // Test memory info
        let memory_info = api.get_memory_info().await.unwrap();
        assert_eq!(memory_info.total_bytes, 1024 * 1024);
        assert_eq!(memory_info.free_bytes, 512 * 1024);

        // Test uptime
        let uptime = api.get_uptime().await.unwrap();
        assert_eq!(uptime, Duration::from_secs(3600));
    }

    #[tokio::test]
    async fn test_logging_apis() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test simple logging
        let result = api.log(LogLevel::Info, "Test message");
        assert!(result.is_ok());

        // Test structured logging
        let context = [("key1", "value1"), ("key2", "value2")];
        let result = api.log_structured(LogLevel::Debug, "Test structured message", Some(&context));
        assert!(result.is_ok());

        // Test all log levels
        for level in [LogLevel::Error, LogLevel::Warn, LogLevel::Info, LogLevel::Debug, LogLevel::Trace] {
            let result = api.log(level, "Test message");
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_device_configuration() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test default configuration
        let default_config = api.get_device_config();
        assert_eq!(default_config.device_id, "unknown");
        assert_eq!(default_config.log_level, LogLevel::Info);

        // Test updating configuration
        let new_config = DeviceConfig {
            device_id: "test-device".to_string(),
            log_level: LogLevel::Debug,
            sensor_update_interval: 60,
            max_storage_keys: 500,
            enable_debug_mode: true,
            heartbeat_interval: 30,
        };

        let result = api.update_device_config(new_config.clone());
        assert!(result.is_ok());

        let updated_config = api.get_device_config();
        assert_eq!(updated_config.device_id, "test-device");
        assert_eq!(updated_config.log_level, LogLevel::Debug);
        assert_eq!(updated_config.sensor_update_interval, 60);
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test empty device ID
        let invalid_config = DeviceConfig {
            device_id: "".to_string(),
            ..DeviceConfig::default()
        };
        let result = api.update_device_config(invalid_config);
        assert!(result.is_err());

        // Test zero sensor update interval
        let invalid_config = DeviceConfig {
            sensor_update_interval: 0,
            ..DeviceConfig::default()
        };
        let result = api.update_device_config(invalid_config);
        assert!(result.is_err());

        // Test zero heartbeat interval
        let invalid_config = DeviceConfig {
            heartbeat_interval: 0,
            ..DeviceConfig::default()
        };
        let result = api.update_device_config(invalid_config);
        assert!(result.is_err());

        // Test zero max storage keys
        let invalid_config = DeviceConfig {
            max_storage_keys: 0,
            ..DeviceConfig::default()
        };
        let result = api.update_device_config(invalid_config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_configuration_persistence() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test saving configuration (will succeed with mock HAL)
        let result = api.save_device_config().await;
        assert!(result.is_ok());

        // Test loading configuration (mock returns None, so should use defaults)
        let result = api.load_device_config().await;
        assert!(result.is_ok());

        // Test resetting configuration
        let result = api.reset_device_config();
        assert!(result.is_ok());

        let config = api.get_device_config();
        assert_eq!(config.device_id, "unknown"); // Should be back to default
    }

    #[tokio::test]
    async fn test_timer_management() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test setting a timer
        let timer_handle = api.set_timer("test_timer", 0.1, "(log-info \"Timer fired!\")").unwrap();
        
        // Verify timer was created
        let timer_info = api.get_timer_info(&timer_handle).unwrap();
        assert_eq!(timer_info.name, "test_timer");
        assert!(timer_info.is_active);
        
        // List timers
        let timers = api.list_timers();
        assert_eq!(timers.len(), 1);
        assert_eq!(timers[0].name, "test_timer");
        
        // Cancel timer
        let cancelled = api.cancel_timer(timer_handle.clone()).unwrap();
        assert!(cancelled);
        
        // Verify timer is no longer active
        let timer_info = api.get_timer_info(&timer_handle).unwrap();
        assert!(!timer_info.is_active);
    }

    #[tokio::test]
    async fn test_timer_validation() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test invalid duration
        let result = api.set_timer("test", -1.0, "(log-info \"test\")");
        assert!(result.is_err());

        let result = api.set_timer("test", 0.0, "(log-info \"test\")");
        assert!(result.is_err());

        let result = api.set_timer("test", 90000.0, "(log-info \"test\")"); // > 24 hours
        assert!(result.is_err());

        // Test empty name
        let result = api.set_timer("", 1.0, "(log-info \"test\")");
        assert!(result.is_err());

        // Test empty callback
        let result = api.set_timer("test", 1.0, "");
        assert!(result.is_err());

        let result = api.set_timer("test", 1.0, "   ");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_task_scheduling() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test scheduling a task
        let task_handle = api.schedule_task("test_task", "*/5 * * * *", "(log-info \"Task executed!\")").unwrap();
        
        // Verify task was created
        let task_info = api.get_task_info(&task_handle).unwrap();
        assert_eq!(task_info.name, "test_task");
        assert_eq!(task_info.cron_expression, "*/5 * * * *");
        assert!(task_info.is_active);
        assert_eq!(task_info.run_count, 0);
        
        // List tasks
        let tasks = api.list_tasks();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "test_task");
        
        // Cancel task
        let cancelled = api.cancel_task(task_handle.clone()).unwrap();
        assert!(cancelled);
        
        // Verify task is no longer active
        let task_info = api.get_task_info(&task_handle).unwrap();
        assert!(!task_info.is_active);
    }

    #[tokio::test]
    async fn test_task_validation() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test empty name
        let result = api.schedule_task("", "*/5 * * * *", "(log-info \"test\")");
        assert!(result.is_err());

        // Test empty cron expression
        let result = api.schedule_task("test", "", "(log-info \"test\")");
        assert!(result.is_err());

        let result = api.schedule_task("test", "   ", "(log-info \"test\")");
        assert!(result.is_err());

        // Test empty callback
        let result = api.schedule_task("test", "*/5 * * * *", "");
        assert!(result.is_err());

        let result = api.schedule_task("test", "*/5 * * * *", "   ");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cron_parsing() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test valid cron expressions
        let result = api.schedule_task("test1", "*/30 * * * *", "(log-info \"every 30 seconds\")");
        assert!(result.is_ok());

        let result = api.schedule_task("test2", "0 */5 * * *", "(log-info \"every 5 minutes\")");
        assert!(result.is_ok());

        let result = api.schedule_task("test3", "0 0 */2 * *", "(log-info \"every 2 hours\")");
        assert!(result.is_ok());

        // Test invalid cron expressions
        let result = api.schedule_task("test4", "invalid", "(log-info \"test\")");
        assert!(result.is_err());

        let result = api.schedule_task("test5", "* * *", "(log-info \"test\")"); // Too few parts
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_event_system() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test creating events
        let event_data = serde_json::json!({
            "sensor": "temperature",
            "value": 25.5,
            "unit": "celsius"
        });

        let event = api.create_event("sensor_reading", event_data.clone());
        assert_eq!(event.event_type, "sensor_reading");
        assert_eq!(event.data, event_data);
        
        // Timestamp should be recent
        let now = Utc::now();
        let time_diff = now.signed_duration_since(event.timestamp);
        assert!(time_diff.num_seconds() < 1);
    }

    #[tokio::test]
    async fn test_timer_event_receiver() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Test taking event receiver
        let receiver = api.take_timer_event_receiver();
        assert!(receiver.is_some());

        // Second call should return None
        let receiver2 = api.take_timer_event_receiver();
        assert!(receiver2.is_none());
    }

    #[tokio::test]
    async fn test_timer_firing() {
        let mock_hal = Arc::new(MockHAL::new());
        let api = RustAPI::new(mock_hal);

        // Take the event receiver first
        let mut receiver = api.take_timer_event_receiver().unwrap();

        // Set a very short timer
        let _timer_handle = api.set_timer("quick_timer", 0.01, "(log-info \"Quick timer!\")").unwrap();

        // Wait for the timer event
        let event = tokio::time::timeout(Duration::from_millis(100), receiver.recv()).await;
        assert!(event.is_ok());
        
        if let Ok(Some(TimerEvent::TimerFired { name, .. })) = event {
            assert_eq!(name, "quick_timer");
        } else {
            panic!("Expected TimerFired event");
        }
    }
}