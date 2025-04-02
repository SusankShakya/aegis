//! Platform-specific implementations of network abstractions
//!
//! This module contains implementations of the network abstractions
//! for different platforms.

// Tokio-based implementation for standard platforms
#[cfg(feature = "platform_tokio_net")]
pub mod tokio_impl; 