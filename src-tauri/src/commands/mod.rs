//! Tauri Command Handlers
//!
//! This module contains all Tauri command handlers that bridge
//! the frontend (TypeScript/React) with the Rust backend.

pub mod editor;
pub mod file;

#[cfg(feature = "terminal")]
pub mod terminal;

// Re-export all commands for easy registration
pub use editor::*;
pub use file::*;

#[cfg(feature = "terminal")]
pub use terminal::*;
