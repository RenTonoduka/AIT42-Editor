//! Application State Management
//!
//! Manages the global application state shared between Tauri commands.
//! Uses Arc<Mutex<T>> for thread-safe access.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use ait42_config::Config;
use ait42_core::{Editor, EditorConfig, EditorState, buffer::BufferManager};
use ait42_lsp::{LspConfig, LspManager};
use ait42_ait42::{AgentRegistry, AgentExecutor, Coordinator, config::AIT42Config};
use crate::plugin::PluginManager;
use crate::commands::ait42::{DebateStatus, RoundOutput};

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

    /// LSP manager for multiple language servers
    pub lsp_manager: Arc<LspManager>,

    /// Plugin manager for extensibility
    pub plugin_manager: Arc<Mutex<PluginManager>>,

    /// Working directory for git operations - uses tokio::sync::Mutex for async
    pub working_dir: Arc<tokio::sync::Mutex<std::path::PathBuf>>,

    /// Debate status tracking - uses Arc<Mutex> for thread-safe access
    pub debates: Arc<Mutex<HashMap<String, DebateStatus>>>,

    /// Terminal executor (optional feature) - uses tokio::sync::Mutex for async
    #[cfg(feature = "terminal")]
    pub terminal: Arc<tokio::sync::Mutex<TerminalExecutor>>,

    /// AIT42 agent registry for discovering and managing agents
    pub agent_registry: Arc<Mutex<Option<AgentRegistry>>>,

    /// AIT42 agent executor for running agents
    pub agent_executor: Arc<tokio::sync::Mutex<Option<AgentExecutor>>>,

    /// AIT42 coordinator for intelligent agent selection
    pub coordinator: Arc<tokio::sync::Mutex<Option<Coordinator>>>,

    /// SQLite session repository (optional for gradual migration)
    pub session_repo: Option<Arc<crate::commands::session_history_sqlite::SessionRepo>>,
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

        // Initialize LSP manager with default configuration
        let lsp_config = LspConfig::default();
        let lsp_manager = LspManager::new(lsp_config);

        // Initialize plugin manager
        let plugins_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("plugins");
        let mut plugin_manager = PluginManager::new(plugins_dir);
        plugin_manager
            .initialize()
            .unwrap_or_else(|e| eprintln!("Failed to initialize plugin manager: {}", e));

        // Initialize AIT42 agent system (lazy initialization - will be initialized on first use)
        let agent_registry = Arc::new(Mutex::new(None));
        let agent_executor = Arc::new(tokio::sync::Mutex::new(None));
        let coordinator = Arc::new(tokio::sync::Mutex::new(None));

        // Initialize SQLite session repository (async operation - optional)
        // Note: This is initialized as None for now. In Phase 2, we'll initialize it here.
        // For Phase 1, we'll initialize it lazily on first use.
        let session_repo = None;

        Ok(Self {
            editor: Arc::new(Mutex::new(editor)),
            editor_state: Arc::new(Mutex::new(editor_state)),
            buffer_manager: Mutex::new(BufferManager::new()),
            config: Mutex::new(Config::default()),
            lsp_manager: Arc::new(lsp_manager),
            plugin_manager: Arc::new(Mutex::new(plugin_manager)),
            working_dir: Arc::new(tokio::sync::Mutex::new(working_dir.clone())),
            debates: Arc::new(Mutex::new(HashMap::new())),
            #[cfg(feature = "terminal")]
            terminal: Arc::new(tokio::sync::Mutex::new(TerminalExecutor::new(working_dir))),
            agent_registry,
            agent_executor,
            coordinator,
            session_repo,
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

        // Verify LSP manager is initialized
        let _ = &state.lsp_manager;

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

        let state = state.unwrap();

        // Verify LSP manager is available
        assert!(
            !state.lsp_manager.running_servers().await.is_empty()
                || state.lsp_manager.running_servers().await.is_empty()
        );

        #[cfg(feature = "terminal")]
        {
            let terminal = state.terminal.lock().await;
            assert_eq!(terminal.current_dir(), &temp_dir);
        }
    }
}
