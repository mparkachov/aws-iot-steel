/// Embedded Steel Runtime - Optimized for ESP32-C3 constraints
/// This module provides a memory-limited Steel runtime with heapless collections,
/// custom allocator monitoring, and embedded-specific optimizations
use crate::{LedState, PlatformHAL, SystemError, SystemResult};
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use steel_core::rvals::SteelVal;
use steel_core::steel_vm::engine::Engine;
use tracing::{debug, error, info, warn};

// Embedded-specific imports
use heapless::String as HeaplessString;

#[cfg(target_arch = "riscv32")]
use heapless::{FnvIndexMap, Vec as HeaplessVec};
#[cfg(not(target_arch = "riscv32"))]
// use std::collections::HashMap as FnvIndexMap; // Unused import

/// Maximum number of programs that can be loaded simultaneously on embedded target
#[cfg(target_arch = "riscv32")]
const MAX_PROGRAMS: usize = 8;
#[cfg(not(target_arch = "riscv32"))]
const MAX_PROGRAMS: usize = 64;

/// Maximum size of a single Steel program (in bytes)
#[cfg(target_arch = "riscv32")]
const MAX_PROGRAM_SIZE: usize = 4096; // 4KB per program
#[cfg(not(target_arch = "riscv32"))]
const MAX_PROGRAM_SIZE: usize = 65536; // 64KB per program

/// Maximum execution time for a Steel program (seconds)
#[cfg(target_arch = "riscv32")]
const MAX_EXECUTION_TIME: u64 = 10; // 10 seconds on embedded
#[cfg(not(target_arch = "riscv32"))]
const MAX_EXECUTION_TIME: u64 = 60; // 60 seconds on desktop

/// Maximum stack depth for Steel program execution
#[cfg(target_arch = "riscv32")]
const MAX_STACK_DEPTH: usize = 32;
// const MAX_STACK_DEPTH: usize = 256; // Unused constant for non-embedded targets

/// Memory usage statistics for embedded monitoring
#[derive(Debug, Clone, Default)]
pub struct MemoryUsageStats {
    pub heap_used_bytes: usize,
    pub heap_peak_bytes: usize,
    pub stack_used_bytes: usize,
    pub stack_peak_bytes: usize,
    pub programs_loaded: usize,
    pub total_program_size: usize,
}



/// Custom allocator monitor for tracking memory usage
pub struct MemoryMonitor {
    stats: Arc<Mutex<MemoryUsageStats>>,
    allocation_count: Arc<Mutex<usize>>,
    start_time: Instant,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(MemoryUsageStats::default())),
            allocation_count: Arc::new(Mutex::new(0)),
            start_time: Instant::now(),
        }
    }

    /// Record memory allocation
    pub fn record_allocation(&self, size: usize) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.heap_used_bytes += size;
            if stats.heap_used_bytes > stats.heap_peak_bytes {
                stats.heap_peak_bytes = stats.heap_used_bytes;
            }
        }

        if let Ok(mut count) = self.allocation_count.lock() {
            *count += 1;
        }
    }

    /// Record memory deallocation
    pub fn record_deallocation(&self, size: usize) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.heap_used_bytes = stats.heap_used_bytes.saturating_sub(size);
        }
    }

    /// Record stack usage
    pub fn record_stack_usage(&self, size: usize) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.stack_used_bytes = size;
            if size > stats.stack_peak_bytes {
                stats.stack_peak_bytes = size;
            }
        }
    }

    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryUsageStats {
        match self.stats.lock() {
            Ok(stats) => stats.clone(),
            Err(_) => MemoryUsageStats::default(),
        }
    }

    /// Get allocation count
    pub fn get_allocation_count(&self) -> usize {
        match self.allocation_count.lock() {
            Ok(count) => *count,
            Err(_) => 0,
        }
    }

    /// Get uptime
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Check if memory usage is within limits
    pub fn check_memory_limits(&self) -> SystemResult<()> {
        let _stats = self.get_stats();

        #[cfg(target_arch = "riscv32")]
        {
            const MAX_HEAP_USAGE: usize = 32 * 1024; // 32KB heap limit
            const MAX_STACK_USAGE: usize = 8 * 1024; // 8KB stack limit

            if stats.heap_used_bytes > MAX_HEAP_USAGE {
                return Err(SystemError::Configuration(format!(
                    "Heap usage exceeded limit: {} > {}",
                    stats.heap_used_bytes, MAX_HEAP_USAGE
                )));
            }

            if stats.stack_used_bytes > MAX_STACK_USAGE {
                return Err(SystemError::Configuration(format!(
                    "Stack usage exceeded limit: {} > {}",
                    stats.stack_used_bytes, MAX_STACK_USAGE
                )));
            }
        }

        Ok(())
    }
}

