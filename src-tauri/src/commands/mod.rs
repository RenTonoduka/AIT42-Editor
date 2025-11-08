//! Tauri Command Modules
//!
//! This module contains all Tauri commands organized by functionality.

pub mod file;
pub mod editor;
pub mod lsp;
pub mod git;
pub mod plugin;
pub mod ait42;
pub mod worktree;
pub mod optimizer;
pub mod session_history;
pub mod workspace;

#[cfg(feature = "terminal")]
pub mod terminal;

// Re-export commands
pub use file::*;
pub use editor::*;
pub use lsp::*;
pub use git::*;
pub use plugin::*;
pub use ait42::*;
pub use worktree::*;
pub use optimizer::*;
pub use session_history::*;
pub use workspace::*;

#[cfg(feature = "terminal")]
pub use terminal::*;

#[cfg(test)]
mod file_tests;

#[cfg(test)]
mod optimizer_tests;
