//! Cursor Management
//!
//! Handles cursor positioning and movement with grapheme cluster awareness.

use unicode_segmentation::UnicodeSegmentation;

use crate::buffer::Buffer;
use crate::error::{EditorError, Result};

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

/// Editor cursor with position and optional selection anchor
///
/// The cursor is always at a byte offset. When a selection is active,
/// the `anchor` defines the selection start and `position` defines the end.
#[derive(Debug, Clone)]
pub struct Cursor {
    /// Current cursor position (byte offset)
    position: usize,
    /// Selection anchor (byte offset) - None means no selection
    anchor: Option<usize>,
    /// Preferred column for vertical movement (preserves column across lines)
    preferred_col: Option<usize>,
}

impl Cursor {
    /// Create cursor at position (byte offset)
    pub fn new(pos: usize) -> Self {
        Self {
            position: pos,
            anchor: None,
            preferred_col: None,
        }
    }

    /// Get cursor position (byte offset)
    #[inline]
    pub fn pos(&self) -> usize {
        self.position
    }

    /// Set cursor position
    pub fn set_pos(&mut self, pos: usize) {
        self.position = pos;
        self.preferred_col = None; // Reset preferred column on explicit move
    }

    /// Get cursor position as (line, col)
    pub fn position(&self, buffer: &Buffer) -> CursorPosition {
        let (line, col) = buffer.pos_to_line_col(self.position);
        CursorPosition::new(line, col)
    }

    /// Move to specific line and column
    pub fn move_to(&mut self, buffer: &Buffer, line: usize, col: usize) -> Result<()> {
        if let Some(pos) = buffer.line_col_to_pos(line, col) {
            self.position = pos;
            self.preferred_col = Some(col);
            Ok(())
        } else {
            Err(EditorError::InvalidLineCol { line, col })
        }
    }

    /// Move cursor left by `count` grapheme clusters
    pub fn move_left(&mut self, buffer: &Buffer, count: usize) {
        let text = buffer.to_string();
        let graphemes: Vec<&str> = text.graphemes(true).collect();

        // Find current grapheme index
        let mut byte_pos = 0;
        let mut grapheme_idx = 0;

        for (idx, g) in graphemes.iter().enumerate() {
            if byte_pos >= self.position {
                grapheme_idx = idx;
                break;
            }
            byte_pos += g.len();
        }

        // Move left by count
        let new_idx = grapheme_idx.saturating_sub(count);

        // Calculate new byte position
        self.position = graphemes.iter().take(new_idx).map(|g| g.len()).sum();
        self.preferred_col = None;
    }

    /// Move cursor right by `count` grapheme clusters
    pub fn move_right(&mut self, buffer: &Buffer, count: usize) {
        let text = buffer.to_string();
        let graphemes: Vec<&str> = text.graphemes(true).collect();

        // Find current grapheme index
        let mut byte_pos = 0;
        let mut grapheme_idx = 0;

        for (idx, g) in graphemes.iter().enumerate() {
            if byte_pos >= self.position {
                grapheme_idx = idx;
                break;
            }
            byte_pos += g.len();
        }

        // Move right by count
        let new_idx = (grapheme_idx + count).min(graphemes.len());

        // Calculate new byte position
        self.position = graphemes.iter().take(new_idx).map(|g| g.len()).sum();
        self.preferred_col = None;
    }

    /// Move cursor up by `count` lines
    ///
    /// Preserves preferred column across lines of different lengths.
    pub fn move_up(&mut self, buffer: &Buffer, count: usize) {
        let (mut line, col) = buffer.pos_to_line_col(self.position);

        // Set preferred column if not set
        if self.preferred_col.is_none() {
            self.preferred_col = Some(col);
        }

        // Move up
        line = line.saturating_sub(count);

        // Use preferred column or current column
        let target_col = self.preferred_col.unwrap_or(col);

        // Clamp to line length
        if let Some(line_text) = buffer.line(line) {
            let line_len = line_text.graphemes(true).count();
            let actual_col = target_col.min(line_len);

            if let Some(pos) = buffer.line_col_to_pos(line, actual_col) {
                self.position = pos;
            }
        }
    }

