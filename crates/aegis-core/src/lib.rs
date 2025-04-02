//! Aegis Core Framework
//!
//! Aegis is a cross-platform framework for building secure, robust applications.
//! This crate provides the core functionality that is shared across all Aegis
//! platform-specific implementations.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::sync::OnceLock;

/// Unique identifier for the Aegis framework
pub const FRAMEWORK_ID: &str = "aegis-core";

/// Error types and utilities
pub mod error;

/// Configuration structures and utilities 
pub mod config;

/// Platform abstraction layer
pub mod platform;

/// Utility functions
pub mod utils;

/// Version information
pub mod version;

/// Common imports
pub mod prelude;

// Internal module for common functionality
mod internal {
    use crate::utils::generate_uuid;
    use std::sync::OnceLock;

    static INSTANCE_ID: OnceLock<String> = OnceLock::new();

    /// Get the unique instance ID for this Aegis instance
    pub fn instance_id() -> &'static str {
        INSTANCE_ID.get_or_init(|| generate_uuid())
    }
}

/// Get the unique instance ID for this Aegis instance
pub fn instance_id() -> &'static str {
    internal::instance_id()
}

/// Initialize the Aegis framework
///
/// This function must be called before using any Aegis functionality.
/// It initializes internal state and sets up logging.
///
/// # Arguments
///
/// * `config` - Optional configuration. If `None`, default configuration is used.
///
/// # Returns
///
/// Returns `AegisResult<()>` indicating success or failure.
pub fn init(config: Option<&config::AegisConfig>) -> error::AegisResult<()> {
    // Use provided config or create default
    let config = config.cloned().unwrap_or_default();
    
    // Initialize the framework
    // TODO: Add actual initialization logic
    
    Ok(())
}

/// Get the framework version
pub fn version() -> &'static str {
    version::VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_id() {
        let id = instance_id();
        assert!(!id.is_empty());
        
        // Calling again should return the same ID
        let id2 = instance_id();
        assert_eq!(id, id2);
    }

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_init() {
        let result = init(None);
        assert!(result.is_ok());
    }
}
