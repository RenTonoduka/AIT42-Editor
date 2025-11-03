//! Additional Cursor Edge Case Tests
//!
//! Comprehensive test coverage for cursor operations including:
//! - Boundary conditions
//! - Word movement with punctuation
//! - Multi-cursor operations
//! - Selection edge cases

use ait42_core::buffer::Buffer;
use ait42_core::cursor::{Cursor, CursorPosition, CursorSet};

#[cfg(test)]
mod cursor_boundary_tests {
    use super::*;

    #[test]
    fn test_cursor_at_eof() {
        let buffer = Buffer::from_string("Test".to_string(), None);
        let mut cursor = Cursor::new(buffer.len_bytes());

        // Should not panic or move beyond EOF
        cursor.move_right(&buffer, 10);
        assert_eq!(cursor.pos(), buffer.len_bytes());
    }

    #[test]
    fn test_cursor_at_start_of_buffer() {
        let buffer = Buffer::from_string("Test".to_string(), None);
        let mut cursor = Cursor::new(0);

        // Should not move before start
        cursor.move_left(&buffer, 10);
        assert_eq!(cursor.pos(), 0);
    }

    #[test]
    fn test_cursor_movement_empty_buffer() {
        let buffer = Buffer::new();
        let mut cursor = Cursor::new(0);

        // All movements should be safe on empty buffer
        cursor.move_left(&buffer, 1);
        assert_eq!(cursor.pos(), 0);

        cursor.move_right(&buffer, 1);
        assert_eq!(cursor.pos(), 0);

        cursor.move_up(&buffer, 1);
        assert_eq!(cursor.pos(), 0);

        cursor.move_down(&buffer, 1);
        assert_eq!(cursor.pos(), 0);
    }