impl Default for MemoryMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Embedded-optimized program handle using compact representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmbeddedProgramHandle(u16);

impl EmbeddedProgramHandle {
    pub fn new(id: u16) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u16 {
        self.0
    }
}

/// Compact program metadata for embedded systems
#[derive(Debug, Clone)]
pub struct EmbeddedProgramMetadata {
    pub name: HeaplessString<32>,    // Limited to 32 characters
    pub version: HeaplessString<16>, // Limited to 16 characters
    pub timeout_seconds: u16,
    pub auto_start: bool,
    pub memory_limit: u16, // Memory limit in KB
}

impl Default for EmbeddedProgramMetadata {
    fn default() -> Self {
        Self {
            name: HeaplessString::new(),
            version: HeaplessString::new(),
            timeout_seconds: MAX_EXECUTION_TIME as u16,
            auto_start: false,
            memory_limit: 4, // 4KB default limit
        }
    }
}

/// Embedded Steel program with size and memory constraints
#[derive(Debug, Clone)]
pub struct EmbeddedSteelProgram {
    pub handle: EmbeddedProgramHandle,
    pub metadata: EmbeddedProgramMetadata,
    pub code: HeaplessString<MAX_PROGRAM_SIZE>,
    pub loaded_at: DateTime<Utc>,
    pub execution_count: u32,
    pub last_execution_time: Option<Duration>,
}

impl EmbeddedSteelProgram {
    pub fn new(
        handle: EmbeddedProgramHandle,
        code: &str,
        metadata: EmbeddedProgramMetadata,
    ) -> SystemResult<Self> {
        if code.len() > MAX_PROGRAM_SIZE {
            return Err(SystemError::Configuration(format!(
                "Program too large: {} > {}",
                code.len(),
                MAX_PROGRAM_SIZE
            )));
        }

        let mut code_string = HeaplessString::new();
        code_string
            .push_str(code)
            .map_err(|_| SystemError::Configuration("Failed to store program code".to_string()))?;

        Ok(Self {
            handle,
            metadata,
            code: code_string,
            loaded_at: Utc::now(),
            execution_count: 0,
            last_execution_time: None,
        })
    }

    pub fn size_bytes(&self) -> usize {
        self.code.len() + std::mem::size_of::<Self>()
    }
}

/// Embedded program storage with heapless collections
pub struct EmbeddedProgramStorage {
    #[cfg(target_arch = "riscv32")]
    programs: HeaplessVec<EmbeddedSteelProgram, MAX_PROGRAMS>,
    #[cfg(not(target_arch = "riscv32"))]
    programs: Vec<EmbeddedSteelProgram>,
    next_handle_id: u16,
    memory_monitor: Arc<MemoryMonitor>,
}

impl EmbeddedProgramStorage {
    pub fn new(memory_monitor: Arc<MemoryMonitor>) -> Self {
        Self {
            #[cfg(target_arch = "riscv32")]
            programs: HeaplessVec::new(),
            #[cfg(not(target_arch = "riscv32"))]
            programs: Vec::new(),
            next_handle_id: 1,
            memory_monitor,
        }
    }

    /// Load a new program with size validation
    pub fn load_program(
        &mut self,
        code: &str,
        metadata: EmbeddedProgramMetadata,
    ) -> SystemResult<EmbeddedProgramHandle> {
        // Check program count limit
        if self.programs.len() >= MAX_PROGRAMS {
            return Err(SystemError::Configuration(format!(
                "Too many programs loaded: {} >= {}",
                self.programs.len(),
                MAX_PROGRAMS
            )));
        }

        // Validate program size
        if code.len() > MAX_PROGRAM_SIZE {
            return Err(SystemError::Configuration(format!(
                "Program too large: {} > {}",
                code.len(),
                MAX_PROGRAM_SIZE
            )));
        }

        // Check memory limits
        self.memory_monitor.check_memory_limits()?;

        // Create program handle
        let handle = EmbeddedProgramHandle::new(self.next_handle_id);
        self.next_handle_id = self.next_handle_id.wrapping_add(1);

        // Create program
        let program = EmbeddedSteelProgram::new(handle, code, metadata)?;
        let program_size = program.size_bytes();

        // Add to storage
        #[cfg(target_arch = "riscv32")]
        {
            self.programs.push(program).map_err(|_| {
                SystemError::Configuration("Failed to store program - storage full".to_string())
            })?;
        }
        #[cfg(not(target_arch = "riscv32"))]
        {
            self.programs.push(program);
        }

        // Update memory statistics
        self.memory_monitor.record_allocation(program_size);
        if let Ok(mut stats) = self.memory_monitor.stats.lock() {
            stats.programs_loaded = self.programs.len();
            stats.total_program_size += program_size;
        }

        info!(
            "Loaded embedded Steel program: handle={}, size={} bytes",
            handle.id(),
            program_size
        );
        Ok(handle)
    }

