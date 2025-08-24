use crate::{IoTError, IoTResult, IoTClientTrait, ProgramMessage, ProgramResult};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rumqttc::QoS;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Program execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProgramExecutionStatus {
    Pending,
    Loading,
    Running,
    Completed,
    Failed,
    Stopped,
}

/// Program delivery status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramDeliveryStatus {
    pub program_id: String,
    pub status: ProgramExecutionStatus,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub execution_time_ms: Option<u64>,
}

/// Program validation result
#[derive(Debug, Clone)]
pub struct ProgramValidationResult {
    pub valid: bool,
    pub checksum_match: bool,
    pub size_valid: bool,
    pub syntax_valid: bool,
    pub error_message: Option<String>,
}

/// Program execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramExecutionRequest {
    pub program_id: String,
    pub action: ProgramAction,
    pub parameters: Option<serde_json::Value>,
    pub timeout_seconds: Option<u64>,
}

/// Program action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgramAction {
    Load,
    Execute,
    Stop,
    Remove,
    Status,
}

/// Program status report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramStatusReport {
    pub device_id: String,
    pub programs: Vec<ProgramDeliveryStatus>,
    pub timestamp: DateTime<Utc>,
}

/// Callback for program execution events
pub type ProgramExecutionCallback = Arc<dyn Fn(ProgramResult) -> Result<(), IoTError> + Send + Sync>;

/// Program delivery manager trait for testability
#[async_trait]
pub trait ProgramDeliveryManagerTrait: Send + Sync {
    async fn initialize(&mut self) -> IoTResult<()>;
    async fn handle_program_message(&self, message: &ProgramMessage) -> IoTResult<ProgramDeliveryStatus>;
    async fn validate_program(&self, message: &ProgramMessage) -> IoTResult<ProgramValidationResult>;
    async fn load_program(&self, message: &ProgramMessage) -> IoTResult<String>;
    async fn execute_program(&self, program_id: &str) -> IoTResult<ProgramResult>;
    async fn stop_program(&self, program_id: &str) -> IoTResult<()>;
    async fn remove_program(&self, program_id: &str) -> IoTResult<()>;
    async fn get_program_status(&self, program_id: &str) -> IoTResult<ProgramDeliveryStatus>;
    async fn list_programs(&self) -> IoTResult<Vec<ProgramDeliveryStatus>>;
    async fn report_status(&self) -> IoTResult<()>;
    fn set_execution_callback(&self, callback: ProgramExecutionCallback);
}

/// Main program delivery manager implementation
pub struct ProgramDeliveryManager {
    device_id: String,
    iot_client: Arc<dyn IoTClientTrait>,
    programs: Arc<RwLock<HashMap<String, ProgramMessage>>>,
    program_status: Arc<RwLock<HashMap<String, ProgramDeliveryStatus>>>,
    execution_callback: Arc<RwLock<Option<ProgramExecutionCallback>>>,
    max_program_size: usize,
    #[allow(dead_code)]
    execution_timeout: u64,
}

impl ProgramDeliveryManager {
    /// Create a new program delivery manager
    pub fn new(
        device_id: String,
        iot_client: Arc<dyn IoTClientTrait>,
    ) -> Self {
        Self {
            device_id,
            iot_client,
            programs: Arc::new(RwLock::new(HashMap::new())),
            program_status: Arc::new(RwLock::new(HashMap::new())),
            execution_callback: Arc::new(RwLock::new(None)),
            max_program_size: 1024 * 1024, // 1MB default
            execution_timeout: 300, // 5 minutes default
        }
    }

    /// Get program topic for specific operation
    fn get_program_topic(&self, operation: &str) -> String {
        format!("steel-programs/{}/{}", self.device_id, operation)
    }

    /// Get broadcast program topic
    fn get_broadcast_topic(&self, operation: &str) -> String {
        format!("steel-programs/broadcast/{}", operation)
    }

