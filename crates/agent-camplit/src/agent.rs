//! Implementation of the Camplit Agent for the Aegis platform
//!
//! This module provides the implementation of the `CamplitAgent` which manages
//! policy distribution and compliance, as well as enforcing recovery policies.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use bytes::Bytes;
use tracing::{debug, error, info, warn};
use serde::{Serialize, Deserialize};

use aegis_agent_framework::{AegisAgent, AgentStatus, AgentContext};
use aegis_core::error::{AegisError, AegisResult};
use aegis_consensus::ConsensusClient;

use crate::policy::{Policy, PolicyState, PolicyStateCommand};
use crate::recovery::{RecoveryPolicyEngine, FailureDetails, RecoveryAction};

/// Message types for communication with the Camplit agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CamplitMessage {
    /// Request to retrieve a policy
    GetPolicy { id: String },
    /// Request to retrieve all policies
    GetAllPolicies,
    /// Request to create or update a policy
    UpsertPolicy { policy: Policy },
    /// Request to remove a policy
    RemovePolicy { id: String },
    /// Request to enable a policy
    EnablePolicy { id: String },
    /// Request to disable a policy
    DisablePolicy { id: String },
    /// Request to retrieve a recovery action for a failure
    GetRecoveryAction { failure: FailureDetails },
}

/// Response message types from the Camplit agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CamplitResponse {
    /// Response with a policy
    Policy { policy: Option<Policy> },
    /// Response with multiple policies
    Policies { policies: Vec<Policy> },
    /// Response with a success status
    Success,
    /// Response with an error
    Error { message: String },
    /// Response with a recovery action
    RecoveryAction { action: RecoveryAction },
}

/// Implementation of the Compliance and Policy Enforcement Agent
pub struct CamplitAgent {
    /// Current status of the agent
    status: AgentStatus,
    
    /// Agent context
    context: Option<AgentContext>,
    
    /// Consensus client for policy state
    consensus_client: Option<ConsensusClient<PolicyState>>,
    
    /// Local cache of policy state
    policy_state: Arc<Mutex<PolicyState>>,
    
    /// Recovery policy engine
    recovery_engine: Option<RecoveryPolicyEngine>,
}

impl CamplitAgent {
    /// Create a new Camplit agent
    pub fn new() -> Self {
        Self {
            status: AgentStatus::Initializing,
            context: None,
            consensus_client: None,
            policy_state: Arc::new(Mutex::new(PolicyState::new())),
            recovery_engine: None,
        }
    }
    
    /// Process a message and generate a response
    async fn process_message(&mut self, message: CamplitMessage) -> AegisResult<CamplitResponse> {
        match message {
            CamplitMessage::GetPolicy { id } => {
                let policy_state = self.policy_state.lock().unwrap();
                let policy = policy_state.get_policy(&id).cloned();
                Ok(CamplitResponse::Policy { policy })
            }
            
            CamplitMessage::GetAllPolicies => {
                let policy_state = self.policy_state.lock().unwrap();
                let policies = policy_state.get_all_policies().into_iter().cloned().collect();
                Ok(CamplitResponse::Policies { policies })
            }
            
            CamplitMessage::UpsertPolicy { policy } => {
                // Update the policy via consensus
                if let Some(consensus) = &self.consensus_client {
                    consensus.submit_command(PolicyStateCommand::UpsertPolicy(policy)).await?;
                    Ok(CamplitResponse::Success)
                } else {
                    // Fallback to direct update if consensus is not available
                    let mut policy_state = self.policy_state.lock().unwrap();
                    policy_state.apply(PolicyStateCommand::UpsertPolicy(policy))?;
                    Ok(CamplitResponse::Success)
                }
            }
            
            CamplitMessage::RemovePolicy { id } => {
                // Remove the policy via consensus
                if let Some(consensus) = &self.consensus_client {
                    consensus.submit_command(PolicyStateCommand::RemovePolicy(id)).await?;
                    Ok(CamplitResponse::Success)
                } else {
                    // Fallback to direct update if consensus is not available
                    let mut policy_state = self.policy_state.lock().unwrap();
                    policy_state.apply(PolicyStateCommand::RemovePolicy(id))?;
                    Ok(CamplitResponse::Success)
                }
            }
            
            CamplitMessage::EnablePolicy { id } => {
                // Enable the policy via consensus
                if let Some(consensus) = &self.consensus_client {
                    consensus.submit_command(PolicyStateCommand::EnablePolicy(id)).await?;
                    Ok(CamplitResponse::Success)
                } else {
                    // Fallback to direct update if consensus is not available
                    let mut policy_state = self.policy_state.lock().unwrap();
                    policy_state.apply(PolicyStateCommand::EnablePolicy(id))?;
                    Ok(CamplitResponse::Success)
                }
            }
            
            CamplitMessage::DisablePolicy { id } => {
                // Disable the policy via consensus
                if let Some(consensus) = &self.consensus_client {
                    consensus.submit_command(PolicyStateCommand::DisablePolicy(id)).await?;
                    Ok(CamplitResponse::Success)
                } else {
                    // Fallback to direct update if consensus is not available
                    let mut policy_state = self.policy_state.lock().unwrap();
                    policy_state.apply(PolicyStateCommand::DisablePolicy(id))?;
                    Ok(CamplitResponse::Success)
                }
            }
            
            CamplitMessage::GetRecoveryAction { failure } => {
                // Get recovery action from the recovery engine
                if let Some(engine) = &self.recovery_engine {
                    let action = engine.get_action_for_failure(&failure)?;
                    Ok(CamplitResponse::RecoveryAction { action })
                } else {
                    Ok(CamplitResponse::Error {
                        message: "Recovery engine not initialized".to_string(),
                    })
                }
            }
        }
    }
    
