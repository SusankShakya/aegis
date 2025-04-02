//! Prelude module that re-exports commonly used types
//!
//! This module provides a convenient way to import common Aegis types
//! and functions with a single `use` statement.

// Re-export error types
pub use crate::error::{AegisError, AegisResult, Severity};

// Re-export config types
pub use crate::config::{
    AegisConfig, 
    LoggingConfig, 
    SecurityConfig, 
    NetworkConfig,
    LogLevel,
    AuthMode,
};

// Re-export platform traits
pub use crate::platform::{
    FileSystem,
    ProcessManager,
    ProcessHandle,
    ProcessStatus,
    Network,
    Environment,
    SystemInfo,
    PlatformFactory,
};

// Re-export utility functions
pub use crate::utils::{
    generate_uuid,
    current_timestamp,
    current_timestamp_ms,
    to_json,
    from_json,
    to_yaml,
    from_yaml,
    sha256_hash,
    format_hex,
    parse_hex,
    random_bytes,
    normalize_path,
    join_paths,
};

// Re-export version functions and types
pub use crate::version::{
    VERSION,
    VersionInfo,
    current_version,
    is_compatible,
    check_version,
    version_info,
};

// Re-export logging functions and types
pub use crate::logging::{
    LogEvent,
    log_trace,
    log_debug,
    log_info,
    log_warn,
    log_error,
    convert_log_level,
    severity_to_log_level,
};

// Constants
pub use crate::FRAMEWORK_ID;

// Framework functions
pub use crate::{init, instance_id, version}; 