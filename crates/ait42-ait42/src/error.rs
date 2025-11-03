//! Error types for AIT42 agent integration

use std::io;
use thiserror::Error;

/// Result type for AIT42 operations
pub type Result<T> = std::result::Result<T, AIT42Error>;

/// Error types for AIT42 agent integration
#[derive(Error, Debug)]
pub enum AIT42Error {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Tmux error: {0}")]
    TmuxError(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid agent metadata: {0}")]
    InvalidMetadata(String),

    #[error("Session timeout: {0}")]
    SessionTimeout(String),

    #[error("Multiple errors occurred: {0}")]
    Multiple(String),
}

impl AIT42Error {
    /// Create a multiple error from a list of errors
    pub fn multiple(errors: Vec<AIT42Error>) -> Self {
        let msg = errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        AIT42Error::Multiple(msg)
    }
}
