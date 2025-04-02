//! Compliance and Policy Enforcement Agent (`camplit_ai`) for the Aegis platform
//!
//! This crate provides the Compliance and Policy Enforcement Agent (`camplit_ai`)
//! for the Aegis platform, responsible for managing policy distribution and compliance,
//! as well as enforcing recovery policies.

mod policy;
mod recovery;
mod agent;

pub use agent::CamplitAgent;
pub use policy::{Policy, PolicyState, PolicyStateCommand, PolicyScope, PolicyPriority};
pub use recovery::{
    RecoveryPolicyEngine,
    RecoveryAction,
    RecoveryActionType,
    RecoveryPriority,
    FailureDetails,
    FailureEntityType,
    FailureSeverity,
}; 