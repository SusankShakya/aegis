use crate::error::FfiError;
use numpy::{PyArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::PyModule;

/// A handle to a Python LogAnomalyDetector instance.
#[derive(Debug, Clone)]
pub struct PythonLogAnomalyDetector {
    py_object: PyObject,
}

impl PythonLogAnomalyDetector {
    /// Creates a new instance of the Python LogAnomalyDetector.
    /// If model_path is None, initializes a new model with default parameters.
    pub fn new(model_path: Option<&str>) -> Result<Self, FfiError> {
        Python::with_gil(|py| {
            // Import the Python module
            let module = PyModule::import(py, "aegis_python_modules.models.anomaly_detection")
                .map_err(|e| FfiError::ModuleNotFound(format!("Failed to import anomaly_detection module: {}", e)))?;

            // Get the LogAnomalyDetector class
            let detector_class = module
                .getattr("LogAnomalyDetector")
                .map_err(|e| FfiError::ObjectNotFound(format!("Failed to get LogAnomalyDetector class: {}", e)))?;

            // Create an instance with optional model path
            let instance = detector_class
                .call1((model_path,))
                .map_err(|e| FfiError::PythonException(format!("Failed to instantiate LogAnomalyDetector: {}", e)))?;

            Ok(PythonLogAnomalyDetector {
                py_object: instance.into(),
            })
        })
    }

    /// Predicts the anomaly score for a given log vector.
    /// Returns a score between 0.0 and 1.0, where higher values indicate more anomalous logs.
    pub fn predict(&self, log_vector: Vec<f32>) -> Result<f32, FfiError> {
        Python::with_gil(|py| {
            // Convert Rust Vec<f32> to NumPy array
            let py_array = log_vector.to_pyarray(py);

            // Call the predict method
            let result = self
                .py_object
                .call_method1(py, "predict", (py_array,))
                .map_err(|e| FfiError::PythonException(format!("Prediction failed: {}", e)))?;

            // Extract the float result
            result
                .extract(py)
                .map_err(|e| FfiError::ConversionError(format!("Failed to extract prediction result: {}", e)))
        })
    }

    /// Saves the model to the specified path.
    pub fn save_model(&self, path: &str) -> Result<(), FfiError> {
        Python::with_gil(|py| {
            self.py_object
                .call_method1(py, "save_model", (path,))
                .map_err(|e| FfiError::PythonException(format!("Failed to save model: {}", e)))?;
            Ok(())
        })
    }

    /// Returns the Python object for advanced usage.
    /// This is unsafe as it allows bypassing the safe interface.
    pub fn as_py_object(&self) -> &PyObject {
        &self.py_object
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::ensure_python_initialized;
    use tempfile::TempDir;

    #[test]
    fn test_detector_creation() {
        ensure_python_initialized().unwrap();
        let detector = PythonLogAnomalyDetector::new(None).expect("Failed to create detector");
        assert!(!detector.py_object.is_none(&Python::acquire_gil().0));
    }

    #[test]
    fn test_prediction() {
        ensure_python_initialized().unwrap();
        let detector = PythonLogAnomalyDetector::new(None).expect("Failed to create detector");
        
        // Create a sample log vector (this should match your model's expected input size)
        let log_vector = vec![0.0; 100]; // Adjust size as needed
        
        let score = detector.predict(log_vector).expect("Prediction failed");
        assert!((0.0..=1.0).contains(&score), "Score should be between 0 and 1");
    }

    #[test]
    fn test_model_save_load() {
        ensure_python_initialized().unwrap();
        
        // Create a temporary directory for the test
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let model_path = temp_dir.path().join("test_model.pkl");
        let model_path_str = model_path.to_str().unwrap();
        
        // Create and save a model
        let detector = PythonLogAnomalyDetector::new(None).expect("Failed to create detector");
        detector.save_model(model_path_str).expect("Failed to save model");
        
        // Load the saved model
        let loaded_detector = PythonLogAnomalyDetector::new(Some(model_path_str))
            .expect("Failed to load saved model");
        assert!(!loaded_detector.py_object.is_none(&Python::acquire_gil().0));
    }
} 