use aws_iot_core::{
    OTAManager, OTAManagerTrait, MockIoTClient, IoTClientTrait, SecurityManager, InMemoryCertificateStore,
    FirmwareUpdateRequest, FirmwareUpdateStatus,
    PreSignedUrlRequest, DownloadProgress
};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_ota_manager_initialization() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let mut ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    let result = ota_manager.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_firmware_update_request() {
    let mut iot_client = MockIoTClient::new();
    iot_client.connect().await.unwrap();
    let iot_client = Arc::new(iot_client);
    
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let mut ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client.clone(),
        security_manager,
    );
    
    ota_manager.initialize().await.unwrap();
    
    // Request firmware update
    let request_id = ota_manager.request_firmware_update("1.1.0").await.unwrap();
    assert!(!request_id.is_empty());
    assert!(request_id.starts_with("fw-req-"));
    
    // Check that the request was stored
    let status = ota_manager.get_update_status(&request_id).await.unwrap();
    assert!(status.is_some());
    
    let status = status.unwrap();
    assert_eq!(status.current_version, "1.0.0");
    assert_eq!(status.target_version, "1.1.0");
    assert!(matches!(status.status, FirmwareUpdateStatus::Requested | FirmwareUpdateStatus::Validating));
}

#[tokio::test]
async fn test_firmware_request_validation() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    // Test valid firmware request
    let valid_request = FirmwareUpdateRequest {
        request_id: "test-req-1".to_string(),
        firmware_version: "1.1.0".to_string(),
        compatibility_version: "1.0.0".to_string(),
        download_url: Some("https://example.com/firmware.bin".to_string()),
        checksum: "a".repeat(64), // Valid SHA256 hex
        checksum_algorithm: "sha256".to_string(),
        size_bytes: 1024 * 1024, // 1MB
        signature: "valid_signature".to_string(),
        public_key_id: "test-key".to_string(),
        metadata: None,
        requested_at: Utc::now(),
    };
    
    let result = ota_manager.validate_firmware_request(&valid_request).await.unwrap();
    assert!(result.is_valid);
    assert!(result.compatibility_verified);
    assert!(result.checksum_verified);
    assert!(result.signature_verified);
    assert!(result.size_verified);
    
    // Test invalid firmware request (incompatible version)
    let invalid_request = FirmwareUpdateRequest {
        firmware_version: "0.9.0".to_string(), // Downgrade
        compatibility_version: "1.1.0".to_string(), // Requires newer version
        checksum: "invalid_checksum".to_string(), // Invalid format
        size_bytes: 200 * 1024 * 1024, // Too large (200MB)
        ..valid_request.clone()
    };
    
    let result = ota_manager.validate_firmware_request(&invalid_request).await.unwrap();
    assert!(!result.is_valid);
    assert!(!result.compatibility_verified);
    assert!(!result.checksum_verified);
    assert!(!result.size_verified);
    assert!(!result.error_messages.is_empty());
}

#[tokio::test]
async fn test_pre_signed_url_request() {
    let mut iot_client = MockIoTClient::new();
    iot_client.connect().await.unwrap();
    let iot_client = Arc::new(iot_client);
    
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client.clone(),
        security_manager,
    );
    
    let url_request = PreSignedUrlRequest {
        request_id: "test-req-1".to_string(),
        firmware_version: "1.1.0".to_string(),
        device_id: "test-device".to_string(),
        requested_at: Utc::now(),
    };
    
    let result = ota_manager.request_pre_signed_url(&url_request).await;
    assert!(result.is_ok());
    
    // Verify that the request was published to IoT
    let messages = iot_client.get_published_messages().await;
    assert!(!messages.is_empty());
    
    let found_request = messages.iter().any(|(topic, _)| {
        topic == "downloads/test-device/firmware-request"
    });
    assert!(found_request);
}

#[tokio::test]
async fn test_firmware_download_validation() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    // Test invalid URL (non-HTTPS)
    let result = ota_manager.download_firmware(
        "http://example.com/firmware.bin",
        "checksum",
        1024
    ).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("HTTPS"));
    
    // Test invalid URL format
    let result = ota_manager.download_firmware(
        "not-a-url",
        "checksum",
        1024
    ).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid download URL"));
}

