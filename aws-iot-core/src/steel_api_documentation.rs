use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive Steel API documentation generator and reference
pub struct SteelAPIDocumentation {
    functions: HashMap<String, FunctionDocumentation>,
    categories: HashMap<String, CategoryDocumentation>,
    examples: HashMap<String, ExampleDocumentation>,
    tutorials: Vec<TutorialDocumentation>,
}

/// Documentation for a Steel API function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDocumentation {
    pub name: String,
    pub category: String,
    pub description: String,
    pub syntax: String,
    pub parameters: Vec<ParameterDocumentation>,
    pub return_type: String,
    pub return_description: String,
    pub examples: Vec<String>,
    pub notes: Vec<String>,
    pub see_also: Vec<String>,
    pub since_version: String,
    pub deprecated: Option<DeprecationInfo>,
    pub security_level: SecurityLevel,
}

/// Parameter documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDocumentation {
    pub name: String,
    pub type_name: String,
    pub description: String,
    pub optional: bool,
    pub default_value: Option<String>,
    pub constraints: Vec<String>,
}

/// Deprecation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationInfo {
    pub since_version: String,
    pub reason: String,
    pub replacement: Option<String>,
    pub removal_version: Option<String>,
}

/// Security level for functions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SecurityLevel {
    Safe,       // No security concerns
    Restricted, // Requires careful usage
    Privileged, // Requires admin approval
}

/// Category documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryDocumentation {
    pub name: String,
    pub description: String,
    pub overview: String,
    pub functions: Vec<String>,
    pub common_patterns: Vec<String>,
    pub best_practices: Vec<String>,
}

/// Example documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleDocumentation {
    pub title: String,
    pub description: String,
    pub code: String,
    pub expected_output: Option<String>,
    pub explanation: String,
    pub difficulty: DifficultyLevel,
    pub tags: Vec<String>,
}

/// Tutorial documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialDocumentation {
    pub title: String,
    pub description: String,
    pub difficulty: DifficultyLevel,
    pub estimated_time: String,
    pub prerequisites: Vec<String>,
    pub sections: Vec<TutorialSection>,
    pub exercises: Vec<Exercise>,
}

/// Tutorial section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialSection {
    pub title: String,
    pub content: String,
    pub code_examples: Vec<String>,
    pub key_points: Vec<String>,
}

/// Exercise for tutorials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    pub title: String,
    pub description: String,
    pub starter_code: Option<String>,
    pub solution: String,
    pub hints: Vec<String>,
}

/// Difficulty level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Documentation output format
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Markdown,
    Html,
    Json,
    PlainText,
}

impl Default for SteelAPIDocumentation {
    fn default() -> Self {
        Self::new()
    }
}

impl SteelAPIDocumentation {
    /// Create a new Steel API documentation generator
    pub fn new() -> Self {
        let mut documentation = Self {
            functions: HashMap::new(),
            categories: HashMap::new(),
            examples: HashMap::new(),
            tutorials: Vec::new(),
        };

        documentation.initialize_documentation();
        documentation
    }

    /// Initialize all documentation
    fn initialize_documentation(&mut self) {
        self.initialize_hardware_functions();
        self.initialize_system_functions();
        self.initialize_logging_functions();
        self.initialize_utility_functions();
        self.initialize_categories();
        self.initialize_examples();
        self.initialize_tutorials();
    }

