//! Editor State
//!
//! Central state management for the editor.

use std::collections::HashMap;

use crate::buffer::{Buffer, BufferId, BufferManager};
use crate::command::{Command, CommandHistory};
use crate::cursor::{Cursor, CursorSet};
use crate::error::Result;
use crate::mode::{Mode, ModeManager};
use crate::selection::{Selection, SelectionRange};
use crate::view::ViewState;

/// Global editor state
///
/// Contains all state for the editor including buffers, cursors, modes, etc.
#[derive(Debug)]
pub struct EditorState {
    /// Buffer manager for all open files
    pub buffer_manager: BufferManager,

    /// Current editing mode
    pub mode: ModeManager,

    /// Cursor positions per buffer
    cursors: HashMap<BufferId, CursorSet>,

    /// Selections per buffer
    selections: HashMap<BufferId, Selection>,

    /// Undo/redo history per buffer
    histories: HashMap<BufferId, CommandHistory>,

    /// View state per buffer
    views: HashMap<BufferId, ViewState>,
}

impl EditorState {
    /// Create new editor state
    pub fn new() -> Self {
        Self {
            buffer_manager: BufferManager::new(),
            mode: ModeManager::new(),
            cursors: HashMap::new(),
            selections: HashMap::new(),
            histories: HashMap::new(),
            views: HashMap::new(),
        }
    }

    /// Get active buffer
    pub fn active_buffer(&self) -> Option<&Buffer> {
        self.buffer_manager.active()
    }

