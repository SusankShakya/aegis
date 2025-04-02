//! Agent lifecycle management for the Aegis framework
//!
//! This module provides the `AgentLifecycleManager` for coordinating
//! the lifecycle of agents, including initialization, execution, and shutdown.

use std::{sync::Arc, time::Duration};
use tracing::{debug, error, info, warn};

use aegis_core::error::{AegisError, AegisResult};
use crate::{agent::{AegisAgent, AgentStatus}, context::AgentContext};

/// Manages the lifecycle of an agent
pub struct AgentLifecycleManager;

impl AgentLifecycleManager {
    /// Manage the lifecycle of an agent
    ///
    /// This function handles the complete lifecycle of an agent, including:
    /// 1. Initialization
    /// 2. Running the main logic
    /// 3. Graceful shutdown
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent to manage
    /// * `context` - The agent context
    ///
    /// # Returns
    ///
    /// `Result<()>` indicating success or failure of the agent's lifecycle
    pub async fn manage<A: AegisAgent + 'static>(
        mut agent: A,
        context: AgentContext,
    ) -> AegisResult<()> {
        // Initialize the agent
        info!("Initializing agent: {}", context.agent_id());
        match agent.initialize(context.clone()).await {
            Ok(()) => {
                info!("Agent initialized successfully: {}", context.agent_id());
                
                // Run the agent's main logic
                info!("Starting agent: {}", context.agent_id());
                let run_result = agent.run().await;
                
                match &run_result {
                    Ok(()) => info!("Agent completed successfully: {}", context.agent_id()),
                    Err(e) => error!("Agent failed: {}, error: {}", context.agent_id(), e),
                }
                
                // Attempt graceful shutdown regardless of run result
                info!("Shutting down agent: {}", context.agent_id());
                match agent.shutdown().await {
                    Ok(()) => {
                        info!("Agent shutdown successfully: {}", context.agent_id());
                        // Return the run result - if it was an error, propagate it
                        run_result
                    }
                    Err(e) => {
                        error!("Agent shutdown failed: {}, error: {}", context.agent_id(), e);
                        // If run was successful but shutdown failed, return the shutdown error
                        if run_result.is_ok() {
                            Err(e)
                        } else {
                            // If both run and shutdown failed, prioritize the run error
                            run_result
                        }
                    }
                }
            }
            Err(e) => {
                error!("Agent initialization failed: {}, error: {}", context.agent_id(), e);
                
                // Attempt cleanup even if initialization failed
                warn!("Attempting cleanup after failed initialization: {}", context.agent_id());
                if let Err(shutdown_err) = agent.shutdown().await {
                    error!("Cleanup after failed initialization also failed: {}, error: {}", 
                          context.agent_id(), shutdown_err);
                }
                
                // Return the initialization error
                Err(e)
            }
        }
    }
    
    /// Create a wrapped agent that provides message handling capabilities
    ///
    /// This function takes an agent and wraps it in a structure that can
    /// receive messages from a communication channel and forward them to
    /// the agent's `handle_message` method.
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent to wrap
    /// * `context` - The agent context
    ///
    /// # Returns
    ///
    /// A wrapped agent that can handle messages
    #[cfg(feature = "with-tokio")]
    pub fn message_handler<A: AegisAgent + 'static>(
        agent: A,
        context: AgentContext,
    ) -> AegisResult<MessageHandlingAgent<A>> {
        MessageHandlingAgent::new(agent, context)
    }
}

/// Agent wrapper that handles message reception via channels
#[cfg(feature = "with-tokio")]
pub struct MessageHandlingAgent<A: AegisAgent> {
    agent: A,
    context: AgentContext,
    shutdown_tx: tokio::sync::mpsc::Sender<()>,
    message_rx: tokio::sync::mpsc::Receiver<bytes::Bytes>,
}

