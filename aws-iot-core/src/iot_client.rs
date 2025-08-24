use crate::{IoTError, IoTResult, IoTConfig, MqttMessage, DeviceState, ConnectionStatus};

use async_trait::async_trait;
use chrono::Utc;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS, TlsConfiguration, Transport};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use url::Url;

// Type alias to simplify complex types
type PublishedMessages = Arc<Mutex<Vec<(String, Vec<u8>)>>>;

/// MQTT subscription handle
pub type SubscriptionHandle = String;

/// Callback for handling incoming MQTT messages
pub type MessageCallback = Arc<dyn Fn(MqttMessage) -> Result<(), IoTError> + Send + Sync>;

/// IoT client trait for testability
#[async_trait]
pub trait IoTClientTrait: Send + Sync {
    async fn connect(&mut self) -> IoTResult<()>;
    async fn disconnect(&mut self) -> IoTResult<()>;
    async fn publish(&self, topic: &str, payload: &[u8], qos: QoS) -> IoTResult<()>;
    async fn subscribe(&self, topic: &str, qos: QoS) -> IoTResult<SubscriptionHandle>;
    async fn unsubscribe(&self, topic: &str) -> IoTResult<()>;
    async fn update_shadow(&self, state: &DeviceState) -> IoTResult<()>;
    async fn get_shadow(&self) -> IoTResult<DeviceState>;
    async fn subscribe_to_program_topics(&self) -> IoTResult<()>;
    fn get_connection_status(&self) -> ConnectionStatus;
    fn set_message_callback(&self, callback: MessageCallback);
}

/// Main IoT client implementation
pub struct IoTClient {
    config: IoTConfig,
    mqtt_client: Option<AsyncClient>,
    connection_status: Arc<RwLock<ConnectionStatus>>,
    subscriptions: Arc<RwLock<HashMap<String, QoS>>>,
    message_callback: Arc<RwLock<Option<MessageCallback>>>,
    reconnect_attempts: Arc<RwLock<u32>>,
    max_reconnect_attempts: u32,
    reconnect_delay: Duration,
}

impl IoTClient {
    /// Create a new IoT client with the given configuration
    pub fn new(config: IoTConfig) -> Self {
        Self {
            config,
            mqtt_client: None,
            connection_status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            message_callback: Arc::new(RwLock::new(None)),
            reconnect_attempts: Arc::new(RwLock::new(0)),
            max_reconnect_attempts: 10,
            reconnect_delay: Duration::from_secs(5),
        }
    }

    /// Create MQTT options from configuration
    fn create_mqtt_options(&self) -> IoTResult<MqttOptions> {
        let client_id = self.config.client_id.clone()
            .unwrap_or_else(|| format!("{}-{}", self.config.device_id, uuid::Uuid::new_v4()));

        // Parse endpoint URL
        let endpoint_url = if self.config.endpoint.starts_with("http") {
            Url::parse(&self.config.endpoint)
                .map_err(|e| IoTError::Configuration(format!("Invalid endpoint URL: {}", e)))?
        } else {
            Url::parse(&format!("mqtts://{}", self.config.endpoint))
                .map_err(|e| IoTError::Configuration(format!("Invalid endpoint: {}", e)))?
        };

        let host = endpoint_url.host_str()
            .ok_or_else(|| IoTError::Configuration("No host in endpoint URL".to_string()))?;
        let port = endpoint_url.port().unwrap_or(8883);

        let mut mqtt_options = MqttOptions::new(client_id, host, port);
        mqtt_options.set_keep_alive(Duration::from_secs(self.config.keep_alive_secs as u64));
        mqtt_options.set_clean_session(self.config.clean_session);

        // Configure TLS if certificates are provided
        if let (Some(cert_path), Some(key_path)) = (&self.config.certificate_path, &self.config.private_key_path) {
            let tls_config = self.create_tls_config(cert_path, key_path)?;
            mqtt_options.set_transport(Transport::Tls(tls_config));
        }

        Ok(mqtt_options)
    }

    /// Create TLS configuration from certificate files
    fn create_tls_config(&self, cert_path: &str, key_path: &str) -> IoTResult<TlsConfiguration> {
        // Load client certificate
        let cert_file = std::fs::read(cert_path)
            .map_err(|e| IoTError::Configuration(format!("Failed to read certificate: {}", e)))?;
        
        let cert_chain = rustls_pemfile::certs(&mut cert_file.as_slice())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| IoTError::Configuration(format!("Failed to parse certificate: {}", e)))?;

