use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Protocol version for compatibility checks
pub const PROTOCOL_VERSION: u32 = 1;

/// Base header included in all messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageHeader {
    pub version: u32,
    pub message_type: MessageType,
    pub source: Option<SocketAddr>,
    pub destination: Option<SocketAddr>,
}

/// Enum defining all possible message types in the protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    ConsensusVoteRequest,
    ConsensusVoteResponse,
    StateUpdate,
    AgentDiscovery,
    AgentHeartbeat,
    Error,
}

/// Message for requesting votes in consensus
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsensusVoteRequest {
    pub header: MessageHeader,
    pub proposal_id: u64,
    pub proposal_data: Vec<u8>,
}

/// Message for responding to vote requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsensusVoteResponse {
    pub header: MessageHeader,
    pub proposal_id: u64,
    pub vote: bool,
    pub voter_id: String,
}

/// Message for state updates between agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateUpdateMessage {
    pub header: MessageHeader,
    pub state_version: u64,
    pub state_data: Vec<u8>,
    pub is_delta: bool,
}

/// Message for agent discovery and registration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentDiscoveryMessage {
    pub header: MessageHeader,
    pub agent_id: String,
    pub capabilities: Vec<String>,
    pub listen_addr: SocketAddr,
}

/// Message for agent heartbeat/liveness
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentHeartbeatMessage {
    pub header: MessageHeader,
    pub agent_id: String,
    pub timestamp: u64,
    pub status: AgentStatus,
}

/// Status information included in heartbeats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Active,
    Busy,
    Draining,
    ShuttingDown,
}

/// Error message for protocol errors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorMessage {
    pub header: MessageHeader,
    pub error_code: u32,
    pub error_message: String,
} 