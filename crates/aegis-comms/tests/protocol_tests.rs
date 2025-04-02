use aegis_comms::protocol::*;
use bincode::{serialize, deserialize};

#[test]
fn test_message_header_roundtrip() {
    // Create a sample message header
    let agent_id = AgentId { id: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16] };
    let header = MessageHeader {
        version: PROTOCOL_VERSION,
        msg_type: MessageType::Ping,
        msg_id: 12345,
        priority: Priority::Normal,
        timestamp_ms: 1648000000000,
        sender_id: agent_id,
        sequence: 42,
    };

    // Serialize the header
    let serialized = serialize(&header).expect("Failed to serialize header");
    
    // Deserialize the header
    let deserialized: MessageHeader = deserialize(&serialized).expect("Failed to deserialize header");
    
    // Verify the deserialized header matches the original
    assert_eq!(header.version, deserialized.version);
    assert_eq!(header.msg_type, deserialized.msg_type);
    assert_eq!(header.msg_id, deserialized.msg_id);
    assert_eq!(header.priority, deserialized.priority);
    assert_eq!(header.timestamp_ms, deserialized.timestamp_ms);
    assert_eq!(header.sender_id.id, deserialized.sender_id.id);
    assert_eq!(header.sequence, deserialized.sequence);
}

#[test]
fn test_ping_message_roundtrip() {
    // Create a sample message header
    let agent_id = AgentId { id: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16] };
    let header = MessageHeader {
        version: PROTOCOL_VERSION,
        msg_type: MessageType::Ping,
        msg_id: 12345,
        priority: Priority::Normal,
        timestamp_ms: 1648000000000,
        sender_id: agent_id,
        sequence: 42,
    };
    
    // Create a ping message
    let ping = PingMessage {
        header,
        payload: Some("Hello, world!".to_string()),
    };
    
    // Serialize the message
    let serialized = serialize(&ping).expect("Failed to serialize ping message");
    
    // Deserialize the message
    let deserialized: PingMessage = deserialize(&serialized).expect("Failed to deserialize ping message");
    
    // Verify the deserialized message matches the original
    assert_eq!(ping.header.msg_id, deserialized.header.msg_id);
    assert_eq!(ping.header.msg_type, deserialized.header.msg_type);
    assert_eq!(ping.payload, deserialized.payload);
}

#[test]
fn test_message_enum_roundtrip() {
    // Create a sample message header
    let agent_id = AgentId { id: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16] };
    let header = MessageHeader {
        version: PROTOCOL_VERSION,
        msg_type: MessageType::Ping,
        msg_id: 12345,
        priority: Priority::Normal,
        timestamp_ms: 1648000000000,
        sender_id: agent_id,
        sequence: 42,
    };
    
    // Create a ping message
    let ping = PingMessage {
        header: header.clone(),
        payload: Some("Hello, world!".to_string()),
    };
    
    // Create a pong message
    let pong = PongMessage {
        header: MessageHeader {
            msg_type: MessageType::Pong,
            ..header.clone()
        },
        echo: Some("Hello, world!".to_string()),
        response_time_ms: 1648000000100,
    };
    
    // Create a top-level message enum
    let messages = vec![
        Message::Ping(ping.clone()),
        Message::Pong(pong.clone()),
    ];
    
    for message in messages {
        // Serialize the message
        let serialized = serialize(&message).expect("Failed to serialize message");
        
        // Deserialize the message
        let deserialized: Message = deserialize(&serialized).expect("Failed to deserialize message");
        
        // Verify the deserialized message matches the original
        match (message, deserialized) {
            (Message::Ping(original), Message::Ping(result)) => {
                assert_eq!(original.header.msg_id, result.header.msg_id);
                assert_eq!(original.payload, result.payload);
            }
            (Message::Pong(original), Message::Pong(result)) => {
                assert_eq!(original.header.msg_id, result.header.msg_id);
                assert_eq!(original.echo, result.echo);
                assert_eq!(original.response_time_ms, result.response_time_ms);
            }
            _ => panic!("Message type mismatch after deserialization"),
        }
    }
}