    /// Subscribe to program delivery topics
    async fn subscribe_to_program_topics(&self) -> IoTResult<()> {
        let topics = vec![
            (self.get_program_topic("load"), QoS::AtLeastOnce),
            (self.get_program_topic("execute"), QoS::AtLeastOnce),
            (self.get_program_topic("stop"), QoS::AtLeastOnce),
            (self.get_program_topic("remove"), QoS::AtLeastOnce),
            (self.get_program_topic("status"), QoS::AtLeastOnce),
            (self.get_broadcast_topic("load"), QoS::AtLeastOnce),
            (self.get_broadcast_topic("execute"), QoS::AtLeastOnce),
        ];

        for (topic, qos) in topics {
            self.iot_client.subscribe(&topic, qos).await?;
            info!("Subscribed to program topic: {}", topic);
        }

        Ok(())
    }

    /// Handle incoming program messages
    pub async fn handle_program_mqtt_message(&self, topic: &str, payload: &[u8]) -> IoTResult<()> {
        let payload_str = String::from_utf8(payload.to_vec())
            .map_err(|e| IoTError::MessageParsing(format!("Invalid UTF-8: {}", e)))?;

        debug!("Received program message on topic: {}", topic);

        if topic.contains("/load") {
            self.handle_load_message(&payload_str).await?;
        } else if topic.contains("/execute") {
            self.handle_execute_message(&payload_str).await?;
        } else if topic.contains("/stop") {
            self.handle_stop_message(&payload_str).await?;
        } else if topic.contains("/remove") {
            self.handle_remove_message(&payload_str).await?;
        } else if topic.contains("/status") {
            self.handle_status_request(&payload_str).await?;
        }

        Ok(())
    }

    /// Handle program load message
    async fn handle_load_message(&self, payload: &str) -> IoTResult<()> {
        let program_message: ProgramMessage = serde_json::from_str(payload)
            .map_err(|e| IoTError::MessageParsing(format!("Failed to parse program message: {}", e)))?;

        info!("Received program load request: {}", program_message.program_id);

        let status = self.handle_program_message(&program_message).await?;
        self.report_program_status(&status).await?;

        Ok(())
    }

    /// Handle program execute message
    async fn handle_execute_message(&self, payload: &str) -> IoTResult<()> {
        let request: ProgramExecutionRequest = serde_json::from_str(payload)
            .map_err(|e| IoTError::MessageParsing(format!("Failed to parse execution request: {}", e)))?;

        info!("Received program execute request: {}", request.program_id);

        match request.action {
            ProgramAction::Execute => {
                let result = self.execute_program(&request.program_id).await?;
                if let Some(callback) = self.execution_callback.read().await.as_ref() {
                    if let Err(e) = callback(result) {
                        error!("Execution callback error: {}", e);
                    }
                }
            }
            ProgramAction::Stop => {
                self.stop_program(&request.program_id).await?;
            }
            ProgramAction::Status => {
                let status = self.get_program_status(&request.program_id).await?;
                self.report_program_status(&status).await?;
            }
            _ => {
                warn!("Unsupported program action in execute message: {:?}", request.action);
            }
        }

        Ok(())
    }

    /// Handle program stop message
    async fn handle_stop_message(&self, payload: &str) -> IoTResult<()> {
        let request: ProgramExecutionRequest = serde_json::from_str(payload)
            .map_err(|e| IoTError::MessageParsing(format!("Failed to parse stop request: {}", e)))?;

        info!("Received program stop request: {}", request.program_id);
        self.stop_program(&request.program_id).await?;

        Ok(())
    }

    /// Handle program remove message
    async fn handle_remove_message(&self, payload: &str) -> IoTResult<()> {
        let request: ProgramExecutionRequest = serde_json::from_str(payload)
            .map_err(|e| IoTError::MessageParsing(format!("Failed to parse remove request: {}", e)))?;

        info!("Received program remove request: {}", request.program_id);
        self.remove_program(&request.program_id).await?;

        Ok(())
    }

    /// Handle status request message
    async fn handle_status_request(&self, _payload: &str) -> IoTResult<()> {
        info!("Received program status request");
        self.report_status().await?;

        Ok(())
    }

    /// Report program status to AWS IoT
    async fn report_program_status(&self, status: &ProgramDeliveryStatus) -> IoTResult<()> {
        let topic = format!("steel-programs/{}/status/report", self.device_id);
        let payload = serde_json::to_vec(status)
            .map_err(|e| IoTError::MessageParsing(format!("Failed to serialize status: {}", e)))?;

        self.iot_client.publish(&topic, &payload, QoS::AtLeastOnce).await?;
        debug!("Reported program status for: {}", status.program_id);

        Ok(())
    }

