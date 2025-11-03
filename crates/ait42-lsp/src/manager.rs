//! LSP Manager
//!
//! Manages multiple LSP clients for different programming languages.

use crate::{LspClient, LspClientBuilder, LspConfig, LspError, Result};
use lsp_types::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Manages LSP clients for multiple languages
pub struct LspManager {
    clients: Arc<RwLock<HashMap<String, Arc<LspClient>>>>,
    config: LspConfig,
}

impl LspManager {
    /// Create a new LSP manager
    pub fn new(config: LspConfig) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Start LSP server for a language
    ///
    /// If a server is already running for this language, it will be reused.
    pub async fn start_server(&self, language_id: &str) -> Result<()> {
        // Check if already running
        {
            let clients = self.clients.read().await;
            if clients.contains_key(language_id) {
                debug!("LSP server for {} already running", language_id);
                return Ok(());
            }
        }

        // Get server config
        let server_config = self
            .config
            .servers
            .get(language_id)
            .ok_or_else(|| LspError::ServerNotAvailable(language_id.to_string()))?;

        info!(
            "Starting LSP server for {}: {} {:?}",
            language_id, server_config.command, server_config.args
        );

        // Build and start client
        let mut builder = LspClientBuilder::new(&server_config.command);

        for arg in &server_config.args {
            builder = builder.arg(arg);
        }

        if let Some(root_uri) = &server_config.root_uri {
            builder = builder.root_uri(root_uri.clone());
        }

        let client = builder.build().await?;

        // Store client
        self.clients
            .write()
            .await
            .insert(language_id.to_string(), Arc::new(client));

        info!("LSP server for {} started successfully", language_id);
        Ok(())
    }

    /// Stop LSP server for a language
    pub async fn stop_server(&self, language_id: &str) -> Result<()> {
        let client = {
            let mut clients = self.clients.write().await;
            clients.remove(language_id)
        };

        if let Some(client) = client {
            info!("Stopping LSP server for {}", language_id);
            client.shutdown().await?;
        } else {
            warn!("No LSP server running for {}", language_id);
        }

        Ok(())
    }

    /// Get client for a language
    ///
    /// Returns None if no server is running for this language.
    pub async fn get_client(&self, language_id: &str) -> Option<Arc<LspClient>> {
        self.clients.read().await.get(language_id).cloned()
    }

    /// Ensure server is running for a language
    ///
    /// Starts the server if not already running.
    pub async fn ensure_server(&self, language_id: &str) -> Result<Arc<LspClient>> {
        self.start_server(language_id).await?;
        self.get_client(language_id)
            .await
            .ok_or_else(|| LspError::ServerNotAvailable(language_id.to_string()))
    }

    /// Detect language from file path
    pub fn detect_language(&self, path: &Path) -> Option<String> {
        let ext = path.extension()?.to_str()?;

        // Common language mappings
        let language = match ext {
            "rs" => "rust",
            "ts" | "tsx" => "typescript",
            "js" | "jsx" => "javascript",
            "py" => "python",
            "go" => "go",
            "c" | "h" => "c",
            "cpp" | "cc" | "cxx" | "hpp" => "cpp",
            "java" => "java",
            "rb" => "ruby",
            "php" => "php",
            "cs" => "csharp",
            "swift" => "swift",
            "kt" => "kotlin",
            "scala" => "scala",
            "hs" => "haskell",
            "ml" => "ocaml",
            "sh" | "bash" => "bash",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "toml" => "toml",
            "md" => "markdown",
            "html" => "html",
            "css" => "css",
            "scss" => "scss",
            "vue" => "vue",
            _ => return None,
        };

        Some(language.to_string())
    }

    /// Get or start server for file
    ///
    /// Detects language from file extension and ensures server is running.
    pub async fn ensure_server_for_file(&self, path: &Path) -> Result<Option<Arc<LspClient>>> {
        let language = match self.detect_language(path) {
            Some(lang) => lang,
            None => return Ok(None),
        };

        match self.ensure_server(&language).await {
            Ok(client) => Ok(Some(client)),
            Err(LspError::ServerNotAvailable(_)) => {
                warn!("No LSP server configured for {}", language);
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Shutdown all servers
    pub async fn shutdown_all(&self) -> Result<()> {
        info!("Shutting down all LSP servers");

        let clients: Vec<_> = {
            let clients = self.clients.write().await;
            clients.values().cloned().collect()
        };

        for client in clients {
            let _ = client.shutdown().await;
        }

        self.clients.write().await.clear();

        info!("All LSP servers shut down");
        Ok(())
    }

    /// Get list of running servers
    pub async fn running_servers(&self) -> Vec<String> {
        self.clients.read().await.keys().cloned().collect()
    }

    /// Check if server is running
    pub async fn is_running(&self, language_id: &str) -> bool {
        self.clients.read().await.contains_key(language_id)
    }
}

impl Drop for LspManager {
    fn drop(&mut self) {
        // Spawn background task to shutdown servers
        let clients = self.clients.clone();
        tokio::spawn(async move {
            let clients_vec: Vec<_> = {
                let clients = clients.write().await;
                clients.values().cloned().collect()
            };

            for client in clients_vec {
                let _ = client.shutdown().await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_language_detection() {
        let config = LspConfig::default();
        let manager = LspManager::new(config);

        assert_eq!(
            manager.detect_language(&PathBuf::from("test.rs")),
            Some("rust".to_string())
        );
        assert_eq!(
            manager.detect_language(&PathBuf::from("test.ts")),
            Some("typescript".to_string())
        );
        assert_eq!(
            manager.detect_language(&PathBuf::from("test.py")),
            Some("python".to_string())
        );
        assert_eq!(manager.detect_language(&PathBuf::from("test.unknown")), None);
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let config = LspConfig::default();
        let manager = LspManager::new(config);

        assert_eq!(manager.running_servers().await.len(), 0);
    }

    #[tokio::test]
    async fn test_is_running() {
        let config = LspConfig::default();
        let manager = LspManager::new(config);

        assert!(!manager.is_running("rust").await);
    }
}
