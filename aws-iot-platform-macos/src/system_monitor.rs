use aws_iot_core::{PlatformResult, PlatformError, DeviceInfo, MemoryInfo, UptimeInfo};
use chrono::{DateTime, Utc};
// Duration is used in tests
use std::process::Command;
use std::collections::HashMap;

/// System monitoring utilities for macOS
pub struct MacOSSystemMonitor {
    boot_time: DateTime<Utc>,
    device_id: String,
}

impl MacOSSystemMonitor {
    pub fn new() -> Self {
        Self {
            boot_time: Utc::now(),
            device_id: Self::generate_device_id(),
        }
    }

    /// Generate a consistent device ID based on system information
    fn generate_device_id() -> String {
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown-macos-device".to_string());
            
        format!("macos-{}", 
            hostname.chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>()
                .to_lowercase())
    }

    /// Get comprehensive device information
    pub async fn get_device_info(&self) -> PlatformResult<DeviceInfo> {
        let os_version = self.get_macos_version().await?;
        let hardware_info = self.get_hardware_model().await?;
        let serial_number = self.get_system_serial().await.unwrap_or_else(|| {
            format!("SIM-{}", uuid::Uuid::new_v4().simple())
        });
        
        Ok(DeviceInfo {
            device_id: self.device_id.clone(),
            platform: format!("macOS {}", os_version),
            version: env!("CARGO_PKG_VERSION").to_string(),
            firmware_version: "1.0.0-simulator".to_string(),
            hardware_revision: Some(hardware_info),
            serial_number: Some(serial_number),
        })
    }

    /// Get macOS version information
    async fn get_macos_version(&self) -> PlatformResult<String> {
        let output = Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .map_err(|e| PlatformError::DeviceInfo(format!("Failed to get macOS version: {}", e)))?;
            
        if !output.status.success() {
            return Err(PlatformError::DeviceInfo("sw_vers command failed".to_string()));
        }
        
        let version = String::from_utf8(output.stdout)
            .map_err(|e| PlatformError::DeviceInfo(format!("Invalid UTF-8 in version output: {}", e)))?
            .trim()
            .to_string();
            
        Ok(version)
    }

    /// Get hardware model information
    async fn get_hardware_model(&self) -> PlatformResult<String> {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("hw.model")
            .output()
            .map_err(|e| PlatformError::DeviceInfo(format!("Failed to get hardware model: {}", e)))?;
            
        if !output.status.success() {
            return Ok("Unknown Mac".to_string());
        }
        
        let model = String::from_utf8(output.stdout)
            .map_err(|e| PlatformError::DeviceInfo(format!("Invalid UTF-8 in model output: {}", e)))?
            .trim()
            .to_string();
            
        Ok(model)
    }

    /// Get system serial number (if available)
    async fn get_system_serial(&self) -> Option<String> {
        let output = Command::new("system_profiler")
            .arg("SPHardwareDataType")
            .output()
            .ok()?;
            
        if !output.status.success() {
            return None;
        }
        
        let output_str = String::from_utf8(output.stdout).ok()?;
        
        // Parse the system profiler output to find serial number
        for line in output_str.lines() {
            if line.trim().starts_with("Serial Number") {
                if let Some(serial) = line.split(':').nth(1) {
                    return Some(serial.trim().to_string());
                }
            }
        }
        
        None
    }

    /// Get detailed memory information
    pub async fn get_memory_info(&self) -> PlatformResult<MemoryInfo> {
        let total_memory = self.get_total_memory().await?;
        let memory_stats = self.get_vm_stats().await?;
        
        let page_size = 4096u64; // macOS page size
        let free_pages = memory_stats.get("free").copied().unwrap_or(1_048_576); // 4GB fallback
        let inactive_pages = memory_stats.get("inactive").copied().unwrap_or(0);
        let speculative_pages = memory_stats.get("speculative").copied().unwrap_or(0);
        
        // Available memory includes free, inactive, and speculative pages
        let available_pages = free_pages + inactive_pages + speculative_pages;
        let free_bytes = available_pages * page_size;
        
        // Ensure free_bytes is not greater than total_memory
        let free_bytes = free_bytes.min(total_memory);
        let used_bytes = total_memory.saturating_sub(free_bytes);
        
        // Estimate largest free block as 75% of available memory
        let largest_free_block = (free_bytes as f64 * 0.75) as u64;
        
        Ok(MemoryInfo {
            total_bytes: total_memory,
            free_bytes,
            used_bytes,
            largest_free_block,
        })
    }