    /// Calculate checksum for program code
    pub fn calculate_checksum(&self, code: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Update program status
    async fn update_program_status(
        &self,
        program_id: &str,
        status: ProgramExecutionStatus,
        message: Option<String>,
        execution_time_ms: Option<u64>,
    ) {
        let status_entry = ProgramDeliveryStatus {
            program_id: program_id.to_string(),
            status,
            message,
            timestamp: Utc::now(),
            execution_time_ms,
        };

        self.program_status.write().await.insert(program_id.to_string(), status_entry);
    }
}

#[async_trait]
impl ProgramDeliveryManagerTrait for ProgramDeliveryManager {
    async fn initialize(&mut self) -> IoTResult<()> {
        info!("Initializing program delivery manager for device: {}", self.device_id);

        // Subscribe to program delivery topics
        self.subscribe_to_program_topics().await?;

        info!("Program delivery manager initialized successfully");
        Ok(())
    }

    async fn handle_program_message(&self, message: &ProgramMessage) -> IoTResult<ProgramDeliveryStatus> {
        info!("Processing program message: {}", message.program_id);

        // Update status to loading
        self.update_program_status(
            &message.program_id,
            ProgramExecutionStatus::Loading,
            Some("Validating program".to_string()),
            None,
        ).await;

        // Validate the program
        let validation = self.validate_program(message).await?;
        if !validation.valid {
            let error_msg = validation.error_message.unwrap_or_else(|| "Validation failed".to_string());
            self.update_program_status(
                &message.program_id,
                ProgramExecutionStatus::Failed,
                Some(error_msg.clone()),
                None,
            ).await;

            return Ok(ProgramDeliveryStatus {
                program_id: message.program_id.clone(),
                status: ProgramExecutionStatus::Failed,
                message: Some(error_msg),
                timestamp: Utc::now(),
                execution_time_ms: None,
            });
        }

        // Load the program
        match self.load_program(message).await {
            Ok(program_id) => {
                self.update_program_status(
                    &program_id,
                    ProgramExecutionStatus::Pending,
                    Some("Program loaded successfully".to_string()),
                    None,
                ).await;

                // Auto-start if requested
                if message.auto_start {
                    info!("Auto-starting program: {}", program_id);
                    match self.execute_program(&program_id).await {
                        Ok(result) => {
                            self.update_program_status(
                                &program_id,
                                if result.success { ProgramExecutionStatus::Completed } else { ProgramExecutionStatus::Failed },
                                result.error.or(Some("Execution completed".to_string())),
                                Some(result.execution_time_ms),
                            ).await;
                        }
                        Err(e) => {
                            self.update_program_status(
                                &program_id,
                                ProgramExecutionStatus::Failed,
                                Some(format!("Execution failed: {}", e)),
                                None,
                            ).await;
                        }
                    }
                }

                self.get_program_status(&program_id).await
            }
            Err(e) => {
                let error_msg = format!("Failed to load program: {}", e);
                self.update_program_status(
                    &message.program_id,
                    ProgramExecutionStatus::Failed,
                    Some(error_msg.clone()),
                    None,
                ).await;

                Ok(ProgramDeliveryStatus {
                    program_id: message.program_id.clone(),
                    status: ProgramExecutionStatus::Failed,
                    message: Some(error_msg),
                    timestamp: Utc::now(),
                    execution_time_ms: None,
                })
            }
        }
    }

