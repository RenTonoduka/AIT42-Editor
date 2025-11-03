//! Command System
//!
//! Implements the Command pattern for undoable/redoable operations.

use std::ops::Range;

use crate::buffer::{Buffer, BufferId};
use crate::error::Result;

/// A command that can be executed and undone
///
/// All editor operations that modify state should be commands.
/// This enables undo/redo functionality.
pub trait Command: Send + Sync + std::fmt::Debug {
    /// Execute the command
    ///
    /// Modifies the editor state according to the command's logic.
    fn execute(&mut self, buffer: &mut Buffer) -> Result<()>;

    /// Undo the command
    ///
    /// Should reverse the effects of `execute()`.
    fn undo(&mut self, buffer: &mut Buffer) -> Result<()>;

    /// Get command description
    ///
    /// Used for command history display.
    fn description(&self) -> &str;

    /// Check if command can be undone
    ///
    /// Some commands (like save) cannot be undone.
    fn can_undo(&self) -> bool {
        true
    }

    /// Try to merge with another command
    ///
    /// Returns Some if commands can be merged (e.g., consecutive inserts),
    /// None otherwise.
    fn merge_with(&mut self, _other: &dyn Command) -> bool {
        false
    }
}

/// Insert text at position
#[derive(Debug, Clone)]
pub struct InsertCommand {
    buffer_id: BufferId,
    pos: usize,
    text: String,
}

impl InsertCommand {
    pub fn new(buffer_id: BufferId, pos: usize, text: impl Into<String>) -> Self {
        Self {
            buffer_id,
            pos,
            text: text.into(),
        }
    }
}

impl Command for InsertCommand {
    fn execute(&mut self, buffer: &mut Buffer) -> Result<()> {
        buffer.insert(self.pos, &self.text)
    }

    fn undo(&mut self, buffer: &mut Buffer) -> Result<()> {
        let end = self.pos + self.text.len();
        buffer.delete(self.pos..end)
    }

    fn description(&self) -> &str {
        "Insert text"
    }

    fn merge_with(&mut self, _other: &dyn Command) -> bool {
        // TODO: Implement merge logic using a different approach
        // Downcasting requires 'static lifetime which trait methods can't enforce
        false
    }
}

/// Delete text range
#[derive(Debug, Clone)]
pub struct DeleteCommand {
    buffer_id: BufferId,
    range: Range<usize>,
    deleted_text: String, // Saved during execute for undo
}

impl DeleteCommand {
    pub fn new(buffer_id: BufferId, range: Range<usize>) -> Self {
        Self {
            buffer_id,
            range,
            deleted_text: String::new(),
        }
    }
}

impl Command for DeleteCommand {
    fn execute(&mut self, buffer: &mut Buffer) -> Result<()> {
        // Save text for undo
        self.deleted_text = buffer.slice(self.range.clone())?;
        buffer.delete(self.range.clone())
    }

    fn undo(&mut self, buffer: &mut Buffer) -> Result<()> {
        buffer.insert(self.range.start, &self.deleted_text)
    }

    fn description(&self) -> &str {
        "Delete text"
    }
}

/// Replace text in range
#[derive(Debug, Clone)]
pub struct ReplaceCommand {
    buffer_id: BufferId,
    range: Range<usize>,
    old_text: String, // Saved during execute for undo
    new_text: String,
}

impl ReplaceCommand {
    pub fn new(buffer_id: BufferId, range: Range<usize>, new_text: String) -> Self {
        Self {
            buffer_id,
            range,
            old_text: String::new(),
            new_text,
        }
    }
}

impl Command for ReplaceCommand {
    fn execute(&mut self, buffer: &mut Buffer) -> Result<()> {
        // Save old text for undo
        self.old_text = buffer.slice(self.range.clone())?;
        buffer.replace(self.range.clone(), &self.new_text)
    }

    fn undo(&mut self, buffer: &mut Buffer) -> Result<()> {
        let new_range = self.range.start..(self.range.start + self.new_text.len());
        buffer.replace(new_range, &self.old_text)
    }

    fn description(&self) -> &str {
        "Replace text"
    }
}

/// Command history for undo/redo
///
/// Maintains two stacks: undo and redo.
/// Executing a new command clears the redo stack.
#[derive(Debug)]
pub struct CommandHistory {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    max_history: usize,
}

impl CommandHistory {
    /// Create new command history with default max size (1000)
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Create command history with specified max size
    pub fn with_capacity(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    /// Push command to undo stack
    ///
    /// Clears redo stack as new commands invalidate redo history.
    pub fn push(&mut self, cmd: Box<dyn Command>) {
        // Try to merge with last command
        if let Some(last) = self.undo_stack.last_mut() {
            if last.merge_with(cmd.as_ref()) {
                return;
            }
        }

        self.undo_stack.push(cmd);
        self.redo_stack.clear();

        // Limit history size
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    /// Undo last command
    ///
    /// Returns the undone command if successful.
    pub fn undo(&mut self, buffer: &mut Buffer) -> Result<bool> {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(buffer)?;
            self.redo_stack.push(cmd);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Redo last undone command
    ///
    /// Returns the redone command if successful.
    pub fn redo(&mut self, buffer: &mut Buffer) -> Result<bool> {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(buffer)?;
            self.undo_stack.push(cmd);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if undo is available
    #[inline]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    #[inline]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get undo stack size
    #[inline]
    pub fn undo_len(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo stack size
    #[inline]
    pub fn redo_len(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_command() {
        let mut buffer = Buffer::new();
        let mut cmd = InsertCommand::new(buffer.id(), 0, "Hello");

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "");
    }

    #[test]
    fn test_delete_command() {
        let mut buffer = Buffer::from_string("Hello World".to_string(), None);
        let mut cmd = DeleteCommand::new(buffer.id(), 5..11);

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello World");
    }

    #[test]
    fn test_replace_command() {
        let mut buffer = Buffer::from_string("Hello World".to_string(), None);
        let mut cmd = ReplaceCommand::new(buffer.id(), 0..5, "Hi".to_string());

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hi World");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello World");
    }

    #[test]
    fn test_command_history() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        let cmd1 = Box::new(InsertCommand::new(buffer.id(), 0, "Hello"));
        history.push(cmd1);
        history
            .undo_stack
            .last_mut()
            .unwrap()
            .execute(&mut buffer)
            .unwrap();

        assert_eq!(buffer.to_string(), "Hello");
        assert!(history.can_undo());

        history.undo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "");
        assert!(history.can_redo());

        history.redo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "Hello");
    }

    #[test]
    fn test_command_history_clear_redo() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        let cmd1 = Box::new(InsertCommand::new(buffer.id(), 0, "Hello"));
        history.push(cmd1);
        history
            .undo_stack
            .last_mut()
            .unwrap()
            .execute(&mut buffer)
            .unwrap();

        history.undo(&mut buffer).unwrap();
        assert!(history.can_redo());

        // Pushing new command should clear redo
        let cmd2 = Box::new(InsertCommand::new(buffer.id(), 0, "World"));
        history.push(cmd2);

        assert!(!history.can_redo());
    }
}
