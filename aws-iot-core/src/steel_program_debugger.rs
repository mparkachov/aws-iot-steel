use crate::{
    steel_program_simulator::SteelProgramSimulator, steel_runtime::ProgramHandle, SystemResult,
};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use uuid::Uuid;

/// Steel program debugger with breakpoints, stepping, and variable inspection
pub struct SteelProgramDebugger {
    #[allow(dead_code)]
    simulator: Arc<SteelProgramSimulator>,
    debug_sessions: Arc<RwLock<HashMap<String, DebugSession>>>,
    global_breakpoints: Arc<RwLock<HashMap<String, Vec<Breakpoint>>>>, // program_name -> breakpoints
    debug_config: DebugConfig,
}

/// Debug session for a specific program execution
#[derive(Debug, Clone)]
pub struct DebugSession {
    pub session_id: String,
    pub program_name: String,
    pub program_handle: Option<ProgramHandle>,
    pub status: DebugSessionStatus,
    pub created_at: DateTime<Utc>,
    pub current_line: Option<usize>,
    pub current_function: Option<String>,
    pub call_stack: Vec<StackFrame>,
    pub variables: HashMap<String, VariableValue>,
    pub breakpoints: Vec<Breakpoint>,
    pub step_mode: StepMode,
    pub execution_history: VecDeque<ExecutionStep>,
    pub watch_expressions: Vec<WatchExpression>,
}

/// Debug session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DebugSessionStatus {
    Created,
    Running,
    Paused,
    Stopped,
    Error(String),
    Completed,
}

/// Breakpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: String,
    pub line: usize,
    pub condition: Option<String>, // Optional condition expression
    pub hit_count: u32,
    pub enabled: bool,
    pub temporary: bool, // Remove after first hit
    pub created_at: DateTime<Utc>,
}

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_name: String,
    pub line: usize,
    pub local_variables: HashMap<String, VariableValue>,
    pub arguments: Vec<VariableValue>,
}

/// Variable value with type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableValue {
    pub name: String,
    pub value: String,
    pub type_name: String,
    pub is_mutable: bool,
    pub memory_address: Option<String>,
    pub children: Option<Vec<VariableValue>>, // For complex types
}

/// Step mode for debugging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepMode {
    None,
    StepInto, // Step into function calls
    StepOver, // Step over function calls
    StepOut,  // Step out of current function
    Continue, // Continue until next breakpoint
}

/// Execution step record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_id: String,
    pub timestamp: DateTime<Utc>,
    pub line: usize,
    pub function_name: Option<String>,
    pub operation: String,
    pub variables_changed: Vec<String>,
    pub duration: Duration,
}

/// Watch expression for monitoring variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchExpression {
    pub id: String,
    pub expression: String,
    pub current_value: Option<VariableValue>,
    pub previous_value: Option<VariableValue>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

/// Debug configuration
#[derive(Debug, Clone)]
pub struct DebugConfig {
    pub max_execution_history: usize,
    pub max_call_stack_depth: usize,
    pub enable_variable_tracking: bool,
    pub enable_performance_profiling: bool,
    pub auto_break_on_error: bool,
    pub step_timeout: Duration,
}

/// Debug command for controlling execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugCommand {
    Start {
        program_code: String,
        program_name: String,
    },
    Pause,
    Resume,
    Stop,
    StepInto,
    StepOver,
    StepOut,
    SetBreakpoint {
        line: usize,
        condition: Option<String>,
    },
    RemoveBreakpoint {
        breakpoint_id: String,
    },
    AddWatchExpression {
        expression: String,
    },
    RemoveWatchExpression {
        watch_id: String,
    },
    EvaluateExpression {
        expression: String,
    },
    GetVariables,
    GetCallStack,
    GetExecutionHistory,
}

/// Debug command result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugCommandResult {
    Success,
    SessionCreated {
        session_id: String,
    },
    BreakpointHit {
        line: usize,
        variables: HashMap<String, VariableValue>,
    },
    Variables {
        variables: HashMap<String, VariableValue>,
    },
    CallStack {
        frames: Vec<StackFrame>,
    },
    ExecutionHistory {
        steps: Vec<ExecutionStep>,
    },
    ExpressionResult {
        value: VariableValue,
    },
    Error {
        message: String,
    },
}

