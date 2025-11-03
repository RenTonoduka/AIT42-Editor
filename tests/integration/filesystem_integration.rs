//! File System Integration Tests
//!
//! Tests for file operations, directory handling, and file watching.

use ait42_core::buffer::{Buffer, BufferManager};
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

#[cfg(test)]
mod file_operations {
    use super::*;

    #[test]
    fn test_create_and_save_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new_file.txt");

        let mut buffer = Buffer::from_string("Hello, World!".to_string(), None);

        assert!(buffer.save_as(&file_path).is_ok());
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_open_and_modify_existing_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "Original content").unwrap();

        let mut buffer = Buffer::from_file(temp_file.path()).unwrap();
        assert_eq!(buffer.to_string(), "Original content");

        buffer.insert(buffer.len_bytes(), " - Modified").unwrap();

        buffer.save().unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(content, "Original content - Modified");
    }

    #[test]
    fn test_save_as_different_location() {
        let mut temp_file1 = NamedTempFile::new().unwrap();
        write!(temp_file1, "Original").unwrap();

        let mut buffer = Buffer::from_file(temp_file1.path()).unwrap();

        let temp_dir = TempDir::new().unwrap();
        let new_path = temp_dir.path().join("new_location.txt");

        buffer.save_as(&new_path).unwrap();

        assert!(new_path.exists());
        let content = fs::read_to_string(&new_path).unwrap();
        assert_eq!(content, "Original");
    }

    #[test]
    fn test_atomic_save_on_crash() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("atomic_test.txt");

        // Create original file
        fs::write(&file_path, "Original content").unwrap();

        let mut buffer = Buffer::from_file(&file_path).unwrap();
        buffer.insert(0, "Modified ").unwrap();

        // Save should be atomic (temp file + rename)
        buffer.save().unwrap();

        // Verify no .tmp files left behind
        let tmp_files: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "tmp")
                    .unwrap_or(false)
            })
            .collect();

        assert_eq!(tmp_files.len(), 0);
    }

    #[test]
    fn test_save_preserves_file_permissions() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("perms_test.txt");

            fs::write(&file_path, "Content").unwrap();

            // Set specific permissions
            let mut perms = fs::metadata(&file_path).unwrap().permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&file_path, perms).unwrap();

            let mut buffer = Buffer::from_file(&file_path).unwrap();
            buffer.insert(0, "Modified ").unwrap();
            buffer.save().unwrap();

            // Check permissions are preserved
            let new_perms = fs::metadata(&file_path).unwrap().permissions();
            assert_eq!(new_perms.mode() & 0o777, 0o644);
        }
    }

    #[test]
    fn test_handle_concurrent_file_access() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "Original").unwrap();

        // Buffer 1 reads file
        let mut buffer1 = Buffer::from_file(temp_file.path()).unwrap();

        // Buffer 2 reads same file
        let mut buffer2 = Buffer::from_file(temp_file.path()).unwrap();

        // Both modify independently
        buffer1.insert(0, "B1: ").unwrap();
        buffer2.insert(0, "B2: ").unwrap();

        // Last save wins
        buffer1.save().unwrap();
        buffer2.save().unwrap();

        let final_content = fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(final_content, "B2: Original");
    }

    #[test]
    fn test_large_file_handling() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large_file.txt");

        // Create a 1MB file
        let large_content = "x".repeat(1_000_000);
        fs::write(&file_path, &large_content).unwrap();

        let buffer = Buffer::from_file(&file_path).unwrap();
        assert_eq!(buffer.len_bytes(), 1_000_000);
        assert_eq!(buffer.to_string(), large_content);
    }

    #[test]
    fn test_empty_file_handling() {
        let mut temp_file = NamedTempFile::new().unwrap();
        // Don't write anything

        let buffer = Buffer::from_file(temp_file.path()).unwrap();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len_lines(), 1);
    }

    #[test]
    fn test_binary_file_as_text() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("binary.dat");

        // Write binary data
        fs::write(&file_path, &[0xFF, 0xFE, 0xFD]).unwrap();

        // Should fail to load as text
        let result = Buffer::from_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_symlink_handling() {
        #[cfg(unix)]
        {
            use std::os::unix::fs as unix_fs;

            let temp_dir = TempDir::new().unwrap();
            let original = temp_dir.path().join("original.txt");
            let symlink = temp_dir.path().join("link.txt");

            fs::write(&original, "Content").unwrap();
            unix_fs::symlink(&original, &symlink).unwrap();

            let buffer = Buffer::from_file(&symlink).unwrap();
            assert_eq!(buffer.to_string(), "Content");
        }
    }
}

#[cfg(test)]
mod buffer_manager_file_operations {
    use super::*;

    #[test]
    fn test_open_multiple_files() {
        let temp_dir = TempDir::new().unwrap();

        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        let file3 = temp_dir.path().join("file3.txt");

        fs::write(&file1, "File 1").unwrap();
        fs::write(&file2, "File 2").unwrap();
        fs::write(&file3, "File 3").unwrap();

        let mut manager = BufferManager::new();

        let id1 = manager.open_file(&file1).unwrap();
        let id2 = manager.open_file(&file2).unwrap();
        let id3 = manager.open_file(&file3).unwrap();

        assert_eq!(manager.len(), 3);
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
    }

