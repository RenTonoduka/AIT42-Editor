//! Configuration Schema
//!
//! Defines the structure of the editor configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub editor: EditorConfig,

    #[serde(default)]
    pub theme: ThemeConfig,

    #[serde(default)]
    pub keybindings: KeyBindingConfig,

    #[serde(default)]
    pub lsp: HashMap<String, LspServerConfig>,

    #[serde(default)]
    pub ait42: AIT42Config,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            theme: ThemeConfig::default(),
            keybindings: KeyBindingConfig::default(),
            lsp: default_lsp_config(),
            ait42: AIT42Config::default(),
        }
    }
}

/// Editor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Tab size in spaces
    #[serde(default = "default_tab_size")]
    pub tab_size: usize,

    /// Auto-save delay in milliseconds (0 = disabled)
    #[serde(default = "default_auto_save_delay")]
    pub auto_save_delay: u64,

    /// Show line numbers
    #[serde(default = "default_true")]
    pub line_numbers: bool,

    /// Show relative line numbers
    #[serde(default)]
    pub relative_line_numbers: bool,

    /// Enable word wrap
    #[serde(default)]
    pub word_wrap: bool,

    /// Insert spaces instead of tabs
    #[serde(default = "default_true")]
    pub insert_spaces: bool,

    /// Highlight current line
    #[serde(default = "default_true")]
    pub highlight_current_line: bool,

    /// Show whitespace characters
    #[serde(default)]
    pub show_whitespace: bool,

    /// Cursor style: "block", "line", "underline"
    #[serde(default = "default_cursor_style")]
    pub cursor_style: String,

    /// Scroll offset (lines to keep visible above/below cursor)
    #[serde(default = "default_scroll_offset")]
    pub scroll_offset: usize,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: 4,
            auto_save_delay: 5000,
            line_numbers: true,
            relative_line_numbers: false,
            word_wrap: false,
            insert_spaces: true,
            highlight_current_line: true,
            show_whitespace: false,
            cursor_style: "block".to_string(),
            scroll_offset: 5,
        }
    }
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Theme name
    #[serde(default = "default_theme")]
    pub name: String,

    /// Color overrides
    #[serde(default)]
    pub colors: HashMap<String, String>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self::monokai()
    }
}

impl ThemeConfig {
    pub fn monokai() -> Self {
        let mut colors = HashMap::new();
        colors.insert("background".to_string(), "#272822".to_string());
        colors.insert("foreground".to_string(), "#F8F8F2".to_string());
        colors.insert("selection".to_string(), "#49483E".to_string());
        colors.insert("comment".to_string(), "#75715E".to_string());
        colors.insert("keyword".to_string(), "#F92672".to_string());
        colors.insert("string".to_string(), "#E6DB74".to_string());
        colors.insert("function".to_string(), "#A6E22E".to_string());
        colors.insert("variable".to_string(), "#FD971F".to_string());

        Self {
            name: "monokai".to_string(),
            colors,
        }
    }

    pub fn gruvbox_dark() -> Self {
        let mut colors = HashMap::new();
        colors.insert("background".to_string(), "#282828".to_string());
        colors.insert("foreground".to_string(), "#ebdbb2".to_string());
        colors.insert("selection".to_string(), "#504945".to_string());
        colors.insert("comment".to_string(), "#928374".to_string());
        colors.insert("keyword".to_string(), "#fb4934".to_string());
        colors.insert("string".to_string(), "#b8bb26".to_string());
        colors.insert("function".to_string(), "#fabd2f".to_string());
        colors.insert("variable".to_string(), "#fe8019".to_string());

        Self {
            name: "gruvbox-dark".to_string(),
            colors,
        }
    }
}

/// Key binding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindingConfig {
    /// Key binding mode: "vim", "emacs", "default"
    #[serde(default = "default_keybinding_mode")]
    pub mode: String,

    /// Custom key bindings
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

impl Default for KeyBindingConfig {
    fn default() -> Self {
        Self::vim()
    }
}

impl KeyBindingConfig {
    pub fn vim() -> Self {
        Self {
            mode: "vim".to_string(),
            custom: HashMap::new(),
        }
    }

    pub fn emacs() -> Self {
        Self {
            mode: "emacs".to_string(),
            custom: HashMap::new(),
        }
    }

    pub fn default_bindings() -> Self {
        Self {
            mode: "default".to_string(),
            custom: HashMap::new(),
        }
    }
}

/// LSP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspServerConfig {
    /// Command to execute
    pub command: String,

    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,

    /// Server settings
    #[serde(default)]
    pub settings: serde_json::Value,
}

/// AIT42 agent integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIT42Config {
    /// Path to AIT42 agents directory
    #[serde(default = "default_agents_path")]
    pub agents_path: PathBuf,

    /// Enable tmux session management
    #[serde(default = "default_true")]
    pub tmux_enabled: bool,

    /// Automatically start Coordinator agent
    #[serde(default = "default_true")]
    pub auto_coordinator: bool,

    /// Default agent to use
    #[serde(default)]
    pub default_agent: Option<String>,

    /// Agent-specific settings
    #[serde(default)]
    pub agent_settings: HashMap<String, serde_json::Value>,
}