#[tokio::test]
async fn test_firmware_installation() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    let firmware_data = b"mock firmware data";
    let request = FirmwareUpdateRequest {
        request_id: "test-req-1".to_string(),
        firmware_version: "1.1.0".to_string(),
        compatibility_version: "1.0.0".to_string(),
        download_url: Some("https://example.com/firmware.bin".to_string()),
        checksum: "checksum".to_string(),
        checksum_algorithm: "sha256".to_string(),
        size_bytes: firmware_data.len() as u64,
        signature: "".to_string(), // No signature for this test
        public_key_id: "test-key".to_string(),
        metadata: None,
        requested_at: Utc::now(),
    };
    
    // Create update entry first
    let update_result = aws_iot_core::FirmwareUpdateResult {
        request_id: request.request_id.clone(),
        status: FirmwareUpdateStatus::Downloaded,
        current_version: "1.0.0".to_string(),
        target_version: "1.1.0".to_string(),
        progress_percent: 100.0,
        error_message: None,
        started_at: Utc::now(),
        completed_at: None,
    };
    
    ota_manager.active_updates.write().await.insert(request.request_id.clone(), update_result);
    
    let result = ota_manager.install_firmware(firmware_data, &request).await;
    assert!(result.is_ok());
    
    // Check that status was updated
    let status = ota_manager.get_update_status(&request.request_id).await.unwrap();
    assert!(status.is_some());
    assert!(matches!(status.unwrap().status, FirmwareUpdateStatus::Installed));
}

#[tokio::test]
async fn test_firmware_installation_size_mismatch() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    let firmware_data = b"mock firmware data";
    let request = FirmwareUpdateRequest {
        request_id: "test-req-1".to_string(),
        firmware_version: "1.1.0".to_string(),
        compatibility_version: "1.0.0".to_string(),
        download_url: Some("https://example.com/firmware.bin".to_string()),
        checksum: "checksum".to_string(),
        checksum_algorithm: "sha256".to_string(),
        size_bytes: 9999, // Wrong size
        signature: "".to_string(),
        public_key_id: "test-key".to_string(),
        metadata: None,
        requested_at: Utc::now(),
    };
    
    // Create update entry first
    let update_result = aws_iot_core::FirmwareUpdateResult {
        request_id: request.request_id.clone(),
        status: FirmwareUpdateStatus::Downloaded,
        current_version: "1.0.0".to_string(),
        target_version: "1.1.0".to_string(),
        progress_percent: 100.0,
        error_message: None,
        started_at: Utc::now(),
        completed_at: None,
    };
    
    ota_manager.active_updates.write().await.insert(request.request_id.clone(), update_result);
    
    let result = ota_manager.install_firmware(firmware_data, &request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("size mismatch"));
}

#[tokio::test]
async fn test_update_cancellation() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    let request_id = "test-req-1";
    
    // Create update entry
    let update_result = aws_iot_core::FirmwareUpdateResult {
        request_id: request_id.to_string(),
        status: FirmwareUpdateStatus::Downloading { progress_percent: 50.0 },
        current_version: "1.0.0".to_string(),
        target_version: "1.1.0".to_string(),
        progress_percent: 50.0,
        error_message: None,
        started_at: Utc::now(),
        completed_at: None,
    };
    
    ota_manager.active_updates.write().await.insert(request_id.to_string(), update_result);
    
    // Cancel the update
    let result = ota_manager.cancel_update(request_id).await;
    assert!(result.is_ok());
    
    // Check that status was updated to failed
    let status = ota_manager.get_update_status(request_id).await.unwrap();
    assert!(status.is_some());
    let status = status.unwrap();
    assert!(matches!(status.status, FirmwareUpdateStatus::Failed { .. }));
    assert!(status.completed_at.is_some());
}

