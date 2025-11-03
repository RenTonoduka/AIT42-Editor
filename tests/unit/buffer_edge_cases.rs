//! Additional Buffer Edge Case Tests
//!
//! Comprehensive test coverage for buffer operations with edge cases,
//! UTF-8 handling, emoji support, and boundary conditions.

use ait42_core::buffer::{Buffer, BufferManager, LineEnding};
use std::path::Path;
use tempfile::NamedTempFile;

#[cfg(test)]
mod buffer_utf8_tests {
    use super::*;

    #[test]
    fn test_insert_at_utf8_multibyte_boundary() {
        let mut buffer = Buffer::from_string("Hello 世界".to_string(), None);

        // "世" is 3 bytes (E4 B8 96 in UTF-8), starts at byte 6
        // Try inserting at valid UTF-8 boundary
        assert!(buffer.insert(6, "!").is_ok());
        assert_eq!(buffer.to_string(), "Hello !世界");
    }

    #[test]
    fn test_insert_at_invalid_utf8_boundary() {
        let mut buffer = Buffer::from_string("Hello 世界".to_string(), None);

        // Try inserting in the middle of a multibyte character (should fail)
        let result = buffer.insert(7, "!");
        assert!(result.is_err());
    }

    #[test]
    fn test_buffer_with_emoji() {
        let emoji_text = "Hello 👋 World 🌍";
        let mut buffer = Buffer::from_string(emoji_text.to_string(), None);

        assert_eq!(buffer.to_string(), emoji_text);
        assert!(buffer.len_bytes() > buffer.len_chars());

        // Insert after emoji
        let pos = buffer.line_col_to_pos(0, 7).unwrap();
        assert!(buffer.insert(pos, "!").is_ok());
    }

    #[test]
    fn test_buffer_with_combining_characters() {
        // "é" as combining character: e + combining acute accent
        let text = "Cafe\u{0301}";  // Café with combining accent
        let buffer = Buffer::from_string(text.to_string(), None);

        assert_eq!(buffer.to_string(), text);
        assert!(buffer.len_bytes() >= text.len());
    }

    #[test]
    fn test_delete_across_utf8_boundaries() {
        let mut buffer = Buffer::from_string("Hello 世界 World".to_string(), None);

        // Delete range that includes multibyte characters
        let start = 6;  // Start of "世"
        let end = 12;   // End of "界"

        assert!(buffer.delete(start..end).is_ok());
        assert_eq!(buffer.to_string(), "Hello  World");
    }

    #[test]
    fn test_replace_with_different_byte_lengths() {
        let mut buffer = Buffer::from_string("Hello World".to_string(), None);

        // Replace ASCII with multibyte characters
        assert!(buffer.replace(0..5, "こんにちは").is_ok());
        assert_eq!(buffer.to_string(), "こんにちは World");

        // Verify byte length changed but operation succeeded
        assert!(buffer.len_bytes() > 16);
    }

    #[test]
    fn test_emoji_with_skin_tone_modifiers() {
        let text = "👋🏽";  // Waving hand with skin tone
        let buffer = Buffer::from_string(text.to_string(), None);

        assert_eq!(buffer.to_string(), text);
        // Emoji with modifier is multiple code points
        assert!(buffer.len_chars() > 1);
    }

    #[test]
    fn test_zero_width_joiner_sequences() {
        // Family emoji: 👨‍👩‍👧‍👦 (man, ZWJ, woman, ZWJ, girl, ZWJ, boy)
        let text = "Family: 👨\u{200D}👩\u{200D}👧\u{200D}👦";
        let buffer = Buffer::from_string(text.to_string(), None);

        assert_eq!(buffer.to_string(), text);
        assert!(buffer.len_chars() > 8);
    }

    #[test]
    fn test_right_to_left_text() {
        let text = "Hello مرحبا שלום";  // Mixed LTR/RTL
        let buffer = Buffer::from_string(text.to_string(), None);

        assert_eq!(buffer.to_string(), text);
        assert!(buffer.len_bytes() > text.chars().count());
    }
}

#[cfg(test)]
mod buffer_line_ending_tests {
    use super::*;

    #[test]
    fn test_delete_across_crlf_line_ending() {
        let mut buffer = Buffer::from_string("Line 1\r\nLine 2\r\nLine 3".to_string(), None);
        assert_eq!(buffer.line_ending(), LineEnding::CrLf);

        // Delete across line ending
        assert!(buffer.delete(5..10).is_ok());
        assert!(buffer.to_string().contains("Line 1"));
    }

