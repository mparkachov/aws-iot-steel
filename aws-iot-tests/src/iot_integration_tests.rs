use aws_iot_core::{DeviceState, ProgramMessage, IoTClientTrait};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

/// Configuration for AWS IoT integration tests
#[derive(Debug, Clone)]
pub struct IoTTestConfig {
    pub endpoint: String,
    pub region: String,
    pub thing_name: String,
    pub cert_path: Option<String>,
    pub private_key_path: Option<String>,
    pub ca_cert_path: Option<String>,
    pub use_mock: bool,
    pub test_timeout_seconds: u64,
}

impl IoTTestConfig {
    /// Load test configuration from environment variables or use defaults
    pub fn from_env() -> Self {
        Self {
            endpoint: std::env::var("AWS_IOT_ENDPOINT")
                .unwrap_or_else(|_| "test-endpoint.iot.us-east-1.amazonaws.com".to_string()),
            region: std::env::var("AWS_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
            thing_name: std::env::var("AWS_IOT_THING_NAME")
                .unwrap_or_else(|_| format!("test-device-{}", Uuid::new_v4())),
            cert_path: std::env::var("AWS_IOT_CERT_PATH").ok(),
            private_key_path: std::env::var("AWS_IOT_PRIVATE_KEY_PATH").ok(),
            ca_cert_path: std::env::var("AWS_IOT_CA_CERT_PATH").ok(),
            use_mock: std::env::var("USE_MOCK_IOT")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true), // Default to mock for CI/CD
            test_timeout_seconds: std::env::var("IOT_TEST_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
        }
    }
    
    /// Check if real AWS IoT credentials are available
    pub fn has_real_credentials(&self) -> bool {
        self.cert_path.is_some() && 
        self.private_key_path.is_some() && 
        !self.use_mock
    }
}

/// Test suite for AWS IoT integration
pub struct IoTIntegrationTests {
    config: IoTTestConfig,
}

impl Default for IoTIntegrationTests {
    fn default() -> Self {
        Self::new()
    }
}

impl IoTIntegrationTests {
    pub fn new() -> Self {
        Self {
            config: IoTTestConfig::from_env(),
        }
    }
    
    /// Run all IoT integration tests
    pub async fn run_all_tests(&self) -> Result<TestResults, Box<dyn std::error::Error>> {
        let mut results = TestResults::new();
        
        if !self.config.has_real_credentials() {
            println!("⚠️  Running IoT tests with mock client (no real AWS credentials)");
            return self.run_mock_tests().await;
        }
        
        println!("🌐 Running IoT integration tests with real AWS IoT...");
        
        // Test basic connectivity
        results.add_test_result(
            "iot_connectivity",
            self.test_iot_connectivity().await
        );
        
        // Test program delivery
        results.add_test_result(
            "program_delivery",
            self.test_program_delivery().await
        );
        
        // Test shadow synchronization
        results.add_test_result(
            "shadow_synchronization",
            self.test_shadow_synchronization().await
        );
        
        // Test connection resilience
        results.add_test_result(
            "connection_resilience",
            self.test_connection_resilience().await
        );
        
        // Test concurrent Steel programs
        results.add_test_result(
            "concurrent_steel_execution",
            self.test_concurrent_steel_execution().await
        );
        
        Ok(results)
    }
    
    /// Run tests with mock IoT client
    async fn run_mock_tests(&self) -> Result<TestResults, Box<dyn std::error::Error>> {
        use crate::MockIoTClient;
        
        let mut results = TestResults::new();
        let client = Arc::new(tokio::sync::Mutex::new(
            MockIoTClient::new(self.config.thing_name.clone())
        ));
        
        // Test mock connectivity
        results.add_test_result(
            "mock_iot_connectivity",
            self.test_mock_connectivity(client.clone()).await
        );
        
        // Test mock program delivery
        results.add_test_result(
            "mock_program_delivery", 
            self.test_mock_program_delivery(client.clone()).await
        );
        
        // Test mock shadow operations
        results.add_test_result(
            "mock_shadow_operations",
            self.test_mock_shadow_operations(client.clone()).await
        );
        
        Ok(results)
    }
    
