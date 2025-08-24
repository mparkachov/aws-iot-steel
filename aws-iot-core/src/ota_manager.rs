use crate::error::{SecurityError, SystemError, SystemResult};
use crate::{IoTClientTrait, IoTResult, SecurityManager};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rumqttc::QoS;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{timeout, Duration};
use tracing::{error, info, warn};
use url::Url;

// Type alias to simplify complex type
type ProgressCallback = Arc<Mutex<Option<Box<dyn Fn(DownloadProgress) + Send + Sync>>>>;

/// Firmware update request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareUpdateRequest {
    pub request_id: String,
    pub firmware_version: String,
    pub compatibility_version: String,
    pub download_url: Option<String>,
    pub checksum: String,
    pub checksum_algorithm: String,
    pub size_bytes: u64,
    pub signature: String,
    pub public_key_id: String,
    pub metadata: Option<FirmwareMetadata>,
    pub requested_at: DateTime<Utc>,
}

/// Firmware metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareMetadata {
    pub description: Option<String>,
    pub release_notes: Option<String>,
    pub critical_update: bool,
    pub rollback_version: Option<String>,
    pub estimated_install_time_seconds: Option<u64>,
}

/// Firmware update status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FirmwareUpdateStatus {
    Requested,
    Validating,
    DownloadRequested,
    Downloading { progress_percent: f64 },
    Downloaded,
    Installing,
    Installed,
    ValidatingInstallation,
    Failed { error: String },
    RolledBack { reason: String },
}

/// Firmware update result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareUpdateResult {
    pub request_id: String,
    pub status: FirmwareUpdateStatus,
    pub current_version: String,
    pub target_version: String,
    pub progress_percent: f64,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Download progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub progress_percent: f64,
    pub download_speed_bps: f64,
    pub estimated_time_remaining_seconds: Option<u64>,
}

/// Firmware validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareValidationResult {
    pub is_valid: bool,
    pub checksum_verified: bool,
    pub signature_verified: bool,
    pub compatibility_verified: bool,
    pub size_verified: bool,
    pub error_messages: Vec<String>,
}

/// Pre-signed URL request for secure firmware download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreSignedUrlRequest {
    pub request_id: String,
    pub firmware_version: String,
    pub device_id: String,
    pub requested_at: DateTime<Utc>,
}

/// Pre-signed URL response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreSignedUrlResponse {
    pub request_id: String,
    pub download_url: String,
    pub expires_at: DateTime<Utc>,
    pub checksum: String,
    pub size_bytes: u64,
}

/// OTA manager trait for testability
#[async_trait]
pub trait OTAManagerTrait: Send + Sync {
    async fn initialize(&mut self) -> SystemResult<()>;
    async fn request_firmware_update(&self, version: &str) -> SystemResult<String>;
    async fn validate_firmware_request(
        &self,
        request: &FirmwareUpdateRequest,
    ) -> SystemResult<FirmwareValidationResult>;
    async fn request_pre_signed_url(&self, request: &PreSignedUrlRequest) -> SystemResult<()>;
    async fn download_firmware(
        &self,
        url: &str,
        expected_checksum: &str,
        expected_size: u64,
    ) -> SystemResult<Vec<u8>>;
    async fn install_firmware(
        &self,
        firmware_data: &[u8],
        request: &FirmwareUpdateRequest,
    ) -> SystemResult<()>;
    async fn rollback_firmware(&self, request_id: &str, reason: &str) -> SystemResult<()>;
    async fn validate_installation(&self, request_id: &str) -> SystemResult<bool>;
    async fn get_update_status(
        &self,
        request_id: &str,
    ) -> SystemResult<Option<FirmwareUpdateResult>>;
    async fn cancel_update(&self, request_id: &str) -> SystemResult<()>;
    fn set_progress_callback(&self, callback: Box<dyn Fn(DownloadProgress) + Send + Sync>);
}

/// Main OTA manager implementation
pub struct OTAManager {
    device_id: String,
    current_firmware_version: String,
    iot_client: Arc<dyn IoTClientTrait>,
    pub security_manager: Arc<SecurityManager>,
    pub active_updates: Arc<RwLock<HashMap<String, FirmwareUpdateResult>>>,
    pub progress_callback: ProgressCallback,
    http_client: reqwest::Client,
    download_timeout: Duration,
    max_download_retries: u32,
    pub rollback_data: Arc<RwLock<HashMap<String, Vec<u8>>>>, // Store previous firmware for rollback
}