    /// Get total system memory
    async fn get_total_memory(&self) -> PlatformResult<u64> {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("hw.memsize")
            .output()
            .map_err(|e| PlatformError::DeviceInfo(format!("Failed to get total memory: {}", e)))?;
            
        if !output.status.success() {
            return Ok(8_589_934_592); // 8GB fallback
        }
        
        let memory_str = String::from_utf8(output.stdout)
            .map_err(|e| PlatformError::DeviceInfo(format!("Invalid UTF-8 in memory output: {}", e)))?;
            
        let total_memory = memory_str.trim().parse::<u64>()
            .map_err(|e| PlatformError::DeviceInfo(format!("Failed to parse memory size: {}", e)))?;
            
        Ok(total_memory)
    }

    /// Get VM statistics from vm_stat command
    async fn get_vm_stats(&self) -> PlatformResult<HashMap<String, u64>> {
        let output = Command::new("vm_stat")
            .output()
            .map_err(|e| PlatformError::DeviceInfo(format!("Failed to get VM stats: {}", e)))?;
            
        if !output.status.success() {
            // Return default values if vm_stat fails
            let mut stats = HashMap::new();
            stats.insert("free".to_string(), 1_048_576); // 4GB worth of pages
            return Ok(stats);
        }
        
        let vm_output = String::from_utf8(output.stdout)
            .map_err(|e| PlatformError::DeviceInfo(format!("Invalid UTF-8 in VM stats: {}", e)))?;
            
        let mut stats = HashMap::new();
        
        for line in vm_output.lines() {
            if let Some((key, value)) = self.parse_vm_stat_line(line) {
                stats.insert(key, value);
            }
        }
        
        Ok(stats)
    }