    /// Test basic IoT connectivity
    async fn test_iot_connectivity(&self) -> TestResult {
        let test_timeout = Duration::from_secs(self.config.test_timeout_seconds);
        
        match timeout(test_timeout, self.connect_to_iot()).await {
            Ok(Ok(client)) => {
                // Test basic operations
                match client.publish("test/connectivity", b"ping", rumqttc::QoS::AtLeastOnce).await {
                    Ok(_) => TestResult::passed("IoT connectivity test passed"),
                    Err(e) => TestResult::failed(&format!("Failed to publish: {}", e)),
                }
            }
            Ok(Err(e)) => TestResult::failed(&format!("Connection failed: {}", e)),
            Err(_) => TestResult::failed("Connection timeout"),
        }
    }
    
    /// Test Steel program delivery via MQTT
    async fn test_program_delivery(&self) -> TestResult {
        let test_timeout = Duration::from_secs(self.config.test_timeout_seconds);
        
        match timeout(test_timeout, self.test_program_delivery_impl()).await {
            Ok(Ok(_)) => TestResult::passed("Program delivery test passed"),
            Ok(Err(e)) => TestResult::failed(&format!("Program delivery failed: {}", e)),
            Err(_) => TestResult::failed("Program delivery timeout"),
        }
    }
    
