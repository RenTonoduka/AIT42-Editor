//! Default Configuration
//!
//! Provides default configuration values.

use crate::schema::*;

/// Get default configuration
pub fn default_config() -> Config {
    Config::default()
}

/// Get minimal configuration (for testing)
pub fn minimal_config() -> Config {
    Config {
        editor: EditorConfig {
            tab_size: 4,
            auto_save_delay: 0, // Disabled
            line_numbers: false,
            relative_line_numbers: false,
            word_wrap: false,
            insert_spaces: true,
            highlight_current_line: false,
            show_whitespace: false,
            cursor_style: "block".to_string(),
            scroll_offset: 0,
        },
        theme: ThemeConfig {
            name: "default".to_string(),
            colors: std::collections::HashMap::new(),
        },
        keybindings: KeyBindingConfig::default_bindings(),
        lsp: std::collections::HashMap::new(),
        ait42: AIT42Config {
            agents_path: std::path::PathBuf::from(".claude/agents"),
            tmux_enabled: false,
            auto_coordinator: false,
            default_agent: None,
            agent_settings: std::collections::HashMap::new(),
        },
    }
}

/// Example configuration with comments (for documentation)
pub fn example_config_toml() -> String {
    r#"# AIT42 Editor Configuration

[editor]
# Tab size in spaces
tab_size = 4

# Auto-save delay in milliseconds (0 = disabled)
auto_save_delay = 5000

# Show line numbers
line_numbers = true

# Show relative line numbers
relative_line_numbers = false

# Enable word wrap
word_wrap = false

# Insert spaces instead of tabs
insert_spaces = true

# Highlight current line
highlight_current_line = true

# Show whitespace characters
show_whitespace = false

# Cursor style: "block", "line", "underline"
cursor_style = "block"

# Scroll offset (lines to keep visible above/below cursor)
scroll_offset = 5

[theme]
# Theme name: "monokai", "gruvbox-dark"
name = "monokai"

# Optional color overrides
# [theme.colors]
# background = "#272822"
# foreground = "#F8F8F2"

[keybindings]
# Keybinding mode: "vim", "emacs", "default"
mode = "vim"

# Custom key bindings
# [keybindings.custom]
# "Ctrl+s" = "save"
# "Ctrl+q" = "quit"

[lsp.rust]
command = "rust-analyzer"
args = []

[lsp.rust.settings.rust-analyzer.checkOnSave]
command = "clippy"

[lsp.typescript]
command = "typescript-language-server"
args = ["--stdio"]

[lsp.python]
command = "pylsp"
args = []

[ait42]
# Path to AIT42 agents directory
agents_path = "../.claude/agents"

# Enable tmux session management
tmux_enabled = true

# Automatically start Coordinator agent
auto_coordinator = true

# Default agent to use (optional)
# default_agent = "Coordinator"
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = default_config();
        assert_eq!(config.editor.tab_size, 4);
        assert!(config.editor.line_numbers);
        assert_eq!(config.theme.name, "monokai");
    }

    #[test]
    fn test_minimal_config() {
        let config = minimal_config();
        assert_eq!(config.editor.tab_size, 4);
        assert!(!config.editor.line_numbers);
        assert_eq!(config.editor.auto_save_delay, 0);
    }

    #[test]
    fn test_example_toml() {
        let toml = example_config_toml();
        assert!(toml.contains("tab_size"));
        assert!(toml.contains("monokai"));
        assert!(toml.contains("vim"));

        // Should be parseable
        let config: Result<Config, _> = toml::from_str(&toml);
        assert!(config.is_ok());
    }
}
