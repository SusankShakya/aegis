//! Agent context for the Aegis framework
//!
//! This module defines the `AgentContext` struct that provides agents with
//! access to framework services.

use std::sync::Arc;

/// AgentID type alias for agent identification
pub type AgentID = String;

/// Agent context providing access to framework services
///
/// This struct is passed to an agent during initialization and provides
/// access to various services required by the agent, such as configuration,
/// communication, task spawning, and timing.
#[derive(Clone)]
pub struct AgentContext {
    /// Unique identifier for this agent
    pub agent_id: AgentID,
    
    /// Configuration for this agent
    pub config: Arc<aegis_core::config::AegisConfig>,
    
    /// Communication client for sending/receiving messages
    pub comms_client: Arc<aegis_comms::CommsClient>,
    
    /// Task spawner for spawning asynchronous tasks
    pub spawner: Arc<dyn aegis_core::platform::concurrency::AsyncTaskSpawner>,
    
    /// Timer for scheduling delayed tasks
    pub timer: Arc<dyn aegis_core::platform::concurrency::AsyncTimer>,
}

impl AgentContext {
    /// Create a new agent context
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Unique identifier for this agent
    /// * `config` - Configuration for this agent
    /// * `comms_client` - Communication client for sending/receiving messages
    /// * `spawner` - Task spawner for spawning asynchronous tasks
    /// * `timer` - Timer for scheduling delayed tasks
    ///
    /// # Returns
    ///
    /// A new `AgentContext` instance
    pub fn new(
        agent_id: AgentID,
        config: Arc<aegis_core::config::AegisConfig>,
        comms_client: Arc<aegis_comms::CommsClient>,
        spawner: Arc<dyn aegis_core::platform::concurrency::AsyncTaskSpawner>,
        timer: Arc<dyn aegis_core::platform::concurrency::AsyncTimer>,
    ) -> Self {
        Self {
            agent_id,
            config,
            comms_client,
            spawner,
            timer,
        }
    }
    
    /// Get the agent ID
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }
} 