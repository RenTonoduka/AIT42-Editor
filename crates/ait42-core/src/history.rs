//! Undo/Redo History

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
}
