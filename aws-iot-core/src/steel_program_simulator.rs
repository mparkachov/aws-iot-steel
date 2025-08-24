use crate::{
    steel_program_validator::{SteelProgramValidator, ValidationResult},
    steel_runtime::{SteelRuntimeAPI, SteelRuntimeImpl},
    DeviceInfo, LedState, MemoryInfo, PlatformHAL, PlatformResult, SystemError, SystemResult,
    UptimeInfo,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info};
use uuid::Uuid;

/// Steel program simulator for development and testing
pub struct SteelProgramSimulator {
    runtime: SteelRuntimeImpl,
    validator: SteelProgramValidator,
    simulation_state: Arc<RwLock<SimulationState>>,
    execution_history: Arc<RwLock<Vec<ExecutionRecord>>>,
    breakpoints: Arc<RwLock<HashMap<String, Vec<usize>>>>, // program_name -> line numbers
    step_mode: Arc<RwLock<bool>>,
    debug_output: Arc<RwLock<Vec<DebugMessage>>>,
}

/// Current simulation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub is_running: bool,
    pub current_program: Option<String>,
    pub execution_start_time: Option<DateTime<Utc>>,
    pub total_programs_executed: u64,
    pub total_execution_time: Duration,
    pub hardware_state: SimulatedHardwareState,
    pub environment_variables: HashMap<String, String>,
}

/// Simulated hardware state for development
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedHardwareState {
    pub led_state: LedState,
    pub sleep_remaining: Option<Duration>,
    pub sensor_values: HashMap<String, f64>,
    pub memory_usage: usize,
    pub uptime: Duration,
    pub device_info: SimulatedDeviceInfo,
}

/// Simulated device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedDeviceInfo {
    pub device_id: String,
    pub platform: String,
    pub version: String,
    pub firmware_version: String,
    pub hardware_revision: Option<String>,
    pub serial_number: Option<String>,
}

/// Execution record for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: String,
    pub program_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub result: ExecutionResult,
    pub debug_output: Vec<DebugMessage>,
    pub validation_result: Option<ValidationResult>,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionResult {
    Success,
    Error(String),
    Timeout,
    Cancelled,
    BreakpointHit(usize), // line number
}

/// Debug message with timestamp and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugMessage {
    pub timestamp: DateTime<Utc>,
    pub level: DebugLevel,
    pub message: String,
    pub context: Option<DebugContext>,
}

/// Debug message levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Debug context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugContext {
    pub program_name: Option<String>,
    pub line_number: Option<usize>,
    pub function_name: Option<String>,
    pub variable_values: HashMap<String, String>,
}

/// Simulation configuration
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub enable_validation: bool,
    pub enable_debugging: bool,
    pub execution_timeout: Duration,
    pub max_history_records: usize,
    pub simulate_hardware_delays: bool,
    pub hardware_delay_multiplier: f64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            enable_validation: true,
            enable_debugging: true,
            execution_timeout: Duration::from_secs(30),
            max_history_records: 1000,
            simulate_hardware_delays: true,
            hardware_delay_multiplier: 0.1, // 10% of real delays for faster simulation
        }
    }
}

/// Mock HAL implementation for simulation
pub struct SimulatorHAL {
    state: Arc<RwLock<SimulatedHardwareState>>,
    config: SimulationConfig,
    start_time: Instant,
}

impl SimulatorHAL {
    pub fn new(config: SimulationConfig) -> Self {
        let device_info = SimulatedDeviceInfo {
            device_id: "sim-device-001".to_string(),
            platform: "simulator".to_string(),
            version: "1.0.0".to_string(),
            firmware_version: "sim-fw-1.0.0".to_string(),
            hardware_revision: Some("sim-rev-1".to_string()),
            serial_number: Some("SIM123456789".to_string()),
        };

        let mut sensor_values = HashMap::new();
        sensor_values.insert("temperature".to_string(), 22.5);
        sensor_values.insert("humidity".to_string(), 65.0);
        sensor_values.insert("pressure".to_string(), 1013.25);
        sensor_values.insert("battery".to_string(), 85.0);

        let state = SimulatedHardwareState {
            led_state: LedState::Off,
            sleep_remaining: None,
            sensor_values,
            memory_usage: 1024 * 512, // 512KB used
            uptime: Duration::from_secs(0),
            device_info,
        };

        Self {
            state: Arc::new(RwLock::new(state)),
            config,
            start_time: Instant::now(),
        }
    }

