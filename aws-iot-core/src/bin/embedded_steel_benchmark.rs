/// Embedded Steel Runtime Performance Benchmark
/// This binary runs performance tests for the embedded Steel runtime to validate
/// memory usage, execution time, and resource constraints

use aws_iot_core::{EmbeddedSteelRuntime, MemoryUsageStats, PlatformHAL, LedState, DeviceInfo, MemoryInfo, UptimeInfo, PlatformResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use clap::{Arg, Command};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, warn, error};

/// Mock HAL for benchmarking
struct BenchmarkHAL {
    led_state: Arc<Mutex<LedState>>,
    operation_count: Arc<Mutex<u64>>,
}

impl BenchmarkHAL {
    fn new() -> Self {
        Self {
            led_state: Arc::new(Mutex::new(LedState::Off)),
            operation_count: Arc::new(Mutex::new(0)),
        }
    }
    
    fn get_operation_count(&self) -> u64 {
        *self.operation_count.lock().unwrap()
    }
    
    fn increment_operations(&self) {
        *self.operation_count.lock().unwrap() += 1;
    }
}

#[async_trait]
impl PlatformHAL for BenchmarkHAL {
    async fn sleep(&self, _duration: Duration) -> PlatformResult<()> {
        self.increment_operations();
        // Don't actually sleep in benchmarks
        Ok(())
    }
    
    async fn set_led(&self, state: LedState) -> PlatformResult<()> {
        self.increment_operations();
        *self.led_state.lock().unwrap() = state;
        Ok(())
    }
    
    async fn get_led_state(&self) -> PlatformResult<LedState> {
        self.increment_operations();
        Ok(*self.led_state.lock().unwrap())
    }
    
    async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
        self.increment_operations();
        Ok(DeviceInfo {
            device_id: "benchmark-device".to_string(),
            platform: "Benchmark".to_string(),
            version: "1.0.0".to_string(),
            firmware_version: "1.0.0".to_string(),
            hardware_revision: Some("bench1".to_string()),
            serial_number: Some("bench123".to_string()),
        })
    }
    
    async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
        self.increment_operations();
        Ok(MemoryInfo {
            total_bytes: 256 * 1024,
            free_bytes: 128 * 1024,
            used_bytes: 128 * 1024,
            largest_free_block: 64 * 1024,
        })
    }
    
    async fn get_uptime(&self) -> PlatformResult<UptimeInfo> {
        self.increment_operations();
        Ok(UptimeInfo {
            uptime: Duration::from_secs(3600),
            boot_time: Utc::now() - chrono::Duration::seconds(3600),
        })
    }
    
    async fn store_secure_data(&self, _key: &str, _data: &[u8]) -> PlatformResult<()> {
        self.increment_operations();
        Ok(())
    }
    
    async fn load_secure_data(&self, _key: &str) -> PlatformResult<Option<Vec<u8>>> {
        self.increment_operations();
        Ok(None)
    }
    
    async fn delete_secure_data(&self, _key: &str) -> PlatformResult<bool> {
        self.increment_operations();
        Ok(false)
    }
    
    async fn list_secure_keys(&self) -> PlatformResult<Vec<String>> {
        self.increment_operations();
        Ok(Vec::new())
    }
    
    async fn initialize(&mut self) -> PlatformResult<()> {
        Ok(())
    }
    
    async fn shutdown(&mut self) -> PlatformResult<()> {
        Ok(())
    }
}

/// Benchmark results
#[derive(Debug)]
struct BenchmarkResults {
    test_name: String,
    execution_time: Duration,
    memory_stats: MemoryUsageStats,
    operations_per_second: f64,
    success_rate: f64,
}

impl BenchmarkResults {
    fn print_summary(&self) {
        println!("\n=== {} ===", self.test_name);
        println!("Execution Time: {:?}", self.execution_time);
        println!("Operations/sec: {:.2}", self.operations_per_second);
        println!("Success Rate: {:.1}%", self.success_rate * 100.0);
        println!("Memory Usage:");
        println!("  Heap Used: {} bytes", self.memory_stats.heap_used_bytes);
        println!("  Heap Peak: {} bytes", self.memory_stats.heap_peak_bytes);
        println!("  Stack Peak: {} bytes", self.memory_stats.stack_peak_bytes);
        println!("  Programs: {}", self.memory_stats.programs_loaded);
        println!("  Program Size: {} bytes", self.memory_stats.total_program_size);
    }
}

