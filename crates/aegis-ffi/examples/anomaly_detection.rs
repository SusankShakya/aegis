use aegis_ffi::{ensure_python_initialized, PythonLogAnomalyDetector};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize Python interpreter and modules
    ensure_python_initialized()?;

    // Create a new detector instance
    let detector = PythonLogAnomalyDetector::new()?;

    // Example log vector (simplified for demonstration)
    let log_vector = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    
    // Get prediction
    let score = detector.predict(&log_vector)?;
    println!("Anomaly score for log vector: {}", score);

    // Save the model (optional)
    detector.save_model("example_model.pkl")?;
    println!("Model saved successfully");

    Ok(())
} 