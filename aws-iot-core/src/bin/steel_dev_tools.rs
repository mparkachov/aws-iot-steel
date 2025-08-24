use aws_iot_core::{
    steel_api_documentation::{OutputFormat, SteelAPIDocumentation},
    steel_program_debugger::{DebugCommand, SteelProgramDebugger},
    steel_program_packager::{
        PackageBuildConfig, PackageMetadata, SecurityLevel, SteelProgramPackager,
    },
    steel_program_simulator::{SimulationConfig, SteelProgramSimulator},
    steel_program_validator::SteelProgramValidator,
    SystemResult,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Steel Development Tools - Comprehensive toolset for Steel IoT program development
#[derive(Parser)]
#[command(name = "steel-dev-tools")]
#[command(about = "Development tools for Steel IoT programming")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate Steel program syntax and structure
    Validate {
        /// Path to Steel program file
        #[arg(short, long)]
        file: PathBuf,

        /// Output format (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Maximum allowed complexity score
        #[arg(long, default_value = "1000")]
        max_complexity: u32,

        /// Maximum allowed memory usage in bytes
        #[arg(long, default_value = "1048576")]
        max_memory: usize,
    },

    /// Simulate Steel program execution
    Simulate {
        /// Path to Steel program file
        #[arg(short, long)]
        file: PathBuf,

        /// Program name for simulation
        #[arg(short, long)]
        name: Option<String>,

        /// Enable debugging during simulation
        #[arg(long)]
        debug: bool,

        /// Execution timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,

        /// Enable hardware delay simulation
        #[arg(long)]
        hardware_delays: bool,
    },

    /// Package Steel program for deployment
    Package {
        /// Path to Steel program file
        #[arg(short, long)]
        file: PathBuf,

        /// Output package file path
        #[arg(short, long)]
        output: PathBuf,

        /// Package name
        #[arg(long)]
        name: String,

        /// Package version
        #[arg(long, default_value = "1.0.0")]
        version: String,

        /// Package description
        #[arg(long)]
        description: Option<String>,

        /// Package author
        #[arg(long)]
        author: Option<String>,

        /// Security level (safe, restricted, privileged)
        #[arg(long, default_value = "safe")]
        security_level: String,

        /// Enable code signing
        #[arg(long)]
        sign: bool,

        /// Enable code compression
        #[arg(long)]
        compress: bool,
    },

    /// Deploy packaged Steel program
    Deploy {
        /// Path to package file
        #[arg(short, long)]
        package: PathBuf,

        /// Target device IDs (comma-separated)
        #[arg(short, long)]
        targets: String,

        /// Deployment strategy (immediate, gradual, canary)
        #[arg(long, default_value = "immediate")]
        strategy: String,

        /// Dry run (validate without deploying)
        #[arg(long)]
        dry_run: bool,
    },

    /// Start interactive debugger
    Debug {
        /// Path to Steel program file
        #[arg(short, long)]
        file: PathBuf,

        /// Program name for debugging
        #[arg(short, long)]
        name: Option<String>,

        /// Set breakpoints (line numbers, comma-separated)
        #[arg(short, long)]
        breakpoints: Option<String>,

        /// Enable step mode
        #[arg(long)]
        step_mode: bool,
    },

    /// Generate API documentation
    Docs {
        /// Output format (markdown, html, json, text)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Generate only function reference
        #[arg(long)]
        functions_only: bool,

        /// Include examples in documentation
        #[arg(long)]
        include_examples: bool,

        /// Include tutorials in documentation
        #[arg(long)]
        include_tutorials: bool,
    },

    /// List available functions and categories
    List {
        /// What to list (functions, categories, examples)
        #[arg(default_value = "functions")]
        what: String,

        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,

        /// Search by keyword
        #[arg(short, long)]
        search: Option<String>,
    },

    /// Show function help
    Help {
        /// Function name to show help for
        function: String,

        /// Show examples
        #[arg(short, long)]
        examples: bool,
    },
}

