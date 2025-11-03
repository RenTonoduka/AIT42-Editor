//! AIT42 Core - Editor Logic
//!
//! This crate contains the core editor functionality including:
//! - Buffer management (text storage and manipulation)
//! - Cursor and selection handling
//! - Edit operations and undo/redo
//! - Editor state management
//! - Mode system (Vim-style modal editing)
//!
//! # Architecture
//!
//! ```text
//! EditorState
//!   ├── BufferManager (manages multiple buffers)
//!   │   └── Buffer (text content with Rope data structure)
//!   ├── CursorSet (cursor positions per buffer)
//!   ├── Selection (selection ranges per buffer)
//!   ├── CommandHistory (undo/redo stack per buffer)
//!   ├── ModeManager (current editing mode)
//!   └── ViewState (viewport information per buffer)
//! ```
//!
//! # Example
//!
//! ```no_run
//! use ait42_core::{EditorState, Buffer};
//!
//! # fn example() -> ait42_core::error::Result<()> {
//! let mut state = EditorState::new();
//!
//! // Open a file
//! let buffer = Buffer::from_file(std::path::Path::new("src/main.rs"))?;
//! let id = state.open_buffer(buffer);
//!
//! // Edit content (using command pattern)
//! // Commands are automatically added to undo history
//! # Ok(())
//! # }
//! ```

// Public modules
pub mod buffer;
pub mod command;
pub mod cursor;
pub mod error;
pub mod mode;
pub mod selection;
pub mod state;
pub mod view;

// Private modules (implementation details)
mod history;
mod editor;

// Re-exports for convenience
pub use buffer::{Buffer, BufferId, BufferManager, LineEnding};
pub use command::{Command, CommandHistory, DeleteCommand, InsertCommand, ReplaceCommand};
pub use cursor::{Cursor, CursorPosition, CursorSet};
pub use error::{EditorError, Result};
pub use mode::{Mode, ModeManager};
pub use selection::{Selection, SelectionRange};
pub use state::EditorState;
pub use view::ViewState;

// Re-export for backward compatibility
pub use editor::{Editor, EditorConfig};
pub use history::{Change, History};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Ensure all modules compile and are accessible
        let _state = EditorState::new();
        let _buffer = Buffer::new();
        let _cursor = Cursor::new(0);
        let _mode = ModeManager::new();
        let _selection = Selection::new();
        let _view = ViewState::default();
        let _history = CommandHistory::new();

        assert!(true);
    }

    #[test]
    fn test_basic_workflow() {
        let mut state = EditorState::new();

        // Create and open buffer
        let buffer = Buffer::from_string("Hello, World!".to_string(), None);
        let _id = state.open_buffer(buffer);

        // Verify state
        assert_eq!(state.buffer_count(), 1);
        assert!(state.active_buffer().is_some());
        assert!(state.cursor().is_some());
        assert!(state.mode.is_normal());
    }

    #[test]
    fn test_mode_transitions() {
        let mut state = EditorState::new();

        // Start in Normal mode
        assert!(state.mode.is_normal());

        // Enter Insert mode
        state.mode.enter_insert();
        assert!(state.mode.is_insert());

        // Return to Normal mode
        state.mode.exit_insert();
        assert!(state.mode.is_normal());
    }

    #[test]
    fn test_buffer_operations() {
        let mut buffer = Buffer::new();

        // Insert text
        buffer.insert(0, "Hello").unwrap();
        assert_eq!(buffer.to_string(), "Hello");
        assert!(buffer.is_dirty());

        // Delete text
        buffer.delete(0..5).unwrap();
        assert_eq!(buffer.to_string(), "");

        // Replace text
        buffer.insert(0, "Hello World").unwrap();
        buffer.replace(0..5, "Hi").unwrap();
        assert_eq!(buffer.to_string(), "Hi World");
    }

    #[test]
    fn test_cursor_movement() {
        let buffer = Buffer::from_string("Line 1\nLine 2\nLine 3".to_string(), None);
        let mut cursor = Cursor::new(0);

        // Move down
        cursor.move_down(&buffer, 1);
        let (line, _) = buffer.pos_to_line_col(cursor.pos());
        assert_eq!(line, 1);

        // Move to line end
        cursor.move_to_line_end(&buffer);
        let (_, col) = buffer.pos_to_line_col(cursor.pos());
        assert!(col > 0);

        // Move to line start
        cursor.move_to_line_start(&buffer);
        let (_, col) = buffer.pos_to_line_col(cursor.pos());
        assert_eq!(col, 0);
    }

    #[test]
    fn test_selection() {
        let mut cursor = Cursor::new(0);

        // Start selection
        cursor.start_selection();
        cursor.set_pos(10);

        assert!(cursor.has_selection());
        assert_eq!(cursor.selection(), Some(0..10));

        // Clear selection
        cursor.clear_selection();
        assert!(!cursor.has_selection());
    }

    #[test]
    fn test_command_execution() {
        let mut buffer = Buffer::new();
        let mut cmd = InsertCommand::new(buffer.id(), 0, "Hello");

        // Execute command
        cmd.execute(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "Hello");

        // Undo command
        cmd.undo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "");
    }

    #[test]
    fn test_undo_redo() {
        let mut state = EditorState::new();
        let buffer = Buffer::new();
        state.open_buffer(buffer);

        // Execute command
        let cmd = Box::new(InsertCommand::new(
            state.active_buffer().unwrap().id(),
            0,
            "Hello",
        ));

        state.execute_command(cmd).unwrap();
        assert_eq!(state.active_buffer().unwrap().to_string(), "Hello");

        // Undo
        state.undo().unwrap();
        assert_eq!(state.active_buffer().unwrap().to_string(), "");

        // Redo
        state.redo().unwrap();
        assert_eq!(state.active_buffer().unwrap().to_string(), "Hello");
    }
}
