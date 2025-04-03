# Aegis FFI

FFI bindings for Aegis Python components, providing a bridge between Rust and Python for AI/ML functionality.

## Features

- Python interpreter initialization and management
- FFI bindings for the Log Anomaly Detector model
- Safe error handling and type conversions
- NumPy array integration for efficient data transfer

## Requirements

- Rust 1.70 or later
- Python 3.8 or later
- NumPy package installed in Python environment

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
aegis-ffi = { path = "../aegis-ffi" }
```

Example usage:

```rust
use aegis_ffi::{ensure_python_initialized, PythonLogAnomalyDetector};

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize Python
    ensure_python_initialized()?;

    // Create detector
    let detector = PythonLogAnomalyDetector::new()?;

    // Make predictions
    let log_vector = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let score = detector.predict(&log_vector)?;
    println!("Anomaly score: {}", score);

    Ok(())
}
```

## Error Handling

The crate uses a custom `FfiError` type that handles various error cases:
- Python module/object not found
- Python exceptions
- Type conversion errors
- Initialization failures
- I/O errors

## Development

Run tests:
```bash
cargo test
```

Run examples:
```bash
cargo run --example anomaly_detection
```

## License

Same as Aegis project 