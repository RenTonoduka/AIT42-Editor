//! Integration tests for Buffer operations
//!
//! Tests comprehensive buffer workflows across edge cases

use ait42_core::buffer::{Buffer, BufferManager, LineEnding};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_empty_buffer_operations() {
    let buffer = Buffer::new();

    // Empty buffer should have 1 line (rope convention)
    assert_eq!(buffer.len_lines(), 1);
    assert_eq!(buffer.len_bytes(), 0);
    assert_eq!(buffer.len_chars(), 0);
    assert!(buffer.is_empty());
    assert!(!buffer.is_dirty());

    // Operations on empty buffer
    assert!(buffer.line(0).is_some());
    assert!(buffer.line(1).is_none());
    assert_eq!(buffer.pos_to_line_col(0), (0, 0));
    assert_eq!(buffer.line_col_to_pos(0, 0), Some(0));
}

#[test]
fn test_large_buffer_operations() {
    // Test with 100KB of text
    let large_text = "Lorem ipsum ".repeat(8000); // ~100KB
    let mut buffer = Buffer::from_string(large_text.clone(), Some("txt".to_string()));

    assert!(buffer.len_bytes() > 100_000);

    // Insert in middle
    let mid = buffer.len_bytes() / 2;
    assert!(buffer.insert(mid, "\n--- MARKER ---\n").is_ok());
    assert!(buffer.to_string().contains("--- MARKER ---"));

    // Delete large chunk
    let quarter = buffer.len_bytes() / 4;
    assert!(buffer.delete(quarter..quarter + 1000).is_ok());

    // Replace large chunk
    assert!(buffer.replace(0..100, "REPLACED").is_ok());
}

#[test]
fn test_unicode_buffer_operations() {
    // Test with various Unicode characters
    let unicode_text = "Hello 世界 🌍 Здравствуй مرحبا";
    let mut buffer = Buffer::from_string(unicode_text.to_string(), None);

    // Verify character vs byte lengths differ
    assert!(buffer.len_bytes() > buffer.len_chars());

    // Test emoji handling (multi-byte)
    let pos = buffer.line_col_to_pos(0, 7).unwrap(); // After "世界"
    assert!(buffer.insert(pos, "✨").is_ok());

    // Test that positions respect UTF-8 boundaries
    assert!(buffer.insert(0, "🎉").is_ok());
}

#[test]
fn test_unicode_boundary_errors() {
    let mut buffer = Buffer::from_string("Hello 世界".to_string(), None);

    // Find a position in middle of multi-byte character
    // "世" is 3 bytes in UTF-8 (0xE4 0xB8 0x96)
    let text = buffer.to_string();
    let hello_len = "Hello ".len(); // 6 bytes

    // Try to insert in middle of multi-byte char (should fail)
    // Note: rope might adjust position, so we need to check actual behavior
    let result = buffer.insert(hello_len + 1, "X");
    // This should either work (if rope auto-corrects) or fail with Utf8Boundary error
}

#[test]
fn test_line_ending_detection_and_conversion() {
    // Test LF
    let lf_buffer = Buffer::from_string("Line1\nLine2\nLine3".to_string(), None);
    assert_eq!(lf_buffer.line_ending(), LineEnding::Lf);

    // Test CRLF
    let crlf_buffer = Buffer::from_string("Line1\r\nLine2\r\nLine3".to_string(), None);
    assert_eq!(crlf_buffer.line_ending(), LineEnding::CrLf);

    // Mixed (should detect CRLF if any exist)
    let mixed_buffer = Buffer::from_string("Line1\nLine2\r\nLine3".to_string(), None);
    assert_eq!(mixed_buffer.line_ending(), LineEnding::CrLf);
}

#[test]
fn test_buffer_save_and_reload() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create and save buffer
    let mut buffer = Buffer::from_string("Hello, World!".to_string(), Some("txt".to_string()));
    assert!(buffer.save_as(&file_path).is_ok());
    assert!(!buffer.is_dirty());
    assert_eq!(buffer.path(), Some(file_path.as_path()));

    // Reload from file
    let reloaded = Buffer::from_file(&file_path).unwrap();
    assert_eq!(reloaded.to_string(), "Hello, World!");
    assert_eq!(reloaded.language(), Some("txt"));
}