    /// Initialize hardware control function documentation
    fn initialize_hardware_functions(&mut self) {
        // Sleep function
        self.functions.insert(
            "sleep".to_string(),
            FunctionDocumentation {
                name: "sleep".to_string(),
                category: "Hardware Control".to_string(),
                description: "Pauses program execution for the specified duration in seconds."
                    .to_string(),
                syntax: "(sleep duration)".to_string(),
                parameters: vec![ParameterDocumentation {
                    name: "duration".to_string(),
                    type_name: "number".to_string(),
                    description: "Duration to sleep in seconds (can be fractional)".to_string(),
                    optional: false,
                    default_value: None,
                    constraints: vec![
                        "Must be positive".to_string(),
                        "Maximum 3600 seconds (1 hour)".to_string(),
                    ],
                }],
                return_type: "boolean".to_string(),
                return_description: "Returns #t on successful completion".to_string(),
                examples: vec![
                    "(sleep 1)      ; Sleep for 1 second".to_string(),
                    "(sleep 0.5)    ; Sleep for 500 milliseconds".to_string(),
                    "(sleep 2.5)    ; Sleep for 2.5 seconds".to_string(),
                ],
                notes: vec![
                    "Sleep duration is approximate and may vary based on system load".to_string(),
                    "During sleep, the program cannot respond to interrupts".to_string(),
                    "Use shorter sleep durations for more responsive programs".to_string(),
                ],
                see_also: vec!["led-on".to_string(), "led-off".to_string()],
                since_version: "1.0.0".to_string(),
                deprecated: None,
                security_level: SecurityLevel::Safe,
            },
        );

        // LED on function
        self.functions.insert(
            "led-on".to_string(),
            FunctionDocumentation {
                name: "led-on".to_string(),
                category: "Hardware Control".to_string(),
                description: "Turns the LED on.".to_string(),
                syntax: "(led-on)".to_string(),
                parameters: vec![],
                return_type: "boolean".to_string(),
                return_description: "Returns #t if LED was successfully turned on".to_string(),
                examples: vec![
                    "(led-on)                    ; Turn LED on".to_string(),
                    "(if (led-on) \"Success\" \"Failed\")  ; Check result".to_string(),
                ],
                notes: vec![
                    "LED state persists until explicitly changed".to_string(),
                    "Multiple calls to led-on have no additional effect".to_string(),
                ],
                see_also: vec!["led-off".to_string(), "led-state".to_string()],
                since_version: "1.0.0".to_string(),
                deprecated: None,
                security_level: SecurityLevel::Safe,
            },
        );

        // LED off function
        self.functions.insert(
            "led-off".to_string(),
            FunctionDocumentation {
                name: "led-off".to_string(),
                category: "Hardware Control".to_string(),
                description: "Turns the LED off.".to_string(),
                syntax: "(led-off)".to_string(),
                parameters: vec![],
                return_type: "boolean".to_string(),
                return_description: "Returns #f when LED is successfully turned off".to_string(),
                examples: vec![
                    "(led-off)                   ; Turn LED off".to_string(),
                    "(unless (led-off) (log \"error\" \"LED failed\"))".to_string(),
                ],
                notes: vec![
                    "LED state persists until explicitly changed".to_string(),
                    "Multiple calls to led-off have no additional effect".to_string(),
                ],
                see_also: vec!["led-on".to_string(), "led-state".to_string()],
                since_version: "1.0.0".to_string(),
                deprecated: None,
                security_level: SecurityLevel::Safe,
            },
        );

        // LED state function
        self.functions.insert(
            "led-state".to_string(),
            FunctionDocumentation {
                name: "led-state".to_string(),
                category: "Hardware Control".to_string(),
                description: "Returns the current state of the LED.".to_string(),
                syntax: "(led-state)".to_string(),
                parameters: vec![],
                return_type: "boolean".to_string(),
                return_description: "Returns #t if LED is on, #f if LED is off".to_string(),
                examples: vec![
                    "(led-state)                 ; Get current LED state".to_string(),
                    "(if (led-state) (led-off) (led-on))  ; Toggle LED".to_string(),
                    "(log \"info\" (if (led-state) \"LED is on\" \"LED is off\"))".to_string(),
                ],
                notes: vec![
                    "This function queries the actual hardware state".to_string(),
                    "State is updated immediately after led-on or led-off calls".to_string(),
                ],
                see_also: vec!["led-on".to_string(), "led-off".to_string()],
                since_version: "1.0.0".to_string(),
                deprecated: None,
                security_level: SecurityLevel::Safe,
            },
        );
    }

    /// Initialize system information function documentation
    fn initialize_system_functions(&mut self) {
        // Device info function
        self.functions.insert(
            "device-info".to_string(),
            FunctionDocumentation {
                name: "device-info".to_string(),
                category: "System Information".to_string(),
                description:
                    "Returns comprehensive device information as a list of key-value strings."
                        .to_string(),
                syntax: "(device-info)".to_string(),
                parameters: vec![],
                return_type: "list".to_string(),
                return_description: "List of strings containing device information".to_string(),
                examples: vec![
                    "(device-info)               ; Get all device info".to_string(),
                    "(let ((info (device-info))) (display info))".to_string(),
                    "(map display (device-info)) ; Print each info item".to_string(),
                ],
                notes: vec![
                    "Information includes device ID, platform, version, and hardware details"
                        .to_string(),
                    "Format: \"key: value\" strings for easy parsing".to_string(),
                    "Available information may vary by platform".to_string(),
                ],
                see_also: vec!["memory-info".to_string(), "uptime".to_string()],
                since_version: "1.0.0".to_string(),
                deprecated: None,
                security_level: SecurityLevel::Safe,
            },
        );

        // Memory info function
        self.functions.insert(
            "memory-info".to_string(),
            FunctionDocumentation {
                name: "memory-info".to_string(),
                category: "System Information".to_string(),
                description:
                    "Returns current memory usage information as a list of key-value strings."
                        .to_string(),
                syntax: "(memory-info)".to_string(),
                parameters: vec![],
                return_type: "list".to_string(),
                return_description: "List of strings containing memory statistics".to_string(),
                examples: vec![
                    "(memory-info)               ; Get memory statistics".to_string(),
                    "(let ((mem (memory-info))) (log \"info\" (car mem)))".to_string(),
                ],
                notes: vec![
                    "Includes total, free, used memory and largest free block".to_string(),
                    "Memory values are in bytes".to_string(),
                    "Usage percentage is calculated automatically".to_string(),
                ],
                see_also: vec!["device-info".to_string(), "uptime".to_string()],
                since_version: "1.0.0".to_string(),
                deprecated: None,
                security_level: SecurityLevel::Safe,
            },
        );

        // Uptime function
        self.functions.insert(
            "uptime".to_string(),
            FunctionDocumentation {
                name: "uptime".to_string(),
                category: "System Information".to_string(),
                description: "Returns the system uptime in seconds since boot.".to_string(),
                syntax: "(uptime)".to_string(),
                parameters: vec![],
                return_type: "number".to_string(),
                return_description: "Uptime in seconds as a floating-point number".to_string(),
                examples: vec![
                    "(uptime)                    ; Get uptime in seconds".to_string(),
                    "(/ (uptime) 60)             ; Convert to minutes".to_string(),
                    "(log \"info\" (format \"Uptime: ~a seconds\" (uptime)))".to_string(),
                ],
                notes: vec![
                    "Precision depends on system clock resolution".to_string(),
                    "Value resets to 0 on system restart".to_string(),
                    "Includes time spent in sleep mode".to_string(),
                ],
                see_also: vec!["device-info".to_string(), "memory-info".to_string()],
                since_version: "1.0.0".to_string(),
                deprecated: None,
                security_level: SecurityLevel::Safe,
            },
        );
    }