        // Load private key
        let key_file = std::fs::read(key_path)
            .map_err(|e| IoTError::Configuration(format!("Failed to read private key: {}", e)))?;
        
        let private_key = rustls_pemfile::private_key(&mut key_file.as_slice())
            .map_err(|e| IoTError::Configuration(format!("Failed to parse private key: {}", e)))?
            .ok_or_else(|| IoTError::Configuration("No private key found".to_string()))?;

        // Create TLS configuration
        let mut root_cert_store = rustls::RootCertStore::empty();
        
        // Add CA certificate if provided
        if let Some(ca_path) = &self.config.ca_cert_path {
            let ca_file = std::fs::read(ca_path)
                .map_err(|e| IoTError::Configuration(format!("Failed to read CA certificate: {}", e)))?;
            
            let ca_certs = rustls_pemfile::certs(&mut ca_file.as_slice())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| IoTError::Configuration(format!("Failed to parse CA certificate: {}", e)))?;
            
            for cert in ca_certs {
                root_cert_store.add(cert)
                    .map_err(|e| IoTError::Configuration(format!("Failed to add CA certificate: {}", e)))?;
            }
        } else {
            // Use system root certificates
            root_cert_store.extend(
                webpki_roots::TLS_SERVER_ROOTS.iter().cloned()
            );
        }

        let client_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_client_auth_cert(cert_chain, private_key)
            .map_err(|e| IoTError::Configuration(format!("Failed to create TLS config: {}", e)))?;

        Ok(TlsConfiguration::Rustls(Arc::new(client_config)))
    }

    /// Validate topic name according to AWS IoT rules
    fn validate_topic(&self, topic: &str) -> IoTResult<()> {
        if topic.is_empty() {
            return Err(IoTError::TopicValidation("Topic cannot be empty".to_string()));
        }

        if topic.len() > 256 {
            return Err(IoTError::TopicValidation("Topic too long (max 256 characters)".to_string()));
        }

        // Check for invalid characters
        if topic.contains('\0') || topic.contains('\n') || topic.contains('\r') {
            return Err(IoTError::TopicValidation("Topic contains invalid characters".to_string()));
        }

        // Check for valid AWS IoT topic structure
        if topic.starts_with('$') && !topic.starts_with("$aws/") {
            return Err(IoTError::TopicValidation("Invalid reserved topic prefix".to_string()));
        }

        Ok(())
    }

    /// Start the event loop to handle MQTT events
    fn start_event_loop(
        mut event_loop: EventLoop,
        connection_status: Arc<RwLock<ConnectionStatus>>,
        _subscriptions: Arc<RwLock<HashMap<String, QoS>>>,
        message_callback: Arc<RwLock<Option<MessageCallback>>>,
        reconnect_attempts: Arc<RwLock<u32>>,
        max_attempts: u32,
        reconnect_delay: Duration,
    ) {
        tokio::spawn(async move {
            loop {
                match event_loop.poll().await {
                    Ok(Event::Incoming(Packet::ConnAck(_))) => {
                        info!("Connected to AWS IoT Core");
                        *connection_status.write().await = ConnectionStatus::Connected;
                        *reconnect_attempts.write().await = 0;

                        // Note: Resubscription should be handled by the client, not the event loop
                        // The event loop only processes incoming/outgoing packets
                    }
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        debug!("Received message on topic: {}", publish.topic);
                        
                        let message = MqttMessage {
                            topic: publish.topic,
                            payload: publish.payload.to_vec(),
                            qos: publish.qos as u8,
                            retain: publish.retain,
                            timestamp: Utc::now(),
                        };

                        if let Some(callback) = message_callback.read().await.as_ref() {
                            if let Err(e) = callback(message) {
                                error!("Message callback error: {}", e);
                            }
                        }
                    }
                    Ok(Event::Incoming(Packet::Disconnect)) => {
                        warn!("Disconnected from AWS IoT Core");
                        *connection_status.write().await = ConnectionStatus::Disconnected;
                    }
                    Ok(Event::Incoming(_)) => {
                        // Other incoming packets (PubAck, PubRec, etc.) - no action needed
                    }
                    Ok(Event::Outgoing(_)) => {
                        // Outgoing packet, no action needed
                    }
                    Err(e) => {
                        error!("MQTT event loop error: {}", e);
                        *connection_status.write().await = ConnectionStatus::Error;

                        // Implement exponential backoff for reconnection
                        let attempts = *reconnect_attempts.read().await;
                        if attempts < max_attempts {
                            *connection_status.write().await = ConnectionStatus::Reconnecting;
                            let delay = reconnect_delay * 2_u32.pow(attempts.min(5));
                            warn!("Reconnecting in {:?} (attempt {})", delay, attempts + 1);
                            
                            sleep(delay).await;
                            *reconnect_attempts.write().await = attempts + 1;
                        } else {
                            error!("Max reconnection attempts reached, giving up");
                            break;
                        }
                    }
                }
            }
        });
    }

    /// Get the shadow topic for this device
    fn get_shadow_topic(&self, operation: &str) -> String {
        format!("$aws/thing/{}/shadow/{}", self.config.thing_name, operation)
    }

    /// Get device-specific program topic
    #[allow(dead_code)]
    fn get_program_topic(&self, operation: &str) -> String {
        format!("steel-programs/{}/{}", self.config.device_id, operation)
    }
}