    async fn validate_program(&self, message: &ProgramMessage) -> IoTResult<ProgramValidationResult> {
        let mut result = ProgramValidationResult {
            valid: true,
            checksum_match: true,
            size_valid: true,
            syntax_valid: true,
            error_message: None,
        };

        // Check program size
        if message.steel_code.len() > self.max_program_size {
            result.valid = false;
            result.size_valid = false;
            result.error_message = Some(format!(
                "Program too large: {} bytes (max: {} bytes)",
                message.steel_code.len(),
                self.max_program_size
            ));
            return Ok(result);
        }

        // Verify checksum
        let calculated_checksum = self.calculate_checksum(&message.steel_code);
        if calculated_checksum != message.checksum {
            result.valid = false;
            result.checksum_match = false;
            result.error_message = Some(format!(
                "Checksum mismatch: expected {}, got {}",
                message.checksum,
                calculated_checksum
            ));
            return Ok(result);
        }

        // Basic syntax validation (check for balanced parentheses)
        let mut paren_count = 0;
        for ch in message.steel_code.chars() {
            match ch {
                '(' => paren_count += 1,
                ')' => {
                    paren_count -= 1;
                    if paren_count < 0 {
                        result.valid = false;
                        result.syntax_valid = false;
                        result.error_message = Some("Unbalanced parentheses in Steel code".to_string());
                        return Ok(result);
                    }
                }
                _ => {}
            }
        }

        if paren_count != 0 {
            result.valid = false;
            result.syntax_valid = false;
            result.error_message = Some("Unbalanced parentheses in Steel code".to_string());
            return Ok(result);
        }

        info!("Program validation successful: {}", message.program_id);
        Ok(result)
    }

    async fn load_program(&self, message: &ProgramMessage) -> IoTResult<String> {
        info!("Loading program: {}", message.program_id);

        // Store the program
        self.programs.write().await.insert(message.program_id.clone(), message.clone());

        // Update status
        self.update_program_status(
            &message.program_id,
            ProgramExecutionStatus::Pending,
            Some("Program loaded and ready for execution".to_string()),
            None,
        ).await;

        info!("Program loaded successfully: {}", message.program_id);
        Ok(message.program_id.clone())
    }

    async fn execute_program(&self, program_id: &str) -> IoTResult<ProgramResult> {
        info!("Executing program: {}", program_id);

        let start_time = std::time::Instant::now();

        // Update status to running
        self.update_program_status(
            program_id,
            ProgramExecutionStatus::Running,
            Some("Program execution started".to_string()),
            None,
        ).await;

        // Get the program
        let program = {
            let programs = self.programs.read().await;
            programs.get(program_id).cloned()
        };

        let program = program.ok_or_else(|| {
            IoTError::MessageParsing(format!("Program not found: {}", program_id))
        })?;

        // Simulate program execution (in real implementation, this would use SteelRuntime)
        let execution_result = self.simulate_program_execution(&program).await;
        let execution_time = start_time.elapsed().as_millis() as u64;

        let result = ProgramResult {
            program_id: program_id.to_string(),
            success: execution_result.is_ok(),
            result: execution_result.as_ref().ok().cloned(),
            error: execution_result.err().map(|e| e.to_string()),
            execution_time_ms: execution_time,
            timestamp: Utc::now(),
        };

        // Update final status
        self.update_program_status(
            program_id,
            if result.success { ProgramExecutionStatus::Completed } else { ProgramExecutionStatus::Failed },
            result.error.clone().or(Some("Execution completed".to_string())),
            Some(execution_time),
        ).await;

        info!("Program execution completed: {} (success: {})", program_id, result.success);
        Ok(result)
    }

    async fn stop_program(&self, program_id: &str) -> IoTResult<()> {
        info!("Stopping program: {}", program_id);

        // Update status to stopped
        self.update_program_status(
            program_id,
            ProgramExecutionStatus::Stopped,
            Some("Program execution stopped".to_string()),
            None,
        ).await;

        // In real implementation, this would signal the SteelRuntime to stop execution
        info!("Program stopped: {}", program_id);
        Ok(())
    }

    async fn remove_program(&self, program_id: &str) -> IoTResult<()> {
        info!("Removing program: {}", program_id);

        // Remove from storage
        self.programs.write().await.remove(program_id);
        self.program_status.write().await.remove(program_id);

        info!("Program removed: {}", program_id);
        Ok(())
    }

    async fn get_program_status(&self, program_id: &str) -> IoTResult<ProgramDeliveryStatus> {
        let status = self.program_status.read().await;
        status.get(program_id).cloned().ok_or_else(|| {
            IoTError::MessageParsing(format!("Program status not found: {}", program_id))
        })
    }

    async fn list_programs(&self) -> IoTResult<Vec<ProgramDeliveryStatus>> {
        let status = self.program_status.read().await;
        Ok(status.values().cloned().collect())
    }