impl Default for AIT42Config {
    fn default() -> Self {
        Self {
            agents_path: PathBuf::from("../.claude/agents"),
            tmux_enabled: true,
            auto_coordinator: true,
            default_agent: None,
            agent_settings: HashMap::new(),
        }
    }
}

// Default value functions for serde
fn default_tab_size() -> usize {
    4
}

fn default_auto_save_delay() -> u64 {
    5000
}

fn default_true() -> bool {
    true
}

fn default_cursor_style() -> String {
    "block".to_string()
}

fn default_scroll_offset() -> usize {
    5
}

fn default_theme() -> String {
    "monokai".to_string()
}

fn default_keybinding_mode() -> String {
    "vim".to_string()
}

fn default_agents_path() -> PathBuf {
    PathBuf::from("../.claude/agents")
}

/// Default LSP configuration
pub fn default_lsp_config() -> HashMap<String, LspServerConfig> {
    let mut config = HashMap::new();

    // Rust
    config.insert(
        "rust".to_string(),
        LspServerConfig {
            command: "rust-analyzer".to_string(),
            args: vec![],
            settings: serde_json::json!({
                "rust-analyzer": {
                    "checkOnSave": {
                        "command": "clippy"
                    }
                }
            }),
        },
    );

    // TypeScript/JavaScript
    config.insert(
        "typescript".to_string(),
        LspServerConfig {
            command: "typescript-language-server".to_string(),
            args: vec!["--stdio".to_string()],
            settings: serde_json::json!({}),
        },
    );

    config.insert(
        "javascript".to_string(),
        LspServerConfig {
            command: "typescript-language-server".to_string(),
            args: vec!["--stdio".to_string()],
            settings: serde_json::json!({}),
        },
    );

    // Python
    config.insert(
        "python".to_string(),
        LspServerConfig {
            command: "pylsp".to_string(),
            args: vec![],
            settings: serde_json::json!({}),
        },
    );

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.editor.tab_size, 4);
        assert!(config.editor.line_numbers);
        assert!(config.editor.insert_spaces);
        assert_eq!(config.theme.name, "monokai");
        assert_eq!(config.keybindings.mode, "vim");
    }

    #[test]
    fn test_editor_config() {
        let config = EditorConfig::default();

        assert_eq!(config.tab_size, 4);
        assert_eq!(config.auto_save_delay, 5000);
        assert!(config.line_numbers);
        assert!(!config.relative_line_numbers);
        assert!(config.insert_spaces);
        assert_eq!(config.cursor_style, "block");
    }

    #[test]
    fn test_theme_monokai() {
        let theme = ThemeConfig::monokai();

        assert_eq!(theme.name, "monokai");
        assert!(theme.colors.contains_key("background"));
        assert_eq!(theme.colors.get("background"), Some(&"#272822".to_string()));
    }

    #[test]
    fn test_theme_gruvbox() {
        let theme = ThemeConfig::gruvbox_dark();

        assert_eq!(theme.name, "gruvbox-dark");
        assert!(theme.colors.contains_key("foreground"));
    }

    #[test]
    fn test_keybinding_modes() {
        let vim = KeyBindingConfig::vim();
        assert_eq!(vim.mode, "vim");

        let emacs = KeyBindingConfig::emacs();
        assert_eq!(emacs.mode, "emacs");

        let default = KeyBindingConfig::default_bindings();
        assert_eq!(default.mode, "default");
    }

    #[test]
    fn test_ait42_config() {
        let config = AIT42Config::default();

        assert!(config.tmux_enabled);
        assert!(config.auto_coordinator);
        assert_eq!(config.agents_path, PathBuf::from("../.claude/agents"));
    }

    #[test]
    fn test_lsp_config() {
        let lsp = default_lsp_config();

        assert!(lsp.contains_key("rust"));
        assert!(lsp.contains_key("typescript"));
        assert!(lsp.contains_key("python"));

        let rust_config = lsp.get("rust").unwrap();
        assert_eq!(rust_config.command, "rust-analyzer");
    }

    #[test]
    fn test_serialization() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();

        assert!(toml.contains("tab_size"));
        assert!(toml.contains("monokai"));
    }

    #[test]
    fn test_deserialization() {
        let toml = r#"
[editor]
tab_size = 2
line_numbers = false

[theme]
name = "gruvbox-dark"

[keybindings]
mode = "emacs"
"#;

        let config: Config = toml::from_str(toml).unwrap();

        assert_eq!(config.editor.tab_size, 2);
        assert!(!config.editor.line_numbers);
        assert_eq!(config.theme.name, "gruvbox-dark");
        assert_eq!(config.keybindings.mode, "emacs");
    }
}
