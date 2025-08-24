use aws_iot_core::types::ProgramMetadata;
use aws_iot_core::{
    iot_client::{IoTClientTrait, MockIoTClient},
    program_delivery::{
        MockProgramDeliveryManager, ProgramDeliveryManager, ProgramDeliveryManagerTrait,
        ProgramExecutionStatus,
    },
    types::ProgramMessage,
    ConnectionStatus,
};
use chrono::Utc;
use std::sync::Arc;

#[tokio::test]
async fn test_mock_program_delivery_manager_initialization() {
    let mut manager = MockProgramDeliveryManager::new("test-device-001".to_string());

    // Test initialization
    let result = manager.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_program_loading_and_validation() {
    let manager = MockProgramDeliveryManager::new("test-device-002".to_string());

    // Create valid program message
    let program_message = ProgramMessage {
        program_id: "test-program-001".to_string(),
        program_name: "LED Blink Program".to_string(),
        steel_code: r#"
            (begin
              (log "info" "Starting LED blink program")
              (led-on)
              (sleep 1)
              (led-off)
              (sleep 1)
              (log "info" "LED blink completed"))
        "#
        .to_string(),
        version: "1.0.0".to_string(),
        checksum: "valid-checksum-123".to_string(),
        auto_start: false,
        metadata: Some(ProgramMetadata {
            description: Some("Simple LED blink demonstration".to_string()),
            author: Some("Test Developer".to_string()),
            created_at: Utc::now(),
            memory_requirement: Some(2048),
            execution_timeout: Some(60),
        }),
    };

    // Test program validation
    let validation = manager.validate_program(&program_message).await.unwrap();
    assert!(validation.valid);
    assert!(validation.checksum_match);
    assert!(validation.size_valid);
    assert!(validation.syntax_valid);
    assert!(validation.error_message.is_none());

    // Test program loading
    let program_id = manager.load_program(&program_message).await.unwrap();
    assert_eq!(program_id, "test-program-001");

    // Verify program was loaded
    let loaded_programs = manager.get_loaded_programs().await;
    assert_eq!(loaded_programs.len(), 1);
    assert_eq!(loaded_programs[0].program_id, "test-program-001");
    assert_eq!(loaded_programs[0].program_name, "LED Blink Program");

    // Test program status
    let status = manager.get_program_status(&program_id).await.unwrap();
    assert_eq!(status.program_id, "test-program-001");
    assert_eq!(status.status, ProgramExecutionStatus::Pending);
    assert!(status.message.is_some());
}

#[tokio::test]
async fn test_program_execution() {
    let manager = MockProgramDeliveryManager::new("test-device-003".to_string());

    let program_message = ProgramMessage {
        program_id: "execution-test-program".to_string(),
        program_name: "Execution Test".to_string(),
        steel_code: r#"
            (begin
              (log "info" "Program execution test")
              (define result (+ 10 20 30))
              (log "info" (format "Calculation result: ~a" result))
              result)
        "#
        .to_string(),
        version: "1.0.0".to_string(),
        checksum: "execution-checksum".to_string(),
        auto_start: false,
        metadata: None,
    };

    // Load program
    let program_id = manager.load_program(&program_message).await.unwrap();

    // Execute program
    let result = manager.execute_program(&program_id).await.unwrap();
    assert!(result.success);
    assert_eq!(result.program_id, "execution-test-program");
    assert!(result.result.is_some());
    assert!(result.error.is_none());
    assert!(result.execution_time_ms > 0);

    // Check final status
    let status = manager.get_program_status(&program_id).await.unwrap();
    assert_eq!(status.status, ProgramExecutionStatus::Completed);
    assert!(status.execution_time_ms.is_some());
}

#[tokio::test]
async fn test_auto_start_program() {
    let manager = MockProgramDeliveryManager::new("test-device-004".to_string());

    let auto_start_program = ProgramMessage {
        program_id: "auto-start-test".to_string(),
        program_name: "Auto Start Test".to_string(),
        steel_code: r#"
            (begin
              (log "info" "Auto-started program running")
              (led-on)
              (sleep 0.5)
              (led-off)
              "auto-start-completed")
        "#
        .to_string(),
        version: "1.0.0".to_string(),
        checksum: "auto-start-checksum".to_string(),
        auto_start: true,
        metadata: Some(ProgramMetadata {
            description: Some("Test auto-start functionality".to_string()),
            author: Some("Auto Test".to_string()),
            created_at: Utc::now(),
            memory_requirement: Some(1024),
            execution_timeout: Some(30),
        }),
    };

    // Handle program message with auto-start
    let status = manager
        .handle_program_message(&auto_start_program)
        .await
        .unwrap();
    assert_eq!(status.program_id, "auto-start-test");

    // Verify execution occurred automatically
    let execution_results = manager.get_execution_results().await;
    assert_eq!(execution_results.len(), 1);
    assert!(execution_results[0].success);
    assert_eq!(execution_results[0].program_id, "auto-start-test");

    // Verify program is in completed state
    let final_status = manager.get_program_status("auto-start-test").await.unwrap();
    assert_eq!(final_status.status, ProgramExecutionStatus::Completed);
}

#[tokio::test]
async fn test_program_lifecycle_management() {
    let manager = MockProgramDeliveryManager::new("test-device-005".to_string());

    let program_message = ProgramMessage {
        program_id: "lifecycle-test".to_string(),
        program_name: "Lifecycle Management Test".to_string(),
        steel_code: r#"
            (begin
              (log "info" "Lifecycle test program")
              (define counter 0)
              (while (< counter 5)
                (set! counter (+ counter 1))
                (log "info" (format "Counter: ~a" counter))
                (sleep 0.1))
              "lifecycle-completed")
        "#
        .to_string(),
        version: "1.0.0".to_string(),
        checksum: "lifecycle-checksum".to_string(),
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

    // Stop program (even though it's completed)
    manager.stop_program(&program_id).await.unwrap();
    let status = manager.get_program_status(&program_id).await.unwrap();
    assert_eq!(status.status, ProgramExecutionStatus::Stopped);

    // Remove program
    manager.remove_program(&program_id).await.unwrap();
    let result = manager.get_program_status(&program_id).await;
    assert!(result.is_err()); // Should not be found after removal

    // Verify program is no longer in loaded programs
    let loaded_programs = manager.get_loaded_programs().await;
    assert_eq!(loaded_programs.len(), 0);
}

#[tokio::test]
async fn test_multiple_programs() {
    let manager = MockProgramDeliveryManager::new("test-device-006".to_string());

    // Load multiple programs
    for i in 1..=5 {
        let program_message = ProgramMessage {
            program_id: format!("multi-program-{:03}", i),
            program_name: format!("Multi Program {}", i),
            steel_code: format!(
                r#"
                (begin
                  (log "info" "Program {} executing")
                  (sleep 0.1)
                  "program-{}-completed")
                "#,
                i, i
            ),
            version: "1.0.0".to_string(),
            checksum: format!("checksum-{}", i),
            auto_start: i % 2 == 0, // Auto-start even-numbered programs
            metadata: None,
        };

        manager
            .handle_program_message(&program_message)
            .await
            .unwrap();
    }

    // Verify all programs were loaded
    let loaded_programs = manager.get_loaded_programs().await;
    assert_eq!(loaded_programs.len(), 5);

    // Verify auto-started programs executed
    let execution_results = manager.get_execution_results().await;
    assert_eq!(execution_results.len(), 2); // Programs 2 and 4 auto-started

    // Execute remaining programs manually
    for i in [1, 3, 5] {
        let program_id = format!("multi-program-{:03}", i);
        let result = manager.execute_program(&program_id).await.unwrap();
        assert!(result.success);
    }

    // List all programs and verify their status
    let all_programs = manager.list_programs().await.unwrap();
    assert_eq!(all_programs.len(), 5);

    for status in all_programs {
        assert_eq!(status.status, ProgramExecutionStatus::Completed);
    }
}

#[tokio::test]
async fn test_program_validation_failures() {
    let manager = MockProgramDeliveryManager::new("test-device-007".to_string());

    // Test empty program code
    let empty_program = ProgramMessage {
        program_id: "empty-program".to_string(),
        program_name: "Empty Program".to_string(),
        steel_code: "".to_string(),
        version: "1.0.0".to_string(),
        checksum: "empty-checksum".to_string(),
        auto_start: false,
        metadata: None,
    };

    let validation = manager.validate_program(&empty_program).await.unwrap();
    assert!(!validation.valid);
    assert!(validation.error_message.is_some());

    // Test oversized program (mock limit is 10000 characters)
    let oversized_code = "a".repeat(15000);
    let oversized_program = ProgramMessage {
        program_id: "oversized-program".to_string(),
        program_name: "Oversized Program".to_string(),
        steel_code: oversized_code,
        version: "1.0.0".to_string(),
        checksum: "oversized-checksum".to_string(),
        auto_start: false,
        metadata: None,
    };

    let validation = manager.validate_program(&oversized_program).await.unwrap();
    assert!(!validation.valid);
    assert!(!validation.size_valid);
}

#[tokio::test]
async fn test_program_status_reporting() {
    let manager = MockProgramDeliveryManager::new("test-device-008".to_string());

    // Load and execute several programs
    let programs = vec![
        ("status-test-1", true),
        ("status-test-2", false),
        ("status-test-3", true),
    ];

    for (program_id, auto_start) in programs {
        let program_message = ProgramMessage {
            program_id: program_id.to_string(),
            program_name: format!("Status Test {}", program_id),
            steel_code: format!(
                r#"
                (begin
                  (log "info" "Status test program {}")
                  "status-test-completed")
                "#,
                program_id
            ),
            version: "1.0.0".to_string(),
            checksum: format!("status-checksum-{}", program_id),
            auto_start,
            metadata: None,
        };

        manager
            .handle_program_message(&program_message)
            .await
            .unwrap();
    }

    // Execute non-auto-start program
    manager.execute_program("status-test-2").await.unwrap();

    // Test status reporting
    let result = manager.report_status().await;
    assert!(result.is_ok());

    // Verify all programs are listed
    let all_programs = manager.list_programs().await.unwrap();
    assert_eq!(all_programs.len(), 3);

    // Verify individual status queries
    for program_id in ["status-test-1", "status-test-2", "status-test-3"] {
        let status = manager.get_program_status(program_id).await.unwrap();
        assert_eq!(status.program_id, program_id);
        assert_eq!(status.status, ProgramExecutionStatus::Completed);
    }
}

#[tokio::test]
async fn test_program_delivery_manager_with_real_iot_client() {
    let mut iot_client = MockIoTClient::new();
    iot_client.connect().await.unwrap();
    assert_eq!(
        iot_client.get_connection_status(),
        ConnectionStatus::Connected
    );

    let mut manager =
        ProgramDeliveryManager::new("integration-test-device".to_string(), Arc::new(iot_client));

    // Test initialization
    manager.initialize().await.unwrap();

    // Create test program with proper checksum
    let steel_code = r#"
        (begin
          (log "info" "Integration test program starting")
          (led-on)
          (sleep 2)
          (led-off)
          (log "info" "Integration test completed")
          "integration-success")
    "#;

    let program_message = ProgramMessage {
        program_id: "integration-test-program".to_string(),
        program_name: "Integration Test Program".to_string(),
        steel_code: steel_code.to_string(),
        version: "2.0.0".to_string(),
        checksum: manager.calculate_checksum(steel_code),
        auto_start: true,
        metadata: Some(ProgramMetadata {
            description: Some("Integration test for program delivery".to_string()),
            author: Some("Integration Test Suite".to_string()),
            created_at: Utc::now(),
            memory_requirement: Some(4096),
            execution_timeout: Some(120),
        }),
    };

    // Test program validation with correct checksum
    let validation = manager.validate_program(&program_message).await.unwrap();
    assert!(validation.valid);
    assert!(validation.checksum_match);
    assert!(validation.size_valid);
    assert!(validation.syntax_valid);

    // Test program handling
    let status = manager
        .handle_program_message(&program_message)
        .await
        .unwrap();
    assert_eq!(status.program_id, "integration-test-program");

    // Test program execution
    let result = manager
        .execute_program("integration-test-program")
        .await
        .unwrap();
    assert!(result.success);
    assert_eq!(result.program_id, "integration-test-program");
    assert!(result.execution_time_ms > 0);
}

#[tokio::test]
async fn test_checksum_validation() {
    let mut iot_client = MockIoTClient::new();
    iot_client.connect().await.unwrap();

    let manager =
        ProgramDeliveryManager::new("checksum-test-device".to_string(), Arc::new(iot_client));

    let steel_code = "(begin (log \"info\" \"Checksum test\") (+ 1 2 3))";
    let correct_checksum = manager.calculate_checksum(steel_code);

    // Test with correct checksum
    let valid_program = ProgramMessage {
        program_id: "checksum-valid".to_string(),
        program_name: "Valid Checksum Program".to_string(),
        steel_code: steel_code.to_string(),
        version: "1.0.0".to_string(),
        checksum: correct_checksum,
        auto_start: false,
        metadata: None,
    };

    let validation = manager.validate_program(&valid_program).await.unwrap();
    assert!(validation.valid);
    assert!(validation.checksum_match);

    // Test with incorrect checksum
    let invalid_program = ProgramMessage {
        program_id: "checksum-invalid".to_string(),
        program_name: "Invalid Checksum Program".to_string(),
        steel_code: steel_code.to_string(),
        version: "1.0.0".to_string(),
        checksum: "wrong-checksum".to_string(),
        auto_start: false,
        metadata: None,
    };

    let validation = manager.validate_program(&invalid_program).await.unwrap();
    assert!(!validation.valid);
    assert!(!validation.checksum_match);
    assert!(validation.error_message.is_some());
    assert!(validation
        .error_message
        .unwrap()
        .contains("Checksum mismatch"));
}

#[tokio::test]
async fn test_program_metadata_handling() {
    let manager = MockProgramDeliveryManager::new("metadata-test-device".to_string());

    let program_with_metadata = ProgramMessage {
        program_id: "metadata-test-program".to_string(),
        program_name: "Metadata Test Program".to_string(),
        steel_code: r#"
            (begin
              (log "info" "Testing metadata handling")
              (define sensors (list "temperature" "humidity" "pressure"))
              (map (lambda (sensor) 
                     (log "info" (format "Reading ~a sensor" sensor))) 
                   sensors)
              "metadata-test-completed")
        "#
        .to_string(),
        version: "3.1.4".to_string(),
        checksum: "metadata-checksum".to_string(),
        auto_start: false,
        metadata: Some(ProgramMetadata {
            description: Some("Comprehensive sensor reading program with metadata".to_string()),
            author: Some("Sensor Team".to_string()),
            created_at: Utc::now(),
            memory_requirement: Some(8192),
            execution_timeout: Some(180),
        }),
    };

    // Load program
    let program_id = manager.load_program(&program_with_metadata).await.unwrap();

    // Verify program was loaded with metadata
    let loaded_programs = manager.get_loaded_programs().await;
    assert_eq!(loaded_programs.len(), 1);

    let loaded_program = &loaded_programs[0];
    assert_eq!(loaded_program.program_id, "metadata-test-program");
    assert_eq!(loaded_program.version, "3.1.4");
    assert!(loaded_program.metadata.is_some());

    let metadata = loaded_program.metadata.as_ref().unwrap();
    assert_eq!(
        metadata.description,
        Some("Comprehensive sensor reading program with metadata".to_string())
    );
    assert_eq!(metadata.author, Some("Sensor Team".to_string()));
    assert_eq!(metadata.memory_requirement, Some(8192));
    assert_eq!(metadata.execution_timeout, Some(180));

    // Execute program
    let result = manager.execute_program(&program_id).await.unwrap();
    assert!(result.success);
    assert_eq!(result.program_id, "metadata-test-program");
}

#[tokio::test]
async fn test_concurrent_program_operations() {
    let manager = Arc::new(MockProgramDeliveryManager::new(
        "concurrent-test-device".to_string(),
    ));

    // Create multiple programs concurrently
    let mut handles = Vec::new();

    for i in 1..=10 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let program_message = ProgramMessage {
                program_id: format!("concurrent-program-{:02}", i),
                program_name: format!("Concurrent Program {}", i),
                steel_code: format!(
                    r#"
                    (begin
                      (log "info" "Concurrent program {} executing")
                      (sleep 0.05)
                      "concurrent-{}-completed")
                    "#,
                    i, i
                ),
                version: "1.0.0".to_string(),
                checksum: format!("concurrent-checksum-{}", i),
                auto_start: false,
                metadata: None,
            };

            manager_clone.load_program(&program_message).await.unwrap();
            manager_clone
                .execute_program(&format!("concurrent-program-{:02}", i))
                .await
                .unwrap()
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }

    // Verify all programs executed successfully
    assert_eq!(results.len(), 10);
    for result in results {
        assert!(result.success);
        assert!(result.program_id.starts_with("concurrent-program-"));
    }

    // Verify all programs are loaded
    let loaded_programs = manager.get_loaded_programs().await;
    assert_eq!(loaded_programs.len(), 10);

    // Verify all programs completed
    let all_programs = manager.list_programs().await.unwrap();
    assert_eq!(all_programs.len(), 10);

    for status in all_programs {
        assert_eq!(status.status, ProgramExecutionStatus::Completed);
    }
}
