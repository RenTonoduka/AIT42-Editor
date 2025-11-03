//! Property-based tests for Cursor operations

use ait42_core::buffer::Buffer;
use ait42_core::cursor::{Cursor, CursorSet};
use proptest::prelude::*;

proptest! {
    /// Cursor movement should never exceed buffer bounds
    #[test]
    fn test_cursor_bounds(
        s in "\\PC{1,200}",
        moves in prop::collection::vec(0..4u8, 1..20)
    ) {
        let buffer = Buffer::from_string(s, None);
        let mut cursor = Cursor::new(0);

        for move_type in moves {
            match move_type {
                0 => cursor.move_left(&buffer, 1),
                1 => cursor.move_right(&buffer, 1),
                2 => cursor.move_up(&buffer, 1),
                _ => cursor.move_down(&buffer, 1),
            }

            // Cursor should never exceed buffer
            prop_assert!(cursor.pos() <= buffer.len_bytes());
        }
    }

    /// Move left then right returns to original position
    #[test]
    fn test_cursor_left_right_inverse(
        s in "\\PC{10,100}",
        count in 1..20usize
    ) {
        let buffer = Buffer::from_string(s, None);
        let start_pos = buffer.len_bytes() / 2;
        let mut cursor = Cursor::new(start_pos);

        let original_pos = cursor.pos();

        cursor.move_left(&buffer, count);
        cursor.move_right(&buffer, count);

        // Should return to same or nearby position
        let diff = (cursor.pos() as i64 - original_pos as i64).abs();
        prop_assert!(diff <= count as i64);
    }

    /// Move up then down returns to original line
    #[test]
    fn test_cursor_up_down_inverse(
        lines in prop::collection::vec("\\PC{5,20}", 5..20),
        count in 1..5usize
    ) {
        let text = lines.join("\n");
        let buffer = Buffer::from_string(text, None);

        let start_line = buffer.len_lines() / 2;
        let mut cursor = Cursor::new(0);
        let _ = cursor.move_to(&buffer, start_line, 0);

        let (original_line, _) = buffer.pos_to_line_col(cursor.pos());

        cursor.move_down(&buffer, count);
        cursor.move_up(&buffer, count);

        let (final_line, _) = buffer.pos_to_line_col(cursor.pos());

        // Should return to same or nearby line
        let diff = (final_line as i64 - original_line as i64).abs();
        prop_assert!(diff <= count as i64);
    }

    /// Selection range is always normalized (start < end)
    #[test]
    fn test_selection_normalized(
        pos1 in 0..1000usize,
        pos2 in 0..1000usize
    ) {
        let mut cursor = Cursor::new(pos1);

        cursor.start_selection();
        cursor.set_pos(pos2);

        if let Some(range) = cursor.selection() {
            prop_assert!(range.start <= range.end);
            prop_assert!(range.start == pos1.min(pos2));
            prop_assert!(range.end == pos1.max(pos2));
        }
    }

    /// CursorSet merge removes duplicates
    #[test]
    fn test_cursor_set_dedup(
        positions in prop::collection::vec(0..100usize, 1..20)
    ) {
        let mut cursor_set = CursorSet::new(0);

        for pos in &positions {
            cursor_set.add_cursor(*pos);
        }

        cursor_set.merge_cursors();

        // Should have at most unique positions + primary
        let unique_count = positions.iter().collect::<std::collections::HashSet<_>>().len();
        prop_assert!(cursor_set.len() <= unique_count + 1);
    }

    /// Move to buffer start always sets position to 0
    #[test]
    fn test_move_to_buffer_start(
        s in "\\PC{0,100}",
        initial_pos in 0..1000usize
    ) {
        let buffer = Buffer::from_string(s, None);
        let mut cursor = Cursor::new(initial_pos.min(buffer.len_bytes()));

        cursor.move_to_buffer_start();

        prop_assert_eq!(cursor.pos(), 0);
    }

    /// Move to buffer end always sets position to len_bytes
    #[test]
    fn test_move_to_buffer_end(
        s in "\\PC{0,100}",
        initial_pos in 0..50usize
    ) {
        let buffer = Buffer::from_string(s, None);
        let mut cursor = Cursor::new(initial_pos);

        cursor.move_to_buffer_end(&buffer);

        prop_assert_eq!(cursor.pos(), buffer.len_bytes());
    }

    /// Move to line start sets column to 0
    #[test]
    fn test_move_to_line_start(
        lines in prop::collection::vec("\\PC{5,20}", 1..10)
    ) {
        let text = lines.join("\n");
        let buffer = Buffer::from_string(text, None);

        for line_idx in 0..buffer.len_lines() {
            let mut cursor = Cursor::new(0);
            let _ = cursor.move_to(&buffer, line_idx, 5);

            cursor.move_to_line_start(&buffer);

            let (_, col) = buffer.pos_to_line_col(cursor.pos());
            prop_assert_eq!(col, 0);
        }
    }

    /// Move to line end sets column to line length
    #[test]
    fn test_move_to_line_end(
        lines in prop::collection::vec("\\PC{1,30}", 1..10)
    ) {
        let text = lines.join("\n");
        let buffer = Buffer::from_string(text, None);

        for line_idx in 0..buffer.len_lines() {
            let mut cursor = Cursor::new(0);
            let _ = cursor.move_to(&buffer, line_idx, 0);

            cursor.move_to_line_end(&buffer);

            let (line, col) = buffer.pos_to_line_col(cursor.pos());
            prop_assert_eq!(line, line_idx);

            if let Some(line_text) = buffer.line(line_idx) {
                // Column should be at or near end of line
                prop_assert!(col <= line_text.len() + 1);
            }
        }
    }

    /// Cursor position is consistent with line/col
    #[test]
    fn test_cursor_position_consistency(
        s in "\\PC{10,100}",
        pos in 0..100usize
    ) {
        let buffer = Buffer::from_string(s, None);
        let pos = pos.min(buffer.len_bytes());
        let cursor = Cursor::new(pos);

        let cursor_pos = cursor.position(&buffer);
        let (line, col) = buffer.pos_to_line_col(pos);

        prop_assert_eq!(cursor_pos.line, line);
        prop_assert_eq!(cursor_pos.col, col);
    }

    /// Multiple cursor operations maintain relative order
    #[test]
    fn test_cursor_set_order(
        positions in prop::collection::vec(0..100usize, 2..10)
    ) {
        let mut sorted_positions = positions.clone();
        sorted_positions.sort_unstable();
        sorted_positions.dedup();

        let mut cursor_set = CursorSet::new(sorted_positions[0]);

        for pos in &sorted_positions[1..] {
            cursor_set.add_cursor(*pos);
        }

        cursor_set.merge_cursors();

        // Cursors should maintain sorted order
        let cursor_positions: Vec<usize> = cursor_set.cursors()
            .map(|c| c.pos())
            .collect();

        let mut sorted = cursor_positions.clone();
        sorted.sort_unstable();

        prop_assert_eq!(cursor_positions, sorted);
    }

    /// Clear selection removes selection
    #[test]
    fn test_clear_selection_idempotent(
        pos1 in 0..100usize,
        pos2 in 50..150usize
    ) {
        let mut cursor = Cursor::new(pos1);

        cursor.start_selection();
        cursor.set_pos(pos2);

        prop_assert!(cursor.has_selection());

        cursor.clear_selection();
        prop_assert!(!cursor.has_selection());

        // Clearing again should be no-op
        cursor.clear_selection();
        prop_assert!(!cursor.has_selection());
    }

    /// Word movement stays within buffer bounds
    #[test]
    fn test_word_movement_bounds(
        words in prop::collection::vec("\\w{1,10}", 5..20)
    ) {
        let text = words.join(" ");
        let buffer = Buffer::from_string(text, None);
        let mut cursor = Cursor::new(0);

        // Move forward through all words
        for _ in 0..words.len() + 5 {
            cursor.move_word_forward(&buffer);
            prop_assert!(cursor.pos() <= buffer.len_bytes());
        }

        // Move backward through all words
        for _ in 0..words.len() + 5 {
            cursor.move_word_backward(&buffer);
            prop_assert!(cursor.pos() <= buffer.len_bytes());
        }
    }

    /// Extend selection multiple times grows selection
    #[test]
    fn test_extend_selection_monotonic(
        positions in prop::collection::vec(0..100usize, 2..10)
    ) {
        let mut cursor = Cursor::new(positions[0]);
        cursor.start_selection();

        let mut prev_size = 0;

        for pos in &positions[1..] {
            cursor.set_pos(*pos);
            cursor.extend_selection();

            if let Some(range) = cursor.selection() {
                let size = range.end - range.start;
                // Selection size should grow or stay same
                prop_assert!(size >= prev_size || size == 0);
                prev_size = size;
            }
        }
    }

    /// CursorSet apply operation affects all cursors
    #[test]
    fn test_cursor_set_apply_all(
        positions in prop::collection::vec(0..50usize, 3..10)
    ) {
        let buffer = Buffer::from_string("x ".repeat(100), None);
        let mut cursor_set = CursorSet::new(positions[0]);

        for pos in &positions[1..] {
            cursor_set.add_cursor(*pos);
        }

        let initial_count = cursor_set.len();

        // Move all cursors right
        cursor_set.apply(&buffer, |cursor, buffer| {
            cursor.move_right(buffer, 1);
        });

        // All cursors should still exist
        prop_assert_eq!(cursor_set.len(), initial_count);

        // All should have moved
        for cursor in cursor_set.cursors() {
            prop_assert!(cursor.pos() > 0);
        }
    }
}
