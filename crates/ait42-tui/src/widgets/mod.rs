//! UI Widgets
//!
//! Reusable UI components for the editor.

pub mod command_palette;
pub mod editor;
pub mod statusline;

pub use command_palette::CommandPalette;
pub use editor::EditorWidget;
pub use statusline::StatusLine;
