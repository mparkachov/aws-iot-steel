use thiserror::Error;

/// Main system error type that encompasses all possible errors
#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),

    #[error("Security error: {0}")]
    Security(#[from] SecurityError),

    #[error("IoT error: {0}")]
    IoT(#[from] IoTError),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Platform-specific errors for HAL operations
#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("Hardware operation failed: {0}")]
    Hardware(String),

    #[error("Sleep operation failed: {0}")]
    Sleep(String),

    #[error("LED operation failed: {0}")]
    Led(String),

    #[error("Storage operation failed: {0}")]
    Storage(String),

    #[error("Device info unavailable: {0}")]
    DeviceInfo(String),

    #[error("Platform not supported: {0}")]
    Unsupported(String),
}

/// Security-related errors
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Certificate error: {0}")]
    Certificate(String),

    #[error("Key management error: {0}")]
    KeyManagement(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),
}

/// Result type alias for system operations
pub type SystemResult<T> = Result<T, SystemError>;

/// Result type alias for platform operations
pub type PlatformResult<T> = Result<T, PlatformError>;

/// IoT-related errors
#[derive(Debug, Error)]
pub enum IoTError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("MQTT error: {0}")]
    Mqtt(String),

    #[error("Shadow error: {0}")]
    Shadow(String),

    #[error("Topic validation error: {0}")]
    TopicValidation(String),

    #[error("Message parsing error: {0}")]
    MessageParsing(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Publish error: {0}")]
    Publish(String),

    #[error("Not connected")]
    NotConnected,
}

/// Result type alias for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Result type alias for IoT operations
pub type IoTResult<T> = Result<T, IoTError>;

/// API-related errors
#[derive(Debug, Error)]
pub enum APIError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Operation not supported: {0}")]
    NotSupported(String),

    #[error("Operation not implemented: {0}")]
    NotImplemented(String),

    #[error("Hardware error: {0}")]
    Hardware(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Result type alias for API operations
pub type APIResult<T> = Result<T, APIError>;
