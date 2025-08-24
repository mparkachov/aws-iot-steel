/// Steel Program Validator for Embedded Constraints
/// This module provides validation and optimization for Steel programs
/// to ensure they meet embedded system constraints
use crate::{SystemError, SystemResult};
use std::collections::HashSet;
use tracing::debug;

/// Maximum program size for embedded targets (bytes)
#[cfg(target_arch = "riscv32")]
pub const MAX_PROGRAM_SIZE: usize = 4096; // 4KB
#[cfg(not(target_arch = "riscv32"))]
pub const MAX_PROGRAM_SIZE: usize = 65536; // 64KB

/// Maximum nesting depth for Steel expressions
#[cfg(target_arch = "riscv32")]
pub const MAX_NESTING_DEPTH: usize = 16;
#[cfg(not(target_arch = "riscv32"))]
pub const MAX_NESTING_DEPTH: usize = 64;

/// Maximum number of variables in a program
#[cfg(target_arch = "riscv32")]
pub const MAX_VARIABLES: usize = 32;
#[cfg(not(target_arch = "riscv32"))]
pub const MAX_VARIABLES: usize = 256;

/// Steel program validation results
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub size_bytes: usize,
    pub max_nesting_depth: usize,
    pub variable_count: usize,
    pub function_count: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub estimated_memory_usage: usize,
    pub estimated_execution_time_ms: u64,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            size_bytes: 0,
            max_nesting_depth: 0,
            variable_count: 0,
            function_count: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
            estimated_memory_usage: 0,
            estimated_execution_time_ms: 0,
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn print_summary(&self) {
        println!("Steel Program Validation Results:");
        println!("  Valid: {}", self.is_valid);
        println!("  Size: {} bytes", self.size_bytes);
        println!("  Max Nesting: {}", self.max_nesting_depth);
        println!("  Variables: {}", self.variable_count);
        println!("  Functions: {}", self.function_count);
        println!("  Est. Memory: {} bytes", self.estimated_memory_usage);
        println!("  Est. Exec Time: {} ms", self.estimated_execution_time_ms);

        if !self.errors.is_empty() {
            println!("  Errors:");
            for error in &self.errors {
                println!("    - {}", error);
            }
        }

        if !self.warnings.is_empty() {
            println!("  Warnings:");
            for warning in &self.warnings {
                println!("    - {}", warning);
            }
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Steel program validator with embedded constraints
pub struct SteelProgramValidator {
    max_program_size: usize,
    max_nesting_depth: usize,
    max_variables: usize,
    allowed_functions: HashSet<String>,
}

impl SteelProgramValidator {
    pub fn new() -> Self {
        let mut allowed_functions = HashSet::new();

        // Hardware control functions
        allowed_functions.insert("sleep".to_string());
        allowed_functions.insert("led-on".to_string());
        allowed_functions.insert("led-off".to_string());
        allowed_functions.insert("led-state".to_string());

        // System information functions
        allowed_functions.insert("device-info".to_string());
        allowed_functions.insert("memory-info".to_string());
        allowed_functions.insert("uptime".to_string());

        // Logging functions
        allowed_functions.insert("log".to_string());
        allowed_functions.insert("log-error".to_string());
        allowed_functions.insert("log-warn".to_string());
        allowed_functions.insert("log-info".to_string());
        allowed_functions.insert("log-debug".to_string());

        // Basic Scheme functions
        allowed_functions.insert("begin".to_string());
        allowed_functions.insert("if".to_string());
        allowed_functions.insert("cond".to_string());
        allowed_functions.insert("when".to_string());
        allowed_functions.insert("unless".to_string());
        allowed_functions.insert("let".to_string());
        allowed_functions.insert("let*".to_string());
        allowed_functions.insert("letrec".to_string());
        allowed_functions.insert("define".to_string());
        allowed_functions.insert("lambda".to_string());
        allowed_functions.insert("quote".to_string());
        allowed_functions.insert("quasiquote".to_string());
        allowed_functions.insert("unquote".to_string());

        // Arithmetic and comparison
        allowed_functions.insert("+".to_string());
        allowed_functions.insert("-".to_string());
        allowed_functions.insert("*".to_string());
        allowed_functions.insert("/".to_string());
        allowed_functions.insert("=".to_string());
        allowed_functions.insert("<".to_string());
        allowed_functions.insert(">".to_string());
        allowed_functions.insert("<=".to_string());
        allowed_functions.insert(">=".to_string());

        // Boolean operations
        allowed_functions.insert("and".to_string());
        allowed_functions.insert("or".to_string());
        allowed_functions.insert("not".to_string());

        // List operations (limited for embedded)
        allowed_functions.insert("list".to_string());
        allowed_functions.insert("car".to_string());
        allowed_functions.insert("cdr".to_string());
        allowed_functions.insert("cons".to_string());
        allowed_functions.insert("null?".to_string());
        allowed_functions.insert("pair?".to_string());

        // String operations (limited)
        allowed_functions.insert("string?".to_string());
        allowed_functions.insert("string-length".to_string());

        // Type predicates
        allowed_functions.insert("number?".to_string());
        allowed_functions.insert("boolean?".to_string());
        allowed_functions.insert("symbol?".to_string());

        // Embedded-specific utility functions
        allowed_functions.insert("repeat".to_string());
        allowed_functions.insert("blink-led".to_string());

        Self {
            max_program_size: MAX_PROGRAM_SIZE,
            max_nesting_depth: MAX_NESTING_DEPTH,
            max_variables: MAX_VARIABLES,
            allowed_functions,
        }
    }

    /// Validate a Steel program for embedded constraints
    pub fn validate(&self, program: &str) -> SystemResult<ValidationResult> {
        let mut result = ValidationResult::new();
        result.size_bytes = program.len();

        // Check program size
        if program.len() > self.max_program_size {
            result.add_error(format!(
                "Program too large: {} > {} bytes",
                program.len(),
                self.max_program_size
            ));
        }

        // Check if program is empty
        if program.trim().is_empty() {
            result.add_error("Program is empty".to_string());
            return Ok(result);
        }

        // Parse and validate structure
        match self.parse_and_validate(program, &mut result) {
            Ok(_) => {
                debug!("Steel program validation completed successfully");
            }
            Err(e) => {
                result.add_error(format!("Parse error: {}", e));
            }
        }

        // Estimate resource usage
        self.estimate_resource_usage(&mut result);

        // Add warnings for potential issues
        self.add_performance_warnings(&mut result);

        Ok(result)
    }

    /// Parse and validate Steel program structure
    fn parse_and_validate(&self, program: &str, result: &mut ValidationResult) -> SystemResult<()> {
        let mut depth: i32 = 0;
        let mut max_depth: i32 = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut variables = HashSet::new();
        let mut functions = HashSet::new();
        let mut current_token = String::new();
        let mut in_define = false;

        for ch in program.chars() {
            if escape_next {
                escape_next = false;
                current_token.push(ch);
                continue;
            }

            match ch {
                '\\' if in_string => {
                    escape_next = true;
                    current_token.push(ch);
                }
                '"' => {
                    in_string = !in_string;
                    current_token.push(ch);
                }
                '(' if !in_string => {
                    depth += 1;
                    max_depth = max_depth.max(depth);

                    if !current_token.is_empty() {
                        self.process_token(
                            &current_token,
                            &mut variables,
                            &mut functions,
                            &mut in_define,
                            result,
                        );
                        current_token.clear();
                    }
                }
                ')' if !in_string => {
                    depth -= 1;
                    if depth < 0 {
                        return Err(SystemError::Configuration(
                            "Unmatched closing parenthesis".to_string(),
                        ));
                    }

                    if !current_token.is_empty() {
                        self.process_token(
                            &current_token,
                            &mut variables,
                            &mut functions,
                            &mut in_define,
                            result,
                        );
                        current_token.clear();
                    }
                    in_define = false;
                }
                ' ' | '\t' | '\n' | '\r' if !in_string => {
                    if !current_token.is_empty() {
                        self.process_token(
                            &current_token,
                            &mut variables,
                            &mut functions,
                            &mut in_define,
                            result,
                        );
                        current_token.clear();
                    }
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if depth != 0 {
            return Err(SystemError::Configuration(
                "Unmatched parentheses".to_string(),
            ));
        }

        if in_string {
            return Err(SystemError::Configuration(
                "Unterminated string literal".to_string(),
            ));
        }

        // Process final token
        if !current_token.is_empty() {
            self.process_token(
                &current_token,
                &mut variables,
                &mut functions,
                &mut in_define,
                result,
            );
        }

        result.max_nesting_depth = max_depth as usize;
        result.variable_count = variables.len();
        result.function_count = functions.len();

        // Check constraints
        if max_depth > self.max_nesting_depth as i32 {
            result.add_error(format!(
                "Nesting too deep: {} > {}",
                max_depth, self.max_nesting_depth
            ));
        }

        if variables.len() > self.max_variables {
            result.add_error(format!(
                "Too many variables: {} > {}",
                variables.len(),
                self.max_variables
            ));
        }

        Ok(())
    }

    /// Process a token and update validation state
    fn process_token(
        &self,
        token: &str,
        variables: &mut HashSet<String>,
        functions: &mut HashSet<String>,
        in_define: &mut bool,
        result: &mut ValidationResult,
    ) {
        let token = token.trim();
        if token.is_empty() {
            return;
        }

        // Check for define statement
        if token == "define" {
            *in_define = true;
            return;
        }

        // If we're in a define, the next token is a variable or function name
        if *in_define && !token.starts_with('(') {
            variables.insert(token.to_string());
            return;
        }

        // Check if token is a function call
        if self.allowed_functions.contains(token) {
            functions.insert(token.to_string());
        } else if !token.starts_with('"') && // Not a string
                  !token.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-') && // Not a number
                  !matches!(token, "#t" | "#f" | "true" | "false") && // Not a boolean
                  !token.starts_with('#')
        {
            // Not a special form

            // Check if it's a potentially unsafe function
            if self.is_potentially_unsafe_function(token) {
                result.add_warning(format!("Potentially unsafe function: {}", token));
            } else {
                result.add_warning(format!("Unknown function: {}", token));
            }
        }
    }

    /// Check if a function is potentially unsafe for embedded systems
    fn is_potentially_unsafe_function(&self, token: &str) -> bool {
        let unsafe_functions = [
            "eval",
            "apply",
            "call/cc",
            "call-with-current-continuation",
            "load",
            "read",
            "write",
            "display",
            "newline",
            "file-exists?",
            "delete-file",
            "open-input-file",
            "open-output-file",
            "system",
            "exit",
            "abort",
            "error",
        ];

        unsafe_functions.contains(&token)
    }

    /// Estimate resource usage for the program
    fn estimate_resource_usage(&self, result: &mut ValidationResult) {
        // Rough estimates based on program characteristics
        let base_memory = 512; // Base Steel runtime overhead
        let per_variable_memory = 32; // Estimated memory per variable
        let per_function_memory = 64; // Estimated memory per function
        let per_nesting_memory = 16; // Estimated memory per nesting level

        result.estimated_memory_usage = base_memory
            + (result.variable_count * per_variable_memory)
            + (result.function_count * per_function_memory)
            + (result.max_nesting_depth * per_nesting_memory)
            + (result.size_bytes / 4); // Rough estimate for code storage

        // Estimate execution time based on complexity
        let base_time = 1; // Base execution time in ms
        let per_function_time = 1; // Time per function call
        let per_nesting_time = 1; // Time per nesting level

        result.estimated_execution_time_ms = base_time
            + (result.function_count as u64 * per_function_time)
            + (result.max_nesting_depth as u64 * per_nesting_time);
    }

    /// Add performance warnings based on program characteristics
    fn add_performance_warnings(&self, result: &mut ValidationResult) {
        // Warn about large programs
        if result.size_bytes > self.max_program_size / 2 {
            result.add_warning(format!(
                "Large program size: {} bytes (consider optimization)",
                result.size_bytes
            ));
        }

        // Warn about deep nesting
        if result.max_nesting_depth > self.max_nesting_depth / 2 {
            result.add_warning(format!(
                "Deep nesting: {} levels (may impact performance)",
                result.max_nesting_depth
            ));
        }

        // Warn about many variables
        if result.variable_count > self.max_variables / 2 {
            result.add_warning(format!(
                "Many variables: {} (consider reducing scope)",
                result.variable_count
            ));
        }

        // Warn about estimated memory usage
        #[cfg(target_arch = "riscv32")]
        {
            const MEMORY_WARNING_THRESHOLD: usize = 16 * 1024; // 16KB
            if result.estimated_memory_usage > MEMORY_WARNING_THRESHOLD {
                result.add_warning(format!(
                    "High estimated memory usage: {} bytes",
                    result.estimated_memory_usage
                ));
            }
        }

        // Warn about long execution time
        if result.estimated_execution_time_ms > 1000 {
            result.add_warning(format!(
                "Long estimated execution time: {} ms",
                result.estimated_execution_time_ms
            ));
        }
    }

    /// Optimize a Steel program for embedded constraints
    pub fn optimize(&self, program: &str) -> SystemResult<String> {
        let mut optimized = program.to_string();

        // Remove comments
        optimized = self.remove_comments(&optimized);

        // Normalize whitespace
        optimized = self.normalize_whitespace(&optimized);

        // Replace common patterns with optimized versions
        optimized = self.optimize_patterns(&optimized);

        Ok(optimized)
    }

    /// Remove comments from Steel program
    fn remove_comments(&self, program: &str) -> String {
        let mut result = String::new();
        let mut in_string = false;
        let mut escape_next = false;

        for line in program.lines() {
            let mut line_result = String::new();

            for ch in line.chars() {
                if escape_next {
                    escape_next = false;
                    line_result.push(ch);
                    continue;
                }

                match ch {
                    '\\' if in_string => {
                        escape_next = true;
                        line_result.push(ch);
                    }
                    '"' => {
                        in_string = !in_string;
                        line_result.push(ch);
                    }
                    ';' if !in_string => {
                        // Start of comment, ignore rest of line
                        break;
                    }
                    _ => {
                        line_result.push(ch);
                    }
                }
            }

            if !line_result.trim().is_empty() {
                result.push_str(&line_result);
                result.push('\n');
            }
        }

        result
    }

    /// Normalize whitespace in Steel program
    fn normalize_whitespace(&self, program: &str) -> String {
        let mut result = String::new();
        let mut in_string = false;
        let mut escape_next = false;
        let mut prev_was_space = false;

        for ch in program.chars() {
            if escape_next {
                escape_next = false;
                result.push(ch);
                prev_was_space = false;
                continue;
            }

            match ch {
                '\\' if in_string => {
                    escape_next = true;
                    result.push(ch);
                    prev_was_space = false;
                }
                '"' => {
                    in_string = !in_string;
                    result.push(ch);
                    prev_was_space = false;
                }
                ' ' | '\t' | '\n' | '\r' if !in_string => {
                    if !prev_was_space && !result.is_empty() {
                        result.push(' ');
                        prev_was_space = true;
                    }
                }
                _ => {
                    result.push(ch);
                    prev_was_space = false;
                }
            }
        }

        // Clean up spaces around parentheses but preserve spaces between expressions
        let mut cleaned = result.trim().to_string();
        cleaned = cleaned.replace("( ", "(");
        cleaned = cleaned.replace(" )", ")");
        
        // Fix double spaces
        while cleaned.contains("  ") {
            cleaned = cleaned.replace("  ", " ");
        }
        
        cleaned
    }

    /// Optimize common patterns in Steel programs
    fn optimize_patterns(&self, program: &str) -> String {
        let mut optimized = program.to_string();

        // Replace (begin (expr)) with (expr)
        optimized = optimized.replace("(begin (", "(");

        // TODO: Add actual optimizations here
        // For now, just return the original code
        // optimized = optimized.replace("(if condition #t #f)", "condition");

        // Other optimizations can be added here

        optimized
    }
}

impl Default for SteelProgramValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = SteelProgramValidator::new();
        assert!(validator.allowed_functions.contains("sleep"));
        assert!(validator.allowed_functions.contains("led-on"));
    }

    #[test]
    fn test_simple_program_validation() {
        let validator = SteelProgramValidator::new();
        let program = "(led-on)";

        let result = validator.validate(program).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.size_bytes, program.len());
        assert_eq!(result.max_nesting_depth, 1);
    }

    #[test]
    fn test_program_size_limit() {
        let validator = SteelProgramValidator::new();
        let large_program = "x".repeat(MAX_PROGRAM_SIZE + 1);

        let result = validator.validate(&large_program).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_nesting_depth() {
        let validator = SteelProgramValidator::new();
        let nested_program = "(((((led-on)))))";

        let result = validator.validate(nested_program).unwrap();
        assert_eq!(result.max_nesting_depth, 5);
    }

    #[test]
    fn test_comment_removal() {
        let validator = SteelProgramValidator::new();
        let program_with_comments = r#"
            ; This is a comment
            (led-on) ; Turn on LED
            ; Another comment
            (sleep 1)
        "#;

        let optimized = validator.optimize(program_with_comments).unwrap();
        assert!(!optimized.contains(';'));
        assert!(optimized.contains("(led-on)"));
        assert!(optimized.contains("(sleep 1)"));
    }

    #[test]
    fn test_whitespace_normalization() {
        let validator = SteelProgramValidator::new();
        let program_with_whitespace = "  (  led-on  )  \n\n  (  sleep   1  )  ";

        let optimized = validator.optimize(program_with_whitespace).unwrap();
        assert_eq!(optimized, "(led-on) (sleep 1)");
    }

    #[test]
    fn test_unknown_function_warning() {
        let validator = SteelProgramValidator::new();
        let program = "(unknown-function)";

        let result = validator.validate(program).unwrap();
        assert!(result.is_valid); // Still valid, just has warnings
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_unsafe_function_warning() {
        let validator = SteelProgramValidator::new();
        let program = "(eval '(+ 1 2))";

        let result = validator.validate(program).unwrap();
        assert!(!result.warnings.is_empty());
        assert!(result.warnings.iter().any(|w| w.contains("unsafe")));
    }
}
