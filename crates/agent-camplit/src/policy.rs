//! Policy state definitions and state machine implementation for the Camplit agent
//!
//! This module defines the policy state and commands that can modify it,
//! implementing the `StateMachine` trait from the consensus framework.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use aegis_core::error::AegisResult;
use aegis_consensus::StateMachine;

/// A unique identifier for a policy
pub type PolicyId = String;

/// Priority level for a policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PolicyPriority {
    /// Lowest priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority (highest)
    Critical,
}

/// Target scope for a policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyScope {
    /// Policy applies to all agents
    Global,
    /// Policy applies to a specific agent type
    AgentType(String),
    /// Policy applies to a specific agent instance
    AgentInstance(String),
}

/// Policy definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    /// Unique identifier for the policy
    pub id: PolicyId,
    /// Human-readable name
    pub name: String,
    /// Detailed description
    pub description: String,
    /// Version of the policy
    pub version: String,
    /// When the policy was created
    pub created_at: String,
    /// When the policy was last updated
    pub updated_at: String,
    /// Priority level
    pub priority: PolicyPriority,
    /// Target scope for the policy
    pub scope: PolicyScope,
    /// The actual policy rules as JSON
    pub rules: serde_json::Value,
    /// Whether the policy is enabled
    pub enabled: bool,
}

/// Commands that can modify the policy state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyStateCommand {
    /// Add a new policy or update an existing one
    UpsertPolicy(Policy),
    /// Remove a policy
    RemovePolicy(PolicyId),
    /// Enable a policy
    EnablePolicy(PolicyId),
    /// Disable a policy
    DisablePolicy(PolicyId),
}

/// The policy state maintained by the Camplit agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyState {
    /// Map of policy ID to policy
    policies: HashMap<PolicyId, Policy>,
}

impl Default for PolicyState {
    fn default() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }
}

impl PolicyState {
    /// Create a new empty policy state
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get a policy by ID
    pub fn get_policy(&self, id: &PolicyId) -> Option<&Policy> {
        self.policies.get(id)
    }
    
    /// Get all policies
    pub fn get_all_policies(&self) -> Vec<&Policy> {
        self.policies.values().collect()
    }
    
    /// Get policies by scope
    pub fn get_policies_for_scope(&self, scope: &PolicyScope) -> Vec<&Policy> {
        self.policies
            .values()
            .filter(|p| p.scope == *scope && p.enabled)
            .collect()
    }
    
    /// Get policies for a specific agent type
    pub fn get_policies_for_agent_type(&self, agent_type: &str) -> Vec<&Policy> {
        let agent_type_scope = PolicyScope::AgentType(agent_type.to_string());
        let global_scope = PolicyScope::Global;
        
        self.policies
            .values()
            .filter(|p| (p.scope == agent_type_scope || p.scope == global_scope) && p.enabled)
            .collect()
    }
    
    /// Get policies for a specific agent instance
    pub fn get_policies_for_agent_instance(&self, agent_id: &str) -> Vec<&Policy> {
        let agent_instance_scope = PolicyScope::AgentInstance(agent_id.to_string());
        
        // First get the agent type from the ID
        // Assuming agent ID format includes the type as a prefix: "type_instance"
        let parts: Vec<&str> = agent_id.split('_').collect();
        let agent_type = if parts.len() > 1 {
            Some(parts[0])
        } else {
            None
        };
        
        // Get global policies
        let mut applicable_policies: Vec<&Policy> = self.policies
            .values()
            .filter(|p| p.scope == PolicyScope::Global && p.enabled)
            .collect();
        
        // Add agent type policies if we can determine the type
        if let Some(agent_type) = agent_type {
            let agent_type_scope = PolicyScope::AgentType(agent_type.to_string());
            let type_policies: Vec<&Policy> = self.policies
                .values()
                .filter(|p| p.scope == agent_type_scope && p.enabled)
                .collect();
            
            applicable_policies.extend(type_policies);
        }
        
        // Add instance-specific policies
        let instance_policies: Vec<&Policy> = self.policies
            .values()
            .filter(|p| p.scope == agent_instance_scope && p.enabled)
            .collect();
        
        applicable_policies.extend(instance_policies);
        
        applicable_policies
    }
}

impl StateMachine for PolicyState {
    type Command = PolicyStateCommand;
    