    #[test]
    fn test_mixed_line_endings() {
        let text = "Line 1\nLine 2\r\nLine 3\rLine 4";
        let buffer = Buffer::from_string(text.to_string(), None);

        // Should detect the first line ending type found
        assert!(buffer.len_lines() >= 4);
    }

    #[test]
    fn test_insert_preserves_line_ending_style() {
        let mut buffer = Buffer::from_string("Line 1\r\nLine 2".to_string(), None);
        let original_ending = buffer.line_ending();

        assert!(buffer.insert(6, " inserted").is_ok());
        assert_eq!(buffer.line_ending(), original_ending);
    }

    #[test]
    fn test_line_ending_detection_empty_buffer() {
        let buffer = Buffer::new();
        // Default should be LF
        assert_eq!(buffer.line_ending(), LineEnding::Lf);
    }

    #[test]
    fn test_line_ending_as_str() {
        assert_eq!(LineEnding::Lf.as_str(), "\n");
        assert_eq!(LineEnding::CrLf.as_str(), "\r\n");
    }
}

#[cfg(test)]
mod buffer_performance_tests {
    use super::*;

    #[test]
    fn test_very_long_lines() {
        let long_line = "a".repeat(10_000);
        let buffer = Buffer::from_string(long_line.clone(), None);

        assert_eq!(buffer.len_chars(), 10_000);
        assert_eq!(buffer.len_lines(), 1);
        assert_eq!(buffer.to_string(), long_line);
    }

    #[test]
    fn test_many_short_lines() {
        let text = (0..1000).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
        let buffer = Buffer::from_string(text.clone(), None);

        assert_eq!(buffer.len_lines(), 1000);
        assert!(buffer.line(500).is_some());
        assert_eq!(buffer.line(500).unwrap().trim(), "Line 500");
    }

    #[test]
    fn test_rapid_sequential_inserts() {
        let mut buffer = Buffer::new();

        for i in 0..1000 {
            let pos = buffer.len_bytes();
            assert!(buffer.insert(pos, &format!("{}", i)).is_ok());
        }

        assert_eq!(buffer.version(), 1000);
        assert!(buffer.is_dirty());
    }

    #[test]
    fn test_rapid_sequential_deletes() {
        let text = "a".repeat(1000);
        let mut buffer = Buffer::from_string(text, None);

        // Delete one character at a time from the end
        for _ in 0..500 {
            let len = buffer.len_bytes();
            if len > 0 {
                assert!(buffer.delete((len - 1)..len).is_ok());
            }
        }

        assert_eq!(buffer.len_chars(), 500);
    }

    #[test]
    fn test_large_file_simulation() {
        // Simulate a 100KB file
        let large_text = "Lorem ipsum dolor sit amet.\n".repeat(3500);
        let buffer = Buffer::from_string(large_text.clone(), None);

        assert!(buffer.len_bytes() > 100_000);
        assert_eq!(buffer.to_string(), large_text);
    }
}

#[cfg(test)]
mod buffer_empty_tests {
    use super::*;

    #[test]
    fn test_empty_buffer_operations() {
        let mut buffer = Buffer::new();

        // All queries should work on empty buffer
        assert_eq!(buffer.len_chars(), 0);
        assert_eq!(buffer.len_bytes(), 0);
        assert_eq!(buffer.len_lines(), 1); // Empty buffer has 1 line
        assert!(buffer.is_empty());
        assert_eq!(buffer.to_string(), "");
    }

    #[test]
    fn test_empty_buffer_line_operations() {
        let buffer = Buffer::new();

        assert!(buffer.line(0).is_some());
        assert!(buffer.line(1).is_none());
        assert_eq!(buffer.line(0).unwrap(), "\n");
    }

    #[test]
    fn test_empty_buffer_position_conversion() {
        let buffer = Buffer::new();

        assert_eq!(buffer.pos_to_line_col(0), (0, 0));
        assert_eq!(buffer.line_col_to_pos(0, 0), Some(0));
    }

    #[test]
    fn test_empty_buffer_char_at() {
        let buffer = Buffer::new();
        assert!(buffer.char_at(0).is_none());
    }

    #[test]
    fn test_empty_buffer_slice() {
        let buffer = Buffer::new();
        assert_eq!(buffer.slice(0..0).unwrap(), "");
    }

