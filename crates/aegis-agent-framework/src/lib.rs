//! Aegis Agent Framework
//!
//! This crate provides the core abstractions for defining and managing agents
//! within the Aegis framework. It includes the `AegisAgent` trait, which all
//! agents must implement, and the `AgentLifecycleManager` for managing the
//! agent lifecycle.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Agent trait and status definitions
pub mod agent;

/// Agent context for accessing framework services
pub mod context;

/// Agent lifecycle management
pub mod lifecycle;

// Re-export key types for convenience
pub use agent::{AegisAgent, AgentStatus};
pub use context::{AgentContext, AgentID};
pub use lifecycle::AgentLifecycleManager;

#[cfg(feature = "with-tokio")]
pub use lifecycle::MessageHandlingAgent;

/// Prelude module that re-exports commonly used types
pub mod prelude {
    pub use crate::agent::{AegisAgent, AgentStatus};
    pub use crate::context::{AgentContext, AgentID};
    pub use crate::lifecycle::AgentLifecycleManager;

    #[cfg(feature = "with-tokio")]
    pub use crate::lifecycle::MessageHandlingAgent;
    
    // Re-export common types from aegis-core
    pub use aegis_core::error::{AegisError, AegisResult};
} 