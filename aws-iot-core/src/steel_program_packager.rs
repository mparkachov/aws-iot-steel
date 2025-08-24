use crate::{
    steel_program_validator::{SteelProgramValidator, ValidationResult},
    SystemError, SystemResult,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Steel program package containing code, metadata, and deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteelProgramPackage {
    pub package_id: String,
    pub metadata: PackageMetadata,
    pub program_code: String,
    pub dependencies: Vec<Dependency>,
    pub deployment_config: DeploymentConfig,
    pub validation_result: Option<ValidationResult>,
    pub signature: Option<PackageSignature>,
    pub created_at: DateTime<Utc>,
    pub package_version: String,
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub minimum_runtime_version: String,
    pub target_platforms: Vec<String>,
    pub estimated_memory_usage: usize,
    pub estimated_execution_time: f64,
    pub security_level: SecurityLevel,
}

/// Package dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_requirement: String,
    pub optional: bool,
    pub features: Vec<String>,
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub auto_start: bool,
    pub restart_policy: RestartPolicy,
    pub timeout_seconds: Option<u64>,
    pub priority: Priority,
    pub resource_limits: ResourceLimits,
    pub environment_variables: HashMap<String, String>,
    pub deployment_strategy: DeploymentStrategy,
    pub rollback_config: RollbackConfig,
}

/// Package security level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityLevel {
    Safe,       // No security concerns
    Restricted, // Requires elevated permissions
    Privileged, // Requires admin approval
}

/// Restart policy for programs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    Never,
    OnFailure,
    Always,
    UnlessStopped,
}

/// Program execution priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Resource limits for program execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: Option<usize>,
    pub max_execution_time_seconds: Option<u64>,
    pub max_cpu_percentage: Option<f64>,
    pub max_storage_bytes: Option<usize>,
}

/// Deployment strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStrategy {
    Immediate,
    Scheduled {
        at: DateTime<Utc>,
    },
    Gradual {
        batch_size: usize,
        delay_seconds: u64,
    },
    BlueGreen,
    Canary {
        percentage: f64,
    },
}

/// Rollback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackConfig {
    pub enabled: bool,
    pub auto_rollback_on_failure: bool,
    pub health_check_timeout_seconds: u64,
    pub max_rollback_attempts: u32,
}

/// Package signature for security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSignature {
    pub algorithm: String,
    pub signature: String,
    pub public_key_id: String,
    pub signed_at: DateTime<Utc>,
}

/// Package build configuration
#[derive(Debug, Clone)]
pub struct PackageBuildConfig {
    pub validate_code: bool,
    pub sign_package: bool,
    pub compress_code: bool,
    pub include_debug_info: bool,
    pub target_platforms: Vec<String>,
    pub optimization_level: OptimizationLevel,
}

/// Code optimization level
#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
}

/// Package deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub deployment_id: String,
    pub package_id: String,
    pub target_devices: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: DeploymentStatus,
    pub results: HashMap<String, DeviceDeploymentResult>,
    pub rollback_info: Option<RollbackInfo>,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    RolledBack,
    Cancelled,
}

/// Per-device deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDeploymentResult {
    pub device_id: String,
    pub status: DeviceDeploymentStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub version_before: Option<String>,
    pub version_after: Option<String>,
}

/// Device deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceDeploymentStatus {
    Pending,
    Downloading,
    Installing,
    Validating,
    Running,
    Failed,
    RolledBack,
}

/// Rollback information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackInfo {
    pub triggered_at: DateTime<Utc>,
    pub reason: String,
    pub previous_version: String,
    pub affected_devices: Vec<String>,
}

/// Steel program packager and deployment manager
pub struct SteelProgramPackager {
    validator: SteelProgramValidator,
    build_config: PackageBuildConfig,
    signing_key: Option<String>, // In a real implementation, this would be a proper key
}

impl Default for PackageBuildConfig {
    fn default() -> Self {
        Self {
            validate_code: true,
            sign_package: false, // Disabled by default for development
            compress_code: true,
            include_debug_info: true,
            target_platforms: vec!["esp32".to_string(), "simulator".to_string()],
            optimization_level: OptimizationLevel::Basic,
        }
    }
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            auto_start: true,
            restart_policy: RestartPolicy::OnFailure,
            timeout_seconds: Some(30),
            priority: Priority::Normal,
            resource_limits: ResourceLimits {
                max_memory_bytes: Some(1024 * 1024),   // 1MB
                max_execution_time_seconds: Some(300), // 5 minutes
                max_cpu_percentage: Some(50.0),
                max_storage_bytes: Some(1024 * 10), // 10KB
            },
            environment_variables: HashMap::new(),
            deployment_strategy: DeploymentStrategy::Immediate,
            rollback_config: RollbackConfig {
                enabled: true,
                auto_rollback_on_failure: true,
                health_check_timeout_seconds: 60,
                max_rollback_attempts: 3,
            },
        }
    }
}

