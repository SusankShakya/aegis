use std::path::PathBuf;
use std::sync::Once;
use pyo3::prelude::*;
use crate::error::FfiError;

static PYTHON_INIT: Once = Once::new();
static mut INITIALIZED: bool = false;

/// Configuration for the Python FFI layer
#[derive(Debug, Clone)]
pub struct FfiConfig {
    /// Path to the aegis_python_modules directory
    pub module_path: PathBuf,
    /// Whether to enable debug logging
    pub debug: bool,
}

impl Default for FfiConfig {
    fn default() -> Self {
        Self {
            module_path: get_default_module_path(),
            debug: false,
        }
    }
}

/// Initialize the Python FFI layer with custom configuration
pub fn initialize_with_config(config: FfiConfig) -> Result<(), FfiError> {
    PYTHON_INIT.call_once(|| {
        // Initialize Python (handled by auto-initialize feature)
        let result = Python::with_gil(|py| -> PyResult<()> {
            if config.debug {
                log::debug!("Initializing Python FFI layer");
            }

            // Import our utility module to help with path setup
            let utils = PyModule::from_code(
                py,
                include_str!("../../../../aegis_python_modules/utils/path_helper.py"),
                "path_helper.py",
                "path_helper",
            )?;

            // Call ensure_module_path with our config path
            let path_str = config.module_path.to_string_lossy().to_string();
            utils.getattr("ensure_module_path")?.call1((path_str,))?;

            // Verify we can import our package
            match py.import("aegis_python_modules") {
                Ok(_) => {
                    if config.debug {
                        log::debug!("Successfully imported aegis_python_modules");
                    }
                    unsafe { INITIALIZED = true; }
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to import aegis_python_modules: {}", e);
                    Err(e)
                }
            }
        });

        if let Err(e) = result {
            log::error!("Python initialization failed: {}", e);
            panic!("Python FFI initialization failed: {}", e);
        }
    });

    if unsafe { !INITIALIZED } {
        return Err(FfiError::InitializationError(
            "Python initialization failed".to_string(),
        ));
    }

    Ok(())
}

/// Initialize the Python FFI layer with default configuration
pub fn initialize() -> Result<(), FfiError> {
    initialize_with_config(FfiConfig::default())
}

/// Get the default path to the aegis_python_modules directory
fn get_default_module_path() -> PathBuf {
    // First try environment variable
    if let Ok(path) = std::env::var("AEGIS_PYTHON_MODULES") {
        return PathBuf::from(path);
    }

    // Fall back to relative path from executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(project_root) = exe_path.parent().and_then(|p| p.parent()) {
            return project_root.join("aegis_python_modules");
        }
    }

    // Last resort: try current directory
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("aegis_python_modules")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn setup_test_env() -> PathBuf {
        let dir = tempdir().unwrap();
        let module_path = dir.path().join("aegis_python_modules");
        fs::create_dir_all(&module_path).unwrap();
        
        // Create __init__.py
        fs::write(
            module_path.join("__init__.py"),
            "VERSION = '0.1.0'\n",
        ).unwrap();

        module_path
    }

    #[test]
    fn test_initialization() {
        let module_path = setup_test_env();
        let config = FfiConfig {
            module_path,
            debug: true,
        };

        let result = initialize_with_config(config);
        assert!(result.is_ok(), "Initialization failed: {:?}", result);

        Python::with_gil(|py| {
            let sys = py.import("sys").unwrap();
            let version: String = sys.getattr("version").unwrap().extract().unwrap();
            println!("Python version: {}", version);

            let aegis = py.import("aegis_python_modules").unwrap();
            assert!(aegis.hasattr("VERSION").unwrap());
        });
    }
} 