//! AIT42 Configuration Management
//!
//! Handles loading and managing editor configuration from files and defaults.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    NotFound(PathBuf),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub editor: EditorConfig,
    pub ui: UiConfig,
    pub keybindings: KeybindingsConfig,
    pub ait42: Ait42Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub tab_size: usize,
    pub insert_spaces: bool,
    pub line_numbers: bool,
    pub relative_line_numbers: bool,
    pub word_wrap: bool,
    pub auto_save: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub font_size: u16,
    pub show_file_tree: bool,
    pub show_status_bar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    // TODO: Keybinding configuration
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ait42Config {
    pub agents_path: PathBuf,
    pub tmux_enabled: bool,
    pub auto_coordinator: bool,
}

impl Config {
    /// Load configuration from file
    pub async fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = tokio::fs::read_to_string(path.as_ref()).await?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        Ok(config)
    }

    /// Load default configuration
    pub async fn load_default() -> Result<Self> {
        Ok(Self::default())
    }

    /// Get config directory path
    pub fn config_dir() -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "ait42", "editor")
            .ok_or_else(|| ConfigError::ParseError("Cannot determine config directory".to_string()))?;
        Ok(dirs.config_dir().to_path_buf())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig {
                tab_size: 4,
                insert_spaces: true,
                line_numbers: true,
                relative_line_numbers: false,
                word_wrap: false,
                auto_save: true,
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                font_size: 14,
                show_file_tree: true,
                show_status_bar: true,
            },
            keybindings: KeybindingsConfig {},
            ait42: Ait42Config {
                agents_path: PathBuf::from("../.claude/agents"),
                tmux_enabled: true,
                auto_coordinator: true,
            },
        }
    }
}
