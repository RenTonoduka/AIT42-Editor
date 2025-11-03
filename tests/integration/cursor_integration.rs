//! Integration tests for Cursor operations

use ait42_core::buffer::Buffer;
use ait42_core::cursor::{Cursor, CursorSet};

#[test]
fn test_cursor_unicode_movement() {
    // Test cursor movement over Unicode grapheme clusters
    let buffer = Buffer::from_string("Hello 👨‍👩‍👧‍👦 World 🌍".to_string(), None);
    let mut cursor = Cursor::new(0);

    // Move right through multi-byte emoji (family emoji is ~25 bytes)
    cursor.move_right(&buffer, 1); // H -> e
    cursor.move_right(&buffer, 5); // e -> space after "Hello"

    // Should handle emoji as single grapheme
    cursor.move_right(&buffer, 1); // Should skip entire family emoji
    let pos_after_emoji = cursor.pos();

    cursor.move_left(&buffer, 1);
    assert_eq!(cursor.pos(), 6); // Back to space
}

#[test]
fn test_cursor_empty_buffer() {
    let buffer = Buffer::new();
    let mut cursor = Cursor::new(0);

    // All movements should keep cursor at 0
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
fn test_cursor_single_character() {
    let buffer = Buffer::from_string("x".to_string(), None);
    let mut cursor = Cursor::new(0);

    cursor.move_right(&buffer, 1);
    assert_eq!(cursor.pos(), 1);

    cursor.move_left(&buffer, 1);
    assert_eq!(cursor.pos(), 0);
}

#[test]
fn test_cursor_line_boundaries() {
    let buffer = Buffer::from_string("abc\ndef\nghi".to_string(), None);
    let mut cursor = Cursor::new(0);

    // Move to end of first line
    cursor.move_to_line_end(&buffer);
    assert_eq!(cursor.pos(), 3); // At '\n'

    // Move down should go to start of next line (preserving column 3)
    cursor.move_down(&buffer, 1);
    let (line, col) = buffer.pos_to_line_col(cursor.pos());
    assert_eq!(line, 1);
    assert_eq!(col, 3); // At '\n' of second line

    // Move to line start
    cursor.move_to_line_start(&buffer);
    let (line, col) = buffer.pos_to_line_col(cursor.pos());
    assert_eq!(line, 1);
    assert_eq!(col, 0);
}

#[test]
fn test_cursor_preferred_column() {
    // Test that cursor preserves column when moving vertically
    let buffer = Buffer::from_string("short\nvery long line here\nshort".to_string(), None);
    let mut cursor = Cursor::new(0);

    // Move to column 3 of first line
    cursor.move_to(&buffer, 0, 3).unwrap();

    // Move down to long line - should stay at column 3
    cursor.move_down(&buffer, 1);
    let (_, col) = buffer.pos_to_line_col(cursor.pos());
    assert_eq!(col, 3);

    // Move down to short line again - should clamp to line length but remember preferred col
    cursor.move_down(&buffer, 1);
    let (_, col) = buffer.pos_to_line_col(cursor.pos());
    assert!(col <= 5); // "short" has 5 chars
}

#[test]
fn test_cursor_buffer_start_end() {
    let buffer = Buffer::from_string("Hello\nWorld\n!".to_string(), None);
    let mut cursor = Cursor::new(5); // Middle position

    cursor.move_to_buffer_start();
    assert_eq!(cursor.pos(), 0);

    cursor.move_to_buffer_end(&buffer);
    assert_eq!(cursor.pos(), buffer.len_bytes());
}

#[test]
fn test_cursor_word_movement() {
    let buffer = Buffer::from_string("hello world test".to_string(), None);
    let mut cursor = Cursor::new(0);

    // Move forward by word
    cursor.move_word_forward(&buffer);
    assert!(cursor.pos() > 0); // Should move past "hello"

    cursor.move_word_forward(&buffer);
    assert!(cursor.pos() > 6); // Should move past "world"

    // Move backward by word
    cursor.move_word_backward(&buffer);
    assert!(cursor.pos() < 12); // Should move back

    cursor.move_word_backward(&buffer);
    assert!(cursor.pos() < 6); // Should move back more
}

#[test]
fn test_cursor_word_movement_boundaries() {
    let buffer = Buffer::from_string("word".to_string(), None);
    let mut cursor = Cursor::new(0);

    // Move forward past last word - should go to end
    cursor.move_word_forward(&buffer);
    assert_eq!(cursor.pos(), buffer.len_bytes());

    // Move backward from end - should go to start of word
    cursor.move_word_backward(&buffer);
    assert_eq!(cursor.pos(), 0);

    // Move backward from start - should stay at start
    cursor.move_word_backward(&buffer);
    assert_eq!(cursor.pos(), 0);
}

#[test]
fn test_cursor_selection_normalization() {
    let mut cursor = Cursor::new(10);

    // Select backwards
    cursor.start_selection();
    cursor.set_pos(5);

    // Selection should be normalized (start < end)
    let selection = cursor.selection().unwrap();
    assert_eq!(selection.start, 5);
    assert_eq!(selection.end, 10);
}

#[test]
fn test_cursor_selection_clear() {
    let mut cursor = Cursor::new(0);

    cursor.start_selection();
    cursor.set_pos(10);
    assert!(cursor.has_selection());

    cursor.clear_selection();
    assert!(!cursor.has_selection());
    assert_eq!(cursor.selection(), None);
}

#[test]
fn test_cursor_selection_extend() {
    let mut cursor = Cursor::new(5);

    // Start selection
    cursor.extend_selection();
    assert!(cursor.has_selection());

    cursor.set_pos(10);
    assert_eq!(cursor.selection(), Some(5..10));

    cursor.set_pos(15);
    assert_eq!(cursor.selection(), Some(5..15));
}

#[test]
fn test_cursor_set_single_cursor() {
    let mut cursors = CursorSet::new(0);

    assert_eq!(cursors.len(), 1);
    assert!(cursors.is_single());

    // Primary cursor operations
    cursors.primary_mut().set_pos(10);
    assert_eq!(cursors.primary().pos(), 10);
}

#[test]
fn test_cursor_set_multiple_cursors() {
    let mut cursors = CursorSet::new(0);

    // Add secondary cursors
    cursors.add_cursor(10);
    cursors.add_cursor(20);
    cursors.add_cursor(30);

    assert_eq!(cursors.len(), 4); // 1 primary + 3 secondary
    assert!(!cursors.is_single());

    // Verify all cursors
    let positions: Vec<usize> = cursors.cursors().map(|c| c.pos()).collect();
    assert_eq!(positions, vec![0, 10, 20, 30]);
}

#[test]
fn test_cursor_set_merge_overlapping() {
    let mut cursors = CursorSet::new(0);

    cursors.add_cursor(10);
    cursors.add_cursor(20);
    cursors.add_cursor(10); // Duplicate

    // After merge, should have 3 unique cursors
    cursors.merge_cursors();
    assert_eq!(cursors.len(), 3);
}

#[test]
fn test_cursor_set_merge_with_primary() {
    let mut cursors = CursorSet::new(0);

    cursors.add_cursor(10);
    cursors.add_cursor(0); // Same as primary

    cursors.merge_cursors();
    assert_eq!(cursors.len(), 2); // Primary + one secondary (duplicate removed)
}

#[test]
fn test_cursor_set_apply_operation() {
    let buffer = Buffer::from_string("abc\ndef\nghi".to_string(), None);
    let mut cursors = CursorSet::new(0);

    cursors.add_cursor(4); // Start of second line
    cursors.add_cursor(8); // Start of third line

    // Move all cursors right
    cursors.apply(&buffer, |cursor, buffer| {
        cursor.move_right(buffer, 1);
    });

    // Verify all moved
    let positions: Vec<usize> = cursors.cursors().map(|c| c.pos()).collect();
    assert!(positions.iter().all(|&pos| pos > 0));
}

#[test]
fn test_cursor_set_remove_cursor() {
    let mut cursors = CursorSet::new(0);

    cursors.add_cursor(10);
    cursors.add_cursor(20);

    assert_eq!(cursors.len(), 3);

    // Remove first secondary cursor
    cursors.remove_cursor(0).unwrap();
    assert_eq!(cursors.len(), 2);

    // Try to remove invalid index
    assert!(cursors.remove_cursor(10).is_err());
}

#[test]
fn test_cursor_set_clear_secondary() {
    let mut cursors = CursorSet::new(0);

    cursors.add_cursor(10);
    cursors.add_cursor(20);
    cursors.add_cursor(30);

    assert_eq!(cursors.len(), 4);

    cursors.clear_secondary();
    assert_eq!(cursors.len(), 1);
    assert!(cursors.is_single());
}

#[test]
fn test_cursor_move_to_invalid_position() {
    let buffer = Buffer::from_string("abc".to_string(), None);
    let mut cursor = Cursor::new(0);

    // Try to move to invalid line/col
    let result = cursor.move_to(&buffer, 100, 100);
    assert!(result.is_err());

    // Cursor should not have moved
    assert_eq!(cursor.pos(), 0);
}

#[test]
fn test_cursor_large_movements() {
    let buffer = Buffer::from_string("a\n".repeat(1000), None);
    let mut cursor = Cursor::new(0);

    // Large downward movement
    cursor.move_down(&buffer, 500);
    let (line, _) = buffer.pos_to_line_col(cursor.pos());
    assert_eq!(line, 500);

    // Large upward movement
    cursor.move_up(&buffer, 250);
    let (line, _) = buffer.pos_to_line_col(cursor.pos());
    assert_eq!(line, 250);

    // Move past end of buffer
    cursor.move_down(&buffer, 10000);
    let (line, _) = buffer.pos_to_line_col(cursor.pos());
    assert!(line >= 999);
}

#[test]
fn test_cursor_empty_lines() {
    let buffer = Buffer::from_string("\n\n\n".to_string(), None);
    let mut cursor = Cursor::new(0);

    // Move down through empty lines
    cursor.move_down(&buffer, 1);
    assert_eq!(cursor.pos(), 1);

    cursor.move_down(&buffer, 1);
    assert_eq!(cursor.pos(), 2);

    // Move to line end on empty line
    cursor.move_to_line_end(&buffer);
    assert_eq!(cursor.pos(), 2); // Should stay at newline

    // Move to line start
    cursor.move_to_line_start(&buffer);
    assert_eq!(cursor.pos(), 2);
}

#[test]
fn test_cursor_trailing_spaces() {
    let buffer = Buffer::from_string("abc   \ndef   ".to_string(), None);
    let mut cursor = Cursor::new(0);

    cursor.move_to_line_end(&buffer);
    let (line, col) = buffer.pos_to_line_col(cursor.pos());
    assert_eq!(line, 0);
    assert_eq!(col, 6); // After trailing spaces
}