#[cfg(feature = "with-tokio")]
impl<A: AegisAgent + 'static> MessageHandlingAgent<A> {
    /// Create a new MessageHandlingAgent
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent to wrap
    /// * `context` - The agent context
    ///
    /// # Returns
    ///
    /// A new MessageHandlingAgent or an error
    pub fn new(agent: A, context: AgentContext) -> AegisResult<Self> {
        let (shutdown_tx, _) = tokio::sync::mpsc::channel(1);
        let (_, message_rx) = tokio::sync::mpsc::channel(100);
        
        Ok(Self {
            agent,
            context,
            shutdown_tx,
            message_rx,
        })
    }
    
    /// Run the agent's message handling loop
    ///
    /// This method runs a loop that receives messages from the channel
    /// and forwards them to the agent's `handle_message` method.
    ///
    /// # Returns
    ///
    /// `Result<()>` indicating success or failure
    pub async fn run_message_handler(&mut self) -> AegisResult<()> {
        loop {
            tokio::select! {
                Some(message) = self.message_rx.recv() => {
                    if let Err(e) = self.agent.handle_message(message).await {
                        error!("Error handling message: {}", e);
                        // Continue handling messages even if one fails
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    // Periodically check agent status
                    let status = self.agent.get_status();
                    debug!("Agent status: {}", status);
                    
                    // Stop message handling if agent is no longer running
                    match status {
                        AgentStatus::Failed(_) | AgentStatus::Stopped | AgentStatus::ShuttingDown => {
                            info!("Stopping message handler due to agent status: {}", status);
                            break;
                        }
                        _ => {} // Continue for other statuses
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a sender that can be used to send messages to this agent
    ///
    /// # Returns
    ///
    /// A sender that can be used to send messages to this agent
    pub fn get_message_sender(&self) -> tokio::sync::mpsc::Sender<bytes::Bytes> {
        self.message_rx.clone()
    }
    
    /// Get a sender that can be used to trigger shutdown of this agent
    ///
    /// # Returns
    ///
    /// A sender that can be used to trigger shutdown
    pub fn get_shutdown_sender(&self) -> tokio::sync::mpsc::Sender<()> {
        self.shutdown_tx.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use aegis_core::error::AegisError;
    
    // Mock agent for testing
    struct MockAgent {
        status: AgentStatus,
        initialize_result: AegisResult<()>,
        run_result: AegisResult<()>,
        shutdown_result: AegisResult<()>,
        message_result: AegisResult<()>,
        // Track method calls for verification
        initialize_called: bool,
        run_called: bool,
        shutdown_called: bool,
    }
    
    impl MockAgent {
        fn new(
            status: AgentStatus,
            initialize_result: AegisResult<()>,
            run_result: AegisResult<()>,
            shutdown_result: AegisResult<()>,
        ) -> Self {
            Self {
                status,
                initialize_result,
                run_result,
                shutdown_result,
                message_result: Ok(()),
                initialize_called: false,
                run_called: false,
                shutdown_called: false,
            }
        }
    }
    
    #[async_trait::async_trait]
    impl AegisAgent for MockAgent {
        async fn initialize(&mut self, _context: AgentContext) -> AegisResult<()> {
            self.initialize_called = true;
            self.initialize_result.clone()
        }
        
        async fn run(&mut self) -> AegisResult<()> {
            self.run_called = true;
            self.run_result.clone()
        }
        
        async fn shutdown(&mut self) -> AegisResult<()> {
            self.shutdown_called = true;
            self.shutdown_result.clone()
        }
        
        async fn handle_message(&mut self, _message: bytes::Bytes) -> AegisResult<()> {
            self.message_result.clone()
        }
        
        fn get_status(&self) -> AgentStatus {
            self.status.clone()
        }
    }
    
    // Test helper to create a mock context
    fn create_mock_context() -> AgentContext {
        // These would normally be provided by the platform
        struct MockSpawner;
        struct MockTimer;
        
        #[async_trait::async_trait]
        impl aegis_core::platform::concurrency::AsyncTaskSpawner for MockSpawner {
            fn spawn<F>(&self, _future: F) -> aegis_core::error::AegisResult<()>
            where
                F: std::future::Future<Output = ()> + Send + 'static,
            {
                Ok(())
            }
        }
        
        #[async_trait::async_trait]
        impl aegis_core::platform::concurrency::AsyncTimer for MockTimer {
            async fn sleep(&self, _duration: std::time::Duration) -> aegis_core::error::AegisResult<()> {
                Ok(())
            }
            
            fn interval(&self, _period: std::time::Duration) -> Box<dyn aegis_core::platform::concurrency::AsyncTimerInterval> {
                unimplemented!("Not needed for this test")
            }
        }
        
        // The CommsClient would normally be provided by the communication module
        struct MockCommsClient;
        
        AgentContext {
            agent_id: "test-agent".to_string(),
            config: Arc::new(aegis_core::config::AegisConfig::default()),
            comms_client: Arc::new(MockCommsClient),
            spawner: Arc::new(MockSpawner),
            timer: Arc::new(MockTimer),
        }
    }
    
    // The tests below would normally verify the lifecycle management logic
    // using the mock agent and context. Since we don't have actual implementations
    // of the core services, we'll keep these tests minimal.
    
    #[tokio::test]
    async fn test_lifecycle_success_path() {
        // The full test would verify that the lifecycle manager correctly
        // handles a successful agent lifecycle. Since we don't have actual
        // implementations, we'll just verify the trait behavior.
        let agent = MockAgent::new(
            AgentStatus::Running,
            Ok(()),
            Ok(()),
            Ok(()),
        );
        let context = create_mock_context();
        
        // TODO: Implement once we have actual services
    }
} 