    fn apply(&mut self, command: Self::Command) -> AegisResult<()> {
        match command {
            PolicyStateCommand::UpsertPolicy(policy) => {
                self.policies.insert(policy.id.clone(), policy);
            }
            PolicyStateCommand::RemovePolicy(id) => {
                self.policies.remove(&id);
            }
            PolicyStateCommand::EnablePolicy(id) => {
                if let Some(policy) = self.policies.get_mut(&id) {
                    policy.enabled = true;
                }
            }
            PolicyStateCommand::DisablePolicy(id) => {
                if let Some(policy) = self.policies.get_mut(&id) {
                    policy.enabled = false;
                }
            }
        }
        
        Ok(())
    }
    
    fn snapshot(&self) -> AegisResult<Vec<u8>> {
        // Serialize the entire state to JSON
        let json = serde_json::to_vec(self)
            .map_err(|e| aegis_core::error::AegisError::Serialization(e))?;
        
        Ok(json)
    }
    
    fn restore(&mut self, snapshot: &[u8]) -> AegisResult<()> {
        // Deserialize from JSON
        let state: PolicyState = serde_json::from_slice(snapshot)
            .map_err(|e| aegis_core::error::AegisError::Serialization(e))?;
        
        *self = state;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_policy(id: &str, name: &str, scope: PolicyScope) -> Policy {
        Policy {
            id: id.to_string(),
            name: name.to_string(),
            description: "Test policy".to_string(),
            version: "1.0".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
            priority: PolicyPriority::Medium,
            scope,
            rules: serde_json::json!({
                "max_resource_usage": 80,
                "log_retention_days": 30,
            }),
            enabled: true,
        }
    }
    
    #[test]
    fn test_policy_state_commands() {
        let mut state = PolicyState::new();
        
        // Create a few test policies
        let global_policy = create_test_policy(
            "global-1", 
            "Global Resource Policy", 
            PolicyScope::Global
        );
        
        let agent_type_policy = create_test_policy(
            "type-1", 
            "Manre Agent Policy", 
            PolicyScope::AgentType("manre".to_string())
        );
        
        let agent_instance_policy = create_test_policy(
            "instance-1", 
            "Specific Agent Policy", 
            PolicyScope::AgentInstance("manre_123".to_string())
        );
        
        // Test adding policies
        state.apply(PolicyStateCommand::UpsertPolicy(global_policy.clone())).unwrap();
        state.apply(PolicyStateCommand::UpsertPolicy(agent_type_policy.clone())).unwrap();
        state.apply(PolicyStateCommand::UpsertPolicy(agent_instance_policy.clone())).unwrap();
        
        // Test querying policies
        assert_eq!(state.get_all_policies().len(), 3);
        assert_eq!(state.get_policy(&"global-1".to_string()), Some(&global_policy));
        
        // Test policies by scope
        let global_policies = state.get_policies_for_scope(&PolicyScope::Global);
        assert_eq!(global_policies.len(), 1);
        assert_eq!(global_policies[0], &global_policy);
        
        // Test policies for agent type
        let manre_policies = state.get_policies_for_agent_type("manre");
        assert_eq!(manre_policies.len(), 2); // Both global and type-specific
        
        // Test policies for agent instance
        let instance_policies = state.get_policies_for_agent_instance("manre_123");
        assert_eq!(instance_policies.len(), 3); // All three should apply
        
        // Test disable policy
        state.apply(PolicyStateCommand::DisablePolicy("global-1".to_string())).unwrap();
        let disabled_policy = state.get_policy(&"global-1".to_string()).unwrap();
        assert_eq!(disabled_policy.enabled, false);
        
        // Test remove policy
        state.apply(PolicyStateCommand::RemovePolicy("type-1".to_string())).unwrap();
        assert_eq!(state.get_policy(&"type-1".to_string()), None);
        assert_eq!(state.get_all_policies().len(), 2);
    }
    
    #[test]
    fn test_snapshot_restore() {
        let mut state = PolicyState::new();
        
        // Add some policies
        state.apply(PolicyStateCommand::UpsertPolicy(create_test_policy(
            "policy-1",
            "Test Policy 1",
            PolicyScope::Global
        ))).unwrap();
        
        state.apply(PolicyStateCommand::UpsertPolicy(create_test_policy(
            "policy-2",
            "Test Policy 2",
            PolicyScope::AgentType("test".to_string())
        ))).unwrap();
        
        // Take a snapshot
        let snapshot = state.snapshot().unwrap();
        
        // Create a new state and restore from snapshot
        let mut new_state = PolicyState::new();
        new_state.restore(&snapshot).unwrap();
        
        // Verify the states are equal
        assert_eq!(state.get_all_policies().len(), new_state.get_all_policies().len());
        assert!(new_state.get_policy(&"policy-1".to_string()).is_some());
        assert!(new_state.get_policy(&"policy-2".to_string()).is_some());
    }
} 