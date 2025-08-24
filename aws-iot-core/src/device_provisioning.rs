use crate::{SystemError, SystemResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use uuid::Uuid;

/// Device provisioning and certificate management system
pub struct DeviceProvisioningManager {
    aws_region: String,
    #[allow(dead_code)]
    iot_endpoint: String,
    ca_cert_path: Option<PathBuf>,
    provisioning_template: Option<String>,
    device_registry: DeviceRegistry,
}

/// Device registry for tracking provisioned devices
#[derive(Debug, Clone)]
pub struct DeviceRegistry {
    devices: HashMap<String, DeviceRecord>,
    certificates: HashMap<String, CertificateRecord>,
}

/// Device record in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRecord {
    pub device_id: String,
    pub thing_name: String,
    pub thing_type: String,
    pub certificate_arn: String,
    pub certificate_id: String,
    pub policy_name: String,
    pub created_at: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
    pub status: DeviceStatus,
    pub attributes: HashMap<String, String>,
    pub shadow_version: Option<u64>,
    pub firmware_version: Option<String>,
}

/// Certificate record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRecord {
    pub certificate_id: String,
    pub certificate_arn: String,
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub public_key_pem: String,
    pub device_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: CertificateStatus,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocation_reason: Option<String>,
}

/// Device status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Provisioning,
    Active,
    Inactive,
    Suspended,
    Decommissioned,
}

/// Certificate status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CertificateStatus {
    Active,
    Inactive,
    Revoked,
    PendingTransfer,
}

/// Device provisioning request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningRequest {
    pub device_id: String,
    pub thing_type: String,
    pub attributes: HashMap<String, String>,
    pub policy_template: Option<String>,
    pub certificate_validity_days: Option<u32>,
}

/// Device provisioning result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningResult {
    pub device_record: DeviceRecord,
    pub certificate_record: CertificateRecord,
    pub thing_arn: String,
    pub policy_arn: String,
    pub provisioning_success: bool,
    pub error_message: Option<String>,
}

/// Certificate generation configuration
#[derive(Debug, Clone)]
pub struct CertificateConfig {
    pub validity_days: u32,
    pub key_size: u32,
    pub country: String,
    pub organization: String,
    pub organizational_unit: String,
    pub common_name_prefix: String,
}

/// Device fleet statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetStatistics {
    pub total_devices: usize,
    pub active_devices: usize,
    pub inactive_devices: usize,
    pub suspended_devices: usize,
    pub decommissioned_devices: usize,
    pub certificates_active: usize,
    pub certificates_revoked: usize,
    pub devices_by_type: HashMap<String, usize>,
    pub certificates_expiring_soon: usize, // Within 30 days
}