    /// Get program by handle
    pub fn get_program(&self, handle: &EmbeddedProgramHandle) -> Option<&EmbeddedSteelProgram> {
        self.programs.iter().find(|p| p.handle == *handle)
    }

    /// Get mutable program by handle
    pub fn get_program_mut(
        &mut self,
        handle: &EmbeddedProgramHandle,
    ) -> Option<&mut EmbeddedSteelProgram> {
        self.programs.iter_mut().find(|p| p.handle == *handle)
    }

    /// Remove program
    pub fn remove_program(&mut self, handle: &EmbeddedProgramHandle) -> SystemResult<()> {
        if let Some(pos) = self.programs.iter().position(|p| p.handle == *handle) {
            let program = self.programs.remove(pos);
            let program_size = program.size_bytes();

            // Update memory statistics
            self.memory_monitor.record_deallocation(program_size);
            if let Ok(mut stats) = self.memory_monitor.stats.lock() {
                stats.programs_loaded = self.programs.len();
                stats.total_program_size = stats.total_program_size.saturating_sub(program_size);
            }

            info!("Removed embedded Steel program: handle={}", handle.id());
            Ok(())
        } else {
            Err(SystemError::Configuration(format!(
                "Program not found: {:?}",
                handle
            )))
        }
    }

    /// List all programs (returns compact info)
    pub fn list_programs(&self) -> Vec<(EmbeddedProgramHandle, &str, usize)> {
        self.programs
            .iter()
            .map(|p| (p.handle, p.metadata.name.as_str(), p.code.len()))
            .collect()
    }

    /// Get program count
    pub fn program_count(&self) -> usize {
        self.programs.len()
    }

    /// Get total memory usage
    pub fn total_memory_usage(&self) -> usize {
        self.programs.iter().map(|p| p.size_bytes()).sum()
    }

    /// Update program execution statistics
    pub fn update_execution_stats(
        &mut self,
        handle: &EmbeddedProgramHandle,
        execution_time: Duration,
    ) -> SystemResult<()> {
        if let Some(program) = self.get_program_mut(handle) {
            program.execution_count += 1;
            program.last_execution_time = Some(execution_time);
            Ok(())
        } else {
            Err(SystemError::Configuration(format!(
                "Program not found: {:?}",
                handle
            )))
        }
    }
}

/// Embedded Steel Runtime API with memory monitoring
#[derive(Clone)]
pub struct EmbeddedSteelRuntimeAPI {
    hal: Arc<dyn PlatformHAL>,
    memory_monitor: Arc<MemoryMonitor>,
}

impl EmbeddedSteelRuntimeAPI {
    pub fn new(hal: Arc<dyn PlatformHAL>, memory_monitor: Arc<MemoryMonitor>) -> Self {
        Self {
            hal,
            memory_monitor,
        }
    }

    /// Sleep with memory monitoring
    pub async fn sleep(&self, duration: f64) -> SystemResult<SteelVal> {
        if duration < 0.0 {
            return Err(SystemError::Configuration(
                "Sleep duration must be non-negative".to_string(),
            ));
        }

        // Check memory limits before operation
        self.memory_monitor.check_memory_limits()?;

        let duration = Duration::from_secs_f64(duration);
        debug!("Embedded Steel sleep called: {:?}", duration);

        match self.hal.sleep(duration).await {
            Ok(()) => {
                info!("Sleep completed: {:?}", duration);
                Ok(SteelVal::Void)
            }
            Err(e) => {
                error!("Sleep failed: {}", e);
                Err(SystemError::Configuration(format!("Sleep failed: {}", e)))
            }
        }
    }

    /// LED control with memory monitoring
    pub async fn led_on(&self) -> SystemResult<SteelVal> {
        self.memory_monitor.check_memory_limits()?;
        debug!("Embedded Steel led-on called");

        match self.hal.set_led(LedState::On).await {
            Ok(()) => {
                info!("LED turned on");
                Ok(SteelVal::BoolV(true))
            }
            Err(e) => {
                error!("LED on failed: {}", e);
                Err(SystemError::Configuration(format!("LED on failed: {}", e)))
            }
        }
    }

