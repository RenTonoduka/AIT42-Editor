//! UI Widgets
//!
//! Reusable UI components for the editor.

pub mod command_palette;
pub mod editor;
pub mod sidebar;
pub mod statusline;
pub mod tab_bar;
pub mod terminal_panel;

pub use command_palette::CommandPalette;
pub use editor::EditorWidget;
pub use sidebar::{FileEntry, FileEntryType, FileTree, Sidebar};
pub use statusline::StatusLine;
pub use tab_bar::{Tab, TabBar};
pub use terminal_panel::TerminalPanel;
