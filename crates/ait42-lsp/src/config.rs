//! LSP Configuration
//!
//! Configuration for LSP servers and their settings.

use lsp_types::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LSP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspConfig {
    /// Server configurations by language ID
    pub servers: HashMap<String, LspServerConfig>,
}

/// Configuration for a single LSP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspServerConfig {
    /// Command to execute the server
    pub command: String,

    /// Command-line arguments
    #[serde(default)]
    pub args: Vec<String>,

    /// Root URI for the workspace
    #[serde(default)]
    pub root_uri: Option<Url>,

    /// Additional settings passed to the server
    #[serde(default)]
    pub settings: serde_json::Value,
}

impl Default for LspConfig {
    fn default() -> Self {
        let mut servers = HashMap::new();

        // Rust
        servers.insert(
            "rust".to_string(),
            LspServerConfig {
                command: "rust-analyzer".to_string(),
                args: vec![],
                root_uri: None,
                settings: serde_json::json!({
                    "rust-analyzer": {
                        "checkOnSave": {
                            "command": "clippy"
                        }
                    }
                }),
            },
        );

        // TypeScript
        servers.insert(
            "typescript".to_string(),
            LspServerConfig {
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                root_uri: None,
                settings: serde_json::json!({}),
            },
        );

        // JavaScript
        servers.insert(
            "javascript".to_string(),
            LspServerConfig {
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                root_uri: None,
                settings: serde_json::json!({}),
            },
        );

        // Python
        servers.insert(
            "python".to_string(),
            LspServerConfig {
                command: "pylsp".to_string(),
                args: vec![],
                root_uri: None,
                settings: serde_json::json!({}),
            },
        );

        // Go
        servers.insert(
            "go".to_string(),
            LspServerConfig {
                command: "gopls".to_string(),
                args: vec![],
                root_uri: None,
                settings: serde_json::json!({}),
            },
        );

        // C/C++
        servers.insert(
            "c".to_string(),
            LspServerConfig {
                command: "clangd".to_string(),
                args: vec![],
                root_uri: None,
                settings: serde_json::json!({}),
            },
        );

        servers.insert(
            "cpp".to_string(),
            LspServerConfig {
                command: "clangd".to_string(),
                args: vec![],
                root_uri: None,
                settings: serde_json::json!({}),
            },
        );

        Self { servers }
    }
}

impl LspConfig {
    /// Load from TOML string
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Save to TOML string
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Add or update server configuration
    pub fn add_server(&mut self, language: String, config: LspServerConfig) {
        self.servers.insert(language, config);
    }

    /// Remove server configuration
    pub fn remove_server(&mut self, language: &str) -> Option<LspServerConfig> {
        self.servers.remove(language)
    }

    /// Get server configuration
    pub fn get_server(&self, language: &str) -> Option<&LspServerConfig> {
        self.servers.get(language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LspConfig::default();

        assert!(config.servers.contains_key("rust"));
        assert!(config.servers.contains_key("typescript"));
        assert!(config.servers.contains_key("python"));

        let rust_config = config.get_server("rust").unwrap();
        assert_eq!(rust_config.command, "rust-analyzer");
    }

    #[test]
    fn test_toml_serialization() {
        let mut config = LspConfig::default();

        // Keep only rust for simpler test
        config.servers.retain(|k, _| k == "rust");

        let toml = config.to_toml().unwrap();
        assert!(toml.contains("rust-analyzer"));

        let deserialized = LspConfig::from_toml(&toml).unwrap();
        assert_eq!(deserialized.servers.len(), 1);
        assert!(deserialized.servers.contains_key("rust"));
    }

    #[test]
    fn test_add_remove_server() {
        let mut config = LspConfig::default();

        // Add custom server
        config.add_server(
            "custom".to_string(),
            LspServerConfig {
                command: "custom-lsp".to_string(),
                args: vec!["--stdio".to_string()],
                root_uri: None,
                settings: serde_json::json!({}),
            },
        );

        assert!(config.servers.contains_key("custom"));

        // Remove server
        let removed = config.remove_server("custom");
        assert!(removed.is_some());
        assert!(!config.servers.contains_key("custom"));
    }

    #[test]
    fn test_toml_format() {
        let toml = r#"
[servers.rust]
command = "rust-analyzer"
args = []

[servers.rust.settings.rust-analyzer.checkOnSave]
command = "clippy"
"#;

        let config = LspConfig::from_toml(toml).unwrap();
        assert_eq!(
            config.servers.get("rust").unwrap().command,
            "rust-analyzer"
        );
    }
}