impl SteelProgramPackager {
    /// Create a new Steel program packager
    pub fn new(build_config: PackageBuildConfig) -> Self {
        Self {
            validator: SteelProgramValidator::new(),
            build_config,
            signing_key: None,
        }
    }

    /// Create a packager with default configuration
    pub fn new_default() -> Self {
        Self::new(PackageBuildConfig::default())
    }

    /// Set signing key for package signatures
    pub fn set_signing_key(&mut self, key: String) {
        self.signing_key = Some(key);
    }

    /// Create a Steel program package from source code
    pub fn create_package(
        &self,
        code: &str,
        metadata: PackageMetadata,
        deployment_config: Option<DeploymentConfig>,
    ) -> SystemResult<SteelProgramPackage> {
        info!("Creating Steel program package: {}", metadata.name);

        // Validate code if enabled
        let validation_result = if self.build_config.validate_code {
            match self.validator.validate(code) {
                Ok(result) => {
                    if !result.is_valid {
                        return Err(SystemError::Configuration(format!(
                            "Code validation failed: {} errors",
                            result.errors.len()
                        )));
                    }
                    Some(result)
                }
                Err(e) => {
                    return Err(SystemError::Configuration(format!(
                        "Validation error: {}",
                        e
                    )));
                }
            }
        } else {
            None
        };

        // Process code (optimization, compression, etc.)
        let processed_code = self.process_code(code)?;

        // Create package
        let package_id = Uuid::new_v4().to_string();
        let created_at = Utc::now();

        let mut package = SteelProgramPackage {
            package_id,
            metadata,
            program_code: processed_code,
            dependencies: Vec::new(), // TODO: Extract dependencies from code
            deployment_config: deployment_config.unwrap_or_default(),
            validation_result,
            signature: None,
            created_at,
            package_version: "1.0.0".to_string(),
        };

        // Sign package if enabled
        if self.build_config.sign_package {
            package.signature = self.sign_package(&package)?;
        }

        info!(
            "Package created successfully: {} ({})",
            package.metadata.name, package.package_id
        );
        Ok(package)
    }

    /// Create a package from a file
    pub fn create_package_from_file(
        &self,
        file_path: &Path,
        metadata: PackageMetadata,
        deployment_config: Option<DeploymentConfig>,
    ) -> SystemResult<SteelProgramPackage> {
        let code = std::fs::read_to_string(file_path)
            .map_err(|e| SystemError::Configuration(format!("Failed to read file: {}", e)))?;

        self.create_package(&code, metadata, deployment_config)
    }

    /// Save package to file
    pub fn save_package(
        &self,
        package: &SteelProgramPackage,
        output_path: &Path,
    ) -> SystemResult<()> {
        let package_json =
            serde_json::to_string_pretty(package).map_err(SystemError::Serialization)?;

        std::fs::write(output_path, package_json)
            .map_err(|e| SystemError::Configuration(format!("Failed to write package: {}", e)))?;

        info!("Package saved to: {}", output_path.display());
        Ok(())
    }

    /// Load package from file
    pub fn load_package(&self, package_path: &Path) -> SystemResult<SteelProgramPackage> {
        let package_json = std::fs::read_to_string(package_path)
            .map_err(|e| SystemError::Configuration(format!("Failed to read package: {}", e)))?;

        let package: SteelProgramPackage =
            serde_json::from_str(&package_json).map_err(SystemError::Serialization)?;

        // Verify signature if present
        if let Some(signature) = &package.signature {
            self.verify_package_signature(&package, signature)?;
        }

        info!(
            "Package loaded: {} ({})",
            package.metadata.name, package.package_id
        );
        Ok(package)
    }

