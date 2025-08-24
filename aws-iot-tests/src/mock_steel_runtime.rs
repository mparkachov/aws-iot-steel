use async_trait::async_trait;
use aws_iot_core::{
    SteelRuntime, SteelValue
};
use aws_iot_core::steel_runtime::{SteelResult, SteelError, ProgramHandle, ProgramInfo, ExecutionStats, ExecutionContext};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Mock implementation of SteelRuntime for testing
#[derive(Debug)]
pub struct MockSteelRuntime {
    programs: Arc<Mutex<HashMap<ProgramHandle, StoredProgram>>>,
    global_variables: Arc<Mutex<HashMap<String, SteelValue>>>,
    execution_history: Arc<Mutex<Vec<ExecutionRecord>>>,
    event_handlers: Arc<Mutex<HashMap<String, ProgramHandle>>>,
    should_fail_execution: Arc<Mutex<bool>>,
    should_fail_loading: Arc<Mutex<bool>>,
    execution_delay_ms: Arc<Mutex<u64>>,
}

#[derive(Debug, Clone)]
pub struct StoredProgram {
    pub handle: ProgramHandle,
    pub name: Option<String>,
    pub code: String,
    pub loaded_at: DateTime<Utc>,
    pub execution_count: u32,
}

#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub handle: Option<ProgramHandle>,
    pub code: Option<String>,
    pub result: Result<SteelValue, String>,
    pub executed_at: DateTime<Utc>,
    pub duration_ms: u64,
}

impl MockSteelRuntime {
    pub fn new() -> Self {
        Self {
            programs: Arc::new(Mutex::new(HashMap::new())),
            global_variables: Arc::new(Mutex::new(HashMap::new())),
            execution_history: Arc::new(Mutex::new(Vec::new())),
            event_handlers: Arc::new(Mutex::new(HashMap::new())),
            should_fail_execution: Arc::new(Mutex::new(false)),
            should_fail_loading: Arc::new(Mutex::new(false)),
            execution_delay_ms: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Get execution history for testing
    pub async fn get_execution_history(&self) -> Vec<ExecutionRecord> {
        self.execution_history.lock().await.clone()
    }
    
    /// Get loaded programs for testing
    pub async fn get_loaded_programs(&self) -> Vec<StoredProgram> {
        self.programs.lock().await.values().cloned().collect()
    }
    
    /// Get global variables for testing
    pub async fn get_global_variables(&self) -> HashMap<String, SteelValue> {
        self.global_variables.lock().await.clone()
    }
    
    /// Set whether execution should fail
    pub async fn set_should_fail_execution(&self, should_fail: bool) {
        *self.should_fail_execution.lock().await = should_fail;
    }
    
    /// Set whether loading should fail
    pub async fn set_should_fail_loading(&self, should_fail: bool) {
        *self.should_fail_loading.lock().await = should_fail;
    }
    
    /// Set execution delay for testing
    pub async fn set_execution_delay(&self, delay_ms: u64) {
        *self.execution_delay_ms.lock().await = delay_ms;
    }
    
    /// Clear all test data
    pub async fn clear_test_data(&self) {
        self.programs.lock().await.clear();
        self.global_variables.lock().await.clear();
        self.execution_history.lock().await.clear();
        self.event_handlers.lock().await.clear();
    }
    
    /// Simulate a successful execution result
    fn create_mock_result(code: &str) -> SteelValue {
        // Simple mock result based on code content
        if code.contains("sleep") {
            SteelValue::BoolV(true)
        } else if code.contains("led") {
            SteelValue::StringV("led_state_changed".to_string())
        } else if code.contains("device-info") {
            SteelValue::StringV("mock-device-001".to_string())
        } else if code.contains("error") || code.contains("fail") {
            SteelValue::BoolV(false)
        } else {
            SteelValue::NumV(42.0)
        }
    }
}

impl Default for MockSteelRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SteelRuntime for MockSteelRuntime {
    async fn load_program(&mut self, program: &str, name: Option<&str>) -> SteelResult<ProgramHandle> {
        if *self.should_fail_loading.lock().await {
            return Err(SteelError::Compilation("Mock loading failure".to_string()));
        }
        
        let handle = ProgramHandle::new();
        let stored_program = StoredProgram {
            handle: handle.clone(),
            name: name.map(|s| s.to_string()),
            code: program.to_string(),
            loaded_at: Utc::now(),
            execution_count: 0,
        };
        
        self.programs.lock().await.insert(handle.clone(), stored_program);
        Ok(handle)
    }
    
    async fn execute_program(&mut self, handle: ProgramHandle) -> SteelResult<SteelValue> {
        let delay = *self.execution_delay_ms.lock().await;
        if delay > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }
        
        if *self.should_fail_execution.lock().await {
            let error_msg = "Mock execution failure".to_string();
            let record = ExecutionRecord {
                handle: Some(handle),
                code: None,
                result: Err(error_msg.clone()),
                executed_at: Utc::now(),
                duration_ms: delay,
            };
            self.execution_history.lock().await.push(record);
            return Err(SteelError::Runtime(error_msg));
        }
        
        let mut programs = self.programs.lock().await;
        if let Some(program) = programs.get_mut(&handle) {
            program.execution_count += 1;
            let result = Self::create_mock_result(&program.code);
            
            let record = ExecutionRecord {
                handle: Some(handle),
                code: Some(program.code.clone()),
                result: Ok(result.clone()),
                executed_at: Utc::now(),
                duration_ms: delay,
            };
            self.execution_history.lock().await.push(record);
            
            Ok(result)
        } else {
            Err(SteelError::Runtime("Program not found".to_string()))
        }
    }
    
    async fn execute_code(&mut self, code: &str) -> SteelResult<SteelValue> {
        let delay = *self.execution_delay_ms.lock().await;
        if delay > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }
        
        if *self.should_fail_execution.lock().await {
            let error_msg = "Mock execution failure".to_string();
            let record = ExecutionRecord {
                handle: None,
                code: Some(code.to_string()),
                result: Err(error_msg.clone()),
                executed_at: Utc::now(),
                duration_ms: delay,
            };
            self.execution_history.lock().await.push(record);
            return Err(SteelError::Runtime(error_msg));
        }
        
        let result = Self::create_mock_result(code);
        let record = ExecutionRecord {
            handle: None,
            code: Some(code.to_string()),
            result: Ok(result.clone()),
            executed_at: Utc::now(),
            duration_ms: delay,
        };
        self.execution_history.lock().await.push(record);
        
        Ok(result)
    }
    