    pub fn get_state(&self) -> SimulatedHardwareState {
        self.state.read().clone()
    }

    pub fn set_sensor_value(&self, sensor: &str, value: f64) {
        self.state
            .write()
            .sensor_values
            .insert(sensor.to_string(), value);
    }

    pub fn simulate_hardware_failure(&self, component: &str) -> SystemResult<()> {
        match component {
            "led" => {
                info!("Simulating LED hardware failure");
                // Could set a flag to make LED operations fail
                Ok(())
            }
            "memory" => {
                info!("Simulating memory pressure");
                self.state.write().memory_usage = 1024 * 1024 - 1024; // Almost full
                Ok(())
            }
            _ => Err(SystemError::Configuration(format!(
                "Unknown component for failure simulation: {}",
                component
            ))),
        }
    }
}

#[async_trait]
impl PlatformHAL for SimulatorHAL {
    async fn sleep(&self, duration: Duration) -> PlatformResult<()> {
        info!("Simulator: Sleep for {:?}", duration);

        // Update state to show sleeping
        {
            let mut state = self.state.write();
            state.sleep_remaining = Some(duration);
        }

        // Simulate sleep with reduced duration for faster testing
        let sim_duration = if self.config.simulate_hardware_delays {
            Duration::from_secs_f64(duration.as_secs_f64() * self.config.hardware_delay_multiplier)
        } else {
            Duration::from_millis(10) // Minimal delay for simulation
        };

        tokio::time::sleep(sim_duration).await;

        // Clear sleep state
        {
            let mut state = self.state.write();
            state.sleep_remaining = None;
        }

        info!("Simulator: Sleep completed");
        Ok(())
    }

    async fn set_led(&self, led_state: LedState) -> PlatformResult<()> {
        info!("Simulator: Set LED to {:?}", led_state);

        // Simulate hardware delay
        if self.config.simulate_hardware_delays {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        self.state.write().led_state = led_state;
        Ok(())
    }

    async fn get_led_state(&self) -> PlatformResult<LedState> {
        let state = self.state.read().led_state;
        debug!("Simulator: LED state is {:?}", state);
        Ok(state)
    }

    async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
        let sim_info = self.state.read().device_info.clone();
        let device_info = DeviceInfo {
            device_id: sim_info.device_id,
            platform: sim_info.platform,
            version: sim_info.version,
            firmware_version: sim_info.firmware_version,
            hardware_revision: sim_info.hardware_revision,
            serial_number: sim_info.serial_number,
        };
        debug!("Simulator: Device info requested");
        Ok(device_info)
    }

    async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
        let state = self.state.read();
        let total_bytes = 1024 * 1024; // 1MB total
        let used_bytes = state.memory_usage;
        let free_bytes = total_bytes - used_bytes;

        let memory_info = MemoryInfo {
            total_bytes: total_bytes as u64,
            free_bytes: free_bytes as u64,
            used_bytes: used_bytes as u64,
            largest_free_block: (free_bytes / 2) as u64, // Simulate fragmentation
        };

