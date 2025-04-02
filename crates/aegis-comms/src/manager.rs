//! High-level communications manager
//!
//! This module provides a higher-level API for communication between agents,
//! abstracting the details of transport, framing, and serialization.

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use bytes::Bytes;
use serde::{Serialize, de::DeserializeOwned};
use bincode::{serialize, deserialize};
use futures::Future;
use tokio::sync::{mpsc, oneshot, Mutex};
use std::collections::HashMap;

use crate::framing::{FramingError, StreamWrapper, wrap_boxed_stream};
use crate::transport::{MessageListener, NetworkConnector, NetworkError, NetworkResult};
use crate::protocol::Message;

/// Error types for the communications API
#[derive(Debug)]
pub enum CommsError {
    /// Network error
    Network(NetworkError),
    /// Framing error
    Framing(FramingError),
    /// Serialization error
    Serialization(String),
    /// Deserialization error
    Deserialization(String),
    /// Connection closed
    ConnectionClosed,
    /// Channel closed
    ChannelClosed,
    /// Timeout
    Timeout,
    /// Invalid message type
    InvalidMessageType,
    /// Other error
    Other(String),
}

impl fmt::Display for CommsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommsError::Network(e) => write!(f, "Network error: {}", e),
            CommsError::Framing(e) => write!(f, "Framing error: {}", e),
            CommsError::Serialization(e) => write!(f, "Serialization error: {}", e),
            CommsError::Deserialization(e) => write!(f, "Deserialization error: {}", e),
            CommsError::ConnectionClosed => write!(f, "Connection closed"),
            CommsError::ChannelClosed => write!(f, "Channel closed"),
            CommsError::Timeout => write!(f, "Operation timed out"),
            CommsError::InvalidMessageType => write!(f, "Invalid message type"),
            CommsError::Other(e) => write!(f, "Communications error: {}", e),
        }
    }
}

impl From<NetworkError> for CommsError {
    fn from(err: NetworkError) -> Self {
        CommsError::Network(err)
    }
}

impl From<FramingError> for CommsError {
    fn from(err: FramingError) -> Self {
        CommsError::Framing(err)
    }
}

impl From<bincode::Error> for CommsError {
    fn from(err: bincode::Error) -> Self {
        CommsError::Deserialization(err.to_string())
    }
}

/// Result type for communications operations
pub type CommsResult<T> = Result<T, CommsError>;

/// Handle to a connection for sending/receiving typed messages
pub struct ConnectionHandle<T> where T: Serialize + DeserializeOwned + Send + 'static {
    /// Channel for sending outgoing messages
    tx: mpsc::Sender<T>,
    /// Channel for receiving incoming messages
    rx: mpsc::Receiver<T>,
    /// Peer address
    peer_addr: SocketAddr,
    /// Connection closer
    closer: Option<oneshot::Sender<()>>,
}

impl<T> ConnectionHandle<T> where T: Serialize + DeserializeOwned + Send + 'static {
    /// Create a new connection handle
    fn new(
        tx: mpsc::Sender<T>,
        rx: mpsc::Receiver<T>,
        peer_addr: SocketAddr,
        closer: oneshot::Sender<()>,
    ) -> Self {
        Self {
            tx,
            rx,
            peer_addr,
            closer: Some(closer),
        }
    }

    /// Send a message to the connected peer
    pub async fn send(&self, message: T) -> CommsResult<()> {
        self.tx.send(message).await.map_err(|_| CommsError::ChannelClosed)
    }

    /// Receive a message from the connected peer
    pub async fn receive(&mut self) -> CommsResult<Option<T>> {
        match self.rx.recv().await {
            Some(msg) => Ok(Some(msg)),
            None => Ok(None),
        }
    }

    /// Get the peer address
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    /// Close the connection
    pub async fn close(&mut self) -> CommsResult<()> {
        if let Some(closer) = self.closer.take() {
            let _ = closer.send(());
        }
        self.tx.closed().await;
        Ok(())
    }
}

impl<T> Drop for ConnectionHandle<T>
where T: Serialize + DeserializeOwned + Send + 'static
{
    fn drop(&mut self) {
        if let Some(closer) = self.closer.take() {
            let _ = closer.send(());
        }
    }
}

