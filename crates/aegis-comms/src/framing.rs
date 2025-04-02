//! Message framing implementation
//!
//! This module provides length-prefixed framing for message streams,
//! allowing complete messages to be read/written over a byte stream.

use std::fmt;
use std::io::{Cursor, Read};
use std::mem::size_of;
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use byteorder::{BigEndian, ByteOrder, ReadBytesExt};

use crate::transport::{MessageStream, NetworkError, NetworkResult};

/// Maximum message size (32MB)
pub const MAX_MESSAGE_SIZE: u32 = 32 * 1024 * 1024;

/// Errors that can occur during message framing
#[derive(Debug)]
pub enum FramingError {
    /// Network error
    Network(NetworkError),
    /// Invalid length prefix
    InvalidLengthPrefix,
    /// Message exceeds maximum size
    MessageTooLarge(u32),
    /// Unexpected end of stream
    UnexpectedEof,
    /// Other error
    Other(String),
}

impl fmt::Display for FramingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FramingError::Network(e) => write!(f, "Network error: {}", e),
            FramingError::InvalidLengthPrefix => write!(f, "Invalid length prefix"),
            FramingError::MessageTooLarge(size) => {
                write!(f, "Message too large: {} bytes (max: {})", size, MAX_MESSAGE_SIZE)
            }
            FramingError::UnexpectedEof => write!(f, "Unexpected end of stream"),
            FramingError::Other(e) => write!(f, "Framing error: {}", e),
        }
    }
}

impl From<NetworkError> for FramingError {
    fn from(err: NetworkError) -> Self {
        FramingError::Network(err)
    }
}

/// Result type for framing operations
pub type FramingResult<T> = Result<T, FramingError>;

/// Wraps a MessageStream to provide length-prefixed framing
pub struct FramedMessageStream<S: MessageStream> {
    /// The underlying stream
    stream: S,
    /// Temporary buffer for message length prefix
    length_buffer: [u8; size_of::<u32>()],
}

impl<S: MessageStream> FramedMessageStream<S> {
    /// Create a new framed message stream
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            length_buffer: [0; size_of::<u32>()],
        }
    }

    /// Get a reference to the underlying stream
    pub fn get_ref(&self) -> &S {
        &self.stream
    }

    /// Get a mutable reference to the underlying stream
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.stream
    }

    /// Unwrap the FramedMessageStream, returning the underlying stream
    pub fn into_inner(self) -> S {
        self.stream
    }

    /// Read a framed message with length prefix
    pub async fn read_framed_message(&mut self) -> FramingResult<Option<Bytes>> {
        // Read the length prefix (4 bytes in network byte order / big endian)
        match self.stream.read_exact(&mut self.length_buffer).await {
            Ok(()) => {}
            Err(NetworkError::ConnectionClosed) => return Ok(None), // Clean shutdown
            Err(e) => return Err(FramingError::Network(e)),
        }

        // Parse the length prefix
        let length = match Cursor::new(&self.length_buffer).read_u32::<BigEndian>() {
            Ok(len) => len,
            Err(_) => return Err(FramingError::InvalidLengthPrefix),
        };

        // Check if message is too large
        if length > MAX_MESSAGE_SIZE {
            return Err(FramingError::MessageTooLarge(length));
        }

        // Read the message body
        let mut buffer = BytesMut::with_capacity(length as usize);
        buffer.resize(length as usize, 0);

        match self.stream.read_exact(&mut buffer).await {
            Ok(()) => Ok(Some(buffer.freeze())),
            Err(NetworkError::ConnectionClosed) => Err(FramingError::UnexpectedEof),
            Err(e) => Err(FramingError::Network(e)),
        }
    }

    /// Write a framed message with length prefix
    pub async fn write_framed_message(&mut self, msg: Bytes) -> FramingResult<()> {
        let length = msg.len();
        if length > MAX_MESSAGE_SIZE as usize {
            return Err(FramingError::MessageTooLarge(length as u32));
        }

        // Write the length prefix (4 bytes in network byte order / big endian)
        let mut length_bytes = [0u8; size_of::<u32>()];
        BigEndian::write_u32(&mut length_bytes, length as u32);

        // Write the length prefix and then the message
        self.stream.write_all(&length_bytes).await?;
        self.stream.write_all(&msg).await?;

        Ok(())
    }

    /// Get the peer address
    pub fn peer_addr(&self) -> NetworkResult<std::net::SocketAddr> {
        self.stream.peer_addr()
    }

    /// Shutdown the connection
    pub async fn shutdown(&mut self) -> NetworkResult<()> {
        self.stream.shutdown().await
    }
}