    /// Move cursor down by `count` lines
    pub fn move_down(&mut self, buffer: &Buffer, count: usize) {
        let (mut line, col) = buffer.pos_to_line_col(self.position);

        // Set preferred column if not set
        if self.preferred_col.is_none() {
            self.preferred_col = Some(col);
        }

        // Move down
        let max_line = buffer.len_lines().saturating_sub(1);
        line = (line + count).min(max_line);

        // Use preferred column or current column
        let target_col = self.preferred_col.unwrap_or(col);

        // Clamp to line length
        if let Some(line_text) = buffer.line(line) {
            let line_len = line_text.graphemes(true).count();
            let actual_col = target_col.min(line_len);

            if let Some(pos) = buffer.line_col_to_pos(line, actual_col) {
                self.position = pos;
            }
        }
    }

    /// Move to start of current line
    pub fn move_to_line_start(&mut self, buffer: &Buffer) {
        let (line, _) = buffer.pos_to_line_col(self.position);
        if let Some(pos) = buffer.line_col_to_pos(line, 0) {
            self.position = pos;
            self.preferred_col = None;
        }
    }

    /// Move to end of current line
    pub fn move_to_line_end(&mut self, buffer: &Buffer) {
        let (line, _) = buffer.pos_to_line_col(self.position);
        if let Some(line_text) = buffer.line(line) {
            let line_len = line_text.graphemes(true).count();
            if let Some(pos) = buffer.line_col_to_pos(line, line_len) {
                self.position = pos;
                self.preferred_col = None;
            }
        }
    }

    /// Move to start of buffer
    pub fn move_to_buffer_start(&mut self) {
        self.position = 0;
        self.preferred_col = None;
    }

    /// Move to end of buffer
    pub fn move_to_buffer_end(&mut self, buffer: &Buffer) {
        self.position = buffer.len_bytes();
        self.preferred_col = None;
    }

    /// Move forward by one word
    pub fn move_word_forward(&mut self, buffer: &Buffer) {
        let text = buffer.to_string();
        let words: Vec<(usize, &str)> = text
            .split_word_bound_indices()
            .filter(|(_, word)| !word.trim().is_empty())
            .collect();

        // Find next word boundary
        for (idx, _) in words {
            if idx > self.position {
                self.position = idx;
                self.preferred_col = None;
                return;
            }
        }

        // No next word, move to end
        self.position = buffer.len_bytes();
        self.preferred_col = None;
    }

    /// Move backward by one word
    pub fn move_word_backward(&mut self, buffer: &Buffer) {
        let text = buffer.to_string();
        let words: Vec<(usize, &str)> = text
            .split_word_bound_indices()
            .filter(|(_, word)| !word.trim().is_empty())
            .collect();

        // Find previous word boundary
        for (idx, _) in words.iter().rev() {
            if *idx < self.position {
                self.position = *idx;
                self.preferred_col = None;
                return;
            }
        }

        // No previous word, move to start
        self.position = 0;
        self.preferred_col = None;
    }

    /// Start selection at current position
    pub fn start_selection(&mut self) {
        self.anchor = Some(self.position);
    }

