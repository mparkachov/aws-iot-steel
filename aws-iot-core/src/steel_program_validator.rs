use crate::SystemResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Steel program validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub metadata: ProgramMetadata,
    pub complexity_score: u32,
    pub estimated_memory_usage: usize,
}

/// Validation error with location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub error_type: ValidationErrorType,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub context: Option<String>,
}

/// Types of validation errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationErrorType {
    SyntaxError,
    UnbalancedParentheses,
    UnterminatedString,
    InvalidFunctionCall,
    UndefinedFunction,
    TypeMismatch,
    SecurityViolation,
    MemoryLimitExceeded,
    ComplexityTooHigh,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub warning_type: ValidationWarningType,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub suggestion: Option<String>,
}

/// Types of validation warnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationWarningType {
    UnusedVariable,
    DeepNesting,
    LongFunction,
    PotentialInfiniteLoop,
    DeprecatedFunction,
    PerformanceIssue,
    StyleIssue,
}

/// Program metadata extracted during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramMetadata {
    pub functions_defined: Vec<String>,
    pub functions_called: Vec<String>,
    pub variables_used: Vec<String>,
    pub max_nesting_depth: usize,
    pub line_count: usize,
    pub character_count: usize,
    pub estimated_execution_time: f64, // seconds
}

/// Steel program validator and syntax checker
pub struct SteelProgramValidator {
    max_complexity: u32,
    max_memory_usage: usize,
    max_nesting_depth: usize,
    allowed_functions: HashMap<String, FunctionSignature>,
    security_rules: Vec<SecurityRule>,
}

/// Function signature for validation
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub min_args: usize,
    pub max_args: Option<usize>,
    pub arg_types: Vec<ArgumentType>,
    pub return_type: ArgumentType,
    pub is_deprecated: bool,
    pub security_level: SecurityLevel,
}

/// Argument types for function validation
#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentType {
    Any,
    Number,
    String,
    Boolean,
    List,
    Function,
    Symbol,
}

/// Security levels for functions
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityLevel {
    Safe,       // No security concerns
    Restricted, // Requires careful usage
    Dangerous,  // Should be avoided or heavily restricted
}

/// Security rule for program validation
#[derive(Debug, Clone)]
pub struct SecurityRule {
    pub name: String,
    pub pattern: String,
    pub severity: SecuritySeverity,
    pub message: String,
}

/// Security rule severity
#[derive(Debug, Clone, PartialEq)]
pub enum SecuritySeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl Default for SteelProgramValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl SteelProgramValidator {
    /// Create a new Steel program validator with default settings
    pub fn new() -> Self {
        let mut validator = Self {
            max_complexity: 1000,
            max_memory_usage: 1024 * 1024, // 1MB
            max_nesting_depth: 20,
            allowed_functions: HashMap::new(),
            security_rules: Vec::new(),
        };

        validator.initialize_default_functions();
        validator.initialize_security_rules();
        validator
    }

    /// Create a validator with custom limits
    pub fn with_limits(
        max_complexity: u32,
        max_memory_usage: usize,
        max_nesting_depth: usize,
    ) -> Self {
        let mut validator = Self {
            max_complexity,
            max_memory_usage,
            max_nesting_depth,
            allowed_functions: HashMap::new(),
            security_rules: Vec::new(),
        };

        validator.initialize_default_functions();
        validator.initialize_security_rules();
        validator
    }

