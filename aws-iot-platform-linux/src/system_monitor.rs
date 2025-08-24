//! Linux system monitoring for CI/CD environments
use aws_iot_core::{PlatformError, PlatformResult};
use serde::{Deserialize, Serialize};
use std::fs;
use sysinfo::System;

/// Linux system monitoring for CI/CD environments
pub struct LinuxSystemMonitor {
    system: System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxCpuInfo {
    pub model: String,
    pub cores: usize,
    pub frequency_mhz: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxDiskInfo {
    pub mount_point: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f64,
}

impl LinuxSystemMonitor {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    pub fn get_cpu_info(&mut self) -> PlatformResult<LinuxCpuInfo> {
        self.system.refresh_cpu();

        // Get basic CPU info - simplified for compatibility
        let cpus = self.system.cpus();
        if cpus.is_empty() {
            return Err(PlatformError::DeviceInfo(
                "No CPU information available".to_string(),
            ));
        }

        let cpu = &cpus[0];
        Ok(LinuxCpuInfo {
            model: cpu.brand().to_string(),
            cores: cpus.len(),
            frequency_mhz: cpu.frequency(),
            usage_percent: cpu.cpu_usage(),
        })
    }

    pub fn get_disk_info(&mut self) -> PlatformResult<Vec<LinuxDiskInfo>> {
        // Simplified disk info - just return root filesystem info
        let mut disk_info = Vec::new();

        // Try to get root filesystem info
        if let Ok(_metadata) = fs::metadata("/") {
            disk_info.push(LinuxDiskInfo {
                mount_point: "/".to_string(),
                file_system: "unknown".to_string(),
                total_bytes: 0, // Would need statvfs for real implementation
                free_bytes: 0,
                used_bytes: 0,
                usage_percent: 0.0,
            });
        }

        Ok(disk_info)
    }

    pub fn get_load_average(&self) -> PlatformResult<(f64, f64, f64)> {
        let load_avg = System::load_average();
        Ok((load_avg.one, load_avg.five, load_avg.fifteen))
    }

    pub fn get_process_count(&mut self) -> PlatformResult<usize> {
        self.system.refresh_processes();
        Ok(self.system.processes().len())
    }

    pub fn get_kernel_version(&self) -> PlatformResult<String> {
        match System::kernel_version() {
            Some(version) => Ok(version),
            None => {
                // Try to read from /proc/version as fallback
                match fs::read_to_string("/proc/version") {
                    Ok(content) => {
                        let version = content
                            .split_whitespace()
                            .nth(2)
                            .unwrap_or("unknown")
                            .to_string();
                        Ok(version)
                    }
                    Err(_) => Ok("unknown".to_string()),
                }
            }
        }
    }

    pub fn get_os_info(&self) -> PlatformResult<String> {
        let name = System::name().unwrap_or_else(|| "Linux".to_string());
        let version = System::os_version().unwrap_or_else(|| "unknown".to_string());
        Ok(format!("{} {}", name, version))
    }

    pub fn get_environment_info(&self) -> PlatformResult<String> {
        let mut info = Vec::new();

        // Add CI/CD environment detection
        if std::env::var("CI").is_ok() {
            info.push("CI Environment".to_string());
        }

        if let Ok(runner) = std::env::var("GITHUB_ACTIONS") {
            if runner == "true" {
                info.push("GitHub Actions".to_string());
            }
        }

        if std::env::var("GITLAB_CI").is_ok() {
            info.push("GitLab CI".to_string());
        }

        if std::env::var("JENKINS_URL").is_ok() {
            info.push("Jenkins".to_string());
        }

        if info.is_empty() {
            info.push("Development Environment".to_string());
        }

        Ok(info.join(", "))
    }
}

impl Default for LinuxSystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_system_monitor_creation() {
        let _monitor = LinuxSystemMonitor::new();
        // Test passes if creation doesn't panic
    }

    #[test]
    fn test_get_load_average() {
        let monitor = LinuxSystemMonitor::new();
        let result = monitor.get_load_average();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_kernel_version() {
        let monitor = LinuxSystemMonitor::new();
        let result = monitor.get_kernel_version();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_os_info() {
        let monitor = LinuxSystemMonitor::new();
        let result = monitor.get_os_info();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_environment_info() {
        let monitor = LinuxSystemMonitor::new();
        let result = monitor.get_environment_info();
        assert!(result.is_ok());
    }
}