#[tokio::test]
async fn test_update_cancellation_invalid_states() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    let request_id = "test-req-1";
    
    // Test cancelling already installed update
    let update_result = aws_iot_core::FirmwareUpdateResult {
        request_id: request_id.to_string(),
        status: FirmwareUpdateStatus::Installed,
        current_version: "1.0.0".to_string(),
        target_version: "1.1.0".to_string(),
        progress_percent: 100.0,
        error_message: None,
        started_at: Utc::now(),
        completed_at: Some(Utc::now()),
    };
    
    ota_manager.active_updates.write().await.insert(request_id.to_string(), update_result);
    
    let result = ota_manager.cancel_update(request_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot cancel completed update"));
    
    // Test cancelling during installation
    let update_result = aws_iot_core::FirmwareUpdateResult {
        request_id: request_id.to_string(),
        status: FirmwareUpdateStatus::Installing,
        current_version: "1.0.0".to_string(),
        target_version: "1.1.0".to_string(),
        progress_percent: 90.0,
        error_message: None,
        started_at: Utc::now(),
        completed_at: None,
    };
    
    ota_manager.active_updates.write().await.clear();
    ota_manager.active_updates.write().await.insert(request_id.to_string(), update_result);
    
    let result = ota_manager.cancel_update(request_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot cancel update during installation"));
}

#[tokio::test]
async fn test_progress_callback() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    let progress_updates = Arc::new(Mutex::new(Vec::new()));
    let progress_updates_clone = Arc::clone(&progress_updates);
    
    // Set progress callback
    ota_manager.set_progress_callback(Box::new(move |progress: DownloadProgress| {
        if let Ok(mut updates) = progress_updates_clone.try_lock() {
            updates.push(progress);
        }
    }));
    
    // Simulate progress update (this would normally happen during download)
    let test_progress = DownloadProgress {
        bytes_downloaded: 512,
        total_bytes: 1024,
        progress_percent: 50.0,
        download_speed_bps: 1024.0,
        estimated_time_remaining_seconds: Some(1),
    };
    
    if let Ok(callback) = ota_manager.progress_callback.try_lock() {
        if let Some(ref cb) = *callback {
            cb(test_progress);
        }
    }
    
    // Check that progress was recorded
    let updates = progress_updates.lock().await;
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].progress_percent, 50.0);
    assert_eq!(updates[0].bytes_downloaded, 512);
    assert_eq!(updates[0].total_bytes, 1024);
}

#[tokio::test]
async fn test_version_compatibility_validation() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.2.0".to_string(), // Current version
        iot_client,
        security_manager,
    );
    
    // Test valid upgrade (1.2.0 -> 1.3.0, requires 1.0.0)
    assert!(ota_manager.validate_version_compatibility("1.3.0", "1.0.0"));
    
    // Test valid upgrade (1.2.0 -> 2.0.0, requires 1.2.0)
    assert!(ota_manager.validate_version_compatibility("2.0.0", "1.2.0"));
    
    // Test invalid downgrade (1.2.0 -> 1.1.0)
    assert!(!ota_manager.validate_version_compatibility("1.1.0", "1.0.0"));
    
    // Test incompatible requirement (current 1.2.0, requires 1.3.0)
    assert!(!ota_manager.validate_version_compatibility("2.0.0", "1.3.0"));
    
    // Test same version (should be invalid)
    assert!(!ota_manager.validate_version_compatibility("1.2.0", "1.0.0"));
}

#[tokio::test]
async fn test_checksum_validation() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    let test_data = b"test data";
    let expected_hash = hex::encode(ota_manager.security_manager.compute_sha256(test_data));
    
    // Test valid checksum
    assert!(ota_manager.validate_checksum(test_data, &expected_hash, "sha256"));
    
    // Test invalid checksum
    assert!(!ota_manager.validate_checksum(test_data, "invalid_hash", "sha256"));
    
    // Test unsupported algorithm
    assert!(!ota_manager.validate_checksum(test_data, &expected_hash, "md5"));
}#
[tokio::test]
async fn test_firmware_rollback() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    let request_id = "test-req-rollback";
    
    // Create update entry
    let update_result = aws_iot_core::FirmwareUpdateResult {
        request_id: request_id.to_string(),
        status: FirmwareUpdateStatus::Installing,
        current_version: "1.0.0".to_string(),
        target_version: "1.1.0".to_string(),
        progress_percent: 90.0,
        error_message: None,
        started_at: Utc::now(),
        completed_at: None,
    };
    
    ota_manager.active_updates.write().await.insert(request_id.to_string(), update_result);
    
    // Store some rollback data
    ota_manager.rollback_data.write().await.insert(
        request_id.to_string(), 
        b"previous_firmware_data".to_vec()
    );
    
    // Perform rollback
    let result = ota_manager.rollback_firmware(request_id, "Test rollback").await;
    assert!(result.is_ok());
    
    // Check that status was updated to rolled back
    let status = ota_manager.get_update_status(request_id).await.unwrap();
    assert!(status.is_some());
    let status = status.unwrap();
    assert!(matches!(status.status, FirmwareUpdateStatus::RolledBack { .. }));
    assert!(status.completed_at.is_some());
}

#[tokio::test]
async fn test_firmware_rollback_no_data() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    let request_id = "test-req-no-rollback-data";
    
    // Try to rollback without rollback data
    let result = ota_manager.rollback_firmware(request_id, "Test rollback").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No rollback data available"));
}

