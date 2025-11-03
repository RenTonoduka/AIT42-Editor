//! AIT42 LSP Client
//!
//! Language Server Protocol integration for intelligent code completion,
//! diagnostics, and other language features.

pub mod client;
pub mod config;
pub mod manager;
pub mod position;

// Re-exports
pub use client::{LspClient, LspClientBuilder};
pub use config::{LspConfig, LspServerConfig};
pub use manager::LspManager;
pub use position::{buffer_pos_to_lsp, lsp_pos_to_buffer};

use lsp_types::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LspError {
    #[error("LSP server not available for language: {0}")]
    ServerNotAvailable(String),

    #[error("LSP communication error: {0}")]
    CommunicationError(String),

    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Server process error: {0}")]
    ProcessError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Server not initialized")]
    NotInitialized,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub type Result<T> = std::result::Result<T, LspError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_types() {
        let err = LspError::ServerNotAvailable("rust".to_string());
        assert!(err.to_string().contains("rust"));

        let err = LspError::NotInitialized;
        assert!(err.to_string().contains("not initialized"));
    }
}
