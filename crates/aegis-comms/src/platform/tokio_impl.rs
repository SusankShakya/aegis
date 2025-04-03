use crate::transport::{MessageStream, MessageListener, NetworkConnector, NetworkError};
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, TcpListener};

pub struct TokioTcpStream(TcpStream);

#[async_trait]
impl MessageStream for TokioTcpStream {
    async fn read_message(&mut self) -> Result<Option<Bytes>, NetworkError> {
        let mut buf = BytesMut::with_capacity(8192);
        match self.0.read_buf(&mut buf).await {
            Ok(0) => Ok(None), // Connection closed
            Ok(_) => Ok(Some(buf.freeze())),
            Err(e) => Err(NetworkError::IoError(e)),
        }
    }
    
    async fn write_message(&mut self, msg: Bytes) -> Result<(), NetworkError> {
        self.0.write_all(&msg).await.map_err(NetworkError::IoError)
    }
    
    fn peer_addr(&self) -> Result<SocketAddr, NetworkError> {
        self.0.peer_addr().map_err(NetworkError::IoError)
    }
    
    async fn shutdown(&mut self) -> Result<(), NetworkError> {
        self.0.shutdown().await.map_err(NetworkError::IoError)
    }
}

pub struct TokioTcpListener(TcpListener);

impl TokioTcpListener {
    pub async fn bind(addr: SocketAddr) -> Result<Self, NetworkError> {
        TcpListener::bind(addr)
            .await
            .map(TokioTcpListener)
            .map_err(NetworkError::IoError)
    }
}

#[async_trait]
impl MessageListener for TokioTcpListener {
    async fn accept(&mut self) -> Result<(Box<dyn MessageStream>, SocketAddr), NetworkError> {
        let (stream, addr) = self.0.accept().await.map_err(NetworkError::IoError)?;
        Ok((Box::new(TokioTcpStream(stream)), addr))
    }
    
    fn local_addr(&self) -> Result<SocketAddr, NetworkError> {
        self.0.local_addr().map_err(NetworkError::IoError)
    }
}

#[derive(Clone)]
pub struct TokioConnector;

impl TokioConnector {
    pub fn new() -> Self {
        TokioConnector
    }
}

#[async_trait]
impl NetworkConnector for TokioConnector {
    async fn connect(&self, addr: SocketAddr) -> Result<Box<dyn MessageStream>, NetworkError> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::ConnectionRefused => NetworkError::ConnectionRefused,
                std::io::ErrorKind::TimedOut => NetworkError::Timeout,
                _ => NetworkError::IoError(e),
            })?;
        Ok(Box::new(TokioTcpStream(stream)))
    }
}

// Helper functions to create platform-specific instances
pub async fn listen_tokio(addr: SocketAddr) -> Result<TokioTcpListener, NetworkError> {
    TokioTcpListener::bind(addr).await
}

pub fn connect_tokio() -> TokioConnector {
    TokioConnector::new()
} 