#[cfg(test)]
mod tests {
    use crate::MockSteelRuntime;
    use aws_iot_core::steel_runtime::{ProgramHandle, SteelError};
    use aws_iot_core::SteelRuntime;
    use aws_iot_core::SteelValue;
    use std::time::Duration;

    #[tokio::test]
    async fn test_program_loading() {
        let mut runtime = MockSteelRuntime::new();

        // Test loading valid program
        let program_code = r#"
            (define (test-function x)
              (+ x 1))
            (test-function 5)
        "#;

        let result = runtime
            .load_program(program_code, Some("test-program"))
            .await;
        assert!(result.is_ok());

        let handle = result.unwrap();
        assert!(!handle.id().to_string().is_empty());

        // Verify program was stored
        let programs = runtime.get_loaded_programs().await;
        assert_eq!(programs.len(), 1);
        assert_eq!(programs[0].name, Some("test-program".to_string()));
    }

    #[tokio::test]
    async fn test_program_loading_failure() {
        let mut runtime = MockSteelRuntime::new();
        runtime.set_should_fail_loading(true).await;

        let program_code = "(invalid syntax";
        let result = runtime.load_program(program_code, None).await;
        assert!(result.is_err());

        match result {
            Err(SteelError::Compilation(_)) => {}
            _ => panic!("Expected compilation error"),
        }
    }

    #[tokio::test]
    async fn test_program_execution() {
        let mut runtime = MockSteelRuntime::new();

        // Load and execute program
        let program_code = r#"
            (define (calculate x y)
              (+ (* x 2) y))
            (calculate 3 4)
        "#;

        let handle = runtime
            .load_program(program_code, Some("calc-program"))
            .await
            .expect("Failed to load program");

        let result = runtime.execute_program(handle).await;
        assert!(result.is_ok());

        // Verify execution was recorded
        let history = runtime.get_execution_history().await;
        assert_eq!(history.len(), 1);
        assert!(history[0].result.is_ok());
    }

    #[tokio::test]
    async fn test_program_execution_failure() {
        let mut runtime = MockSteelRuntime::new();
        runtime.set_should_fail_execution(true).await;

        let program_code = "(+ 1 2)";
        let handle = runtime
            .load_program(program_code, None)
            .await
            .expect("Failed to load program");

        let result = runtime.execute_program(handle).await;
        assert!(result.is_err());

        match result {
            Err(SteelError::Runtime(_)) => {}
            _ => panic!("Expected runtime error"),
        }

        // Verify failure was recorded
        let history = runtime.get_execution_history().await;
        assert_eq!(history.len(), 1);
        assert!(history[0].result.is_err());
    }

    #[tokio::test]
    async fn test_direct_code_execution() {
        let mut runtime = MockSteelRuntime::new();

        // Test simple expression
        let result = runtime.execute_code("(+ 2 3)").await;
        assert!(result.is_ok());

        // Test function definition and call
        let code = r#"
            (define (square x) (* x x))
            (square 4)
        "#;
        let result = runtime.execute_code(code).await;
        assert!(result.is_ok());

        // Verify executions were recorded
        let history = runtime.get_execution_history().await;
        assert_eq!(history.len(), 2);
    }

    #[tokio::test]
    async fn test_global_variables() {
        let mut runtime = MockSteelRuntime::new();

        // Set global variable
        let value = SteelValue::NumV(42.0);
        let result = runtime.set_global_variable("test-var", value.clone()).await;
        assert!(result.is_ok());

        // Get global variable
        let retrieved = runtime.get_global_variable("test-var").await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap(), Some(value));

