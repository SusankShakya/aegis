//! Aegis Communications Library
//! 
//! This crate provides the communication layer for the Aegis agent framework,
//! with platform-agnostic abstractions and implementations for different platforms.

mod framing;
mod manager;
mod platform;
mod protocol;
mod transport;

pub use framing::{FramedMessageStream, FramingError};
pub use manager::{CommsClient, CommsError, ConnectionHandle};
pub use protocol::*;
pub use transport::{MessageListener, MessageStream, NetworkConnector, NetworkError};

// Re-export platform-specific implementations when enabled
#[cfg(feature = "platform_tokio_net")]
pub use platform::tokio_impl::{connect_tokio, listen_tokio, TokioConnector, TokioTcpListener};