impl Default for CertificateConfig {
    fn default() -> Self {
        Self {
            validity_days: 365,
            key_size: 2048,
            country: "US".to_string(),
            organization: "IoT Device".to_string(),
            organizational_unit: "Device Management".to_string(),
            common_name_prefix: "iot-device".to_string(),
        }
    }
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            certificates: HashMap::new(),
        }
    }

    pub fn add_device(&mut self, device: DeviceRecord) {
        self.devices.insert(device.device_id.clone(), device);
    }

    pub fn add_certificate(&mut self, certificate: CertificateRecord) {
        self.certificates
            .insert(certificate.certificate_id.clone(), certificate);
    }

    pub fn get_device(&self, device_id: &str) -> Option<&DeviceRecord> {
        self.devices.get(device_id)
    }

    pub fn get_certificate(&self, certificate_id: &str) -> Option<&CertificateRecord> {
        self.certificates.get(certificate_id)
    }

    pub fn list_devices(&self) -> Vec<&DeviceRecord> {
        self.devices.values().collect()
    }

    pub fn list_certificates(&self) -> Vec<&CertificateRecord> {
        self.certificates.values().collect()
    }

    pub fn update_device_status(
        &mut self,
        device_id: &str,
        status: DeviceStatus,
    ) -> SystemResult<()> {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.status = status;
            Ok(())
        } else {
            Err(SystemError::Configuration(format!(
                "Device not found: {}",
                device_id
            )))
        }
    }

    pub fn update_device_last_seen(
        &mut self,
        device_id: &str,
        timestamp: DateTime<Utc>,
    ) -> SystemResult<()> {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.last_seen = Some(timestamp);
            Ok(())
        } else {
            Err(SystemError::Configuration(format!(
                "Device not found: {}",
                device_id
            )))
        }
    }

    pub fn get_statistics(&self) -> FleetStatistics {
        let total_devices = self.devices.len();
        let mut active_devices = 0;
        let mut inactive_devices = 0;
        let mut suspended_devices = 0;
        let mut decommissioned_devices = 0;
        let mut devices_by_type = HashMap::new();

        for device in self.devices.values() {
            match device.status {
                DeviceStatus::Active => active_devices += 1,
                DeviceStatus::Inactive => inactive_devices += 1,
                DeviceStatus::Suspended => suspended_devices += 1,
                DeviceStatus::Decommissioned => decommissioned_devices += 1,
                DeviceStatus::Provisioning => {} // Don't count provisioning devices
            }

            *devices_by_type
                .entry(device.thing_type.clone())
                .or_insert(0) += 1;
        }

        let certificates_active = self
            .certificates
            .values()
            .filter(|cert| cert.status == CertificateStatus::Active)
            .count();

        let certificates_revoked = self
            .certificates
            .values()
            .filter(|cert| cert.status == CertificateStatus::Revoked)
            .count();

        let now = Utc::now();
        let thirty_days_from_now = now + chrono::Duration::days(30);
        let certificates_expiring_soon = self
            .certificates
            .values()
            .filter(|cert| {
                cert.expires_at <= thirty_days_from_now && cert.status == CertificateStatus::Active
            })
            .count();

        FleetStatistics {
            total_devices,
            active_devices,
            inactive_devices,
            suspended_devices,
            decommissioned_devices,
            certificates_active,
            certificates_revoked,
            devices_by_type,
            certificates_expiring_soon,
        }
    }
}

impl DeviceProvisioningManager {
    /// Create a new device provisioning manager
    pub fn new(aws_region: String, iot_endpoint: String) -> Self {
        Self {
            aws_region,
            iot_endpoint,
            ca_cert_path: None,
            provisioning_template: None,
            device_registry: DeviceRegistry::new(),
        }
    }

    /// Set CA certificate path for certificate validation
    pub fn set_ca_cert_path(&mut self, path: PathBuf) {
        self.ca_cert_path = Some(path);
    }

    /// Set provisioning template for automated provisioning
    pub fn set_provisioning_template(&mut self, template: String) {
        self.provisioning_template = Some(template);
    }

    /// Provision a new device with certificate generation
    pub async fn provision_device(
        &mut self,
        request: ProvisioningRequest,
    ) -> SystemResult<ProvisioningResult> {
        info!("Provisioning device: {}", request.device_id);

        // Generate certificate
        let certificate_config = CertificateConfig {
            validity_days: request.certificate_validity_days.unwrap_or(365),
            common_name_prefix: format!("iot-device-{}", request.device_id),
            ..Default::default()
        };

        let certificate_record = self
            .generate_certificate(&request.device_id, certificate_config)
            .await?;

        // Create IoT Thing
        let thing_name = format!("thing-{}", request.device_id);
        let thing_arn = self
            .create_iot_thing(&thing_name, &request.thing_type, &request.attributes)
            .await?;

        // Create and attach policy
        let policy_name = format!("policy-{}", request.device_id);
        let policy_arn = self
            .create_device_policy(
                &policy_name,
                &request.device_id,
                request.policy_template.as_deref(),
            )
            .await?;

        // Attach certificate to thing and policy
        self.attach_certificate_to_thing(&certificate_record.certificate_arn, &thing_name)
            .await?;
        self.attach_policy_to_certificate(&policy_name, &certificate_record.certificate_arn)
            .await?;

        // Create device record
        let device_record = DeviceRecord {
            device_id: request.device_id.clone(),
            thing_name: thing_name.clone(),
            thing_type: request.thing_type,
            certificate_arn: certificate_record.certificate_arn.clone(),
            certificate_id: certificate_record.certificate_id.clone(),
            policy_name: policy_name.clone(),
            created_at: Utc::now(),
            last_seen: None,
            status: DeviceStatus::Active,
            attributes: request.attributes,
            shadow_version: None,
            firmware_version: None,
        };

        // Add to registry
        self.device_registry.add_device(device_record.clone());
        self.device_registry
            .add_certificate(certificate_record.clone());

        let result = ProvisioningResult {
            device_record,
            certificate_record,
            thing_arn,
            policy_arn,
            provisioning_success: true,
            error_message: None,
        };

        info!("Device provisioned successfully: {}", request.device_id);
        Ok(result)
    }