    #[test]
    fn test_delete_all_content_returns_to_empty() {
        let mut buffer = Buffer::from_string("Test content".to_string(), None);
        let len = buffer.len_bytes();

        assert!(buffer.delete(0..len).is_ok());
        assert!(buffer.is_empty());
        assert_eq!(buffer.len_lines(), 1);
    }
}

#[cfg(test)]
mod buffer_boundary_tests {
    use super::*;

    #[test]
    fn test_insert_at_zero_position() {
        let mut buffer = Buffer::from_string("World".to_string(), None);
        assert!(buffer.insert(0, "Hello ").is_ok());
        assert_eq!(buffer.to_string(), "Hello World");
    }

    #[test]
    fn test_insert_at_end_position() {
        let mut buffer = Buffer::from_string("Hello".to_string(), None);
        let len = buffer.len_bytes();
        assert!(buffer.insert(len, " World").is_ok());
        assert_eq!(buffer.to_string(), "Hello World");
    }

    #[test]
    fn test_insert_beyond_end_fails() {
        let mut buffer = Buffer::from_string("Test".to_string(), None);
        let result = buffer.insert(1000, "text");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_empty_range() {
        let mut buffer = Buffer::from_string("Test".to_string(), None);
        assert!(buffer.delete(2..2).is_ok());
        assert_eq!(buffer.to_string(), "Test"); // Unchanged
    }

    #[test]
    fn test_delete_inverted_range_fails() {
        let mut buffer = Buffer::from_string("Test".to_string(), None);
        let result = buffer.delete(3..1);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_out_of_bounds_fails() {
        let mut buffer = Buffer::from_string("Test".to_string(), None);
        let result = buffer.delete(0..100);
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_entire_buffer() {
        let mut buffer = Buffer::from_string("Old content".to_string(), None);
        let len = buffer.len_bytes();

        assert!(buffer.replace(0..len, "New content").is_ok());
        assert_eq!(buffer.to_string(), "New content");
    }

    #[test]
    fn test_slice_at_boundaries() {
        let buffer = Buffer::from_string("Test".to_string(), None);

        assert_eq!(buffer.slice(0..0).unwrap(), "");
        assert_eq!(buffer.slice(0..4).unwrap(), "Test");
        assert_eq!(buffer.slice(4..4).unwrap(), "");
    }

    #[test]
    fn test_line_col_at_last_position() {
        let buffer = Buffer::from_string("Line 1\nLine 2".to_string(), None);
        let len = buffer.len_bytes();

        let (line, col) = buffer.pos_to_line_col(len);
        assert!(line < buffer.len_lines());
    }

    #[test]
    fn test_line_col_beyond_last_position() {
        let buffer = Buffer::from_string("Test".to_string(), None);

        let (line, col) = buffer.pos_to_line_col(1000);
        // Should clamp to last valid position
        assert_eq!(line, 0);
    }
}

#[cfg(test)]
mod buffer_versioning_tests {
    use super::*;

    #[test]
    fn test_version_starts_at_zero() {
        let buffer = Buffer::new();
        assert_eq!(buffer.version(), 0);
    }

    #[test]
    fn test_version_increments_on_insert() {
        let mut buffer = Buffer::new();
        assert_eq!(buffer.version(), 0);

        buffer.insert(0, "test").unwrap();
        assert_eq!(buffer.version(), 1);

        buffer.insert(4, "!").unwrap();
        assert_eq!(buffer.version(), 2);
    }

    #[test]
    fn test_version_increments_on_delete() {
        let mut buffer = Buffer::from_string("test".to_string(), None);
        let v0 = buffer.version();

        buffer.delete(0..1).unwrap();
        assert_eq!(buffer.version(), v0 + 1);
    }

    #[test]
    fn test_version_increments_on_replace() {
        let mut buffer = Buffer::from_string("test".to_string(), None);
        let v0 = buffer.version();

        buffer.replace(0..2, "ab").unwrap();
        assert_eq!(buffer.version(), v0 + 1);
    }

    #[test]
    fn test_version_unchanged_on_failed_operations() {
        let mut buffer = Buffer::from_string("test".to_string(), None);
        let v0 = buffer.version();

        // Failed insert
        let _ = buffer.insert(100, "x");
        assert_eq!(buffer.version(), v0);

        // Failed delete
        let _ = buffer.delete(0..100);
        assert_eq!(buffer.version(), v0);
    }
}

#[cfg(test)]
mod buffer_dirty_flag_tests {
    use super::*;

    #[test]
    fn test_new_buffer_not_dirty() {
        let buffer = Buffer::new();
        assert!(!buffer.is_dirty());
    }

    #[test]
    fn test_from_string_not_dirty() {
        let buffer = Buffer::from_string("test".to_string(), None);
        assert!(!buffer.is_dirty());
    }

    #[test]
    fn test_insert_sets_dirty() {
        let mut buffer = Buffer::new();
        buffer.insert(0, "test").unwrap();
        assert!(buffer.is_dirty());
    }

    #[test]
    fn test_delete_sets_dirty() {
        let mut buffer = Buffer::from_string("test".to_string(), None);
        buffer.delete(0..1).unwrap();
        assert!(buffer.is_dirty());
    }

    #[test]
    fn test_replace_sets_dirty() {
        let mut buffer = Buffer::from_string("test".to_string(), None);
        buffer.replace(0..1, "x").unwrap();
        assert!(buffer.is_dirty());
    }

    #[test]
    fn test_mark_clean_clears_dirty() {
        let mut buffer = Buffer::new();
        buffer.insert(0, "test").unwrap();
        assert!(buffer.is_dirty());

        buffer.mark_clean();
        assert!(!buffer.is_dirty());
    }

    #[test]
    fn test_failed_operations_dont_set_dirty() {
        let mut buffer = Buffer::from_string("test".to_string(), None);

        let _ = buffer.insert(100, "x");
        assert!(!buffer.is_dirty());
    }
}

#[cfg(test)]
mod buffer_file_operations_tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_buffer_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "File content").unwrap();

        let buffer = Buffer::from_file(temp_file.path()).unwrap();
        assert_eq!(buffer.to_string(), "File content");
        assert_eq!(buffer.path(), Some(temp_file.path()));
    }

