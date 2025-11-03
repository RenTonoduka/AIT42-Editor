//! Selection Management

use crate::cursor::CursorPosition;

/// Selection range in buffer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionRange {
    pub start: CursorPosition,
    pub end: CursorPosition,
}

/// Text selection state
#[derive(Debug, Clone, Default)]
pub struct Selection {
    ranges: Vec<SelectionRange>,
}

impl Selection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_range(&mut self, range: SelectionRange) {
        self.ranges.push(range);
    }

    pub fn clear(&mut self) {
        self.ranges.clear();
    }
}
