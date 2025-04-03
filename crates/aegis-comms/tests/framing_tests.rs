use aegis_comms::{FramedMessageStream, MessageStream, NetworkError};
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use std::net::SocketAddr;
use tokio::sync::mpsc;

// Mock stream for testing
struct MockStream {
    read_tx: mpsc::UnboundedSender<Bytes>,
    read_rx: mpsc::UnboundedReceiver<Bytes>,
    write_tx: mpsc::UnboundedSender<Bytes>,
    write_rx: mpsc::UnboundedReceiver<Bytes>,
}

impl MockStream {
    fn new() -> (Self, mpsc::UnboundedSender<Bytes>, mpsc::UnboundedReceiver<Bytes>) {
        let (read_tx, read_rx) = mpsc::unbounded_channel();
        let (write_tx, write_rx) = mpsc::unbounded_channel();
        let stream = MockStream {
            read_tx: read_tx.clone(),
            read_rx,
            write_tx,
            write_rx,
        };
        (stream, read_tx, write_rx)
    }
}

#[async_trait]
impl MessageStream for MockStream {
    async fn read_message(&mut self) -> Result<Option<Bytes>, NetworkError> {
        match self.read_rx.recv().await {
            Some(bytes) => Ok(Some(bytes)),
            None => Ok(None),
        }
    }
    
    async fn write_message(&mut self, msg: Bytes) -> Result<(), NetworkError> {
        self.write_tx.send(msg).map_err(|_| NetworkError::ConnectionClosed)
    }
    
    fn peer_addr(&self) -> Result<SocketAddr, NetworkError> {
        Ok("127.0.0.1:8080".parse().unwrap())
    }
    
    async fn shutdown(&mut self) -> Result<(), NetworkError> {
        Ok(())
    }
}

#[tokio::test]
async fn test_framing_single_message() {
    let (mock_stream, read_tx, mut write_rx) = MockStream::new();
    let mut framed = FramedMessageStream::new(mock_stream);
    
    // Write a message
    let message = Bytes::from_static(b"Hello, World!");
    framed.write_framed_message(message.clone()).await.unwrap();
    
    // Verify the written message format
    let written = write_rx.recv().await.unwrap();
    assert_eq!(&written[..4], &(message.len() as u32).to_be_bytes());
    
    let written_body = write_rx.recv().await.unwrap();
    assert_eq!(written_body, message);
    
    // Read the message back
    read_tx.send(written).unwrap();
    read_tx.send(written_body).unwrap();
    
    let read_message = framed.read_framed_message().await.unwrap().unwrap();
    assert_eq!(read_message, message);
}

#[tokio::test]
async fn test_framing_empty_message() {
    let (mock_stream, read_tx, mut write_rx) = MockStream::new();
    let mut framed = FramedMessageStream::new(mock_stream);
    
    // Write an empty message
    let message = Bytes::new();
    framed.write_framed_message(message).await.unwrap();
    
    // Verify the written message format
    let written = write_rx.recv().await.unwrap();
    assert_eq!(&written[..4], &[0, 0, 0, 0]); // Length prefix of 0
    
    // Read the message back
    read_tx.send(written).unwrap();
    
    let read_message = framed.read_framed_message().await.unwrap().unwrap();
    assert!(read_message.is_empty());
}

#[tokio::test]
async fn test_framing_multiple_messages() {
    let (mock_stream, read_tx, mut write_rx) = MockStream::new();
    let mut framed = FramedMessageStream::new(mock_stream);
    
    let messages = vec![
        Bytes::from_static(b"First"),
        Bytes::from_static(b"Second"),
        Bytes::from_static(b"Third"),
    ];
    
    // Write all messages
    for msg in messages.iter() {
        framed.write_framed_message(msg.clone()).await.unwrap();
    }
    
    // Collect written frames
    let mut written_frames = Vec::new();
    for _ in 0..messages.len() * 2 {
        written_frames.push(write_rx.recv().await.unwrap());
    }
    
    // Send them back in one batch
    let mut combined = BytesMut::new();
    for frame in written_frames {
        combined.extend_from_slice(&frame);
    }
    read_tx.send(combined.freeze()).unwrap();
    
    // Read them back
    for expected in messages {
        let read = framed.read_framed_message().await.unwrap().unwrap();
        assert_eq!(read, expected);
    }
}

#[tokio::test]
async fn test_framing_partial_message() {
    let (mock_stream, read_tx, _write_rx) = MockStream::new();
    let mut framed = FramedMessageStream::new(mock_stream);
    
    // Send just the length prefix
    read_tx.send(Bytes::from_static(&[0, 0, 0, 4])).unwrap();
    
    // No complete message yet
    tokio::select! {
        _ = framed.read_framed_message() => {
            panic!("Should not complete until full message is received");
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
            // Expected timeout
        }
    }
    
    // Send the message body
    read_tx.send(Bytes::from_static(b"test")).unwrap();
    
    // Now we should get the complete message
    let read = framed.read_framed_message().await.unwrap().unwrap();
    assert_eq!(read, Bytes::from_static(b"test"));
} 