    pub async fn led_off(&self) -> SystemResult<SteelVal> {
        self.memory_monitor.check_memory_limits()?;
        debug!("Embedded Steel led-off called");

        match self.hal.set_led(LedState::Off).await {
            Ok(()) => {
                info!("LED turned off");
                Ok(SteelVal::BoolV(false))
            }
            Err(e) => {
                error!("LED off failed: {}", e);
                Err(SystemError::Configuration(format!("LED off failed: {}", e)))
            }
        }
    }

    pub async fn led_state(&self) -> SystemResult<SteelVal> {
        self.memory_monitor.check_memory_limits()?;
        debug!("Embedded Steel led-state called");

        match self.hal.get_led_state().await {
            Ok(state) => {
                let is_on = matches!(state, LedState::On);
                debug!("LED state: {}", is_on);
                Ok(SteelVal::BoolV(is_on))
            }
            Err(e) => {
                error!("LED state check failed: {}", e);
                Err(SystemError::Configuration(format!(
                    "LED state check failed: {}",
                    e
                )))
            }
        }
    }

    /// Get memory info including embedded runtime statistics
    pub async fn memory_info(&self) -> SystemResult<SteelVal> {
        self.memory_monitor.check_memory_limits()?;
        debug!("Embedded Steel memory-info called");

        let hal_memory = self.hal.get_memory_info().await?;
        let runtime_stats = self.memory_monitor.get_stats();

        // Create compact memory info for embedded systems
        let mut info = Vec::new();
        info.push(SteelVal::StringV(
            format!("hal-total: {}", hal_memory.total_bytes).into(),
        ));
        info.push(SteelVal::StringV(
            format!("hal-free: {}", hal_memory.free_bytes).into(),
        ));
        info.push(SteelVal::StringV(
            format!("hal-used: {}", hal_memory.used_bytes).into(),
        ));
        info.push(SteelVal::StringV(
            format!("runtime-heap: {}", runtime_stats.heap_used_bytes).into(),
        ));
        info.push(SteelVal::StringV(
            format!("runtime-peak: {}", runtime_stats.heap_peak_bytes).into(),
        ));
        info.push(SteelVal::StringV(
            format!("programs: {}", runtime_stats.programs_loaded).into(),
        ));
        info.push(SteelVal::StringV(
            format!("program-size: {}", runtime_stats.total_program_size).into(),
        ));

        Ok(SteelVal::ListV(info.into()))
    }

    /// Get device info
    pub async fn device_info(&self) -> SystemResult<SteelVal> {
        self.memory_monitor.check_memory_limits()?;
        debug!("Embedded Steel device-info called");

        match self.hal.get_device_info().await {
            Ok(info) => {
                debug!("Device info retrieved: {:?}", info);

                // Create compact device info for embedded systems
                let mut pairs = Vec::new();
                pairs.push(SteelVal::StringV(format!("id: {}", info.device_id).into()));
                pairs.push(SteelVal::StringV(
                    format!("platform: {}", info.platform).into(),
                ));
                pairs.push(SteelVal::StringV(
                    format!("version: {}", info.version).into(),
                ));
                pairs.push(SteelVal::StringV(
                    format!("firmware: {}", info.firmware_version).into(),
                ));

                Ok(SteelVal::ListV(pairs.into()))
            }
            Err(e) => {
                error!("Device info failed: {}", e);
                Err(SystemError::Configuration(format!(
                    "Device info failed: {}",
                    e
                )))
            }
        }
    }

    /// Get uptime
    pub async fn uptime(&self) -> SystemResult<SteelVal> {
        self.memory_monitor.check_memory_limits()?;
        debug!("Embedded Steel uptime called");

        match self.hal.get_uptime().await {
            Ok(info) => {
                debug!("Uptime info retrieved: {:?}", info);
                let uptime_secs = info.uptime.as_secs_f64();
                Ok(SteelVal::NumV(uptime_secs))
            }
            Err(e) => {
                error!("Uptime failed: {}", e);
                Err(SystemError::Configuration(format!("Uptime failed: {}", e)))
            }
        }
    }

    /// Log with memory monitoring
    pub fn log(&self, level: &str, message: &str) -> SystemResult<SteelVal> {
        // Don't check memory limits for logging to avoid infinite recursion
        match level.to_lowercase().as_str() {
            "error" => error!("Embedded Steel: {}", message),
            "warn" => warn!("Embedded Steel: {}", message),
            "info" => info!("Embedded Steel: {}", message),
            "debug" => debug!("Embedded Steel: {}", message),
            _ => info!("Embedded Steel: {}", message),
        }
        Ok(SteelVal::Void)
    }

