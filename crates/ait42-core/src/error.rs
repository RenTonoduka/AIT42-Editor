//! Error Types
//!
//! Comprehensive error handling for the core editor.

use std::ops::Range;
use thiserror::Error;

/// Core editor errors
#[derive(Error, Debug)]
pub enum EditorError {
    #[error("Invalid position: {0}")]
    InvalidPosition(usize),

    #[error("Invalid position: line {line}, col {col}")]
    InvalidLineCol { line: usize, col: usize },

    #[error("UTF-8 boundary error at position {0}")]
    Utf8Boundary(usize),

    #[error("Buffer is empty")]
    EmptyBuffer,

    #[error("Invalid range: {0:?}")]
    InvalidRange(Range<usize>),

    #[error("Buffer not found: {0}")]
    BufferNotFound(uuid::Uuid),

    #[error("No active buffer")]
    NoActiveBuffer,

    #[error("Cannot undo: {0}")]
    CannotUndo(String),

    #[error("Cannot redo: {0}")]
    CannotRedo(String),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Line {0} is out of bounds")]
    LineOutOfBounds(usize),

    #[error("Invalid grapheme cluster at position {0}")]
    InvalidGrapheme(usize),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, EditorError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EditorError::InvalidPosition(42);
        assert_eq!(err.to_string(), "Invalid position: 42");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: EditorError = io_err.into();
        assert!(matches!(err, EditorError::Io(_)));
    }
}