    #[test]
    fn test_open_same_file_twice() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "Content").unwrap();

        let mut manager = BufferManager::new();

        let id1 = manager.open_file(temp_file.path()).unwrap();
        let id2 = manager.open_file(temp_file.path()).unwrap();

        // Should return same buffer ID
        assert_eq!(id1, id2);
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_save_all_dirty_buffers() {
        let temp_dir = TempDir::new().unwrap();

        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        fs::write(&file1, "F1").unwrap();
        fs::write(&file2, "F2").unwrap();

        let mut manager = BufferManager::new();

        let id1 = manager.open_file(&file1).unwrap();
        let id2 = manager.open_file(&file2).unwrap();

        // Modify both
        manager.get_mut(id1).unwrap().insert(0, "Modified ").unwrap();
        manager.get_mut(id2).unwrap().insert(0, "Changed ").unwrap();

        let dirty = manager.dirty_buffers();
        assert_eq!(dirty.len(), 2);

        // Save all
        for id in dirty {
            manager.save(id).unwrap();
        }

        // Verify files updated
        assert_eq!(fs::read_to_string(&file1).unwrap(), "Modified F1");
        assert_eq!(fs::read_to_string(&file2).unwrap(), "Changed F2");
    }

    #[test]
    fn test_close_all_with_unsaved_changes() {
        let temp_dir = TempDir::new().unwrap();

        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        fs::write(&file1, "F1").unwrap();
        fs::write(&file2, "F2").unwrap();

        let mut manager = BufferManager::new();

        let id1 = manager.open_file(&file1).unwrap();
        manager.open_file(&file2).unwrap();

        // Modify one
        manager.get_mut(id1).unwrap().insert(0, "Modified ").unwrap();

        // Try to close all without force
        let dirty = manager.close_all(false).unwrap();
        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0], id1);

        // Buffers should still be open
        assert_eq!(manager.len(), 2);
    }
}

#[cfg(test)]
mod line_ending_integration {
    use super::*;
    use ait42_core::buffer::LineEnding;

    #[test]
    fn test_preserve_unix_line_endings() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("unix.txt");

        fs::write(&file_path, "Line 1\nLine 2\nLine 3").unwrap();

        let mut buffer = Buffer::from_file(&file_path).unwrap();
        assert_eq!(buffer.line_ending(), LineEnding::Lf);

        buffer.insert(6, " Modified").unwrap();
        buffer.save().unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("\n"));
        assert!(!content.contains("\r\n"));
    }

    #[test]
    fn test_preserve_windows_line_endings() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("windows.txt");

        fs::write(&file_path, "Line 1\r\nLine 2\r\nLine 3").unwrap();

        let buffer = Buffer::from_file(&file_path).unwrap();
        assert_eq!(buffer.line_ending(), LineEnding::CrLf);

        // Line endings should be preserved in buffer
        let content = buffer.to_string();
        assert!(content.contains("\r\n"));
    }

    #[test]
    fn test_convert_line_endings() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("convert.txt");

        fs::write(&file_path, "Line 1\r\nLine 2\r\nLine 3").unwrap();

        let mut buffer = Buffer::from_file(&file_path).unwrap();

        // Read as string and convert
        let content = buffer.to_string();
        let converted = content.replace("\r\n", "\n");

        // Create new buffer with converted line endings
        let new_buffer = Buffer::from_string(converted, None);
        assert_eq!(new_buffer.line_ending(), LineEnding::Lf);
    }
}

#[cfg(test)]
mod file_metadata_tests {
    use super::*;

    #[test]
    fn test_language_detection_from_extension() {
        let temp_dir = TempDir::new().unwrap();

        let test_cases = vec![
            ("test.rs", Some("rs")),
            ("test.py", Some("py")),
            ("test.js", Some("js")),
            ("test.txt", Some("txt")),
            ("test", None),
        ];

        for (filename, expected_lang) in test_cases {
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, "content").unwrap();

            let buffer = Buffer::from_file(&file_path).unwrap();
            assert_eq!(buffer.language(), expected_lang);
        }
    }

    #[test]
    fn test_file_path_stored() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "Content").unwrap();

        let buffer = Buffer::from_file(temp_file.path()).unwrap();
        assert_eq!(buffer.path(), Some(temp_file.path()));
    }

    #[test]
    fn test_buffer_id_uniqueness() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Content").unwrap();

        let buffer1 = Buffer::from_file(&file_path).unwrap();
        let buffer2 = Buffer::from_file(&file_path).unwrap();

        // Different buffer instances should have different IDs
        assert_ne!(buffer1.id(), buffer2.id());
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    fn test_open_many_files() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = BufferManager::new();

        // Create and open 100 files
        for i in 0..100 {
            let file_path = temp_dir.path().join(format!("file_{}.txt", i));
            fs::write(&file_path, format!("Content {}", i)).unwrap();

            manager.open_file(&file_path).unwrap();
        }

        assert_eq!(manager.len(), 100);
    }

    #[test]
    fn test_rapid_save_operations() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "Initial").unwrap();

        let mut buffer = Buffer::from_file(temp_file.path()).unwrap();

        // Perform 100 modifications and saves
        for i in 0..100 {
            buffer.insert(buffer.len_bytes(), &format!(" {}", i)).unwrap();
            buffer.save().unwrap();
        }

        let final_content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(final_content.starts_with("Initial"));
        assert!(final_content.contains(" 99"));
    }
}