    /// Validate a Steel program
    pub fn validate(&self, code: &str) -> SystemResult<ValidationResult> {
        info!("Starting Steel program validation");
        debug!("Code length: {} characters", code.len());

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Basic syntax validation
        if let Err(syntax_errors) = self.validate_syntax(code) {
            errors.extend(syntax_errors);
        }

        // Extract metadata
        let metadata = self.extract_metadata(code)?;

        // Complexity analysis
        let complexity_score = self.calculate_complexity(code, &metadata);
        if complexity_score > self.max_complexity {
            errors.push(ValidationError {
                error_type: ValidationErrorType::ComplexityTooHigh,
                message: format!(
                    "Program complexity {} exceeds maximum {}",
                    complexity_score, self.max_complexity
                ),
                line: None,
                column: None,
                context: None,
            });
        }

        // Memory usage estimation
        let estimated_memory = self.estimate_memory_usage(&metadata);
        if estimated_memory > self.max_memory_usage {
            errors.push(ValidationError {
                error_type: ValidationErrorType::MemoryLimitExceeded,
                message: format!(
                    "Estimated memory usage {} bytes exceeds limit {} bytes",
                    estimated_memory, self.max_memory_usage
                ),
                line: None,
                column: None,
                context: None,
            });
        }

        // Function validation
        if let Err(function_errors) = self.validate_functions(code, &metadata) {
            errors.extend(function_errors);
        }

        // Security validation
        if let Err(security_errors) = self.validate_security(code) {
            errors.extend(security_errors);
        }

        // Generate warnings
        warnings.extend(self.generate_warnings(code, &metadata));

        let is_valid = errors.is_empty();

        let result = ValidationResult {
            is_valid,
            errors,
            warnings,
            metadata,
            complexity_score,
            estimated_memory_usage: estimated_memory,
        };

        if is_valid {
            info!("Steel program validation passed");
        } else {
            warn!(
                "Steel program validation failed with {} errors",
                result.errors.len()
            );
        }

        Ok(result)
    }

    /// Validate basic syntax (parentheses, strings, etc.)
    fn validate_syntax(&self, code: &str) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let mut paren_stack = Vec::new();
        let mut in_string = false;
        let mut escape_next = false;
        let mut line = 1;
        let mut column = 1;

        for ch in code.chars() {
            if escape_next {
                escape_next = false;
                column += 1;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '(' if !in_string => {
                    paren_stack.push((line, column));
                }
                ')' if !in_string => {
                    if paren_stack.is_empty() {
                        errors.push(ValidationError {
                            error_type: ValidationErrorType::UnbalancedParentheses,
                            message: "Unmatched closing parenthesis".to_string(),
                            line: Some(line),
                            column: Some(column),
                            context: None,
                        });
                    } else {
                        paren_stack.pop();
                    }
                }
                '\n' => {
                    line += 1;
                    column = 1;
                    continue;
                }
                _ => {}
            }

            column += 1;
        }

        // Check for unmatched opening parentheses
        for (paren_line, paren_column) in paren_stack {
            errors.push(ValidationError {
                error_type: ValidationErrorType::UnbalancedParentheses,
                message: "Unmatched opening parenthesis".to_string(),
                line: Some(paren_line),
                column: Some(paren_column),
                context: None,
            });
        }

