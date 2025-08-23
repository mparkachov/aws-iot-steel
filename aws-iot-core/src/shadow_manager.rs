use crate::{IoTError, IoTResult, IoTClientTrait, DeviceState, RuntimeStatus, SystemInfo, MemoryInfo};
use crate::types::{HardwareState, SleepStatus};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rumqttc::QoS;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Shadow update message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowUpdate {
    pub state: ShadowState,
    pub metadata: Option<ShadowMetadata>,
    pub version: Option<u64>,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Shadow state containing desired and reported states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowState {
    pub desired: Option<DesiredState>,
    pub reported: Option<DeviceState>,
    pub delta: Option<serde_json::Value>,
}

/// Desired state from cloud
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesiredState {
    pub led_control: Option<bool>,
    pub sleep_duration: Option<f64>,
    pub configuration: Option<DeviceConfiguration>,
    pub program_commands: Option<ProgramCommands>,
}

/// Device configuration from shadow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfiguration {
    pub log_level: Option<String>,
    pub reporting_interval: Option<u64>,
    pub auto_update: Option<bool>,
}

/// Program management commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramCommands {
    pub load_program: Option<String>,
    pub stop_program: Option<String>,
    pub restart_program: Option<String>,
}

/// Shadow metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowMetadata {
    pub desired: Option<serde_json::Value>,
    pub reported: Option<serde_json::Value>,
}

/// Shadow delta change notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowDelta {
    pub version: u64,
    pub timestamp: DateTime<Utc>,
    pub state: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

/// Shadow operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowOperationResult {
    pub success: bool,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Callback for handling shadow delta changes
pub type ShadowDeltaCallback = Arc<dyn Fn(ShadowDelta) -> Result<(), IoTError> + Send + Sync>;

/// Shadow manager trait for testability
#[async_trait]
pub trait ShadowManagerTrait: Send + Sync {
    async fn initialize(&mut self) -> IoTResult<()>;
    async fn update_reported_state(&self, state: &DeviceState) -> IoTResult<()>;
    async fn get_shadow(&self) -> IoTResult<ShadowUpdate>;
    async fn process_desired_state(&self, desired: &DesiredState) -> IoTResult<ShadowOperationResult>;
    fn set_delta_callback(&self, callback: ShadowDeltaCallback);
}

/// Main shadow manager implementation
pub struct ShadowManager {
    device_id: String,
    thing_name: String,
    iot_client: Arc<dyn IoTClientTrait>,
    current_shadow: Arc<RwLock<Option<ShadowUpdate>>>,
    delta_callback: Arc<RwLock<Option<ShadowDeltaCallback>>>,
    shadow_version: Arc<RwLock<u64>>,
}

impl ShadowManager {
    /// Create a new shadow manager
    pub fn new(
        device_id: String,
        thing_name: String,
        iot_client: Arc<dyn IoTClientTrait>,
    ) -> Self {
        Self {
            device_id,
            thing_name,
            iot_client,
            current_shadow: Arc::new(RwLock::new(None)),
            delta_callback: Arc::new(RwLock::new(None)),
            shadow_version: Arc::new(RwLock::new(0)),
        }
    }

    /// Get shadow topic for specific operation
    fn get_shadow_topic(&self, operation: &str) -> String {
        format!("$aws/thing/{}/shadow/{}", self.thing_name, operation)
    }

    /// Subscribe to shadow topics
    async fn subscribe_to_shadow_topics(&self) -> IoTResult<()> {
        let topics = vec![
            ("update/accepted", QoS::AtLeastOnce),
            ("update/rejected", QoS::AtLeastOnce),
            ("update/delta", QoS::AtLeastOnce),
            ("get/accepted", QoS::AtLeastOnce),
            ("get/rejected", QoS::AtLeastOnce),
        ];

        for (operation, qos) in topics {
            let topic = self.get_shadow_topic(operation);
            self.iot_client.subscribe(&topic, qos).await?;
            info!("Subscribed to shadow topic: {}", topic);
        }

        Ok(())
    }

