use async_trait::async_trait;
use aws_iot_core::{
    IoTResult, IoTError, DeviceState, ProgramMessage, 
    ConnectionStatus, IoTClientTrait
};
use aws_iot_core::iot_client::{MessageCallback, SubscriptionHandle};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

use rumqttc::QoS;

/// Mock implementation of IoTClient for testing
#[derive(Debug)]
pub struct MockIoTClient {
    device_id: String,
    connected: Arc<Mutex<bool>>,
    published_messages: Arc<Mutex<Vec<PublishedMessage>>>,
    subscriptions: Arc<Mutex<Vec<String>>>,
    shadow_state: Arc<Mutex<DeviceState>>,
    program_messages: Arc<Mutex<Vec<ProgramMessage>>>,
    connection_attempts: Arc<Mutex<u32>>,
    should_fail_connection: Arc<Mutex<bool>>,
    should_fail_publish: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone)]
pub struct PublishedMessage {
    pub topic: String,
    pub payload: String,
    pub timestamp: DateTime<Utc>,
}

impl MockIoTClient {
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            connected: Arc::new(Mutex::new(false)),
            published_messages: Arc::new(Mutex::new(Vec::new())),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            shadow_state: Arc::new(Mutex::new(DeviceState::default())),
            program_messages: Arc::new(Mutex::new(Vec::new())),
            connection_attempts: Arc::new(Mutex::new(0)),
            should_fail_connection: Arc::new(Mutex::new(false)),
            should_fail_publish: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Get all published messages for testing
    pub async fn get_published_messages(&self) -> Vec<PublishedMessage> {
        self.published_messages.lock().await.clone()
    }
    
    /// Get all subscriptions for testing
    pub async fn get_subscriptions(&self) -> Vec<String> {
        self.subscriptions.lock().await.clone()
    }
    
    /// Get current shadow state for testing
    pub async fn get_shadow_state(&self) -> DeviceState {
        self.shadow_state.lock().await.clone()
    }
    
    /// Get program messages for testing
    pub async fn get_program_messages(&self) -> Vec<ProgramMessage> {
        self.program_messages.lock().await.clone()
    }
    
    /// Get connection attempt count
    pub async fn get_connection_attempts(&self) -> u32 {
        *self.connection_attempts.lock().await
    }
    
    /// Set whether connection should fail
    pub async fn set_should_fail_connection(&self, should_fail: bool) {
        *self.should_fail_connection.lock().await = should_fail;
    }
    
    /// Set whether publish should fail
    pub async fn set_should_fail_publish(&self, should_fail: bool) {
        *self.should_fail_publish.lock().await = should_fail;
    }
    
    /// Simulate receiving a program message
    pub async fn simulate_program_message(&self, message: ProgramMessage) {
        self.program_messages.lock().await.push(message);
    }
    
    /// Clear all test data
    pub async fn clear_test_data(&self) {
        self.published_messages.lock().await.clear();
        self.subscriptions.lock().await.clear();
        self.program_messages.lock().await.clear();
        *self.connection_attempts.lock().await = 0;
    }
}

#[async_trait]
impl aws_iot_core::IoTClientTrait for MockIoTClient {
    async fn connect(&mut self) -> IoTResult<()> {
        let mut attempts = self.connection_attempts.lock().await;
        *attempts += 1;
        
        if *self.should_fail_connection.lock().await {
            return Err(IoTError::Connection("Mock connection failure".to_string()));
        }
        
        *self.connected.lock().await = true;
        Ok(())
    }
    
    async fn disconnect(&mut self) -> IoTResult<()> {
        *self.connected.lock().await = false;
        Ok(())
    }
    
    async fn publish(&self, topic: &str, payload: &[u8], _qos: QoS) -> IoTResult<()> {
        if !*self.connected.lock().await {
            return Err(IoTError::NotConnected);
        }
        
        if *self.should_fail_publish.lock().await {
            return Err(IoTError::Publish("Mock publish failure".to_string()));
        }
        
        let message = PublishedMessage {
            topic: topic.to_string(),
            payload: String::from_utf8_lossy(payload).to_string(),
            timestamp: Utc::now(),
        };
        
        self.published_messages.lock().await.push(message);
        Ok(())
    }
    
    async fn subscribe(&self, topic: &str, _qos: QoS) -> IoTResult<SubscriptionHandle> {
        if !*self.connected.lock().await {
            return Err(IoTError::NotConnected);
        }
        
        self.subscriptions.lock().await.push(topic.to_string());
        Ok(topic.to_string())
    }
    
    async fn unsubscribe(&self, topic: &str) -> IoTResult<()> {
        if !*self.connected.lock().await {
            return Err(IoTError::NotConnected);
        }
        
        let mut subscriptions = self.subscriptions.lock().await;
        subscriptions.retain(|t| t != topic);
        Ok(())
    }
    
    async fn update_shadow(&self, state: &DeviceState) -> IoTResult<()> {
        if !*self.connected.lock().await {
            return Err(IoTError::Connection("Not connected".to_string()));
        }
        
        *self.shadow_state.lock().await = state.clone();
        Ok(())
    }
    
    fn get_connection_status(&self) -> ConnectionStatus {
        if self.connected.try_lock().is_ok_and(|guard| *guard) {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        }
    }
    
    async fn get_shadow(&self) -> IoTResult<DeviceState> {
        if !*self.connected.lock().await {
            return Err(IoTError::NotConnected);
        }
        
        Ok(self.shadow_state.lock().await.clone())
    }
    
    async fn subscribe_to_program_topics(&self) -> IoTResult<()> {
        if !*self.connected.lock().await {
            return Err(IoTError::NotConnected);
        }
        
        let topics = vec![
            format!("steel-programs/{}/load", self.device_id),
            format!("steel-programs/{}/execute", self.device_id),
            format!("steel-programs/{}/status", self.device_id),
            format!("steel-programs/{}/eval", self.device_id),
        ];
        
        for topic in topics {
            self.subscriptions.lock().await.push(topic);
        }
        
        Ok(())
    }

    fn set_message_callback(&self, _callback: MessageCallback) {
        // Mock implementation - store callback if needed
    }
}

// Additional helper methods for testing (not part of the trait)
impl MockIoTClient {
    /// Check if connected (helper method for tests)
    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }
    
    /// Publish string payload (helper method for tests)
    pub async fn publish_string(&self, topic: &str, payload: &str) -> IoTResult<()> {
        self.publish(topic, payload.as_bytes(), QoS::AtMostOnce).await
    }
    
    /// Handle program delivery (helper method for tests)
    pub async fn handle_program_delivery(&self, message: ProgramMessage) -> IoTResult<()> {
        self.program_messages.lock().await.push(message);
        Ok(())
    }
    
    /// Request program (helper method for tests)
    pub async fn request_program(&self, program_name: &str) -> IoTResult<()> {
        let request_topic = format!("steel-programs/{}/request", self.device_id);
        let request_payload = serde_json::json!({
            "program_name": program_name,
            "timestamp": Utc::now()
        });
        
        self.publish_string(&request_topic, &request_payload.to_string()).await
    }
    
    /// Get device ID (helper method for tests)
    pub fn device_id(&self) -> &str {
        &self.device_id
    }
}