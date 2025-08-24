use aws_iot_core::{SystemResult, PlatformError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use sysinfo::{System, SystemExt, CpuExt, DiskExt};
use tracing::{debug, warn};

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
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub mount_point: String,
    pub file_system: String,
}

impl LinuxSystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self { system }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    pub fn get_cpu_info(&mut self) -> SystemResult<LinuxCpuInfo> {
        self.system.refresh_cpu();
        
        let cpus = self.system.cpus();
        if cpus.is_empty() {
            return Err(PlatformError::SystemInfo("No CPU information available".to_string()).into());
        }

        let cpu = &cpus[0];
        let model = cpu.brand().to_string();
        let cores = cpus.len();
        let frequency_mhz = cpu.frequency();
        let usage_percent = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cores as f32;

        Ok(LinuxCpuInfo {
            model,
            cores,
            frequency_mhz,
            usage_percent,
        })
    }

    pub fn get_disk_info(&mut self) -> SystemResult<Vec<LinuxDiskInfo>> {
        self.system.refresh_disks();
        
        let mut disk_info = Vec::new();
        
        for disk in self.system.disks() {
            let mount_point = disk.mount_point().to_string_lossy().to_string();
            let file_system = String::from_utf8_lossy(disk.file_system()).to_string();
            let total_bytes = disk.total_space();
            let free_bytes = disk.available_space();
            let used_bytes = total_bytes - free_bytes;

            disk_info.push(LinuxDiskInfo {
                total_bytes,
                free_bytes,
                used_bytes,
                mount_point,
                file_system,
            });
        }

        if disk_info.is_empty() {
            warn!("No disk information available");
        }

        Ok(disk_info)
    }

    pub fn get_load_average(&self) -> SystemResult<(f64, f64, f64)> {
        let load_avg = self.system.load_average();
        Ok((load_avg.one, load_avg.five, load_avg.fifteen))
    }

    pub fn get_process_count(&mut self) -> SystemResult<usize> {
        self.system.refresh_processes();
        Ok(self.system.processes().len())
    }

    pub fn get_kernel_version(&self) -> SystemResult<String> {
        match self.system.kernel_version() {
            Some(version) => Ok(version),
            None => {
                // Fallback to reading /proc/version
                match fs::read_to_string("/proc/version") {
                    Ok(content) => {
                        let version = content.split_whitespace()
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

    pub fn get_os_info(&self) -> SystemResult<String> {
        let name = self.system.name().unwrap_or_else(|| "Linux".to_string());
        let version = self.system.os_version().unwrap_or_else(|| "unknown".to_string());
        Ok(format!("{} {}", name, version))
    }

    pub fn is_container_environment(&self) -> bool {
        // Check for common container indicators
        Path::new("/.dockerenv").exists() ||
        std::env::var("container").is_ok() ||
        fs::read_to_string("/proc/1/cgroup")
            .map(|content| content.contains("docker") || content.contains("containerd"))
            .unwrap_or(false)
    }

    pub fn get_environment_info(&self) -> SystemResult<String> {
        let mut info = Vec::new();
        
        if self.is_container_environment() {
            info.push("Container");
        }
        
        if std::env::var("CI").is_ok() {
            info.push("CI");
        }
        
        if std::env::var("GITHUB_ACTIONS").is_ok() {
            info.push("GitHub Actions");
        }
        
        if info.is_empty() {
            info.push("Native Linux");
        }
        
        Ok(info.join(", "))
    }
}

impl Default for LinuxSystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxDiskInfo {
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

    #[test]
    fn test_system_monitor_creation() {
        let monitor = LinuxSystemMonitor::new();
        assert!(!monitor.system.cpus().is_empty());
    }

    #[test]
    fn test_cpu_info() {
        let mut monitor = LinuxSystemMonitor::new();
        let cpu_info = monitor.get_cpu_info().unwrap();
        
        assert!(!cpu_info.model.is_empty());
        assert!(cpu_info.cores > 0);
        assert!(cpu_info.usage_percent >= 0.0);
    }

    #[test]
    fn test_disk_info() {
        let mut monitor = LinuxSystemMonitor::new();
        let disk_info = monitor.get_disk_info().unwrap();
        
        // Should have at least one disk
        assert!(!disk_info.is_empty());
        
        for disk in &disk_info {
            assert!(disk.total_bytes > 0);
            assert!(!disk.mount_point.is_empty());
            assert!(disk.usage_percentage() >= 0.0);
            assert!(disk.usage_percentage() <= 100.0);
        }
    }

    #[test]
    fn test_load_average() {
        let monitor = LinuxSystemMonitor::new();
        let (one, five, fifteen) = monitor.get_load_average().unwrap();
        
        assert!(one >= 0.0);
        assert!(five >= 0.0);
        assert!(fifteen >= 0.0);
    }

    #[test]
    fn test_process_count() {
        let mut monitor = LinuxSystemMonitor::new();
        let count = monitor.get_process_count().unwrap();
        
        assert!(count > 0);
    }

    #[test]
    fn test_kernel_version() {
        let monitor = LinuxSystemMonitor::new();
        let version = monitor.get_kernel_version().unwrap();
        
        assert!(!version.is_empty());
        assert_ne!(version, "unknown");
    }

    #[test]
    fn test_os_info() {
        let monitor = LinuxSystemMonitor::new();
        let os_info = monitor.get_os_info().unwrap();
        
        assert!(!os_info.is_empty());
    }

    #[test]
    fn test_environment_info() {
        let monitor = LinuxSystemMonitor::new();
        let env_info = monitor.get_environment_info().unwrap();
        
        assert!(!env_info.is_empty());
    }
}