    async fn report_status(&self) -> IoTResult<()> {
        let programs = self.list_programs().await?;
        
        let report = ProgramStatusReport {
            device_id: self.device_id.clone(),
            programs,
            timestamp: Utc::now(),
        };

        let topic = format!("steel-programs/{}/status/full-report", self.device_id);
        let payload = serde_json::to_vec(&report)
            .map_err(|e| IoTError::MessageParsing(format!("Failed to serialize status report: {}", e)))?;

        self.iot_client.publish(&topic, &payload, QoS::AtLeastOnce).await?;
        info!("Reported full program status ({} programs)", report.programs.len());

        Ok(())
    }

    fn set_execution_callback(&self, callback: ProgramExecutionCallback) {
        if let Ok(mut cb) = self.execution_callback.try_write() {
            *cb = Some(callback);
        }
    }
}

impl ProgramDeliveryManager {
    /// Simulate program execution (placeholder for real Steel integration)
    async fn simulate_program_execution(&self, program: &ProgramMessage) -> Result<String, IoTError> {
        // Simulate some processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Simple simulation based on program content
        if program.steel_code.contains("(error") {
            Err(IoTError::MessageParsing("Simulated program error".to_string()))
        } else if program.steel_code.contains("(sleep") {
            Ok("Sleep command executed".to_string())
        } else if program.steel_code.contains("(led") {
            Ok("LED command executed".to_string())
        } else {
            Ok("Program executed successfully".to_string())
        }
    }
}

/// Mock program delivery manager for testing
pub struct MockProgramDeliveryManager {
    device_id: String,
    programs: Arc<RwLock<HashMap<String, ProgramMessage>>>,
    program_status: Arc<RwLock<HashMap<String, ProgramDeliveryStatus>>>,
    execution_results: Arc<RwLock<Vec<ProgramResult>>>,
}

impl MockProgramDeliveryManager {
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            programs: Arc::new(RwLock::new(HashMap::new())),
            program_status: Arc::new(RwLock::new(HashMap::new())),
            execution_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_loaded_programs(&self) -> Vec<ProgramMessage> {
        self.programs.read().await.values().cloned().collect()
    }

    pub async fn get_execution_results(&self) -> Vec<ProgramResult> {
        self.execution_results.read().await.clone()
    }
}

#[async_trait]
impl ProgramDeliveryManagerTrait for MockProgramDeliveryManager {
    async fn initialize(&mut self) -> IoTResult<()> {
        info!("Mock program delivery manager initialized for device: {}", self.device_id);
        Ok(())
    }

    async fn handle_program_message(&self, message: &ProgramMessage) -> IoTResult<ProgramDeliveryStatus> {
        let validation = self.validate_program(message).await?;
        
        if validation.valid {
            self.load_program(message).await?;
            
            if message.auto_start {
                let result = self.execute_program(&message.program_id).await?;
                self.execution_results.write().await.push(result);
            }
        }

        self.get_program_status(&message.program_id).await
    }

    async fn validate_program(&self, message: &ProgramMessage) -> IoTResult<ProgramValidationResult> {
        // Simple mock validation
        let valid = !message.steel_code.is_empty() && message.steel_code.len() < 10000;
        
        Ok(ProgramValidationResult {
            valid,
            checksum_match: true, // Mock always passes checksum
            size_valid: message.steel_code.len() < 10000,
            syntax_valid: valid,
            error_message: if valid { None } else { Some("Mock validation failed".to_string()) },
        })
    }

    async fn load_program(&self, message: &ProgramMessage) -> IoTResult<String> {
        self.programs.write().await.insert(message.program_id.clone(), message.clone());
        
        let status = ProgramDeliveryStatus {
            program_id: message.program_id.clone(),
            status: ProgramExecutionStatus::Pending,
            message: Some("Mock program loaded".to_string()),
            timestamp: Utc::now(),
            execution_time_ms: None,
        };
        
        self.program_status.write().await.insert(message.program_id.clone(), status);
        Ok(message.program_id.clone())
    }