#[async_trait]
impl IoTClientTrait for IoTClient {
    async fn connect(&mut self) -> IoTResult<()> {
        info!("Connecting to AWS IoT Core endpoint: {}", self.config.endpoint);
        
        *self.connection_status.write().await = ConnectionStatus::Connecting;

        let mqtt_options = self.create_mqtt_options()?;
        let (client, event_loop) = AsyncClient::new(mqtt_options, 10);

        self.mqtt_client = Some(client);

        // Start the event loop in a separate task
        Self::start_event_loop(
            event_loop,
            Arc::clone(&self.connection_status),
            Arc::clone(&self.subscriptions),
            Arc::clone(&self.message_callback),
            Arc::clone(&self.reconnect_attempts),
            self.max_reconnect_attempts,
            self.reconnect_delay,
        );

        // Wait for connection with timeout
        let connection_timeout = Duration::from_secs(30);
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < connection_timeout {
            let status = *self.connection_status.read().await;
            match status {
                ConnectionStatus::Connected => {
                    info!("Successfully connected to AWS IoT Core");
                    return Ok(());
                }
                ConnectionStatus::Error => {
                    return Err(IoTError::Connection("Failed to connect".to_string()));
                }
                _ => {
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }

        Err(IoTError::Timeout("Connection timeout".to_string()))
    }

    async fn disconnect(&mut self) -> IoTResult<()> {
        info!("Disconnecting from AWS IoT Core");
        
        if let Some(client) = &self.mqtt_client {
            client.disconnect().await
                .map_err(|e| IoTError::Connection(format!("Disconnect failed: {}", e)))?;
        }

        *self.connection_status.write().await = ConnectionStatus::Disconnected;
        self.mqtt_client = None;

        Ok(())
    }

    async fn publish(&self, topic: &str, payload: &[u8], qos: QoS) -> IoTResult<()> {
        self.validate_topic(topic)?;

        let client = self.mqtt_client.as_ref()
            .ok_or_else(|| IoTError::Connection("Not connected".to_string()))?;

        debug!("Publishing to topic: {} (payload size: {} bytes)", topic, payload.len());

        client.publish(topic, qos, false, payload).await
            .map_err(|e| IoTError::Mqtt(format!("Publish failed: {}", e)))?;

        Ok(())
    }

    async fn subscribe(&self, topic: &str, qos: QoS) -> IoTResult<SubscriptionHandle> {
        self.validate_topic(topic)?;

        let client = self.mqtt_client.as_ref()
            .ok_or_else(|| IoTError::Connection("Not connected".to_string()))?;

        info!("Subscribing to topic: {}", topic);

        client.subscribe(topic, qos).await
            .map_err(|e| IoTError::Mqtt(format!("Subscribe failed: {}", e)))?;

        // Store subscription for reconnection
        self.subscriptions.write().await.insert(topic.to_string(), qos);

        Ok(topic.to_string())
    }

    async fn unsubscribe(&self, topic: &str) -> IoTResult<()> {
        let client = self.mqtt_client.as_ref()
            .ok_or_else(|| IoTError::Connection("Not connected".to_string()))?;

        info!("Unsubscribing from topic: {}", topic);

        client.unsubscribe(topic).await
            .map_err(|e| IoTError::Mqtt(format!("Unsubscribe failed: {}", e)))?;

        // Remove from stored subscriptions
        self.subscriptions.write().await.remove(topic);

        Ok(())
    }

    async fn update_shadow(&self, state: &DeviceState) -> IoTResult<()> {
        let shadow_topic = self.get_shadow_topic("update");
        
        let shadow_update = serde_json::json!({
            "state": {
                "reported": state
            }
        });

        let payload = serde_json::to_vec(&shadow_update)
            .map_err(|e| IoTError::MessageParsing(format!("Failed to serialize shadow update: {}", e)))?;

        self.publish(&shadow_topic, &payload, QoS::AtLeastOnce).await?;

        debug!("Updated device shadow");
        Ok(())
    }

    async fn get_shadow(&self) -> IoTResult<DeviceState> {
        let shadow_topic = self.get_shadow_topic("get");
        
        // Publish empty message to get shadow
        self.publish(&shadow_topic, &[], QoS::AtLeastOnce).await?;
        
        // In a real implementation, we would wait for the response on the accepted topic
        // For now, return a default state as this is mainly used for testing
        use crate::types::{RuntimeStatus, HardwareState, SystemInfo, SleepStatus, MemoryInfo};
        use chrono::Utc;
        
        Ok(DeviceState {
            runtime_status: RuntimeStatus::Idle,
            hardware_state: HardwareState {
                led_status: false,
                sleep_status: SleepStatus::Awake,
                memory_usage: MemoryInfo {
                    total_bytes: 1024,
                    free_bytes: 512,
                    used_bytes: 512,
                    largest_free_block: 256,
                },
            },
            system_info: SystemInfo {
                firmware_version: "1.0.0".to_string(),
                platform: "test".to_string(),
                uptime_seconds: 100,
                steel_runtime_version: "0.7.0".to_string(),
            },
            timestamp: Utc::now(),
        })
    }

    async fn subscribe_to_program_topics(&self) -> IoTResult<()> {
        let topics = vec![
            self.get_program_topic("load"),
            self.get_program_topic("execute"),
            self.get_program_topic("remove"),
        ];

        for topic in topics {
            self.subscribe(&topic, QoS::AtLeastOnce).await?;
        }

        debug!("Subscribed to program topics");
        Ok(())
    }

    fn get_connection_status(&self) -> ConnectionStatus {
        // Use try_read for non-blocking access
        match self.connection_status.try_read() {
            Ok(status) => *status,
            Err(_) => ConnectionStatus::Disconnected, // Default if lock is held
        }
    }

    fn set_message_callback(&self, callback: MessageCallback) {
        // Use try_write for non-blocking access
        if let Ok(mut cb) = self.message_callback.try_write() {
            *cb = Some(callback);
        }
        // If we can't get the lock, the callback will be set later
    }
}

/// Mock IoT client for testing
pub struct MockIoTClient {
    connection_status: ConnectionStatus,
    published_messages: PublishedMessages,
    subscriptions: Arc<Mutex<Vec<String>>>,
    shadow_updates: Arc<Mutex<Vec<DeviceState>>>,
}

impl Default for MockIoTClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MockIoTClient {
    pub fn new() -> Self {
        Self {
            connection_status: ConnectionStatus::Disconnected,
            published_messages: Arc::new(Mutex::new(Vec::new())),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            shadow_updates: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn get_published_messages(&self) -> Vec<(String, Vec<u8>)> {
        self.published_messages.lock().await.clone()
    }

    pub async fn get_subscriptions(&self) -> Vec<String> {
        self.subscriptions.lock().await.clone()
    }

    pub async fn get_shadow_updates(&self) -> Vec<DeviceState> {
        self.shadow_updates.lock().await.clone()
    }

    pub fn set_connection_status(&mut self, status: ConnectionStatus) {
        self.connection_status = status;
    }
}

#[async_trait]
impl IoTClientTrait for MockIoTClient {
    async fn connect(&mut self) -> IoTResult<()> {
        self.connection_status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn disconnect(&mut self) -> IoTResult<()> {
        self.connection_status = ConnectionStatus::Disconnected;
        Ok(())
    }

    async fn publish(&self, topic: &str, payload: &[u8], _qos: QoS) -> IoTResult<()> {
        self.published_messages.lock().await.push((topic.to_string(), payload.to_vec()));
        Ok(())
    }

    async fn subscribe(&self, topic: &str, _qos: QoS) -> IoTResult<SubscriptionHandle> {
        self.subscriptions.lock().await.push(topic.to_string());
        Ok(topic.to_string())
    }

    async fn unsubscribe(&self, topic: &str) -> IoTResult<()> {
        let mut subs = self.subscriptions.lock().await;
        subs.retain(|t| t != topic);
        Ok(())
    }

    async fn update_shadow(&self, state: &DeviceState) -> IoTResult<()> {
        self.shadow_updates.lock().await.push(state.clone());
        Ok(())
    }

    async fn get_shadow(&self) -> IoTResult<DeviceState> {
        if self.connection_status != ConnectionStatus::Connected {
            return Err(IoTError::NotConnected);
        }
        
        // Return the last shadow update or a default state
        let shadow_updates = self.shadow_updates.lock().await;
        if let Some(last_state) = shadow_updates.last() {
            Ok(last_state.clone())
        } else {
            use crate::types::{RuntimeStatus, HardwareState, SystemInfo, SleepStatus, MemoryInfo};
            use chrono::Utc;
            
            Ok(DeviceState {
                runtime_status: RuntimeStatus::Idle,
                hardware_state: HardwareState {
                    led_status: false,
                    sleep_status: SleepStatus::Awake,
                    memory_usage: MemoryInfo {
                        total_bytes: 1024,
                        free_bytes: 512,
                        used_bytes: 512,
                        largest_free_block: 256,
                    },
                },
                system_info: SystemInfo {
                    firmware_version: "1.0.0".to_string(),
                    platform: "test".to_string(),
                    uptime_seconds: 100,
                    steel_runtime_version: "0.7.0".to_string(),
                },
                timestamp: Utc::now(),
            })
        }
    }

    async fn subscribe_to_program_topics(&self) -> IoTResult<()> {
        if self.connection_status != ConnectionStatus::Connected {
            return Err(IoTError::NotConnected);
        }
        
        let topics = vec![
            "steel-programs/test-device/load".to_string(),
            "steel-programs/test-device/execute".to_string(),
            "steel-programs/test-device/remove".to_string(),
        ];

        let mut subs = self.subscriptions.lock().await;
        for topic in topics {
            subs.push(topic);
        }
        
        Ok(())
    }

    fn get_connection_status(&self) -> ConnectionStatus {
        self.connection_status
    }

    fn set_message_callback(&self, _callback: MessageCallback) {
        // Mock implementation - store callback if needed for testing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{RuntimeStatus, HardwareState, SystemInfo, SleepStatus, MemoryInfo};

    #[tokio::test]
    async fn test_mock_iot_client_basic_operations() {
        let mut client = MockIoTClient::new();
        
        // Test connection
        assert_eq!(client.get_connection_status(), ConnectionStatus::Disconnected);
        client.connect().await.unwrap();
        assert_eq!(client.get_connection_status(), ConnectionStatus::Connected);

        // Test subscription
        client.subscribe("test/topic", QoS::AtMostOnce).await.unwrap();
        let subscriptions = client.get_subscriptions().await;
        assert_eq!(subscriptions, vec!["test/topic"]);

        // Test publishing
        client.publish("test/topic", b"test message", QoS::AtMostOnce).await.unwrap();
        let messages = client.get_published_messages().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, "test/topic");
        assert_eq!(messages[0].1, b"test message");

        // Test shadow update
        let device_state = DeviceState {
            runtime_status: RuntimeStatus::Idle,
            hardware_state: HardwareState {
                led_status: false,
                sleep_status: SleepStatus::Awake,
                memory_usage: MemoryInfo {
                    total_bytes: 1024,
                    free_bytes: 512,
                    used_bytes: 512,
                    largest_free_block: 256,
                },
            },
            system_info: SystemInfo {
                firmware_version: "1.0.0".to_string(),
                platform: "test".to_string(),
                uptime_seconds: 100,
                steel_runtime_version: "0.7.0".to_string(),
            },
            timestamp: Utc::now(),
        };

        client.update_shadow(&device_state).await.unwrap();
        let shadow_updates = client.get_shadow_updates().await;
        assert_eq!(shadow_updates.len(), 1);

        // Test disconnection
        client.disconnect().await.unwrap();
        assert_eq!(client.get_connection_status(), ConnectionStatus::Disconnected);
    }

    #[test]
    fn test_topic_validation() {
        let config = IoTConfig::default();
        let client = IoTClient::new(config);

        // Valid topics
        assert!(client.validate_topic("test/topic").is_ok());
        assert!(client.validate_topic("$aws/thing/test/shadow/update").is_ok());
        assert!(client.validate_topic("device/123/data").is_ok());

        // Invalid topics
        assert!(client.validate_topic("").is_err());
        assert!(client.validate_topic("topic\0with\0nulls").is_err());
        assert!(client.validate_topic("topic\nwith\nnewlines").is_err());
        assert!(client.validate_topic("$invalid/reserved").is_err());
        
        // Too long topic
        let long_topic = "a".repeat(257);
        assert!(client.validate_topic(&long_topic).is_err());
    }

    #[test]
    fn test_iot_config_default() {
        let config = IoTConfig::default();
        assert_eq!(config.device_id, "default-device");
        assert_eq!(config.thing_name, "default-thing");
        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.keep_alive_secs, 60);
        assert!(config.clean_session);
    }
}