    /// Parse a single line from vm_stat output
    fn parse_vm_stat_line(&self, line: &str) -> Option<(String, u64)> {
        if line.contains("Pages") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let key = parts[1].trim_end_matches(':').to_lowercase();
                let value_str = parts[2].trim_end_matches('.');
                if let Ok(value) = value_str.parse::<u64>() {
                    return Some((key, value));
                }
            }
        }
        None
    }

    /// Get system uptime information
    pub async fn get_uptime(&self) -> PlatformResult<UptimeInfo> {
        let now = Utc::now();
        let uptime = now.signed_duration_since(self.boot_time)
            .to_std()
            .map_err(|e| PlatformError::DeviceInfo(format!("Invalid uptime calculation: {}", e)))?;
            
        Ok(UptimeInfo {
            uptime,
            boot_time: self.boot_time,
        })
    }

    /// Get system boot time from system information
    pub async fn get_system_boot_time(&self) -> PlatformResult<DateTime<Utc>> {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("kern.boottime")
            .output()
            .map_err(|e| PlatformError::DeviceInfo(format!("Failed to get boot time: {}", e)))?;
            
        if !output.status.success() {
            return Ok(self.boot_time);
        }
        
        let boot_output = String::from_utf8(output.stdout)
            .map_err(|e| PlatformError::DeviceInfo(format!("Invalid UTF-8 in boot time: {}", e)))?;
            
        // Parse boot time from format like "{ sec = 1640995200, usec = 0 }"
        if let Some(sec_start) = boot_output.find("sec = ") {
            let sec_str = &boot_output[sec_start + 6..];
            if let Some(sec_end) = sec_str.find(',') {
                let timestamp_str = &sec_str[..sec_end];
                if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                    if let Some(boot_time) = DateTime::from_timestamp(timestamp, 0) {
                        return Ok(boot_time);
                    }
                }
            }
        }
        
        // Fallback to initialization time
        Ok(self.boot_time)
    }

    /// Get CPU information
    pub async fn get_cpu_info(&self) -> PlatformResult<CpuInfo> {
        let cpu_count = self.get_cpu_count().await?;
        let cpu_model = self.get_cpu_model().await?;
        let cpu_frequency = self.get_cpu_frequency().await?;
        
        Ok(CpuInfo {
            model: cpu_model,
            cores: cpu_count,
            frequency_mhz: cpu_frequency,
        })
    }

    /// Get CPU core count
    async fn get_cpu_count(&self) -> PlatformResult<u32> {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("hw.ncpu")
            .output()
            .map_err(|e| PlatformError::DeviceInfo(format!("Failed to get CPU count: {}", e)))?;
            
        if !output.status.success() {
            return Ok(4); // Fallback to 4 cores
        }
        
        let cpu_str = String::from_utf8(output.stdout)
            .map_err(|e| PlatformError::DeviceInfo(format!("Invalid UTF-8 in CPU count: {}", e)))?;
            
        let cpu_count = cpu_str.trim().parse::<u32>()
            .map_err(|e| PlatformError::DeviceInfo(format!("Failed to parse CPU count: {}", e)))?;
            
        Ok(cpu_count)
    }

    /// Get CPU model name
    async fn get_cpu_model(&self) -> PlatformResult<String> {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
            .map_err(|e| PlatformError::DeviceInfo(format!("Failed to get CPU model: {}", e)))?;
            
        if !output.status.success() {
            return Ok("Unknown CPU".to_string());
        }
        
        let model = String::from_utf8(output.stdout)
            .map_err(|e| PlatformError::DeviceInfo(format!("Invalid UTF-8 in CPU model: {}", e)))?
            .trim()
            .to_string();
            
        Ok(model)
    }

    /// Get CPU frequency in MHz
    async fn get_cpu_frequency(&self) -> PlatformResult<u32> {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("hw.cpufrequency_max")
            .output()
            .map_err(|e| PlatformError::DeviceInfo(format!("Failed to get CPU frequency: {}", e)))?;
            
        if !output.status.success() {
            return Ok(2400); // Fallback to 2.4 GHz
        }
        
        let freq_str = String::from_utf8(output.stdout)
            .map_err(|e| PlatformError::DeviceInfo(format!("Invalid UTF-8 in CPU frequency: {}", e)))?;
            
        let freq_str = freq_str.trim();
        if freq_str.is_empty() {
            return Ok(2400); // Fallback if empty
        }
            
        let freq_hz = freq_str.parse::<u64>()
            .map_err(|e| PlatformError::DeviceInfo(format!("Failed to parse CPU frequency: {}", e)))?;
            
        // Convert Hz to MHz
        let freq_mhz = (freq_hz / 1_000_000) as u32;
        Ok(freq_mhz)
    }

    /// Get disk usage information
    pub async fn get_disk_info(&self) -> PlatformResult<DiskInfo> {
        let output = Command::new("df")
            .arg("-h")
            .arg("/")
            .output()
            .map_err(|e| PlatformError::DeviceInfo(format!("Failed to get disk info: {}", e)))?;
            
        if !output.status.success() {
            return Ok(DiskInfo {
                total_bytes: 500_000_000_000, // 500GB fallback
                free_bytes: 250_000_000_000,  // 250GB free
                used_bytes: 250_000_000_000,  // 250GB used
            });
        }
        
        let df_output = String::from_utf8(output.stdout)
            .map_err(|e| PlatformError::DeviceInfo(format!("Invalid UTF-8 in disk info: {}", e)))?;
            
        // Parse df output (skip header line)
        for line in df_output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let total = self.parse_disk_size(parts[1]).unwrap_or(500_000_000_000);
                let used = self.parse_disk_size(parts[2]).unwrap_or(250_000_000_000);
                let available = self.parse_disk_size(parts[3]).unwrap_or(250_000_000_000);
                
                return Ok(DiskInfo {
                    total_bytes: total,
                    free_bytes: available,
                    used_bytes: used,
                });
            }
        }
        
        // Fallback values
        Ok(DiskInfo {
            total_bytes: 500_000_000_000,
            free_bytes: 250_000_000_000,
            used_bytes: 250_000_000_000,
        })
    }

    /// Parse disk size from df output (e.g., "500Gi", "1.2Ti")
    fn parse_disk_size(&self, size_str: &str) -> Option<u64> {
        if size_str.is_empty() {
            return None;
        }
        
        let size_str = size_str.trim();
        let (number_part, unit) = if size_str.ends_with("Ti") {
            (&size_str[..size_str.len()-2], 1_099_511_627_776u64) // 1024^4
        } else if size_str.ends_with("Gi") {
            (&size_str[..size_str.len()-2], 1_073_741_824u64) // 1024^3
        } else if size_str.ends_with("Mi") {
            (&size_str[..size_str.len()-2], 1_048_576u64) // 1024^2
        } else if size_str.ends_with("Ki") {
            (&size_str[..size_str.len()-2], 1024u64)
        } else {
            (size_str, 1u64)
        };
        
        if let Ok(number) = number_part.parse::<f64>() {
            Some((number * unit as f64) as u64)
        } else {
            None
        }
    }
}

