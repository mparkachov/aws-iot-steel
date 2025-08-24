use crate::{LogLevel, SystemResult};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Logging configuration structure
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub target: LogTarget,
    pub format: LogFormat,
    pub include_timestamps: bool,
    pub include_thread_ids: bool,
    pub include_file_locations: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            target: LogTarget::Stdout,
            format: LogFormat::Pretty,
            include_timestamps: true,
            include_thread_ids: false,
            include_file_locations: false,
        }
    }
}

/// Log output target
#[derive(Debug, Clone)]
pub enum LogTarget {
    Stdout,
    Stderr,
    File(String),
}

/// Log format options
#[derive(Debug, Clone)]
pub enum LogFormat {
    Pretty,
    Json,
    Compact,
}

/// Initialize the logging framework with the specified configuration
///
/// # Arguments
/// * `config` - The logging configuration to use
///
/// # Returns
/// * `Ok(())` if logging was initialized successfully
/// * `Err(SystemError)` if initialization failed
pub fn initialize_logging(config: LoggingConfig) -> SystemResult<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let level_str = match config.level {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        };
        EnvFilter::new(level_str)
    });

    let fmt_layer = match config.format {
        LogFormat::Pretty => fmt::layer()
            .pretty()
            .with_thread_ids(config.include_thread_ids)
            .with_file(config.include_file_locations)
            .with_line_number(config.include_file_locations)
            .boxed(),
        LogFormat::Json => fmt::layer()
            .json()
            .with_thread_ids(config.include_thread_ids)
            .with_file(config.include_file_locations)
            .with_line_number(config.include_file_locations)
            .boxed(),
        LogFormat::Compact => fmt::layer()
            .compact()
            .with_thread_ids(config.include_thread_ids)
            .with_file(config.include_file_locations)
            .with_line_number(config.include_file_locations)
            .boxed(),
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    tracing::info!("Logging initialized with level: {}", config.level);
    Ok(())
}

/// Initialize logging with default configuration
/// This is a convenience function for quick setup
pub fn initialize_default_logging() -> SystemResult<()> {
    initialize_logging(LoggingConfig::default())
}

/// Initialize logging for development with debug level and pretty formatting
pub fn initialize_dev_logging() -> SystemResult<()> {
    let config = LoggingConfig {
        level: LogLevel::Debug,
        format: LogFormat::Pretty,
        include_file_locations: true,
        ..Default::default()
    };
    initialize_logging(config)
}

/// Initialize logging for production with info level and JSON formatting
pub fn initialize_prod_logging() -> SystemResult<()> {
    let config = LoggingConfig {
        level: LogLevel::Info,
        format: LogFormat::Json,
        include_timestamps: true,
        include_thread_ids: true,
        ..Default::default()
    };
    initialize_logging(config)
}

/// Structured logging macros for consistent log formatting
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        tracing::trace!($($arg)*)
    };
}