    /// Update policy state from consensus
    async fn update_policy_state_from_consensus(&mut self) -> AegisResult<()> {
        if let Some(consensus) = &self.consensus_client {
            match consensus.get_state().await {
                Ok(state) => {
                    let new_state = Arc::new(state);
                    *self.policy_state.lock().unwrap() = (*new_state).clone();
                    
                    // Update the recovery engine with the new state
                    if let Some(engine) = &mut self.recovery_engine {
                        engine.update_policy_state(new_state);
                    }
                    
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to get policy state from consensus: {}", e);
                    Err(e)
                }
            }
        } else {
            Ok(()) // No consensus client, so nothing to update
        }
    }
}

#[async_trait]
impl AegisAgent for CamplitAgent {
    async fn initialize(&mut self, context: AgentContext) -> AegisResult<()> {
        info!("Initializing Camplit agent");
        
        // Store the context
        self.context = Some(context);
        
        // Create the consensus client
        self.consensus_client = Some(ConsensusClient::<PolicyState>::new());
        
        // Initialize the policy state
        let policy_state_arc = self.policy_state.clone();
        
        // Initialize the recovery engine
        self.recovery_engine = Some(RecoveryPolicyEngine::new(policy_state_arc));
        
        // Update the policy state from consensus
        self.update_policy_state_from_consensus().await?;
        
        // Update the status
        self.status = AgentStatus::Running;
        
        info!("Camplit agent initialized");
        
        Ok(())
    }
    
    async fn run(&mut self) -> AegisResult<()> {
        info!("Starting Camplit agent");
        
        // Verify context is available
        let context = self.context.as_ref().ok_or_else(|| {
            AegisError::Generic("Agent context not initialized".to_string())
        })?;
        
        // Main agent loop - periodically update policy state from consensus
        let mut interval_count = 0;
        loop {
            // Sleep for a while
            let sleep_duration = std::time::Duration::from_secs(10);
            if let Err(e) = context.timer.sleep(sleep_duration).await {
                error!("Error in sleep: {}", e);
                self.status = AgentStatus::Degraded("Sleep error".to_string());
                continue;
            }
            
            interval_count += 1;
            debug!("Camplit agent heartbeat: {}", interval_count);
            
            // Update policy state from consensus
            if let Err(e) = self.update_policy_state_from_consensus().await {
                error!("Error updating policy state: {}", e);
                self.status = AgentStatus::Degraded("Consensus error".to_string());
            } else if self.status == AgentStatus::Degraded("Consensus error".to_string()) {
                // Recovered from consensus error
                self.status = AgentStatus::Running;
            }
            
            // Check if we should stop
            if matches!(self.status, AgentStatus::ShuttingDown | AgentStatus::Stopped) {
                break;
            }
        }
        
        info!("Camplit agent run completed");
        
        Ok(())
    }
    
    async fn shutdown(&mut self) -> AegisResult<()> {
        info!("Shutting down Camplit agent");
        
        // Update status
        self.status = AgentStatus::ShuttingDown;
        
        // Perform cleanup
        
        // Update status
        self.status = AgentStatus::Stopped;
        
        info!("Camplit agent shutdown complete");
        
        Ok(())
    }
    
    async fn handle_message(&mut self, message: Bytes) -> AegisResult<()> {
        debug!("Received message of {} bytes", message.len());
        
        // Deserialize the message
        let camplit_message: CamplitMessage = match serde_json::from_slice(&message) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to deserialize message: {}", e);
                return Err(AegisError::Serialization(e));
            }
        };
        
        debug!("Processing message: {:?}", camplit_message);
        
        // Process the message and generate a response
        let response = match self.process_message(camplit_message).await {
            Ok(response) => response,
            Err(e) => {
                error!("Error processing message: {}", e);
                CamplitResponse::Error {
                    message: format!("Error processing message: {}", e),
                }
            }
        };
        
        // If we have a context, send the response
        if let Some(context) = &self.context {
            // Serialize the response
            let response_bytes = match serde_json::to_vec(&response) {
                Ok(bytes) => Bytes::from(bytes),
                Err(e) => {
                    error!("Failed to serialize response: {}", e);
                    return Err(AegisError::Serialization(e));
                }
            };
            
            // TODO: Send the response to the appropriate destination
            // This would involve getting the sender from the message and using
            // the comms_client to send the response
        }
        
        Ok(())
    }
    
    fn get_status(&self) -> AgentStatus {
        self.status.clone()
    }
}

impl Default for CamplitAgent {
    fn default() -> Self {
        Self::new()
    }
} 