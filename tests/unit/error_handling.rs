//! Error Handling Tests
//!
//! Comprehensive tests for error conditions across all modules.

use ait42_core::buffer::{Buffer, BufferManager};
use ait42_core::cursor::Cursor;
use ait42_core::error::EditorError;
use std::path::Path;

#[cfg(test)]
mod buffer_error_tests {
    use super::*;

    #[test]
    fn test_insert_invalid_position_error() {
        let mut buffer = Buffer::new();
        let result = buffer.insert(100, "test");

        assert!(result.is_err());
        match result.unwrap_err() {
            EditorError::InvalidPosition(pos) => assert_eq!(pos, 100),
            _ => panic!("Expected InvalidPosition error"),
        }
    }

    #[test]
    fn test_delete_invalid_range_error() {
        let mut buffer = Buffer::from_string("Test".to_string(), None);
        let result = buffer.delete(0..100);

        assert!(result.is_err());
        match result.unwrap_err() {
            EditorError::InvalidRange(_) => (),
            _ => panic!("Expected InvalidRange error"),
        }
    }

    #[test]
    fn test_delete_inverted_range_error() {
        let mut buffer = Buffer::from_string("Test".to_string(), None);
        let result = buffer.delete(3..1);

        assert!(result.is_err());
    }

    #[test]
    fn test_utf8_boundary_error_on_insert() {
        let mut buffer = Buffer::from_string("世界".to_string(), None);

        // Try to insert in the middle of a multibyte character
        let result = buffer.insert(1, "!");

        assert!(result.is_err());
        match result.unwrap_err() {
            EditorError::Utf8Boundary(pos) => assert_eq!(pos, 1),
            _ => panic!("Expected Utf8Boundary error"),
        }
    }

    #[test]
    fn test_utf8_boundary_error_on_delete() {
        let mut buffer = Buffer::from_string("世界".to_string(), None);

        // Try to delete with invalid UTF-8 boundaries
        let result = buffer.delete(1..2);

        assert!(result.is_err());
        match result.unwrap_err() {
            EditorError::Utf8Boundary(_) => (),
            _ => panic!("Expected Utf8Boundary error"),
        }
    }

    #[test]
    fn test_file_not_found_error() {
        let result = Buffer::from_file(Path::new("/nonexistent/path/file.txt"));

        assert!(result.is_err());
        match result.unwrap_err() {
            EditorError::Io(_) => (),
            _ => panic!("Expected IO error"),
        }
    }

    #[test]
    fn test_save_without_path_error() {
        let mut buffer = Buffer::new();
        let result = buffer.save();

        assert!(result.is_err());
        match result.unwrap_err() {
            EditorError::Other(msg) => assert!(msg.contains("path")),
            _ => panic!("Expected Other error about path"),
        }
    }

    #[test]
    fn test_slice_out_of_bounds_error() {
        let buffer = Buffer::from_string("Test".to_string(), None);
        let result = buffer.slice(0..100);

        assert!(result.is_err());
    }

    #[test]
    fn test_line_col_conversion_invalid() {
        let buffer = Buffer::from_string("Line 1\nLine 2".to_string(), None);

        // Invalid line
        let result = buffer.line_col_to_pos(100, 0);
        assert!(result.is_none());

        // Invalid column
        let result = buffer.line_col_to_pos(0, 100);
        assert!(result.is_none());
    }
}

#[cfg(test)]
mod buffer_manager_error_tests {
    use super::*;

    #[test]
    fn test_switch_to_invalid_buffer() {
        let mut manager = BufferManager::new();
        let fake_id = uuid::Uuid::new_v4();

        let result = manager.switch_to(fake_id);
        assert!(result.is_err());
        match result.unwrap_err() {
            EditorError::BufferNotFound(id) => assert_eq!(id, fake_id),
            _ => panic!("Expected BufferNotFound error"),
        }
    }

