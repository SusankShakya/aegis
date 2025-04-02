//! Logging module for the Aegis framework
//!
//! This module provides structured logging functionality for the Aegis framework,
//! allowing for consistent log formatting and filtering across all components.

use crate::config::{LoggingConfig, LogLevel, LogOutput};
use crate::error::{AegisError, AegisResult, Severity};
use std::path::Path;
use tracing::{debug, error, info, trace, warn};

/// Initialize the logging subsystem
///
/// # Arguments
///
/// * `config` - Logging configuration
///
/// # Returns
///
/// Returns `AegisResult<()>` indicating success or failure.
pub fn init(config: &LoggingConfig) -> AegisResult<()> {
    // Note: This is a placeholder that would be replaced with actual
    // tracing initialization code in a real implementation.
    // For this prototype, we just log the configuration.
    
    info!("Initializing logging with level: {:?}", config.level);
    
    match &config.output {
        LogOutput::Stdout => {
            info!("Logging to stdout");
        }
        LogOutput::File { path } => {
            info!("Logging to file: {:?}", path);
            info!("Max file size: {:?}", config.max_file_size);
            info!("Max file count: {:?}", config.max_files);
        }
        LogOutput::Syslog => {
            info!("Logging to syslog");
        }
        LogOutput::Multiple { outputs } => {
            info!("Logging to multiple outputs: {}", outputs.len());
        }
    }
    
    Ok(())
}

/// Log levels from tracing, mapped to our LogLevel enum
#[inline]
pub fn log_trace(message: &str) {
    trace!("{}", message);
}

/// Log at debug level
#[inline]
pub fn log_debug(message: &str) {
    debug!("{}", message);
}

/// Log at info level
#[inline]
pub fn log_info(message: &str) {
    info!("{}", message);
}

/// Log at warning level
#[inline]
pub fn log_warn(message: &str) {
    warn!("{}", message);
}

/// Log at error level
#[inline]
pub fn log_error(message: &str) {
    error!("{}", message);
}

/// Convert our LogLevel to tracing's level filter
pub fn convert_log_level(level: LogLevel) -> tracing::level_filters::LevelFilter {
    match level {
        LogLevel::Trace => tracing::level_filters::LevelFilter::TRACE,
        LogLevel::Debug => tracing::level_filters::LevelFilter::DEBUG,
        LogLevel::Info => tracing::level_filters::LevelFilter::INFO,
        LogLevel::Warn => tracing::level_filters::LevelFilter::WARN,
        LogLevel::Error => tracing::level_filters::LevelFilter::ERROR,
    }
}

/// Convert Severity to LogLevel
pub fn severity_to_log_level(severity: Severity) -> LogLevel {
    match severity {
        Severity::Debug => LogLevel::Debug,
        Severity::Info => LogLevel::Info,
        Severity::Warning => LogLevel::Warn,
        Severity::Error => LogLevel::Error,
        Severity::Critical => LogLevel::Error,
    }
}

/// A structured log event with standardized fields
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEvent {
    /// Timestamp when the event occurred
    pub timestamp: String,
    
    /// Log level
    pub level: LogLevel,
    
    /// Message
    pub message: String,
    
    /// Module path where the log originated
    pub module_path: Option<String>,
    
    /// File where the log originated
    pub file: Option<String>,
    
    /// Line number where the log originated
    pub line: Option<u32>,
    
    /// Additional context as key-value pairs
    pub context: std::collections::HashMap<String, serde_json::Value>,
}

impl LogEvent {
    /// Create a new log event
    pub fn new(level: LogLevel, message: String) -> Self {
        Self {
            timestamp: crate::utils::current_timestamp(),
            level,
            message,
            module_path: None,
            file: None,
            line: None,
            context: std::collections::HashMap::new(),
        }
    }
    
    /// Add source location information
    pub fn with_location(mut self, module_path: Option<&str>, file: Option<&str>, line: Option<u32>) -> Self {
        self.module_path = module_path.map(String::from);
        self.file = file.map(String::from);
        self.line = line;
        self
    }
    
    /// Add context key-value pair
    pub fn with_context<T: serde::Serialize>(mut self, key: &str, value: T) -> AegisResult<Self> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| AegisError::Serialization(e.to_string()))?;
        
        self.context.insert(key.to_string(), json_value);
        Ok(self)
    }
    
    /// Serialize to JSON
    pub fn to_json(&self) -> AegisResult<String> {
        serde_json::to_string(self)
            .map_err(|e| AegisError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_level_conversion() {
        assert_eq!(
            convert_log_level(LogLevel::Debug),
            tracing::level_filters::LevelFilter::DEBUG
        );
        
        assert_eq!(
            convert_log_level(LogLevel::Error),
            tracing::level_filters::LevelFilter::ERROR
        );
    }
    
    #[test]
    fn test_severity_conversion() {
        assert_eq!(
            severity_to_log_level(Severity::Debug),
            LogLevel::Debug
        );
        
        assert_eq!(
            severity_to_log_level(Severity::Critical),
            LogLevel::Error
        );
    }
    
    #[test]
    fn test_log_event_serialization() {
        let event = LogEvent::new(LogLevel::Info, "Test message".to_string())
            .with_location(Some("mod_path"), Some("file.rs"), Some(42));
        
        let json = event.to_json().unwrap();
        assert!(json.contains("Test message"));
        assert!(json.contains("file.rs"));
        assert!(json.contains("42"));
    }
    
    #[test]
    fn test_log_event_with_context() {
        let event = LogEvent::new(LogLevel::Info, "Test with context".to_string())
            .with_context("user_id", "user-123").unwrap()
            .with_context("count", 42).unwrap();
        
        let json = event.to_json().unwrap();
        assert!(json.contains("user-123"));
        assert!(json.contains("42"));
    }
} 