    /// Extend selection to current position
    pub fn extend_selection(&mut self) {
        // If no anchor, start selection
        if self.anchor.is_none() {
            self.anchor = Some(self.position);
        }
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    /// Get selection range (if any)
    ///
    /// Returns None if no selection is active.
    /// Range is always normalized (start < end).
    pub fn selection(&self) -> Option<std::ops::Range<usize>> {
        self.anchor.map(|anchor| {
            if anchor < self.position {
                anchor..self.position
            } else {
                self.position..anchor
            }
        })
    }

    /// Check if selection is active
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.anchor.is_some()
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Multi-cursor support (Phase 2)
///
/// Supports Sublime Text / VS Code style multi-cursor editing.
#[derive(Debug, Clone)]
pub struct CursorSet {
    primary: Cursor,
    secondary: Vec<Cursor>,
}

impl CursorSet {
    /// Create cursor set with single cursor at position
    pub fn new(pos: usize) -> Self {
        Self {
            primary: Cursor::new(pos),
            secondary: Vec::new(),
        }
    }

    /// Get primary cursor
    #[inline]
    pub fn primary(&self) -> &Cursor {
        &self.primary
    }

    /// Get mutable primary cursor
    #[inline]
    pub fn primary_mut(&mut self) -> &mut Cursor {
        &mut self.primary
    }

    /// Add cursor at position
    ///
    /// Cursors are automatically merged if they overlap.
    pub fn add_cursor(&mut self, pos: usize) {
        let cursor = Cursor::new(pos);
        self.secondary.push(cursor);
        self.merge_cursors();
    }

    /// Remove cursor by index (secondary cursors only)
    pub fn remove_cursor(&mut self, index: usize) -> Result<()> {
        if index >= self.secondary.len() {
            return Err(EditorError::Other(format!(
                "Cursor index {} out of bounds",
                index
            )));
        }
        self.secondary.remove(index);
        Ok(())
    }

    /// Get all cursors (primary + secondary)
    pub fn cursors(&self) -> impl Iterator<Item = &Cursor> {
        std::iter::once(&self.primary).chain(self.secondary.iter())
    }

    /// Get mutable cursor iterator
    pub fn cursors_mut(&mut self) -> impl Iterator<Item = &mut Cursor> {
        std::iter::once(&mut self.primary).chain(self.secondary.iter_mut())
    }

    /// Apply operation to all cursors
    pub fn apply<F>(&mut self, buffer: &Buffer, mut f: F)
    where
        F: FnMut(&mut Cursor, &Buffer),
    {
        f(&mut self.primary, buffer);
        for cursor in &mut self.secondary {
            f(cursor, buffer);
        }
    }

    /// Merge overlapping cursors
    pub fn merge_cursors(&mut self) {
        // Remove duplicates by position
        self.secondary.sort_by_key(|c| c.pos());
        self.secondary.dedup_by_key(|c| c.pos());

        // Remove any that overlap with primary
        let primary_pos = self.primary.pos();
        self.secondary.retain(|c| c.pos() != primary_pos);
    }

    /// Clear all secondary cursors
    pub fn clear_secondary(&mut self) {
        self.secondary.clear();
    }

    /// Get cursor count
    #[inline]
    pub fn len(&self) -> usize {
        1 + self.secondary.len()
    }

    /// Check if only primary cursor exists
    #[inline]
    pub fn is_single(&self) -> bool {
        self.secondary.is_empty()
    }
}

impl Default for CursorSet {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_creation() {
        let cursor = Cursor::new(0);
        assert_eq!(cursor.pos(), 0);
        assert!(!cursor.has_selection());
    }

    #[test]
    fn test_cursor_movement_left_right() {
        let buffer = Buffer::from_string("Hello World".to_string(), None);
        let mut cursor = Cursor::new(5);

        cursor.move_left(&buffer, 1);
        assert_eq!(cursor.pos(), 4);

        cursor.move_right(&buffer, 2);
        assert_eq!(cursor.pos(), 6);
    }

    #[test]
    fn test_cursor_movement_up_down() {
        let buffer = Buffer::from_string("Line 1\nLine 2\nLine 3".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_down(&buffer, 1);
        let (line, _) = buffer.pos_to_line_col(cursor.pos());
        assert_eq!(line, 1);

        cursor.move_up(&buffer, 1);
        let (line, _) = buffer.pos_to_line_col(cursor.pos());
        assert_eq!(line, 0);
    }

    #[test]
    fn test_cursor_line_start_end() {
        let buffer = Buffer::from_string("Hello World".to_string(), None);
        let mut cursor = Cursor::new(5);

        cursor.move_to_line_start(&buffer);
        assert_eq!(cursor.pos(), 0);

        cursor.move_to_line_end(&buffer);
        assert_eq!(cursor.pos(), 11);
    }

    #[test]
    fn test_cursor_selection() {
        let mut cursor = Cursor::new(0);

        cursor.start_selection();
        cursor.set_pos(5);

        assert!(cursor.has_selection());
        assert_eq!(cursor.selection(), Some(0..5));

        cursor.clear_selection();
        assert!(!cursor.has_selection());
    }

    #[test]
    fn test_cursor_set() {
        let mut cursors = CursorSet::new(0);
        assert_eq!(cursors.len(), 1);

        cursors.add_cursor(10);
        assert_eq!(cursors.len(), 2);

        cursors.clear_secondary();
        assert!(cursors.is_single());
    }

    #[test]
    fn test_cursor_set_merge() {
        let mut cursors = CursorSet::new(0);
        cursors.add_cursor(10);
        cursors.add_cursor(10); // Duplicate

        cursors.merge_cursors();
        assert_eq!(cursors.len(), 2); // Primary + 1 secondary
    }
}