    fn list_programs(&self) -> Vec<ProgramInfo> {
        // This is a synchronous method, so we can't use async lock
        // In a real implementation, this would need to be redesigned
        // For now, return empty vec in mock
        Vec::new()
    }
    
    async fn remove_program(&mut self, handle: ProgramHandle) -> SteelResult<()> {
        let mut programs = self.programs.lock().await;
        if programs.remove(&handle).is_some() {
            Ok(())
        } else {
            Err(SteelError::Runtime("Program not found".to_string()))
        }
    }
    
    async fn set_global_variable(&mut self, name: &str, value: SteelValue) -> SteelResult<()> {
        self.global_variables.lock().await.insert(name.to_string(), value);
        Ok(())
    }
    
    async fn get_global_variable(&self, name: &str) -> SteelResult<Option<SteelValue>> {
        Ok(self.global_variables.lock().await.get(name).cloned())
    }
    
    fn get_execution_stats(&self) -> ExecutionStats {
        // Return mock stats
        ExecutionStats {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time_ms: 0.0,
            total_programs_loaded: 0,
        }
    }
    
    async fn register_event_handler(&mut self, event: &str, handler: ProgramHandle) -> SteelResult<()> {
        self.event_handlers.lock().await.insert(event.to_string(), handler);
        Ok(())
    }
    
    async fn emit_event(&mut self, event: &str, _data: SteelValue) -> SteelResult<()> {
        let handlers = self.event_handlers.lock().await;
        if handlers.contains_key(event) {
            // In a real implementation, this would execute the handler
            Ok(())
        } else {
            Err(SteelError::Runtime(format!("No handler for event: {}", event)))
        }
    }
    
    fn get_execution_context(&self) -> ExecutionContext {
        ExecutionContext::default()
    }
}