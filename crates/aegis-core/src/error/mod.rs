//! Error types for the Aegis framework
//!
//! This module defines common error types used throughout the Aegis framework.

use thiserror::Error;
use std::fmt;

/// Primary error type for the Aegis framework
#[derive(Error, Debug)]
pub enum AegisError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Platform-specific error
    #[error("Platform error: {0}")]
    Platform(String),

    /// Security-related error
    #[error("Security error: {0}")]
    Security(String),
    
    /// Communication error
    #[error("Communication error: {0}")]
    Communication(String),
    
    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),
    
    /// Generic error with message
    #[error("{0}")]
    Generic(String),
}

impl From<serde_json::Error> for AegisError {
    fn from(err: serde_json::Error) -> Self {
        AegisError::Serialization(err.to_string())
    }
}

impl From<serde_yaml::Error> for AegisError {
    fn from(err: serde_yaml::Error) -> Self {
        AegisError::Serialization(err.to_string())
    }
}

/// Result type alias using AegisError
pub type AegisResult<T> = Result<T, AegisError>;

/// Severity level for errors and logging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Debug information, not an error
    Debug,
    /// Informational message, not an error
    Info,
    /// Warning, operation succeeded but with issues
    Warning,
    /// Error, operation failed
    Error,
    /// Critical error, system functionality compromised
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Debug => write!(f, "DEBUG"),
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = AegisError::Config("Missing required field".to_string());
        assert!(error.to_string().contains("Configuration error"));
        
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error = AegisError::from(io_error);
        assert!(error.to_string().contains("I/O error"));
    }
}
