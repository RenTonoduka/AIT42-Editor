//! Configuration File Watcher
//!
//! Watches the configuration file for changes and reloads automatically.

use crate::{Config, ConfigLoader, Result};
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Configuration file watcher
pub struct ConfigWatcher {
    loader: ConfigLoader,
    _watcher: Arc<RecommendedWatcher>,
    rx: mpsc::Receiver<Config>,
}

impl ConfigWatcher {
    /// Create a new config watcher
    pub fn new(loader: ConfigLoader) -> Result<Self> {
        let (tx, rx) = mpsc::channel(10);
        let config_path = loader.path().to_path_buf();
        let loader_clone = ConfigLoader::with_path(config_path.clone());

        // Get current runtime handle
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|e| crate::ConfigError::ParseError(format!("No tokio runtime: {}", e)))?;

        // Create the notify watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                let tx = tx.clone();
                let loader = loader_clone.clone();
                let handle = handle.clone();

                // Spawn on the captured runtime handle
                handle.spawn(async move {
                    match res {
                        Ok(event) => {
                            // Only reload on modify events
                            if matches!(event.kind, notify::EventKind::Modify(_)) {
                                debug!("Config file changed, reloading");

                                match loader.load().await {
                                    Ok(config) => {
                                        if let Err(e) = tx.send(config).await {
                                            error!("Failed to send config update: {}", e);
                                        } else {
                                            info!("Configuration reloaded");
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to reload config: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Watch error: {}", e);
                        }
                    }
                });
            },
            NotifyConfig::default(),
        )
        .map_err(|e| crate::ConfigError::ParseError(format!("Failed to create watcher: {}", e)))?;

        // Watch the config file
        watcher
            .watch(&config_path, RecursiveMode::NonRecursive)
            .map_err(|e| {
                crate::ConfigError::ParseError(format!("Failed to watch config: {}", e))
            })?;

        info!("Watching config file: {}", config_path.display());

        Ok(Self {
            loader,
            _watcher: Arc::new(watcher),
            rx,
        })
    }

    /// Wait for configuration changes
    ///
    /// Returns the new configuration when the file is modified.
    pub async fn watch(&mut self) -> Option<Config> {
        self.rx.recv().await
    }

    /// Try to get configuration change without blocking
    pub fn try_watch(&mut self) -> Option<Config> {
        self.rx.try_recv().ok()
    }

    /// Get the config loader
    pub fn loader(&self) -> &ConfigLoader {
        &self.loader
    }
}

// Clone implementation for ConfigLoader
impl Clone for ConfigLoader {
    fn clone(&self) -> Self {
        Self::with_path(self.path().to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_config_watch() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create initial config
        let loader = ConfigLoader::with_path(config_path.clone());
        loader.create_default().await.unwrap();

        // Create watcher
        let mut watcher = ConfigWatcher::new(loader).unwrap();

        // Give watcher time to initialize
        sleep(Duration::from_millis(100)).await;

        // Modify config file
        let mut config = Config::default();
        config.editor.tab_size = 8;
        let toml = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, toml).await.unwrap();

        // Wait for change event
        sleep(Duration::from_millis(300)).await;

        // Try to get updated config
        if let Some(updated) = watcher.try_watch() {
            assert_eq!(updated.editor.tab_size, 8);
        }
    }

    #[tokio::test]
    async fn test_loader_clone() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let loader1 = ConfigLoader::with_path(config_path.clone());
        let loader2 = loader1.clone();

        assert_eq!(loader1.path(), loader2.path());
    }

    #[tokio::test]
    async fn test_multiple_changes() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let loader = ConfigLoader::with_path(config_path.clone());
        loader.create_default().await.unwrap();

        let mut watcher = ConfigWatcher::new(loader).unwrap();

        sleep(Duration::from_millis(100)).await;

        // Multiple changes
        for i in 2..=4 {
            let mut config = Config::default();
            config.editor.tab_size = i;
            let toml = toml::to_string_pretty(&config).unwrap();
            fs::write(&config_path, toml).await.unwrap();
            sleep(Duration::from_millis(200)).await;
        }

        // Should receive at least one update
        sleep(Duration::from_millis(200)).await;
        let updated = watcher.try_watch();
        assert!(updated.is_some());
    }
}