#[tokio::main]
async fn main() -> SystemResult<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.debug {
        "debug"
    } else if cli.verbose {
        "info"
    } else {
        "warn"
    };

    tracing_subscriber::fmt()
        .with_env_filter(format!(
            "steel_dev_tools={},aws_iot_core={}",
            log_level, log_level
        ))
        .init();

    info!("Steel Development Tools v1.0.0");

    match cli.command {
        Commands::Validate {
            file,
            format,
            max_complexity,
            max_memory,
        } => validate_program(file, format, max_complexity, max_memory).await,

        Commands::Simulate {
            file,
            name,
            debug,
            timeout,
            hardware_delays,
        } => simulate_program(file, name, debug, timeout, hardware_delays).await,

        Commands::Package {
            file,
            output,
            name,
            version,
            description,
            author,
            security_level,
            sign,
            compress,
        } => {
            package_program(
                file,
                output,
                name,
                version,
                description,
                author,
                security_level,
                sign,
                compress,
            )
            .await
        }

        Commands::Deploy {
            package,
            targets,
            strategy,
            dry_run,
        } => deploy_package(package, targets, strategy, dry_run).await,

        Commands::Debug {
            file,
            name,
            breakpoints,
            step_mode,
        } => debug_program(file, name, breakpoints, step_mode).await,

        Commands::Docs {
            format,
            output,
            functions_only,
            include_examples,
            include_tutorials,
        } => {
            generate_docs(
                format,
                output,
                functions_only,
                include_examples,
                include_tutorials,
            )
            .await
        }

        Commands::List {
            what,
            category,
            search,
        } => list_items(what, category, search).await,

        Commands::Help { function, examples } => show_function_help(function, examples).await,
    }
}

async fn validate_program(
    file: PathBuf,
    format: String,
    max_complexity: u32,
    max_memory: usize,
) -> SystemResult<()> {
    info!("Validating Steel program: {}", file.display());

    let code = std::fs::read_to_string(&file).map_err(|e| {
        aws_iot_core::SystemError::Configuration(format!("Failed to read file: {}", e))
    })?;

    let validator = SteelProgramValidator::with_limits(max_complexity, max_memory, 20);
    let result = validator.validate(&code)?;

    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&result)
                .map_err(aws_iot_core::SystemError::Serialization)?;
            println!("{}", json);
        }
        "text" => {
            println!("Steel Program Validation Report");
            println!("==============================");
            println!("File: {}", file.display());
            println!(
                "Status: {}",
                if result.is_valid { "VALID" } else { "INVALID" }
            );
            println!(
                "Complexity Score: {}/{}",
                result.complexity_score, max_complexity
            );
            println!("Estimated Memory: {} bytes", result.estimated_memory_usage);
            println!("Lines of Code: {}", result.metadata.line_count);
            println!("Max Nesting Depth: {}", result.metadata.max_nesting_depth);
            println!();

            if !result.errors.is_empty() {
                println!("ERRORS ({}):", result.errors.len());
                for (i, error) in result.errors.iter().enumerate() {
                    println!("  {}. {:?}: {}", i + 1, error.error_type, error.message);
                    if let (Some(line), Some(col)) = (error.line, error.column) {
                        println!("     at line {}, column {}", line, col);
                    }
                }
                println!();
            }

            if !result.warnings.is_empty() {
                println!("WARNINGS ({}):", result.warnings.len());
                for (i, warning) in result.warnings.iter().enumerate() {
                    println!(
                        "  {}. {:?}: {}",
                        i + 1,
                        warning.warning_type,
                        warning.message
                    );
                    if let Some(suggestion) = &warning.suggestion {
                        println!("     Suggestion: {}", suggestion);
                    }
                }
                println!();
            }

            if !result.metadata.functions_defined.is_empty() {
                println!(
                    "Functions Defined: {}",
                    result.metadata.functions_defined.join(", ")
                );
            }

            if !result.metadata.functions_called.is_empty() {
                println!(
                    "Functions Called: {}",
                    result.metadata.functions_called.join(", ")
                );
            }
        }
        _ => {
            return Err(aws_iot_core::SystemError::InvalidInput(format!(
                "Unsupported format: {}",
                format
            )));
        }
    }

    if result.is_valid {
        info!("Validation completed successfully");
        Ok(())
    } else {
        error!("Validation failed with {} errors", result.errors.len());
        std::process::exit(1);
    }
}