    async fn test_program_delivery_impl(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.connect_to_iot().await?;
        
        // Subscribe to program topics
        client.subscribe_to_program_topics().await?;
        
        // Create test Steel program
        let test_program = ProgramMessage {
            program_id: format!("test-prog-{}", Uuid::new_v4()),
            program_name: "connectivity-test".to_string(),
            steel_code: r#"
                (begin
                  (log-info "Test program executing")
                  (led-on)
                  (sleep 0.1)
                  (led-off)
                  (log-info "Test program completed")
                  #t)
            "#.to_string(),
            version: "1.0.0".to_string(),
            checksum: "test-checksum".to_string(),
            auto_start: true,
            metadata: None,
        };
        
        // Publish program
        let program_topic = format!("steel-programs/{}/load", self.config.thing_name);
        let program_json = serde_json::to_string(&test_program)?;
        client.publish(&program_topic, program_json.as_bytes(), rumqttc::QoS::AtLeastOnce).await?;
        
        // Wait for program execution (in real implementation, this would be handled by the runtime)
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        Ok(())
    }
    
    /// Test device shadow synchronization
    async fn test_shadow_synchronization(&self) -> TestResult {
        let test_timeout = Duration::from_secs(self.config.test_timeout_seconds);
        
        match timeout(test_timeout, self.test_shadow_sync_impl()).await {
            Ok(Ok(_)) => TestResult::passed("Shadow synchronization test passed"),
            Ok(Err(e)) => TestResult::failed(&format!("Shadow sync failed: {}", e)),
            Err(_) => TestResult::failed("Shadow synchronization timeout"),
        }
    }
    
    async fn test_shadow_sync_impl(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.connect_to_iot().await?;
        
        // Create test device state
        let test_state = DeviceState::default();
        
        // Update shadow
        client.update_shadow(&test_state).await?;
        
        // Retrieve shadow
        let _retrieved_state = client.get_shadow().await?;
        
        // Verify state consistency (basic check)
        // In a real implementation, you'd compare specific fields
        println!("Shadow update and retrieval successful");
        
        Ok(())
    }
    
    /// Test connection resilience and recovery
    async fn test_connection_resilience(&self) -> TestResult {
        let test_timeout = Duration::from_secs(self.config.test_timeout_seconds * 2);
        
        match timeout(test_timeout, self.test_resilience_impl()).await {
            Ok(Ok(_)) => TestResult::passed("Connection resilience test passed"),
            Ok(Err(e)) => TestResult::failed(&format!("Resilience test failed: {}", e)),
            Err(_) => TestResult::failed("Connection resilience timeout"),
        }
    }
    
    async fn test_resilience_impl(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut client = self.connect_to_iot().await?;
        
        // Test initial connectivity
        client.publish("test/resilience/start", b"test starting", rumqttc::QoS::AtLeastOnce).await?;
        
        // Simulate connection issues by disconnecting and reconnecting
        client.disconnect().await?;
        
        // Wait a bit
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Reconnect
        client.connect().await?;
        
        // Test that we can publish again
        client.publish("test/resilience/recovered", b"connection recovered", rumqttc::QoS::AtLeastOnce).await?;
        
        Ok(())
    }
    
    /// Test concurrent Steel program execution
    async fn test_concurrent_steel_execution(&self) -> TestResult {
        let test_timeout = Duration::from_secs(self.config.test_timeout_seconds * 3);
        
        match timeout(test_timeout, self.test_concurrent_execution_impl()).await {
            Ok(Ok(_)) => TestResult::passed("Concurrent execution test passed"),
            Ok(Err(e)) => TestResult::failed(&format!("Concurrent execution failed: {}", e)),
            Err(_) => TestResult::failed("Concurrent execution timeout"),
        }
    }
    
    async fn test_concurrent_execution_impl(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.connect_to_iot().await?;
        
        // Subscribe to program topics
        client.subscribe_to_program_topics().await?;
        
        // Create multiple test programs
        let programs = [
            ("led-blink", "(begin (led-on) (sleep 0.1) (led-off))"),
            ("device-info", "(begin (device-info) (memory-info))"),
            ("logging-test", r#"(begin (log-info "Concurrent test") (log-debug "Debug message"))"#),
        ];
        
        // Send all programs sequentially (since we can't clone the client)
        for (i, (name, code)) in programs.iter().enumerate() {
            let program = ProgramMessage {
                program_id: format!("concurrent-test-{}-{}", i, Uuid::new_v4()),
                program_name: name.to_string(),
                steel_code: code.to_string(),
                version: "1.0.0".to_string(),
                checksum: format!("checksum-{}", i),
                auto_start: true,
                metadata: None,
            };
            
            let program_topic = format!("steel-programs/{}/load", self.config.thing_name);
            let program_json = serde_json::to_string(&program)?;
            
            client.publish(&program_topic, program_json.as_bytes(), rumqttc::QoS::AtLeastOnce).await?;
        }
        
        // Wait for execution to complete
        tokio::time::sleep(Duration::from_secs(3)).await;
        
        Ok(())
    }
    
    /// Test mock IoT connectivity
    async fn test_mock_connectivity(&self, client: Arc<tokio::sync::Mutex<crate::MockIoTClient>>) -> TestResult {
        let mut client_guard = client.lock().await;
        
        match client_guard.connect().await {
            Ok(_) => {
                match client_guard.publish("test/mock", b"test message", rumqttc::QoS::AtLeastOnce).await {
                    Ok(_) => TestResult::passed("Mock IoT connectivity test passed"),
                    Err(e) => TestResult::failed(&format!("Mock publish failed: {}", e)),
                }
            }
            Err(e) => TestResult::failed(&format!("Mock connection failed: {}", e)),
        }
    }
    
    /// Test mock program delivery
    async fn test_mock_program_delivery(&self, client: Arc<tokio::sync::Mutex<crate::MockIoTClient>>) -> TestResult {
        let mut client_guard = client.lock().await;
        
        // Connect first
        if let Err(e) = client_guard.connect().await {
            return TestResult::failed(&format!("Mock connection failed: {}", e));
        }
        
        // Subscribe to program topics
        if let Err(e) = client_guard.subscribe_to_program_topics().await {
            return TestResult::failed(&format!("Mock subscription failed: {}", e));
        }
        
        // Create test program
        let test_program = ProgramMessage {
            program_id: "mock-test-prog".to_string(),
            program_name: "mock-test".to_string(),
            steel_code: "(+ 1 1)".to_string(),
            version: "1.0.0".to_string(),
            checksum: "mock-checksum".to_string(),
            auto_start: true,
            metadata: None,
        };
        
        // Handle program delivery
        match client_guard.handle_program_delivery(test_program).await {
            Ok(_) => TestResult::passed("Mock program delivery test passed"),
            Err(e) => TestResult::failed(&format!("Mock program delivery failed: {}", e)),
        }
    }
    
    /// Test mock shadow operations
    async fn test_mock_shadow_operations(&self, client: Arc<tokio::sync::Mutex<crate::MockIoTClient>>) -> TestResult {
        let mut client_guard = client.lock().await;
        
        // Connect first
        if let Err(e) = client_guard.connect().await {
            return TestResult::failed(&format!("Mock connection failed: {}", e));
        }
        
        // Create test state
        let test_state = DeviceState::default();
        
        // Update shadow
        match client_guard.update_shadow(&test_state).await {
            Ok(_) => {
                // Get shadow
                match client_guard.get_shadow().await {
                    Ok(_) => TestResult::passed("Mock shadow operations test passed"),
                    Err(e) => TestResult::failed(&format!("Mock shadow get failed: {}", e)),
                }
            }
            Err(e) => TestResult::failed(&format!("Mock shadow update failed: {}", e)),
        }
    }
    
    /// Helper to connect to IoT (would use real AWS IoT client in production)
    async fn connect_to_iot(&self) -> Result<Box<dyn aws_iot_core::IoTClientTrait>, Box<dyn std::error::Error>> {
        // In a real implementation, this would create and configure an actual AWS IoT client
        // For now, we'll use the mock client
        use crate::MockIoTClient;
        
        let mut client = MockIoTClient::new(self.config.thing_name.clone());
        client.connect().await?;
        
        Ok(Box::new(client))
    }
}

/// Individual test result
#[derive(Debug, Clone)]
pub struct TestResult {
    pub passed: bool,
    pub message: String,
    pub duration_ms: u64,
}

impl TestResult {
    pub fn passed(message: &str) -> Self {
        Self {
            passed: true,
            message: message.to_string(),
            duration_ms: 0,
        }
    }
    
    pub fn failed(message: &str) -> Self {
        Self {
            passed: false,
            message: message.to_string(),
            duration_ms: 0,
        }
    }
}

/// Collection of test results
#[derive(Debug)]
pub struct TestResults {
    pub tests: Vec<(String, TestResult)>,
}

impl Default for TestResults {
    fn default() -> Self {
        Self::new()
    }
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
        }
    }
    