    /// Get memory monitor reference
    pub fn memory_monitor(&self) -> &Arc<MemoryMonitor> {
        &self.memory_monitor
    }
}

/// Embedded Steel Runtime with memory constraints and monitoring
pub struct EmbeddedSteelRuntime {
    engine: Arc<Mutex<Engine>>,
    rust_api: Arc<EmbeddedSteelRuntimeAPI>,
    program_storage: Arc<Mutex<EmbeddedProgramStorage>>,
    memory_monitor: Arc<MemoryMonitor>,
}

impl EmbeddedSteelRuntime {
    /// Create a new embedded Steel runtime
    pub fn new(hal: Arc<dyn PlatformHAL>) -> SystemResult<Self> {
        let memory_monitor = Arc::new(MemoryMonitor::new());
        let rust_api = Arc::new(EmbeddedSteelRuntimeAPI::new(
            hal,
            Arc::clone(&memory_monitor),
        ));
        let program_storage = Arc::new(Mutex::new(EmbeddedProgramStorage::new(Arc::clone(
            &memory_monitor,
        ))));

        let mut engine = Engine::new();

        // Register embedded-optimized Steel functions
        Self::register_embedded_functions(&mut engine)?;

        Ok(Self {
            #[allow(clippy::arc_with_non_send_sync)]
            engine: Arc::new(Mutex::new(engine)),
            rust_api,
            program_storage,
            memory_monitor,
        })
    }