    async fn execute_program(&self, program_id: &str) -> IoTResult<ProgramResult> {
        let result = ProgramResult {
            program_id: program_id.to_string(),
            success: true,
            result: Some("Mock execution successful".to_string()),
            error: None,
            execution_time_ms: 50,
            timestamp: Utc::now(),
        };

        let status = ProgramDeliveryStatus {
            program_id: program_id.to_string(),
            status: ProgramExecutionStatus::Completed,
            message: Some("Mock execution completed".to_string()),
            timestamp: Utc::now(),
            execution_time_ms: Some(50),
        };

        self.program_status.write().await.insert(program_id.to_string(), status);
        Ok(result)
    }

    async fn stop_program(&self, program_id: &str) -> IoTResult<()> {
        if let Some(status) = self.program_status.write().await.get_mut(program_id) {
            status.status = ProgramExecutionStatus::Stopped;
            status.message = Some("Mock program stopped".to_string());
            status.timestamp = Utc::now();
        }
        Ok(())
    }

    async fn remove_program(&self, program_id: &str) -> IoTResult<()> {
        self.programs.write().await.remove(program_id);
        self.program_status.write().await.remove(program_id);
        Ok(())
    }

    async fn get_program_status(&self, program_id: &str) -> IoTResult<ProgramDeliveryStatus> {
        self.program_status.read().await.get(program_id).cloned().ok_or_else(|| {
            IoTError::MessageParsing(format!("Program status not found: {}", program_id))
        })
    }

    async fn list_programs(&self) -> IoTResult<Vec<ProgramDeliveryStatus>> {
        Ok(self.program_status.read().await.values().cloned().collect())
    }

    async fn report_status(&self) -> IoTResult<()> {
        // Mock implementation - just log
        let programs = self.list_programs().await?;
        info!("Mock status report: {} programs", programs.len());
        Ok(())
    }