        // Get non-existent variable
        let missing = runtime.get_global_variable("missing-var").await;
        assert!(missing.is_ok());
        assert_eq!(missing.unwrap(), None);
    }

    #[tokio::test]
    async fn test_program_management() {
        let mut runtime = MockSteelRuntime::new();

        // Load multiple programs
        let program1 = runtime
            .load_program("(+ 1 2)", Some("prog1"))
            .await
            .expect("Failed to load program 1");
        let _program2 = runtime
            .load_program("(* 3 4)", Some("prog2"))
            .await
            .expect("Failed to load program 2");

        // Verify both programs are loaded
        let programs = runtime.get_loaded_programs().await;
        assert_eq!(programs.len(), 2);

        // Remove one program
        let result = runtime.remove_program(program1).await;
        assert!(result.is_ok());

        // Verify only one program remains
        let programs = runtime.get_loaded_programs().await;
        assert_eq!(programs.len(), 1);

        // Try to remove non-existent program
        let fake_handle = ProgramHandle::new();
        let result = runtime.remove_program(fake_handle).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_event_handling() {
        let mut runtime = MockSteelRuntime::new();

        // Load a program to use as event handler
        let handler_program = runtime
            .load_program("(display \"Event handled\")", Some("handler"))
            .await
            .expect("Failed to load handler program");

        // Register event handler
        let result = runtime
            .register_event_handler("test-event", handler_program)
            .await;
        assert!(result.is_ok());

        // Emit event
        let event_data = SteelValue::StringV("test data".to_string());
        let result = runtime.emit_event("test-event", event_data).await;
        assert!(result.is_ok());

        // Try to emit event with no handler
        let result = runtime
            .emit_event("unknown-event", SteelValue::BoolV(true))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execution_timing() {
        let mut runtime = MockSteelRuntime::new();
        runtime.set_execution_delay(100).await; // 100ms delay

        let start = std::time::Instant::now();
        let result = runtime.execute_code("(+ 1 1)").await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration >= Duration::from_millis(90)); // Allow some tolerance

        // Verify timing was recorded
        let history = runtime.get_execution_history().await;
        assert_eq!(history.len(), 1);
        assert!(history[0].duration_ms >= 90);
    }

    #[tokio::test]
    async fn test_concurrent_program_execution() {
        let runtime = std::sync::Arc::new(tokio::sync::Mutex::new(MockSteelRuntime::new()));

        let mut handles = Vec::new();

        // Execute multiple programs concurrently
        for i in 0..5 {
            let runtime_clone = runtime.clone();
            let handle = tokio::spawn(async move {
                let mut rt = runtime_clone.lock().await;
                rt.execute_code(&format!("(+ {} {})", i, i + 1)).await
            });
            handles.push(handle);
        }

        // Wait for all executions to complete
        for handle in handles {
            let result = handle.await.expect("Task panicked");
            assert!(result.is_ok());
        }

        // Verify all executions were recorded
        let rt = runtime.lock().await;
        let history = rt.get_execution_history().await;
        assert_eq!(history.len(), 5);
    }

    #[tokio::test]
    async fn test_steel_value_types() {
        let mut runtime = MockSteelRuntime::new();

        // Test different Steel value types
        let test_cases = vec![
            ("(+ 1 2)", "NumV"),
            ("\"hello world\"", "StringV"),
            ("#t", "BoolV"),
            ("'(1 2 3)", "ListV or other"),
        ];

        for (code, _expected_type) in test_cases {
            let result = runtime.execute_code(code).await;
            assert!(result.is_ok(), "Failed to execute: {}", code);

            // In a real implementation, we'd check the actual SteelValue type
            // For the mock, we just verify execution succeeded
        }
    }

    #[tokio::test]
    async fn test_error_recovery() {
        let mut runtime = MockSteelRuntime::new();

        // Execute failing code
        runtime.set_should_fail_execution(true).await;
        let result = runtime.execute_code("(error \"test error\")").await;
        assert!(result.is_err());

        // Reset failure mode and execute successful code
        runtime.set_should_fail_execution(false).await;
        let result = runtime.execute_code("(+ 1 1)").await;
        assert!(result.is_ok());

        // Verify both executions were recorded
        let history = runtime.get_execution_history().await;
        assert_eq!(history.len(), 2);
        assert!(history[0].result.is_err());
        assert!(history[1].result.is_ok());
    }

    #[tokio::test]
    async fn test_runtime_cleanup() {
        let mut runtime = MockSteelRuntime::new();

        // Load programs and execute code
        let _handle1 = runtime
            .load_program("(+ 1 2)", Some("prog1"))
            .await
            .unwrap();
        let _handle2 = runtime
            .load_program("(* 3 4)", Some("prog2"))
            .await
            .unwrap();
        runtime.execute_code("(+ 5 6)").await.unwrap();
        runtime
            .set_global_variable("test", SteelValue::NumV(42.0))
            .await
            .unwrap();

        // Verify data exists
        assert_eq!(runtime.get_loaded_programs().await.len(), 2);
        assert_eq!(runtime.get_execution_history().await.len(), 1);
        assert_eq!(runtime.get_global_variables().await.len(), 1);

        // Clear all data
        runtime.clear_test_data().await;

        // Verify cleanup
        assert_eq!(runtime.get_loaded_programs().await.len(), 0);
        assert_eq!(runtime.get_execution_history().await.len(), 0);
        assert_eq!(runtime.get_global_variables().await.len(), 0);
    }
}