    /// Decommission a device (revoke certificate, delete thing, detach policies)
    pub async fn decommission_device(&mut self, device_id: &str) -> SystemResult<()> {
        info!("Decommissioning device: {}", device_id);

        let device = self
            .device_registry
            .get_device(device_id)
            .ok_or_else(|| SystemError::Configuration(format!("Device not found: {}", device_id)))?
            .clone();

        // Revoke certificate
        self.revoke_certificate(&device.certificate_id, "Device decommissioned")
            .await?;

        // Detach policy from certificate
        self.detach_policy_from_certificate(&device.policy_name, &device.certificate_arn)
            .await?;

        // Detach certificate from thing
        self.detach_certificate_from_thing(&device.certificate_arn, &device.thing_name)
            .await?;

        // Delete IoT thing
        self.delete_iot_thing(&device.thing_name).await?;

        // Delete policy
        self.delete_device_policy(&device.policy_name).await?;

        // Update device status
        self.device_registry
            .update_device_status(device_id, DeviceStatus::Decommissioned)?;

        info!("Device decommissioned successfully: {}", device_id);
        Ok(())
    }

    /// Rotate certificate for a device
    pub async fn rotate_certificate(&mut self, device_id: &str) -> SystemResult<CertificateRecord> {
        info!("Rotating certificate for device: {}", device_id);

        let device = self
            .device_registry
            .get_device(device_id)
            .ok_or_else(|| SystemError::Configuration(format!("Device not found: {}", device_id)))?
            .clone();

        // Generate new certificate
        let certificate_config = CertificateConfig {
            common_name_prefix: format!("iot-device-{}", device_id),
            ..Default::default()
        };

        let new_certificate = self
            .generate_certificate(device_id, certificate_config)
            .await?;

        // Attach new certificate to thing
        self.attach_certificate_to_thing(&new_certificate.certificate_arn, &device.thing_name)
            .await?;

        // Attach policy to new certificate
        self.attach_policy_to_certificate(&device.policy_name, &new_certificate.certificate_arn)
            .await?;

        // Revoke old certificate
        self.revoke_certificate(&device.certificate_id, "Certificate rotation")
            .await?;

        // Update device record with new certificate
        if let Some(device_record) = self.device_registry.devices.get_mut(device_id) {
            device_record.certificate_arn = new_certificate.certificate_arn.clone();
            device_record.certificate_id = new_certificate.certificate_id.clone();
        }

        // Add new certificate to registry
        self.device_registry
            .add_certificate(new_certificate.clone());

        info!("Certificate rotated successfully for device: {}", device_id);
        Ok(new_certificate)
    }

