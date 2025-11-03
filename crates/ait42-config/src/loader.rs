//! Configuration Loader
//!
//! Handles loading and saving configuration files.

use crate::{Config, ConfigError, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

/// Configuration loader
pub struct ConfigLoader {
    config_path: PathBuf,
}

impl ConfigLoader {
    /// Create a new config loader with default path
    ///
    /// Default path: `~/.config/ait42-editor/config.toml`
    pub fn new() -> Result<Self> {
        let config_path = Self::default_config_path()?;
        Ok(Self { config_path })
    }

    /// Create a config loader with custom path
    pub fn with_path(path: PathBuf) -> Self {
        Self { config_path: path }
    }

    /// Get default configuration directory
    ///
    /// Returns `~/.config/ait42-editor/` on Unix
    /// Returns `%APPDATA%\ait42-editor\` on Windows
    pub fn default_config_dir() -> Result<PathBuf> {
        directories::ProjectDirs::from("com", "ait42", "ait42-editor")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .ok_or_else(|| ConfigError::ParseError("Cannot determine config directory".to_string()))
    }

    /// Get default configuration file path
    pub fn default_config_path() -> Result<PathBuf> {
        Ok(Self::default_config_dir()?.join("config.toml"))
    }

    /// Load configuration from file
    ///
    /// If file doesn't exist, returns default configuration.
    pub async fn load(&self) -> Result<Config> {
        if !self.exists().await {
            info!("Config file not found at {}, using defaults", self.config_path.display());
            return Ok(Config::default());
        }

        debug!("Loading config from {}", self.config_path.display());

        let content = fs::read_to_string(&self.config_path).await?;
        let config: Config = toml::from_str(&content)?;

        self.validate(&config)?;

        info!("Configuration loaded successfully");
        Ok(config)
    }

    /// Save configuration to file
    pub async fn save(&self, config: &Config) -> Result<()> {
        debug!("Saving config to {}", self.config_path.display());

        self.validate(config)?;

        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let toml = toml::to_string_pretty(config)?;
        fs::write(&self.config_path, toml).await?;

        info!("Configuration saved successfully");
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self, config: &Config) -> Result<()> {
        // Validate tab size
        if config.editor.tab_size == 0 || config.editor.tab_size > 16 {
            return Err(ConfigError::ValidationError(format!(
                "Invalid tab size: {} (must be 1-16)",
                config.editor.tab_size
            )));
        }

        // Validate cursor style
        let valid_cursor_styles = ["block", "line", "underline"];
        if !valid_cursor_styles.contains(&config.editor.cursor_style.as_str()) {
            return Err(ConfigError::ValidationError(format!(
                "Invalid cursor style: {} (must be one of: {})",
                config.editor.cursor_style,
                valid_cursor_styles.join(", ")
            )));
        }

        // Validate keybinding mode
        let valid_modes = ["vim", "emacs", "default"];
        if !valid_modes.contains(&config.keybindings.mode.as_str()) {
            return Err(ConfigError::ValidationError(format!(
                "Invalid keybinding mode: {} (must be one of: {})",
                config.keybindings.mode,
                valid_modes.join(", ")
            )));
        }

        // Validate agents path exists (warning only)
        if !config.ait42.agents_path.exists() {
            warn!("Agents path does not exist: {}", config.ait42.agents_path.display());
        }

        Ok(())
    }

    /// Check if config file exists
    pub async fn exists(&self) -> bool {
        fs::metadata(&self.config_path).await.is_ok()
    }

    /// Get config file path
    pub fn path(&self) -> &Path {
        &self.config_path
    }

    /// Create default configuration file
    pub async fn create_default(&self) -> Result<()> {
        info!("Creating default config file at {}", self.config_path.display());

        let config = Config::default();
        self.save(&config).await?;

        Ok(())
    }

    /// Load or create configuration
    ///
    /// If config file doesn't exist, creates it with defaults.
    pub async fn load_or_create(&self) -> Result<Config> {
        if !self.exists().await {
            self.create_default().await?;
        }

        self.load().await
    }

    /// Reset configuration to defaults
    pub async fn reset(&self) -> Result<()> {
        info!("Resetting configuration to defaults");

        let config = Config::default();
        self.save(&config).await?;

        Ok(())
    }

    /// Backup current configuration
    pub async fn backup(&self) -> Result<PathBuf> {
        if !self.exists().await {
            return Err(ConfigError::NotFound(self.config_path.clone()));
        }

        let backup_path = self.config_path.with_extension("toml.bak");
        fs::copy(&self.config_path, &backup_path).await?;

        info!("Config backed up to {}", backup_path.display());
        Ok(backup_path)
    }

    /// Restore from backup
    pub async fn restore_backup(&self) -> Result<()> {
        let backup_path = self.config_path.with_extension("toml.bak");

        if fs::metadata(&backup_path).await.is_err() {
            return Err(ConfigError::NotFound(backup_path));
        }

        fs::copy(&backup_path, &self.config_path).await?;

        info!("Config restored from backup");
        Ok(())
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create config loader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_load_default() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let loader = ConfigLoader::with_path(config_path);

        // Should return default config if file doesn't exist
        let config = loader.load().await.unwrap();
        assert_eq!(config.editor.tab_size, 4);
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let loader = ConfigLoader::with_path(config_path);

        // Save config
        let mut config = Config::default();
        config.editor.tab_size = 2;
        loader.save(&config).await.unwrap();

        // Load config
        let loaded = loader.load().await.unwrap();
        assert_eq!(loaded.editor.tab_size, 2);
    }

    #[tokio::test]
    async fn test_validation() {
        let loader = ConfigLoader::with_path(PathBuf::from("/tmp/test.toml"));

        // Valid config
        let config = Config::default();
        assert!(loader.validate(&config).is_ok());

        // Invalid tab size
        let mut config = Config::default();
        config.editor.tab_size = 0;
        assert!(loader.validate(&config).is_err());

        let mut config = Config::default();
        config.editor.tab_size = 20;
        assert!(loader.validate(&config).is_err());

        // Invalid cursor style
        let mut config = Config::default();
        config.editor.cursor_style = "invalid".to_string();
        assert!(loader.validate(&config).is_err());

        // Invalid keybinding mode
        let mut config = Config::default();
        config.keybindings.mode = "invalid".to_string();
        assert!(loader.validate(&config).is_err());
    }

    #[tokio::test]
    async fn test_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let loader = ConfigLoader::with_path(config_path.clone());

        assert!(!loader.exists().await);

        // Create file
        fs::write(&config_path, "").await.unwrap();
        assert!(loader.exists().await);
    }

    #[tokio::test]
    async fn test_create_default() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let loader = ConfigLoader::with_path(config_path.clone());

        loader.create_default().await.unwrap();
        assert!(loader.exists().await);

        let config = loader.load().await.unwrap();
        assert_eq!(config.editor.tab_size, 4);
    }

    #[tokio::test]
    async fn test_load_or_create() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let loader = ConfigLoader::with_path(config_path);

        let config = loader.load_or_create().await.unwrap();
        assert_eq!(config.editor.tab_size, 4);
        assert!(loader.exists().await);
    }

    #[tokio::test]
    async fn test_reset() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let loader = ConfigLoader::with_path(config_path);

        // Save custom config
        let mut config = Config::default();
        config.editor.tab_size = 8;
        loader.save(&config).await.unwrap();

        // Reset
        loader.reset().await.unwrap();

        // Load and verify defaults
        let config = loader.load().await.unwrap();
        assert_eq!(config.editor.tab_size, 4);
    }

    #[tokio::test]
    async fn test_backup_and_restore() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let loader = ConfigLoader::with_path(config_path);

        // Save original config
        let mut config = Config::default();
        config.editor.tab_size = 2;
        loader.save(&config).await.unwrap();

        // Backup
        let backup_path = loader.backup().await.unwrap();
        assert!(fs::metadata(&backup_path).await.is_ok());

        // Modify config
        config.editor.tab_size = 8;
        loader.save(&config).await.unwrap();

        // Restore
        loader.restore_backup().await.unwrap();

        // Verify restored
        let config = loader.load().await.unwrap();
        assert_eq!(config.editor.tab_size, 2);
    }

    #[test]
    fn test_default_paths() {
        let dir = ConfigLoader::default_config_dir();
        assert!(dir.is_ok());

        let path = ConfigLoader::default_config_path();
        assert!(path.is_ok());
    }
}
