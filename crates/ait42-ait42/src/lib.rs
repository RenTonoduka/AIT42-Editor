//! AIT42 Agent Integration
//!
//! Complete integration with the AIT42 agent system for AI-powered development tasks.
//!
//! # Features
//!
//! - **Agent Registry**: Discover and manage 49 AI agents
//! - **Tmux Session Management**: Run agents in isolated tmux sessions
//! - **Coordinator**: Intelligent agent selection and orchestration
//! - **Executor**: Single, parallel, and sequential execution modes
//! - **Output Streaming**: Real-time output from running agents
//! - **Command Palette**: Integration with editor commands
//! - **Editor Bridge**: Run agents on buffer content and selections
//!
//! # Quick Start
//!
//! ```no_run
//! use ait42_ait42::{config::AIT42Config, coordinator::Coordinator, executor::AgentExecutor};
//!
//! # async fn example() -> ait42_ait42::error::Result<()> {
//! // Load configuration
//! let config = AIT42Config::load()?;
//!
//! // Create coordinator and executor
//! let coordinator = Coordinator::new(config)?;
//! let mut executor = AgentExecutor::new(coordinator);
//!
//! // Execute a task
//! let results = executor.execute_single("backend-developer", "Implement user authentication").await?;
//! println!("Output: {}", results.output);
//! # Ok(())
//! # }
//! ```

// Public modules
pub mod commands;
pub mod config;
pub mod coordinator;
pub mod editor_integration;
pub mod error;
pub mod executor;
pub mod registry;
pub mod stream;
pub mod tmux;

// Re-exports for convenience
pub use commands::{AgentCommand, CommandResult};
pub use config::AIT42Config;
pub use coordinator::{Coordinator, ExecutionResult};
pub use editor_integration::EditorAgentBridge;
pub use error::{AIT42Error, Result};
pub use executor::{AgentExecutor, ExecutionMode};
pub use registry::{AgentCategory, AgentMetadata, AgentRegistry};
pub use stream::{OutputStream, SessionStream, StreamEvent, StreamManager};
pub use tmux::{SessionStatus, TmuxManager, TmuxSession};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::commands::{AgentCommand, CommandResult};
    pub use crate::config::AIT42Config;
    pub use crate::coordinator::{Coordinator, ExecutionResult};
    pub use crate::error::{AIT42Error, Result};
    pub use crate::executor::{AgentExecutor, ExecutionMode};
    pub use crate::registry::{AgentCategory, AgentMetadata};
    pub use crate::tmux::{SessionStatus, TmuxSession};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Ensure all modules are accessible
        let _ = std::any::type_name::<AIT42Config>();
        let _ = std::any::type_name::<Coordinator>();
        let _ = std::any::type_name::<AgentExecutor>();
        let _ = std::any::type_name::<AgentRegistry>();
        let _ = std::any::type_name::<TmuxManager>();
    }
}
