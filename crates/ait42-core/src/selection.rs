//! Selection Management
//!
//! Handles text selection ranges.

use std::ops::Range;

use crate::cursor::CursorPosition;

/// Selection range in buffer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionRange {
    /// Start position (line, column)
    pub start: CursorPosition,
    /// End position (line, column)
    pub end: CursorPosition,
}

impl SelectionRange {
    /// Create new selection range
    pub fn new(start: CursorPosition, end: CursorPosition) -> Self {
        Self { start, end }
    }

    /// Check if selection is normalized (start before end)
    pub fn is_normalized(&self) -> bool {
        self.start.line < self.end.line
            || (self.start.line == self.end.line && self.start.col <= self.end.col)
    }

    /// Get normalized range (start always before end)
    pub fn normalized(&self) -> Self {
        if self.is_normalized() {
            self.clone()
        } else {
            Self {
                start: self.end,
                end: self.start,
            }
        }
    }

    /// Convert to byte range (requires buffer for conversion)
    pub fn to_byte_range(
        &self,
        line_col_to_pos: impl Fn(usize, usize) -> Option<usize>,
    ) -> Option<Range<usize>> {
        let start_pos = line_col_to_pos(self.start.line, self.start.col)?;
        let end_pos = line_col_to_pos(self.end.line, self.end.col)?;
        Some(start_pos..end_pos)
    }
}

/// Text selection state
///
/// Supports multiple selection ranges (multi-cursor editing).
#[derive(Debug, Clone, Default)]
pub struct Selection {
    ranges: Vec<SelectionRange>,
}

impl Selection {
    /// Create new empty selection
    pub fn new() -> Self {
        Self::default()
    }

    /// Add selection range
    pub fn add_range(&mut self, range: SelectionRange) {
        self.ranges.push(range);
    }

    /// Get all selection ranges
    pub fn ranges(&self) -> &[SelectionRange] {
        &self.ranges
    }

    /// Get mutable selection ranges
    pub fn ranges_mut(&mut self) -> &mut Vec<SelectionRange> {
        &mut self.ranges
    }

    /// Clear all selections
    pub fn clear(&mut self) {
        self.ranges.clear();
    }

    /// Check if any selection is active
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// Get count of selection ranges
    #[inline]
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    /// Get primary selection (first range)
    pub fn primary(&self) -> Option<&SelectionRange> {
        self.ranges.first()
    }

    /// Get mutable primary selection
    pub fn primary_mut(&mut self) -> Option<&mut SelectionRange> {
        self.ranges.first_mut()
    }

    /// Merge overlapping ranges
    pub fn merge_overlapping(&mut self) {
        if self.ranges.len() <= 1 {
            return;
        }

        // Sort by start position
        self.ranges.sort_by(|a, b| {
            a.start
                .line
                .cmp(&b.start.line)
                .then(a.start.col.cmp(&b.start.col))
        });

        // Merge overlapping
        let mut merged = Vec::new();
        let mut current = self.ranges[0].clone();

        for range in &self.ranges[1..] {
            // Check if ranges overlap
            let overlap = current.end.line > range.start.line
                || (current.end.line == range.start.line && current.end.col >= range.start.col);

            if overlap {
                // Extend current range
                if range.end.line > current.end.line
                    || (range.end.line == current.end.line && range.end.col > current.end.col)
                {
                    current.end = range.end;
                }
            } else {
                // No overlap, save current and start new
                merged.push(current);
                current = range.clone();
            }
        }

        merged.push(current);
        self.ranges = merged;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_range_creation() {
        let start = CursorPosition::new(0, 0);
        let end = CursorPosition::new(0, 5);
        let range = SelectionRange::new(start, end);

        assert!(range.is_normalized());
    }

    #[test]
    fn test_selection_range_normalized() {
        let start = CursorPosition::new(0, 5);
        let end = CursorPosition::new(0, 0);
        let range = SelectionRange::new(start, end);

        assert!(!range.is_normalized());

        let norm = range.normalized();
        assert!(norm.is_normalized());
        assert_eq!(norm.start.col, 0);
        assert_eq!(norm.end.col, 5);
    }

    #[test]
    fn test_selection_add_range() {
        let mut sel = Selection::new();
        assert!(sel.is_empty());

        let range = SelectionRange::new(CursorPosition::new(0, 0), CursorPosition::new(0, 5));
        sel.add_range(range);

        assert_eq!(sel.len(), 1);
        assert!(!sel.is_empty());
    }

    #[test]
    fn test_selection_clear() {
        let mut sel = Selection::new();
        sel.add_range(SelectionRange::new(CursorPosition::new(0, 0), CursorPosition::new(0, 5)));

        sel.clear();
        assert!(sel.is_empty());
    }

    #[test]
    fn test_selection_merge_overlapping() {
        let mut sel = Selection::new();

        // Add overlapping ranges
        sel.add_range(SelectionRange::new(CursorPosition::new(0, 0), CursorPosition::new(0, 5)));
        sel.add_range(SelectionRange::new(CursorPosition::new(0, 3), CursorPosition::new(0, 8)));

        sel.merge_overlapping();

        assert_eq!(sel.len(), 1);
        let range = sel.primary().unwrap();
        assert_eq!(range.start.col, 0);
        assert_eq!(range.end.col, 8);
    }

    #[test]
    fn test_selection_merge_non_overlapping() {
        let mut sel = Selection::new();

        // Add non-overlapping ranges
        sel.add_range(SelectionRange::new(CursorPosition::new(0, 0), CursorPosition::new(0, 5)));
        sel.add_range(SelectionRange::new(CursorPosition::new(0, 10), CursorPosition::new(0, 15)));

        sel.merge_overlapping();

        assert_eq!(sel.len(), 2);
    }
}
