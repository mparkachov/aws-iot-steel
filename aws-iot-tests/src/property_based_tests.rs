#[cfg(test)]
mod tests {
    use crate::MockSteelRuntime;
    use aws_iot_core::{SteelRuntime, SteelValue};
    use std::collections::HashSet;

    // Property-based test utilities
    fn generate_valid_steel_expressions() -> Vec<String> {
        vec![
            "(+ 1 2)".to_string(),
            "(* 3 4)".to_string(),
            "(- 10 5)".to_string(),
            "(/ 8 2)".to_string(),
            "(= 5 5)".to_string(),
            "(< 3 7)".to_string(),
            "(> 9 2)".to_string(),
            "42".to_string(),
            "\"hello\"".to_string(),
            "#t".to_string(),
            "#f".to_string(),
            "'(1 2 3)".to_string(),
            "(if #t 1 0)".to_string(),
            "(cond [(= 1 1) \"yes\"] [else \"no\"])".to_string(),
            "(let ([x 5]) (+ x 1))".to_string(),
            "(define (square x) (* x x))".to_string(),
            "(lambda (x) (+ x 1))".to_string(),
            "(begin (+ 1 2) (* 3 4))".to_string(),
        ]
    }

    fn generate_program_names() -> Vec<Option<String>> {
        vec![
            None,
            Some("test".to_string()),
            Some("".to_string()),
            Some("a".repeat(100)),
            Some("test-program-123".to_string()),
            Some("program with spaces".to_string()),
            Some("program_with_underscores".to_string()),
            Some("program-with-dashes".to_string()),
            Some("UPPERCASE".to_string()),
            Some("MixedCase".to_string()),
        ]
    }

    #[tokio::test]
    async fn property_all_valid_expressions_should_load() {
        let mut runtime = MockSteelRuntime::new();
        let valid_expressions = generate_valid_steel_expressions();

        for expr in valid_expressions {
            let result = runtime.load_program(&expr, None).await;
            assert!(result.is_ok(), "Failed to load valid expression: {}", expr);
        }
    }

    #[tokio::test]
    async fn property_loaded_programs_should_execute() {
        let mut runtime = MockSteelRuntime::new();
        let valid_expressions = generate_valid_steel_expressions();
        let mut handles = Vec::new();

        // Load all programs
        for expr in &valid_expressions {
            let handle = runtime
                .load_program(expr, None)
                .await
                .unwrap_or_else(|_| panic!("Failed to load: {}", expr));
            handles.push(handle);
        }

        // Execute all programs
        for (i, handle) in handles.into_iter().enumerate() {
            let result = runtime.execute_program(handle).await;
            assert!(
                result.is_ok(),
                "Failed to execute program {}: {}",
                i,
                valid_expressions[i]
            );
        }
    }

    #[tokio::test]
    async fn property_program_names_should_be_preserved() {
        let mut runtime = MockSteelRuntime::new();
        let program_names = generate_program_names();
        let mut loaded_handles = Vec::new();

        for name in &program_names {
            let handle = runtime
                .load_program("(+ 1 1)", name.as_deref())
                .await
                .expect("Failed to load program");
            loaded_handles.push((handle, name.clone()));
        }

        let loaded_programs = runtime.get_loaded_programs().await;
        assert_eq!(loaded_programs.len(), program_names.len());

        // Verify names are preserved (in mock implementation)
        for (handle, expected_name) in loaded_handles.iter() {
            // Find the stored program with matching handle
            let stored_program = loaded_programs
                .iter()
                .find(|p| p.handle.id() == handle.id())
                .expect("Program not found");
            assert_eq!(stored_program.name, *expected_name);
        }
    }

    #[tokio::test]
    async fn property_execution_order_independence() {
        let mut runtime = MockSteelRuntime::new();
        let expressions = vec!["(+ 1 2)", "(* 3 4)", "(- 10 5)", "(/ 8 2)"];

        // Execute in original order
        let mut results1 = Vec::new();
        for expr in &expressions {
            let result = runtime.execute_code(expr).await.unwrap();
            results1.push(result);
        }

        runtime.clear_test_data().await;

        // Execute in reverse order
        let mut results2 = Vec::new();
        for expr in expressions.iter().rev() {
            let result = runtime.execute_code(expr).await.unwrap();
            results2.push(result);
        }

        // Results should be the same (order-independent)
        results2.reverse();

        // In mock implementation, results are deterministic based on content
        for (r1, r2) in results1.iter().zip(results2.iter()) {
            // Mock results are based on code content, so they should match
            assert_eq!(format!("{:?}", r1), format!("{:?}", r2));
        }
    }

    #[tokio::test]
    async fn property_concurrent_execution_safety() {
        let runtime = std::sync::Arc::new(tokio::sync::Mutex::new(MockSteelRuntime::new()));
        let expressions = generate_valid_steel_expressions();

        let mut handles = Vec::new();

        // Execute all expressions concurrently
        for expr in expressions {
            let runtime_clone = runtime.clone();
            let handle = tokio::spawn(async move {
                let mut rt = runtime_clone.lock().await;
                rt.execute_code(&expr).await
            });
            handles.push(handle);
        }

        // All should complete successfully
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await.expect("Task panicked");
            assert!(result.is_ok());
            results.push(result.unwrap());
        }