    /// Validate a package
    pub fn validate_package(
        &self,
        package: &SteelProgramPackage,
    ) -> SystemResult<ValidationResult> {
        info!("Validating package: {}", package.metadata.name);

        // Validate the Steel code
        let validation_result = self.validator.validate(&package.program_code)?;

        // Additional package-level validations
        if package.metadata.name.is_empty() {
            return Err(SystemError::Configuration(
                "Package name cannot be empty".to_string(),
            ));
        }

        if package.metadata.version.is_empty() {
            return Err(SystemError::Configuration(
                "Package version cannot be empty".to_string(),
            ));
        }

        // Validate deployment configuration
        self.validate_deployment_config(&package.deployment_config)?;

        // Validate dependencies
        for dependency in &package.dependencies {
            self.validate_dependency(dependency)?;
        }

        info!(
            "Package validation completed: {}",
            if validation_result.is_valid {
                "PASSED"
            } else {
                "FAILED"
            }
        );
        Ok(validation_result)
    }

    /// Deploy a package to target devices
    pub async fn deploy_package(
        &self,
        package: &SteelProgramPackage,
        target_devices: Vec<String>,
    ) -> SystemResult<DeploymentResult> {
        info!(
            "Starting deployment of package {} to {} devices",
            package.metadata.name,
            target_devices.len()
        );

        let deployment_id = Uuid::new_v4().to_string();
        let started_at = Utc::now();

        let mut deployment_result = DeploymentResult {
            deployment_id: deployment_id.clone(),
            package_id: package.package_id.clone(),
            target_devices: target_devices.clone(),
            started_at,
            completed_at: None,
            status: DeploymentStatus::InProgress,
            results: HashMap::new(),
            rollback_info: None,
        };

        // Execute deployment strategy
        match &package.deployment_config.deployment_strategy {
            DeploymentStrategy::Immediate => {
                self.deploy_immediate(package, &target_devices, &mut deployment_result)
                    .await?;
            }
            DeploymentStrategy::Scheduled { at } => {
                self.deploy_scheduled(package, &target_devices, *at, &mut deployment_result)
                    .await?;
            }
            DeploymentStrategy::Gradual {
                batch_size,
                delay_seconds,
            } => {
                self.deploy_gradual(
                    package,
                    &target_devices,
                    *batch_size,
                    *delay_seconds,
                    &mut deployment_result,
                )
                .await?;
            }
            DeploymentStrategy::BlueGreen => {
                self.deploy_blue_green(package, &target_devices, &mut deployment_result)
                    .await?;
            }
            DeploymentStrategy::Canary { percentage } => {
                self.deploy_canary(
                    package,
                    &target_devices,
                    *percentage,
                    &mut deployment_result,
                )
                .await?;
            }
        }

        deployment_result.completed_at = Some(Utc::now());
        deployment_result.status = if deployment_result
            .results
            .values()
            .all(|r| matches!(r.status, DeviceDeploymentStatus::Running))
        {
            DeploymentStatus::Completed
        } else {
            DeploymentStatus::Failed
        };

        info!(
            "Deployment completed: {:?} ({})",
            deployment_result.status, deployment_id
        );
        Ok(deployment_result)
    }

    /// Rollback a deployment
    pub async fn rollback_deployment(
        &self,
        deployment_result: &mut DeploymentResult,
        reason: &str,
    ) -> SystemResult<()> {
        info!(
            "Rolling back deployment: {} - {}",
            deployment_result.deployment_id, reason
        );

        let rollback_info = RollbackInfo {
            triggered_at: Utc::now(),
            reason: reason.to_string(),
            previous_version: "unknown".to_string(), // Would be tracked in real implementation
            affected_devices: deployment_result.target_devices.clone(),
        };

        // Simulate rollback process
        for device_id in &deployment_result.target_devices {
            if let Some(device_result) = deployment_result.results.get_mut(device_id) {
                device_result.status = DeviceDeploymentStatus::RolledBack;
                device_result.completed_at = Some(Utc::now());
            }
        }

        deployment_result.status = DeploymentStatus::RolledBack;
        deployment_result.rollback_info = Some(rollback_info);

        info!(
            "Rollback completed for deployment: {}",
            deployment_result.deployment_id
        );
        Ok(())
    }

    /// Get deployment status
    pub fn get_deployment_status(&self, deployment_id: &str) -> SystemResult<DeploymentStatus> {
        // In a real implementation, this would query a deployment database
        // For now, return a placeholder
        info!("Querying deployment status for: {}", deployment_id);
        Ok(DeploymentStatus::Completed)
    }

