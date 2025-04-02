//! Network transport abstractions
//!
//! This module defines abstract traits for network operations,
//! allowing for different implementations based on platform.

use std::net::SocketAddr;
use std::fmt;
use async_trait::async_trait;
use bytes::Bytes;
use aegis_core::error::AegisError;

/// Errors that can occur during network operations
#[derive(Debug)]
pub enum NetworkError {
    /// Connection refused by remote host
    ConnectionRefused,
    /// Connection timed out
    ConnectionTimeout,
    /// Connection closed/reset by peer
    ConnectionClosed,
    /// Host unreachable
    HostUnreachable,
    /// Network unreachable
    NetworkUnreachable,
    /// Address in use
    AddressInUse,
    /// Invalid address
    InvalidAddress,
    /// Permission denied
    PermissionDenied,
    /// Connection already established
    AlreadyConnected,
    /// Not connected
    NotConnected,
    /// Buffer full
    BufferFull,
    /// I/O error (platform-specific)
    IoError(std::io::Error),
    /// Address resolution error
    AddrParseError(std::net::AddrParseError),
    /// Protocol Error
    ProtocolError(String),
    /// Shutdown error
    ShutdownError(String),
    /// Other error
    Other(String),
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::ConnectionRefused => write!(f, "Connection refused"),
            NetworkError::ConnectionTimeout => write!(f, "Connection timed out"),
            NetworkError::ConnectionClosed => write!(f, "Connection closed by peer"),
            NetworkError::HostUnreachable => write!(f, "Host unreachable"),
            NetworkError::NetworkUnreachable => write!(f, "Network unreachable"),
            NetworkError::AddressInUse => write!(f, "Address already in use"),
            NetworkError::InvalidAddress => write!(f, "Invalid address"),
            NetworkError::PermissionDenied => write!(f, "Permission denied"),
            NetworkError::AlreadyConnected => write!(f, "Already connected"),
            NetworkError::NotConnected => write!(f, "Not connected"),
            NetworkError::BufferFull => write!(f, "Buffer full"),
            NetworkError::IoError(e) => write!(f, "I/O error: {}", e),
            NetworkError::AddrParseError(e) => write!(f, "Address parse error: {}", e),
            NetworkError::ProtocolError(e) => write!(f, "Protocol error: {}", e),
            NetworkError::ShutdownError(e) => write!(f, "Shutdown error: {}", e),
            NetworkError::Other(e) => write!(f, "Network error: {}", e),
        }
    }
}

impl From<std::io::Error> for NetworkError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::ConnectionRefused => NetworkError::ConnectionRefused,
            std::io::ErrorKind::ConnectionAborted => NetworkError::ConnectionClosed,
            std::io::ErrorKind::ConnectionReset => NetworkError::ConnectionClosed,
            std::io::ErrorKind::NotConnected => NetworkError::NotConnected,
            std::io::ErrorKind::AddrInUse => NetworkError::AddressInUse,
            std::io::ErrorKind::AddrNotAvailable => NetworkError::InvalidAddress,
            std::io::ErrorKind::BrokenPipe => NetworkError::ConnectionClosed,
            std::io::ErrorKind::TimedOut => NetworkError::ConnectionTimeout,
            std::io::ErrorKind::PermissionDenied => NetworkError::PermissionDenied,
            _ => NetworkError::IoError(err),
        }
    }
}

impl From<std::net::AddrParseError> for NetworkError {
    fn from(err: std::net::AddrParseError) -> Self {
        NetworkError::AddrParseError(err)
    }
}

impl From<NetworkError> for AegisError {
    fn from(err: NetworkError) -> Self {
        AegisError::Communication(err.to_string())
    }
}

/// Result type for network operations
pub type NetworkResult<T> = Result<T, NetworkError>;

/// Abstract trait for message streams (connections)
#[async_trait]
pub trait MessageStream: Send + Unpin {
    /// Read raw bytes from the stream
    async fn read_bytes(&mut self, buf: &mut [u8]) -> NetworkResult<usize>;
    
    /// Read exact number of bytes, returning error if not enough available
    async fn read_exact(&mut self, buf: &mut [u8]) -> NetworkResult<()>;
    
    /// Read a complete message from the stream
    async fn read_message(&mut self) -> NetworkResult<Option<Bytes>>;
    
    /// Write bytes to the stream
    async fn write_bytes(&mut self, buf: &[u8]) -> NetworkResult<usize>;
    
    /// Write all bytes, ensuring complete write
    async fn write_all(&mut self, buf: &[u8]) -> NetworkResult<()>;
    
    /// Write a complete message to the stream
    async fn write_message(&mut self, msg: Bytes) -> NetworkResult<()>;
    
    /// Get the peer address
    fn peer_addr(&self) -> NetworkResult<SocketAddr>;
    
    /// Shutdown the connection
    async fn shutdown(&mut self) -> NetworkResult<()>;
}

/// Abstract trait for listening for connections
#[async_trait]
pub trait MessageListener: Send + Unpin {
    /// Accept a new connection
    async fn accept(&mut self) -> NetworkResult<(Box<dyn MessageStream>, SocketAddr)>;
    
    /// Get the local address
    fn local_addr(&self) -> NetworkResult<SocketAddr>;
    
    /// Close the listener
    async fn close(&mut self) -> NetworkResult<()>;
}

/// Abstract trait for establishing connections
#[async_trait]
pub trait NetworkConnector: Send + Sync {
    /// Connect to a remote address
    async fn connect(&self, addr: SocketAddr) -> NetworkResult<Box<dyn MessageStream>>;
} 