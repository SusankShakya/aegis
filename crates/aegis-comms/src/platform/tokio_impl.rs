//! Tokio-based implementation of network abstractions
//!
//! This module provides implementations of the network abstractions
//! using Tokio for standard platforms (Linux, macOS, Windows).

#![cfg(feature = "platform_tokio_net")]

use std::net::SocketAddr;
use std::sync::Arc;
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::transport::{MessageStream, MessageListener, NetworkConnector, NetworkError, NetworkResult};

/// Tokio TCP stream implementation of MessageStream
pub struct TokioTcpStream {
    /// Underlying Tokio TCP stream
    stream: TcpStream,
    /// Read buffer
    read_buffer: BytesMut,
}

impl TokioTcpStream {
    /// Create a new TokioTcpStream from a Tokio TcpStream
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            read_buffer: BytesMut::with_capacity(8192), // 8KB initial buffer
        }
    }

    /// Get a reference to the underlying Tokio TcpStream
    pub fn get_ref(&self) -> &TcpStream {
        &self.stream
    }

    /// Get a mutable reference to the underlying Tokio TcpStream
    pub fn get_mut(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    /// Consume the TokioTcpStream and return the underlying Tokio TcpStream
    pub fn into_inner(self) -> TcpStream {
        self.stream
    }
}

#[async_trait]
impl MessageStream for TokioTcpStream {
    async fn read_bytes(&mut self, buf: &mut [u8]) -> NetworkResult<usize> {
        match self.stream.read(buf).await {
            Ok(0) => Err(NetworkError::ConnectionClosed),
            Ok(n) => Ok(n),
            Err(e) => Err(e.into()),
        }
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> NetworkResult<()> {
        match self.stream.read_exact(buf).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                Err(NetworkError::ConnectionClosed)
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn read_message(&mut self) -> NetworkResult<Option<Bytes>> {
        // Read into the buffer
        let mut temp_buf = [0u8; 4096];
        match self.stream.read(&mut temp_buf).await {
            Ok(0) => return Ok(None), // Connection closed
            Ok(n) => {
                self.read_buffer.extend_from_slice(&temp_buf[..n]);
            }
            Err(e) => return Err(e.into()),
        }

        // Return the entire buffer as a message
        // Note: This is a basic implementation. The FramedMessageStream will provide proper framing.
        let bytes = self.read_buffer.split().freeze();
        Ok(Some(bytes))
    }

    async fn write_bytes(&mut self, buf: &[u8]) -> NetworkResult<usize> {
        match self.stream.write(buf).await {
            Ok(n) => Ok(n),
            Err(e) => Err(e.into()),
        }
    }

    async fn write_all(&mut self, buf: &[u8]) -> NetworkResult<()> {
        match self.stream.write_all(buf).await {
            Ok(()) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn write_message(&mut self, msg: Bytes) -> NetworkResult<()> {
        self.write_all(&msg).await
    }

    fn peer_addr(&self) -> NetworkResult<SocketAddr> {
        match self.stream.peer_addr() {
            Ok(addr) => Ok(addr),
            Err(e) => Err(e.into()),
        }
    }

    async fn shutdown(&mut self) -> NetworkResult<()> {
        match self.stream.shutdown().await {
            Ok(()) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

/// Tokio TCP listener implementation of MessageListener
pub struct TokioTcpListener {
    /// Underlying Tokio TCP listener
    listener: TcpListener,
}

impl TokioTcpListener {
    /// Create a new TokioTcpListener
    pub async fn bind(addr: SocketAddr) -> NetworkResult<Self> {
        match TcpListener::bind(addr).await {
            Ok(listener) => Ok(Self { listener }),
            Err(e) => Err(e.into()),
        }
    }

    /// Get a reference to the underlying Tokio TcpListener
    pub fn get_ref(&self) -> &TcpListener {
        &self.listener
    }

    /// Get a mutable reference to the underlying Tokio TcpListener
    pub fn get_mut(&mut self) -> &mut TcpListener {
        &mut self.listener
    }

    /// Consume the TokioTcpListener and return the underlying Tokio TcpListener
    pub fn into_inner(self) -> TcpListener {
        self.listener
    }
}

#[async_trait]
impl MessageListener for TokioTcpListener {
    async fn accept(&mut self) -> NetworkResult<(Box<dyn MessageStream>, SocketAddr)> {
        match self.listener.accept().await {
            Ok((stream, addr)) => {
                let stream = TokioTcpStream::new(stream);
                Ok((Box::new(stream), addr))
            }
            Err(e) => Err(e.into()),
        }
    }

    fn local_addr(&self) -> NetworkResult<SocketAddr> {
        match self.listener.local_addr() {
            Ok(addr) => Ok(addr),
            Err(e) => Err(e.into()),
        }
    }

    async fn close(&mut self) -> NetworkResult<()> {
        // TcpListener doesn't have a close method in Tokio,
        // but it will be closed when dropped
        Ok(())
    }
}

/// Tokio-based network connector
#[derive(Clone)]
pub struct TokioConnector {
    // Optional configuration can be added here
}

impl TokioConnector {
    /// Create a new TokioConnector
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TokioConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NetworkConnector for TokioConnector {
    async fn connect(&self, addr: SocketAddr) -> NetworkResult<Box<dyn MessageStream>> {
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                let stream = TokioTcpStream::new(stream);
                Ok(Box::new(stream))
            }
            Err(e) => Err(e.into()),
        }
    }
}

/// Create a new Tokio-based message listener bound to the specified address
pub async fn listen_tokio(addr: SocketAddr) -> NetworkResult<TokioTcpListener> {
    TokioTcpListener::bind(addr).await
}

/// Connect to a remote address using Tokio
pub async fn connect_tokio(addr: SocketAddr) -> NetworkResult<Box<dyn MessageStream>> {
    let connector = TokioConnector::new();
    connector.connect(addr).await
}