impl OTAManager {
    /// Create a new OTA manager
    pub fn new(
        device_id: String,
        current_firmware_version: String,
        iot_client: Arc<dyn IoTClientTrait>,
        security_manager: Arc<SecurityManager>,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minute timeout for downloads
            .build()
            .expect("Failed to create HTTP client");

        Self {
            device_id,
            current_firmware_version,
            iot_client,
            security_manager,
            active_updates: Arc::new(RwLock::new(HashMap::new())),
            progress_callback: Arc::new(Mutex::new(None)),
            http_client,
            download_timeout: Duration::from_secs(1800), // 30 minutes
            max_download_retries: 3,
            rollback_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to OTA-related MQTT topics
    async fn subscribe_to_ota_topics(&self) -> IoTResult<()> {
        let topics = vec![
            format!("$aws/things/{}/jobs/notify", self.device_id),
            format!("$aws/things/{}/jobs/notify-next", self.device_id),
            format!("downloads/{}/firmware-url", self.device_id),
            format!("ota/{}/update-request", self.device_id),
            format!("ota/{}/status", self.device_id),
        ];

        for topic in topics {
            self.iot_client.subscribe(&topic, QoS::AtLeastOnce).await?;
            info!("Subscribed to OTA topic: {}", topic);
        }

        Ok(())
    }

    /// Get firmware update topic for device
    fn get_firmware_update_topic(&self, operation: &str) -> String {
        format!("ota/{}/{}", self.device_id, operation)
    }

    /// Validate firmware version compatibility
    pub fn validate_version_compatibility(
        &self,
        target_version: &str,
        compatibility_version: &str,
    ) -> bool {
        // Simple version comparison - in production this would be more sophisticated
        let current_parts: Vec<u32> = self
            .current_firmware_version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        let compatibility_parts: Vec<u32> = compatibility_version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        let target_parts: Vec<u32> = target_version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        // Check if current version meets compatibility requirements
        if current_parts.len() >= 2 && compatibility_parts.len() >= 2 {
            let current_major = current_parts[0];
            let current_minor = current_parts[1];
            let compat_major = compatibility_parts[0];
            let compat_minor = compatibility_parts[1];

            // Must be same major version and at least the required minor version
            if current_major == compat_major && current_minor >= compat_minor {
                // Target version should be newer than current
                if target_parts.len() >= 2 {
                    let target_major = target_parts[0];
                    let target_minor = target_parts[1];
                    return target_major > current_major
                        || (target_major == current_major && target_minor > current_minor);
                }
            }
        }

        false
    }

    /// Validate checksum of downloaded firmware
    pub fn validate_checksum(&self, data: &[u8], expected_checksum: &str, algorithm: &str) -> bool {
        match algorithm.to_lowercase().as_str() {
            "sha256" => {
                let computed_hash = self.security_manager.compute_sha256(data);
                let computed_hex = hex::encode(computed_hash);
                computed_hex.eq_ignore_ascii_case(expected_checksum)
            }
            _ => {
                warn!("Unsupported checksum algorithm: {}", algorithm);
                false
            }
        }
    }

    /// Update firmware update status
    async fn update_status(
        &self,
        request_id: &str,
        status: FirmwareUpdateStatus,
    ) -> SystemResult<()> {
        let mut updates = self.active_updates.write().await;
        if let Some(update) = updates.get_mut(request_id) {
            update.status = status.clone();

            // Set completion time for terminal states
            match status {
                FirmwareUpdateStatus::Installed
                | FirmwareUpdateStatus::Failed { .. }
                | FirmwareUpdateStatus::RolledBack { .. } => {
                    update.completed_at = Some(Utc::now());
                }
                _ => {}
            }

            // Report status to AWS IoT
            let status_topic = self.get_firmware_update_topic("status");
            let status_message = serde_json::to_vec(&update).map_err(SystemError::Serialization)?;

            self.iot_client
                .publish(&status_topic, &status_message, QoS::AtLeastOnce)
                .await
                .map_err(SystemError::IoT)?;

            info!("Updated firmware update status: {:?}", status);
        }

        Ok(())
    }

    /// Handle pre-signed URL response
    #[allow(dead_code)]
    async fn handle_pre_signed_url_response(
        &self,
        response: PreSignedUrlResponse,
    ) -> SystemResult<()> {
        info!(
            "Received pre-signed URL for request: {}",
            response.request_id
        );

        // Update status to downloading
        self.update_status(
            &response.request_id,
            FirmwareUpdateStatus::DownloadRequested,
        )
        .await?;

        // Start download in background
        let manager = self.clone();
        let response_clone = response.clone();
        tokio::spawn(async move {
            match manager
                .download_firmware(
                    &response_clone.download_url,
                    &response_clone.checksum,
                    response_clone.size_bytes,
                )
                .await
            {
                Ok(firmware_data) => {
                    info!(
                        "Firmware download completed for request: {}",
                        response_clone.request_id
                    );

                    // Update status to downloaded
                    if let Err(e) = manager
                        .update_status(&response_clone.request_id, FirmwareUpdateStatus::Downloaded)
                        .await
                    {
                        error!("Failed to update status to downloaded: {}", e);
                        return;
                    }

                    // Get the original request for installation
                    let updates = manager.active_updates.read().await;
                    if let Some(update_result) = updates.get(&response_clone.request_id) {
                        // Create a firmware request from the update result
                        let firmware_request = FirmwareUpdateRequest {
                            request_id: response_clone.request_id.clone(),
                            firmware_version: update_result.target_version.clone(),
                            compatibility_version: "1.0.0".to_string(), // Default compatibility
                            download_url: Some(response_clone.download_url),
                            checksum: response_clone.checksum,
                            checksum_algorithm: "sha256".to_string(),
                            size_bytes: response_clone.size_bytes,
                            signature: "".to_string(), // Would be provided in real implementation
                            public_key_id: "default".to_string(),
                            metadata: None,
                            requested_at: update_result.started_at,
                        };

                        // Install firmware
                        match manager
                            .install_firmware(&firmware_data, &firmware_request)
                            .await
                        {
                            Ok(()) => {
                                info!(
                                    "Firmware installation completed for request: {}",
                                    response_clone.request_id
                                );
                            }
                            Err(e) => {
                                error!("Firmware installation failed: {}", e);
                                let _ = manager
                                    .update_status(
                                        &response_clone.request_id,
                                        FirmwareUpdateStatus::Failed {
                                            error: e.to_string(),
                                        },
                                    )
                                    .await;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Firmware download failed: {}", e);
                    let _ = manager
                        .update_status(
                            &response_clone.request_id,
                            FirmwareUpdateStatus::Failed {
                                error: e.to_string(),
                            },
                        )
                        .await;
                }
            }
        });

        Ok(())
    }
}

// Implement Clone for OTAManager to enable spawning async tasks
impl Clone for OTAManager {
    fn clone(&self) -> Self {
        Self {
            device_id: self.device_id.clone(),
            current_firmware_version: self.current_firmware_version.clone(),
            iot_client: Arc::clone(&self.iot_client),
            security_manager: Arc::clone(&self.security_manager),
            active_updates: Arc::clone(&self.active_updates),
            progress_callback: Arc::clone(&self.progress_callback),
            http_client: self.http_client.clone(),
            download_timeout: self.download_timeout,
            max_download_retries: self.max_download_retries,
            rollback_data: Arc::clone(&self.rollback_data),
        }
    }
}

#[async_trait]
impl OTAManagerTrait for OTAManager {
    async fn initialize(&mut self) -> SystemResult<()> {
        info!("Initializing OTA manager for device: {}", self.device_id);

        // Subscribe to OTA topics
        self.subscribe_to_ota_topics()
            .await
            .map_err(SystemError::IoT)?;

        info!("OTA manager initialized successfully");
        Ok(())
    }

    async fn request_firmware_update(&self, version: &str) -> SystemResult<String> {
        let request_id = format!("fw-req-{}", uuid::Uuid::new_v4());

        info!(
            "Requesting firmware update to version: {} (request: {})",
            version, request_id
        );

        // Create firmware update request
        let update_request = FirmwareUpdateRequest {
            request_id: request_id.clone(),
            firmware_version: version.to_string(),
            compatibility_version: self.current_firmware_version.clone(),
            download_url: None,
            checksum: "".to_string(), // Will be provided by server
            checksum_algorithm: "sha256".to_string(),
            size_bytes: 0,             // Will be provided by server
            signature: "".to_string(), // Will be provided by server
            public_key_id: "default".to_string(),
            metadata: None,
            requested_at: Utc::now(),
        };

        // Validate the request
        let validation_result = self.validate_firmware_request(&update_request).await?;
        if !validation_result.is_valid {
            let error_msg = format!(
                "Firmware request validation failed: {:?}",
                validation_result.error_messages
            );
            return Err(SystemError::Configuration(error_msg));
        }

        // Create update result entry
        let update_result = FirmwareUpdateResult {
            request_id: request_id.clone(),
            status: FirmwareUpdateStatus::Requested,
            current_version: self.current_firmware_version.clone(),
            target_version: version.to_string(),
            progress_percent: 0.0,
            error_message: None,
            started_at: Utc::now(),
            completed_at: None,
        };

        // Store the update request
        self.active_updates
            .write()
            .await
            .insert(request_id.clone(), update_result);

        // Request pre-signed URL for download
        let url_request = PreSignedUrlRequest {
            request_id: request_id.clone(),
            firmware_version: version.to_string(),
            device_id: self.device_id.clone(),
            requested_at: Utc::now(),
        };

        self.request_pre_signed_url(&url_request).await?;

        Ok(request_id)
    }

    async fn validate_firmware_request(
        &self,
        request: &FirmwareUpdateRequest,
    ) -> SystemResult<FirmwareValidationResult> {
        let mut result = FirmwareValidationResult {
            is_valid: true,
            checksum_verified: false,
            signature_verified: false,
            compatibility_verified: false,
            size_verified: false,
            error_messages: Vec::new(),
        };

        // Validate version compatibility
        if !self.validate_version_compatibility(
            &request.firmware_version,
            &request.compatibility_version,
        ) {
            result.is_valid = false;
            result.error_messages.push(format!(
                "Version {} is not compatible with current version {}",
                request.firmware_version, self.current_firmware_version
            ));
        } else {
            result.compatibility_verified = true;
        }

        // Validate checksum format (if provided)
        if !request.checksum.is_empty() {
            match request.checksum_algorithm.as_str() {
                "sha256" => {
                    if request.checksum.len() == 64
                        && request.checksum.chars().all(|c| c.is_ascii_hexdigit())
                    {
                        result.checksum_verified = true;
                    } else {
                        result.is_valid = false;
                        result
                            .error_messages
                            .push("Invalid SHA256 checksum format".to_string());
                    }
                }
                _ => {
                    result.is_valid = false;
                    result.error_messages.push(format!(
                        "Unsupported checksum algorithm: {}",
                        request.checksum_algorithm
                    ));
                }
            }
        }

        // Validate signature (if provided)
        if !request.signature.is_empty() && !request.public_key_id.is_empty() {
            // In production, this would verify the actual signature
            result.signature_verified = true;
        }

        // Validate size (basic sanity check)
        if request.size_bytes > 0 {
            if request.size_bytes > 100 * 1024 * 1024 {
                // 100MB max
                result.is_valid = false;
                result
                    .error_messages
                    .push("Firmware size too large (max 100MB)".to_string());
            } else {
                result.size_verified = true;
            }
        }

        info!("Firmware request validation result: {:?}", result);
        Ok(result)
    }

    async fn request_pre_signed_url(&self, request: &PreSignedUrlRequest) -> SystemResult<()> {
        info!(
            "Requesting pre-signed URL for firmware download: {}",
            request.request_id
        );

        // Update status to validating
        self.update_status(&request.request_id, FirmwareUpdateStatus::Validating)
            .await?;

        // Publish request to AWS IoT (Lambda will respond with pre-signed URL)
        let request_topic = format!("downloads/{}/firmware-request", self.device_id);
        let request_message = serde_json::to_vec(request).map_err(SystemError::Serialization)?;

        self.iot_client
            .publish(&request_topic, &request_message, QoS::AtLeastOnce)
            .await
            .map_err(SystemError::IoT)?;

        info!("Pre-signed URL request sent for: {}", request.request_id);
        Ok(())
    }

    async fn download_firmware(
        &self,
        url: &str,
        expected_checksum: &str,
        expected_size: u64,
    ) -> SystemResult<Vec<u8>> {
        info!("Starting firmware download from: {}", url);

        // Validate URL
        let parsed_url = Url::parse(url)
            .map_err(|e| SystemError::Configuration(format!("Invalid download URL: {}", e)))?;

        if parsed_url.scheme() != "https" {
            return Err(SystemError::Configuration(
                "Download URL must use HTTPS".to_string(),
            ));
        }

        let mut retry_count = 0;
        let mut last_error = None;

        while retry_count < self.max_download_retries {
            match self
                .attempt_download(url, expected_checksum, expected_size)
                .await
            {
                Ok(data) => {
                    info!("Firmware download completed successfully");
                    return Ok(data);
                }
                Err(e) => {
                    retry_count += 1;
                    last_error = Some(e);

                    if retry_count < self.max_download_retries {
                        warn!("Download attempt {} failed, retrying...", retry_count);
                        tokio::time::sleep(Duration::from_secs(retry_count as u64 * 2)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| SystemError::Configuration("Download failed".to_string())))
    }

    async fn install_firmware(
        &self,
        firmware_data: &[u8],
        request: &FirmwareUpdateRequest,
    ) -> SystemResult<()> {
        info!(
            "Starting firmware installation for request: {}",
            request.request_id
        );

        // Update status to installing
        self.update_status(&request.request_id, FirmwareUpdateStatus::Installing)
            .await?;

        // Verify signature before installation
        if !request.signature.is_empty() {
            let signature_valid = self
                .security_manager
                .verify_program_signature(
                    firmware_data,
                    request.signature.as_bytes(),
                    &request.public_key_id,
                )
                .map_err(SystemError::Security)?;

            if !signature_valid {
                let error_msg = "Firmware signature verification failed";
                self.update_status(
                    &request.request_id,
                    FirmwareUpdateStatus::Failed {
                        error: error_msg.to_string(),
                    },
                )
                .await?;
                return Err(SystemError::Security(SecurityError::Authentication(
                    error_msg.to_string(),
                )));
            }
        }

        // Validate firmware size
        if firmware_data.len() != request.size_bytes as usize {
            let error_msg = format!(
                "Firmware size mismatch: expected {}, got {}",
                request.size_bytes,
                firmware_data.len()
            );
            self.update_status(
                &request.request_id,
                FirmwareUpdateStatus::Failed {
                    error: error_msg.clone(),
                },
            )
            .await?;
            return Err(SystemError::Configuration(error_msg));
        }

        // Store current firmware for rollback (simulate reading current firmware)
        let current_firmware = self.get_current_firmware().await?;
        self.rollback_data
            .write()
            .await
            .insert(request.request_id.clone(), current_firmware);

        // In a real implementation, this would:
        // 1. Write firmware to a staging area
        // 2. Verify the firmware integrity
        // 3. Set up rollback capability
        // 4. Apply the firmware update
        // 5. Verify the update was successful

        // Simulate installation time
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Update status to validating installation
        self.update_status(
            &request.request_id,
            FirmwareUpdateStatus::ValidatingInstallation,
        )
        .await?;

        // Validate the installation
        let validation_successful = self.perform_validation(&request.request_id).await?;

        if validation_successful {
            // Update status to installed
            self.update_status(&request.request_id, FirmwareUpdateStatus::Installed)
                .await?;
            info!(
                "Firmware installation completed successfully for request: {}",
                request.request_id
            );
        } else {
            // Installation validation failed, rollback
            warn!(
                "Firmware installation validation failed, rolling back: {}",
                request.request_id
            );
            self.perform_rollback(&request.request_id, "Installation validation failed")
                .await?;
        }

        Ok(())
    }

    async fn get_update_status(
        &self,
        request_id: &str,
    ) -> SystemResult<Option<FirmwareUpdateResult>> {
        let updates = self.active_updates.read().await;
        Ok(updates.get(request_id).cloned())
    }

    async fn cancel_update(&self, request_id: &str) -> SystemResult<()> {
        info!("Cancelling firmware update: {}", request_id);

        let mut updates = self.active_updates.write().await;
        if let Some(update) = updates.get_mut(request_id) {
            match &update.status {
                FirmwareUpdateStatus::Installed
                | FirmwareUpdateStatus::Failed { .. }
                | FirmwareUpdateStatus::RolledBack { .. } => {
                    return Err(SystemError::Configuration(
                        "Cannot cancel completed update".to_string(),
                    ));
                }
                FirmwareUpdateStatus::Installing | FirmwareUpdateStatus::ValidatingInstallation => {
                    return Err(SystemError::Configuration(
                        "Cannot cancel update during installation".to_string(),
                    ));
                }
                _ => {
                    update.status = FirmwareUpdateStatus::Failed {
                        error: "Cancelled by user".to_string(),
                    };
                    update.completed_at = Some(Utc::now());
                }
            }
        } else {
            return Err(SystemError::Configuration(
                "Update request not found".to_string(),
            ));
        }

        info!("Firmware update cancelled: {}", request_id);
        Ok(())
    }

    async fn rollback_firmware(&self, request_id: &str, reason: &str) -> SystemResult<()> {
        self.perform_rollback(request_id, reason).await
    }

    async fn validate_installation(&self, request_id: &str) -> SystemResult<bool> {
        self.perform_validation(request_id).await
    }

    fn set_progress_callback(&self, callback: Box<dyn Fn(DownloadProgress) + Send + Sync>) {
        if let Ok(mut cb) = self.progress_callback.try_lock() {
            *cb = Some(callback);
        }
    }
}

impl OTAManager {
    /// Attempt to download firmware with progress tracking
    async fn attempt_download(
        &self,
        url: &str,
        expected_checksum: &str,
        expected_size: u64,
    ) -> SystemResult<Vec<u8>> {
        let response = timeout(self.download_timeout, self.http_client.get(url).send())
            .await
            .map_err(|_| SystemError::Configuration("Download timeout".to_string()))?
            .map_err(|e| SystemError::Configuration(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(SystemError::Configuration(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        // Check content length
        let content_length = response.content_length().unwrap_or(0);
        if content_length != expected_size {
            return Err(SystemError::Configuration(format!(
                "Content length mismatch: expected {}, got {}",
                expected_size, content_length
            )));
        }

        let mut downloaded_bytes = 0u64;
        let mut firmware_data = Vec::with_capacity(expected_size as usize);
        let start_time = std::time::Instant::now();

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| SystemError::Configuration(format!("Download chunk error: {}", e)))?;

            firmware_data.extend_from_slice(&chunk);
            downloaded_bytes += chunk.len() as u64;

            // Calculate progress and speed
            let elapsed = start_time.elapsed().as_secs_f64();
            let progress_percent = (downloaded_bytes as f64 / expected_size as f64) * 100.0;
            let download_speed = if elapsed > 0.0 {
                downloaded_bytes as f64 / elapsed
            } else {
                0.0
            };

            let estimated_remaining = if download_speed > 0.0 {
                Some(((expected_size - downloaded_bytes) as f64 / download_speed) as u64)
            } else {
                None
            };

            // Report progress
            let progress = DownloadProgress {
                bytes_downloaded: downloaded_bytes,
                total_bytes: expected_size,
                progress_percent,
                download_speed_bps: download_speed,
                estimated_time_remaining_seconds: estimated_remaining,
            };

            if let Ok(callback) = self.progress_callback.try_lock() {
                if let Some(ref cb) = *callback {
                    cb(progress);
                }
            }
        }

        // Verify final size
        if firmware_data.len() != expected_size as usize {
            return Err(SystemError::Configuration(format!(
                "Downloaded size mismatch: expected {}, got {}",
                expected_size,
                firmware_data.len()
            )));
        }

        // Verify checksum
        if !self.validate_checksum(&firmware_data, expected_checksum, "sha256") {
            return Err(SystemError::Configuration(
                "Checksum verification failed".to_string(),
            ));
        }

        Ok(firmware_data)
    }

    /// Get current firmware data (simulation)
    async fn get_current_firmware(&self) -> SystemResult<Vec<u8>> {
        // In a real implementation, this would read the current firmware from flash
        // For simulation, we'll return mock data
        Ok(format!("current_firmware_v{}", self.current_firmware_version).into_bytes())
    }

    async fn perform_rollback(&self, request_id: &str, reason: &str) -> SystemResult<()> {
        info!(
            "Rolling back firmware for request: {} (reason: {})",
            request_id, reason
        );

        // Get rollback data
        let rollback_data = self.rollback_data.read().await;
        let previous_firmware = rollback_data
            .get(request_id)
            .ok_or_else(|| SystemError::Configuration("No rollback data available".to_string()))?;

        // In a real implementation, this would:
        // 1. Stop the current firmware
        // 2. Restore the previous firmware from backup
        // 3. Restart with the previous firmware
        // 4. Verify the rollback was successful

        // Simulate rollback time
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Update status to rolled back
        self.update_status(
            request_id,
            FirmwareUpdateStatus::RolledBack {
                reason: reason.to_string(),
            },
        )
        .await?;

        info!(
            "Firmware rollback completed for request: {} (restored {} bytes)",
            request_id,
            previous_firmware.len()
        );
        Ok(())
    }

    async fn perform_validation(&self, request_id: &str) -> SystemResult<bool> {
        info!(
            "Validating firmware installation for request: {}",
            request_id
        );

        // In a real implementation, this would:
        // 1. Check if the new firmware boots correctly
        // 2. Verify system functionality
        // 3. Check firmware version and integrity
        // 4. Run basic system tests

        // Simulate validation time
        tokio::time::sleep(Duration::from_secs(2)).await;

        // For simulation, we'll randomly succeed/fail to test rollback
        // In production, this would be based on actual system checks
        let validation_successful = true; // Always succeed for now

        if validation_successful {
            info!(
                "Firmware installation validation successful for request: {}",
                request_id
            );
        } else {
            warn!(
                "Firmware installation validation failed for request: {}",
                request_id
            );
        }

        Ok(validation_successful)
    }
}

/// Mock OTA manager for testing
pub struct MockOTAManager {
    device_id: String,
    #[allow(dead_code)]
    current_firmware_version: String,
    update_requests: Arc<RwLock<Vec<String>>>,
    validation_results: Arc<RwLock<HashMap<String, FirmwareValidationResult>>>,
    download_results: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MockOTAManager {
    pub fn new(device_id: String, current_firmware_version: String) -> Self {
        Self {
            device_id,
            current_firmware_version,
            update_requests: Arc::new(RwLock::new(Vec::new())),
            validation_results: Arc::new(RwLock::new(HashMap::new())),
            download_results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_update_requests(&self) -> Vec<String> {
        self.update_requests.read().await.clone()
    }

    pub async fn set_validation_result(&self, request_id: &str, result: FirmwareValidationResult) {
        self.validation_results
            .write()
            .await
            .insert(request_id.to_string(), result);
    }

    pub async fn set_download_result(&self, url: &str, data: Vec<u8>) {
        self.download_results
            .write()
            .await
            .insert(url.to_string(), data);
    }
}

#[async_trait]
impl OTAManagerTrait for MockOTAManager {
    async fn initialize(&mut self) -> SystemResult<()> {
        info!(
            "Mock OTA manager initialized for device: {}",
            self.device_id
        );
        Ok(())
    }

    async fn request_firmware_update(&self, version: &str) -> SystemResult<String> {
        let request_id = format!("mock-fw-req-{}", uuid::Uuid::new_v4());
        self.update_requests.write().await.push(version.to_string());
        Ok(request_id)
    }

    async fn validate_firmware_request(
        &self,
        request: &FirmwareUpdateRequest,
    ) -> SystemResult<FirmwareValidationResult> {
        let validation_results = self.validation_results.read().await;
        if let Some(result) = validation_results.get(&request.request_id) {
            Ok(result.clone())
        } else {
            // Default validation result
            Ok(FirmwareValidationResult {
                is_valid: true,
                checksum_verified: true,
                signature_verified: true,
                compatibility_verified: true,
                size_verified: true,
                error_messages: Vec::new(),
            })
        }
    }

    async fn request_pre_signed_url(&self, _request: &PreSignedUrlRequest) -> SystemResult<()> {
        Ok(())
    }

    async fn download_firmware(
        &self,
        url: &str,
        _expected_checksum: &str,
        _expected_size: u64,
    ) -> SystemResult<Vec<u8>> {
        let download_results = self.download_results.read().await;
        if let Some(data) = download_results.get(url) {
            Ok(data.clone())
        } else {
            Ok(b"mock firmware data".to_vec())
        }
    }

    async fn install_firmware(
        &self,
        _firmware_data: &[u8],
        _request: &FirmwareUpdateRequest,
    ) -> SystemResult<()> {
        Ok(())
    }

    async fn rollback_firmware(&self, _request_id: &str, _reason: &str) -> SystemResult<()> {
        Ok(())
    }

    async fn validate_installation(&self, _request_id: &str) -> SystemResult<bool> {
        Ok(true)
    }

    async fn get_update_status(
        &self,
        _request_id: &str,
    ) -> SystemResult<Option<FirmwareUpdateResult>> {
        Ok(None)
    }

    async fn cancel_update(&self, _request_id: &str) -> SystemResult<()> {
        Ok(())
    }

    fn set_progress_callback(&self, _callback: Box<dyn Fn(DownloadProgress) + Send + Sync>) {
        // Mock implementation
    }
}