/// Trait for stream wrappers that support framed messaging
#[async_trait]
pub trait StreamWrapper: Send + Sync + 'static {
    /// Read a framed message
    async fn read_framed_message(&mut self) -> FramingResult<Option<Bytes>>;
    
    /// Write a framed message
    async fn write_framed_message(&mut self, msg: Bytes) -> FramingResult<()>;
    
    /// Get the peer address
    fn peer_addr(&self) -> NetworkResult<std::net::SocketAddr>;
    
    /// Shutdown the stream
    async fn shutdown(&mut self) -> NetworkResult<()>;
}

/// Create a new framed message stream from a boxed MessageStream
pub fn wrap_boxed_stream(
    stream: Box<dyn MessageStream>,
) -> Box<dyn StreamWrapper> {
    // We'll use a new trait to handle the wrapped stream
    struct FramedWrapper {
        stream: Box<dyn MessageStream>,
        length_buffer: [u8; size_of::<u32>()],
    }

    impl FramedWrapper {
        fn new(stream: Box<dyn MessageStream>) -> Self {
            Self {
                stream,
                length_buffer: [0; size_of::<u32>()],
            }
        }
    }

    #[async_trait]
    impl StreamWrapper for FramedWrapper {
        async fn read_framed_message(&mut self) -> FramingResult<Option<Bytes>> {
            // Read the length prefix (4 bytes in network byte order / big endian)
            match self.stream.read_exact(&mut self.length_buffer).await {
                Ok(()) => {}
                Err(NetworkError::ConnectionClosed) => return Ok(None), // Clean shutdown
                Err(e) => return Err(FramingError::Network(e)),
            }

            // Parse the length prefix
            let length = match Cursor::new(&self.length_buffer).read_u32::<BigEndian>() {
                Ok(len) => len,
                Err(_) => return Err(FramingError::InvalidLengthPrefix),
            };

            // Check if message is too large
            if length > MAX_MESSAGE_SIZE {
                return Err(FramingError::MessageTooLarge(length));
            }

            // Read the message body
            let mut buffer = BytesMut::with_capacity(length as usize);
            buffer.resize(length as usize, 0);

            match self.stream.read_exact(&mut buffer).await {
                Ok(()) => Ok(Some(buffer.freeze())),
                Err(NetworkError::ConnectionClosed) => Err(FramingError::UnexpectedEof),
                Err(e) => Err(FramingError::Network(e)),
            }
        }

        async fn write_framed_message(&mut self, msg: Bytes) -> FramingResult<()> {
            let length = msg.len();
            if length > MAX_MESSAGE_SIZE as usize {
                return Err(FramingError::MessageTooLarge(length as u32));
            }

            // Write the length prefix (4 bytes in network byte order / big endian)
            let mut length_bytes = [0u8; size_of::<u32>()];
            BigEndian::write_u32(&mut length_bytes, length as u32);

            // Write the length prefix and then the message
            self.stream.write_all(&length_bytes).await?;
            self.stream.write_all(&msg).await?;

            Ok(())
        }

        fn peer_addr(&self) -> NetworkResult<std::net::SocketAddr> {
            self.stream.peer_addr()
        }

        async fn shutdown(&mut self) -> NetworkResult<()> {
            self.stream.shutdown().await
        }
    }

    Box::new(FramedWrapper::new(stream))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use bytes::BufMut;
    use std::pin::Pin;

    // A simple mock implementation of MessageStream for testing
    struct MockMessageStream {
        read_data: Vec<u8>,
        write_data: Vec<u8>,
        read_position: usize,
    }

    impl MockMessageStream {
        fn new(read_data: Vec<u8>) -> Self {
            Self {
                read_data,
                write_data: Vec::new(),
                read_position: 0,
            }
        }

        fn written_data(&self) -> &[u8] {
            &self.write_data
        }
    }

    #[async_trait]
    impl MessageStream for MockMessageStream {
        async fn read_bytes(&mut self, buf: &mut [u8]) -> NetworkResult<usize> {
            if self.read_position >= self.read_data.len() {
                return Err(NetworkError::ConnectionClosed);
            }

            let available = self.read_data.len() - self.read_position;
            let to_read = std::cmp::min(buf.len(), available);
            buf[..to_read].copy_from_slice(&self.read_data[self.read_position..self.read_position + to_read]);
            self.read_position += to_read;
            Ok(to_read)
        }

        async fn read_exact(&mut self, buf: &mut [u8]) -> NetworkResult<()> {
            if self.read_position + buf.len() > self.read_data.len() {
                return Err(NetworkError::ConnectionClosed);
            }

            buf.copy_from_slice(&self.read_data[self.read_position..self.read_position + buf.len()]);
            self.read_position += buf.len();
            Ok(())
        }

        async fn read_message(&mut self) -> NetworkResult<Option<Bytes>> {
            unimplemented!("Not needed for framing tests")
        }

        async fn write_bytes(&mut self, buf: &[u8]) -> NetworkResult<usize> {
            self.write_data.extend_from_slice(buf);
            Ok(buf.len())
        }

        async fn write_all(&mut self, buf: &[u8]) -> NetworkResult<()> {
            self.write_data.extend_from_slice(buf);
            Ok(())
        }

        async fn write_message(&mut self, _msg: Bytes) -> NetworkResult<()> {
            unimplemented!("Not needed for framing tests")
        }

        fn peer_addr(&self) -> NetworkResult<std::net::SocketAddr> {
            unimplemented!("Not needed for framing tests")
        }

        async fn shutdown(&mut self) -> NetworkResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_read_framed_message() {
        // Create a mock stream with a valid framed message
        let message = b"Hello, world!";
        let message_len = message.len() as u32;
        
        let mut buffer = Vec::new();
        buffer.put_u32(message_len);
        buffer.extend_from_slice(message);
        
        let mock_stream = MockMessageStream::new(buffer);
        let mut framed_stream = FramedMessageStream::new(mock_stream);
        
        // Read the message
        let result = framed_stream.read_framed_message().await.unwrap().unwrap();
        assert_eq!(result, Bytes::from_static(message));
    }

    #[tokio::test]
    async fn test_write_framed_message() {
        // Create a mock stream
        let mock_stream = MockMessageStream::new(Vec::new());
        let mut framed_stream = FramedMessageStream::new(mock_stream);
        
        // Write a message
        let message = Bytes::from_static(b"Hello, world!");
        framed_stream.write_framed_message(message.clone()).await.unwrap();
        
        // Verify the output
        let written = framed_stream.get_ref().written_data();
        
        // First 4 bytes should be the length in big endian
        let mut cursor = Cursor::new(&written[..4]);
        let len = cursor.read_u32::<BigEndian>().unwrap();
        assert_eq!(len, message.len() as u32);
        
        // The rest should be the message
        assert_eq!(&written[4..], &message[..]);
    }

    #[tokio::test]
    async fn test_message_too_large() {
        // Create a mock stream
        let mock_stream = MockMessageStream::new(Vec::new());
        let mut framed_stream = FramedMessageStream::new(mock_stream);
        
        // Create a message that exceeds the maximum size
        let message = Bytes::from(vec![0; (MAX_MESSAGE_SIZE + 1) as usize]);
        
        // Write should fail with MessageTooLarge
        let result = framed_stream.write_framed_message(message).await;
        assert!(matches!(result, Err(FramingError::MessageTooLarge(_))));
    }

    #[tokio::test]
    async fn test_invalid_length_prefix() {
        // Create a mock stream with an invalid length prefix
        let mock_stream = MockMessageStream::new(vec![0xFF, 0xFF, 0xFF, 0xFF]);
        let mut framed_stream = FramedMessageStream::new(mock_stream);
        
        // Read should succeed but report the message is too large
        let result = framed_stream.read_framed_message().await;
        assert!(matches!(result, Err(FramingError::MessageTooLarge(_))));
    }
} 