    /// Get mutable active buffer
    pub fn active_buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.buffer_manager.active_mut()
    }

    /// Get cursor for active buffer
    pub fn cursor(&self) -> Option<&Cursor> {
        let id = self.buffer_manager.active_buffer_id()?;
        self.cursors.get(&id).map(|set| set.primary())
    }

    /// Get mutable cursor for active buffer
    pub fn cursor_mut(&mut self) -> Option<&mut Cursor> {
        let id = self.buffer_manager.active_buffer_id()?;
        self.cursors.get_mut(&id).map(|set| set.primary_mut())
    }

    /// Get cursor set for active buffer
    pub fn cursor_set(&self) -> Option<&CursorSet> {
        let id = self.buffer_manager.active_buffer_id()?;
        self.cursors.get(&id)
    }

    /// Get mutable cursor set for active buffer
    pub fn cursor_set_mut(&mut self) -> Option<&mut CursorSet> {
        let id = self.buffer_manager.active_buffer_id()?;
        self.cursors.get_mut(&id)
    }

    /// Get selection for active buffer
    pub fn selection(&self) -> Option<&Selection> {
        let id = self.buffer_manager.active_buffer_id()?;
        self.selections.get(&id)
    }

    /// Get mutable selection for active buffer
    pub fn selection_mut(&mut self) -> Option<&mut Selection> {
        let id = self.buffer_manager.active_buffer_id()?;
        self.selections.get_mut(&id)
    }

    /// Get view state for active buffer
    pub fn view(&self) -> Option<&ViewState> {
        let id = self.buffer_manager.active_buffer_id()?;
        self.views.get(&id)
    }

    /// Get mutable view state for active buffer
    pub fn view_mut(&mut self) -> Option<&mut ViewState> {
        let id = self.buffer_manager.active_buffer_id()?;
        self.views.get_mut(&id)
    }

    /// Get command history for active buffer
    fn history_mut(&mut self) -> Option<&mut CommandHistory> {
        let id = self.buffer_manager.active_buffer_id()?;
        self.histories.get_mut(&id)
    }

    /// Open new buffer
    ///
    /// Initializes cursor, selection, history, and view for the buffer.
    pub fn open_buffer(&mut self, buffer: Buffer) -> BufferId {
        let id = buffer.id();

        // Add buffer to buffer manager
        self.buffer_manager.add_buffer(buffer);

        // Initialize state for new buffer
        self.cursors.insert(id, CursorSet::new(0));
        self.selections.insert(id, Selection::new());
        self.histories.insert(id, CommandHistory::new());
        self.views.insert(id, ViewState::default());

        id
    }

    /// Close buffer and clean up associated state
    pub fn close_buffer(&mut self, id: BufferId, force: bool) -> Result<()> {
        self.buffer_manager.close(id, force)?;

        // Clean up associated state
        self.cursors.remove(&id);
        self.selections.remove(&id);
        self.histories.remove(&id);
        self.views.remove(&id);

        Ok(())
    }

    /// Execute command on active buffer
    ///
    /// Commands are automatically added to undo history.
    pub fn execute_command(&mut self, mut cmd: Box<dyn Command>) -> Result<()> {
        let buffer = self.active_buffer_mut()
            .ok_or_else(|| crate::error::EditorError::NoActiveBuffer)?;

        // Execute command
        cmd.execute(buffer)?;

        // Add to history if undoable
        if cmd.can_undo() {
            if let Some(history) = self.history_mut() {
                history.push(cmd);
            }
        }

        // Update view to ensure cursor is visible
        let cursor_line = self.cursor().and_then(|cursor| {
            self.active_buffer().map(|buffer| {
                let (line, _) = buffer.pos_to_line_col(cursor.pos());
                line
            })
        });

        if let Some(line) = cursor_line {
            if let Some(view) = self.view_mut() {
                view.ensure_cursor_visible(line);
            }
        }

        Ok(())
    }

    /// Undo last command on active buffer
    ///
    /// Returns true if undo was successful, false if nothing to undo.
    pub fn undo(&mut self) -> Result<bool> {
        let id = self.buffer_manager.active_buffer_id()
            .ok_or_else(|| crate::error::EditorError::NoActiveBuffer)?;

        let buffer = self.buffer_manager.active_mut()
            .ok_or_else(|| crate::error::EditorError::NoActiveBuffer)?;

        if let Some(history) = self.histories.get_mut(&id) {
            history.undo(buffer)
        } else {
            Ok(false)
        }
    }

    /// Redo last undone command on active buffer
    ///
    /// Returns true if redo was successful, false if nothing to redo.
    pub fn redo(&mut self) -> Result<bool> {
        let id = self.buffer_manager.active_buffer_id()
            .ok_or_else(|| crate::error::EditorError::NoActiveBuffer)?;

        let buffer = self.buffer_manager.active_mut()
            .ok_or_else(|| crate::error::EditorError::NoActiveBuffer)?;

        if let Some(history) = self.histories.get_mut(&id) {
            history.redo(buffer)
        } else {
            Ok(false)
        }
    }

    /// Check if undo is available for active buffer
    pub fn can_undo(&self) -> bool {
        if let Some(id) = self.buffer_manager.active_buffer_id() {
            self.histories.get(&id).map_or(false, |h| h.can_undo())
        } else {
            false
        }
    }

    /// Check if redo is available for active buffer
    pub fn can_redo(&self) -> bool {
        if let Some(id) = self.buffer_manager.active_buffer_id() {
            self.histories.get(&id).map_or(false, |h| h.can_redo())
        } else {
            false
        }
    }

    /// Switch to different buffer
    pub fn switch_buffer(&mut self, id: BufferId) -> Result<()> {
        self.buffer_manager.switch_to(id)
    }

    /// Get count of open buffers
    #[inline]
    pub fn buffer_count(&self) -> usize {
        self.buffer_manager.len()
    }

    /// Get all buffer IDs
    pub fn buffer_ids(&self) -> Vec<BufferId> {
        self.buffer_manager.buffer_ids()
    }

    /// Get buffers with unsaved changes
    pub fn dirty_buffers(&self) -> Vec<BufferId> {
        self.buffer_manager.dirty_buffers()
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_state_creation() {
        let state = EditorState::new();
        assert_eq!(state.buffer_count(), 0);
        assert!(state.mode.is_normal());
    }

    #[test]
    fn test_editor_state_open_buffer() {
        let mut state = EditorState::new();
        let buffer = Buffer::new();
        let id = state.open_buffer(buffer);

        assert_eq!(state.buffer_count(), 1);
        assert!(state.cursor_set().is_some());
        assert!(state.selection().is_some());
        assert!(state.view().is_some());
    }

    #[test]
    fn test_editor_state_mode_switch() {
        let mut state = EditorState::new();

        state.mode.enter_insert();
        assert!(state.mode.is_insert());

        state.mode.exit_insert();
        assert!(state.mode.is_normal());
    }

    #[test]
    fn test_editor_state_dirty_buffers() {
        let mut state = EditorState::new();
        let mut buffer = Buffer::new();
        buffer.insert(0, "test").unwrap();

        let id = buffer.id();
        state.open_buffer(buffer);

        let dirty = state.dirty_buffers();
        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0], id);
    }
}
