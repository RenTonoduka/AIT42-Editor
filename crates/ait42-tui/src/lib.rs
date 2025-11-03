//! AIT42 TUI - Terminal User Interface
//!
//! Provides the terminal-based UI layer using ratatui.
//!
//! # Architecture
//!
//! ```text
//! TuiApp
//!   ├── EventLoop (async event handling)
//!   ├── Renderer (terminal rendering)
//!   ├── KeyMap (key bindings)
//!   ├── Theme (color schemes)
//!   └── Widgets
//!       ├── EditorWidget
//!       ├── StatusLine
//!       └── CommandPalette
//! ```
//!
//! # Example
//!
//! ```no_run
//! use ait42_tui::TuiApp;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut app = TuiApp::new().await?;
//!     app.run().await?;
//!     Ok(())
//! }
//! ```

pub mod event;
pub mod keybinds;
pub mod layout;
pub mod renderer;
pub mod state;
pub mod terminal_executor;
pub mod theme;
pub mod themes;
pub mod tui_app;
pub mod widgets;

// Re-exports
pub use event::{EditorEvent, EventLoop};
pub use keybinds::{EditorCommand, KeyBinding, KeyMap, Mode};
pub use layout::{EditorLayout, LayoutConfig};
pub use renderer::Renderer;
pub use state::EditorState as Phase10bEditorState;
pub use terminal_executor::TerminalExecutor;
pub use theme::Theme;
pub use themes::{CursorTheme, DefaultTheme, Theme as ThemeTrait};
pub use tui_app::{EditorState, TuiApp};
pub use widgets::{
    editor::ViewState, CommandPalette, EditorWidget, Sidebar, StatusLine, TabBar, TerminalPanel,
};

use anyhow::Result;
use tracing::info;

/// Run the TUI application with default settings
pub async fn run() -> Result<()> {
    info!("Initializing AIT42 TUI");

    let mut app = TuiApp::new().await?;
    app.run().await?;

    Ok(())
}

/// Run TUI with a specific file
pub async fn run_with_file(path: std::path::PathBuf) -> Result<()> {
    info!("Initializing AIT42 TUI with file: {:?}", path);

    let mut app = TuiApp::new().await?;
    app.load_file(path)?;
    app.run().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Ensure all modules compile
        assert!(true);
    }
}