    /// List all devices with optional filtering
    pub fn list_devices(
        &self,
        status_filter: Option<DeviceStatus>,
        thing_type_filter: Option<&str>,
    ) -> Vec<&DeviceRecord> {
        self.device_registry
            .devices
            .values()
            .filter(|device| {
                if let Some(status) = &status_filter {
                    if device.status != *status {
                        return false;
                    }
                }
                if let Some(thing_type) = thing_type_filter {
                    if device.thing_type != thing_type {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// Get device information
    pub fn get_device(&self, device_id: &str) -> Option<&DeviceRecord> {
        self.device_registry.get_device(device_id)
    }

    /// Update device last seen timestamp
    pub fn update_device_last_seen(&mut self, device_id: &str) -> SystemResult<()> {
        self.device_registry
            .update_device_last_seen(device_id, Utc::now())
    }

    /// Get fleet statistics
    pub fn get_fleet_statistics(&self) -> FleetStatistics {
        self.device_registry.get_statistics()
    }

    /// Export device registry to file
    pub fn export_registry(&self, file_path: &Path) -> SystemResult<()> {
        let registry_data = serde_json::to_string_pretty(&self.device_registry.devices)
            .map_err(SystemError::Serialization)?;

        std::fs::write(file_path, registry_data)
            .map_err(|e| SystemError::Configuration(format!("Failed to write registry: {}", e)))?;

        info!("Device registry exported to: {}", file_path.display());
        Ok(())
    }

    /// Import device registry from file
    pub fn import_registry(&mut self, file_path: &Path) -> SystemResult<()> {
        let registry_data = std::fs::read_to_string(file_path)
            .map_err(|e| SystemError::Configuration(format!("Failed to read registry: {}", e)))?;

        let devices: HashMap<String, DeviceRecord> =
            serde_json::from_str(&registry_data).map_err(SystemError::Serialization)?;

        self.device_registry.devices = devices;

        info!("Device registry imported from: {}", file_path.display());
        Ok(())
    }

    // ========== Private Implementation Methods ==========

    /// Generate a new certificate for a device
    async fn generate_certificate(
        &self,
        device_id: &str,
        config: CertificateConfig,
    ) -> SystemResult<CertificateRecord> {
        info!("Generating certificate for device: {}", device_id);

        // In a real implementation, this would use AWS IoT CreateKeysAndCertificate API
        // For this demo, we'll create mock certificate data
        let certificate_id = Uuid::new_v4().to_string();
        let certificate_arn = format!(
            "arn:aws:iot:{}:123456789012:cert/{}",
            self.aws_region, certificate_id
        );

        let certificate_pem = format!(
            "-----BEGIN CERTIFICATE-----\n\
            MIICertificateDataForDevice{}\n\
            -----END CERTIFICATE-----",
            device_id
        );

        let private_key_pem = format!(
            "-----BEGIN RSA PRIVATE KEY-----\n\
            MIIPrivateKeyDataForDevice{}\n\
            -----END RSA PRIVATE KEY-----",
            device_id
        );

        let public_key_pem = format!(
            "-----BEGIN PUBLIC KEY-----\n\
            MIIPublicKeyDataForDevice{}\n\
            -----END PUBLIC KEY-----",
            device_id
        );

        let expires_at = Utc::now() + chrono::Duration::days(config.validity_days as i64);

        let certificate_record = CertificateRecord {
            certificate_id,
            certificate_arn,
            certificate_pem,
            private_key_pem,
            public_key_pem,
            device_id: device_id.to_string(),
            created_at: Utc::now(),
            expires_at,
            status: CertificateStatus::Active,
            revoked_at: None,
            revocation_reason: None,
        };

        debug!(
            "Certificate generated: {}",
            certificate_record.certificate_id
        );
        Ok(certificate_record)
    }

    /// Create IoT Thing
    async fn create_iot_thing(
        &self,
        thing_name: &str,
        thing_type: &str,
        attributes: &HashMap<String, String>,
    ) -> SystemResult<String> {
        info!("Creating IoT Thing: {}", thing_name);

        // In a real implementation, this would use AWS IoT CreateThing API
        let thing_arn = format!(
            "arn:aws:iot:{}:123456789012:thing/{}",
            self.aws_region, thing_name
        );

        debug!("IoT Thing created: {} (type: {})", thing_name, thing_type);
        debug!("Thing attributes: {:?}", attributes);

        Ok(thing_arn)
    }

    /// Create device policy
    async fn create_device_policy(
        &self,
        policy_name: &str,
        device_id: &str,
        template: Option<&str>,
    ) -> SystemResult<String> {
        info!("Creating device policy: {}", policy_name);

        let default_policy = format!(
            r#"{{
            "Version": "2012-10-17",
            "Statement": [
                {{
                    "Effect": "Allow",
                    "Action": [
                        "iot:Connect",
                        "iot:Publish",
                        "iot:Subscribe",
                        "iot:Receive"
                    ],
                    "Resource": [
                        "arn:aws:iot:{}:*:client/{}",
                        "arn:aws:iot:{}:*:topic/device/{}/data",
                        "arn:aws:iot:{}:*:topic/device/{}/shadow/*",
                        "arn:aws:iot:{}:*:topicfilter/device/{}/data",
                        "arn:aws:iot:{}:*:topicfilter/device/{}/shadow/*"
                    ]
                }}
            ]
        }}"#,
            self.aws_region,
            device_id,
            self.aws_region,
            device_id,
            self.aws_region,
            device_id,
            self.aws_region,
            device_id,
            self.aws_region,
            device_id
        );
        let policy_document = template.unwrap_or(&default_policy);

        // In a real implementation, this would use AWS IoT CreatePolicy API
        let policy_arn = format!(
            "arn:aws:iot:{}:123456789012:policy/{}",
            self.aws_region, policy_name
        );

        debug!("Device policy created: {}", policy_name);
        debug!("Policy document: {}", policy_document);

        Ok(policy_arn)
    }

    /// Attach certificate to thing
    async fn attach_certificate_to_thing(
        &self,
        certificate_arn: &str,
        thing_name: &str,
    ) -> SystemResult<()> {
        info!(
            "Attaching certificate to thing: {} -> {}",
            certificate_arn, thing_name
        );

        // In a real implementation, this would use AWS IoT AttachThingPrincipal API
        debug!("Certificate attached to thing successfully");

        Ok(())
    }

    /// Attach policy to certificate
    async fn attach_policy_to_certificate(
        &self,
        policy_name: &str,
        certificate_arn: &str,
    ) -> SystemResult<()> {
        info!(
            "Attaching policy to certificate: {} -> {}",
            policy_name, certificate_arn
        );

        // In a real implementation, this would use AWS IoT AttachPrincipalPolicy API
        debug!("Policy attached to certificate successfully");

        Ok(())
    }

    /// Revoke certificate
    async fn revoke_certificate(&mut self, certificate_id: &str, reason: &str) -> SystemResult<()> {
        info!(
            "Revoking certificate: {} (reason: {})",
            certificate_id, reason
        );

        // In a real implementation, this would use AWS IoT UpdateCertificate API
        if let Some(certificate) = self.device_registry.certificates.get_mut(certificate_id) {
            certificate.status = CertificateStatus::Revoked;
            certificate.revoked_at = Some(Utc::now());
            certificate.revocation_reason = Some(reason.to_string());
        }

        debug!("Certificate revoked successfully");
        Ok(())
    }

    /// Detach policy from certificate
    async fn detach_policy_from_certificate(
        &self,
        policy_name: &str,
        certificate_arn: &str,
    ) -> SystemResult<()> {
        info!(
            "Detaching policy from certificate: {} -> {}",
            policy_name, certificate_arn
        );

        // In a real implementation, this would use AWS IoT DetachPrincipalPolicy API
        debug!("Policy detached from certificate successfully");

        Ok(())
    }

    /// Detach certificate from thing
    async fn detach_certificate_from_thing(
        &self,
        certificate_arn: &str,
        thing_name: &str,
    ) -> SystemResult<()> {
        info!(
            "Detaching certificate from thing: {} -> {}",
            certificate_arn, thing_name
        );

        // In a real implementation, this would use AWS IoT DetachThingPrincipal API
        debug!("Certificate detached from thing successfully");

        Ok(())
    }

    /// Delete IoT thing
    async fn delete_iot_thing(&self, thing_name: &str) -> SystemResult<()> {
        info!("Deleting IoT thing: {}", thing_name);

        // In a real implementation, this would use AWS IoT DeleteThing API
        debug!("IoT thing deleted successfully");

        Ok(())
    }

    /// Delete device policy
    async fn delete_device_policy(&self, policy_name: &str) -> SystemResult<()> {
        info!("Deleting device policy: {}", policy_name);

        // In a real implementation, this would use AWS IoT DeletePolicy API
        debug!("Device policy deleted successfully");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_device_provisioning() {
        let mut manager = DeviceProvisioningManager::new(
            "us-east-1".to_string(),
            "https://iot.us-east-1.amazonaws.com".to_string(),
        );

        let mut attributes = HashMap::new();
        attributes.insert("location".to_string(), "factory-floor-1".to_string());
        attributes.insert("model".to_string(), "sensor-v2".to_string());

        let request = ProvisioningRequest {
            device_id: "test-device-001".to_string(),
            thing_type: "IoTSensor".to_string(),
            attributes,
            policy_template: None,
            certificate_validity_days: Some(365),
        };

        let result = manager.provision_device(request).await.unwrap();

        assert!(result.provisioning_success);
        assert_eq!(result.device_record.device_id, "test-device-001");
        assert_eq!(result.device_record.status, DeviceStatus::Active);
        assert!(!result.certificate_record.certificate_pem.is_empty());
    }

    #[tokio::test]
    async fn test_certificate_rotation() {
        let mut manager = DeviceProvisioningManager::new(
            "us-east-1".to_string(),
            "https://iot.us-east-1.amazonaws.com".to_string(),
        );

        // First provision a device
        let request = ProvisioningRequest {
            device_id: "test-device-002".to_string(),
            thing_type: "IoTSensor".to_string(),
            attributes: HashMap::new(),
            policy_template: None,
            certificate_validity_days: Some(365),
        };

        let _result = manager.provision_device(request).await.unwrap();

        // Then rotate its certificate
        let new_certificate = manager.rotate_certificate("test-device-002").await.unwrap();

        assert_eq!(new_certificate.device_id, "test-device-002");
        assert_eq!(new_certificate.status, CertificateStatus::Active);
    }

    #[tokio::test]
    async fn test_device_decommissioning() {
        let mut manager = DeviceProvisioningManager::new(
            "us-east-1".to_string(),
            "https://iot.us-east-1.amazonaws.com".to_string(),
        );

        // First provision a device
        let request = ProvisioningRequest {
            device_id: "test-device-003".to_string(),
            thing_type: "IoTSensor".to_string(),
            attributes: HashMap::new(),
            policy_template: None,
            certificate_validity_days: Some(365),
        };

        let _result = manager.provision_device(request).await.unwrap();

        // Then decommission it
        manager
            .decommission_device("test-device-003")
            .await
            .unwrap();

        let device = manager.get_device("test-device-003").unwrap();
        assert_eq!(device.status, DeviceStatus::Decommissioned);
    }

    #[test]
    fn test_fleet_statistics() {
        let mut manager = DeviceProvisioningManager::new(
            "us-east-1".to_string(),
            "https://iot.us-east-1.amazonaws.com".to_string(),
        );

        // Add some test devices
        let device1 = DeviceRecord {
            device_id: "device-1".to_string(),
            thing_name: "thing-1".to_string(),
            thing_type: "Sensor".to_string(),
            certificate_arn: "arn:aws:iot:us-east-1:123456789012:cert/cert-1".to_string(),
            certificate_id: "cert-1".to_string(),
            policy_name: "policy-1".to_string(),
            created_at: Utc::now(),
            last_seen: None,
            status: DeviceStatus::Active,
            attributes: HashMap::new(),
            shadow_version: None,
            firmware_version: None,
        };

        let device2 = DeviceRecord {
            device_id: "device-2".to_string(),
            thing_name: "thing-2".to_string(),
            thing_type: "Actuator".to_string(),
            certificate_arn: "arn:aws:iot:us-east-1:123456789012:cert/cert-2".to_string(),
            certificate_id: "cert-2".to_string(),
            policy_name: "policy-2".to_string(),
            created_at: Utc::now(),
            last_seen: None,
            status: DeviceStatus::Inactive,
            attributes: HashMap::new(),
            shadow_version: None,
            firmware_version: None,
        };

        manager.device_registry.add_device(device1);
        manager.device_registry.add_device(device2);

        let stats = manager.get_fleet_statistics();
        assert_eq!(stats.total_devices, 2);
        assert_eq!(stats.active_devices, 1);
        assert_eq!(stats.inactive_devices, 1);
        assert_eq!(stats.devices_by_type.get("Sensor"), Some(&1));
        assert_eq!(stats.devices_by_type.get("Actuator"), Some(&1));
    }

    #[test]
    fn test_registry_export_import() {
        use tempfile::TempDir;

        let mut manager = DeviceProvisioningManager::new(
            "us-east-1".to_string(),
            "https://iot.us-east-1.amazonaws.com".to_string(),
        );

        // Add a test device
        let device = DeviceRecord {
            device_id: "export-test-device".to_string(),
            thing_name: "export-test-thing".to_string(),
            thing_type: "TestDevice".to_string(),
            certificate_arn: "arn:aws:iot:us-east-1:123456789012:cert/export-test-cert".to_string(),
            certificate_id: "export-test-cert".to_string(),
            policy_name: "export-test-policy".to_string(),
            created_at: Utc::now(),
            last_seen: None,
            status: DeviceStatus::Active,
            attributes: HashMap::new(),
            shadow_version: None,
            firmware_version: None,
        };

        manager.device_registry.add_device(device);

        // Export registry
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path().join("registry.json");
        manager.export_registry(&export_path).unwrap();

        // Create new manager and import
        let mut new_manager = DeviceProvisioningManager::new(
            "us-east-1".to_string(),
            "https://iot.us-east-1.amazonaws.com".to_string(),
        );
        new_manager.import_registry(&export_path).unwrap();

        // Verify import
        let imported_device = new_manager.get_device("export-test-device").unwrap();
        assert_eq!(imported_device.device_id, "export-test-device");
        assert_eq!(imported_device.thing_type, "TestDevice");
    }
}
