//! UI Widgets
//!
//! Reusable UI components for the editor.

pub mod editor;
pub mod statusline;
pub mod command_palette;

pub use editor::EditorWidget;
pub use statusline::StatusLine;
pub use command_palette::CommandPalette;
