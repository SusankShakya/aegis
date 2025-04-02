//! Main Aegis Framework Library
//!
//! This is the main entry point for the Aegis Framework.
//! It re-exports components from the various crates that make up the framework.

pub use aegis_core as core;

#[cfg(feature = "comms")]
pub use aegis_comms as comms;

/// Framework version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_set() {
        assert!(!super::VERSION.is_empty());
    }
} 