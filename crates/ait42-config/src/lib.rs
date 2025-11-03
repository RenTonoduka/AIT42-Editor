//! AIT42 Configuration Management
//!
//! Handles loading and managing editor configuration from files and defaults.

pub mod defaults;
pub mod loader;
pub mod schema;
pub mod watch;

// Re-exports
pub use defaults::default_config;
pub use loader::ConfigLoader;
pub use schema::{
    AIT42Config, EditorConfig, KeyBindingConfig, LspServerConfig, ThemeConfig, Config as EditorConfiguration,
};
pub use watch::ConfigWatcher;

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    NotFound(PathBuf),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

/// Main configuration structure (re-exported from schema)
pub use schema::Config;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_types() {
        let err = ConfigError::NotFound(PathBuf::from("/test"));
        assert!(err.to_string().contains("/test"));

        let err = ConfigError::ValidationError("invalid value".to_string());
        assert!(err.to_string().contains("invalid value"));
    }
}
