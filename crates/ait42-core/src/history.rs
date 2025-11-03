//! Undo/Redo History
//!
//! Legacy module - kept for backward compatibility.
//! Use `command::CommandHistory` for new code.

/// Represents a change in the buffer
#[derive(Debug, Clone)]
pub enum Change {
    Insert {
        line: usize,
        col: usize,
        text: String,
    },
    Delete {
        line: usize,
        col: usize,
        text: String,
    },
}

/// History manager for undo/redo
///
/// **Deprecated**: Use `crate::command::CommandHistory` instead.
#[derive(Debug, Default)]
pub struct History {
    changes: Vec<Change>,
    current: usize,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, change: Change) {
        self.changes.truncate(self.current);
        self.changes.push(change);
        self.current += 1;
    }

    pub fn undo(&mut self) -> Option<&Change> {
        if self.current > 0 {
            self.current -= 1;
            self.changes.get(self.current)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<&Change> {
        if self.current < self.changes.len() {
            let change = self.changes.get(self.current);
            self.current += 1;
            change
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        self.current > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current < self.changes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_push() {
        let mut history = History::new();
        history.push(Change::Insert {
            line: 0,
            col: 0,
            text: "Hello".to_string(),
        });

        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_undo_redo() {
        let mut history = History::new();
        history.push(Change::Insert {
            line: 0,
            col: 0,
            text: "Hello".to_string(),
        });

        assert!(history.undo().is_some());
        assert!(history.can_redo());

        assert!(history.redo().is_some());
        assert!(history.can_undo());
    }
}
