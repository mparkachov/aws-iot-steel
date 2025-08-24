#[cfg(test)]
mod tests {
    use aws_iot_core::{LogLevel, LoggingConfig, LogFormat, LogTarget};
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_test_logging() {
        INIT.call_once(|| {
            let config = LoggingConfig {
                level: LogLevel::Debug,
                target: LogTarget::Stdout,
                format: LogFormat::Compact,
                include_timestamps: false,
                include_thread_ids: false,
                include_file_locations: false,
            };
            let _ = aws_iot_core::initialize_logging(config);
        });
    }

    #[tokio::test]
    async fn test_logging_initialization() {
        // Test that logging can be initialized without errors
        // Note: We can't test this multiple times in the same process
        // due to tracing's global subscriber limitation
        init_test_logging();
        
        // Test logging macros work
        aws_iot_core::log_info!("Test info message");
        aws_iot_core::log_debug!("Test debug message");
        aws_iot_core::log_warn!("Test warning message");
    }

    #[test]
    fn test_error_types() {
        use aws_iot_core::{SystemError, PlatformError, SecurityError};
        
        // Test error creation and conversion
        let platform_error = PlatformError::Hardware("Test hardware error".to_string());
        let system_error: SystemError = platform_error.into();
        
        match system_error {
            SystemError::Platform(_) => {}, // Expected
            _ => panic!("Error conversion failed"),
        }
        
        // Test error display
        let security_error = SecurityError::Certificate("Invalid certificate".to_string());
        let error_string = format!("{}", security_error);
        assert!(error_string.contains("Certificate error"));
    }

    #[test]
    fn test_led_state_conversions() {
        use aws_iot_core::LedState;
        
        // Test bool to LedState conversion
        assert_eq!(LedState::from(true), LedState::On);
        assert_eq!(LedState::from(false), LedState::Off);
        
        // Test LedState to bool conversion
        assert!(bool::from(LedState::On));
        assert!(!bool::from(LedState::Off));
    }

    #[test]
    fn test_memory_info_calculations() {
        use aws_iot_core::MemoryInfo;
        
        let memory_info = MemoryInfo {
            total_bytes: 1000,
            free_bytes: 300,
            used_bytes: 700,
            largest_free_block: 200,
        };
        
        assert_eq!(memory_info.usage_percentage(), 70.0);
        
        // Test edge case with zero total
        let zero_memory = MemoryInfo {
            total_bytes: 0,
            free_bytes: 0,
            used_bytes: 0,
            largest_free_block: 0,
        };
        
        assert_eq!(zero_memory.usage_percentage(), 0.0);
    }

    #[test]
    fn test_log_level_conversions() {
        use aws_iot_core::LogLevel;
        
        // Test Display trait
        assert_eq!(format!("{}", LogLevel::Error), "ERROR");
        assert_eq!(format!("{}", LogLevel::Warn), "WARN");
        assert_eq!(format!("{}", LogLevel::Info), "INFO");
        assert_eq!(format!("{}", LogLevel::Debug), "DEBUG");
        assert_eq!(format!("{}", LogLevel::Trace), "TRACE");
        
        // Test conversion to tracing::Level
        let tracing_level: tracing::Level = LogLevel::Info.into();
        assert_eq!(tracing_level, tracing::Level::INFO);
    }
}