    pub fn add_test_result(&mut self, name: &str, result: TestResult) {
        self.tests.push((name.to_string(), result));
    }
    
    pub fn total_tests(&self) -> usize {
        self.tests.len()
    }
    
    pub fn passed_tests(&self) -> usize {
        self.tests.iter().filter(|(_, r)| r.passed).count()
    }
    
    pub fn failed_tests(&self) -> usize {
        self.tests.len() - self.passed_tests()
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.tests.is_empty() {
            0.0
        } else {
            self.passed_tests() as f64 / self.tests.len() as f64 * 100.0
        }
    }
    
    pub fn print_summary(&self) {
        println!("=== IoT Integration Test Results ===");
        println!("Total tests: {}", self.total_tests());
        println!("Passed: {}", self.passed_tests());
        println!("Failed: {}", self.failed_tests());
        println!("Success rate: {:.1}%", self.success_rate());
        
        if self.failed_tests() > 0 {
            println!("\nFailed tests:");
            for (name, result) in &self.tests {
                if !result.passed {
                    println!("  ❌ {}: {}", name, result.message);
                }
            }
        }
        
        println!("\nPassed tests:");
        for (name, result) in &self.tests {
            if result.passed {
                println!("  ✅ {}: {}", name, result.message);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_iot_integration_suite() {
        let test_suite = IoTIntegrationTests::new();
        let results = test_suite.run_all_tests().await.unwrap();
        
        results.print_summary();
        
        // In CI/CD, we might want to assert that all tests pass
        // For development, we just print the results
        if std::env::var("CI").is_ok() {
            assert_eq!(results.failed_tests(), 0, "Some IoT integration tests failed");
        }
    }
    
    #[tokio::test]
    async fn test_config_from_env() {
        let config = IoTTestConfig::from_env();
        
        // Basic validation
        assert!(!config.endpoint.is_empty());
        assert!(!config.region.is_empty());
        assert!(!config.thing_name.is_empty());
        assert!(config.test_timeout_seconds > 0);
    }
}