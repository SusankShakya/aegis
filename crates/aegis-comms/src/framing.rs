use crate::transport::{MessageStream, NetworkError};
use bytes::{Bytes, BytesMut, BufMut};
use byteorder::{BigEndian, ByteOrder};
use std::net::SocketAddr;

const MAX_MESSAGE_SIZE: u32 = 16 * 1024 * 1024; // 16MB
const LENGTH_PREFIX_SIZE: usize = 4;

/// Errors specific to message framing
#[derive(Debug)]
pub enum FramingError {
    /// Error from underlying transport
    Transport(NetworkError),
    /// Message exceeds maximum allowed size
    MessageTooLarge(u32),
    /// Invalid length prefix
    InvalidLengthPrefix,
    /// Incomplete message received
    IncompleteMessage,
}

impl From<NetworkError> for FramingError {
    fn from(err: NetworkError) -> Self {
        FramingError::Transport(err)
    }
}

/// A wrapper around MessageStream that handles length-prefixed framing
pub struct FramedMessageStream<S: MessageStream> {
    inner: S,
    read_buffer: BytesMut,
}

impl<S: MessageStream> FramedMessageStream<S> {
    /// Create a new framed stream
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            read_buffer: BytesMut::with_capacity(8192),
        }
    }
    
    /// Read a complete framed message
    pub async fn read_framed_message(&mut self) -> Result<Option<Bytes>, FramingError> {
        loop {
            // Check if we have a complete message in the buffer
            if self.read_buffer.len() >= LENGTH_PREFIX_SIZE {
                let message_len = BigEndian::read_u32(&self.read_buffer[..LENGTH_PREFIX_SIZE]) as usize;
                
                if message_len > MAX_MESSAGE_SIZE as usize {
                    return Err(FramingError::MessageTooLarge(message_len as u32));
                }
                
                let total_len = LENGTH_PREFIX_SIZE + message_len;
                
                if self.read_buffer.len() >= total_len {
                    // We have a complete message
                    let _ = self.read_buffer.split_to(LENGTH_PREFIX_SIZE);
                    let message = self.read_buffer.split_to(message_len).freeze();
                    return Ok(Some(message));
                }
            }
            
            // Need more data
            match self.inner.read_message().await? {
                Some(data) => {
                    self.read_buffer.extend_from_slice(&data);
                }
                None => {
                    // Connection closed
                    if !self.read_buffer.is_empty() {
                        return Err(FramingError::IncompleteMessage);
                    }
                    return Ok(None);
                }
            }
        }
    }
    
    /// Write a framed message
    pub async fn write_framed_message(&mut self, msg: Bytes) -> Result<(), FramingError> {
        if msg.len() > MAX_MESSAGE_SIZE as usize {
            return Err(FramingError::MessageTooLarge(msg.len() as u32));
        }
        
        let mut length_prefix = [0u8; LENGTH_PREFIX_SIZE];
        BigEndian::write_u32(&mut length_prefix, msg.len() as u32);
        
        self.inner.write_message(Bytes::copy_from_slice(&length_prefix)).await?;
        self.inner.write_message(msg).await?;
        
        Ok(())
    }
    
    /// Get the peer address from the underlying stream
    pub fn peer_addr(&self) -> Result<SocketAddr, NetworkError> {
        self.inner.peer_addr()
    }
    
    /// Shutdown the underlying stream
    pub async fn shutdown(&mut self) -> Result<(), NetworkError> {
        self.inner.shutdown().await
    }
} 