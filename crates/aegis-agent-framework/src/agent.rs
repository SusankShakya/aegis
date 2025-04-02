//! Agent trait definition for the Aegis framework
//!
//! This module defines the core `AegisAgent` trait that all agents in the Aegis
//! framework must implement, as well as the `AgentStatus` enum for reporting
//! agent state.

use async_trait::async_trait;
use bytes::Bytes;
use std::fmt;

use crate::context::AgentContext;

/// Status of an agent in the Aegis framework
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    /// Agent is initializing
    Initializing,
    
    /// Agent is running normally
    Running,
    
    /// Agent is running in a degraded state
    Degraded(String),
    
    /// Agent is in the process of shutting down
    ShuttingDown,
    
    /// Agent has stopped
    Stopped,
    
    /// Agent has failed
    Failed(String),
}

impl fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentStatus::Initializing => write!(f, "Initializing"),
            AgentStatus::Running => write!(f, "Running"),
            AgentStatus::Degraded(reason) => write!(f, "Degraded: {}", reason),
            AgentStatus::ShuttingDown => write!(f, "Shutting Down"),
            AgentStatus::Stopped => write!(f, "Stopped"),
            AgentStatus::Failed(reason) => write!(f, "Failed: {}", reason),
        }
    }
}

/// Core trait that all Aegis agents must implement
#[async_trait]
pub trait AegisAgent: Send + Sync {
    /// Initialize the agent with the provided context
    ///
    /// This method is called once when the agent is started.
    ///
    /// # Arguments
    ///
    /// * `context` - The agent context providing access to framework services
    ///
    /// # Returns
    ///
    /// `Result<()>` indicating success or failure
    async fn initialize(&mut self, context: AgentContext) -> aegis_core::error::AegisResult<()>;
    
    /// Run the agent's main logic
    ///
    /// This method is called after initialization and represents the main
    /// operational phase of the agent. It might implement a loop internally
    /// or represent a single task.
    ///
    /// # Returns
    ///
    /// `Result<()>` indicating success or failure
    async fn run(&mut self) -> aegis_core::error::AegisResult<()>;
    
    /// Shut down the agent gracefully
    ///
    /// This method is called when the agent is requested to stop,
    /// allowing it to clean up resources and terminate gracefully.
    ///
    /// # Returns
    ///
    /// `Result<()>` indicating success or failure
    async fn shutdown(&mut self) -> aegis_core::error::AegisResult<()>;
    
    /// Handle an incoming message
    ///
    /// This method is called when a message is received for this agent.
    ///
    /// # Arguments
    ///
    /// * `message` - The message payload as bytes
    ///
    /// # Returns
    ///
    /// `Result<()>` indicating success or failure
    async fn handle_message(&mut self, message: Bytes) -> aegis_core::error::AegisResult<()>;
    
    /// Get the current status of the agent
    ///
    /// # Returns
    ///
    /// The current `AgentStatus`
    fn get_status(&self) -> AgentStatus;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test the Display implementation for AgentStatus
    #[test]
    fn test_agent_status_display() {
        assert_eq!(AgentStatus::Initializing.to_string(), "Initializing");
        assert_eq!(AgentStatus::Running.to_string(), "Running");
        assert_eq!(
            AgentStatus::Degraded("low memory".to_string()).to_string(),
            "Degraded: low memory"
        );
        assert_eq!(AgentStatus::ShuttingDown.to_string(), "Shutting Down");
        assert_eq!(AgentStatus::Stopped.to_string(), "Stopped");
        assert_eq!(
            AgentStatus::Failed("connection error".to_string()).to_string(),
            "Failed: connection error"
        );
    }
} 