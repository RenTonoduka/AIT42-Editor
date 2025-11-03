//! Tauri Command Modules
//!
//! This module contains all Tauri commands organized by functionality.

pub mod file;
pub mod editor;
pub mod lsp;
pub mod git;
pub mod plugin;

#[cfg(feature = "terminal")]
pub mod terminal;

// Re-export commands
pub use file::*;
pub use editor::*;
pub use lsp::*;
pub use git::*;
pub use plugin::*;

#[cfg(feature = "terminal")]
pub use terminal::*;

#[cfg(test)]
mod file_tests;