/// High-level communications client
pub struct CommsClient {
    /// Network connector for establishing connections
    connector: Arc<dyn NetworkConnector + Send + Sync>,
    /// Active listeners and their handlers
    listeners: Arc<Mutex<HashMap<SocketAddr, oneshot::Sender<()>>>>,
}

impl CommsClient {
    /// Create a new communications client
    pub fn new(connector: Arc<dyn NetworkConnector + Send + Sync>) -> Self {
        Self {
            connector,
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Connect to a remote peer and return a handle for sending/receiving typed messages
    pub async fn connect_to<T>(&self, addr: SocketAddr) -> CommsResult<ConnectionHandle<T>>
    where
        T: Serialize + DeserializeOwned + Send + 'static
    {
        // Establish the raw connection and wrap with framing
        let stream = self.connector.connect(addr).await?;
        let framed_stream = wrap_boxed_stream(stream);
        
        // Create channels for communication
        let (tx_user, mut rx_network) = mpsc::channel::<T>(32);
        let (tx_network, rx_user) = mpsc::channel::<T>(32);
        let (closer_tx, mut closer_rx) = oneshot::channel::<()>();
        
        // Spawn a task to handle the connection
        let stream_task = {
            let tx_network = tx_network.clone();
            
            async move {
                let mut framed_stream = framed_stream;
                
                // Process incoming and outgoing messages
                loop {
                    tokio::select! {
                        // Handle incoming messages from the network
                        maybe_msg = framed_stream.read_framed_message() => {
                            match maybe_msg {
                                Ok(Some(bytes)) => {
                                    // Deserialize the message
                                    match deserialize::<T>(&bytes) {
                                        Ok(msg) => {
                                            if tx_network.send(msg).await.is_err() {
                                                // User channel closed, stop processing
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            // Log deserialization error but continue
                                            eprintln!("Deserialization error: {}", e);
                                        }
                                    }
                                }
                                Ok(None) => {
                                    // Connection closed cleanly
                                    break;
                                }
                                Err(e) => {
                                    // Error reading from network
                                    eprintln!("Error reading from network: {}", e);
                                    break;
                                }
                            }
                        }
                        
                        // Handle outgoing messages to the network
                        maybe_msg = rx_network.recv() => {
                            match maybe_msg {
                                Some(msg) => {
                                    // Serialize and send the message
                                    match serialize(&msg) {
                                        Ok(bytes) => {
                                            if let Err(e) = framed_stream.write_framed_message(Bytes::from(bytes)).await {
                                                eprintln!("Error writing to network: {}", e);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Serialization error: {}", e);
                                        }
                                    }
                                }
                                None => {
                                    // User channel closed
                                    break;
                                }
                            }
                        }
                        
                        // Handle graceful shutdown request
                        _ = &mut closer_rx => {
                            break;
                        }
                    }
                }
                
                // Shut down the connection
                let _ = framed_stream.shutdown().await;
                
                // Signal to the user that the connection is closed
                drop(tx_network);
            }
        };
        
        // Spawn the connection handler
        tokio::spawn(stream_task);
        
        // Return the handle to the user
        Ok(ConnectionHandle::new(tx_user, rx_user, addr, closer_tx))
    }

    /// Start a listener for incoming connections
    pub async fn start_listener<T, F, Fut>(
        &self,
        addr: SocketAddr,
        handler: F,
    ) -> CommsResult<SocketAddr>
    where
        T: Serialize + DeserializeOwned + Send + 'static,
        F: Fn(ConnectionHandle<T>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        // Create the listener
        let listener = self.create_listener(addr).await?;
        let local_addr = listener.local_addr()?;
        
        // Create a channel to signal shutdown
        let (closer_tx, mut closer_rx) = oneshot::channel::<()>();
        
        // Store the closer
        {
            let mut listeners = self.listeners.lock().await;
            listeners.insert(local_addr, closer_tx);
        }

        // Spawn a task to handle the listener
        let listeners = self.listeners.clone();
        tokio::spawn(async move {
            // Accept connections until shutdown is requested or error occurs
            let mut listener = listener;
            
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, peer_addr)) => {
                                // Set up framed stream and channels
                                let framed_stream = wrap_boxed_stream(stream);
                                let (tx_user, mut rx_network) = mpsc::channel::<T>(32);
                                let (tx_network, rx_user) = mpsc::channel::<T>(32);
                                let (conn_closer_tx, mut conn_closer_rx) = oneshot::channel::<()>();
                                
                                // Create the connection handle
                                let handle = ConnectionHandle::new(tx_user, rx_user, peer_addr, conn_closer_tx);
                                
                                // Spawn a task to handle this connection's I/O
                                tokio::spawn({
                                    let tx_network = tx_network.clone();
                                    
                                    async move {
                                        let mut framed_stream = framed_stream;
                                        
                                        loop {
                                            tokio::select! {
                                                // Handle incoming messages from the network
                                                maybe_msg = framed_stream.read_framed_message() => {
                                                    match maybe_msg {
                                                        Ok(Some(bytes)) => {
                                                            // Deserialize and forward to user
                                                            match deserialize::<T>(&bytes) {
                                                                Ok(msg) => {
                                                                    if tx_network.send(msg).await.is_err() {
                                                                        break;
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    eprintln!("Deserialization error: {}", e);
                                                                }
                                                            }
                                                        }
                                                        Ok(None) => {
                                                            // Connection closed cleanly
                                                            break;
                                                        }
                                                        Err(e) => {
                                                            // Error reading from network
                                                            eprintln!("Error reading from network: {}", e);
                                                            break;
                                                        }
                                                    }
                                                }
                                                
                                                // Handle outgoing messages to the network
                                                maybe_msg = rx_network.recv() => {
                                                    match maybe_msg {
                                                        Some(msg) => {
                                                            // Serialize and send
                                                            match serialize(&msg) {
                                                                Ok(bytes) => {
                                                                    if let Err(e) = framed_stream.write_framed_message(Bytes::from(bytes)).await {
                                                                        eprintln!("Error writing to network: {}", e);
                                                                        break;
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    eprintln!("Serialization error: {}", e);
                                                                }
                                                            }
                                                        }
                                                        None => {
                                                            // User channel closed
                                                            break;
                                                        }
                                                    }
                                                }
                                                
                                                // Handle connection close request
                                                _ = &mut conn_closer_rx => {
                                                    break;
                                                }
                                            }
                                        }
                                        
                                        // Shut down the connection
                                        let _ = framed_stream.shutdown().await;
                                        drop(tx_network);
                                    }
                                });
                                
                                // Call the handler with the connection handle
                                tokio::spawn(handler(handle));
                            }
                            Err(e) => {
                                eprintln!("Error accepting connection: {}", e);
                                // Continue accepting connections despite errors
                            }
                        }
                    }
                    
                    // Handle shutdown request
                    _ = &mut closer_rx => {
                        break;
                    }
                }
            }
            
            // Close the listener
            let _ = listener.close().await;
            
            // Remove from active listeners
            let mut listeners = listeners.lock().await;
            listeners.remove(&local_addr);
        });
        
        Ok(local_addr)
    }

    /// Stop a listener
    pub async fn stop_listener(&self, addr: SocketAddr) -> CommsResult<()> {
        let mut listeners = self.listeners.lock().await;
        if let Some(closer) = listeners.remove(&addr) {
            let _ = closer.send(());
            Ok(())
        } else {
            Err(CommsError::Other(format!("No listener at address: {}", addr)))
        }
    }

    /// Create a platform-specific listener
    async fn create_listener(&self, addr: SocketAddr) -> CommsResult<Box<dyn MessageListener>> {
        // This would typically dispatch based on enabled features
        #[cfg(feature = "platform_tokio_net")]
        {
            use crate::platform::tokio_impl::TokioTcpListener;
            let listener = TokioTcpListener::bind(addr).await?;
            Ok(Box::new(listener))
        }
        
        #[cfg(not(feature = "platform_tokio_net"))]
        {
            Err(CommsError::Other("No platform implementation available".to_string()))
        }
    }
}