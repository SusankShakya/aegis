use aegis_comms::{
    AgentDiscoveryMessage, CommsClient, MessageHeader, MessageStream, MessageType,
    NetworkConnector, NetworkError, PROTOCOL_VERSION,
};
use async_trait::async_trait;
use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;

// Mock connector for testing
struct MockConnector {
    stream_tx: mpsc::UnboundedSender<(MockStream, SocketAddr)>,
    stream_rx: mpsc::UnboundedReceiver<(MockStream, SocketAddr)>,
}

impl MockConnector {
    fn new() -> (Self, mpsc::UnboundedSender<(MockStream, SocketAddr)>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (MockConnector {
            stream_tx: tx.clone(),
            stream_rx: rx,
        }, tx)
    }
}

#[async_trait]
impl NetworkConnector for MockConnector {
    async fn connect(&self, addr: SocketAddr) -> Result<Box<dyn MessageStream>, NetworkError> {
        let (stream, _) = self.stream_rx.recv().await.ok_or(NetworkError::ConnectionRefused)?;
        Ok(Box::new(stream))
    }
}

// Mock stream for testing
struct MockStream {
    read_tx: mpsc::UnboundedSender<Bytes>,
    read_rx: mpsc::UnboundedReceiver<Bytes>,
    write_tx: mpsc::UnboundedSender<Bytes>,
    write_rx: mpsc::UnboundedReceiver<Bytes>,
    peer_addr: SocketAddr,
}

impl MockStream {
    fn new(addr: SocketAddr) -> (Self, mpsc::UnboundedSender<Bytes>, mpsc::UnboundedReceiver<Bytes>) {
        let (read_tx, read_rx) = mpsc::unbounded_channel();
        let (write_tx, write_rx) = mpsc::unbounded_channel();
        let stream = MockStream {
            read_tx: read_tx.clone(),
            read_rx,
            write_tx,
            write_rx,
            peer_addr: addr,
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
        Ok(self.peer_addr)
    }
    
    async fn shutdown(&mut self) -> Result<(), NetworkError> {
        Ok(())
    }
}

#[tokio::test]
async fn test_comms_client_send_receive() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    
    // Set up mock connector
    let (connector, stream_provider) = MockConnector::new();
    let client = CommsClient::new(connector);
    
    // Create mock stream
    let (mock_stream, read_tx, mut write_rx) = MockStream::new(addr);
    
    // When client tries to connect, provide the mock stream
    stream_provider.send((mock_stream, addr)).unwrap();
    
    // Connect and get a typed handle
    let mut handle = client.connect_to::<AgentDiscoveryMessage>(addr).await.unwrap();
    
    // Create a test message
    let message = AgentDiscoveryMessage {
        header: MessageHeader {
            version: PROTOCOL_VERSION,
            message_type: MessageType::AgentDiscovery,
            source: Some(addr),
            destination: None,
        },
        agent_id: "test_agent".to_string(),
        capabilities: vec!["test".to_string()],
        listen_addr: addr,
    };
    
    // Send the message
    handle.send(message.clone()).await.unwrap();
    
    // Collect the framed message from the mock stream
    let mut received_bytes = Vec::new();
    while let Some(bytes) = write_rx.recv().await {
        received_bytes.extend_from_slice(&bytes);
        if received_bytes.len() >= 4 {
            let len = u32::from_be_bytes(received_bytes[0..4].try_into().unwrap()) as usize;
            if received_bytes.len() >= len + 4 {
                break;
            }
        }
    }
    
    // Send it back through the mock stream
    read_tx.send(Bytes::from(received_bytes)).unwrap();
    
    // Receive and verify the message
    let received = handle.receive().await.unwrap().unwrap();
    assert_eq!(received.agent_id, message.agent_id);
    assert_eq!(received.capabilities, message.capabilities);
    assert_eq!(received.listen_addr, message.listen_addr);
} 