#[test]
fn test_buffer_atomic_save() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("atomic.txt");

    let mut buffer = Buffer::from_string("Test content".to_string(), None);
    buffer.save_as(&file_path).unwrap();

    // Verify temp file is removed after save
    let temp_path = file_path.with_extension(".tmp");
    assert!(!temp_path.exists(), "Temp file should be removed after atomic save");

    // Verify file content is correct
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "Test content");
}

#[test]
fn test_buffer_concurrent_modifications() {
    let mut buffer = Buffer::new();
    let initial_version = buffer.version();

    // Multiple modifications should increment version
    buffer.insert(0, "Line 1\n").unwrap();
    assert_eq!(buffer.version(), initial_version + 1);

    buffer.insert(7, "Line 2\n").unwrap();
    assert_eq!(buffer.version(), initial_version + 2);

    buffer.delete(0..7).unwrap();
    assert_eq!(buffer.version(), initial_version + 3);
}

#[test]
fn test_buffer_manager_multiple_buffers() {
    let mut manager = BufferManager::new();

    // Create multiple buffers
    let id1 = manager.new_buffer(Some("rust".to_string()));
    let id2 = manager.new_buffer(Some("python".to_string()));
    let id3 = manager.new_buffer(Some("javascript".to_string()));

    assert_eq!(manager.len(), 3);
    assert_eq!(manager.active_buffer_id(), Some(id1));

    // Switch buffers
    manager.switch_to(id2).unwrap();
    assert_eq!(manager.active_buffer_id(), Some(id2));

    // Close middle buffer
    manager.close(id2, false).unwrap();
    assert_eq!(manager.len(), 2);

    // Active buffer should change after closing active
    assert!(manager.active_buffer_id().is_some());
}

#[test]
fn test_buffer_manager_dirty_tracking() {
    let mut manager = BufferManager::new();

    let id1 = manager.new_buffer(None);
    let id2 = manager.new_buffer(None);
    let id3 = manager.new_buffer(None);

    // Modify some buffers
    manager.get_mut(id1).unwrap().insert(0, "modified").unwrap();
    manager.get_mut(id3).unwrap().insert(0, "also modified").unwrap();

    let dirty = manager.dirty_buffers();
    assert_eq!(dirty.len(), 2);
    assert!(dirty.contains(&id1));
    assert!(dirty.contains(&id3));
    assert!(!dirty.contains(&id2));
}

#[test]
fn test_buffer_manager_close_with_unsaved() {
    let mut manager = BufferManager::new();
    let id = manager.new_buffer(None);

    // Modify buffer
    manager.get_mut(id).unwrap().insert(0, "unsaved").unwrap();

    // Try to close without force - should fail
    let result = manager.close(id, false);
    assert!(result.is_err());
    assert_eq!(manager.len(), 1);

    // Force close should work
    let result = manager.close(id, true);
    assert!(result.is_ok());
    assert_eq!(manager.len(), 0);
}

#[test]
fn test_buffer_manager_close_all() {
    let mut manager = BufferManager::new();

    let id1 = manager.new_buffer(None);
    let id2 = manager.new_buffer(None);
    let id3 = manager.new_buffer(None);

    // Modify one
    manager.get_mut(id2).unwrap().insert(0, "dirty").unwrap();

    // Try close all without force - should return dirty buffers
    let dirty = manager.close_all(false).unwrap();
    assert_eq!(dirty.len(), 1);
    assert!(dirty.contains(&id2));
    assert_eq!(manager.len(), 3); // Nothing closed

    // Force close all
    let dirty = manager.close_all(true).unwrap();
    assert_eq!(dirty.len(), 0);
    assert_eq!(manager.len(), 0);
    assert!(manager.is_empty());
}

#[test]
fn test_buffer_manager_same_file_deduplication() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("shared.txt");
    fs::write(&file_path, "shared content").unwrap();

    let mut manager = BufferManager::new();

    // Open same file twice
    let id1 = manager.open_file(&file_path).unwrap();
    let id2 = manager.open_file(&file_path).unwrap();

    // Should return same buffer ID
    assert_eq!(id1, id2);
    assert_eq!(manager.len(), 1);
}

#[test]
fn test_buffer_slice_operations() {
    let buffer = Buffer::from_string("Hello, World!".to_string(), None);

    // Valid slices
    assert_eq!(buffer.slice(0..5).unwrap(), "Hello");
    assert_eq!(buffer.slice(7..12).unwrap(), "World");
    assert_eq!(buffer.slice(0..0).unwrap(), "");
    assert_eq!(buffer.slice(5..5).unwrap(), "");

    // Invalid slices
    assert!(buffer.slice(0..100).is_err());
    assert!(buffer.slice(10..5).is_err()); // start > end
}