#[tokio::test]
async fn test_installation_validation() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    let request_id = "test-req-validation";
    
    // Test installation validation
    let result = ota_manager.validate_installation(request_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap()); // Should return true for successful validation
}

#[tokio::test]
async fn test_firmware_installation_with_validation() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    let firmware_data = b"mock firmware data with validation";
    let request = FirmwareUpdateRequest {
        request_id: "test-req-validation".to_string(),
        firmware_version: "1.1.0".to_string(),
        compatibility_version: "1.0.0".to_string(),
        download_url: Some("https://example.com/firmware.bin".to_string()),
        checksum: "checksum".to_string(),
        checksum_algorithm: "sha256".to_string(),
        size_bytes: firmware_data.len() as u64,
        signature: "".to_string(), // No signature for this test
        public_key_id: "test-key".to_string(),
        metadata: None,
        requested_at: Utc::now(),
    };
    
    // Create update entry first
    let update_result = aws_iot_core::FirmwareUpdateResult {
        request_id: request.request_id.clone(),
        status: FirmwareUpdateStatus::Downloaded,
        current_version: "1.0.0".to_string(),
        target_version: "1.1.0".to_string(),
        progress_percent: 100.0,
        error_message: None,
        started_at: Utc::now(),
        completed_at: None,
    };
    
    ota_manager.active_updates.write().await.insert(request.request_id.clone(), update_result);
    
    let result = ota_manager.install_firmware(firmware_data, &request).await;
    assert!(result.is_ok());
    
    // Check that status progressed through validation to installed
    let status = ota_manager.get_update_status(&request.request_id).await.unwrap();
    assert!(status.is_some());
    let status = status.unwrap();
    assert!(matches!(status.status, FirmwareUpdateStatus::Installed));
    
    // Check that rollback data was stored
    let rollback_data = ota_manager.rollback_data.read().await;
    assert!(rollback_data.contains_key(&request.request_id));
}

#[tokio::test]
async fn test_signature_verification_failure() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    let firmware_data = b"mock firmware data";
    let request = FirmwareUpdateRequest {
        request_id: "test-req-sig-fail".to_string(),
        firmware_version: "1.1.0".to_string(),
        compatibility_version: "1.0.0".to_string(),
        download_url: Some("https://example.com/firmware.bin".to_string()),
        checksum: "checksum".to_string(),
        checksum_algorithm: "sha256".to_string(),
        size_bytes: firmware_data.len() as u64,
        signature: "invalid_signature".to_string(), // Invalid signature
        public_key_id: "".to_string(), // Empty key ID will cause verification to fail
        metadata: None,
        requested_at: Utc::now(),
    };
    
    // Create update entry first
    let update_result = aws_iot_core::FirmwareUpdateResult {
        request_id: request.request_id.clone(),
        status: FirmwareUpdateStatus::Downloaded,
        current_version: "1.0.0".to_string(),
        target_version: "1.1.0".to_string(),
        progress_percent: 100.0,
        error_message: None,
        started_at: Utc::now(),
        completed_at: None,
    };
    
    ota_manager.active_updates.write().await.insert(request.request_id.clone(), update_result);
    
    let result = ota_manager.install_firmware(firmware_data, &request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("signature verification failed"));
    
    // Check that status was updated to failed
    let status = ota_manager.get_update_status(&request.request_id).await.unwrap();
    assert!(status.is_some());
    let status = status.unwrap();
    assert!(matches!(status.status, FirmwareUpdateStatus::Failed { .. }));
}

#[tokio::test]
async fn test_cancel_update_during_validation() {
    let iot_client = Arc::new(MockIoTClient::new());
    let cert_store = Arc::new(InMemoryCertificateStore::new());
    let security_manager = Arc::new(SecurityManager::new(cert_store));
    
    let ota_manager = OTAManager::new(
        "test-device".to_string(),
        "1.0.0".to_string(),
        iot_client,
        security_manager,
    );
    
    let request_id = "test-req-cancel-validation";
    
    // Create update entry in validation state
    let update_result = aws_iot_core::FirmwareUpdateResult {
        request_id: request_id.to_string(),
        status: FirmwareUpdateStatus::ValidatingInstallation,
        current_version: "1.0.0".to_string(),
        target_version: "1.1.0".to_string(),
        progress_percent: 95.0,
        error_message: None,
        started_at: Utc::now(),
        completed_at: None,
    };
    
    ota_manager.active_updates.write().await.insert(request_id.to_string(), update_result);
    
    // Try to cancel during validation (should fail)
    let result = ota_manager.cancel_update(request_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot cancel update during installation"));
}