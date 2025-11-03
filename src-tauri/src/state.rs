//! Application State Management
//!
//! Manages the global application state shared between Tauri commands.
//! Uses Arc<Mutex<T>> for thread-safe access.

use std::sync::{Arc, Mutex};
use ait42_config::Config;
use ait42_core::{Editor, EditorConfig, EditorState, buffer::BufferManager};
use ait42_lsp::client::LspClient;

// Import TerminalExecutor from ait42-tui if available
// Note: This will compile if ait42-tui is in dependencies
#[cfg(feature = "terminal")]
use ait42_tui::terminal_executor::TerminalExecutor;

/// Application-wide state
pub struct AppState {
    /// Editor instance with buffer management
    pub editor: Arc<Mutex<Editor>>,

    /// Editor state with undo/redo history and mode management
    pub editor_state: Arc<Mutex<EditorState>>,

    /// Legacy buffer manager (for backward compatibility)
    pub buffer_manager: Mutex<BufferManager>,

    /// Configuration
    pub config: Mutex<Config>,

    /// LSP clients
    pub lsp_clients: Mutex<Vec<LspClient>>,

    /// Terminal executor (optional feature) - uses tokio::sync::Mutex for async
    #[cfg(feature = "terminal")]
    pub terminal: Arc<tokio::sync::Mutex<TerminalExecutor>>,
}

impl AppState {
    /// Create new application state
    ///
    /// # Arguments
    /// * `working_dir` - Initial working directory for terminal (if terminal feature enabled)
    ///
    /// # Returns
    /// * `Ok(state)` - Initialized application state
    /// * `Err(e)` - Error during initialization
    pub fn new(#[allow(unused_variables)] working_dir: std::path::PathBuf) -> anyhow::Result<Self> {
        let editor_config = EditorConfig::default();
        let editor = Editor::new(editor_config)?;
        let editor_state = EditorState::new();

        Ok(Self {
            editor: Arc::new(Mutex::new(editor)),
            editor_state: Arc::new(Mutex::new(editor_state)),
            buffer_manager: Mutex::new(BufferManager::new()),
            config: Mutex::new(Config::default()),
            lsp_clients: Mutex::new(Vec::new()),
            #[cfg(feature = "terminal")]
            terminal: Arc::new(tokio::sync::Mutex::new(TerminalExecutor::new(working_dir))),
        })
    }

    /// Create default application state
    ///
    /// Uses current working directory for terminal.
    pub fn default_with_dir() -> anyhow::Result<Self> {
        let working_dir = std::env::current_dir()?;
        Self::new(working_dir)
    }
}

impl Default for AppState {
    fn default() -> Self {
        let working_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        Self::new(working_dir).expect("Failed to initialize AppState")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::default_with_dir();
        assert!(state.is_ok());

        let state = state.unwrap();

        // Verify editor can be locked
        assert!(state.editor.lock().is_ok());

        // Verify editor state can be locked
        assert!(state.editor_state.lock().is_ok());

        // Terminal uses tokio::sync::Mutex, test in async context if needed
        #[cfg(feature = "terminal")]
        {
            let _ = &state.terminal;
        }
    }

    #[tokio::test]
    async fn test_app_state_with_custom_dir() {
        let temp_dir = std::env::temp_dir();
        let state = AppState::new(temp_dir.clone());
        assert!(state.is_ok());

        #[cfg(feature = "terminal")]
        {
            let state = state.unwrap();
            let terminal = state.terminal.lock().await;
            assert_eq!(terminal.current_dir(), &temp_dir);
        }
    }
}
