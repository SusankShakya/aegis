//! Version information for the Aegis framework
//!
//! This module provides version information and version checking utilities
//! for the Aegis framework.

use semver::{Version, VersionReq};
use crate::error::{AegisError, AegisResult};

/// Current version of the Aegis framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parse the current version into a semver Version
pub fn current_version() -> Version {
    Version::parse(VERSION).expect("Invalid version format in Cargo.toml")
}

/// Check if the current version is compatible with a version requirement
///
/// # Arguments
///
/// * `req` - Version requirement string (e.g. "^1.0.0", ">=2.0.0, <3.0.0")
///
/// # Returns
///
/// `true` if the current version satisfies the requirement
pub fn is_compatible(req: &str) -> AegisResult<bool> {
    let req = VersionReq::parse(req)
        .map_err(|e| AegisError::Serialization(format!("Invalid version requirement: {}", e)))?;
    
    Ok(req.matches(&current_version()))
}

/// Check if a version string is compatible with a version requirement
///
/// # Arguments
///
/// * `version` - Version string to check
/// * `req` - Version requirement string
///
/// # Returns
///
/// `true` if the version satisfies the requirement
pub fn check_version(version: &str, req: &str) -> AegisResult<bool> {
    let version = Version::parse(version)
        .map_err(|e| AegisError::Serialization(format!("Invalid version string: {}", e)))?;
    
    let req = VersionReq::parse(req)
        .map_err(|e| AegisError::Serialization(format!("Invalid version requirement: {}", e)))?;
    
    Ok(req.matches(&version))
}

/// Get version metadata as a structured object
pub fn version_info() -> VersionInfo {
    VersionInfo {
        version: VERSION.to_string(),
        major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or(0),
        minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or(0),
        patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or(0),
        pre_release: if VERSION.contains('-') {
            VERSION.split('-').nth(1).unwrap_or("").to_string()
        } else {
            String::new()
        },
    }
}

/// Structured version information
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VersionInfo {
    /// Full version string
    pub version: String,
    /// Major version number
    pub major: u64,
    /// Minor version number
    pub minor: u64,
    /// Patch version number
    pub patch: u64,
    /// Pre-release identifier (if any)
    pub pre_release: String,
}

impl VersionInfo {
    /// Check if this is a pre-release version
    pub fn is_pre_release(&self) -> bool {
        !self.pre_release.is_empty()
    }
    
    /// Convert to semver Version
    pub fn to_semver(&self) -> Version {
        Version::parse(&self.version).unwrap_or_else(|_| {
            Version::new(self.major, self.minor, self.patch)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version() {
        let version = current_version();
        assert_eq!(version.to_string(), VERSION);
    }

    #[test]
    fn test_version_info() {
        let info = version_info();
        assert_eq!(info.version, VERSION);
        
        let semver = current_version();
        assert_eq!(info.major, semver.major);
        assert_eq!(info.minor, semver.minor);
        assert_eq!(info.patch, semver.patch);
    }

    #[test]
    fn test_is_compatible() {
        // Same version is always compatible
        assert!(is_compatible(VERSION).unwrap());
        
        // Compatible with any version
        assert!(is_compatible("*").unwrap());
        
        // Test with the current version components
        let version = current_version();
        let req = format!("^{}.{}.0", version.major, version.minor);
        assert!(is_compatible(&req).unwrap());
    }

    #[test]
    fn test_check_version() {
        assert!(check_version("1.0.0", ">=1.0.0").unwrap());
        assert!(check_version("1.0.0", "^1.0.0").unwrap());
        assert!(check_version("1.1.0", "^1.0.0").unwrap());
        assert!(!check_version("2.0.0", "^1.0.0").unwrap());
        assert!(!check_version("0.9.0", "^1.0.0").unwrap());
    }
} 