/// Benchmark suite for embedded Steel runtime
struct EmbeddedSteelBenchmark {
    runtime: EmbeddedSteelRuntime,
    hal: Arc<BenchmarkHAL>,
}

impl EmbeddedSteelBenchmark {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let hal = Arc::new(BenchmarkHAL::new());
        let runtime = EmbeddedSteelRuntime::new(Arc::clone(&hal) as Arc<dyn PlatformHAL>)?;
        
        Ok(Self { runtime, hal })
    }
    
    /// Benchmark simple LED operations
    async fn benchmark_led_operations(&mut self, iterations: u32) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        info!("Running LED operations benchmark with {} iterations", iterations);
        
        let start_time = Instant::now();
        let mut successes = 0;
        
        for i in 0..iterations {
            let program_code = if i % 2 == 0 { "(led-on)" } else { "(led-off)" };
            let program_name = format!("led_test_{}", i);
            
            match self.runtime.load_program(program_code, Some(&program_name)).await {
                Ok(handle) => {
                    match self.runtime.execute_program(handle).await {
                        Ok(_) => successes += 1,
                        Err(e) => warn!("Program execution failed: {}", e),
                    }
                    let _ = self.runtime.remove_program(handle).await;
                }
                Err(e) => warn!("Program loading failed: {}", e),
            }
        }
        
        let execution_time = start_time.elapsed();
        let memory_stats = self.runtime.get_memory_stats();
        let operations_per_second = iterations as f64 / execution_time.as_secs_f64();
        let success_rate = successes as f64 / iterations as f64;
        
        Ok(BenchmarkResults {
            test_name: "LED Operations".to_string(),
            execution_time,
            memory_stats,
            operations_per_second,
            success_rate,
        })
    }
    
    /// Benchmark sleep operations with various durations
    async fn benchmark_sleep_operations(&mut self, iterations: u32) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        info!("Running sleep operations benchmark with {} iterations", iterations);
        
        let start_time = Instant::now();
        let mut successes = 0;
        
        for i in 0..iterations {
            let duration = 0.001 + (i % 10) as f64 * 0.001; // 0.001 to 0.01 seconds
            let program_code = format!("(sleep {})", duration);
            let program_name = format!("sleep_test_{}", i);
            
            match self.runtime.load_program(&program_code, Some(&program_name)).await {
                Ok(handle) => {
                    match self.runtime.execute_program(handle).await {
                        Ok(_) => successes += 1,
                        Err(e) => warn!("Sleep program execution failed: {}", e),
                    }
                    let _ = self.runtime.remove_program(handle).await;
                }
                Err(e) => warn!("Sleep program loading failed: {}", e),
            }
        }
        
        let execution_time = start_time.elapsed();
        let memory_stats = self.runtime.get_memory_stats();
        let operations_per_second = iterations as f64 / execution_time.as_secs_f64();
        let success_rate = successes as f64 / iterations as f64;
        
        Ok(BenchmarkResults {
            test_name: "Sleep Operations".to_string(),
            execution_time,
            memory_stats,
            operations_per_second,
            success_rate,
        })
    }
    
    /// Benchmark blink LED pattern (optimized path)
    async fn benchmark_blink_operations(&mut self, iterations: u32) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        info!("Running blink operations benchmark with {} iterations", iterations);
        
        let start_time = Instant::now();
        let mut successes = 0;
        
        for i in 0..iterations {
            let blinks = 1 + (i % 5); // 1 to 5 blinks
            let delay = 0.01; // 10ms delay
            let program_code = format!("(blink-led {} {})", blinks, delay);
            let program_name = format!("blink_test_{}", i);
            
            match self.runtime.load_program(&program_code, Some(&program_name)).await {
                Ok(handle) => {
                    match self.runtime.execute_program(handle).await {
                        Ok(_) => successes += 1,
                        Err(e) => warn!("Blink program execution failed: {}", e),
                    }
                    let _ = self.runtime.remove_program(handle).await;
                }
                Err(e) => warn!("Blink program loading failed: {}", e),
            }
        }
        
        let execution_time = start_time.elapsed();
        let memory_stats = self.runtime.get_memory_stats();
        let operations_per_second = iterations as f64 / execution_time.as_secs_f64();
        let success_rate = successes as f64 / iterations as f64;
        
        Ok(BenchmarkResults {
            test_name: "Blink Operations".to_string(),
            execution_time,
            memory_stats,
            operations_per_second,
            success_rate,
        })
    }
    
    /// Benchmark system information queries
    async fn benchmark_system_info(&mut self, iterations: u32) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        info!("Running system info benchmark with {} iterations", iterations);
        
        let start_time = Instant::now();
        let mut successes = 0;
        
        let info_functions = ["(device-info)", "(memory-info)", "(uptime)"];
        
        for i in 0..iterations {
            let program_code = info_functions[i as usize % info_functions.len()];
            let program_name = format!("info_test_{}", i);
            
            match self.runtime.load_program(program_code, Some(&program_name)).await {
                Ok(handle) => {
                    match self.runtime.execute_program(handle).await {
                        Ok(_) => successes += 1,
                        Err(e) => warn!("Info program execution failed: {}", e),
                    }
                    let _ = self.runtime.remove_program(handle).await;
                }
                Err(e) => warn!("Info program loading failed: {}", e),
            }
        }
        
        let execution_time = start_time.elapsed();
        let memory_stats = self.runtime.get_memory_stats();
        let operations_per_second = iterations as f64 / execution_time.as_secs_f64();
        let success_rate = successes as f64 / iterations as f64;
        
        Ok(BenchmarkResults {
            test_name: "System Info".to_string(),
            execution_time,
            memory_stats,
            operations_per_second,
            success_rate,
        })
    }
    
    /// Benchmark memory limits and program capacity
    async fn benchmark_memory_limits(&mut self) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        info!("Running memory limits benchmark");
        
        let start_time = Instant::now();
        let mut programs_loaded = 0;
        let mut handles = Vec::new();
        
        // Load programs until we hit the limit
        loop {
            let program_code = format!("(begin (led-on) (sleep 0.001) (led-off))");
            let program_name = format!("memory_test_{}", programs_loaded);
            
            match self.runtime.load_program(&program_code, Some(&program_name)).await {
                Ok(handle) => {
                    handles.push(handle);
                    programs_loaded += 1;
                    
                    // Check if we're approaching limits
                    if programs_loaded >= 50 { // Safety limit for benchmark
                        break;
                    }
                }
                Err(_) => {
                    info!("Hit program loading limit at {} programs", programs_loaded);
                    break;
                }
            }
        }
        
        // Execute all loaded programs
        let mut successes = 0;
        for handle in &handles {
            match self.runtime.execute_program(*handle).await {
                Ok(_) => successes += 1,
                Err(e) => warn!("Memory test program execution failed: {}", e),
            }
        }
        
        // Clean up
        for handle in handles {
            let _ = self.runtime.remove_program(handle).await;
        }
        
        let execution_time = start_time.elapsed();
        let memory_stats = self.runtime.get_memory_stats();
        let operations_per_second = programs_loaded as f64 / execution_time.as_secs_f64();
        let success_rate = successes as f64 / programs_loaded as f64;
        
        Ok(BenchmarkResults {
            test_name: "Memory Limits".to_string(),
            execution_time,
            memory_stats,
            operations_per_second,
            success_rate,
        })
    }
    
    /// Run all benchmarks
    async fn run_all_benchmarks(&mut self, iterations: u32) -> Result<Vec<BenchmarkResults>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        
        // Run individual benchmarks
        results.push(self.benchmark_led_operations(iterations).await?);
        results.push(self.benchmark_sleep_operations(iterations).await?);
        results.push(self.benchmark_blink_operations(iterations / 4).await?); // Fewer iterations for blink
        results.push(self.benchmark_system_info(iterations).await?);
        results.push(self.benchmark_memory_limits().await?);
        
        Ok(results)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let matches = Command::new("Embedded Steel Runtime Benchmark")
        .version("1.0")
        .about("Performance benchmarks for embedded Steel runtime")
        .arg(
            Arg::new("iterations")
                .short('i')
                .long("iterations")
                .value_name("NUMBER")
                .help("Number of iterations for each benchmark")
                .default_value("100")
        )
        .arg(
            Arg::new("test")
                .short('t')
                .long("test")
                .value_name("TEST_NAME")
                .help("Run specific test (led, sleep, blink, info, memory, all)")
                .default_value("all")
        )
        .get_matches();
    
    let iterations: u32 = matches.get_one::<String>("iterations")
        .unwrap()
        .parse()
        .expect("Invalid iterations number");
    
    let test_name = matches.get_one::<String>("test").unwrap();
    
    info!("Starting embedded Steel runtime benchmarks");
    info!("Iterations: {}", iterations);
    info!("Test: {}", test_name);
    
    let mut benchmark = EmbeddedSteelBenchmark::new()?;
    
    let results = match test_name.as_str() {
        "led" => vec![benchmark.benchmark_led_operations(iterations).await?],
        "sleep" => vec![benchmark.benchmark_sleep_operations(iterations).await?],
        "blink" => vec![benchmark.benchmark_blink_operations(iterations).await?],
        "info" => vec![benchmark.benchmark_system_info(iterations).await?],
        "memory" => vec![benchmark.benchmark_memory_limits().await?],
        "all" => benchmark.run_all_benchmarks(iterations).await?,
        _ => {
            error!("Unknown test: {}", test_name);
            return Err("Invalid test name".into());
        }
    };
    
    // Print results
    println!("\n🚀 Embedded Steel Runtime Benchmark Results 🚀");
    println!("================================================");
    
    for result in &results {
        result.print_summary();
    }
    
    // Print overall summary
    let total_operations: f64 = results.iter().map(|r| r.operations_per_second).sum();
    let avg_success_rate: f64 = results.iter().map(|r| r.success_rate).sum::<f64>() / results.len() as f64;
    let max_memory_usage = results.iter().map(|r| r.memory_stats.heap_peak_bytes).max().unwrap_or(0);
    
    println!("\n=== Overall Summary ===");
    println!("Total Operations/sec: {:.2}", total_operations);
    println!("Average Success Rate: {:.1}%", avg_success_rate * 100.0);
    println!("Peak Memory Usage: {} bytes", max_memory_usage);
    println!("HAL Operations: {}", benchmark.hal.get_operation_count());
    
    // Performance assessment
    println!("\n=== Performance Assessment ===");
    if avg_success_rate > 0.95 {
        println!("✅ Success Rate: EXCELLENT ({:.1}%)", avg_success_rate * 100.0);
    } else if avg_success_rate > 0.90 {
        println!("⚠️  Success Rate: GOOD ({:.1}%)", avg_success_rate * 100.0);
    } else {
        println!("❌ Success Rate: POOR ({:.1}%)", avg_success_rate * 100.0);
    }
    
    #[cfg(target_arch = "riscv32")]
    {
        const EMBEDDED_MEMORY_LIMIT: usize = 32 * 1024; // 32KB
        if max_memory_usage <= EMBEDDED_MEMORY_LIMIT {
            println!("✅ Memory Usage: WITHIN LIMITS ({} / {} bytes)", max_memory_usage, EMBEDDED_MEMORY_LIMIT);
        } else {
            println!("❌ Memory Usage: EXCEEDS LIMITS ({} / {} bytes)", max_memory_usage, EMBEDDED_MEMORY_LIMIT);
        }
    }
    
    #[cfg(not(target_arch = "riscv32"))]
    {
        println!("ℹ️  Memory Usage: {} bytes (desktop target)", max_memory_usage);
    }
    
    if total_operations > 100.0 {
        println!("✅ Performance: EXCELLENT ({:.2} ops/sec)", total_operations);
    } else if total_operations > 50.0 {
        println!("⚠️  Performance: GOOD ({:.2} ops/sec)", total_operations);
    } else {
        println!("❌ Performance: POOR ({:.2} ops/sec)", total_operations);
    }
    
    info!("Embedded Steel runtime benchmarks completed successfully");
    Ok(())
}