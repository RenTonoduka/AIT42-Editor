//! AIT42 Core - Editor Logic
//!
//! This crate contains the core editor functionality including:
//! - Buffer management (text storage and manipulation)
//! - Cursor and selection handling
//! - Edit operations and undo/redo
//! - Editor state management
//!
//! # Architecture
//!
//! ```text
//! Editor
//!   ├── BufferManager (manages multiple buffers)
//!   │   └── Buffer (text content with Rope data structure)
//!   ├── SelectionManager (cursor and selection state)
//!   ├── History (undo/redo stack)
//!   └── CommandExecutor (processes editor commands)
//! ```
//!
//! # Example
//!
//! ```no_run
//! use ait42_core::{Editor, EditorConfig};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = EditorConfig::default();
//! let mut editor = Editor::new(config)?;
//!
//! // Open a file
//! editor.open_file("src/main.rs").await?;
//!
//! // Edit content
//! editor.insert_text("Hello, AIT42!")?;
//! # Ok(())
//! # }
//! ```

pub mod buffer;
pub mod cursor;
pub mod editor;
pub mod history;
pub mod selection;

// Re-exports
pub use buffer::{Buffer, BufferId, BufferManager};
pub use cursor::{Cursor, CursorPosition};
pub use editor::{Editor, EditorConfig};
pub use history::{Change, History};
pub use selection::{Selection, SelectionRange};

use thiserror::Error;

/// Core editor errors
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Buffer not found: {0}")]
    BufferNotFound(BufferId),

    #[error("Invalid cursor position: ({0}, {1})")]
    InvalidCursorPosition(usize, usize),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Ensure all modules compile
        assert!(true);
    }
}
