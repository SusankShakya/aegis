//! FFI bindings for Aegis Python components.
//! 
//! This crate provides the bridge between Rust and Python components,
//! particularly for AI/ML functionality implemented in Python.

pub mod error;
pub mod models;
pub mod setup;

pub use error::FfiError;
pub use models::PythonLogAnomalyDetector;
pub use setup::ensure_python_initialized;

/// Re-export commonly used PyO3 types and traits
pub use pyo3::{
    prelude::*,
    types::{PyDict, PyList, PyTuple},
};

/// Initialize the FFI layer. This must be called before any other FFI operations.
pub fn initialize() -> Result<(), FfiError> {
    ensure_python_initialized()
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        initialize().expect("FFI initialization failed");
    }

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
