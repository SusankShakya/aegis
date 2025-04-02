//! Protocol definition for inter-agent communication
//! 
//! This module defines the binary message protocol used for communication
//! between Aegis agents, using platform-neutral types and serialization.

use serde::{Serialize, Deserialize};
use std::fmt;

/// Protocol version
pub const PROTOCOL_VERSION: u16 = 1;

/// Protocol version identifier
pub const PROTOCOL_ID: &[u8; 4] = b"AGIS";

/// Message priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// Base header included in all messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Protocol version
    pub version: u16,
    /// Message type identifier
    pub msg_type: MessageType,
    /// Unique message identifier
    pub msg_id: u64,
    /// Message priority
    pub priority: Priority,
    /// Timestamp in milliseconds since UNIX epoch
    pub timestamp_ms: u64,
    /// Sender identifier
    pub sender_id: AgentId,
    /// Sequence number for this sender
    pub sequence: u32,
}

/// Unique identifier for an agent
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId {
    /// Fixed-size agent identifier (UUID-like)
    pub id: [u8; 16],
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, byte) in self.id.iter().enumerate() {
            if i == 4 || i == 6 || i == 8 || i == 10 {
                write!(f, "-")?;
            }
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// Message type identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum MessageType {
    Ping = 0,
    Pong = 1,
    StateUpdate = 2,
    Command = 3,
    Event = 4,
    ConsensusProposal = 5,
    ConsensusVote = 6,
    Discovery = 7,
    Error = 100,
}

/// Basic ping message to check connectivity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PingMessage {
    /// Common message header
    pub header: MessageHeader,
    /// Optional payload/content
    pub payload: Option<String>,
}

/// Response to a ping message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PongMessage {
    /// Common message header
    pub header: MessageHeader,
    /// Echo of the ping payload if provided
    pub echo: Option<String>,
    /// Response timestamp in milliseconds since UNIX epoch
    pub response_time_ms: u64,
}

/// Error response message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorMessage {
    /// Common message header
    pub header: MessageHeader,
    /// Error code
    pub error_code: u16,
    /// Error message
    pub error_message: String,
    /// Original message ID that caused error
    pub original_msg_id: Option<u64>,
}

/// State update notification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateUpdateMessage {
    /// Common message header
    pub header: MessageHeader,
    /// State version
    pub state_version: u64,
    /// State data encoded as bytes
    pub state_data: Vec<u8>,
    /// Hash of the previous state
    pub prev_state_hash: [u8; 32],
    /// Hash of this state
    pub state_hash: [u8; 32],
}

/// Command message to request an action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandMessage {
    /// Common message header
    pub header: MessageHeader,
    /// Command identifier
    pub command_id: u32,
    /// Command parameters as serialized bytes
    pub parameters: Vec<u8>,
    /// Expected response message ID if applicable
    pub response_to: Option<u64>,
}

/// Consensus proposal message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusProposalMessage {
    /// Common message header
    pub header: MessageHeader,
    /// Round number
    pub round: u64,
    /// Proposal data
    pub proposal_data: Vec<u8>,
    /// Hash of the proposal for verification
    pub proposal_hash: [u8; 32],
}

/// Consensus vote message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusVoteMessage {
    /// Common message header
    pub header: MessageHeader,
    /// Round number
    pub round: u64,
    /// Vote for proposal (true = accept, false = reject)
    pub vote: bool,
    /// Hash of the proposal being voted on
    pub proposal_hash: [u8; 32],
    /// Voter signature
    pub signature: [u8; 64],
}

/// Discovery message for peer identification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryMessage {
    /// Common message header
    pub header: MessageHeader,
    /// Agent capabilities bitmap
    pub capabilities: u64,
    /// Network address for direct connection
    pub listen_addr: Option<String>,
    /// Protocol versions supported
    pub supported_versions: Vec<u16>,
}

/// Top-level message enum that can be serialized/deserialized
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Message {
    Ping(PingMessage),
    Pong(PongMessage),
    Error(ErrorMessage),
    StateUpdate(StateUpdateMessage),
    Command(CommandMessage),
    ConsensusProposal(ConsensusProposalMessage),
    ConsensusVote(ConsensusVoteMessage),
    Discovery(DiscoveryMessage),
} 