    fn set_execution_callback(&self, _callback: ProgramExecutionCallback) {
        // Mock implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iot_client::MockIoTClient;

    #[tokio::test]
    async fn test_mock_program_delivery_manager_basic_operations() {
        let mut manager = MockProgramDeliveryManager::new("test-device".to_string());
        
        // Test initialization
        manager.initialize().await.unwrap();
        
        // Create test program message
        let program_message = ProgramMessage {
            program_id: "test-program-001".to_string(),
            program_name: "Test Program".to_string(),
            steel_code: "(begin (log \"info\" \"Hello from Steel!\") (led-on) (sleep 1) (led-off))".to_string(),
            version: "1.0.0".to_string(),
            checksum: "abc123".to_string(),
            auto_start: false,
            metadata: Some(crate::types::ProgramMetadata {
                description: Some("Test program description".to_string()),
                author: Some("Test Author".to_string()),
                created_at: Utc::now(),
                memory_requirement: Some(1024),
                execution_timeout: Some(30),
            }),
        };
        
        // Test program validation
        let validation = manager.validate_program(&program_message).await.unwrap();
        assert!(validation.valid);
        assert!(validation.checksum_match);
        assert!(validation.size_valid);
        assert!(validation.syntax_valid);
        
        // Test program loading
        let program_id = manager.load_program(&program_message).await.unwrap();
        assert_eq!(program_id, "test-program-001");
        
        let loaded_programs = manager.get_loaded_programs().await;
        assert_eq!(loaded_programs.len(), 1);
        assert_eq!(loaded_programs[0].program_id, "test-program-001");
        
        // Test program execution
        let result = manager.execute_program(&program_id).await.unwrap();
        assert!(result.success);
        assert_eq!(result.program_id, "test-program-001");
        assert!(result.result.is_some());
        
        // Test program status
        let status = manager.get_program_status(&program_id).await.unwrap();
        assert_eq!(status.program_id, "test-program-001");
        assert_eq!(status.status, ProgramExecutionStatus::Completed);
        
        // Test program removal
        manager.remove_program(&program_id).await.unwrap();
        let loaded_programs = manager.get_loaded_programs().await;
        assert_eq!(loaded_programs.len(), 0);
    }

    #[tokio::test]
    async fn test_program_message_handling() {
        let manager = MockProgramDeliveryManager::new("test-device-002".to_string());
        
        // Test auto-start program
        let auto_start_program = ProgramMessage {
            program_id: "auto-start-program".to_string(),
            program_name: "Auto Start Test".to_string(),
            steel_code: "(begin (log \"info\" \"Auto-started program\"))".to_string(),
            version: "1.0.0".to_string(),
            checksum: "def456".to_string(),
            auto_start: true,
            metadata: None,
        };
        
        let status = manager.handle_program_message(&auto_start_program).await.unwrap();
        assert_eq!(status.program_id, "auto-start-program");
        
        // Verify execution occurred
        let execution_results = manager.get_execution_results().await;
        assert_eq!(execution_results.len(), 1);
        assert!(execution_results[0].success);
        assert_eq!(execution_results[0].program_id, "auto-start-program");
    }

    #[tokio::test]
    async fn test_program_validation() {
        let manager = MockProgramDeliveryManager::new("test-device-003".to_string());
        
        // Test valid program
        let valid_program = ProgramMessage {
            program_id: "valid-program".to_string(),
            program_name: "Valid Program".to_string(),
            steel_code: "(begin (+ 1 2 3))".to_string(),
            version: "1.0.0".to_string(),
            checksum: "valid123".to_string(),
            auto_start: false,
            metadata: None,
        };
        
        let validation = manager.validate_program(&valid_program).await.unwrap();
        assert!(validation.valid);
        
        // Test invalid program (empty code)
        let invalid_program = ProgramMessage {
            program_id: "invalid-program".to_string(),
            program_name: "Invalid Program".to_string(),
            steel_code: "".to_string(),
            version: "1.0.0".to_string(),
            checksum: "invalid123".to_string(),
            auto_start: false,
            metadata: None,
        };
        
        let validation = manager.validate_program(&invalid_program).await.unwrap();
        assert!(!validation.valid);
        assert!(validation.error_message.is_some());
    }

    #[tokio::test]
    async fn test_program_lifecycle() {
        let manager = MockProgramDeliveryManager::new("test-device-004".to_string());
        
        let program_message = ProgramMessage {
            program_id: "lifecycle-test".to_string(),
            program_name: "Lifecycle Test Program".to_string(),
            steel_code: "(begin (log \"info\" \"Lifecycle test\"))".to_string(),
            version: "1.0.0".to_string(),
            checksum: "lifecycle123".to_string(),
            auto_start: false,
            metadata: None,
        };
        
        // Load program
        let program_id = manager.load_program(&program_message).await.unwrap();
        let status = manager.get_program_status(&program_id).await.unwrap();
        assert_eq!(status.status, ProgramExecutionStatus::Pending);
        
        // Execute program
        let result = manager.execute_program(&program_id).await.unwrap();
        assert!(result.success);
        
        let status = manager.get_program_status(&program_id).await.unwrap();
        assert_eq!(status.status, ProgramExecutionStatus::Completed);
        
        // Stop program
        manager.stop_program(&program_id).await.unwrap();
        let status = manager.get_program_status(&program_id).await.unwrap();
        assert_eq!(status.status, ProgramExecutionStatus::Stopped);
        
        // Remove program
        manager.remove_program(&program_id).await.unwrap();
        let result = manager.get_program_status(&program_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_program_delivery_manager_with_iot_client() {
        let mut iot_client = MockIoTClient::new();
        iot_client.connect().await.unwrap();
        
        let mut manager = ProgramDeliveryManager::new(
            "integration-test-device".to_string(),
            Arc::new(iot_client),
        );
        
        // Test initialization
        manager.initialize().await.unwrap();
        
        // Test program validation
        let program_message = ProgramMessage {
            program_id: "integration-test-program".to_string(),
            program_name: "Integration Test".to_string(),
            steel_code: "(begin (log \"info\" \"Integration test program\") (+ 1 2 3))".to_string(),
            version: "1.0.0".to_string(),
            checksum: manager.calculate_checksum("(begin (log \"info\" \"Integration test program\") (+ 1 2 3))"),
            auto_start: true,
            metadata: None,
        };
        
        let validation = manager.validate_program(&program_message).await.unwrap();
        assert!(validation.valid);
        assert!(validation.checksum_match);
        
        // Test program handling
        let status = manager.handle_program_message(&program_message).await.unwrap();
        assert_eq!(status.program_id, "integration-test-program");
    }
}