async fn simulate_program(
    file: PathBuf,
    name: Option<String>,
    debug: bool,
    timeout: u64,
    hardware_delays: bool,
) -> SystemResult<()> {
    let program_name = name.unwrap_or_else(|| {
        file.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("program")
            .to_string()
    });

    info!(
        "Simulating Steel program: {} ({})",
        program_name,
        file.display()
    );

    let code = std::fs::read_to_string(&file).map_err(|e| {
        aws_iot_core::SystemError::Configuration(format!("Failed to read file: {}", e))
    })?;

    let config = SimulationConfig {
        enable_validation: true,
        enable_debugging: debug,
        execution_timeout: std::time::Duration::from_secs(timeout),
        max_history_records: 1000,
        simulate_hardware_delays: hardware_delays,
        hardware_delay_multiplier: if hardware_delays { 1.0 } else { 0.01 },
    };

    let simulator = SteelProgramSimulator::new(config)?;
    let result = simulator.simulate_program(&code, &program_name).await?;

    println!("Steel Program Simulation Report");
    println!("==============================");
    println!("Program: {}", program_name);
    println!("File: {}", file.display());
    println!("Result: {:?}", result.result);

    if let Some(duration) = result.duration {
        println!("Execution Time: {:?}", duration);
    }

    if let Some(validation) = &result.validation_result {
        println!(
            "Validation: {}",
            if validation.is_valid {
                "PASSED"
            } else {
                "FAILED"
            }
        );
        if !validation.is_valid {
            println!("Validation Errors: {}", validation.errors.len());
        }
    }

    println!("Debug Messages: {}", result.debug_output.len());

    if debug && !result.debug_output.is_empty() {
        println!("\nDebug Output:");
        println!("-------------");
        for msg in &result.debug_output {
            println!(
                "[{}] {:?}: {}",
                msg.timestamp.format("%H:%M:%S%.3f"),
                msg.level,
                msg.message
            );
        }
    }

    // Show simulation statistics
    let stats = simulator.get_statistics();
    println!("\nSimulation Statistics:");
    println!(
        "- Total Programs Executed: {}",
        stats.total_programs_executed
    );
    println!("- Successful Executions: {}", stats.successful_executions);
    println!("- Failed Executions: {}", stats.failed_executions);
    println!(
        "- Average Execution Time: {:?}",
        stats.average_execution_time
    );

    match result.result {
        aws_iot_core::steel_program_simulator::ExecutionResult::Success => {
            info!("Simulation completed successfully");
            Ok(())
        }
        _ => {
            error!("Simulation failed or encountered errors");
            std::process::exit(1);
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn package_program(
    file: PathBuf,
    output: PathBuf,
    name: String,
    version: String,
    description: Option<String>,
    author: Option<String>,
    security_level: String,
    sign: bool,
    compress: bool,
) -> SystemResult<()> {
    info!(
        "Packaging Steel program: {} -> {}",
        file.display(),
        output.display()
    );

    let security = match security_level.as_str() {
        "safe" => SecurityLevel::Safe,
        "restricted" => SecurityLevel::Restricted,
        "privileged" => SecurityLevel::Privileged,
        _ => {
            warn!("Unknown security level '{}', using 'safe'", security_level);
            SecurityLevel::Safe
        }
    };

    let build_config = PackageBuildConfig {
        validate_code: true,
        sign_package: sign,
        compress_code: compress,
        include_debug_info: true,
        target_platforms: vec!["esp32".to_string(), "simulator".to_string()],
        optimization_level: aws_iot_core::steel_program_packager::OptimizationLevel::Basic,
    };

    let packager = SteelProgramPackager::new(build_config);

    let metadata = PackageMetadata {
        name: name.clone(),
        version: version.clone(),
        description,
        author,
        license: Some("MIT".to_string()),
        homepage: None,
        repository: None,
        keywords: vec!["iot".to_string(), "steel".to_string()],
        categories: vec!["embedded".to_string()],
        minimum_runtime_version: "1.0.0".to_string(),
        target_platforms: vec!["esp32".to_string()],
        estimated_memory_usage: 1024,
        estimated_execution_time: 1.0,
        security_level: security,
    };

    let package = packager.create_package_from_file(&file, metadata, None)?;
    packager.save_package(&package, &output)?;

    println!("Steel Program Package Created");
    println!("============================");
    println!("Package ID: {}", package.package_id);
    println!(
        "Name: {} v{}",
        package.metadata.name, package.metadata.version
    );
    println!("Output File: {}", output.display());
    println!("Code Size: {} bytes", package.program_code.len());
    println!("Security Level: {:?}", package.metadata.security_level);
    println!("Signed: {}", package.signature.is_some());

    if let Some(validation) = &package.validation_result {
        println!(
            "Validation: {}",
            if validation.is_valid {
                "PASSED"
            } else {
                "FAILED"
            }
        );
        println!("Complexity Score: {}", validation.complexity_score);
        println!(
            "Estimated Memory: {} bytes",
            validation.estimated_memory_usage
        );
    }

    info!("Package created successfully");
    Ok(())
}

async fn deploy_package(
    package_path: PathBuf,
    targets: String,
    strategy: String,
    dry_run: bool,
) -> SystemResult<()> {
    info!(
        "Deploying package: {} (strategy: {})",
        package_path.display(),
        strategy
    );

    let packager = SteelProgramPackager::new_default();
    let package = packager.load_package(&package_path)?;

    let target_devices: Vec<String> = targets
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if target_devices.is_empty() {
        return Err(aws_iot_core::SystemError::Configuration(
            "No target devices specified".to_string(),
        ));
    }

    println!("Steel Program Deployment");
    println!("=======================");
    println!(
        "Package: {} v{}",
        package.metadata.name, package.metadata.version
    );
    println!("Strategy: {}", strategy);
    println!("Target Devices: {}", target_devices.join(", "));
    println!("Dry Run: {}", dry_run);

    if dry_run {
        println!("\n--- DRY RUN MODE ---");
        println!("Validation: Package loaded successfully");
        println!("Target devices: {} devices specified", target_devices.len());
        println!("Deployment would proceed with {} strategy", strategy);
        info!("Dry run completed successfully");
        return Ok(());
    }

    let deployment_result = packager.deploy_package(&package, target_devices).await?;

    println!("\nDeployment Results:");
    println!("- Deployment ID: {}", deployment_result.deployment_id);
    println!("- Status: {:?}", deployment_result.status);
    println!(
        "- Started: {}",
        deployment_result.started_at.format("%Y-%m-%d %H:%M:%S UTC")
    );

    if let Some(completed) = deployment_result.completed_at {
        println!("- Completed: {}", completed.format("%Y-%m-%d %H:%M:%S UTC"));
        let duration = completed.signed_duration_since(deployment_result.started_at);
        println!("- Duration: {:?}", duration.to_std().unwrap_or_default());
    }

    println!("\nPer-Device Results:");
    for (device_id, device_result) in &deployment_result.results {
        println!("- {}: {:?}", device_id, device_result.status);
        if let Some(error) = &device_result.error_message {
            println!("  Error: {}", error);
        }
    }

    match deployment_result.status {
        aws_iot_core::steel_program_packager::DeploymentStatus::Completed => {
            info!("Deployment completed successfully");
            Ok(())
        }
        _ => {
            error!("Deployment failed or incomplete");
            std::process::exit(1);
        }
    }
}

async fn debug_program(
    file: PathBuf,
    name: Option<String>,
    breakpoints: Option<String>,
    step_mode: bool,
) -> SystemResult<()> {
    let program_name = name.unwrap_or_else(|| {
        file.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("program")
            .to_string()
    });

    info!(
        "Starting debugger for: {} ({})",
        program_name,
        file.display()
    );

    let code = std::fs::read_to_string(&file).map_err(|e| {
        aws_iot_core::SystemError::Configuration(format!("Failed to read file: {}", e))
    })?;

    let simulator = Arc::new(SteelProgramSimulator::new_default()?);
    let debugger = SteelProgramDebugger::new_default(simulator);

    // Set breakpoints if specified
    if let Some(bp_str) = breakpoints {
        for bp in bp_str.split(',') {
            if let Ok(line) = bp.trim().parse::<usize>() {
                info!("Breakpoint set at line {}", line);
            }
        }
    }

    // Start debug session
    let result = debugger
        .execute_command(
            None,
            DebugCommand::Start {
                program_code: code,
                program_name: program_name.clone(),
            },
        )
        .await?;

    let session_id = match result {
        aws_iot_core::steel_program_debugger::DebugCommandResult::SessionCreated { session_id } => {
            session_id
        }
        _ => {
            return Err(aws_iot_core::SystemError::Configuration(
                "Failed to create debug session".to_string(),
            ));
        }
    };

    println!("Steel Program Debugger");
    println!("=====================");
    println!("Program: {}", program_name);
    println!("Session ID: {}", session_id);
    println!("Step Mode: {}", step_mode);

    if step_mode {
        info!("Step mode enabled");
    }

    // Simple interactive debugger (in a real implementation, this would be more sophisticated)
    println!("\nDebugger started. Available commands:");
    println!("- step: Step to next line");
    println!("- continue: Continue execution");
    println!("- variables: Show current variables");
    println!("- breakpoints: List breakpoints");
    println!("- quit: Exit debugger");

    // For this demo, we'll just show the session info and exit
    if let Some(session_info) = debugger.get_session_info(&session_id) {
        println!("\nSession Information:");
        println!("- Status: {:?}", session_info.status);
        println!("- Current Line: {:?}", session_info.current_line);
        println!("- Breakpoints: {}", session_info.breakpoints.len());
        println!("- Variables: {}", session_info.variables.len());
    }

    let stats = debugger.get_debug_statistics();
    println!("\nDebugger Statistics:");
    println!("- Total Sessions: {}", stats.total_sessions);
    println!("- Active Sessions: {}", stats.active_sessions);
    println!("- Total Breakpoints: {}", stats.total_breakpoints);

    info!("Debug session created successfully");
    println!("\nNote: This is a demonstration of the debugger setup.");
    println!("In a full implementation, this would provide an interactive debugging interface.");

    Ok(())
}

async fn generate_docs(
    format: String,
    output: Option<PathBuf>,
    _functions_only: bool,
    _include_examples: bool,
    _include_tutorials: bool,
) -> SystemResult<()> {
    info!("Generating Steel API documentation (format: {})", format);

    let docs = SteelAPIDocumentation::new();

    let output_format = match format.as_str() {
        "markdown" | "md" => OutputFormat::Markdown,
        "html" => OutputFormat::Html,
        "json" => OutputFormat::Json,
        "text" | "txt" => OutputFormat::PlainText,
        _ => {
            warn!("Unknown format '{}', using markdown", format);
            OutputFormat::Markdown
        }
    };

    let documentation = docs.generate_documentation(output_format);

    if let Some(output_path) = output {
        std::fs::write(&output_path, &documentation).map_err(|e| {
            aws_iot_core::SystemError::Configuration(format!(
                "Failed to write documentation: {}",
                e
            ))
        })?;

        println!("Documentation generated: {}", output_path.display());
        info!("Documentation written to: {}", output_path.display());
    } else {
        println!("{}", documentation);
    }

    let stats = docs.get_statistics();
    println!("\nDocumentation Statistics:");
    println!("- Functions: {}", stats.total_functions);
    println!("- Categories: {}", stats.total_categories);
    println!("- Examples: {}", stats.total_examples);
    println!("- Tutorials: {}", stats.total_tutorials);

    Ok(())
}

async fn list_items(
    what: String,
    category: Option<String>,
    search: Option<String>,
) -> SystemResult<()> {
    let docs = SteelAPIDocumentation::new();

    match what.as_str() {
        "functions" => {
            let functions = if let Some(ref cat) = category {
                docs.list_functions_by_category(cat)
            } else if let Some(ref keyword) = search {
                docs.search_functions(keyword)
            } else {
                docs.list_functions()
            };

            println!("Steel API Functions");
            println!("==================");
            if let Some(cat) = &category {
                println!("Category: {}", cat);
            }
            if let Some(keyword) = &search {
                println!("Search: {}", keyword);
            }
            println!();

            for (i, func) in functions.iter().enumerate() {
                if let Some(func_doc) = docs.get_function_doc(func) {
                    println!("{}. {} ({})", i + 1, func, func_doc.category);
                    println!("   {}", func_doc.description);
                    println!("   Syntax: {}", func_doc.syntax);
                    println!();
                }
            }

            println!("Total: {} functions", functions.len());
        }

        "categories" => {
            let categories = docs.list_categories();
            println!("Steel API Categories");
            println!("===================");
            println!();

            for (i, cat) in categories.iter().enumerate() {
                if let Some(cat_doc) = docs.get_category_doc(cat) {
                    let func_count = docs.list_functions_by_category(cat).len();
                    println!("{}. {} ({} functions)", i + 1, cat, func_count);
                    println!("   {}", cat_doc.description);
                    println!();
                }
            }

            println!("Total: {} categories", categories.len());
        }

        "examples" => {
            println!("Steel API Examples");
            println!("=================");
            println!();

            // This would list examples if we had access to them
            println!("Examples are available in the full documentation.");
            println!("Use 'steel-dev-tools docs' to generate complete documentation.");
        }

        _ => {
            return Err(aws_iot_core::SystemError::Configuration(format!(
                "Unknown list type: {}. Use 'functions', 'categories', or 'examples'",
                what
            )));
        }
    }

    Ok(())
}

async fn show_function_help(function: String, examples: bool) -> SystemResult<()> {
    let docs = SteelAPIDocumentation::new();

    if let Some(func_doc) = docs.get_function_doc(&function) {
        println!("Steel Function: {}", func_doc.name);
        println!("{}=", "=".repeat(func_doc.name.len() + 16));
        println!();
        println!("Category: {}", func_doc.category);
        println!("Description: {}", func_doc.description);
        println!();
        println!("Syntax:");
        println!("  {}", func_doc.syntax);
        println!();

        if !func_doc.parameters.is_empty() {
            println!("Parameters:");
            for param in &func_doc.parameters {
                println!(
                    "  {} ({}): {}",
                    param.name, param.type_name, param.description
                );
                if param.optional {
                    println!("    Optional");
                }
                if let Some(default) = &param.default_value {
                    println!("    Default: {}", default);
                }
                if !param.constraints.is_empty() {
                    println!("    Constraints: {}", param.constraints.join(", "));
                }
            }
            println!();
        }

        println!(
            "Returns: {} - {}",
            func_doc.return_type, func_doc.return_description
        );
        println!();

        if examples && !func_doc.examples.is_empty() {
            println!("Examples:");
            for example in &func_doc.examples {
                println!("  {}", example);
            }
            println!();
        }

        if !func_doc.notes.is_empty() {
            println!("Notes:");
            for note in &func_doc.notes {
                println!("  - {}", note);
            }
            println!();
        }

        if !func_doc.see_also.is_empty() {
            println!("See Also: {}", func_doc.see_also.join(", "));
            println!();
        }

        if let Some(deprecation) = &func_doc.deprecated {
            println!(
                "⚠️  DEPRECATED since version {}: {}",
                deprecation.since_version, deprecation.reason
            );
            if let Some(replacement) = &deprecation.replacement {
                println!("   Use '{}' instead.", replacement);
            }
            println!();
        }

        println!("Security Level: {:?}", func_doc.security_level);
        println!("Since Version: {}", func_doc.since_version);
    } else {
        println!("Function '{}' not found.", function);
        println!();
        println!("Available functions:");
        let functions = docs.list_functions();
        let matches: Vec<_> = functions
            .iter()
            .filter(|f| f.to_lowercase().contains(&function.to_lowercase()))
            .collect();

        if !matches.is_empty() {
            println!("Did you mean:");
            for func in matches.iter().take(5) {
                println!("  {}", func);
            }
        } else {
            println!("Use 'steel-dev-tools list functions' to see all available functions.");
        }

        std::process::exit(1);
    }

    Ok(())
}