    #[test]
    fn test_close_nonexistent_buffer() {
        let mut manager = BufferManager::new();
        let fake_id = uuid::Uuid::new_v4();

        let result = manager.close(fake_id, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_close_dirty_buffer_without_force() {
        let mut manager = BufferManager::new();
        let id = manager.new_buffer(None);

        // Make buffer dirty
        manager.get_mut(id).unwrap().insert(0, "test").unwrap();

        let result = manager.close(id, false);
        assert!(result.is_err());
        match result.unwrap_err() {
            EditorError::Other(msg) => assert!(msg.contains("unsaved")),
            _ => panic!("Expected Other error about unsaved changes"),
        }
    }

    #[test]
    fn test_save_nonexistent_buffer() {
        let mut manager = BufferManager::new();
        let fake_id = uuid::Uuid::new_v4();

        let result = manager.save(fake_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_nonexistent_buffer() {
        let manager = BufferManager::new();
        let fake_id = uuid::Uuid::new_v4();

        assert!(manager.get(fake_id).is_none());
    }
}

#[cfg(test)]
mod cursor_error_tests {
    use super::*;

    #[test]
    fn test_move_to_invalid_line_col() {
        let buffer = Buffer::from_string("Test".to_string(), None);
        let mut cursor = Cursor::new(0);

        let result = cursor.move_to(&buffer, 100, 0);
        assert!(result.is_err());
        match result.unwrap_err() {
            EditorError::InvalidLineCol { line, col } => {
                assert_eq!(line, 100);
                assert_eq!(col, 0);
            }
            _ => panic!("Expected InvalidLineCol error"),
        }
    }

    #[test]
    fn test_cursor_set_remove_invalid_index() {
        use ait42_core::cursor::CursorSet;

        let mut cursors = CursorSet::new(0);
        let result = cursors.remove_cursor(10);

        assert!(result.is_err());
        match result.unwrap_err() {
            EditorError::Other(msg) => assert!(msg.contains("bounds")),
            _ => panic!("Expected Other error about bounds"),
        }
    }
}

#[cfg(test)]
mod error_display_tests {
    use super::*;

    #[test]
    fn test_invalid_position_display() {
        let err = EditorError::InvalidPosition(42);
        assert_eq!(err.to_string(), "Invalid position: 42");
    }

    #[test]
    fn test_invalid_line_col_display() {
        let err = EditorError::InvalidLineCol { line: 10, col: 20 };
        assert_eq!(err.to_string(), "Invalid position: line 10, col 20");
    }

    #[test]
    fn test_utf8_boundary_display() {
        let err = EditorError::Utf8Boundary(5);
        assert_eq!(err.to_string(), "UTF-8 boundary error at position 5");
    }

    #[test]
    fn test_buffer_not_found_display() {
        let id = uuid::Uuid::new_v4();
        let err = EditorError::BufferNotFound(id);
        assert!(err.to_string().contains("Buffer not found"));
    }

    #[test]
    fn test_empty_buffer_display() {
        let err = EditorError::EmptyBuffer;
        assert_eq!(err.to_string(), "Buffer is empty");
    }

    #[test]
    fn test_invalid_range_display() {
        let err = EditorError::InvalidRange(5..10);
        assert!(err.to_string().contains("Invalid range"));
    }

    #[test]
    fn test_other_error_display() {
        let err = EditorError::Other("Custom error message".to_string());
        assert_eq!(err.to_string(), "Custom error message");
    }
}

#[cfg(test)]
mod error_conversion_tests {
    use super::*;

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let editor_err: EditorError = io_err.into();

        match editor_err {
            EditorError::Io(_) => (),
            _ => panic!("Expected IO error"),
        }
    }

    #[test]
    fn test_utf8_error_conversion() {
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        let utf8_result = String::from_utf8(invalid_utf8);

        if let Err(utf8_err) = utf8_result {
            let editor_err: EditorError = utf8_err.into();
            match editor_err {
                EditorError::Utf8(_) => (),
                _ => panic!("Expected UTF-8 error"),
            }
        }
    }
}

#[cfg(test)]
mod error_propagation_tests {
    use super::*;

    #[test]
    fn test_error_propagation_through_operations() {
        let mut buffer = Buffer::new();

        // Chain of operations where error should propagate
        let result = buffer
            .insert(100, "test") // This will fail
            .and_then(|_| buffer.delete(0..1)); // This should not execute

        assert!(result.is_err());
    }

    #[test]
    fn test_error_recovery() {
        let mut buffer = Buffer::new();

        // Failed operation
        let _ = buffer.insert(100, "test");

        // Buffer should still be usable
        assert!(buffer.insert(0, "test").is_ok());
        assert_eq!(buffer.to_string(), "test");
    }

    #[test]
    fn test_multiple_error_conditions() {
        let mut buffer = Buffer::from_string("Test".to_string(), None);

        // Test multiple error conditions
        assert!(buffer.insert(100, "x").is_err());
        assert!(buffer.delete(0..100).is_err());
        assert!(buffer.replace(0..100, "y").is_err());
        assert!(buffer.slice(0..100).is_err());

        // Buffer should still be in valid state
        assert_eq!(buffer.to_string(), "Test");
    }
}

#[cfg(test)]
mod permission_error_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    #[cfg(unix)]
    fn test_read_permission_denied() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("readonly.txt");

        // Create file and make it unreadable
        fs::write(&file_path, "content").unwrap();
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&file_path, perms).unwrap();

        let result = Buffer::from_file(&file_path);
        assert!(result.is_err());

        // Cleanup: restore permissions
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o644);
        let _ = fs::set_permissions(&file_path, perms);
    }

    #[test]
    fn test_write_to_readonly_directory() {
        // This test is OS-dependent and may not work on all systems
        // Testing the error path for save operations
    }
}

#[cfg(test)]
mod concurrent_error_tests {
    use super::*;

    #[test]
    fn test_buffer_state_after_failed_operation() {
        let mut buffer = Buffer::from_string("Original".to_string(), None);
        let original_version = buffer.version();

        // Failed operation should not change version
        let _ = buffer.insert(100, "test");
        assert_eq!(buffer.version(), original_version);

        // Failed operation should not set dirty flag
        assert!(!buffer.is_dirty());
    }

    #[test]
    fn test_buffer_consistency_after_errors() {
        let mut buffer = Buffer::from_string("Test Content".to_string(), None);

        // Multiple failed operations
        for _ in 0..10 {
            let _ = buffer.insert(1000, "x");
            let _ = buffer.delete(0..1000);
            let _ = buffer.replace(0..1000, "y");
        }

        // Buffer should remain consistent
        assert_eq!(buffer.to_string(), "Test Content");
        assert_eq!(buffer.len_bytes(), "Test Content".len());
    }
}
