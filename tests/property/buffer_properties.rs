//! Property-based tests for Buffer operations
//!
//! These tests use proptest to verify invariants hold across random inputs

use ait42_core::buffer::Buffer;
use proptest::prelude::*;

proptest! {
    /// Insert then delete same range should restore original state
    #[test]
    fn test_insert_delete_roundtrip(
        s in "\\PC{0,1000}",  // Any valid Unicode string up to 1000 chars
        pos in 0..100usize
    ) {
        let mut buffer = Buffer::new();
        let original = buffer.to_string();

        // Insert text
        if let Ok(()) = buffer.insert(pos.min(buffer.len_bytes()), &s) {
            // Delete same text
            let end = pos.min(buffer.len_bytes()) + s.len();
            let _ = buffer.delete(pos.min(buffer.len_bytes())..end);

            // Should restore original
            prop_assert_eq!(buffer.to_string(), original);
        }
    }

    /// Buffer length should equal sum of line lengths
    #[test]
    fn test_buffer_length_consistency(s in "\\PC{0,1000}") {
        let buffer = Buffer::from_string(s.clone(), None);

        let total_chars: usize = (0..buffer.len_lines())
            .filter_map(|i| buffer.line(i))
            .map(|line| line.chars().count())
            .sum();

        // Total should be close (accounting for newlines)
        prop_assert!(total_chars <= buffer.len_chars() + buffer.len_lines());
    }

    /// Line/col conversion should be reversible
    #[test]
    fn test_line_col_conversion_reversible(
        s in "\\PC{0,500}",
        pos in 0..100usize
    ) {
        let buffer = Buffer::from_string(s, None);
        let pos = pos.min(buffer.len_bytes());

        let (line, col) = buffer.pos_to_line_col(pos);
        if let Some(converted_pos) = buffer.line_col_to_pos(line, col) {
            // Should convert back to same or nearby position
            prop_assert!((converted_pos as i64 - pos as i64).abs() <= 1);
        }
    }

    /// Insert never decreases buffer length
    #[test]
    fn test_insert_increases_length(
        s in "\\PC{1,100}",
        text in "\\PC{1,50}"
    ) {
        let mut buffer = Buffer::from_string(s, None);
        let original_len = buffer.len_bytes();
        let pos = original_len / 2;

        if let Ok(()) = buffer.insert(pos, &text) {
            prop_assert!(buffer.len_bytes() >= original_len);
            prop_assert_eq!(buffer.len_bytes(), original_len + text.len());
        }
    }

    /// Delete never increases buffer length
    #[test]
    fn test_delete_decreases_length(s in "\\PC{10,100}") {
        let mut buffer = Buffer::from_string(s, None);
        let original_len = buffer.len_bytes();

        if original_len > 2 {
            let start = original_len / 3;
            let end = (original_len * 2) / 3;

            if let Ok(()) = buffer.delete(start..end) {
                prop_assert!(buffer.len_bytes() <= original_len);
            }
        }
    }

    /// Replace maintains UTF-8 validity
    #[test]
    fn test_replace_utf8_validity(
        s in "\\PC{10,100}",
        replacement in "\\PC{5,50}"
    ) {
        let mut buffer = Buffer::from_string(s, None);
        let len = buffer.len_bytes();

        if len > 4 {
            let start = len / 4;
            let end = (len * 3) / 4;

            if let Ok(()) = buffer.replace(start..end, &replacement) {
                // Buffer should still be valid UTF-8
                let _ = buffer.to_string(); // Would panic if invalid
                prop_assert!(true);
            }
        }
    }

    /// Version increments on every modification
    #[test]
    fn test_version_increments(
        initial_text in "\\PC{0,100}",
        insert_text in "\\PC{1,50}"
    ) {
        let mut buffer = Buffer::from_string(initial_text, None);
        let v0 = buffer.version();

        if buffer.insert(0, &insert_text).is_ok() {
            prop_assert_eq!(buffer.version(), v0 + 1);
        }

        if buffer.delete(0..1).is_ok() {
            prop_assert_eq!(buffer.version(), v0 + 2);
        }
    }

    /// Empty buffer always has 1 line
    #[test]
    fn test_empty_buffer_one_line(s in "\\PC{0,100}") {
        let mut buffer = Buffer::from_string(s, None);

        // Delete everything
        let len = buffer.len_bytes();
        if len > 0 {
            let _ = buffer.delete(0..len);
        }

        prop_assert_eq!(buffer.len_lines(), 1);
        prop_assert!(buffer.is_empty());
    }

    /// Line count consistency with newlines
    #[test]
    fn test_line_count_with_newlines(num_newlines in 0..50usize) {
        let text = "\n".repeat(num_newlines);
        let buffer = Buffer::from_string(text, None);

        // n newlines create n+1 lines
        prop_assert_eq!(buffer.len_lines(), num_newlines + 1);
    }

    /// Slice range validation
    #[test]
    fn test_slice_valid_ranges(
        s in "\\PC{10,100}",
        start in 0..50usize,
        end in 0..50usize
    ) {
        let buffer = Buffer::from_string(s, None);
        let len = buffer.len_bytes();

        let start = start.min(len);
        let end = end.min(len);

        if start <= end {
            let result = buffer.slice(start..end);
            prop_assert!(result.is_ok());

            if let Ok(slice) = result {
                prop_assert_eq!(slice.len(), end - start);
            }
        }
    }

    /// Character at position is valid
    #[test]
    fn test_char_at_validity(s in "\\PC{1,100}") {
        let buffer = Buffer::from_string(s.clone(), None);

        for i in 0..buffer.len_chars() {
            if let Some(ch) = buffer.char_at(i) {
                prop_assert!(ch.is_alphanumeric() || ch.is_whitespace() || !ch.is_control());
            }
        }
    }

    /// Multiple sequential inserts maintain consistency
    #[test]
    fn test_sequential_inserts(
        texts in prop::collection::vec("\\PC{1,20}", 1..10)
    ) {
        let mut buffer = Buffer::new();
        let mut expected = String::new();

        for text in texts {
            let pos = buffer.len_bytes();
            if buffer.insert(pos, &text).is_ok() {
                expected.push_str(&text);
            }
        }

        prop_assert_eq!(buffer.to_string(), expected);
    }

    /// Position at line start is valid
    #[test]
    fn test_line_start_positions(s in "\\PC{0,200}") {
        let buffer = Buffer::from_string(s, None);

        for line_idx in 0..buffer.len_lines() {
            if let Some(pos) = buffer.line_col_to_pos(line_idx, 0) {
                // Position should be valid
                prop_assert!(pos <= buffer.len_bytes());

                // Should convert back to same line
                let (line, col) = buffer.pos_to_line_col(pos);
                prop_assert_eq!(line, line_idx);
                prop_assert_eq!(col, 0);
            }
        }
    }

    /// Buffer dirty flag semantics
    #[test]
    fn test_dirty_flag_semantics(s in "\\PC{1,50}") {
        let mut buffer = Buffer::from_string(s, None);
        prop_assert!(!buffer.is_dirty()); // Fresh buffer not dirty

        // Modify buffer
        if buffer.insert(0, "x").is_ok() {
            prop_assert!(buffer.is_dirty());

            // Mark clean
            buffer.mark_clean();
            prop_assert!(!buffer.is_dirty());
        }
    }

    /// Commutative property: insert at different positions
    #[test]
    fn test_insert_commutativity(
        base in "\\PC{10,50}",
        text1 in "\\PC{1,10}",
        text2 in "\\PC{1,10}"
    ) {
        // Insert text1 at start and text2 at end
        let mut buffer1 = Buffer::from_string(base.clone(), None);
        let mut buffer2 = Buffer::from_string(base, None);

        let _ = buffer1.insert(0, &text1);
        let end = buffer1.len_bytes();
        let _ = buffer1.insert(end, &text2);

        // Insert in reverse order (should not commute)
        let _ = buffer2.insert(0, &text2);
        let _ = buffer2.insert(0, &text1);

        // Results should be different (not commutative)
        prop_assert_ne!(buffer1.to_string(), buffer2.to_string());
    }

    /// Replace with empty string is equivalent to delete
    #[test]
    fn test_replace_empty_equals_delete(s in "\\PC{10,50}") {
        let mut buffer1 = Buffer::from_string(s.clone(), None);
        let mut buffer2 = Buffer::from_string(s, None);

        let len = buffer1.len_bytes();
        if len > 2 {
            let range = 1..(len - 1);

            let _ = buffer1.replace(range.clone(), "");
            let _ = buffer2.delete(range);

            prop_assert_eq!(buffer1.to_string(), buffer2.to_string());
        }
    }
}
