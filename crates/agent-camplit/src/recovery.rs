//! Recovery Policy Engine for Camplit agent
//!
//! This module implements the recovery policy engine that determines what
//! actions to take in response to failures in the system.

use serde::{Serialize, Deserialize};
use std::sync::Arc;
use aegis_core::error::AegisResult;
use crate::policy::{Policy, PolicyState};

/// Type of component or system that failed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureEntityType {
    /// Agent failure
    Agent,
    /// Service failure
    Service,
    /// Node failure
    Node,
    /// Network failure
    Network,
    /// Resource failure (disk, memory, etc.)
    Resource,
    /// Other type of failure
    Other(String),
}

/// Severity of a failure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureSeverity {
    /// Low severity - can continue operation with minor impact
    Low,
    /// Medium severity - significant impact but not critical
    Medium,
    /// High severity - major impact on functionality
    High,
    /// Critical severity - complete loss of functionality
    Critical,
}

/// Detailed information about a failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureDetails {
    /// ID of the entity that failed
    pub entity_id: String,
    /// Type of entity that failed
    pub entity_type: FailureEntityType,
    /// Timestamp of the failure
    pub timestamp: String,
    /// Severity of the failure
    pub severity: FailureSeverity,
    /// Error code or type
    pub error_code: Option<String>,
    /// Detailed error message
    pub error_message: String,
    /// Additional context or metadata
    pub context: serde_json::Value,
}

/// Type of recovery action to take
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryActionType {
    /// No action required
    NoAction,
    /// Restart the failed entity
    Restart,
    /// Failover to a backup or standby
    Failover,
    /// Scale resources up
    ScaleUp,
    /// Scale resources down
    ScaleDown,
    /// Alert a human operator
    Alert,
    /// Apply a specific configuration change
    Configure(serde_json::Value),
    /// Custom action with details
    Custom(String, serde_json::Value),
}

/// Priority level for a recovery action
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecoveryPriority {
    /// Low priority - can be deferred
    Low,
    /// Normal priority - should be handled promptly
    Normal,
    /// High priority - should be handled immediately
    High,
    /// Critical priority - must be handled immediately
    Critical,
}

/// Recovery action to take in response to a failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAction {
    /// Type of action to take
    pub action_type: RecoveryActionType,
    /// Priority of the action
    pub priority: RecoveryPriority,
    /// Entity ID the action applies to
    pub target_entity_id: String,
    /// Additional parameters for the action
    pub parameters: serde_json::Value,
    /// Text description of the action
    pub description: String,
    /// ID of the policy that generated this action
    pub policy_id: String,
}

/// Engine that determines recovery actions based on policies
pub struct RecoveryPolicyEngine {
    /// Reference to the policy state
    policy_state: Arc<PolicyState>,
}

impl RecoveryPolicyEngine {
    /// Create a new recovery policy engine
    pub fn new(policy_state: Arc<PolicyState>) -> Self {
        Self {
            policy_state,
        }
    }
    
    /// Determine the appropriate recovery action for a failure
    pub fn get_action_for_failure(&self, failure: &FailureDetails) -> AegisResult<RecoveryAction> {
        // This is a placeholder implementation
        // In a real implementation, we would:
        // 1. Get all policies applicable to the failed entity
        // 2. Filter to policies that have recovery rules
        // 3. Apply the policies in order of priority
        // 4. Return the highest priority action
        
        // For now, we'll just return a placeholder action based on failure severity
        let (action_type, priority) = match failure.severity {
            FailureSeverity::Low => (
                RecoveryActionType::NoAction,
                RecoveryPriority::Low,
            ),
            FailureSeverity::Medium => (
                RecoveryActionType::Restart,
                RecoveryPriority::Normal,
            ),
            FailureSeverity::High => (
                RecoveryActionType::Failover,
                RecoveryPriority::High,
            ),
            FailureSeverity::Critical => (
                RecoveryActionType::Alert,
                RecoveryPriority::Critical,
            ),
        };
        
        Ok(RecoveryAction {
            action_type,
            priority,
            target_entity_id: failure.entity_id.clone(),
            parameters: serde_json::json!({}),
            description: format!("Automatic response to {} failure", failure.severity),
            policy_id: "default-recovery-policy".to_string(),
        })
    }
    
    /// Update the policy state
    pub fn update_policy_state(&mut self, policy_state: Arc<PolicyState>) {
        self.policy_state = policy_state;
    }
    
    /// Get all available recovery policies
    pub fn get_recovery_policies(&self) -> Vec<&Policy> {
        // In a real implementation, we would filter to policies with recovery rules
        // For now, just return all policies
        self.policy_state.get_all_policies()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{PolicyScope, PolicyPriority};
    
    fn create_test_failure() -> FailureDetails {
        FailureDetails {
            entity_id: "agent-123".to_string(),
            entity_type: FailureEntityType::Agent,
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            severity: FailureSeverity::Medium,
            error_code: Some("CONNECTION_TIMEOUT".to_string()),
            error_message: "Connection timed out".to_string(),
            context: serde_json::json!({
                "attempts": 3,
                "last_successful_connection": "2023-01-01T00:00:00Z",
            }),
        }
    }
    
    fn create_test_policy_state() -> PolicyState {
        let mut state = PolicyState::new();
        
        // Add a test policy
        let policy = Policy {
            id: "recovery-policy-1".to_string(),
            name: "Basic Recovery Policy".to_string(),
            description: "Basic recovery policy for testing".to_string(),
            version: "1.0".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
            priority: PolicyPriority::High,
            scope: PolicyScope::Global,
            rules: serde_json::json!({
                "recovery": {
                    "agent_failure": {
                        "medium": {
                            "action": "restart",
                            "priority": "normal"
                        }
                    }
                }
            }),
            enabled: true,
        };
        
        state.apply(crate::policy::PolicyStateCommand::UpsertPolicy(policy)).unwrap();
        
        state
    }
    
    #[test]
    fn test_recovery_action_for_failure() {
        let policy_state = create_test_policy_state();
        let engine = RecoveryPolicyEngine::new(Arc::new(policy_state));
        
        let failure = create_test_failure();
        let action = engine.get_action_for_failure(&failure).unwrap();
        
        // Verify the action is appropriate for the failure
        assert_eq!(action.action_type, RecoveryActionType::Restart);
        assert_eq!(action.priority, RecoveryPriority::Normal);
        assert_eq!(action.target_entity_id, failure.entity_id);
    }
} 