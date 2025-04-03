use aegis_comms::{
    AgentDiscoveryMessage, AgentHeartbeatMessage, AgentStatus, ConsensusVoteRequest,
    ConsensusVoteResponse, ErrorMessage, MessageHeader, MessageType, StateUpdateMessage,
    PROTOCOL_VERSION,
};
use std::net::SocketAddr;

#[test]
fn test_consensus_vote_request_serialization() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let header = MessageHeader {
        version: PROTOCOL_VERSION,
        message_type: MessageType::ConsensusVoteRequest,
        source: Some(addr),
        destination: Some(addr),
    };
    
    let request = ConsensusVoteRequest {
        header,
        proposal_id: 42,
        proposal_data: vec![1, 2, 3, 4],
    };
    
    let serialized = bincode::serialize(&request).unwrap();
    let deserialized: ConsensusVoteRequest = bincode::deserialize(&serialized).unwrap();
    
    assert_eq!(request.proposal_id, deserialized.proposal_id);
    assert_eq!(request.proposal_data, deserialized.proposal_data);
    assert_eq!(request.header.version, deserialized.header.version);
    assert_eq!(request.header.message_type, deserialized.header.message_type);
    assert_eq!(request.header.source, deserialized.header.source);
    assert_eq!(request.header.destination, deserialized.header.destination);
}

#[test]
fn test_consensus_vote_response_serialization() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let header = MessageHeader {
        version: PROTOCOL_VERSION,
        message_type: MessageType::ConsensusVoteResponse,
        source: Some(addr),
        destination: Some(addr),
    };
    
    let response = ConsensusVoteResponse {
        header,
        proposal_id: 42,
        vote: true,
        voter_id: "node1".to_string(),
    };
    
    let serialized = bincode::serialize(&response).unwrap();
    let deserialized: ConsensusVoteResponse = bincode::deserialize(&serialized).unwrap();
    
    assert_eq!(response.proposal_id, deserialized.proposal_id);
    assert_eq!(response.vote, deserialized.vote);
    assert_eq!(response.voter_id, deserialized.voter_id);
}

#[test]
fn test_state_update_message_serialization() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let header = MessageHeader {
        version: PROTOCOL_VERSION,
        message_type: MessageType::StateUpdate,
        source: Some(addr),
        destination: Some(addr),
    };
    
    let update = StateUpdateMessage {
        header,
        state_version: 1,
        state_data: vec![5, 6, 7, 8],
        is_delta: true,
    };
    
    let serialized = bincode::serialize(&update).unwrap();
    let deserialized: StateUpdateMessage = bincode::deserialize(&serialized).unwrap();
    
    assert_eq!(update.state_version, deserialized.state_version);
    assert_eq!(update.state_data, deserialized.state_data);
    assert_eq!(update.is_delta, deserialized.is_delta);
}

#[test]
fn test_agent_discovery_message_serialization() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let header = MessageHeader {
        version: PROTOCOL_VERSION,
        message_type: MessageType::AgentDiscovery,
        source: Some(addr),
        destination: None,
    };
    
    let discovery = AgentDiscoveryMessage {
        header,
        agent_id: "agent1".to_string(),
        capabilities: vec!["compute".to_string(), "storage".to_string()],
        listen_addr: addr,
    };
    
    let serialized = bincode::serialize(&discovery).unwrap();
    let deserialized: AgentDiscoveryMessage = bincode::deserialize(&serialized).unwrap();
    
    assert_eq!(discovery.agent_id, deserialized.agent_id);
    assert_eq!(discovery.capabilities, deserialized.capabilities);
    assert_eq!(discovery.listen_addr, deserialized.listen_addr);
}

#[test]
fn test_agent_heartbeat_message_serialization() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let header = MessageHeader {
        version: PROTOCOL_VERSION,
        message_type: MessageType::AgentHeartbeat,
        source: Some(addr),
        destination: None,
    };
    
    let heartbeat = AgentHeartbeatMessage {
        header,
        agent_id: "agent1".to_string(),
        timestamp: 1234567890,
        status: AgentStatus::Active,
    };
    
    let serialized = bincode::serialize(&heartbeat).unwrap();
    let deserialized: AgentHeartbeatMessage = bincode::deserialize(&serialized).unwrap();
    
    assert_eq!(heartbeat.agent_id, deserialized.agent_id);
    assert_eq!(heartbeat.timestamp, deserialized.timestamp);
    assert_eq!(heartbeat.status, deserialized.status);
}

#[test]
fn test_error_message_serialization() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let header = MessageHeader {
        version: PROTOCOL_VERSION,
        message_type: MessageType::Error,
        source: Some(addr),
        destination: Some(addr),
    };
    
    let error = ErrorMessage {
        header,
        error_code: 404,
        error_message: "Not found".to_string(),
    };
    
    let serialized = bincode::serialize(&error).unwrap();
    let deserialized: ErrorMessage = bincode::deserialize(&serialized).unwrap();
    
    assert_eq!(error.error_code, deserialized.error_code);
    assert_eq!(error.error_message, deserialized.error_message);
} 