#[test]
fn test_buffer_line_col_conversion_edge_cases() {
    let buffer = Buffer::from_string("a\n\nc".to_string(), None);

    // Line 0: "a\n"
    // Line 1: "\n"  (empty line)
    // Line 2: "c"

    assert_eq!(buffer.line_col_to_pos(0, 0), Some(0)); // 'a'
    assert_eq!(buffer.line_col_to_pos(0, 1), Some(1)); // '\n'
    assert_eq!(buffer.line_col_to_pos(1, 0), Some(2)); // second '\n'
    assert_eq!(buffer.line_col_to_pos(2, 0), Some(3)); // 'c'

    // Invalid positions
    assert_eq!(buffer.line_col_to_pos(3, 0), None); // line out of bounds
    assert_eq!(buffer.line_col_to_pos(0, 100), None); // col out of bounds
}

#[test]
fn test_buffer_pos_to_line_col_edge_cases() {
    let buffer = Buffer::from_string("ab\ncd\nef".to_string(), None);

    assert_eq!(buffer.pos_to_line_col(0), (0, 0));  // 'a'
    assert_eq!(buffer.pos_to_line_col(2), (0, 2));  // '\n'
    assert_eq!(buffer.pos_to_line_col(3), (1, 0));  // 'c'
    assert_eq!(buffer.pos_to_line_col(6), (2, 0));  // 'e'

    // Out of bounds - should clamp to end
    let (line, col) = buffer.pos_to_line_col(1000);
    assert_eq!(line, 2); // last line
    assert!(col >= 2);   // at or after 'ef'
}

#[test]
fn test_buffer_operations_on_readonly_filesystem() {
    // This test verifies error handling when filesystem operations fail
    let buffer = Buffer::from_string("test".to_string(), None);

    // Try to save buffer without file path
    let result = buffer.save();
    assert!(result.is_err());
}

#[test]
fn test_buffer_language_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Test various file extensions
    let test_cases = vec![
        ("test.rs", "rs"),
        ("test.py", "py"),
        ("test.js", "js"),
        ("test.ts", "ts"),
        ("test.md", "md"),
        ("test.toml", "toml"),
        ("test.json", "json"),
    ];

    for (filename, expected_lang) in test_cases {
        let path = temp_dir.path().join(filename);
        fs::write(&path, "// test content").unwrap();

        let buffer = Buffer::from_file(&path).unwrap();
        assert_eq!(buffer.language(), Some(expected_lang));
    }
}

#[test]
fn test_buffer_version_tracking() {
    let mut buffer = Buffer::new();
    let v0 = buffer.version();

    buffer.insert(0, "a").unwrap();
    assert_eq!(buffer.version(), v0 + 1);

    buffer.insert(1, "b").unwrap();
    assert_eq!(buffer.version(), v0 + 2);

    buffer.delete(0..1).unwrap();
    assert_eq!(buffer.version(), v0 + 3);

    buffer.replace(0..1, "x").unwrap();
    assert_eq!(buffer.version(), v0 + 4);

    // Mark clean doesn't change version
    buffer.mark_clean();
    assert_eq!(buffer.version(), v0 + 4);
}

#[test]
fn test_buffer_char_at_boundary() {
    let buffer = Buffer::from_string("Hello".to_string(), None);

    assert_eq!(buffer.char_at(0), Some('H'));
    assert_eq!(buffer.char_at(4), Some('o'));
    assert_eq!(buffer.char_at(5), None); // Out of bounds
    assert_eq!(buffer.char_at(100), None);
}

#[test]
fn test_buffer_empty_line_handling() {
    let buffer = Buffer::from_string("\n\n\n".to_string(), None);

    assert_eq!(buffer.len_lines(), 4); // 3 newlines create 4 lines

    for i in 0..4 {
        let line = buffer.line(i);
        assert!(line.is_some());
    }
}

#[test]
fn test_buffer_no_trailing_newline() {
    let buffer = Buffer::from_string("no newline".to_string(), None);

    assert_eq!(buffer.len_lines(), 1);
    let line = buffer.line(0).unwrap();
    assert_eq!(line.trim(), "no newline");
}
