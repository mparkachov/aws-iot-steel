use aws_iot_core::types::SleepStatus;
use aws_iot_core::{
    iot_client::{IoTClientTrait, MockIoTClient},
    shadow_manager::{
        DesiredState, DeviceConfiguration, MockShadowManager, ProgramCommands, ShadowManager,
        ShadowManagerTrait, ShadowState,
    },
    ConnectionStatus, MemoryInfo, RuntimeStatus,
};
use chrono::Utc;
use std::sync::Arc;

#[tokio::test]
async fn test_mock_shadow_manager_initialization() {
    let mut manager = MockShadowManager::new("test-device-001".to_string());

    // Test initialization
    let result = manager.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mock_shadow_manager_reported_state_updates() {
    let manager = MockShadowManager::new("test-device-002".to_string());

    // Create test device state
    let device_state = ShadowManager::create_device_state(
        RuntimeStatus::Idle,
        false,
        SleepStatus::Awake,
        MemoryInfo {
            total_bytes: 4096,
            free_bytes: 2048,
            used_bytes: 2048,
            largest_free_block: 1024,
        },
        "1.0.0".to_string(),
        "test-platform".to_string(),
        1800,
    )
    .await;

    // Update reported state
    manager.update_reported_state(&device_state).await.unwrap();

    // Verify the update was recorded
    let updates = manager.get_shadow_updates().await;
    assert_eq!(updates.len(), 1);

    let updated_state = &updates[0];
    assert!(matches!(updated_state.runtime_status, RuntimeStatus::Idle));
    assert!(!updated_state.hardware_state.led_status);
    assert_eq!(updated_state.system_info.platform, "test-platform");
    assert_eq!(updated_state.system_info.uptime_seconds, 1800);
}

#[tokio::test]
async fn test_mock_shadow_manager_multiple_updates() {
    let manager = MockShadowManager::new("test-device-003".to_string());

    // Send multiple updates with different states
    for i in 0..5 {
        let device_state = ShadowManager::create_device_state(
            if i % 2 == 0 {
                RuntimeStatus::Idle
            } else {
                RuntimeStatus::ExecutingProgram {
                    program_id: format!("program-{}", i),
                    started_at: Utc::now(),
                }
            },
            i % 2 == 1, // Alternate LED state
            SleepStatus::Awake,
            MemoryInfo {
                total_bytes: 1024,
                free_bytes: 512 + i * 10,
                used_bytes: 512 - i * 10,
                largest_free_block: 256,
            },
            "1.0.0".to_string(),
            "test".to_string(),
            100 + i,
        )
        .await;

        manager.update_reported_state(&device_state).await.unwrap();
    }

    let updates = manager.get_shadow_updates().await;
    assert_eq!(updates.len(), 5);

    // Verify alternating LED states
    for (i, update) in updates.iter().enumerate() {
        assert_eq!(update.hardware_state.led_status, i % 2 == 1);
        assert_eq!(update.system_info.uptime_seconds, 100 + i as u64);
    }
}

#[tokio::test]
async fn test_desired_state_processing() {
    let manager = MockShadowManager::new("test-device-004".to_string());

    // Test LED control desired state
    let led_desired = DesiredState {
        led_control: Some(true),
        sleep_duration: None,
        configuration: None,
        program_commands: None,
    };

    let result = manager.process_desired_state(&led_desired).await.unwrap();
    assert!(result.success);
    assert!(result.message.is_some());

    // Test sleep duration desired state
    let sleep_desired = DesiredState {
        led_control: None,
        sleep_duration: Some(10.0),
        configuration: None,
        program_commands: None,
    };

    let result = manager.process_desired_state(&sleep_desired).await.unwrap();
    assert!(result.success);

    // Test configuration desired state
    let config_desired = DesiredState {
        led_control: None,
        sleep_duration: None,
        configuration: Some(DeviceConfiguration {
            log_level: Some("debug".to_string()),
            reporting_interval: Some(30),
            auto_update: Some(true),
        }),
        program_commands: None,
    };

    let result = manager
        .process_desired_state(&config_desired)
        .await
        .unwrap();
    assert!(result.success);

    // Verify all desired states were processed
    let processed = manager.get_processed_desired_states().await;
    assert_eq!(processed.len(), 3);
    assert_eq!(processed[0].led_control, Some(true));
    assert_eq!(processed[1].sleep_duration, Some(10.0));
    assert!(processed[2].configuration.is_some());
}

#[tokio::test]
async fn test_program_commands_processing() {
    let manager = MockShadowManager::new("test-device-005".to_string());

    let program_desired = DesiredState {
        led_control: None,
        sleep_duration: None,
        configuration: None,
        program_commands: Some(ProgramCommands {
            load_program: Some("sensor-monitor-v2".to_string()),
            stop_program: Some("old-program".to_string()),
            restart_program: Some("system-monitor".to_string()),
        }),
    };

    let result = manager
        .process_desired_state(&program_desired)
        .await
        .unwrap();
    assert!(result.success);

    let processed = manager.get_processed_desired_states().await;
    assert_eq!(processed.len(), 1);

    let commands = processed[0].program_commands.as_ref().unwrap();
    assert_eq!(commands.load_program, Some("sensor-monitor-v2".to_string()));
    assert_eq!(commands.stop_program, Some("old-program".to_string()));
    assert_eq!(commands.restart_program, Some("system-monitor".to_string()));
}

#[tokio::test]
async fn test_complex_desired_state() {
    let manager = MockShadowManager::new("test-device-006".to_string());

    // Test complex desired state with all fields
    let complex_desired = DesiredState {
        led_control: Some(false),
        sleep_duration: Some(5.5),
        configuration: Some(DeviceConfiguration {
            log_level: Some("info".to_string()),
            reporting_interval: Some(60),
            auto_update: Some(false),
        }),
        program_commands: Some(ProgramCommands {
            load_program: Some("multi-sensor-system".to_string()),
            stop_program: None,
            restart_program: Some("watchdog".to_string()),
        }),
    };

    let result = manager
        .process_desired_state(&complex_desired)
        .await
        .unwrap();
    assert!(result.success);
    assert!(result.message.is_some());

    let processed = manager.get_processed_desired_states().await;
    assert_eq!(processed.len(), 1);

    let state = &processed[0];
    assert_eq!(state.led_control, Some(false));
    assert_eq!(state.sleep_duration, Some(5.5));

    let config = state.configuration.as_ref().unwrap();
    assert_eq!(config.log_level, Some("info".to_string()));
    assert_eq!(config.reporting_interval, Some(60));
    assert_eq!(config.auto_update, Some(false));

    let commands = state.program_commands.as_ref().unwrap();
    assert_eq!(
        commands.load_program,
        Some("multi-sensor-system".to_string())
    );
    assert_eq!(commands.restart_program, Some("watchdog".to_string()));
    assert!(commands.stop_program.is_none());
}

#[tokio::test]
async fn test_shadow_get_and_set() {
    let manager = MockShadowManager::new("test-device-007".to_string());

    // Initially should return empty shadow
    let shadow = manager.get_shadow().await.unwrap();
    assert!(shadow.state.desired.is_none());
    assert!(shadow.state.reported.is_none());

    // Set a mock shadow
    let mock_shadow = aws_iot_core::shadow_manager::ShadowUpdate {
        state: ShadowState {
            desired: Some(DesiredState {
                led_control: Some(true),
                sleep_duration: Some(3.0),
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

    manager.set_mock_shadow(mock_shadow.clone()).await;

    // Verify we can retrieve the shadow
    let retrieved_shadow = manager.get_shadow().await.unwrap();
    assert_eq!(retrieved_shadow.version, Some(42));
    assert!(retrieved_shadow.state.desired.is_some());

    let desired = retrieved_shadow.state.desired.unwrap();
    assert_eq!(desired.led_control, Some(true));
    assert_eq!(desired.sleep_duration, Some(3.0));
}

#[tokio::test]
async fn test_shadow_manager_with_real_iot_client() {
    let mut iot_client = MockIoTClient::new();
    iot_client.connect().await.unwrap();
    assert_eq!(
        iot_client.get_connection_status(),
        ConnectionStatus::Connected
    );

    let mut manager = ShadowManager::new(
        "integration-test-device".to_string(),
        "integration-test-thing".to_string(),
        Arc::new(iot_client),
    );

    // Test initialization
    manager.initialize().await.unwrap();

    // Test updating reported state
    let device_state = ShadowManager::create_device_state(
        RuntimeStatus::ExecutingProgram {
            program_id: "integration-test-program".to_string(),
            started_at: Utc::now(),
        },
        true,
        SleepStatus::Sleeping {
            wake_time: Utc::now() + chrono::Duration::seconds(300),
        },
        MemoryInfo {
            total_bytes: 8192,
            free_bytes: 4096,
            used_bytes: 4096,
            largest_free_block: 2048,
        },
        "2.0.0".to_string(),
        "esp32-c3-devkit-rust-1".to_string(),
        7200,
    )
    .await;

    manager.update_reported_state(&device_state).await.unwrap();

    // Test processing desired state
    let desired_state = DesiredState {
        led_control: Some(false),
        sleep_duration: Some(1.0),
        configuration: Some(DeviceConfiguration {
            log_level: Some("trace".to_string()),
            reporting_interval: Some(15),
            auto_update: Some(true),
        }),
        program_commands: Some(ProgramCommands {
            load_program: Some("integration-test-suite".to_string()),
            stop_program: None,
            restart_program: None,
        }),
    };

    let result = manager.process_desired_state(&desired_state).await.unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn test_device_state_creation() {
    // Test creating device state with different runtime statuses
    let idle_state = ShadowManager::create_device_state(
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
        0,
    )
    .await;

    assert!(matches!(idle_state.runtime_status, RuntimeStatus::Idle));
    assert!(!idle_state.hardware_state.led_status);
    assert!(matches!(
        idle_state.hardware_state.sleep_status,
        SleepStatus::Awake
    ));

    // Test with executing program status
    let executing_state = ShadowManager::create_device_state(
        RuntimeStatus::ExecutingProgram {
            program_id: "test-program".to_string(),
            started_at: Utc::now(),
        },
        true,
        SleepStatus::Sleeping {
            wake_time: Utc::now() + chrono::Duration::seconds(60),
        },
        MemoryInfo {
            total_bytes: 2048,
            free_bytes: 1024,
            used_bytes: 1024,
            largest_free_block: 512,
        },
        "1.1.0".to_string(),
        "esp32-c3-devkit-rust-1".to_string(),
        3600,
    )
    .await;

    match executing_state.runtime_status {
        RuntimeStatus::ExecutingProgram { program_id, .. } => {
            assert_eq!(program_id, "test-program");
        }
        _ => panic!("Expected ExecutingProgram status"),
    }

    assert!(executing_state.hardware_state.led_status);
    assert!(matches!(
        executing_state.hardware_state.sleep_status,
        SleepStatus::Sleeping { .. }
    ));
    assert_eq!(executing_state.system_info.firmware_version, "1.1.0");
    assert_eq!(
        executing_state.system_info.platform,
        "esp32-c3-devkit-rust-1"
    );
    assert_eq!(executing_state.system_info.uptime_seconds, 3600);
}

#[tokio::test]
async fn test_error_state_handling() {
    let manager = MockShadowManager::new("test-device-008".to_string());

    // Test creating device state with error status
    let error_state = ShadowManager::create_device_state(
        RuntimeStatus::Error {
            message: "Test error condition".to_string(),
            timestamp: Utc::now(),
        },
        false,
        SleepStatus::Awake,
        MemoryInfo {
            total_bytes: 512,
            free_bytes: 100,
            used_bytes: 412,
            largest_free_block: 50,
        },
        "1.0.0".to_string(),
        "test".to_string(),
        500,
    )
    .await;

    match &error_state.runtime_status {
        RuntimeStatus::Error { message, .. } => {
            assert_eq!(message, "Test error condition");
        }
        _ => panic!("Expected Error status"),
    }

    // Update reported state with error
    manager.update_reported_state(&error_state).await.unwrap();

    let updates = manager.get_shadow_updates().await;
    assert_eq!(updates.len(), 1);

    match &updates[0].runtime_status {
        RuntimeStatus::Error { message, .. } => {
            assert_eq!(message, "Test error condition");
        }
        _ => panic!("Expected Error status in update"),
    }
}