    /// Initialize logging function documentation
    fn initialize_logging_functions(&mut self) {
        // Log function
        self.functions.insert(
            "log".to_string(),
            FunctionDocumentation {
                name: "log".to_string(),
                category: "Logging".to_string(),
                description: "Logs a message with the specified level to the system log."
                    .to_string(),
                syntax: "(log level message)".to_string(),
                parameters: vec![
                    ParameterDocumentation {
                        name: "level".to_string(),
                        type_name: "string".to_string(),
                        description:
                            "Log level: \"error\", \"warn\", \"info\", \"debug\", or \"trace\""
                                .to_string(),
                        optional: false,
                        default_value: None,
                        constraints: vec![
                            "Must be a valid log level string".to_string(),
                            "Case insensitive".to_string(),
                        ],
                    },
                    ParameterDocumentation {
                        name: "message".to_string(),
                        type_name: "string".to_string(),
                        description: "The message to log".to_string(),
                        optional: false,
                        default_value: None,
                        constraints: vec!["Maximum 1024 characters".to_string()],
                    },
                ],
                return_type: "boolean".to_string(),
                return_description: "Returns #t on successful logging".to_string(),
                examples: vec![
                    "(log \"info\" \"Program started\")".to_string(),
                    "(log \"error\" \"Something went wrong\")".to_string(),
                    "(log \"debug\" (format \"Value: ~a\" x))".to_string(),
                ],
                notes: vec![
                    "Log messages include timestamp and program context".to_string(),
                    "Log level filtering may be configured system-wide".to_string(),
                    "Excessive logging may impact performance".to_string(),
                ],
                see_also: vec!["log-error".to_string(), "log-info".to_string()],
                since_version: "1.0.0".to_string(),
                deprecated: None,
                security_level: SecurityLevel::Safe,
            },
        );

        // Convenience logging functions
        let log_levels = vec![
            ("log-error", "error", "Log an error message"),
            ("log-warn", "warn", "Log a warning message"),
            ("log-info", "info", "Log an informational message"),
            ("log-debug", "debug", "Log a debug message"),
        ];

        for (func_name, level, description) in log_levels {
            self.functions.insert(
                func_name.to_string(),
                FunctionDocumentation {
                    name: func_name.to_string(),
                    category: "Logging".to_string(),
                    description: description.to_string(),
                    syntax: format!("({} message)", func_name),
                    parameters: vec![ParameterDocumentation {
                        name: "message".to_string(),
                        type_name: "string".to_string(),
                        description: "The message to log".to_string(),
                        optional: false,
                        default_value: None,
                        constraints: vec!["Maximum 1024 characters".to_string()],
                    }],
                    return_type: "boolean".to_string(),
                    return_description: "Returns #t on successful logging".to_string(),
                    examples: vec![
                        format!("({} \"This is a {} message\")", func_name, level),
                        format!("({} (format \"Value: ~a\" x))", func_name),
                    ],
                    notes: vec![
                        format!("Equivalent to (log \"{}\" message)", level),
                        "Convenience function for common log levels".to_string(),
                    ],
                    see_also: vec!["log".to_string()],
                    since_version: "1.0.0".to_string(),
                    deprecated: None,
                    security_level: SecurityLevel::Safe,
                },
            );
        }
    }

