use pyo3::PyErr;
use thiserror::Error;

/// Errors that can occur during FFI operations
#[derive(Error, Debug)]
pub enum FfiError {
    /// Python exception occurred
    #[error("Python exception: {0}")]
    PythonException(String),

    /// Python module not found
    #[error("Python module not found: {0}")]
    ModuleNotFound(String),

    /// Python object or attribute not found
    #[error("Python object or attribute not found: {0}")]
    ObjectNotFound(String),

    /// Error converting data between Rust and Python
    #[error("Error converting data between Rust and Python: {0}")]
    ConversionError(String),

    /// Python interpreter initialization failed
    #[error("Python interpreter initialization failed: {0}")]
    InitializationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<PyErr> for FfiError {
    fn from(err: PyErr) -> Self {
        match err.get_type().name() {
            Ok(name) => match name {
                "ModuleNotFoundError" => FfiError::ModuleNotFound(err.to_string()),
                "AttributeError" => FfiError::ObjectNotFound(err.to_string()),
                "TypeError" | "ValueError" => FfiError::ConversionError(err.to_string()),
                _ => FfiError::PythonException(err.to_string()),
            },
            Err(_) => FfiError::PythonException(err.to_string()),
        }
    }
} 