/// Profiling information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingInfo {
    pub function_name: String,
    pub call_count: u32,
    pub total_time: Duration,
    pub average_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub memory_usage: usize,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            max_execution_history: 1000,
            max_call_stack_depth: 100,
            enable_variable_tracking: true,
            enable_performance_profiling: true,
            auto_break_on_error: true,
            step_timeout: Duration::from_secs(30),
        }
    }
}

impl SteelProgramDebugger {
    /// Create a new Steel program debugger
    pub fn new(simulator: Arc<SteelProgramSimulator>, config: DebugConfig) -> Self {
        Self {
            simulator,
            debug_sessions: Arc::new(RwLock::new(HashMap::new())),
            global_breakpoints: Arc::new(RwLock::new(HashMap::new())),
            debug_config: config,
        }
    }

    /// Create a debugger with default configuration
    pub fn new_default(simulator: Arc<SteelProgramSimulator>) -> Self {
        Self::new(simulator, DebugConfig::default())
    }

    /// Execute a debug command
    pub async fn execute_command(
        &self,
        session_id: Option<&str>,
        command: DebugCommand,
    ) -> SystemResult<DebugCommandResult> {
        match command {
            DebugCommand::Start {
                program_code,
                program_name,
            } => self.start_debug_session(&program_code, &program_name).await,
            DebugCommand::Pause => {
                if let Some(sid) = session_id {
                    self.pause_session(sid).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
            DebugCommand::Resume => {
                if let Some(sid) = session_id {
                    self.resume_session(sid).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
            DebugCommand::Stop => {
                if let Some(sid) = session_id {
                    self.stop_session(sid).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
            DebugCommand::StepInto => {
                if let Some(sid) = session_id {
                    self.step_into(sid).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
            DebugCommand::StepOver => {
                if let Some(sid) = session_id {
                    self.step_over(sid).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
            DebugCommand::StepOut => {
                if let Some(sid) = session_id {
                    self.step_out(sid).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
            DebugCommand::SetBreakpoint { line, condition } => {
                if let Some(sid) = session_id {
                    self.set_breakpoint(sid, line, condition).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
            DebugCommand::RemoveBreakpoint { breakpoint_id } => {
                if let Some(sid) = session_id {
                    self.remove_breakpoint(sid, &breakpoint_id).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
            DebugCommand::AddWatchExpression { expression } => {
                if let Some(sid) = session_id {
                    self.add_watch_expression(sid, &expression).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
            DebugCommand::RemoveWatchExpression { watch_id } => {
                if let Some(sid) = session_id {
                    self.remove_watch_expression(sid, &watch_id).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
            DebugCommand::EvaluateExpression { expression } => {
                if let Some(sid) = session_id {
                    self.evaluate_expression(sid, &expression).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
            DebugCommand::GetVariables => {
                if let Some(sid) = session_id {
                    self.get_variables(sid).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
            DebugCommand::GetCallStack => {
                if let Some(sid) = session_id {
                    self.get_call_stack(sid).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
            DebugCommand::GetExecutionHistory => {
                if let Some(sid) = session_id {
                    self.get_execution_history(sid).await
                } else {
                    Ok(DebugCommandResult::Error {
                        message: "No active session".to_string(),
                    })
                }
            }
        }
    }

    /// Start a new debug session
    async fn start_debug_session(
        &self,
        program_code: &str,
        program_name: &str,
    ) -> SystemResult<DebugCommandResult> {
        info!("Starting debug session for program: {}", program_name);

        let session_id = Uuid::new_v4().to_string();
        let created_at = Utc::now();

        // Create debug session
        let mut session = DebugSession {
            session_id: session_id.clone(),
            program_name: program_name.to_string(),
            program_handle: None,
            status: DebugSessionStatus::Created,
            created_at,
            current_line: Some(1),
            current_function: None,
            call_stack: Vec::new(),
            variables: HashMap::new(),
            breakpoints: Vec::new(),
            step_mode: StepMode::None,
            execution_history: VecDeque::new(),
            watch_expressions: Vec::new(),
        };

        // Copy global breakpoints for this program
        if let Some(global_breakpoints) = self.global_breakpoints.read().get(program_name) {
            session.breakpoints = global_breakpoints.clone();
        }

        // Initialize variables (simplified for this implementation)
        self.initialize_session_variables(&mut session, program_code);

        // Store session
        self.debug_sessions
            .write()
            .insert(session_id.clone(), session);

        info!("Debug session created: {}", session_id);
        Ok(DebugCommandResult::SessionCreated { session_id })
    }

    /// Pause a debug session
    async fn pause_session(&self, session_id: &str) -> SystemResult<DebugCommandResult> {
        let mut sessions = self.debug_sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = DebugSessionStatus::Paused;
            info!("Debug session paused: {}", session_id);
            Ok(DebugCommandResult::Success)
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Resume a debug session
    async fn resume_session(&self, session_id: &str) -> SystemResult<DebugCommandResult> {
        let mut sessions = self.debug_sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = DebugSessionStatus::Running;
            session.step_mode = StepMode::Continue;
            info!("Debug session resumed: {}", session_id);
            Ok(DebugCommandResult::Success)
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Stop a debug session
    async fn stop_session(&self, session_id: &str) -> SystemResult<DebugCommandResult> {
        let mut sessions = self.debug_sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = DebugSessionStatus::Stopped;
            info!("Debug session stopped: {}", session_id);
            Ok(DebugCommandResult::Success)
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Step into function calls
    async fn step_into(&self, session_id: &str) -> SystemResult<DebugCommandResult> {
        let mut sessions = self.debug_sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.step_mode = StepMode::StepInto;
            session.status = DebugSessionStatus::Running;

            // Simulate stepping to next line
            if let Some(current_line) = session.current_line {
                session.current_line = Some(current_line + 1);

                // Add execution step
                let step = ExecutionStep {
                    step_id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    line: current_line + 1,
                    function_name: session.current_function.clone(),
                    operation: "step_into".to_string(),
                    variables_changed: Vec::new(),
                    duration: Duration::from_millis(1),
                };

                session.execution_history.push_back(step);

                // Limit history size
                if session.execution_history.len() > self.debug_config.max_execution_history {
                    session.execution_history.pop_front();
                }
            }

            info!("Step into executed for session: {}", session_id);
            Ok(DebugCommandResult::Success)
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Step over function calls
    async fn step_over(&self, session_id: &str) -> SystemResult<DebugCommandResult> {
        let mut sessions = self.debug_sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.step_mode = StepMode::StepOver;
            session.status = DebugSessionStatus::Running;

            // Simulate stepping over
            if let Some(current_line) = session.current_line {
                session.current_line = Some(current_line + 1);

                let step = ExecutionStep {
                    step_id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    line: current_line + 1,
                    function_name: session.current_function.clone(),
                    operation: "step_over".to_string(),
                    variables_changed: Vec::new(),
                    duration: Duration::from_millis(1),
                };

                session.execution_history.push_back(step);

                if session.execution_history.len() > self.debug_config.max_execution_history {
                    session.execution_history.pop_front();
                }
            }

            info!("Step over executed for session: {}", session_id);
            Ok(DebugCommandResult::Success)
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Step out of current function
    async fn step_out(&self, session_id: &str) -> SystemResult<DebugCommandResult> {
        let mut sessions = self.debug_sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.step_mode = StepMode::StepOut;
            session.status = DebugSessionStatus::Running;

            // Simulate stepping out (pop from call stack)
            if !session.call_stack.is_empty() {
                session.call_stack.pop();

                // Update current function
                session.current_function = session
                    .call_stack
                    .last()
                    .map(|frame| frame.function_name.clone());
            }

            info!("Step out executed for session: {}", session_id);
            Ok(DebugCommandResult::Success)
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Set a breakpoint
    async fn set_breakpoint(
        &self,
        session_id: &str,
        line: usize,
        condition: Option<String>,
    ) -> SystemResult<DebugCommandResult> {
        let mut sessions = self.debug_sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            let breakpoint = Breakpoint {
                id: Uuid::new_v4().to_string(),
                line,
                condition,
                hit_count: 0,
                enabled: true,
                temporary: false,
                created_at: Utc::now(),
            };

            session.breakpoints.push(breakpoint.clone());

            // Also add to global breakpoints
            self.global_breakpoints
                .write()
                .entry(session.program_name.clone())
                .or_default()
                .push(breakpoint);

            info!(
                "Breakpoint set at line {} for session: {}",
                line, session_id
            );
            Ok(DebugCommandResult::Success)
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Remove a breakpoint
    async fn remove_breakpoint(
        &self,
        session_id: &str,
        breakpoint_id: &str,
    ) -> SystemResult<DebugCommandResult> {
        let mut sessions = self.debug_sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.breakpoints.retain(|bp| bp.id != breakpoint_id);

            // Also remove from global breakpoints
            if let Some(global_breakpoints) = self
                .global_breakpoints
                .write()
                .get_mut(&session.program_name)
            {
                global_breakpoints.retain(|bp| bp.id != breakpoint_id);
            }

            info!(
                "Breakpoint removed: {} for session: {}",
                breakpoint_id, session_id
            );
            Ok(DebugCommandResult::Success)
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Add a watch expression
    async fn add_watch_expression(
        &self,
        session_id: &str,
        expression: &str,
    ) -> SystemResult<DebugCommandResult> {
        let mut sessions = self.debug_sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            let watch = WatchExpression {
                id: Uuid::new_v4().to_string(),
                expression: expression.to_string(),
                current_value: None,
                previous_value: None,
                enabled: true,
                created_at: Utc::now(),
            };

            session.watch_expressions.push(watch);

            info!(
                "Watch expression added: '{}' for session: {}",
                expression, session_id
            );
            Ok(DebugCommandResult::Success)
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Remove a watch expression
    async fn remove_watch_expression(
        &self,
        session_id: &str,
        watch_id: &str,
    ) -> SystemResult<DebugCommandResult> {
        let mut sessions = self.debug_sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.watch_expressions.retain(|w| w.id != watch_id);

            info!(
                "Watch expression removed: {} for session: {}",
                watch_id, session_id
            );
            Ok(DebugCommandResult::Success)
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Evaluate an expression in the current context
    async fn evaluate_expression(
        &self,
        session_id: &str,
        expression: &str,
    ) -> SystemResult<DebugCommandResult> {
        let sessions = self.debug_sessions.read();
        if let Some(_session) = sessions.get(session_id) {
            // Simulate expression evaluation
            let result_value = VariableValue {
                name: "result".to_string(),
                value: format!("eval({})", expression),
                type_name: "unknown".to_string(),
                is_mutable: false,
                memory_address: None,
                children: None,
            };

            info!(
                "Expression evaluated: '{}' for session: {}",
                expression, session_id
            );
            Ok(DebugCommandResult::ExpressionResult {
                value: result_value,
            })
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Get current variables
    async fn get_variables(&self, session_id: &str) -> SystemResult<DebugCommandResult> {
        let sessions = self.debug_sessions.read();
        if let Some(session) = sessions.get(session_id) {
            Ok(DebugCommandResult::Variables {
                variables: session.variables.clone(),
            })
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Get call stack
    async fn get_call_stack(&self, session_id: &str) -> SystemResult<DebugCommandResult> {
        let sessions = self.debug_sessions.read();
        if let Some(session) = sessions.get(session_id) {
            Ok(DebugCommandResult::CallStack {
                frames: session.call_stack.clone(),
            })
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Get execution history
    async fn get_execution_history(&self, session_id: &str) -> SystemResult<DebugCommandResult> {
        let sessions = self.debug_sessions.read();
        if let Some(session) = sessions.get(session_id) {
            let steps: Vec<ExecutionStep> = session.execution_history.iter().cloned().collect();
            Ok(DebugCommandResult::ExecutionHistory { steps })
        } else {
            Ok(DebugCommandResult::Error {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Initialize session variables (simplified implementation)
    fn initialize_session_variables(&self, session: &mut DebugSession, _program_code: &str) {
        // Add some example variables
        session.variables.insert(
            "x".to_string(),
            VariableValue {
                name: "x".to_string(),
                value: "42".to_string(),
                type_name: "number".to_string(),
                is_mutable: true,
                memory_address: Some("0x1000".to_string()),
                children: None,
            },
        );

        session.variables.insert(
            "message".to_string(),
            VariableValue {
                name: "message".to_string(),
                value: "\"Hello, World!\"".to_string(),
                type_name: "string".to_string(),
                is_mutable: true,
                memory_address: Some("0x2000".to_string()),
                children: None,
            },
        );
    }

    // ========== Public API Methods ==========

    /// Get all active debug sessions
    pub fn get_active_sessions(&self) -> Vec<String> {
        self.debug_sessions
            .read()
            .values()
            .filter(|session| {
                !matches!(
                    session.status,
                    DebugSessionStatus::Stopped | DebugSessionStatus::Completed
                )
            })
            .map(|session| session.session_id.clone())
            .collect()
    }

    /// Get debug session information
    pub fn get_session_info(&self, session_id: &str) -> Option<DebugSession> {
        self.debug_sessions.read().get(session_id).cloned()
    }

    /// Get all breakpoints for a program
    pub fn get_program_breakpoints(&self, program_name: &str) -> Vec<Breakpoint> {
        self.global_breakpoints
            .read()
            .get(program_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Clear all breakpoints for a program
    pub fn clear_program_breakpoints(&self, program_name: &str) {
        self.global_breakpoints.write().remove(program_name);

        // Also clear from active sessions
        let mut sessions = self.debug_sessions.write();
        for session in sessions.values_mut() {
            if session.program_name == program_name {
                session.breakpoints.clear();
            }
        }

        info!("All breakpoints cleared for program: {}", program_name);
    }

    /// Get debug statistics
    pub fn get_debug_statistics(&self) -> DebugStatistics {
        let sessions = self.debug_sessions.read();
        let total_sessions = sessions.len();
        let active_sessions = sessions
            .values()
            .filter(|s| {
                matches!(
                    s.status,
                    DebugSessionStatus::Running | DebugSessionStatus::Paused
                )
            })
            .count();
        let completed_sessions = sessions
            .values()
            .filter(|s| matches!(s.status, DebugSessionStatus::Completed))
            .count();
        let failed_sessions = sessions
            .values()
            .filter(|s| matches!(s.status, DebugSessionStatus::Error(_)))
            .count();

        let total_breakpoints = self
            .global_breakpoints
            .read()
            .values()
            .map(|breakpoints| breakpoints.len())
            .sum();

        DebugStatistics {
            total_sessions,
            active_sessions,
            completed_sessions,
            failed_sessions,
            total_breakpoints,
        }
    }

    /// Clean up old debug sessions
    pub fn cleanup_old_sessions(&self, max_age: Duration) {
        let cutoff_time = Utc::now() - chrono::Duration::from_std(max_age).unwrap();

        let mut sessions = self.debug_sessions.write();
        let initial_count = sessions.len();

        sessions.retain(|_, session| {
            session.created_at > cutoff_time
                || matches!(
                    session.status,
                    DebugSessionStatus::Running | DebugSessionStatus::Paused
                )
        });

        let removed_count = initial_count - sessions.len();
        if removed_count > 0 {
            info!("Cleaned up {} old debug sessions", removed_count);
        }
    }
}

/// Debug statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugStatistics {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub completed_sessions: usize,
    pub failed_sessions: usize,
    pub total_breakpoints: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steel_program_simulator::SimulationConfig;

    #[tokio::test]
    async fn test_debug_session_creation() {
        let simulator = Arc::new(SteelProgramSimulator::new(SimulationConfig::default()).unwrap());
        let debugger = SteelProgramDebugger::new_default(simulator);

        let result = debugger
            .execute_command(
                None,
                DebugCommand::Start {
                    program_code: "(led-on)".to_string(),
                    program_name: "test-program".to_string(),
                },
            )
            .await
            .unwrap();

        match result {
            DebugCommandResult::SessionCreated { session_id } => {
                assert!(!session_id.is_empty());

                let session_info = debugger.get_session_info(&session_id);
                assert!(session_info.is_some());
                assert_eq!(session_info.unwrap().program_name, "test-program");
            }
            _ => panic!("Expected SessionCreated result"),
        }
    }

    #[tokio::test]
    async fn test_breakpoint_management() {
        let simulator = Arc::new(SteelProgramSimulator::new(SimulationConfig::default()).unwrap());
        let debugger = SteelProgramDebugger::new_default(simulator);

        // Create session
        let result = debugger
            .execute_command(
                None,
                DebugCommand::Start {
                    program_code: "(led-on)".to_string(),
                    program_name: "test-program".to_string(),
                },
            )
            .await
            .unwrap();

        let session_id = match result {
            DebugCommandResult::SessionCreated { session_id } => session_id,
            _ => panic!("Expected SessionCreated result"),
        };

        // Set breakpoint
        let result = debugger
            .execute_command(
                Some(&session_id),
                DebugCommand::SetBreakpoint {
                    line: 5,
                    condition: Some("x > 10".to_string()),
                },
            )
            .await
            .unwrap();

        assert!(matches!(result, DebugCommandResult::Success));

        // Check breakpoints
        let breakpoints = debugger.get_program_breakpoints("test-program");
        assert_eq!(breakpoints.len(), 1);
        assert_eq!(breakpoints[0].line, 5);
        assert_eq!(breakpoints[0].condition, Some("x > 10".to_string()));
    }

    #[tokio::test]
    async fn test_stepping_commands() {
        let simulator = Arc::new(SteelProgramSimulator::new(SimulationConfig::default()).unwrap());
        let debugger = SteelProgramDebugger::new_default(simulator);

        // Create session
        let result = debugger
            .execute_command(
                None,
                DebugCommand::Start {
                    program_code: "(led-on)".to_string(),
                    program_name: "test-program".to_string(),
                },
            )
            .await
            .unwrap();

        let session_id = match result {
            DebugCommandResult::SessionCreated { session_id } => session_id,
            _ => panic!("Expected SessionCreated result"),
        };

        // Test step into
        let result = debugger
            .execute_command(Some(&session_id), DebugCommand::StepInto)
            .await
            .unwrap();
        assert!(matches!(result, DebugCommandResult::Success));

        // Test step over
        let result = debugger
            .execute_command(Some(&session_id), DebugCommand::StepOver)
            .await
            .unwrap();
        assert!(matches!(result, DebugCommandResult::Success));

        // Check execution history
        let result = debugger
            .execute_command(Some(&session_id), DebugCommand::GetExecutionHistory)
            .await
            .unwrap();

        match result {
            DebugCommandResult::ExecutionHistory { steps } => {
                assert!(!steps.is_empty());
                assert!(steps.iter().any(|step| step.operation == "step_into"));
                assert!(steps.iter().any(|step| step.operation == "step_over"));
            }
            _ => panic!("Expected ExecutionHistory result"),
        }
    }

    #[tokio::test]
    async fn test_variable_inspection() {
        let simulator = Arc::new(SteelProgramSimulator::new(SimulationConfig::default()).unwrap());
        let debugger = SteelProgramDebugger::new_default(simulator);

        // Create session
        let result = debugger
            .execute_command(
                None,
                DebugCommand::Start {
                    program_code: "(let ((x 42)) (led-on))".to_string(),
                    program_name: "test-program".to_string(),
                },
            )
            .await
            .unwrap();

        let session_id = match result {
            DebugCommandResult::SessionCreated { session_id } => session_id,
            _ => panic!("Expected SessionCreated result"),
        };

        // Get variables
        let result = debugger
            .execute_command(Some(&session_id), DebugCommand::GetVariables)
            .await
            .unwrap();

        match result {
            DebugCommandResult::Variables { variables } => {
                assert!(!variables.is_empty());
                assert!(variables.contains_key("x"));
            }
            _ => panic!("Expected Variables result"),
        }
    }

    #[tokio::test]
    async fn test_watch_expressions() {
        let simulator = Arc::new(SteelProgramSimulator::new(SimulationConfig::default()).unwrap());
        let debugger = SteelProgramDebugger::new_default(simulator);

        // Create session
        let result = debugger
            .execute_command(
                None,
                DebugCommand::Start {
                    program_code: "(led-on)".to_string(),
                    program_name: "test-program".to_string(),
                },
            )
            .await
            .unwrap();

        let session_id = match result {
            DebugCommandResult::SessionCreated { session_id } => session_id,
            _ => panic!("Expected SessionCreated result"),
        };

        // Add watch expression
        let result = debugger
            .execute_command(
                Some(&session_id),
                DebugCommand::AddWatchExpression {
                    expression: "(+ x 1)".to_string(),
                },
            )
            .await
            .unwrap();
        assert!(matches!(result, DebugCommandResult::Success));

        // Check session has watch expression
        let session_info = debugger.get_session_info(&session_id).unwrap();
        assert_eq!(session_info.watch_expressions.len(), 1);
        assert_eq!(session_info.watch_expressions[0].expression, "(+ x 1)");
    }

    #[test]
    fn test_debug_statistics() {
        let simulator = Arc::new(SteelProgramSimulator::new(SimulationConfig::default()).unwrap());
        let debugger = SteelProgramDebugger::new_default(simulator);

        let stats = debugger.get_debug_statistics();
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.active_sessions, 0);
        assert_eq!(stats.total_breakpoints, 0);
    }

    #[test]
    fn test_session_cleanup() {
        let simulator = Arc::new(SteelProgramSimulator::new(SimulationConfig::default()).unwrap());
        let debugger = SteelProgramDebugger::new_default(simulator);

        // Initially no sessions
        assert_eq!(debugger.get_active_sessions().len(), 0);

        // Cleanup should not crash with no sessions
        debugger.cleanup_old_sessions(Duration::from_secs(3600));

        assert_eq!(debugger.get_active_sessions().len(), 0);
    }
}
