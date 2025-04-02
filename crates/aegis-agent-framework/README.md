# Aegis Agent Framework

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

The core agent framework for the Aegis platform, providing abstractions and lifecycle management for agents.

## Overview

The Aegis Agent Framework defines the core abstractions that all agents in the Aegis ecosystem must implement. It provides:

- The `AegisAgent` trait that defines the contract for all agents
- The `AgentContext` struct that provides access to framework services
- The `AgentLifecycleManager` for managing the agent lifecycle

## Key Components

### AegisAgent Trait

The `AegisAgent` trait defines the contract that all agents must implement:

```rust
#[async_trait]
pub trait AegisAgent: Send + Sync {
    async fn initialize(&mut self, context: AgentContext) -> AegisResult<()>;
    async fn run(&mut self) -> AegisResult<()>;
    async fn shutdown(&mut self) -> AegisResult<()>;
    async fn handle_message(&mut self, message: Bytes) -> AegisResult<()>;
    fn get_status(&self) -> AgentStatus;
}
```

### AgentContext

The `AgentContext` provides agents with access to framework services:

```rust
#[derive(Clone)]
pub struct AgentContext {
    pub agent_id: AgentID,
    pub config: Arc<aegis_core::config::AegisConfig>,
    pub comms_client: Arc<aegis_comms::CommsClient>,
    pub spawner: Arc<dyn aegis_core::platform::concurrency::AsyncTaskSpawner>,
    pub timer: Arc<dyn aegis_core::platform::concurrency::AsyncTimer>,
}
```

### Agent Lifecycle

The `AgentLifecycleManager` handles the complete lifecycle of an agent:

1. **Initialization**: The agent is initialized with its context
2. **Execution**: The agent's main logic is executed
3. **Shutdown**: The agent is gracefully shut down

## Usage

To create an agent using the Aegis Agent Framework:

```rust
use aegis_agent_framework::prelude::*;

struct MyAgent {
    status: AgentStatus,
    context: Option<AgentContext>,
}

#[async_trait::async_trait]
impl AegisAgent for MyAgent {
    async fn initialize(&mut self, context: AgentContext) -> AegisResult<()> {
        self.context = Some(context);
        self.status = AgentStatus::Running;
        Ok(())
    }
    
    async fn run(&mut self) -> AegisResult<()> {
        // Main agent logic here
        Ok(())
    }
    
    async fn shutdown(&mut self) -> AegisResult<()> {
        self.status = AgentStatus::Stopped;
        Ok(())
    }
    
    async fn handle_message(&mut self, message: Bytes) -> AegisResult<()> {
        // Handle incoming message
        Ok(())
    }
    
    fn get_status(&self) -> AgentStatus {
        self.status.clone()
    }
}

// To run the agent
async fn main() -> AegisResult<()> {
    let agent = MyAgent { 
        status: AgentStatus::Initializing,
        context: None,
    };
    
    // Create context with appropriate services
    let context = /* ... */;
    
    // Manage the agent lifecycle
    AgentLifecycleManager::manage(agent, context).await
}
```

## Features

- **with-tokio**: Enables Tokio-specific functionality, including the `MessageHandlingAgent`

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option. 