use async_trait::async_trait;
use bytes::Bytes;
use std::net::SocketAddr;
use std::fmt;

/// Errors that can occur during network operations
#[derive(Debug)]
pub enum NetworkError {
    /// I/O error during read/write
    IoError(std::io::Error),
    /// Connection was refused
    ConnectionRefused,
    /// Connection was reset
    ConnectionReset,
    /// Invalid address format
    AddrParseError(String),
    /// Connection timed out
    Timeout,
    /// Connection was closed
    ConnectionClosed,
    /// Other error with description
    Other(String),
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::IoError(e) => write!(f, "I/O error: {}", e),
            NetworkError::ConnectionRefused => write!(f, "Connection refused"),
            NetworkError::ConnectionReset => write!(f, "Connection reset"),
            NetworkError::AddrParseError(e) => write!(f, "Address parse error: {}", e),
            NetworkError::Timeout => write!(f, "Operation timed out"),
            NetworkError::ConnectionClosed => write!(f, "Connection closed"),
            NetworkError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl std::error::Error for NetworkError {}

/// Trait for stream-based message transport
#[async_trait]
pub trait MessageStream: Send + Unpin {
    /// Read raw bytes from the stream
    async fn read_message(&mut self) -> Result<Option<Bytes>, NetworkError>;
    
    /// Write raw bytes to the stream
    async fn write_message(&mut self, msg: Bytes) -> Result<(), NetworkError>;
    
    /// Get the peer's address
    fn peer_addr(&self) -> Result<SocketAddr, NetworkError>;
    
    /// Shutdown the stream gracefully
    async fn shutdown(&mut self) -> Result<(), NetworkError>;
}

/// Trait for accepting incoming connections
#[async_trait]
pub trait MessageListener: Send + Unpin {
    /// Accept a new connection
    async fn accept(&mut self) -> Result<(Box<dyn MessageStream>, SocketAddr), NetworkError>;
    
    /// Get the local address being listened on
    fn local_addr(&self) -> Result<SocketAddr, NetworkError>;
}

/// Trait for initiating connections
#[async_trait]
pub trait NetworkConnector: Send + Sync {
    /// Connect to a remote address
    async fn connect(&self, addr: SocketAddr) -> Result<Box<dyn MessageStream>, NetworkError>;
} 