    /// Initialize utility function documentation
    fn initialize_utility_functions(&mut self) {
        // Format function (Steel built-in, but documented for completeness)
        self.functions.insert(
            "format".to_string(),
            FunctionDocumentation {
                name: "format".to_string(),
                category: "Utilities".to_string(),
                description: "Formats a string with placeholders replaced by values.".to_string(),
                syntax: "(format format-string . args)".to_string(),
                parameters: vec![
                    ParameterDocumentation {
                        name: "format-string".to_string(),
                        type_name: "string".to_string(),
                        description: "Format string with ~a placeholders".to_string(),
                        optional: false,
                        default_value: None,
                        constraints: vec![],
                    },
                    ParameterDocumentation {
                        name: "args".to_string(),
                        type_name: "any".to_string(),
                        description: "Values to substitute for placeholders".to_string(),
                        optional: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ],
                return_type: "string".to_string(),
                return_description: "Formatted string with substitutions".to_string(),
                examples: vec![
                    "(format \"Hello, ~a!\" \"World\")".to_string(),
                    "(format \"Value: ~a, Count: ~a\" x 42)".to_string(),
                    "(format \"LED is ~a\" (if (led-state) \"on\" \"off\"))".to_string(),
                ],
                notes: vec![
                    "~a is the universal placeholder for any value".to_string(),
                    "Number of placeholders should match number of arguments".to_string(),
                    "Useful for creating dynamic log messages".to_string(),
                ],
                see_also: vec!["log".to_string(), "display".to_string()],
                since_version: "1.0.0".to_string(),
                deprecated: None,
                security_level: SecurityLevel::Safe,
            },
        );
    }

    /// Initialize category documentation
    fn initialize_categories(&mut self) {
        self.categories.insert("Hardware Control".to_string(), CategoryDocumentation {
            name: "Hardware Control".to_string(),
            description: "Functions for controlling physical hardware components".to_string(),
            overview: "The Hardware Control category provides functions to interact with the physical components of your IoT device, including LEDs, sensors, and timing controls.".to_string(),
            functions: vec!["sleep".to_string(), "led-on".to_string(), "led-off".to_string(), "led-state".to_string()],
            common_patterns: vec![
                "LED blinking: (led-on) (sleep 1) (led-off) (sleep 1)".to_string(),
                "State checking: (if (led-state) (led-off) (led-on))".to_string(),
                "Timed sequences: Use sleep between hardware operations".to_string(),
            ],
            best_practices: vec![
                "Always check return values for error handling".to_string(),
                "Use appropriate sleep durations to avoid excessive power consumption".to_string(),
                "Consider hardware limitations when designing timing sequences".to_string(),
            ],
        });

        self.categories.insert("System Information".to_string(), CategoryDocumentation {
            name: "System Information".to_string(),
            description: "Functions for retrieving system and device information".to_string(),
            overview: "System Information functions provide access to device status, memory usage, and runtime information for monitoring and diagnostics.".to_string(),
            functions: vec!["device-info".to_string(), "memory-info".to_string(), "uptime".to_string()],
            common_patterns: vec![
                "Health monitoring: Regularly check memory-info and uptime".to_string(),
                "Device identification: Use device-info for unique identification".to_string(),
                "Resource monitoring: Track memory usage over time".to_string(),
            ],
            best_practices: vec![
                "Cache device-info results as they rarely change".to_string(),
                "Monitor memory usage to prevent out-of-memory conditions".to_string(),
                "Use uptime for calculating intervals and scheduling".to_string(),
            ],
        });

        self.categories.insert("Logging".to_string(), CategoryDocumentation {
            name: "Logging".to_string(),
            description: "Functions for logging messages and debugging information".to_string(),
            overview: "Logging functions provide structured ways to record program execution, errors, and diagnostic information for debugging and monitoring.".to_string(),
            functions: vec!["log".to_string(), "log-error".to_string(), "log-warn".to_string(), "log-info".to_string(), "log-debug".to_string()],
            common_patterns: vec![
                "Error handling: (log-error \"Operation failed\")".to_string(),
                "Progress tracking: (log-info \"Step completed\")".to_string(),
                "Debug information: (log-debug (format \"Value: ~a\" x))".to_string(),
            ],
            best_practices: vec![
                "Use appropriate log levels for different types of messages".to_string(),
                "Include context information in log messages".to_string(),
                "Avoid excessive logging in production code".to_string(),
                "Use format strings for dynamic log messages".to_string(),
            ],
        });
    }

    /// Initialize example documentation
    fn initialize_examples(&mut self) {
        self.examples.insert("basic-led-blink".to_string(), ExampleDocumentation {
            title: "Basic LED Blinking".to_string(),
            description: "Simple example that blinks an LED on and off repeatedly".to_string(),
            code: r#"
;; Basic LED blinking program
(define (blink-led times)
  (if (> times 0)
      (begin
        (led-on)
        (sleep 0.5)
        (led-off)
        (sleep 0.5)
        (blink-led (- times 1)))
      (log-info "Blinking completed")))

;; Blink the LED 10 times
(blink-led 10)
"#.to_string(),
            expected_output: Some("LED blinks 10 times, then logs completion message".to_string()),
            explanation: "This example demonstrates basic LED control and timing. The blink-led function recursively calls itself, decrementing the counter each time until it reaches zero.".to_string(),
            difficulty: DifficultyLevel::Beginner,
            tags: vec!["led".to_string(), "sleep".to_string(), "recursion".to_string()],
        });

        self.examples.insert("system-monitor".to_string(), ExampleDocumentation {
            title: "System Monitoring".to_string(),
            description: "Monitor system resources and log status information".to_string(),
            code: r#"
;; System monitoring program
(define (monitor-system)
  (let ((device (device-info))
        (memory (memory-info))
        (up (uptime)))
    (log-info "=== System Status ===")
    (log-info (format "Uptime: ~a seconds" up))
    (map (lambda (info) (log-info info)) device)
    (map (lambda (mem) (log-info mem)) memory)
    (log-info "=====================")))

;; Monitor system every 30 seconds
(define (monitoring-loop count)
  (if (> count 0)
      (begin
        (monitor-system)
        (sleep 30)
        (monitoring-loop (- count 1)))
      (log-info "Monitoring completed")))

;; Run monitoring for 5 cycles
(monitoring-loop 5)
"#.to_string(),
            expected_output: Some("System information logged every 30 seconds for 5 cycles".to_string()),
            explanation: "This example shows how to gather and log system information. It demonstrates using multiple system functions together and creating a monitoring loop.".to_string(),
            difficulty: DifficultyLevel::Intermediate,
            tags: vec!["monitoring".to_string(), "system-info".to_string(), "logging".to_string()],
        });

        self.examples.insert("led-patterns".to_string(), ExampleDocumentation {
            title: "LED Pattern Generator".to_string(),
            description: "Create complex LED blinking patterns using lists and higher-order functions".to_string(),
            code: r#"
;; LED pattern generator
(define (execute-pattern pattern)
  (map (lambda (step)
         (let ((action (car step))
               (duration (cadr step)))
           (cond
             [(eq? action 'on) (led-on)]
             [(eq? action 'off) (led-off)]
             [else (log-warn "Unknown action")])
           (sleep duration)))
       pattern))

;; Define some patterns
(define morse-sos
  '((on 0.2) (off 0.2)   ; S
    (on 0.2) (off 0.2)
    (on 0.2) (off 0.4)
    (on 0.6) (off 0.2)   ; O
    (on 0.6) (off 0.2)
    (on 0.6) (off 0.4)
    (on 0.2) (off 0.2)   ; S
    (on 0.2) (off 0.2)
    (on 0.2) (off 1.0)))

(define heartbeat
  '((on 0.1) (off 0.1)
    (on 0.1) (off 0.5)))

;; Execute patterns
(log-info "Sending SOS signal")
(execute-pattern morse-sos)
(sleep 2)
(log-info "Heartbeat pattern")
(execute-pattern heartbeat)
"#.to_string(),
            expected_output: Some("LED displays SOS morse code pattern followed by heartbeat pattern".to_string()),
            explanation: "This advanced example demonstrates using lists to define LED patterns and higher-order functions to execute them. It shows how to create reusable pattern definitions.".to_string(),
            difficulty: DifficultyLevel::Advanced,
            tags: vec!["led".to_string(), "patterns".to_string(), "lists".to_string(), "higher-order".to_string()],
        });
    }

    /// Initialize tutorial documentation
    fn initialize_tutorials(&mut self) {
        self.tutorials.push(TutorialDocumentation {
            title: "Getting Started with Steel IoT Programming".to_string(),
            description: "Learn the basics of programming IoT devices using Steel".to_string(),
            difficulty: DifficultyLevel::Beginner,
            estimated_time: "30 minutes".to_string(),
            prerequisites: vec![
                "Basic understanding of programming concepts".to_string(),
                "Familiarity with Lisp/Scheme syntax (helpful but not required)".to_string(),
            ],
            sections: vec![
                TutorialSection {
                    title: "Introduction to Steel".to_string(),
                    content: "Steel is a Scheme-based scripting language designed for IoT devices. It provides a simple, functional programming interface to hardware components while maintaining safety and security.".to_string(),
                    code_examples: vec![
                        ";; Your first Steel program\n(log-info \"Hello, IoT World!\")".to_string(),
                    ],
                    key_points: vec![
                        "Steel uses Scheme syntax with parentheses".to_string(),
                        "Functions are called with (function-name arguments)".to_string(),
                        "Steel is designed for embedded systems".to_string(),
                    ],
                },
                TutorialSection {
                    title: "Basic Hardware Control".to_string(),
                    content: "Learn to control LEDs and use timing functions to create simple programs.".to_string(),
                    code_examples: vec![
                        ";; Turn LED on and off\n(led-on)\n(sleep 1)\n(led-off)".to_string(),
                        ";; Check LED state\n(if (led-state)\n    (log-info \"LED is on\")\n    (log-info \"LED is off\"))".to_string(),
                    ],
                    key_points: vec![
                        "led-on and led-off control the LED".to_string(),
                        "sleep pauses execution for specified seconds".to_string(),
                        "led-state returns the current LED state".to_string(),
                    ],
                },
                TutorialSection {
                    title: "System Information and Logging".to_string(),
                    content: "Access device information and create informative log messages.".to_string(),
                    code_examples: vec![
                        ";; Get and log device information\n(let ((info (device-info)))\n  (map (lambda (item) (log-info item)) info))".to_string(),
                        ";; Monitor memory usage\n(let ((mem (memory-info)))\n  (log-info (format \"Memory status: ~a\" (car mem))))".to_string(),
                    ],
                    key_points: vec![
                        "device-info returns system information".to_string(),
                        "memory-info shows memory usage".to_string(),
                        "Use format for dynamic log messages".to_string(),
                    ],
                },
            ],
            exercises: vec![
                Exercise {
                    title: "LED Blink Counter".to_string(),
                    description: "Create a program that blinks the LED a specified number of times and logs the count".to_string(),
                    starter_code: Some(";; Complete this function\n(define (blink-counter n)\n  ;; Your code here\n  )".to_string()),
                    solution: "(define (blink-counter n)\n  (if (> n 0)\n      (begin\n        (log-info (format \"Blink ~a\" n))\n        (led-on)\n        (sleep 0.5)\n        (led-off)\n        (sleep 0.5)\n        (blink-counter (- n 1)))\n      (log-info \"Blinking complete\")))".to_string(),
                    hints: vec![
                        "Use recursion to count down".to_string(),
                        "Log the current count before each blink".to_string(),
                        "Don't forget the base case when n reaches 0".to_string(),
                    ],
                },
            ],
        });

        self.tutorials.push(TutorialDocumentation {
            title: "Advanced Steel Programming Patterns".to_string(),
            description: "Learn advanced techniques for creating robust IoT applications".to_string(),
            difficulty: DifficultyLevel::Advanced,
            estimated_time: "60 minutes".to_string(),
            prerequisites: vec![
                "Completion of 'Getting Started' tutorial".to_string(),
                "Understanding of functional programming concepts".to_string(),
                "Experience with lists and higher-order functions".to_string(),
            ],
            sections: vec![
                TutorialSection {
                    title: "Error Handling and Robustness".to_string(),
                    content: "Learn to write robust programs that handle errors gracefully and provide meaningful feedback.".to_string(),
                    code_examples: vec![
                        ";; Safe LED operation with error checking\n(define (safe-led-on)\n  (if (led-on)\n      (log-info \"LED turned on successfully\")\n      (log-error \"Failed to turn on LED\")))".to_string(),
                    ],
                    key_points: vec![
                        "Always check return values from hardware functions".to_string(),
                        "Use appropriate log levels for different situations".to_string(),
                        "Implement fallback behavior for failures".to_string(),
                    ],
                },
                TutorialSection {
                    title: "State Machines and Complex Logic".to_string(),
                    content: "Implement state machines for complex device behavior and decision making.".to_string(),
                    code_examples: vec![
                        ";; Simple state machine example\n(define current-state 'idle)\n\n(define (state-machine)\n  (cond\n    [(eq? current-state 'idle)\n     (led-off)\n     (set! current-state 'waiting)]\n    [(eq? current-state 'waiting)\n     (led-on)\n     (set! current-state 'active)]\n    [else\n     (led-off)\n     (set! current-state 'idle)]))".to_string(),
                    ],
                    key_points: vec![
                        "Use state variables to track program state".to_string(),
                        "Implement clear state transitions".to_string(),
                        "Consider all possible states and transitions".to_string(),
                    ],
                },
            ],
            exercises: vec![
                Exercise {
                    title: "Traffic Light Controller".to_string(),
                    description: "Implement a traffic light state machine using the LED".to_string(),
                    starter_code: Some(";; Traffic light states: red, yellow, green\n;; Use LED on/off and different timing for each state\n(define traffic-state 'red)\n\n(define (traffic-light-cycle)\n  ;; Your implementation here\n  )".to_string()),
                    solution: "(define traffic-state 'red)\n\n(define (traffic-light-cycle)\n  (cond\n    [(eq? traffic-state 'red)\n     (led-on)\n     (log-info \"RED - Stop\")\n     (sleep 3)\n     (set! traffic-state 'green)]\n    [(eq? traffic-state 'green)\n     (led-off)\n     (log-info \"GREEN - Go\")\n     (sleep 2)\n     (set! traffic-state 'yellow)]\n    [(eq? traffic-state 'yellow)\n     (led-on)\n     (log-info \"YELLOW - Caution\")\n     (sleep 1)\n     (set! traffic-state 'red)]))\n\n;; Run the cycle\n(define (run-traffic-light cycles)\n  (if (> cycles 0)\n      (begin\n        (traffic-light-cycle)\n        (run-traffic-light (- cycles 1)))\n      (log-info \"Traffic light stopped\")))".to_string(),
                    hints: vec![
                        "Use different LED states and timing for each traffic light color".to_string(),
                        "Implement proper state transitions".to_string(),
                        "Add logging to show current state".to_string(),
                    ],
                },
            ],
        });
    }

    // ========== Public API Methods ==========

    /// Generate documentation in the specified format
    pub fn generate_documentation(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Markdown => self.generate_markdown(),
            OutputFormat::Html => self.generate_html(),
            OutputFormat::Json => self.generate_json(),
            OutputFormat::PlainText => self.generate_plain_text(),
        }
    }

    /// Generate Markdown documentation
    fn generate_markdown(&self) -> String {
        let mut markdown = String::new();

        // Title and introduction
        markdown.push_str("# Steel IoT API Documentation\n\n");
        markdown.push_str("This document provides comprehensive documentation for the Steel IoT programming API.\n\n");

        // Table of contents
        markdown.push_str("## Table of Contents\n\n");
        markdown.push_str("- [Function Reference](#function-reference)\n");
        markdown.push_str("- [Categories](#categories)\n");
        markdown.push_str("- [Examples](#examples)\n");
        markdown.push_str("- [Tutorials](#tutorials)\n\n");

        // Function reference
        markdown.push_str("## Function Reference\n\n");

        let mut functions: Vec<_> = self.functions.values().collect();
        functions.sort_by(|a, b| a.name.cmp(&b.name));

        for func in functions {
            markdown.push_str(&format!("### {}\n\n", func.name));
            markdown.push_str(&format!("**Category:** {}\n\n", func.category));
            markdown.push_str(&format!("{}\n\n", func.description));

            markdown.push_str("**Syntax:**\n```scheme\n");
            markdown.push_str(&format!("{}\n", func.syntax));
            markdown.push_str("```\n\n");

            if !func.parameters.is_empty() {
                markdown.push_str("**Parameters:**\n");
                for param in &func.parameters {
                    markdown.push_str(&format!(
                        "- `{}` ({}): {}",
                        param.name, param.type_name, param.description
                    ));
                    if param.optional {
                        markdown.push_str(" (optional)");
                    }
                    markdown.push('\n');
                }
                markdown.push('\n');
            }

            markdown.push_str(&format!(
                "**Returns:** {} - {}\n\n",
                func.return_type, func.return_description
            ));

            if !func.examples.is_empty() {
                markdown.push_str("**Examples:**\n```scheme\n");
                for example in &func.examples {
                    markdown.push_str(&format!("{}\n", example));
                }
                markdown.push_str("```\n\n");
            }

            if !func.notes.is_empty() {
                markdown.push_str("**Notes:**\n");
                for note in &func.notes {
                    markdown.push_str(&format!("- {}\n", note));
                }
                markdown.push('\n');
            }

            if let Some(deprecation) = &func.deprecated {
                markdown.push_str(&format!(
                    "**⚠️ Deprecated since version {}:** {}\n",
                    deprecation.since_version, deprecation.reason
                ));
                if let Some(replacement) = &deprecation.replacement {
                    markdown.push_str(&format!("Use `{}` instead.\n", replacement));
                }
                markdown.push('\n');
            }

            markdown.push_str("---\n\n");
        }

        // Categories
        markdown.push_str("## Categories\n\n");
        for category in self.categories.values() {
            markdown.push_str(&format!("### {}\n\n", category.name));
            markdown.push_str(&format!("{}\n\n", category.description));
            markdown.push_str(&format!("{}\n\n", category.overview));

            if !category.common_patterns.is_empty() {
                markdown.push_str("**Common Patterns:**\n");
                for pattern in &category.common_patterns {
                    markdown.push_str(&format!("- {}\n", pattern));
                }
                markdown.push('\n');
            }

            if !category.best_practices.is_empty() {
                markdown.push_str("**Best Practices:**\n");
                for practice in &category.best_practices {
                    markdown.push_str(&format!("- {}\n", practice));
                }
                markdown.push('\n');
            }
        }

        // Examples
        markdown.push_str("## Examples\n\n");
        for example in self.examples.values() {
            markdown.push_str(&format!("### {}\n\n", example.title));
            markdown.push_str(&format!("**Difficulty:** {:?}\n\n", example.difficulty));
            markdown.push_str(&format!("{}\n\n", example.description));

            markdown.push_str("```scheme\n");
            markdown.push_str(&example.code);
            markdown.push_str("\n```\n\n");

            if let Some(output) = &example.expected_output {
                markdown.push_str(&format!("**Expected Output:** {}\n\n", output));
            }

            markdown.push_str(&format!("**Explanation:** {}\n\n", example.explanation));
        }

        // Tutorials
        markdown.push_str("## Tutorials\n\n");
        for tutorial in &self.tutorials {
            markdown.push_str(&format!("### {}\n\n", tutorial.title));
            markdown.push_str(&format!(
                "**Difficulty:** {:?} | **Time:** {}\n\n",
                tutorial.difficulty, tutorial.estimated_time
            ));
            markdown.push_str(&format!("{}\n\n", tutorial.description));

            if !tutorial.prerequisites.is_empty() {
                markdown.push_str("**Prerequisites:**\n");
                for prereq in &tutorial.prerequisites {
                    markdown.push_str(&format!("- {}\n", prereq));
                }
                markdown.push('\n');
            }

            for section in &tutorial.sections {
                markdown.push_str(&format!("#### {}\n\n", section.title));
                markdown.push_str(&format!("{}\n\n", section.content));

                if !section.code_examples.is_empty() {
                    for code in &section.code_examples {
                        markdown.push_str("```scheme\n");
                        markdown.push_str(code);
                        markdown.push_str("\n```\n\n");
                    }
                }
            }
        }

        markdown
    }

    /// Generate HTML documentation
    fn generate_html(&self) -> String {
        // This would generate a full HTML document
        // For brevity, returning a simple HTML structure
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Steel IoT API Documentation</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1, h2, h3 {{ color: #333; }}
        code {{ background-color: #f4f4f4; padding: 2px 4px; }}
        pre {{ background-color: #f4f4f4; padding: 10px; overflow-x: auto; }}
    </style>
</head>
<body>
    <h1>Steel IoT API Documentation</h1>
    <p>This is the HTML version of the Steel IoT API documentation.</p>
    <p>Functions documented: {}</p>
    <p>Categories: {}</p>
    <p>Examples: {}</p>
    <p>Tutorials: {}</p>
</body>
</html>"#,
            self.functions.len(),
            self.categories.len(),
            self.examples.len(),
            self.tutorials.len()
        )
    }

    /// Generate JSON documentation
    fn generate_json(&self) -> String {
        let doc_data = serde_json::json!({
            "functions": self.functions,
            "categories": self.categories,
            "examples": self.examples,
            "tutorials": self.tutorials,
            "metadata": {
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "version": "1.0.0",
                "total_functions": self.functions.len(),
                "total_categories": self.categories.len(),
                "total_examples": self.examples.len(),
                "total_tutorials": self.tutorials.len()
            }
        });

        serde_json::to_string_pretty(&doc_data).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate plain text documentation
    fn generate_plain_text(&self) -> String {
        let mut text = String::new();

        text.push_str("STEEL IOT API DOCUMENTATION\n");
        text.push_str("===========================\n\n");

        text.push_str("FUNCTION REFERENCE\n");
        text.push_str("------------------\n\n");

        let mut functions: Vec<_> = self.functions.values().collect();
        functions.sort_by(|a, b| a.name.cmp(&b.name));

        for func in functions {
            text.push_str(&format!("{}\n", func.name.to_uppercase()));
            text.push_str(&format!("Category: {}\n", func.category));
            text.push_str(&format!("Syntax: {}\n", func.syntax));
            text.push_str(&format!("Description: {}\n", func.description));
            text.push_str(&format!("Returns: {}\n", func.return_description));
            text.push('\n');
        }

        text.push_str(&format!(
            "\nTotal functions documented: {}\n",
            self.functions.len()
        ));
        text.push_str(&format!("Total categories: {}\n", self.categories.len()));
        text.push_str(&format!("Total examples: {}\n", self.examples.len()));
        text.push_str(&format!("Total tutorials: {}\n", self.tutorials.len()));

        text
    }

    /// Get function documentation by name
    pub fn get_function_doc(&self, name: &str) -> Option<&FunctionDocumentation> {
        self.functions.get(name)
    }

    /// Get category documentation by name
    pub fn get_category_doc(&self, name: &str) -> Option<&CategoryDocumentation> {
        self.categories.get(name)
    }

    /// Get example documentation by name
    pub fn get_example_doc(&self, name: &str) -> Option<&ExampleDocumentation> {
        self.examples.get(name)
    }

    /// List all function names
    pub fn list_functions(&self) -> Vec<String> {
        let mut names: Vec<String> = self.functions.keys().cloned().collect();
        names.sort();
        names
    }

    /// List all category names
    pub fn list_categories(&self) -> Vec<String> {
        let mut names: Vec<String> = self.categories.keys().cloned().collect();
        names.sort();
        names
    }

    /// List functions by category
    pub fn list_functions_by_category(&self, category: &str) -> Vec<String> {
        self.functions
            .values()
            .filter(|func| func.category == category)
            .map(|func| func.name.clone())
            .collect()
    }

    /// Search functions by keyword
    pub fn search_functions(&self, keyword: &str) -> Vec<String> {
        let keyword_lower = keyword.to_lowercase();
        self.functions
            .values()
            .filter(|func| {
                func.name.to_lowercase().contains(&keyword_lower)
                    || func.description.to_lowercase().contains(&keyword_lower)
                    || func.category.to_lowercase().contains(&keyword_lower)
            })
            .map(|func| func.name.clone())
            .collect()
    }

    /// Get documentation statistics
    pub fn get_statistics(&self) -> DocumentationStatistics {
        let total_examples_in_functions: usize = self
            .functions
            .values()
            .map(|func| func.examples.len())
            .sum();

        DocumentationStatistics {
            total_functions: self.functions.len(),
            total_categories: self.categories.len(),
            total_examples: self.examples.len(),
            total_tutorials: self.tutorials.len(),
            total_function_examples: total_examples_in_functions,
            functions_by_security_level: {
                let mut counts = HashMap::new();
                for func in self.functions.values() {
                    *counts.entry(func.security_level.clone()).or_insert(0) += 1;
                }
                counts
            },
        }
    }
}

/// Documentation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationStatistics {
    pub total_functions: usize,
    pub total_categories: usize,
    pub total_examples: usize,
    pub total_tutorials: usize,
    pub total_function_examples: usize,
    pub functions_by_security_level: HashMap<SecurityLevel, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documentation_creation() {
        let docs = SteelAPIDocumentation::new();

        assert!(!docs.functions.is_empty());
        assert!(!docs.categories.is_empty());
        assert!(!docs.examples.is_empty());
        assert!(!docs.tutorials.is_empty());
    }

    #[test]
    fn test_function_lookup() {
        let docs = SteelAPIDocumentation::new();

        let sleep_doc = docs.get_function_doc("sleep");
        assert!(sleep_doc.is_some());
        assert_eq!(sleep_doc.unwrap().name, "sleep");
        assert_eq!(sleep_doc.unwrap().category, "Hardware Control");
    }

    #[test]
    fn test_category_functions() {
        let docs = SteelAPIDocumentation::new();

        let hardware_functions = docs.list_functions_by_category("Hardware Control");
        assert!(hardware_functions.contains(&"sleep".to_string()));
        assert!(hardware_functions.contains(&"led-on".to_string()));
        assert!(hardware_functions.contains(&"led-off".to_string()));
    }

    #[test]
    fn test_function_search() {
        let docs = SteelAPIDocumentation::new();

        let led_functions = docs.search_functions("led");
        assert!(led_functions.contains(&"led-on".to_string()));
        assert!(led_functions.contains(&"led-off".to_string()));
        assert!(led_functions.contains(&"led-state".to_string()));
    }

    #[test]
    fn test_markdown_generation() {
        let docs = SteelAPIDocumentation::new();

        let markdown = docs.generate_documentation(OutputFormat::Markdown);
        assert!(markdown.contains("# Steel IoT API Documentation"));
        assert!(markdown.contains("## Function Reference"));
        assert!(markdown.contains("### sleep"));
    }

    #[test]
    fn test_json_generation() {
        let docs = SteelAPIDocumentation::new();

        let json = docs.generate_documentation(OutputFormat::Json);
        assert!(json.contains("functions"));
        assert!(json.contains("categories"));
        assert!(json.contains("examples"));
    }

    #[test]
    fn test_statistics() {
        let docs = SteelAPIDocumentation::new();

        let stats = docs.get_statistics();
        assert!(stats.total_functions > 0);
        assert!(stats.total_categories > 0);
        assert!(stats.total_examples > 0);
        assert!(stats.total_tutorials > 0);
    }
}