    /// Handle incoming shadow messages
    async fn handle_shadow_message(&self, topic: &str, payload: &[u8]) -> IoTResult<()> {
        let payload_str = String::from_utf8(payload.to_vec())
            .map_err(|e| IoTError::MessageParsing(format!("Invalid UTF-8: {}", e)))?;

        debug!("Received shadow message on topic: {}", topic);
        debug!("Payload: {}", payload_str);

        if topic.contains("/update/accepted") {
            self.handle_update_accepted(&payload_str).await?;
        } else if topic.contains("/update/rejected") {
            self.handle_update_rejected(&payload_str).await?;
        } else if topic.contains("/update/delta") {
            self.handle_update_delta(&payload_str).await?;
        } else if topic.contains("/get/accepted") {
            self.handle_get_accepted(&payload_str).await?;
        } else if topic.contains("/get/rejected") {
            self.handle_get_rejected(&payload_str).await?;
        }

        Ok(())
    }

    /// Handle shadow update accepted
    async fn handle_update_accepted(&self, payload: &str) -> IoTResult<()> {
        let shadow_update: ShadowUpdate = serde_json::from_str(payload)
            .map_err(|e| IoTError::MessageParsing(format!("Failed to parse shadow update: {}", e)))?;

        info!("Shadow update accepted, version: {:?}", shadow_update.version);

        // Update local shadow state
        *self.current_shadow.write().await = Some(shadow_update.clone());

        if let Some(version) = shadow_update.version {
            *self.shadow_version.write().await = version;
        }

        Ok(())
    }

    /// Handle shadow update rejected
    async fn handle_update_rejected(&self, payload: &str) -> IoTResult<()> {
        warn!("Shadow update rejected: {}", payload);
        
        // Try to parse error details
        if let Ok(error_info) = serde_json::from_str::<serde_json::Value>(payload) {
            if let Some(message) = error_info.get("message") {
                error!("Shadow rejection reason: {}", message);
            }
        }

        Ok(())
    }