    /// List all packages
    pub fn list_packages(&self, package_dir: &Path) -> SystemResult<Vec<PackageInfo>> {
        let mut packages = Vec::new();

        if !package_dir.exists() {
            return Ok(packages);
        }

        for entry in std::fs::read_dir(package_dir).map_err(|e| {
            SystemError::Configuration(format!("Failed to read package directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                SystemError::Configuration(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_package(&path) {
                    Ok(package) => {
                        packages.push(PackageInfo {
                            package_id: package.package_id,
                            name: package.metadata.name,
                            version: package.metadata.version,
                            created_at: package.created_at,
                            file_path: path,
                            size_bytes: package.program_code.len(),
                            is_signed: package.signature.is_some(),
                        });
                    }
                    Err(e) => {
                        warn!("Failed to load package from {}: {}", path.display(), e);
                    }
                }
            }
        }

        packages.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(packages)
    }

    // ========== Private Implementation Methods ==========

    /// Process code (optimization, compression, etc.)
    fn process_code(&self, code: &str) -> SystemResult<String> {
        let mut processed_code = code.to_string();

        // Apply optimizations based on level
        match self.build_config.optimization_level {
            OptimizationLevel::None => {
                // No optimization
            }
            OptimizationLevel::Basic => {
                // Basic optimizations: remove comments, extra whitespace
                processed_code = self.remove_comments(&processed_code);
                processed_code = self.normalize_whitespace(&processed_code);
            }
            OptimizationLevel::Aggressive => {
                // Aggressive optimizations: all basic + more
                processed_code = self.remove_comments(&processed_code);
                processed_code = self.normalize_whitespace(&processed_code);
                processed_code = self.optimize_expressions(&processed_code);
            }
        }

        // Compress if enabled
        if self.build_config.compress_code {
            // In a real implementation, this might use actual compression
            // For now, just remove extra whitespace
            processed_code = processed_code
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
        }

        Ok(processed_code)
    }

    /// Remove comments from Steel code
    fn remove_comments(&self, code: &str) -> String {
        code.lines()
            .map(|line| {
                if let Some(pos) = line.find(';') {
                    // Simple comment removal - doesn't handle strings with semicolons
                    line[..pos].trim_end()
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Normalize whitespace in Steel code
    fn normalize_whitespace(&self, code: &str) -> String {
        code.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Optimize Steel expressions (placeholder implementation)
    fn optimize_expressions(&self, code: &str) -> String {
        // This would contain actual Steel-specific optimizations
        // For now, just return the code as-is
        code.to_string()
    }

    /// Sign a package
    fn sign_package(
        &self,
        package: &SteelProgramPackage,
    ) -> SystemResult<Option<PackageSignature>> {
        if let Some(_signing_key) = &self.signing_key {
            // Create package hash
            let package_data = format!(
                "{}{}{}",
                package.package_id,
                package.program_code,
                package.created_at.to_rfc3339()
            );

            let mut hasher = Sha256::new();
            hasher.update(package_data.as_bytes());
            let hash = hasher.finalize();
            let signature = format!("{:x}", hash);

            Ok(Some(PackageSignature {
                algorithm: "SHA256".to_string(),
                signature,
                public_key_id: "dev-key-001".to_string(),
                signed_at: Utc::now(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Verify package signature
    fn verify_package_signature(
        &self,
        package: &SteelProgramPackage,
        signature: &PackageSignature,
    ) -> SystemResult<()> {
        // In a real implementation, this would verify the cryptographic signature
        // For now, just check that the signature exists and has the expected format
        if signature.algorithm != "SHA256" {
            return Err(SystemError::Configuration(
                "Unsupported signature algorithm".to_string(),
            ));
        }

        if signature.signature.is_empty() {
            return Err(SystemError::Configuration("Invalid signature".to_string()));
        }

        debug!("Package signature verified for: {}", package.metadata.name);
        Ok(())
    }

    /// Validate deployment configuration
    fn validate_deployment_config(&self, config: &DeploymentConfig) -> SystemResult<()> {
        // Validate resource limits
        if let Some(max_memory) = config.resource_limits.max_memory_bytes {
            if max_memory == 0 {
                return Err(SystemError::Configuration(
                    "Max memory must be greater than 0".to_string(),
                ));
            }
        }

        if let Some(max_time) = config.resource_limits.max_execution_time_seconds {
            if max_time == 0 {
                return Err(SystemError::Configuration(
                    "Max execution time must be greater than 0".to_string(),
                ));
            }
        }

        // Validate rollback configuration
        if config.rollback_config.enabled && config.rollback_config.max_rollback_attempts == 0 {
            return Err(SystemError::Configuration(
                "Max rollback attempts must be greater than 0 when rollback is enabled".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate a dependency
    fn validate_dependency(&self, dependency: &Dependency) -> SystemResult<()> {
        if dependency.name.is_empty() {
            return Err(SystemError::Configuration(
                "Dependency name cannot be empty".to_string(),
            ));
        }

        if dependency.version_requirement.is_empty() {
            return Err(SystemError::Configuration(
                "Dependency version requirement cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    // ========== Deployment Strategy Implementations ==========

    async fn deploy_immediate(
        &self,
        package: &SteelProgramPackage,
        target_devices: &[String],
        deployment_result: &mut DeploymentResult,
    ) -> SystemResult<()> {
        info!(
            "Executing immediate deployment to {} devices",
            target_devices.len()
        );

        for device_id in target_devices {
            let device_result = self.deploy_to_device(package, device_id).await?;
            deployment_result
                .results
                .insert(device_id.clone(), device_result);
        }

        Ok(())
    }

    async fn deploy_scheduled(
        &self,
        package: &SteelProgramPackage,
        target_devices: &[String],
        scheduled_time: DateTime<Utc>,
        deployment_result: &mut DeploymentResult,
    ) -> SystemResult<()> {
        info!("Scheduling deployment for {}", scheduled_time);

        // In a real implementation, this would schedule the deployment
        // For now, we'll simulate immediate deployment
        self.deploy_immediate(package, target_devices, deployment_result)
            .await
    }

    async fn deploy_gradual(
        &self,
        package: &SteelProgramPackage,
        target_devices: &[String],
        batch_size: usize,
        delay_seconds: u64,
        deployment_result: &mut DeploymentResult,
    ) -> SystemResult<()> {
        info!(
            "Executing gradual deployment: batch_size={}, delay={}s",
            batch_size, delay_seconds
        );

        for batch in target_devices.chunks(batch_size) {
            for device_id in batch {
                let device_result = self.deploy_to_device(package, device_id).await?;
                deployment_result
                    .results
                    .insert(device_id.clone(), device_result);
            }

            // Wait between batches
            if batch.len() == batch_size {
                tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
            }
        }

        Ok(())
    }

    async fn deploy_blue_green(
        &self,
        package: &SteelProgramPackage,
        target_devices: &[String],
        deployment_result: &mut DeploymentResult,
    ) -> SystemResult<()> {
        info!("Executing blue-green deployment");

        // In a real implementation, this would deploy to a staging environment first
        // For now, simulate immediate deployment
        self.deploy_immediate(package, target_devices, deployment_result)
            .await
    }

    async fn deploy_canary(
        &self,
        package: &SteelProgramPackage,
        target_devices: &[String],
        percentage: f64,
        deployment_result: &mut DeploymentResult,
    ) -> SystemResult<()> {
        info!("Executing canary deployment: {}%", percentage);

        let canary_count = ((target_devices.len() as f64) * (percentage / 100.0)).ceil() as usize;
        let canary_devices = &target_devices[..canary_count.min(target_devices.len())];

        // Deploy to canary devices first
        for device_id in canary_devices {
            let device_result = self.deploy_to_device(package, device_id).await?;
            deployment_result
                .results
                .insert(device_id.clone(), device_result);
        }

        // In a real implementation, we would monitor canary health before proceeding
        // For now, deploy to remaining devices
        let remaining_devices = &target_devices[canary_count..];
        for device_id in remaining_devices {
            let device_result = self.deploy_to_device(package, device_id).await?;
            deployment_result
                .results
                .insert(device_id.clone(), device_result);
        }

        Ok(())
    }

    /// Deploy package to a single device
    async fn deploy_to_device(
        &self,
        package: &SteelProgramPackage,
        device_id: &str,
    ) -> SystemResult<DeviceDeploymentResult> {
        let started_at = Utc::now();

        info!(
            "Deploying package {} to device {}",
            package.metadata.name, device_id
        );

        // Simulate deployment process
        let mut device_result = DeviceDeploymentResult {
            device_id: device_id.to_string(),
            status: DeviceDeploymentStatus::Downloading,
            started_at,
            completed_at: None,
            error_message: None,
            version_before: Some("1.0.0".to_string()),
            version_after: Some(package.metadata.version.clone()),
        };

        // Simulate deployment steps
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        device_result.status = DeviceDeploymentStatus::Installing;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        device_result.status = DeviceDeploymentStatus::Validating;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        device_result.status = DeviceDeploymentStatus::Running;
        device_result.completed_at = Some(Utc::now());

        info!("Successfully deployed to device: {}", device_id);
        Ok(device_result)
    }
}

/// Package information for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub package_id: String,
    pub name: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub file_path: PathBuf,
    pub size_bytes: usize,
    pub is_signed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_basic_package() {
        let packager = SteelProgramPackager::new_default();

        let code = r#"
            (define (test-function)
              (begin
                (led-on)
                (sleep 1)
                (led-off)))
        "#;

        let metadata = PackageMetadata {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test package".to_string()),
            author: Some("Test Author".to_string()),
            license: Some("MIT".to_string()),
            homepage: None,
            repository: None,
            keywords: vec!["test".to_string()],
            categories: vec!["example".to_string()],
            minimum_runtime_version: "1.0.0".to_string(),
            target_platforms: vec!["esp32".to_string()],
            estimated_memory_usage: 1024,
            estimated_execution_time: 1.0,
            security_level: SecurityLevel::Safe,
        };

        let package = packager.create_package(code, metadata, None).unwrap();

        assert_eq!(package.metadata.name, "test-package");
        assert_eq!(package.metadata.version, "1.0.0");
        assert!(!package.program_code.is_empty());
        assert!(package.validation_result.is_some());
        assert!(package.validation_result.as_ref().unwrap().is_valid);
    }

    #[test]
    fn test_package_validation() {
        let packager = SteelProgramPackager::new_default();

        let invalid_code = r#"
            (define (invalid-function
              (undefined-function))
        "#;

        let metadata = PackageMetadata {
            name: "invalid-package".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            license: None,
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            minimum_runtime_version: "1.0.0".to_string(),
            target_platforms: vec!["esp32".to_string()],
            estimated_memory_usage: 1024,
            estimated_execution_time: 1.0,
            security_level: SecurityLevel::Safe,
        };

        let result = packager.create_package(invalid_code, metadata, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_load_package() {
        let packager = SteelProgramPackager::new_default();
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path().join("test-package.json");

        let code = "(led-on)";
        let metadata = PackageMetadata {
            name: "save-load-test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            license: None,
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            minimum_runtime_version: "1.0.0".to_string(),
            target_platforms: vec!["esp32".to_string()],
            estimated_memory_usage: 1024,
            estimated_execution_time: 1.0,
            security_level: SecurityLevel::Safe,
        };

        let original_package = packager.create_package(code, metadata, None).unwrap();
        packager
            .save_package(&original_package, &package_path)
            .unwrap();

        let loaded_package = packager.load_package(&package_path).unwrap();

        assert_eq!(original_package.package_id, loaded_package.package_id);
        assert_eq!(original_package.metadata.name, loaded_package.metadata.name);
        assert_eq!(original_package.program_code, loaded_package.program_code);
    }

    #[tokio::test]
    async fn test_deployment() {
        let packager = SteelProgramPackager::new_default();

        let code = "(led-on)";
        let metadata = PackageMetadata {
            name: "deployment-test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            license: None,
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            minimum_runtime_version: "1.0.0".to_string(),
            target_platforms: vec!["esp32".to_string()],
            estimated_memory_usage: 1024,
            estimated_execution_time: 1.0,
            security_level: SecurityLevel::Safe,
        };

        let package = packager.create_package(code, metadata, None).unwrap();
        let target_devices = vec!["device-001".to_string(), "device-002".to_string()];

        let deployment_result = packager
            .deploy_package(&package, target_devices)
            .await
            .unwrap();

        assert_eq!(deployment_result.target_devices.len(), 2);
        assert!(matches!(
            deployment_result.status,
            DeploymentStatus::Completed
        ));
        assert_eq!(deployment_result.results.len(), 2);
    }

    #[test]
    fn test_code_optimization() {
        let config = PackageBuildConfig {
            optimization_level: OptimizationLevel::Basic,
            ..Default::default()
        };

        let packager = SteelProgramPackager::new(config);

        let code_with_comments = r#"
            ; This is a comment
            (define (test-function)  ; Another comment
              (begin
                (led-on)    ; Turn on LED
                (sleep 1)   ; Wait 1 second
                (led-off))) ; Turn off LED
        "#;

        let processed = packager.remove_comments(code_with_comments);
        assert!(!processed.contains(';'));

        let normalized = packager.normalize_whitespace(&processed);
        assert!(!normalized.contains("  ")); // No double spaces
    }
}