    #[test]
    fn test_buffer_from_nonexistent_file() {
        let result = Buffer::from_file(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_language_detection_from_extension() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("rs");
        write!(temp_file, "fn main() {}").unwrap();
        std::fs::rename(temp_file.path(), &path).unwrap();

        let buffer = Buffer::from_file(&path).unwrap();
        assert_eq!(buffer.language(), Some("rs"));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_save_as_creates_file() {
        let mut buffer = Buffer::from_string("Test content".to_string(), None);
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        assert!(buffer.save_as(path).is_ok());

        let content = std::fs::read_to_string(path).unwrap();
        assert_eq!(content, "Test content");
        assert!(!buffer.is_dirty());
    }

    #[test]
    fn test_save_without_path_fails() {
        let mut buffer = Buffer::new();
        let result = buffer.save();
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod buffer_manager_tests {
    use super::*;

    #[test]
    fn test_manager_starts_empty() {
        let manager = BufferManager::new();
        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
        assert!(manager.active_buffer_id().is_none());
    }

    #[test]
    fn test_new_buffer_becomes_active() {
        let mut manager = BufferManager::new();
        let id = manager.new_buffer(None);

        assert_eq!(manager.active_buffer_id(), Some(id));
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_multiple_buffers() {
        let mut manager = BufferManager::new();
        let id1 = manager.new_buffer(None);
        let id2 = manager.new_buffer(None);
        let id3 = manager.new_buffer(None);

        assert_eq!(manager.len(), 3);
        assert_eq!(manager.active_buffer_id(), Some(id1)); // First remains active
    }

    #[test]
    fn test_switch_buffer() {
        let mut manager = BufferManager::new();
        let id1 = manager.new_buffer(None);
        let id2 = manager.new_buffer(None);

        assert!(manager.switch_to(id2).is_ok());
        assert_eq!(manager.active_buffer_id(), Some(id2));

        assert!(manager.switch_to(id1).is_ok());
        assert_eq!(manager.active_buffer_id(), Some(id1));
    }

    #[test]
    fn test_switch_to_invalid_buffer_fails() {
        let mut manager = BufferManager::new();
        let fake_id = uuid::Uuid::new_v4();

        assert!(manager.switch_to(fake_id).is_err());
    }

    #[test]
    fn test_close_buffer() {
        let mut manager = BufferManager::new();
        let id = manager.new_buffer(None);

        assert!(manager.close(id, false).is_ok());
        assert_eq!(manager.len(), 0);
        assert!(manager.active_buffer_id().is_none());
    }

    #[test]
    fn test_close_dirty_buffer_without_force_fails() {
        let mut manager = BufferManager::new();
        let id = manager.new_buffer(None);

        manager.get_mut(id).unwrap().insert(0, "test").unwrap();

        assert!(manager.close(id, false).is_err());
        assert_eq!(manager.len(), 1); // Buffer still open
    }

    #[test]
    fn test_close_dirty_buffer_with_force_succeeds() {
        let mut manager = BufferManager::new();
        let id = manager.new_buffer(None);

        manager.get_mut(id).unwrap().insert(0, "test").unwrap();

        assert!(manager.close(id, true).is_ok());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_close_switches_to_next_buffer() {
        let mut manager = BufferManager::new();
        let id1 = manager.new_buffer(None);
        let id2 = manager.new_buffer(None);

        manager.switch_to(id1).unwrap();
        manager.close(id1, false).unwrap();

        assert_eq!(manager.active_buffer_id(), Some(id2));
    }

    #[test]
    fn test_dirty_buffers_list() {
        let mut manager = BufferManager::new();
        let id1 = manager.new_buffer(None);
        let id2 = manager.new_buffer(None);
        let id3 = manager.new_buffer(None);

        manager.get_mut(id1).unwrap().insert(0, "a").unwrap();
        manager.get_mut(id3).unwrap().insert(0, "c").unwrap();

        let dirty = manager.dirty_buffers();
        assert_eq!(dirty.len(), 2);
        assert!(dirty.contains(&id1));
        assert!(dirty.contains(&id3));
        assert!(!dirty.contains(&id2));
    }

    #[test]
    fn test_close_all_without_dirty() {
        let mut manager = BufferManager::new();
        manager.new_buffer(None);
        manager.new_buffer(None);

        let result = manager.close_all(false).unwrap();
        assert_eq!(result.len(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_close_all_with_dirty_without_force() {
        let mut manager = BufferManager::new();
        let id1 = manager.new_buffer(None);
        manager.new_buffer(None);

        manager.get_mut(id1).unwrap().insert(0, "test").unwrap();

        let result = manager.close_all(false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], id1);
        assert_eq!(manager.len(), 2); // Nothing closed
    }

    #[test]
    fn test_close_all_with_force() {
        let mut manager = BufferManager::new();
        let id1 = manager.new_buffer(None);
        manager.new_buffer(None);

        manager.get_mut(id1).unwrap().insert(0, "test").unwrap();

        let result = manager.close_all(true).unwrap();
        assert_eq!(result.len(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_open_same_file_twice_returns_existing() {
        let mut manager = BufferManager::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "content").unwrap();

        let id1 = manager.open_file(temp_file.path()).unwrap();
        let id2 = manager.open_file(temp_file.path()).unwrap();

        assert_eq!(id1, id2);
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_buffer_ids_list() {
        let mut manager = BufferManager::new();
        let id1 = manager.new_buffer(None);
        let id2 = manager.new_buffer(None);

        let ids = manager.buffer_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_active_mut_returns_active_buffer() {
        let mut manager = BufferManager::new();
        let id = manager.new_buffer(None);

        {
            let buffer = manager.active_mut().unwrap();
            buffer.insert(0, "test").unwrap();
        }

        assert_eq!(manager.get(id).unwrap().to_string(), "test");
    }
}

#[cfg(test)]
mod buffer_concurrent_operations {
    use super::*;

    #[test]
    fn test_multiple_inserts_at_different_positions() {
        let mut buffer = Buffer::from_string("123456789".to_string(), None);

        // Insert at beginning
        buffer.insert(0, "A").unwrap();
        // Insert at end
        let len = buffer.len_bytes();
        buffer.insert(len, "Z").unwrap();
        // Insert in middle
        buffer.insert(5, "M").unwrap();

        assert!(buffer.to_string().starts_with("A"));
        assert!(buffer.to_string().ends_with("Z"));
        assert!(buffer.to_string().contains("M"));
    }

    #[test]
    fn test_interleaved_insert_delete() {
        let mut buffer = Buffer::from_string("ABCDEF".to_string(), None);

        buffer.insert(3, "X").unwrap();  // ABCXDEF
        buffer.delete(0..1).unwrap();     // BCXDEF
        buffer.insert(0, "Z").unwrap();   // ZBCXDEF

        assert_eq!(buffer.to_string(), "ZBCXDEF");
    }
}
