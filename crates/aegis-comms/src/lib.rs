//! Aegis Communications Library
//!
//! `aegis-comms` provides platform-agnostic communication infrastructure
//! for the Aegis framework, enabling agents to exchange messages over
//! various transport mechanisms.

// Protocol definitions
pub mod protocol;

// Transport abstractions
pub mod transport;

// Message framing
pub mod framing;

// Platform-specific implementations
pub mod platform;

// High-level communication API
pub mod manager;

// Re-exports for convenience
pub use protocol::{Message, MessageHeader, MessageType, Priority, AgentId};
pub use transport::{MessageStream, MessageListener, NetworkConnector, NetworkError, NetworkResult};
pub use framing::{FramedMessageStream, FramingError, FramingResult};
pub use manager::{CommsClient, ConnectionHandle, CommsError, CommsResult};

// Conditionally re-export platform-specific constructors
#[cfg(feature = "platform_tokio_net")]
pub use platform::tokio_impl::{connect_tokio, listen_tokio, TokioConnector};

/// Create a default connector based on available platform implementations
#[cfg(feature = "platform_tokio_net")]
pub fn create_default_connector() -> std::sync::Arc<dyn NetworkConnector + Send + Sync> {
    let connector = platform::tokio_impl::TokioConnector::new();
    std::sync::Arc::new(connector)
}

/// Create a default CommsClient based on available platform implementations
#[cfg(feature = "platform_tokio_net")]
pub fn create_default_client() -> CommsClient {
    let connector = create_default_connector();
    CommsClient::new(connector)
}

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Tests
#[cfg(test)]
mod tests {
    #[test]
    fn version_is_set() {
        assert!(!super::VERSION.is_empty());
    }
}
