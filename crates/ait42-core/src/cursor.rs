//! Cursor Management

/// Cursor position in buffer (line, column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPosition {
    pub line: usize,
    pub col: usize,
}

impl CursorPosition {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

/// Editor cursor with position and selection
#[derive(Debug, Clone)]
pub struct Cursor {
    position: CursorPosition,
    anchor: Option<CursorPosition>,
}

impl Cursor {
    pub fn new(line: usize, col: usize) -> Self {
        Self {
            position: CursorPosition::new(line, col),
            anchor: None,
        }
    }

    pub fn position(&self) -> CursorPosition {
        self.position
    }

    pub fn move_to(&mut self, line: usize, col: usize) {
        self.position = CursorPosition::new(line, col);
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new(0, 0)
    }
}