impl Default for MacOSSystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU information structure
#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub model: String,
    pub cores: u32,
    pub frequency_mhz: u32,
}

/// Disk information structure
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
}

impl DiskInfo {
    pub fn usage_percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.used_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::test;

    #[test]
    async fn test_system_monitor_creation() {
        let monitor = MacOSSystemMonitor::new();
        assert!(!monitor.device_id.is_empty());
        assert!(monitor.device_id.starts_with("macos-"));
    }

    #[test]
    async fn test_device_info() {
        let monitor = MacOSSystemMonitor::new();
        let device_info = monitor.get_device_info().await.expect("Failed to get device info");
        
        assert!(!device_info.device_id.is_empty());
        assert!(device_info.platform.contains("macOS"));
        assert!(!device_info.version.is_empty());
        assert!(device_info.hardware_revision.is_some());
        assert!(device_info.serial_number.is_some());
    }

    #[test]
    async fn test_memory_info() {
        let monitor = MacOSSystemMonitor::new();
        let memory_info = monitor.get_memory_info().await.expect("Failed to get memory info");
        
        assert!(memory_info.total_bytes > 0);
        assert!(memory_info.free_bytes > 0);
        assert!(memory_info.used_bytes > 0);
        assert!(memory_info.largest_free_block > 0);
        
        // Verify memory calculations
        assert_eq!(
            memory_info.total_bytes,
            memory_info.free_bytes + memory_info.used_bytes
        );
    }

    #[test]
    async fn test_uptime_info() {
        let monitor = MacOSSystemMonitor::new();
        
        // Wait a small amount to ensure uptime is measurable
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let uptime_info = monitor.get_uptime().await.expect("Failed to get uptime info");
        
        assert!(uptime_info.uptime > Duration::ZERO);
        assert!(uptime_info.boot_time <= Utc::now());
    }

    #[test]
    async fn test_cpu_info() {
        let monitor = MacOSSystemMonitor::new();
        let cpu_info = monitor.get_cpu_info().await.expect("Failed to get CPU info");
        
        assert!(!cpu_info.model.is_empty());
        assert!(cpu_info.cores > 0);
        assert!(cpu_info.frequency_mhz > 0);
    }

    #[test]
    async fn test_disk_info() {
        let monitor = MacOSSystemMonitor::new();
        let disk_info = monitor.get_disk_info().await.expect("Failed to get disk info");
        
        assert!(disk_info.total_bytes > 0);
        assert!(disk_info.free_bytes > 0);
        assert!(disk_info.used_bytes > 0);
        
        let usage_percentage = disk_info.usage_percentage();
        assert!(usage_percentage >= 0.0 && usage_percentage <= 100.0);
    }

    #[test]
    async fn test_parse_disk_size() {
        let monitor = MacOSSystemMonitor::new();
        
        assert_eq!(monitor.parse_disk_size("1Gi"), Some(1_073_741_824));
        assert_eq!(monitor.parse_disk_size("500Mi"), Some(524_288_000));
        assert_eq!(monitor.parse_disk_size("2Ti"), Some(2_199_023_255_552));
        assert_eq!(monitor.parse_disk_size("1024"), Some(1024));
        assert_eq!(monitor.parse_disk_size(""), None);
    }

    #[test]
    async fn test_parse_vm_stat_line() {
        let monitor = MacOSSystemMonitor::new();
        
        let result = monitor.parse_vm_stat_line("Pages free:                               123456.");
        assert_eq!(result, Some(("free".to_string(), 123456)));
        
        let result = monitor.parse_vm_stat_line("Pages inactive:                           789012.");
        assert_eq!(result, Some(("inactive".to_string(), 789012)));
        
        let result = monitor.parse_vm_stat_line("Invalid line format");
        assert_eq!(result, None);
    }
}