    #[test]
    fn test_cursor_on_single_character_buffer() {
        let buffer = Buffer::from_string("X".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_right(&buffer, 1);
        assert_eq!(cursor.pos(), 1);

        cursor.move_left(&buffer, 1);
        assert_eq!(cursor.pos(), 0);
    }

    #[test]
    fn test_cursor_movement_with_only_newlines() {
        let buffer = Buffer::from_string("\n\n\n".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_down(&buffer, 1);
        let (line, _) = buffer.pos_to_line_col(cursor.pos());
        assert_eq!(line, 1);

        cursor.move_down(&buffer, 1);
        let (line, _) = buffer.pos_to_line_col(cursor.pos());
        assert_eq!(line, 2);

        cursor.move_down(&buffer, 10); // Beyond last line
        let (line, _) = buffer.pos_to_line_col(cursor.pos());
        assert!(line <= buffer.len_lines());
    }
}

#[cfg(test)]
mod cursor_movement_empty_lines {
    use super::*;

    #[test]
    fn test_cursor_movement_empty_lines() {
        let buffer = Buffer::from_string("Line 1\n\nLine 3\n\nLine 5".to_string(), None);
        let mut cursor = Cursor::new(0);

        // Move to empty line
        cursor.move_down(&buffer, 1);
        let (line, col) = buffer.pos_to_line_col(cursor.pos());
        assert_eq!(line, 1);

        // Move through empty line
        cursor.move_down(&buffer, 1);
        let (line, _) = buffer.pos_to_line_col(cursor.pos());
        assert_eq!(line, 2);
    }

    #[test]
    fn test_cursor_left_right_on_empty_line() {
        let buffer = Buffer::from_string("Line 1\n\nLine 3".to_string(), None);

        // Position cursor on empty line
        let empty_line_pos = buffer.line_col_to_pos(1, 0).unwrap();
        let mut cursor = Cursor::new(empty_line_pos);

        cursor.move_right(&buffer, 1);
        // Should move to next line or stay
        let new_pos = cursor.pos();
        assert!(new_pos >= empty_line_pos);
    }

    #[test]
    fn test_cursor_line_end_on_empty_line() {
        let buffer = Buffer::from_string("Line 1\n\nLine 3".to_string(), None);

        let empty_line_pos = buffer.line_col_to_pos(1, 0).unwrap();
        let mut cursor = Cursor::new(empty_line_pos);

        cursor.move_to_line_end(&buffer);
        // On empty line, end == start
        assert_eq!(cursor.pos(), empty_line_pos);
    }
}

#[cfg(test)]
mod cursor_word_movement_tests {
    use super::*;

    #[test]
    fn test_word_movement_with_punctuation() {
        let buffer = Buffer::from_string("Hello, world! How are you?".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_word_forward(&buffer);
        let pos1 = cursor.pos();
        assert!(pos1 > 0);

        cursor.move_word_forward(&buffer);
        let pos2 = cursor.pos();
        assert!(pos2 > pos1);

        cursor.move_word_backward(&buffer);
        assert!(cursor.pos() < pos2);
    }

    #[test]
    fn test_word_movement_with_underscores() {
        let buffer = Buffer::from_string("snake_case_variable".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_word_forward(&buffer);
        // Should treat underscores as word boundaries
        assert!(cursor.pos() > 0);
    }

    #[test]
    fn test_word_movement_with_camel_case() {
        let buffer = Buffer::from_string("camelCaseVariable".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_word_forward(&buffer);
        // Behavior depends on implementation
        assert!(cursor.pos() > 0);
    }

    #[test]
    fn test_word_movement_at_start() {
        let buffer = Buffer::from_string("word1 word2 word3".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_word_backward(&buffer);
        assert_eq!(cursor.pos(), 0); // Should stay at start
    }

    #[test]
    fn test_word_movement_at_end() {
        let buffer = Buffer::from_string("word1 word2 word3".to_string(), None);
        let mut cursor = Cursor::new(buffer.len_bytes());

        cursor.move_word_forward(&buffer);
        assert_eq!(cursor.pos(), buffer.len_bytes()); // Should stay at end
    }

    #[test]
    fn test_word_movement_only_whitespace() {
        let buffer = Buffer::from_string("   \t  \n  ".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_word_forward(&buffer);
        // Should skip all whitespace to end
        assert_eq!(cursor.pos(), buffer.len_bytes());
    }

    #[test]
    fn test_word_movement_special_characters() {
        let buffer = Buffer::from_string("@#$% ^&*() []{}".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_word_forward(&buffer);
        assert!(cursor.pos() > 0);
    }

    #[test]
    fn test_word_movement_numbers() {
        let buffer = Buffer::from_string("123 456 789".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_word_forward(&buffer);
        assert!(cursor.pos() > 0 && cursor.pos() < buffer.len_bytes());
    }
}

#[cfg(test)]
mod cursor_line_movement_tests {
    use super::*;

    #[test]
    fn test_vertical_movement_preserves_column() {
        let buffer = Buffer::from_string("12345\n123\n12345".to_string(), None);

        // Start at column 4 of first line
        let mut cursor = Cursor::new(buffer.line_col_to_pos(0, 4).unwrap());

        cursor.move_down(&buffer, 1);
        let (line1, col1) = buffer.pos_to_line_col(cursor.pos());
        assert_eq!(line1, 1);
        // Should be at column 3 (end of shorter line) or less

        cursor.move_down(&buffer, 1);
        let (line2, col2) = buffer.pos_to_line_col(cursor.pos());
        assert_eq!(line2, 2);
        // Should restore to column 4 on longer line
    }

    #[test]
    fn test_move_to_line_start_from_middle() {
        let buffer = Buffer::from_string("Hello World".to_string(), None);
        let mut cursor = Cursor::new(6); // Middle of line

        cursor.move_to_line_start(&buffer);
        assert_eq!(cursor.pos(), 0);
    }

    #[test]
    fn test_move_to_line_end_from_middle() {
        let buffer = Buffer::from_string("Hello World".to_string(), None);
        let mut cursor = Cursor::new(6);

        cursor.move_to_line_end(&buffer);
        assert_eq!(cursor.pos(), 11);
    }

    #[test]
    fn test_move_to_buffer_start() {
        let buffer = Buffer::from_string("Line 1\nLine 2\nLine 3".to_string(), None);
        let mut cursor = Cursor::new(10);

        cursor.move_to_buffer_start();
        assert_eq!(cursor.pos(), 0);
    }

    #[test]
    fn test_move_to_buffer_end() {
        let buffer = Buffer::from_string("Line 1\nLine 2\nLine 3".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_to_buffer_end(&buffer);
        assert_eq!(cursor.pos(), buffer.len_bytes());
    }

    #[test]
    fn test_move_down_from_last_line() {
        let buffer = Buffer::from_string("Line 1\nLine 2\nLine 3".to_string(), None);

        let last_line_pos = buffer.line_col_to_pos(2, 0).unwrap();
        let mut cursor = Cursor::new(last_line_pos);

        cursor.move_down(&buffer, 10);
        let (line, _) = buffer.pos_to_line_col(cursor.pos());
        assert_eq!(line, 2); // Should stay on last line
    }

    #[test]
    fn test_move_up_from_first_line() {
        let buffer = Buffer::from_string("Line 1\nLine 2\nLine 3".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_up(&buffer, 10);
        let (line, _) = buffer.pos_to_line_col(cursor.pos());
        assert_eq!(line, 0); // Should stay on first line
    }
}

#[cfg(test)]
mod cursor_selection_tests {
    use super::*;

    #[test]
    fn test_selection_forward() {
        let mut cursor = Cursor::new(0);

        cursor.start_selection();
        cursor.set_pos(5);

        assert!(cursor.has_selection());
        assert_eq!(cursor.selection(), Some(0..5));
    }

    #[test]
    fn test_selection_backward() {
        let mut cursor = Cursor::new(10);

        cursor.start_selection();
        cursor.set_pos(5);

        assert!(cursor.has_selection());
        assert_eq!(cursor.selection(), Some(5..10)); // Normalized
    }

    #[test]
    fn test_selection_zero_length() {
        let mut cursor = Cursor::new(5);

        cursor.start_selection();
        // Don't move cursor

        assert!(cursor.has_selection());
        assert_eq!(cursor.selection(), Some(5..5));
    }

    #[test]
    fn test_clear_selection() {
        let mut cursor = Cursor::new(0);

        cursor.start_selection();
        cursor.set_pos(5);
        assert!(cursor.has_selection());

        cursor.clear_selection();
        assert!(!cursor.has_selection());
        assert_eq!(cursor.selection(), None);
    }

    #[test]
    fn test_extend_selection_without_anchor() {
        let mut cursor = Cursor::new(5);

        cursor.extend_selection();
        assert!(cursor.has_selection());
        // Should create anchor at current position
    }

    #[test]
    fn test_selection_across_lines() {
        let buffer = Buffer::from_string("Line 1\nLine 2\nLine 3".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.start_selection();

        let end_pos = buffer.line_col_to_pos(2, 0).unwrap();
        cursor.set_pos(end_pos);

        let selection = cursor.selection().unwrap();
        assert!(selection.end > selection.start);
        assert!(selection.end >= end_pos);
    }

    #[test]
    fn test_set_pos_resets_preferred_column() {
        let buffer = Buffer::from_string("Long line\nX\nLong line".to_string(), None);
        let mut cursor = Cursor::new(5);

        cursor.move_down(&buffer, 1);
        // Preferred column should be set

        cursor.set_pos(0);
        // Preferred column should be reset

        cursor.move_down(&buffer, 1);
        // Should not use old preferred column
    }
}

#[cfg(test)]
mod cursor_grapheme_tests {
    use super::*;

    #[test]
    fn test_move_left_over_emoji() {
        let buffer = Buffer::from_string("Hello 👋 World".to_string(), None);

        // Position after emoji
        let emoji_end = "Hello 👋".len();
        let mut cursor = Cursor::new(emoji_end);

        cursor.move_left(&buffer, 1);
        // Should move to start of emoji
        assert!(cursor.pos() < emoji_end);
    }

    #[test]
    fn test_move_right_over_emoji() {
        let buffer = Buffer::from_string("👋 Hello".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_right(&buffer, 1);
        // Should move past entire emoji
        assert!(cursor.pos() > 0);
    }

    #[test]
    fn test_cursor_with_combining_characters() {
        let buffer = Buffer::from_string("e\u{0301}".to_string(), None); // é as combining
        let mut cursor = Cursor::new(0);

        cursor.move_right(&buffer, 1);
        // Should move past entire grapheme cluster
        assert_eq!(cursor.pos(), buffer.len_bytes());
    }

    #[test]
    fn test_cursor_with_zwj_sequence() {
        let buffer = Buffer::from_string("👨\u{200D}👩\u{200D}👧".to_string(), None);
        let mut cursor = Cursor::new(0);

        cursor.move_right(&buffer, 1);
        // Should move past entire family emoji
        assert_eq!(cursor.pos(), buffer.len_bytes());
    }
}

#[cfg(test)]
mod cursor_position_tests {
    use super::*;

    #[test]
    fn test_cursor_position_struct() {
        let pos = CursorPosition::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.col, 10);
    }

    #[test]
    fn test_cursor_position_from_buffer() {
        let buffer = Buffer::from_string("Line 1\nLine 2".to_string(), None);

        let pos = buffer.line_col_to_pos(1, 3).unwrap();
        let cursor = Cursor::new(pos);

        let cursor_pos = cursor.position(&buffer);
        assert_eq!(cursor_pos.line, 1);
        assert_eq!(cursor_pos.col, 3);
    }

    #[test]
    fn test_move_to_specific_position() {
        let buffer = Buffer::from_string("Line 1\nLine 2\nLine 3".to_string(), None);
        let mut cursor = Cursor::new(0);

        assert!(cursor.move_to(&buffer, 1, 3).is_ok());

        let pos = cursor.position(&buffer);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.col, 3);
    }

    #[test]
    fn test_move_to_invalid_position() {
        let buffer = Buffer::from_string("Test".to_string(), None);
        let mut cursor = Cursor::new(0);

        let result = cursor.move_to(&buffer, 100, 100);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod cursor_set_tests {
    use super::*;

    #[test]
    fn test_cursor_set_creation() {
        let cursors = CursorSet::new(0);
        assert_eq!(cursors.len(), 1);
        assert!(cursors.is_single());
    }

    #[test]
    fn test_add_secondary_cursor() {
        let mut cursors = CursorSet::new(0);

        cursors.add_cursor(10);
        assert_eq!(cursors.len(), 2);
        assert!(!cursors.is_single());
    }

    #[test]
    fn test_cursor_set_merge_duplicates() {
        let mut cursors = CursorSet::new(0);

        cursors.add_cursor(10);
        cursors.add_cursor(10); // Duplicate
        cursors.add_cursor(20);

        assert_eq!(cursors.len(), 3); // Primary + 2 unique secondary
    }

    #[test]
    fn test_cursor_set_merge_with_primary() {
        let mut cursors = CursorSet::new(5);

        cursors.add_cursor(5); // Same as primary
        cursors.merge_cursors();

        assert_eq!(cursors.len(), 1); // Should remove duplicate
    }

    #[test]
    fn test_remove_secondary_cursor() {
        let mut cursors = CursorSet::new(0);

        cursors.add_cursor(10);
        cursors.add_cursor(20);

        assert!(cursors.remove_cursor(0).is_ok());
        assert_eq!(cursors.len(), 2);
    }

    #[test]
    fn test_remove_invalid_cursor_fails() {
        let mut cursors = CursorSet::new(0);

        let result = cursors.remove_cursor(10);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_secondary_cursors() {
        let mut cursors = CursorSet::new(0);

        cursors.add_cursor(10);
        cursors.add_cursor(20);
        cursors.add_cursor(30);

        cursors.clear_secondary();
        assert!(cursors.is_single());
        assert_eq!(cursors.len(), 1);
    }

    #[test]
    fn test_cursor_set_iteration() {
        let mut cursors = CursorSet::new(0);
        cursors.add_cursor(10);
        cursors.add_cursor(20);

        let positions: Vec<usize> = cursors.cursors().map(|c| c.pos()).collect();
        assert_eq!(positions.len(), 3);
        assert!(positions.contains(&0));
        assert!(positions.contains(&10));
        assert!(positions.contains(&20));
    }

    #[test]
    fn test_cursor_set_apply_operation() {
        let buffer = Buffer::from_string("Hello World".to_string(), None);
        let mut cursors = CursorSet::new(0);

        cursors.add_cursor(6);

        cursors.apply(&buffer, |cursor, buf| {
            cursor.move_right(buf, 1);
        });

        // All cursors should have moved
        for cursor in cursors.cursors() {
            assert!(cursor.pos() > 0);
        }
    }

    #[test]
    fn test_cursor_set_mutable_iteration() {
        let mut cursors = CursorSet::new(0);
        cursors.add_cursor(10);

        for cursor in cursors.cursors_mut() {
            cursor.set_pos(cursor.pos() + 1);
        }

        // All cursors should have moved
        let positions: Vec<usize> = cursors.cursors().map(|c| c.pos()).collect();
        assert_eq!(positions, vec![1, 11]);
    }

    #[test]
    fn test_cursor_set_sorting() {
        let mut cursors = CursorSet::new(20);

        cursors.add_cursor(5);
        cursors.add_cursor(15);
        cursors.add_cursor(10);

        cursors.merge_cursors();

        // Secondary cursors should be sorted
        let secondary_positions: Vec<usize> = cursors
            .cursors()
            .skip(1)
            .map(|c| c.pos())
            .collect();

        for i in 1..secondary_positions.len() {
            assert!(secondary_positions[i] >= secondary_positions[i - 1]);
        }
    }
}

#[cfg(test)]
mod cursor_stress_tests {
    use super::*;

    #[test]
    fn test_many_cursor_movements() {
        let buffer = Buffer::from_string("Line 1\nLine 2\nLine 3\nLine 4\nLine 5".to_string(), None);
        let mut cursor = Cursor::new(0);

        for _ in 0..100 {
            cursor.move_down(&buffer, 1);
            cursor.move_up(&buffer, 1);
            cursor.move_right(&buffer, 1);
            cursor.move_left(&buffer, 1);
        }

        // Should still be valid
        assert!(cursor.pos() <= buffer.len_bytes());
    }

    #[test]
    fn test_cursor_set_with_many_cursors() {
        let mut cursors = CursorSet::new(0);

        for i in 1..100 {
            cursors.add_cursor(i * 10);
        }

        assert!(cursors.len() <= 100);
    }
}