        debug!(
            "Simulator: Memory info - used: {}, free: {}",
            used_bytes, free_bytes
        );
        Ok(memory_info)
    }

    async fn get_uptime(&self) -> PlatformResult<UptimeInfo> {
        let uptime = self.start_time.elapsed();
        let uptime_info = UptimeInfo {
            uptime,
            boot_time: Utc::now() - chrono::Duration::from_std(uptime).unwrap(),
        };
        debug!("Simulator: Uptime is {:?}", uptime);
        Ok(uptime_info)
    }

    async fn store_secure_data(&self, key: &str, data: &[u8]) -> PlatformResult<()> {
        info!("Simulator: Store secure data for key: {}", key);
        // In a real implementation, this would store to a simulated secure storage
        // For now, we just log the operation
        debug!("Simulator: Stored {} bytes for key '{}'", data.len(), key);
        Ok(())
    }

    async fn load_secure_data(&self, key: &str) -> PlatformResult<Option<Vec<u8>>> {
        info!("Simulator: Load secure data for key: {}", key);
        // In a real implementation, this would load from simulated secure storage
        // For now, we return None (no data found)
        debug!("Simulator: No data found for key '{}'", key);
        Ok(None)
    }

    async fn delete_secure_data(&self, key: &str) -> PlatformResult<bool> {
        info!("Simulator: Delete secure data for key: {}", key);
        // In a real implementation, this would delete from simulated secure storage
        debug!("Simulator: Data deleted for key '{}'", key);
        Ok(true)
    }

    async fn list_secure_keys(&self) -> PlatformResult<Vec<String>> {
        info!("Simulator: List secure storage keys");
        // Return empty list for simulation
        Ok(vec![])
    }

    async fn initialize(&mut self) -> PlatformResult<()> {
        info!("Simulator: Initialize");
        Ok(())
    }

    async fn shutdown(&mut self) -> PlatformResult<()> {
        info!("Simulator: Shutdown");
        Ok(())
    }
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            is_running: false,
            current_program: None,
            execution_start_time: None,
            total_programs_executed: 0,
            total_execution_time: Duration::from_secs(0),
            hardware_state: SimulatedHardwareState {
                led_state: LedState::Off,
                sleep_remaining: None,
                sensor_values: HashMap::new(),
                memory_usage: 1024 * 512,
                uptime: Duration::from_secs(0),
                device_info: SimulatedDeviceInfo {
                    device_id: "sim-device-001".to_string(),
                    platform: "simulator".to_string(),
                    version: "1.0.0".to_string(),
                    firmware_version: "sim-fw-1.0.0".to_string(),
                    hardware_revision: Some("sim-rev-1".to_string()),
                    serial_number: Some("SIM123456789".to_string()),
                },
            },
            environment_variables: HashMap::new(),
        }
    }
}

// Safety: SteelProgramSimulator uses Arc<RwLock<...>> for all shared state,
// making it safe to send between threads and share references
unsafe impl Send for SteelProgramSimulator {}
unsafe impl Sync for SteelProgramSimulator {}