        // Verify all executions were recorded
        let rt = runtime.lock().await;
        let history = rt.get_execution_history().await;
        assert_eq!(history.len(), results.len());
    }

    #[tokio::test]
    async fn property_program_handle_uniqueness() {
        let mut runtime = MockSteelRuntime::new();
        let mut handles = HashSet::new();

        // Load many programs and verify handle uniqueness
        for i in 0..100 {
            let program = format!("(+ {} {})", i, i + 1);
            let handle = runtime
                .load_program(&program, Some(&format!("prog_{}", i)))
                .await
                .expect("Failed to load program");

            // Handle should be unique
            let handle_str = handle.id().to_string();
            assert!(
                !handles.contains(&handle_str),
                "Duplicate handle: {}",
                handle.id()
            );
            handles.insert(handle_str);
        }

        assert_eq!(handles.len(), 100);
    }

    #[tokio::test]
    async fn property_global_variable_consistency() {
        let mut runtime = MockSteelRuntime::new();

        let test_values = vec![
            ("num_var", SteelValue::NumV(42.0)),
            ("string_var", SteelValue::StringV("hello".to_string())),
            ("bool_var", SteelValue::BoolV(true)),
            ("zero_var", SteelValue::NumV(0.0)),
            ("empty_string", SteelValue::StringV("".to_string())),
            ("false_var", SteelValue::BoolV(false)),
        ];

        // Set all variables
        for (name, value) in &test_values {
            let result = runtime.set_global_variable(name, value.clone()).await;
            assert!(result.is_ok(), "Failed to set variable: {}", name);
        }

        // Retrieve and verify all variables
        for (name, expected_value) in &test_values {
            let result = runtime.get_global_variable(name).await;
            assert!(result.is_ok(), "Failed to get variable: {}", name);

            let retrieved_value = result.unwrap();
            assert_eq!(
                retrieved_value,
                Some(expected_value.clone()),
                "Variable {} has wrong value",
                name
            );
        }

        // Verify non-existent variables return None
        let missing = runtime.get_global_variable("non_existent").await.unwrap();
        assert_eq!(missing, None);
    }

    #[tokio::test]
    async fn property_execution_history_completeness() {
        let mut runtime = MockSteelRuntime::new();
        let expressions = generate_valid_steel_expressions();

        // Execute all expressions
        for expr in &expressions {
            runtime.execute_code(expr).await.expect("Execution failed");
        }

        // Verify history completeness
        let history = runtime.get_execution_history().await;
        assert_eq!(history.len(), expressions.len());

        // Verify each execution is recorded with proper data
        for (i, record) in history.iter().enumerate() {
            assert!(record.code.is_some(), "Missing code for execution {}", i);
            assert!(record.result.is_ok(), "Execution {} failed", i);
            assert!(
                record.executed_at <= chrono::Utc::now(),
                "Invalid timestamp for execution {}",
                i
            );
        }
    }

    #[tokio::test]
    async fn property_error_isolation() {
        let mut runtime = MockSteelRuntime::new();

        // Mix valid and invalid operations
        let operations = vec![
            ("valid1", "(+ 1 2)", false),
            ("invalid1", "invalid", true),
            ("valid2", "(* 3 4)", false),
            ("invalid2", "also invalid", true),
            ("valid3", "(- 5 1)", false),
        ];

        for (name, code, should_fail) in operations {
            if should_fail {
                runtime.set_should_fail_execution(true).await;
            } else {
                runtime.set_should_fail_execution(false).await;
            }

            let result = runtime.execute_code(code).await;

            if should_fail {
                assert!(result.is_err(), "Expected {} to fail", name);
            } else {
                assert!(result.is_ok(), "Expected {} to succeed", name);
            }
        }

        // Verify all operations were recorded
        let history = runtime.get_execution_history().await;
        assert_eq!(history.len(), 5);

        // Verify error isolation - failures don't affect subsequent successes
        assert!(history[0].result.is_ok()); // valid1
        assert!(history[1].result.is_err()); // invalid1
        assert!(history[2].result.is_ok()); // valid2
        assert!(history[3].result.is_err()); // invalid2
        assert!(history[4].result.is_ok()); // valid3
    }

    #[tokio::test]
    async fn property_resource_cleanup() {
        let mut runtime = MockSteelRuntime::new();

        // Load many programs
        let mut handles = Vec::new();
        for i in 0..50 {
            let handle = runtime
                .load_program(&format!("(+ {} 1)", i), Some(&format!("prog_{}", i)))
                .await
                .expect("Failed to load program");
            handles.push(handle);
        }

        // Verify programs are loaded
        assert_eq!(runtime.get_loaded_programs().await.len(), 50);

        // Remove half the programs
        for handle in handles.into_iter().take(25) {
            let result = runtime.remove_program(handle).await;
            assert!(result.is_ok(), "Failed to remove program");
        }

        // Verify correct number remain
        assert_eq!(runtime.get_loaded_programs().await.len(), 25);

        // Clear all data
        runtime.clear_test_data().await;

        // Verify complete cleanup
        assert_eq!(runtime.get_loaded_programs().await.len(), 0);
        assert_eq!(runtime.get_execution_history().await.len(), 0);
        assert_eq!(runtime.get_global_variables().await.len(), 0);
    }
}