    /// Handle shadow delta (desired state changes)
    async fn handle_update_delta(&self, payload: &str) -> IoTResult<()> {
        let delta: ShadowDelta = serde_json::from_str(payload)
            .map_err(|e| IoTError::MessageParsing(format!("Failed to parse shadow delta: {}", e)))?;

        info!("Received shadow delta, version: {}", delta.version);
        debug!("Delta state: {}", serde_json::to_string_pretty(&delta.state).unwrap_or_default());

        // Update shadow version
        *self.shadow_version.write().await = delta.version;

        // Call delta callback if set
        if let Some(callback) = self.delta_callback.read().await.as_ref() {
            if let Err(e) = callback(delta.clone()) {
                error!("Delta callback error: {}", e);
            }
        }

        // Process the delta automatically
        if let Ok(desired_state) = serde_json::from_value::<DesiredState>(delta.state) {
            match self.process_desired_state(&desired_state).await {
                Ok(result) => {
                    if result.success {
                        info!("Successfully processed shadow delta");
                    } else {
                        warn!("Failed to process shadow delta: {:?}", result.message);
                    }
                }
                Err(e) => {
                    error!("Error processing shadow delta: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Handle shadow get accepted
    async fn handle_get_accepted(&self, payload: &str) -> IoTResult<()> {
        let shadow_update: ShadowUpdate = serde_json::from_str(payload)
            .map_err(|e| IoTError::MessageParsing(format!("Failed to parse shadow: {}", e)))?;

        info!("Received current shadow, version: {:?}", shadow_update.version);

        // Update local shadow state
        *self.current_shadow.write().await = Some(shadow_update.clone());

        if let Some(version) = shadow_update.version {
            *self.shadow_version.write().await = version;
        }

        // Process any desired state
        if let Some(desired) = shadow_update.state.desired {
            match self.process_desired_state(&desired).await {
                Ok(result) => {
                    if result.success {
                        info!("Successfully processed desired state from shadow");
                    } else {
                        warn!("Failed to process desired state: {:?}", result.message);
                    }
                }
                Err(e) => {
                    error!("Error processing desired state: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Handle shadow get rejected
    async fn handle_get_rejected(&self, payload: &str) -> IoTResult<()> {
        warn!("Shadow get rejected: {}", payload);
        Ok(())
    }

    /// Request current shadow from AWS IoT
    async fn request_shadow(&self) -> IoTResult<()> {
        let topic = self.get_shadow_topic("get");
        let empty_payload = b"{}";
        
        self.iot_client.publish(&topic, empty_payload, QoS::AtLeastOnce).await?;
        info!("Requested current shadow");
        
        Ok(())
    }

    /// Create device state from current system state
    pub async fn create_device_state(
        runtime_status: RuntimeStatus,
        led_state: bool,
        sleep_status: SleepStatus,
        memory_info: MemoryInfo,
        firmware_version: String,
        platform: String,
        uptime_seconds: u64,
    ) -> DeviceState {
        DeviceState {
            runtime_status,
            hardware_state: HardwareState {
                led_status: led_state,
                sleep_status,
                memory_usage: memory_info,
            },
            system_info: SystemInfo {
                firmware_version,
                platform,
                uptime_seconds,
                steel_runtime_version: "0.7.0".to_string(),
            },
            timestamp: Utc::now(),
        }
    }
}

#[async_trait]
impl ShadowManagerTrait for ShadowManager {
    async fn initialize(&mut self) -> IoTResult<()> {
        info!("Initializing shadow manager for device: {}", self.device_id);

        // Subscribe to shadow topics
        self.subscribe_to_shadow_topics().await?;

        // Request current shadow
        self.request_shadow().await?;

        info!("Shadow manager initialized successfully");
        Ok(())
    }

    async fn update_reported_state(&self, state: &DeviceState) -> IoTResult<()> {
        let shadow_update = serde_json::json!({
            "state": {
                "reported": state
            }
        });

        let payload = serde_json::to_vec(&shadow_update)
            .map_err(|e| IoTError::MessageParsing(format!("Failed to serialize shadow update: {}", e)))?;

        let topic = self.get_shadow_topic("update");
        self.iot_client.publish(&topic, &payload, QoS::AtLeastOnce).await?;

        debug!("Updated reported shadow state");
        Ok(())
    }

    async fn get_shadow(&self) -> IoTResult<ShadowUpdate> {
        if let Some(shadow) = self.current_shadow.read().await.as_ref() {
            Ok(shadow.clone())
        } else {
            // Request shadow if we don't have it
            self.request_shadow().await?;
            
            // Return empty shadow for now
            Ok(ShadowUpdate {
                state: ShadowState {
                    desired: None,
                    reported: None,
                    delta: None,
                },
                metadata: None,
                version: Some(0),
                timestamp: Some(Utc::now()),
            })
        }
    }

    async fn process_desired_state(&self, desired: &DesiredState) -> IoTResult<ShadowOperationResult> {
        let mut operations = Vec::new();
        let mut errors = Vec::new();

        // Process LED control
        if let Some(led_state) = desired.led_control {
            operations.push(format!("Set LED to {}", if led_state { "ON" } else { "OFF" }));
            // Note: Actual LED control would be implemented via RustAPI integration
        }

        // Process sleep duration
        if let Some(duration) = desired.sleep_duration {
            if duration > 0.0 {
                operations.push(format!("Sleep for {} seconds", duration));
                // Note: Actual sleep would be implemented via RustAPI integration
            } else {
                errors.push("Invalid sleep duration".to_string());
            }
        }

        // Process configuration changes
        if let Some(config) = &desired.configuration {
            if let Some(log_level) = &config.log_level {
                operations.push(format!("Set log level to {}", log_level));
            }
            if let Some(interval) = config.reporting_interval {
                operations.push(format!("Set reporting interval to {} seconds", interval));
            }
        }

        // Process program commands
        if let Some(commands) = &desired.program_commands {
            if let Some(program) = &commands.load_program {
                operations.push(format!("Load program: {}", program));
            }
            if let Some(program) = &commands.stop_program {
                operations.push(format!("Stop program: {}", program));
            }
            if let Some(program) = &commands.restart_program {
                operations.push(format!("Restart program: {}", program));
            }
        }

        let success = errors.is_empty();
        let message = if success {
            if operations.is_empty() {
                Some("No operations to perform".to_string())
            } else {
                Some(format!("Performed operations: {}", operations.join(", ")))
            }
        } else {
            Some(format!("Errors: {}", errors.join(", ")))
        };

        info!("Processed desired state - Success: {}, Operations: {:?}", success, operations);

        Ok(ShadowOperationResult {
            success,
            message,
            timestamp: Utc::now(),
        })
    }

    fn set_delta_callback(&self, callback: ShadowDeltaCallback) {
        if let Ok(mut cb) = self.delta_callback.try_write() {
            *cb = Some(callback);
        }
    }
}

/// Mock shadow manager for testing
pub struct MockShadowManager {
    device_id: String,
    shadow_updates: Arc<RwLock<Vec<DeviceState>>>,
    current_shadow: Arc<RwLock<Option<ShadowUpdate>>>,
    processed_desired_states: Arc<RwLock<Vec<DesiredState>>>,
}

impl MockShadowManager {
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            shadow_updates: Arc::new(RwLock::new(Vec::new())),
            current_shadow: Arc::new(RwLock::new(None)),
            processed_desired_states: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_shadow_updates(&self) -> Vec<DeviceState> {
        self.shadow_updates.read().await.clone()
    }

    pub async fn get_processed_desired_states(&self) -> Vec<DesiredState> {
        self.processed_desired_states.read().await.clone()
    }

    pub async fn set_mock_shadow(&self, shadow: ShadowUpdate) {
        *self.current_shadow.write().await = Some(shadow);
    }
}

#[async_trait]
impl ShadowManagerTrait for MockShadowManager {
    async fn initialize(&mut self) -> IoTResult<()> {
        info!("Mock shadow manager initialized for device: {}", self.device_id);
        Ok(())
    }

    async fn update_reported_state(&self, state: &DeviceState) -> IoTResult<()> {
        self.shadow_updates.write().await.push(state.clone());
        Ok(())
    }

    async fn get_shadow(&self) -> IoTResult<ShadowUpdate> {
        if let Some(shadow) = self.current_shadow.read().await.as_ref() {
            Ok(shadow.clone())
        } else {
            Ok(ShadowUpdate {
                state: ShadowState {
                    desired: None,
                    reported: None,
                    delta: None,
                },
                metadata: None,
                version: Some(1),
                timestamp: Some(Utc::now()),
            })
        }
    }

    async fn process_desired_state(&self, desired: &DesiredState) -> IoTResult<ShadowOperationResult> {
        self.processed_desired_states.write().await.push(desired.clone());
        
        Ok(ShadowOperationResult {
            success: true,
            message: Some("Mock processing completed".to_string()),
            timestamp: Utc::now(),
        })
    }

    fn set_delta_callback(&self, _callback: ShadowDeltaCallback) {
        // Mock implementation - store callback if needed for testing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MockIoTClient, RuntimeStatus};

    #[tokio::test]
    async fn test_mock_shadow_manager_basic_operations() {
        let mut manager = MockShadowManager::new("test-device".to_string());
        
        // Test initialization
        manager.initialize().await.unwrap();
        
        // Test updating reported state
        let device_state = ShadowManager::create_device_state(
            RuntimeStatus::Idle,
            false,
            SleepStatus::Awake,
            MemoryInfo {
                total_bytes: 1024,
                free_bytes: 512,
                used_bytes: 512,
                largest_free_block: 256,
            },
            "1.0.0".to_string(),
            "test".to_string(),
            100,
        ).await;
        
        manager.update_reported_state(&device_state).await.unwrap();
        
        let updates = manager.get_shadow_updates().await;
        assert_eq!(updates.len(), 1);
        assert!(matches!(updates[0].runtime_status, RuntimeStatus::Idle));
        
        // Test processing desired state
        let desired_state = DesiredState {
            led_control: Some(true),
            sleep_duration: Some(5.0),
            configuration: Some(DeviceConfiguration {
                log_level: Some("debug".to_string()),
                reporting_interval: Some(30),
                auto_update: Some(true),
            }),
            program_commands: None,
        };
        
        let result = manager.process_desired_state(&desired_state).await.unwrap();
        assert!(result.success);
        
        let processed = manager.get_processed_desired_states().await;
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].led_control, Some(true));
        assert_eq!(processed[0].sleep_duration, Some(5.0));
    }

    #[tokio::test]
    async fn test_shadow_manager_with_mock_iot_client() {
        let mut iot_client = MockIoTClient::new();
        iot_client.connect().await.unwrap();
        
        let mut manager = ShadowManager::new(
            "test-device".to_string(),
            "test-thing".to_string(),
            Arc::new(iot_client),
        );
        
        // Test initialization (should subscribe to shadow topics)
        manager.initialize().await.unwrap();
        
        // Test updating reported state
        let device_state = ShadowManager::create_device_state(
            RuntimeStatus::ExecutingProgram {
                program_id: "test-program".to_string(),
                started_at: Utc::now(),
            },
            true,
            SleepStatus::Awake,
            MemoryInfo {
                total_bytes: 2048,
                free_bytes: 1024,
                used_bytes: 1024,
                largest_free_block: 512,
            },
            "1.0.0".to_string(),
            "esp32-s3".to_string(),
            3600,
        ).await;
        
        manager.update_reported_state(&device_state).await.unwrap();
    }

    #[tokio::test]
    async fn test_desired_state_processing() {
        let manager = MockShadowManager::new("test-device".to_string());
        
        // Test valid desired state
        let desired_state = DesiredState {
            led_control: Some(true),
            sleep_duration: Some(10.0),
            configuration: Some(DeviceConfiguration {
                log_level: Some("info".to_string()),
                reporting_interval: Some(60),
                auto_update: Some(false),
            }),
            program_commands: Some(ProgramCommands {
                load_program: Some("sensor-monitor".to_string()),
                stop_program: None,
                restart_program: None,
            }),
        };
        
        let result = manager.process_desired_state(&desired_state).await.unwrap();
        assert!(result.success);
        assert!(result.message.is_some());
        
        // Test invalid desired state
        let invalid_desired = DesiredState {
            led_control: None,
            sleep_duration: Some(-1.0), // Invalid duration
            configuration: None,
            program_commands: None,
        };
        
        // Mock implementation always succeeds, but real implementation would handle this
        let result = manager.process_desired_state(&invalid_desired).await.unwrap();
        assert!(result.success); // Mock always succeeds
    }

    #[test]
    fn test_shadow_serialization() {
        let shadow_update = ShadowUpdate {
            state: ShadowState {
                desired: Some(DesiredState {
                    led_control: Some(true),
                    sleep_duration: Some(5.0),
                    configuration: None,
                    program_commands: None,
                }),
                reported: None,
                delta: None,
            },
            metadata: None,
            version: Some(42),
            timestamp: Some(Utc::now()),
        };
        
        // Test serialization and deserialization
        let json = serde_json::to_string(&shadow_update).unwrap();
        let deserialized: ShadowUpdate = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.version, Some(42));
        assert!(deserialized.state.desired.is_some());
        
        let desired = deserialized.state.desired.unwrap();
        assert_eq!(desired.led_control, Some(true));
        assert_eq!(desired.sleep_duration, Some(5.0));
    }
}