impl SteelProgramSimulator {
    /// Create a new Steel program simulator
    pub fn new(config: SimulationConfig) -> SystemResult<Self> {
        let hal = Arc::new(SimulatorHAL::new(config.clone()));
        let rust_api = Arc::new(SteelRuntimeAPI::new(hal)?);
        let runtime = SteelRuntimeImpl::new(rust_api)?;
        let validator = SteelProgramValidator::new();

        Ok(Self {
            runtime,
            validator,
            simulation_state: Arc::new(RwLock::new(SimulationState::default())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            breakpoints: Arc::new(RwLock::new(HashMap::new())),
            step_mode: Arc::new(RwLock::new(false)),
            debug_output: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Create a simulator with default configuration
    pub fn new_default() -> SystemResult<Self> {
        Self::new(SimulationConfig::default())
    }

    /// Simulate a Steel program with full debugging support
    pub async fn simulate_program(
        &self,
        code: &str,
        program_name: &str,
    ) -> SystemResult<ExecutionRecord> {
        info!("Starting simulation of program: {}", program_name);

        let execution_id = Uuid::new_v4().to_string();
        let start_time = Utc::now();

        // Clear previous debug output
        self.debug_output.write().clear();

        // Add debug message
        self.add_debug_message(
            DebugLevel::Info,
            &format!("Starting simulation of program: {}", program_name),
            Some(DebugContext {
                program_name: Some(program_name.to_string()),
                line_number: None,
                function_name: None,
                variable_values: HashMap::new(),
            }),
        );

        // Validate program if enabled
        let validation_result = if self.get_config().enable_validation {
            match self.validator.validate(code) {
                Ok(result) => {
                    if !result.is_valid {
                        let error_msg =
                            format!("Program validation failed: {} errors", result.errors.len());
                        self.add_debug_message(DebugLevel::Error, &error_msg, None);

                        let execution_record = ExecutionRecord {
                            id: execution_id,
                            program_name: program_name.to_string(),
                            start_time,
                            end_time: Some(Utc::now()),
                            duration: Some(Duration::from_millis(0)),
                            result: ExecutionResult::Error(error_msg),
                            debug_output: self.debug_output.read().clone(),
                            validation_result: Some(result),
                        };

                        self.add_execution_record(execution_record.clone());
                        return Ok(execution_record);
                    }
                    Some(result)
                }
                Err(e) => {
                    let error_msg = format!("Validation error: {}", e);
                    self.add_debug_message(DebugLevel::Error, &error_msg, None);

                    let execution_record = ExecutionRecord {
                        id: execution_id,
                        program_name: program_name.to_string(),
                        start_time,
                        end_time: Some(Utc::now()),
                        duration: Some(Duration::from_millis(0)),
                        result: ExecutionResult::Error(error_msg),
                        debug_output: self.debug_output.read().clone(),
                        validation_result: None,
                    };

                    self.add_execution_record(execution_record.clone());
                    return Ok(execution_record);
                }
            }
        } else {
            None
        };

        // Update simulation state
        {
            let mut state = self.simulation_state.write();
            state.is_running = true;
            state.current_program = Some(program_name.to_string());
            state.execution_start_time = Some(start_time);
        }

        // Execute the program
        let execution_result = match tokio::time::timeout(
            self.get_config().execution_timeout,
            self.execute_with_debugging(code, program_name),
        )
        .await
        {
            Ok(Ok(_)) => {
                self.add_debug_message(
                    DebugLevel::Info,
                    "Program execution completed successfully",
                    None,
                );
                ExecutionResult::Success
            }
            Ok(Err(e)) => {
                let error_msg = format!("Execution error: {}", e);
                self.add_debug_message(DebugLevel::Error, &error_msg, None);
                ExecutionResult::Error(error_msg)
            }
            Err(_) => {
                self.add_debug_message(DebugLevel::Error, "Program execution timed out", None);
                ExecutionResult::Timeout
            }
        };

        let end_time = Utc::now();
        let duration = end_time
            .signed_duration_since(start_time)
            .to_std()
            .unwrap_or(Duration::from_secs(0));

        // Update simulation state
        {
            let mut state = self.simulation_state.write();
            state.is_running = false;
            state.current_program = None;
            state.execution_start_time = None;
            state.total_programs_executed += 1;
            state.total_execution_time += duration;
        }

        // Create execution record
        let execution_record = ExecutionRecord {
            id: execution_id,
            program_name: program_name.to_string(),
            start_time,
            end_time: Some(end_time),
            duration: Some(duration),
            result: execution_result,
            debug_output: self.debug_output.read().clone(),
            validation_result,
        };

        self.add_execution_record(execution_record.clone());

        info!(
            "Simulation completed for program: {} in {:?}",
            program_name, duration
        );
        Ok(execution_record)
    }

    /// Execute program with debugging support
    async fn execute_with_debugging(&self, code: &str, program_name: &str) -> SystemResult<()> {
        self.add_debug_message(DebugLevel::Debug, "Loading program into runtime", None);

        // Load program into runtime
        let handle = self.runtime.load_program(code, Some(program_name)).await?;

        self.add_debug_message(
            DebugLevel::Debug,
            &format!("Program loaded with handle: {:?}", handle),
            None,
        );

        // Check for breakpoints if in debug mode
        if self.get_config().enable_debugging {
            self.check_breakpoints(program_name, 1).await?;
        }

        // Execute the program
        self.add_debug_message(DebugLevel::Debug, "Starting program execution", None);
        let _result = self.runtime.execute_program(handle).await?;

        self.add_debug_message(DebugLevel::Debug, "Program execution completed", None);
        Ok(())
    }

    /// Check if execution should pause at breakpoints
    async fn check_breakpoints(&self, program_name: &str, line: usize) -> SystemResult<()> {
        let breakpoints = self.breakpoints.read();
        if let Some(program_breakpoints) = breakpoints.get(program_name) {
            if program_breakpoints.contains(&line) {
                self.add_debug_message(
                    DebugLevel::Info,
                    &format!("Breakpoint hit at line {}", line),
                    Some(DebugContext {
                        program_name: Some(program_name.to_string()),
                        line_number: Some(line),
                        function_name: None,
                        variable_values: HashMap::new(),
                    }),
                );

                // In a real debugger, this would pause execution
                // For simulation, we just log the breakpoint hit
                info!(
                    "Breakpoint hit at line {} in program {}",
                    line, program_name
                );
            }
        }
        Ok(())
    }

    /// Add a debug message
    fn add_debug_message(&self, level: DebugLevel, message: &str, context: Option<DebugContext>) {
        let debug_msg = DebugMessage {
            timestamp: Utc::now(),
            level,
            message: message.to_string(),
            context,
        };

        self.debug_output.write().push(debug_msg);
    }

    /// Add execution record to history
    fn add_execution_record(&self, record: ExecutionRecord) {
        let mut history = self.execution_history.write();
        history.push(record);

        // Limit history size
        let max_records = self.get_config().max_history_records;
        if history.len() > max_records {
            let excess = history.len() - max_records;
            history.drain(0..excess);
        }
    }

    /// Get simulation configuration (simplified for this implementation)
    fn get_config(&self) -> SimulationConfig {
        SimulationConfig::default()
    }

    // ========== Public API Methods ==========

    /// Set a breakpoint at a specific line in a program
    pub fn set_breakpoint(&self, program_name: &str, line: usize) -> SystemResult<()> {
        let mut breakpoints = self.breakpoints.write();
        breakpoints
            .entry(program_name.to_string())
            .or_default()
            .push(line);

        info!(
            "Breakpoint set at line {} in program {}",
            line, program_name
        );
        Ok(())
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&self, program_name: &str, line: usize) -> SystemResult<bool> {
        let mut breakpoints = self.breakpoints.write();
        if let Some(program_breakpoints) = breakpoints.get_mut(program_name) {
            if let Some(pos) = program_breakpoints.iter().position(|&x| x == line) {
                program_breakpoints.remove(pos);
                info!(
                    "Breakpoint removed from line {} in program {}",
                    line, program_name
                );
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Clear all breakpoints for a program
    pub fn clear_breakpoints(&self, program_name: &str) -> SystemResult<()> {
        let mut breakpoints = self.breakpoints.write();
        breakpoints.remove(program_name);
        info!("All breakpoints cleared for program {}", program_name);
        Ok(())
    }

    /// List all breakpoints
    pub fn list_breakpoints(&self) -> HashMap<String, Vec<usize>> {
        self.breakpoints.read().clone()
    }

    /// Enable or disable step mode
    pub fn set_step_mode(&self, enabled: bool) {
        *self.step_mode.write() = enabled;
        info!("Step mode {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Get current simulation state
    pub fn get_simulation_state(&self) -> SimulationState {
        self.simulation_state.read().clone()
    }

    /// Get execution history
    pub fn get_execution_history(&self) -> Vec<ExecutionRecord> {
        self.execution_history.read().clone()
    }

    /// Get recent debug output
    pub fn get_debug_output(&self) -> Vec<DebugMessage> {
        self.debug_output.read().clone()
    }

    /// Clear execution history
    pub fn clear_history(&self) {
        self.execution_history.write().clear();
        info!("Execution history cleared");
    }

    /// Get simulator hardware state
    pub fn get_hardware_state(&self) -> SystemResult<SimulatedHardwareState> {
        // This would need access to the HAL, which we don't have directly
        // For now, return the state from simulation_state
        Ok(self.simulation_state.read().hardware_state.clone())
    }

    /// Set environment variable for simulation
    pub fn set_environment_variable(&self, key: &str, value: &str) {
        self.simulation_state
            .write()
            .environment_variables
            .insert(key.to_string(), value.to_string());
        info!("Environment variable set: {} = {}", key, value);
    }

    /// Get environment variable
    pub fn get_environment_variable(&self, key: &str) -> Option<String> {
        self.simulation_state
            .read()
            .environment_variables
            .get(key)
            .cloned()
    }

    /// Reset simulation state
    pub fn reset(&self) {
        *self.simulation_state.write() = SimulationState::default();
        self.execution_history.write().clear();
        self.breakpoints.write().clear();
        self.debug_output.write().clear();
        *self.step_mode.write() = false;
        info!("Simulator reset to initial state");
    }

    /// Get simulation statistics
    pub fn get_statistics(&self) -> SimulationStatistics {
        let state = self.simulation_state.read();
        let history = self.execution_history.read();

        let successful_executions = history
            .iter()
            .filter(|r| matches!(r.result, ExecutionResult::Success))
            .count();

        let failed_executions = history
            .iter()
            .filter(|r| matches!(r.result, ExecutionResult::Error(_)))
            .count();

        let average_execution_time = if !history.is_empty() {
            let total_time: Duration = history.iter().filter_map(|r| r.duration).sum();
            total_time / history.len() as u32
        } else {
            Duration::from_secs(0)
        };

        SimulationStatistics {
            total_programs_executed: state.total_programs_executed,
            successful_executions,
            failed_executions,
            total_execution_time: state.total_execution_time,
            average_execution_time,
            current_memory_usage: state.hardware_state.memory_usage,
            breakpoints_set: self.breakpoints.read().values().map(|v| v.len()).sum(),
        }
    }
}

/// Simulation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationStatistics {
    pub total_programs_executed: u64,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub total_execution_time: Duration,
    pub average_execution_time: Duration,
    pub current_memory_usage: usize,
    pub breakpoints_set: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_simulation() {
        let simulator = SteelProgramSimulator::new_default().unwrap();

        let code = r#"
            (begin
              (log "info" "Starting test program")
              (led-on)
              (sleep 0.1)
              (led-off)
              (log "info" "Test program completed"))
        "#;

        let result = simulator
            .simulate_program(code, "test-program")
            .await
            .unwrap();
        assert!(matches!(result.result, ExecutionResult::Success));
        assert!(result.duration.is_some());
        assert!(!result.debug_output.is_empty());
    }

    #[tokio::test]
    async fn test_validation_failure() {
        let simulator = SteelProgramSimulator::new_default().unwrap();

        let invalid_code = r#"
            (begin
              (undefined-function)
              (led-on)
        "#; // Missing closing parenthesis

        let result = simulator
            .simulate_program(invalid_code, "invalid-program")
            .await
            .unwrap();
        assert!(matches!(result.result, ExecutionResult::Error(_)));
        assert!(result.validation_result.is_some());
        assert!(!result.validation_result.as_ref().unwrap().is_valid);
    }

    #[tokio::test]
    async fn test_breakpoint_functionality() {
        let simulator = SteelProgramSimulator::new_default().unwrap();

        simulator.set_breakpoint("test-program", 3).unwrap();

        let breakpoints = simulator.list_breakpoints();
        assert!(breakpoints.get("test-program").unwrap().contains(&3));

        simulator.remove_breakpoint("test-program", 3).unwrap();
        let breakpoints = simulator.list_breakpoints();
        assert!(breakpoints.get("test-program").is_none_or(|v| v.is_empty()));
    }

    #[tokio::test]
    async fn test_simulation_state_tracking() {
        let simulator = SteelProgramSimulator::new_default().unwrap();

        let initial_state = simulator.get_simulation_state();
        assert!(!initial_state.is_running);
        assert_eq!(initial_state.total_programs_executed, 0);

        let code = "(led-on)";
        let _result = simulator
            .simulate_program(code, "state-test")
            .await
            .unwrap();

        let final_state = simulator.get_simulation_state();
        assert_eq!(final_state.total_programs_executed, 1);
        assert!(final_state.total_execution_time > Duration::from_secs(0));
    }

    #[test]
    fn test_environment_variables() {
        let simulator = SteelProgramSimulator::new_default().unwrap();

        simulator.set_environment_variable("TEST_VAR", "test_value");
        assert_eq!(
            simulator.get_environment_variable("TEST_VAR"),
            Some("test_value".to_string())
        );
        assert_eq!(simulator.get_environment_variable("NONEXISTENT"), None);
    }

    #[test]
    fn test_statistics() {
        let simulator = SteelProgramSimulator::new_default().unwrap();

        let stats = simulator.get_statistics();
        assert_eq!(stats.total_programs_executed, 0);
        assert_eq!(stats.successful_executions, 0);
        assert_eq!(stats.failed_executions, 0);
    }
}