#[test]
fn test_consensus_messages_roundtrip() {
    // Create a sample message header
    let agent_id = AgentId { id: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16] };
    let header = MessageHeader {
        version: PROTOCOL_VERSION,
        msg_type: MessageType::ConsensusProposal,
        msg_id: 12345,
        priority: Priority::High,
        timestamp_ms: 1648000000000,
        sender_id: agent_id,
        sequence: 42,
    };
    
    // Create a consensus proposal message
    let proposal = ConsensusProposalMessage {
        header: header.clone(),
        round: 1,
        proposal_data: vec![1, 2, 3, 4, 5],
        proposal_hash: [0; 32],
    };
    
    // Create a consensus vote message
    let vote = ConsensusVoteMessage {
        header: MessageHeader {
            msg_type: MessageType::ConsensusVote,
            ..header.clone()
        },
        round: 1,
        vote: true,
        proposal_hash: [0; 32],
        signature: [0; 64],
    };
    
    // Serialize and deserialize the proposal
    let serialized = serialize(&proposal).expect("Failed to serialize proposal");
    let deserialized: ConsensusProposalMessage = deserialize(&serialized).expect("Failed to deserialize proposal");
    
    // Verify the deserialized proposal matches the original
    assert_eq!(proposal.header.msg_id, deserialized.header.msg_id);
    assert_eq!(proposal.round, deserialized.round);
    assert_eq!(proposal.proposal_data, deserialized.proposal_data);
    assert_eq!(proposal.proposal_hash, deserialized.proposal_hash);
    
    // Serialize and deserialize the vote
    let serialized = serialize(&vote).expect("Failed to serialize vote");
    let deserialized: ConsensusVoteMessage = deserialize(&serialized).expect("Failed to deserialize vote");
    
    // Verify the deserialized vote matches the original
    assert_eq!(vote.header.msg_id, deserialized.header.msg_id);
    assert_eq!(vote.round, deserialized.round);
    assert_eq!(vote.vote, deserialized.vote);
    assert_eq!(vote.proposal_hash, deserialized.proposal_hash);
    assert_eq!(vote.signature, deserialized.signature);
}

#[test]
fn test_empty_and_large_messages() {
    // Create a sample message header
    let agent_id = AgentId { id: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16] };
    let header = MessageHeader {
        version: PROTOCOL_VERSION,
        msg_type: MessageType::StateUpdate,
        msg_id: 12345,
        priority: Priority::Normal,
        timestamp_ms: 1648000000000,
        sender_id: agent_id,
        sequence: 42,
    };
    
    // Create an empty state update
    let empty_state = StateUpdateMessage {
        header: header.clone(),
        state_version: 1,
        state_data: vec![],
        prev_state_hash: [0; 32],
        state_hash: [0; 32],
    };
    
    // Create a large state update (100KB)
    let large_state = StateUpdateMessage {
        header: header.clone(),
        state_version: 2,
        state_data: vec![0; 100 * 1024],
        prev_state_hash: [1; 32],
        state_hash: [2; 32],
    };
    
    // Test empty state
    let serialized = serialize(&empty_state).expect("Failed to serialize empty state");
    let deserialized: StateUpdateMessage = deserialize(&serialized).expect("Failed to deserialize empty state");
    assert_eq!(empty_state.state_data.len(), deserialized.state_data.len());
    assert_eq!(empty_state.state_version, deserialized.state_version);
    
    // Test large state
    let serialized = serialize(&large_state).expect("Failed to serialize large state");
    let deserialized: StateUpdateMessage = deserialize(&serialized).expect("Failed to deserialize large state");
    assert_eq!(large_state.state_data.len(), deserialized.state_data.len());
    assert_eq!(large_state.state_version, deserialized.state_version);
} 