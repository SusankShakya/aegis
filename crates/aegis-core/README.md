# Aegis Core

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

The core platform-agnostic components of the Aegis framework, providing essential functionality that is shared across all platform-specific implementations.

## Features

- **Error Handling**: Standardized error types and result aliases
- **Configuration**: Flexible configuration system with serialization support
- **Platform Abstraction**: Trait-based platform abstractions for file system, process management, networking, etc.
- **Utilities**: Common utility functions for UUID generation, cryptography, path handling, etc.
- **Versioning**: Semantic version checking and compatibility
- **Logging**: Structured logging with multiple output destinations

## Usage

Add `aegis-core` to your dependencies:

```toml
[dependencies]
aegis-core = "0.1.0"
```

Basic example:

```rust
use aegis_core::prelude::*;

fn main() -> AegisResult<()> {
    // Initialize with default configuration
    init(None)?;
    
    log_info("Aegis core initialized successfully");
    log_info(&format!("Framework ID: {}", FRAMEWORK_ID));
    log_info(&format!("Instance ID: {}", instance_id()));
    log_info(&format!("Version: {}", version()));
    
    // Your application code here...
    
    Ok(())
}
```

## Architecture

The Aegis Core is designed to be platform-agnostic, providing traits and interfaces that are implemented by platform-specific crates. This architecture allows applications to be written once and run on multiple platforms without modification.

Key modules include:

- **error**: Error types and utilities
- **config**: Configuration structures and utilities
- **platform**: Platform abstraction traits
- **utils**: Utility functions
- **version**: Version information and checking
- **logging**: Structured logging functionality

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option. 