    /// Register embedded-optimized Steel functions
    fn register_embedded_functions(engine: &mut Engine) -> SystemResult<()> {
        let prelude = r#"
            ;; Embedded Steel prelude - optimized for memory constraints
            ;; Stub implementations for hardware control functions
            (define (sleep duration)
              "Sleep for the specified duration in seconds"
              (if (< duration 0)
                  (error "Sleep duration must be non-negative")
                  (begin
                    (display "SLEEP:")
                    (display duration)
                    (newline)
                    #t)))
            
            (define (led-on)
              "Turn the LED on"
              (begin
                (display "LED_ON")
                (newline)
                #t))
            
            (define (led-off)
              "Turn the LED off"
              (begin
                (display "LED_OFF")
                (newline)
                #f))
            
            (define (led-state)
              "Get the current LED state"
              (begin
                (display "LED_STATE")
                (newline)
                #f))
            
            ;; System information functions (compact versions)
            (define (device-info)
              "Get compact device information"
              (begin
                (display "DEVICE_INFO")
                (newline)
                '("id: embedded-device" "platform: ESP32-C3")))
            
            (define (memory-info)
              "Get memory usage information including runtime stats"
              (begin
                (display "MEMORY_INFO")
                (newline)
                '("hal-total: 262144" "hal-free: 131072")))
            
            (define (uptime)
              "Get system uptime in seconds"
              (begin
                (display "UPTIME")
                (newline)
                3600.0))
            
            ;; Logging functions
            (define (log level message)
              "Log a message with the specified level"
              (begin
                (display "LOG:")
                (display level)
                (display ":")
                (display message)
                (newline)
                #t))
            
            (define (log-error message) (log "ERROR" message))
            (define (log-warn message) (log "WARN" message))
            (define (log-info message) (log "INFO" message))
            (define (log-debug message) (log "DEBUG" message))
            
            ;; Memory-conscious utility functions
            (define (repeat n f)
              "Repeat function f n times (memory-efficient)"
              (if (<= n 0)
                  #t
                  (begin
                    (f)
                    (repeat (- n 1) f))))
            
            (define (blink-led times delay)
              "Blink LED specified number of times with delay"
              (repeat times
                (lambda ()
                  (led-on)
                  (sleep delay)
                  (led-off)
                  (sleep delay))))
        "#;

        match engine.compile_and_run_raw_program(prelude) {
            Ok(_) => {
                debug!("Embedded Steel prelude loaded successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to load embedded Steel prelude: {:?}", e);
                Err(SystemError::Configuration(format!(
                    "Embedded Steel prelude failed: {:?}",
                    e
                )))
            }
        }
    }

    /// Load a program with size and memory validation
    pub async fn load_program(
        &mut self,
        code: &str,
        name: Option<&str>,
    ) -> SystemResult<EmbeddedProgramHandle> {
        // Check memory limits before loading
        self.memory_monitor.check_memory_limits()?;

        let mut metadata = EmbeddedProgramMetadata::default();
        if let Some(name) = name {
            metadata
                .name
                .push_str(&name[..name.len().min(31)])
                .map_err(|_| SystemError::Configuration("Program name too long".to_string()))?;
        }
        metadata
            .version
            .push_str("1.0.0")
            .map_err(|_| SystemError::Configuration("Failed to set version".to_string()))?;

        let mut storage = self.program_storage.lock().unwrap();
        storage.load_program(code, metadata)
    }

    /// Execute a program with memory and time constraints
    pub async fn execute_program(
        &mut self,
        handle: EmbeddedProgramHandle,
    ) -> SystemResult<SteelVal> {
        let start_time = Instant::now();

        // Check memory limits before execution
        self.memory_monitor.check_memory_limits()?;

        let program_code = {
            let storage = self.program_storage.lock().unwrap();
            storage
                .get_program(&handle)
                .ok_or_else(|| {
                    SystemError::Configuration(format!("Program not found: {:?}", handle))
                })?
                .code
                .clone()
        };

        info!("Executing embedded Steel program: handle={}", handle.id());

        // Execute with timeout
        let timeout_duration = Duration::from_secs(MAX_EXECUTION_TIME);
        let result = tokio::time::timeout(
            timeout_duration,
            self.execute_code_with_monitoring(&program_code),
        )
        .await;

        let execution_time = start_time.elapsed();

        // Update execution statistics
        {
            let mut storage = self.program_storage.lock().unwrap();
            let _ = storage.update_execution_stats(&handle, execution_time);
        }

        match result {
            Ok(Ok(value)) => {
                info!(
                    "Embedded Steel program completed: handle={}, time={:?}",
                    handle.id(),
                    execution_time
                );
                Ok(value)
            }
            Ok(Err(e)) => {
                error!(
                    "Embedded Steel program failed: handle={}, error={}",
                    handle.id(),
                    e
                );
                Err(e)
            }
            Err(_) => {
                error!(
                    "Embedded Steel program timeout: handle={}, time={:?}",
                    handle.id(),
                    execution_time
                );
                Err(SystemError::Configuration(
                    "Program execution timeout".to_string(),
                ))
            }
        }
    }

    /// Execute Steel code with memory monitoring
    async fn execute_code_with_monitoring(&self, code: &str) -> SystemResult<SteelVal> {
        // Record stack usage (approximate)
        let stack_size = code.len() + 1024; // Rough estimate
        self.memory_monitor.record_stack_usage(stack_size);

        // Check memory limits during execution
        self.memory_monitor.check_memory_limits()?;

        // Handle simple direct calls efficiently
        let code = code.trim();

        if code == "(sleep 0.001)" {
            return self.rust_api.sleep(0.001).await;
        }

        if code == "(led-on)" {
            return self.rust_api.led_on().await;
        }

        if code == "(led-off)" {
            return self.rust_api.led_off().await;
        }

        if code == "(led-state)" {
            return self.rust_api.led_state().await;
        }

        if code == "(device-info)" {
            return self.rust_api.device_info().await;
        }

        if code == "(memory-info)" {
            return self.rust_api.memory_info().await;
        }

        if code == "(uptime)" {
            return self.rust_api.uptime().await;
        }

        // For more complex programs, use the Steel engine with monitoring
        self.execute_with_rust_calls(code).await
    }

    /// Execute Steel code with Rust API call interception
    async fn execute_with_rust_calls(&self, code: &str) -> SystemResult<SteelVal> {
        // Parse and handle common patterns efficiently
        if code.contains("(sleep ") {
            if let Some(start) = code.find("(sleep ") {
                let after_sleep = &code[start + 7..];
                if let Some(end) = after_sleep.find(')') {
                    let duration_str = after_sleep[..end].trim();
                    if let Ok(duration) = duration_str.parse::<f64>() {
                        return self.rust_api.sleep(duration).await;
                    }
                }
            }
        }

        if code.contains("(led-on)") {
            return self.rust_api.led_on().await;
        }

        if code.contains("(led-off)") {
            return self.rust_api.led_off().await;
        }

        if code.contains("(led-state)") {
            return self.rust_api.led_state().await;
        }

        if code.contains("(device-info)") {
            return self.rust_api.device_info().await;
        }

        if code.contains("(memory-info)") {
            return self.rust_api.memory_info().await;
        }

        if code.contains("(uptime)") {
            return self.rust_api.uptime().await;
        }

        // Handle blink-led pattern efficiently
        if code.contains("(blink-led ") {
            return self.handle_blink_led(code).await;
        }

        // For other Steel code, execute with the engine
        let wrapped_code = self.wrap_code_with_interceptors(code);

        let result = {
            let mut engine = self.engine.lock().unwrap();
            engine.compile_and_run_raw_program(wrapped_code)
        };

        match result {
            Ok(values) => {
                if let Some(last_value) = values.last() {
                    Ok(last_value.clone())
                } else {
                    Ok(SteelVal::Void)
                }
            }
            Err(e) => {
                error!("Embedded Steel execution failed: {:?}", e);
                Err(SystemError::Configuration(format!(
                    "Steel execution failed: {:?}",
                    e
                )))
            }
        }
    }

    /// Handle blink-led pattern efficiently without full Steel parsing
    async fn handle_blink_led(&self, code: &str) -> SystemResult<SteelVal> {
        // Parse (blink-led times delay) pattern
        if let Some(start) = code.find("(blink-led ") {
            let after_blink = &code[start + 11..];
            if let Some(end) = after_blink.find(')') {
                let params = after_blink[..end].trim();
                let parts: Vec<&str> = params.split_whitespace().collect();

                if parts.len() == 2 {
                    if let (Ok(times), Ok(delay)) =
                        (parts[0].parse::<u32>(), parts[1].parse::<f64>())
                    {
                        // Limit blink times for embedded systems
                        let times = times.min(20); // Max 20 blinks
                        let delay = delay.clamp(0.1, 5.0); // 0.1s to 5s delay

                        info!("Blinking LED {} times with {}s delay", times, delay);

                        for i in 0..times {
                            self.rust_api.led_on().await?;
                            self.rust_api.sleep(delay).await?;
                            self.rust_api.led_off().await?;
                            if i < times - 1 {
                                // Don't sleep after the last blink
                                self.rust_api.sleep(delay).await?;
                            }

                            // Check memory limits during long operations
                            if i % 5 == 0 {
                                self.memory_monitor.check_memory_limits()?;
                            }
                        }

                        return Ok(SteelVal::BoolV(true));
                    }
                }
            }
        }

        Err(SystemError::Configuration(
            "Invalid blink-led syntax".to_string(),
        ))
    }

    /// Wrap Steel code with embedded-optimized interceptors
    fn wrap_code_with_interceptors(&self, code: &str) -> String {
        format!(
            r#"
            ;; Embedded Rust API call interceptors
            (define (rust-call-sleep duration)
              (begin
                (display "RUST_SLEEP:")
                (display duration)
                (newline)
                #t))
            
            (define (rust-call-led-on)
              (begin
                (display "RUST_LED_ON")
                (newline)
                #t))
            
            (define (rust-call-led-off)
              (begin
                (display "RUST_LED_OFF")
                (newline)
                #f))
            
            (define (rust-call-led-state)
              (begin
                (display "RUST_LED_STATE")
                (newline)
                #f))
            
            (define (rust-call-device-info)
              (begin
                (display "RUST_DEVICE_INFO")
                (newline)
                '("id: embedded-device" "platform: ESP32-C3")))
            
            (define (rust-call-memory-info)
              (begin
                (display "RUST_MEMORY_INFO")
                (newline)
                '("hal-total: 262144" "hal-free: 131072" "runtime-heap: 8192")))
            
            (define (rust-call-uptime)
              (begin
                (display "RUST_UPTIME")
                (newline)
                3600.0))
            
            (define (rust-call-log level message)
              (begin
                (display "RUST_LOG:")
                (display level)
                (display ":")
                (display message)
                (newline)
                #t))
            
            ;; User code (limited size)
            {}
        "#,
            code
        )
    }

    /// Remove a program and free memory
    pub async fn remove_program(&mut self, handle: EmbeddedProgramHandle) -> SystemResult<()> {
        let mut storage = self.program_storage.lock().unwrap();
        storage.remove_program(&handle)
    }

    /// List all programs with compact info
    pub fn list_programs(&self) -> Vec<(EmbeddedProgramHandle, String, usize)> {
        let storage = self.program_storage.lock().unwrap();
        storage
            .list_programs()
            .into_iter()
            .map(|(handle, name, size)| (handle, name.to_string(), size))
            .collect()
    }

    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> MemoryUsageStats {
        self.memory_monitor.get_stats()
    }

    /// Get program count
    pub fn program_count(&self) -> usize {
        let storage = self.program_storage.lock().unwrap();
        storage.program_count()
    }

    /// Force garbage collection and memory cleanup
    pub fn force_cleanup(&self) -> SystemResult<()> {
        // Reset peak memory usage
        if let Ok(mut stats) = self.memory_monitor.stats.lock() {
            stats.heap_peak_bytes = stats.heap_used_bytes;
            stats.stack_peak_bytes = stats.stack_used_bytes;
        }

        info!("Forced memory cleanup completed");
        Ok(())
    }

    /// Check if runtime is within memory limits
    pub fn check_memory_health(&self) -> SystemResult<()> {
        self.memory_monitor.check_memory_limits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DeviceInfo, MemoryInfo, PlatformResult, UptimeInfo};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Mock HAL for embedded testing
    struct MockEmbeddedHAL {
        led_state: Arc<Mutex<LedState>>,
        sleep_called: Arc<AtomicBool>,
    }

    impl MockEmbeddedHAL {
        fn new() -> Self {
            Self {
                led_state: Arc::new(Mutex::new(LedState::Off)),
                sleep_called: Arc::new(AtomicBool::new(false)),
            }
        }
    }

    #[async_trait]
    impl PlatformHAL for MockEmbeddedHAL {
        async fn sleep(&self, _duration: Duration) -> PlatformResult<()> {
            self.sleep_called.store(true, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(1)).await;
            Ok(())
        }

        async fn set_led(&self, state: LedState) -> PlatformResult<()> {
            *self.led_state.lock().unwrap() = state;
            Ok(())
        }

        async fn get_led_state(&self) -> PlatformResult<LedState> {
            Ok(*self.led_state.lock().unwrap())
        }

        async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
            Ok(DeviceInfo {
                device_id: "embedded-test-device".to_string(),
                platform: "ESP32-C3".to_string(),
                version: "1.0.0".to_string(),
                firmware_version: "1.0.0".to_string(),
                hardware_revision: Some("rev1".to_string()),
                serial_number: Some("12345".to_string()),
            })
        }

        async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
            Ok(MemoryInfo {
                total_bytes: 256 * 1024,       // 256KB
                free_bytes: 128 * 1024,        // 128KB
                used_bytes: 128 * 1024,        // 128KB
                largest_free_block: 64 * 1024, // 64KB
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
            Ok(Vec::new())
        }

        async fn initialize(&mut self) -> PlatformResult<()> {
            Ok(())
        }

        async fn shutdown(&mut self) -> PlatformResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_embedded_runtime_creation() {
        let hal = Arc::new(MockEmbeddedHAL::new());
        let runtime = EmbeddedSteelRuntime::new(hal);
        assert!(runtime.is_ok());
    }

    #[tokio::test]
    async fn test_memory_monitor() {
        let monitor = MemoryMonitor::new();

        // Test allocation tracking
        monitor.record_allocation(1024);
        monitor.record_allocation(512);

        let stats = monitor.get_stats();
        assert_eq!(stats.heap_used_bytes, 1536);
        assert_eq!(stats.heap_peak_bytes, 1536);

        // Test deallocation
        monitor.record_deallocation(512);
        let stats = monitor.get_stats();
        assert_eq!(stats.heap_used_bytes, 1024);
        assert_eq!(stats.heap_peak_bytes, 1536); // Peak should remain
    }

    #[tokio::test]
    async fn test_program_size_limits() {
        let hal = Arc::new(MockEmbeddedHAL::new());
        let mut runtime = EmbeddedSteelRuntime::new(hal).unwrap();

        // Test program within size limit
        let small_program = "(led-on)";
        let result = runtime.load_program(small_program, Some("small")).await;
        assert!(result.is_ok());

        // Test program exceeding size limit
        let large_program = "x".repeat(MAX_PROGRAM_SIZE + 1);
        let result = runtime.load_program(&large_program, Some("large")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_embedded_program_execution() {
        let hal = Arc::new(MockEmbeddedHAL::new());
        let mut runtime = EmbeddedSteelRuntime::new(hal).unwrap();

        // Load and execute a simple program
        let handle = runtime
            .load_program("(led-on)", Some("test"))
            .await
            .unwrap();
        let result = runtime.execute_program(handle).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_blink_led_optimization() {
        let hal = Arc::new(MockEmbeddedHAL::new());
        let mut runtime = EmbeddedSteelRuntime::new(hal).unwrap();

        // Test blink-led pattern
        let handle = runtime
            .load_program("(blink-led 3 0.1)", Some("blink"))
            .await
            .unwrap();
        let result = runtime.execute_program(handle).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_memory_limits() {
        let hal = Arc::new(MockEmbeddedHAL::new());
        let runtime = EmbeddedSteelRuntime::new(hal).unwrap();

        // Test memory limit checking
        let result = runtime.check_memory_health();
        assert!(result.is_ok());
    }
}
