//! Configuration structures and utilities for the Aegis framework
//!
//! This module provides configuration-related functionality that is used across
//! the framework regardless of platform.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{AegisError, AegisResult};

/// Core configuration structure for the Aegis framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AegisConfig {
    /// Unique identifier for this instance
    pub instance_id: String,
    
    /// Base directory for storage
    pub base_dir: PathBuf,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
    
    /// Network configuration
    pub network: NetworkConfig,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: LogLevel,
    
    /// Log output destination
    pub output: LogOutput,
    
    /// Maximum log file size in bytes (if file output is used)
    pub max_file_size: Option<u64>,
    
    /// Maximum number of log files to keep (if file output is used)
    pub max_files: Option<u32>,
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Trace level (most verbose)
    Trace,
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warn,
    /// Error level
    Error,
}

/// Log output destination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum LogOutput {
    /// Output to standard output/console
    Stdout,
    /// Output to a file
    File { path: PathBuf },
    /// Output to syslog
    Syslog,
    /// Multiple outputs
    Multiple { outputs: Vec<LogOutput> },
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable encryption
    pub encryption_enabled: bool,
    
    /// Path to encryption keys
    pub keys_path: Option<PathBuf>,
    
    /// Authorization settings
    pub authorization: AuthorizationConfig,
}

/// Authorization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    /// Authorization mode
    pub mode: AuthMode,
    
    /// Path to authorization policy file
    pub policy_path: Option<PathBuf>,
}

/// Authorization mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    /// No authorization checks
    None,
    /// Basic authorization
    Basic,
    /// Role-based authorization
    Rbac,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Host to bind to
    pub host: String,
    
    /// Port to listen on
    pub port: u16,
    
    /// TLS configuration
    pub tls: Option<TlsConfig>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to certificate file
    pub cert_path: PathBuf,
    
    /// Path to private key file
    pub key_path: PathBuf,
}

/// Default implementation for AegisConfig
impl Default for AegisConfig {
    fn default() -> Self {
        Self {
            instance_id: format!("aegis-{}", uuid::Uuid::new_v4()),
            base_dir: PathBuf::from("./data"),
            logging: LoggingConfig {
                level: LogLevel::Info,
                output: LogOutput::Stdout,
                max_file_size: Some(10 * 1024 * 1024), // 10 MB
                max_files: Some(5),
            },
            security: SecurityConfig {
                encryption_enabled: true,
                keys_path: None,
                authorization: AuthorizationConfig {
                    mode: AuthMode::Basic,
                    policy_path: None,
                },
            },
            network: NetworkConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                tls: None,
            },
        }
    }
}

/// Load configuration from a file
pub fn load_config(path: &PathBuf) -> AegisResult<AegisConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| AegisError::Io(e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| AegisError::Serialization(e))
}

/// Save configuration to a file
pub fn save_config(config: &AegisConfig, path: &PathBuf) -> AegisResult<()> {
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| AegisError::Serialization(e))?;
    
    std::fs::write(path, content)
        .map_err(|e| AegisError::Io(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AegisConfig::default();
        assert_eq!(config.logging.level, LogLevel::Info);
        assert_eq!(config.security.authorization.mode, AuthMode::Basic);
    }

    #[test]
    fn test_serialization() {
        let config = AegisConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AegisConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.network.port, config.network.port);
    }
} 