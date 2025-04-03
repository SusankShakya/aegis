use crate::framing::{FramedMessageStream, FramingError};
use crate::transport::{MessageListener, MessageStream, NetworkConnector, NetworkError};
use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Errors that can occur in the comms system
#[derive(Debug)]
pub enum CommsError {
    Network(NetworkError),
    Framing(FramingError),
    Serialization(bincode::Error),
    ChannelClosed,
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
        CommsError::Serialization(err)
    }
}

/// Handle for sending/receiving messages on a connection
pub struct ConnectionHandle<T> {
    tx: mpsc::Sender<T>,
    rx: mpsc::Receiver<T>,
    _type: std::marker::PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned + Send + 'static> ConnectionHandle<T> {
    /// Send a message through the connection
    pub async fn send(&self, msg: T) -> Result<(), CommsError> {
        self.tx.send(msg).await.map_err(|_| CommsError::ChannelClosed)
    }
    
    /// Receive a message from the connection
    pub async fn receive(&mut self) -> Result<Option<T>, CommsError> {
        self.rx.recv().await.ok_or(CommsError::ChannelClosed).map(Some)
    }
}

/// High-level communications client
pub struct CommsClient {
    connector: Arc<dyn NetworkConnector>,
}

impl CommsClient {
    /// Create a new comms client with the given connector
    pub fn new(connector: impl NetworkConnector + 'static) -> Self {
        Self {
            connector: Arc::new(connector),
        }
    }
    
    /// Connect to a remote address and get a typed connection handle
    pub async fn connect_to<T: Serialize + DeserializeOwned + Send + 'static>(
        &self,
        addr: SocketAddr,
    ) -> Result<ConnectionHandle<T>, CommsError> {
        let stream = self.connector.connect(addr).await?;
        let framed = FramedMessageStream::new(stream);
        
        let (tx_raw, mut rx_raw) = mpsc::channel::<T>(32);
        let (tx_processed, rx_processed) = mpsc::channel::<T>(32);
        
        // Spawn send task
        let mut framed_clone = framed;
        tokio::spawn(async move {
            while let Some(msg) = rx_raw.recv().await {
                let bytes = Bytes::from(bincode::serialize(&msg).unwrap());
                if framed_clone.write_framed_message(bytes).await.is_err() {
                    break;
                }
            }
        });
        
        // Spawn receive task
        let mut framed_clone = framed;
        tokio::spawn(async move {
            while let Ok(Some(bytes)) = framed_clone.read_framed_message().await {
                if let Ok(msg) = bincode::deserialize::<T>(&bytes) {
                    if tx_processed.send(msg).await.is_err() {
                        break;
                    }
                }
            }
        });
        
        Ok(ConnectionHandle {
            tx: tx_raw,
            rx: rx_processed,
            _type: std::marker::PhantomData,
        })
    }
    
    /// Start a listener for incoming connections
    pub async fn start_listener<T, F, Fut>(
        &self,
        addr: SocketAddr,
        listener: impl MessageListener,
        mut handler: F,
    ) -> Result<(), CommsError>
    where
        T: Serialize + DeserializeOwned + Send + 'static,
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let mut listener = listener;
        
        loop {
            let (stream, _addr) = listener.accept().await?;
            let framed = FramedMessageStream::new(stream);
            
            tokio::spawn(async move {
                let mut framed = framed;
                while let Ok(Some(bytes)) = framed.read_framed_message().await {
                    if let Ok(msg) = bincode::deserialize::<T>(&bytes) {
                        handler(msg).await;
                    }
                }
            });
        }
    }
} 