        // Check for unterminated strings
        if in_string {
            errors.push(ValidationError {
                error_type: ValidationErrorType::UnterminatedString,
                message: "Unterminated string literal".to_string(),
                line: Some(line),
                column: Some(column),
                context: None,
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Extract program metadata
    fn extract_metadata(&self, code: &str) -> SystemResult<ProgramMetadata> {
        let lines: Vec<&str> = code.lines().collect();
        let line_count = lines.len();
        let character_count = code.len();

        let mut functions_defined = Vec::new();
        let mut functions_called = Vec::new();
        let mut variables_used = Vec::new();
        let mut max_nesting_depth = 0;
        let mut current_depth: i32 = 0;

        // Simple parsing to extract function definitions and calls
        for line in &lines {
            let trimmed = line.trim();

            // Count nesting depth
            for ch in line.chars() {
                match ch {
                    '(' => {
                        current_depth += 1;
                        max_nesting_depth = max_nesting_depth.max(current_depth);
                    }
                    ')' => current_depth = current_depth.saturating_sub(1),
                    _ => {}
                }
            }

            // Extract function definitions
            if trimmed.starts_with("(define (") {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    functions_defined.push(func_name);
                }
            }

            // Extract function calls (simplified)
            functions_called.extend(self.extract_function_calls(trimmed));
            variables_used.extend(self.extract_variables(trimmed));
        }

        // Remove duplicates
        functions_defined.sort();
        functions_defined.dedup();
        functions_called.sort();
        functions_called.dedup();
        variables_used.sort();
        variables_used.dedup();

        // Estimate execution time based on complexity
        let estimated_execution_time =
            self.estimate_execution_time(line_count, max_nesting_depth as usize);

        Ok(ProgramMetadata {
            functions_defined,
            functions_called,
            variables_used,
            max_nesting_depth: max_nesting_depth as usize,
            line_count,
            character_count,
            estimated_execution_time,
        })
    }

    /// Calculate program complexity score
    fn calculate_complexity(&self, _code: &str, metadata: &ProgramMetadata) -> u32 {
        let mut complexity = 0;

        // Base complexity from line count
        complexity += metadata.line_count as u32;

        // Complexity from nesting depth
        complexity += (metadata.max_nesting_depth as u32) * 5;

        // Complexity from function definitions
        complexity += (metadata.functions_defined.len() as u32) * 10;

        // Complexity from function calls
        complexity += (metadata.functions_called.len() as u32) * 2;

        // Complexity from variables
        complexity += metadata.variables_used.len() as u32;

        complexity
    }

    /// Estimate memory usage
    fn estimate_memory_usage(&self, metadata: &ProgramMetadata) -> usize {
        let mut memory = 0;

        // Base memory for program storage
        memory += metadata.character_count;

        // Memory for function definitions (estimated)
        memory += metadata.functions_defined.len() * 1024;

        // Memory for variables (estimated)
        memory += metadata.variables_used.len() * 256;

        // Memory for execution stack (based on nesting depth)
        memory += metadata.max_nesting_depth * 512;

        memory
    }

    /// Validate function calls
    fn validate_functions(
        &self,
        _code: &str,
        metadata: &ProgramMetadata,
    ) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for function_name in &metadata.functions_called {
            if let Some(signature) = self.allowed_functions.get(function_name) {
                // Check if function is deprecated
                if signature.is_deprecated {
                    // This would be a warning, not an error
                    continue;
                }

                // Check security level
                if signature.security_level == SecurityLevel::Dangerous {
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::SecurityViolation,
                        message: format!("Dangerous function '{}' is not allowed", function_name),
                        line: None,
                        column: None,
                        context: Some(format!("Function: {}", function_name)),
                    });
                }
            } else if !metadata.functions_defined.contains(function_name) {
                // Function is not defined in the program and not in allowed functions
                errors.push(ValidationError {
                    error_type: ValidationErrorType::UndefinedFunction,
                    message: format!("Undefined function: {}", function_name),
                    line: None,
                    column: None,
                    context: Some(format!("Function: {}", function_name)),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate security rules
    fn validate_security(&self, code: &str) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for rule in &self.security_rules {
            if code.contains(&rule.pattern) {
                match rule.severity {
                    SecuritySeverity::Error | SecuritySeverity::Critical => {
                        errors.push(ValidationError {
                            error_type: ValidationErrorType::SecurityViolation,
                            message: rule.message.clone(),
                            line: None,
                            column: None,
                            context: Some(format!("Rule: {}", rule.name)),
                        });
                    }
                    _ => {
                        // Warnings are handled separately
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Generate warnings for potential issues
    fn generate_warnings(&self, code: &str, metadata: &ProgramMetadata) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        // Check for deep nesting
        if metadata.max_nesting_depth > 10 {
            warnings.push(ValidationWarning {
                warning_type: ValidationWarningType::DeepNesting,
                message: format!(
                    "Deep nesting detected: {} levels",
                    metadata.max_nesting_depth
                ),
                line: None,
                column: None,
                suggestion: Some(
                    "Consider breaking complex expressions into smaller functions".to_string(),
                ),
            });
        }

        // Check for long programs
        if metadata.line_count > 200 {
            warnings.push(ValidationWarning {
                warning_type: ValidationWarningType::LongFunction,
                message: format!("Long program: {} lines", metadata.line_count),
                line: None,
                column: None,
                suggestion: Some("Consider splitting into multiple smaller programs".to_string()),
            });
        }

        // Check for deprecated functions
        for function_name in &metadata.functions_called {
            if let Some(signature) = self.allowed_functions.get(function_name) {
                if signature.is_deprecated {
                    warnings.push(ValidationWarning {
                        warning_type: ValidationWarningType::DeprecatedFunction,
                        message: format!("Deprecated function used: {}", function_name),
                        line: None,
                        column: None,
                        suggestion: Some("Consider using a newer alternative".to_string()),
                    });
                }
            }
        }

        // Check security warnings
        for rule in &self.security_rules {
            if code.contains(&rule.pattern) && rule.severity == SecuritySeverity::Warning {
                warnings.push(ValidationWarning {
                    warning_type: ValidationWarningType::PerformanceIssue,
                    message: rule.message.clone(),
                    line: None,
                    column: None,
                    suggestion: None,
                });
            }
        }

        warnings
    }

    /// Initialize default allowed functions
    fn initialize_default_functions(&mut self) {
        // Hardware control functions
        self.allowed_functions.insert(
            "sleep".to_string(),
            FunctionSignature {
                name: "sleep".to_string(),
                min_args: 1,
                max_args: Some(1),
                arg_types: vec![ArgumentType::Number],
                return_type: ArgumentType::Boolean,
                is_deprecated: false,
                security_level: SecurityLevel::Safe,
            },
        );

        self.allowed_functions.insert(
            "led-on".to_string(),
            FunctionSignature {
                name: "led-on".to_string(),
                min_args: 0,
                max_args: Some(0),
                arg_types: vec![],
                return_type: ArgumentType::Boolean,
                is_deprecated: false,
                security_level: SecurityLevel::Safe,
            },
        );

        self.allowed_functions.insert(
            "led-off".to_string(),
            FunctionSignature {
                name: "led-off".to_string(),
                min_args: 0,
                max_args: Some(0),
                arg_types: vec![],
                return_type: ArgumentType::Boolean,
                is_deprecated: false,
                security_level: SecurityLevel::Safe,
            },
        );

        self.allowed_functions.insert(
            "led-state".to_string(),
            FunctionSignature {
                name: "led-state".to_string(),
                min_args: 0,
                max_args: Some(0),
                arg_types: vec![],
                return_type: ArgumentType::Boolean,
                is_deprecated: false,
                security_level: SecurityLevel::Safe,
            },
        );

        // System information functions
        self.allowed_functions.insert(
            "device-info".to_string(),
            FunctionSignature {
                name: "device-info".to_string(),
                min_args: 0,
                max_args: Some(0),
                arg_types: vec![],
                return_type: ArgumentType::List,
                is_deprecated: false,
                security_level: SecurityLevel::Safe,
            },
        );

        self.allowed_functions.insert(
            "memory-info".to_string(),
            FunctionSignature {
                name: "memory-info".to_string(),
                min_args: 0,
                max_args: Some(0),
                arg_types: vec![],
                return_type: ArgumentType::List,
                is_deprecated: false,
                security_level: SecurityLevel::Safe,
            },
        );

        self.allowed_functions.insert(
            "uptime".to_string(),
            FunctionSignature {
                name: "uptime".to_string(),
                min_args: 0,
                max_args: Some(0),
                arg_types: vec![],
                return_type: ArgumentType::Number,
                is_deprecated: false,
                security_level: SecurityLevel::Safe,
            },
        );

        // Logging functions
        self.allowed_functions.insert(
            "log".to_string(),
            FunctionSignature {
                name: "log".to_string(),
                min_args: 2,
                max_args: Some(2),
                arg_types: vec![ArgumentType::String, ArgumentType::String],
                return_type: ArgumentType::Boolean,
                is_deprecated: false,
                security_level: SecurityLevel::Safe,
            },
        );

        // Standard Scheme functions
        let standard_functions = vec![
            "define",
            "lambda",
            "let",
            "let*",
            "letrec",
            "if",
            "cond",
            "case",
            "and",
            "or",
            "not",
            "begin",
            "do",
            "map",
            "filter",
            "fold",
            "apply",
            "call/cc",
            "values",
            "call-with-values",
            "+",
            "-",
            "*",
            "/",
            "=",
            "<",
            ">",
            "<=",
            ">=",
            "eq?",
            "eqv?",
            "equal?",
            "cons",
            "car",
            "cdr",
            "list",
            "length",
            "append",
            "reverse",
            "null?",
            "pair?",
            "number?",
            "string?",
            "boolean?",
            "symbol?",
            "procedure?",
            "string-append",
            "string-length",
            "substring",
            "string->number",
            "number->string",
            "display",
            "newline",
            "write",
            "read",
            "format",
        ];

        for func in standard_functions {
            self.allowed_functions.insert(
                func.to_string(),
                FunctionSignature {
                    name: func.to_string(),
                    min_args: 0,
                    max_args: None,
                    arg_types: vec![ArgumentType::Any],
                    return_type: ArgumentType::Any,
                    is_deprecated: false,
                    security_level: SecurityLevel::Safe,
                },
            );
        }
    }

    /// Initialize security rules
    fn initialize_security_rules(&mut self) {
        self.security_rules.push(SecurityRule {
            name: "infinite_loop_warning".to_string(),
            pattern: "(while #t".to_string(),
            severity: SecuritySeverity::Warning,
            message: "Potential infinite loop detected".to_string(),
        });

        self.security_rules.push(SecurityRule {
            name: "eval_usage".to_string(),
            pattern: "(eval".to_string(),
            severity: SecuritySeverity::Error,
            message: "Dynamic code evaluation is not allowed for security reasons".to_string(),
        });

        self.security_rules.push(SecurityRule {
            name: "system_call".to_string(),
            pattern: "(system".to_string(),
            severity: SecuritySeverity::Critical,
            message: "System calls are not allowed".to_string(),
        });
    }

    /// Extract function name from definition
    fn extract_function_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("(define (") {
            let after_define = &line[start + 9..];
            if let Some(end) = after_define.find(' ') {
                Some(after_define[..end].to_string())
            } else {
                after_define
                    .find(')')
                    .map(|end| after_define[..end].to_string())
            }
        } else {
            None
        }
    }

    /// Extract function calls from a line (simplified)
    fn extract_function_calls(&self, line: &str) -> Vec<String> {
        let mut calls = Vec::new();
        let chars = line.chars().peekable();
        let mut in_string = false;
        let mut current_token = String::new();
        let mut after_paren = false;

        for ch in chars {
            match ch {
                '"' => in_string = !in_string,
                '(' if !in_string => {
                    after_paren = true;
                    current_token.clear();
                }
                ' ' | ')' | '\t' | '\n' if !in_string => {
                    if after_paren && !current_token.is_empty() {
                        calls.push(current_token.clone());
                        after_paren = false;
                    }
                    current_token.clear();
                }
                _ if !in_string => {
                    current_token.push(ch);
                }
                _ => {}
            }
        }

        calls
    }

    /// Extract variables from a line (simplified)
    fn extract_variables(&self, _line: &str) -> Vec<String> {
        // This is a simplified implementation
        // In a real implementation, you'd want proper parsing
        Vec::new()
    }

    /// Estimate execution time based on program characteristics
    fn estimate_execution_time(&self, line_count: usize, nesting_depth: usize) -> f64 {
        // Simple heuristic: base time + time per line + time per nesting level
        let base_time = 0.001; // 1ms base
        let time_per_line = 0.0001; // 0.1ms per line
        let time_per_depth = 0.001; // 1ms per nesting level

        base_time + (line_count as f64 * time_per_line) + (nesting_depth as f64 * time_per_depth)
    }

    /// Add a custom function to the allowed list
    pub fn add_allowed_function(&mut self, signature: FunctionSignature) {
        self.allowed_functions
            .insert(signature.name.clone(), signature);
    }

    /// Add a custom security rule
    pub fn add_security_rule(&mut self, rule: SecurityRule) {
        self.security_rules.push(rule);
    }

    /// Get validation statistics
    pub fn get_validation_stats(&self) -> ValidationStats {
        ValidationStats {
            allowed_functions_count: self.allowed_functions.len(),
            security_rules_count: self.security_rules.len(),
            max_complexity: self.max_complexity,
            max_memory_usage: self.max_memory_usage,
            max_nesting_depth: self.max_nesting_depth,
        }
    }
}

/// Validation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    pub allowed_functions_count: usize,
    pub security_rules_count: usize,
    pub max_complexity: u32,
    pub max_memory_usage: usize,
    pub max_nesting_depth: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_validation() {
        let validator = SteelProgramValidator::new();

        let valid_code = r#"
            (define (test-function x)
              (if (> x 0)
                  (led-on)
                  (led-off)))
            
            (test-function 5)
        "#;

        let result = validator.validate(valid_code).unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_syntax_error_detection() {
        let validator = SteelProgramValidator::new();

        let invalid_code = r#"
            (define (test-function x
              (if (> x 0)
                  (led-on)
                  (led-off))
        "#;

        let result = validator.validate(invalid_code).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e.error_type, ValidationErrorType::UnbalancedParentheses)));
    }

    #[test]
    fn test_undefined_function_detection() {
        let validator = SteelProgramValidator::new();

        let code_with_undefined = r#"
            (define (test-function)
              (undefined-function 123))
        "#;

        let result = validator.validate(code_with_undefined).unwrap();
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e.error_type, ValidationErrorType::UndefinedFunction)));
    }

    #[test]
    fn test_security_violation_detection() {
        let validator = SteelProgramValidator::new();

        let dangerous_code = r#"
            (define (dangerous-function)
              (eval "(system \"rm -rf /\")"))
        "#;

        let result = validator.validate(dangerous_code).unwrap();
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e.error_type, ValidationErrorType::SecurityViolation)));
    }

    #[test]
    fn test_complexity_calculation() {
        let validator = SteelProgramValidator::with_limits(50, 1024, 5);

        let complex_code = r#"
            (define (complex-function x)
              (if (> x 0)
                  (if (< x 10)
                      (if (= x 5)
                          (if (even? x)
                              (if (positive? x)
                                  (led-on)
                                  (led-off))
                              (sleep 1))
                          (display "not 5"))
                      (display "too big"))
                  (display "negative")))
        "#;

        let result = validator.validate(complex_code).unwrap();
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e.error_type, ValidationErrorType::ComplexityTooHigh)));
    }

    #[test]
    fn test_metadata_extraction() {
        let validator = SteelProgramValidator::new();

        let code = r#"
            (define (helper x) (+ x 1))
            (define (main-function y)
              (let ((result (helper y)))
                (if (> result 10)
                    (led-on)
                    (led-off))))
        "#;

        let result = validator.validate(code).unwrap();
        assert!(result
            .metadata
            .functions_defined
            .contains(&"helper".to_string()));
        assert!(result
            .metadata
            .functions_defined
            .contains(&"main-function".to_string()));
        assert!(result
            .metadata
            .functions_called
            .contains(&"led-on".to_string()));
        assert!(result
            .metadata
            .functions_called
            .contains(&"led-off".to_string()));
    }
}
