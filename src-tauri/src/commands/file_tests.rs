//! Backend File Command Tests
//!
//! Comprehensive testing for Tauri file system commands

#[cfg(test)]
mod file_command_tests {
    use super::super::*;
    use crate::state::AppState;
    use ait42_core::Editor;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use tauri::State;
    use tempfile::TempDir;

    /// Create a test AppState
    fn create_test_state() -> AppState {
        AppState::new(std::env::current_dir().unwrap()).unwrap()
    }

    /// Helper to create temporary test directory
    fn setup_test_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    #[tokio::test]
    async fn test_open_file_success() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").expect("Failed to write test file");

        let state = create_test_state();
        let state = State::new(state);

        let result = open_file(
            test_file.to_string_lossy().to_string(),
            state,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.content, "Hello, World!");
        assert_eq!(response.path, test_file.to_string_lossy().to_string());
        assert!(response.buffer_id.len() > 0);
    }

    #[tokio::test]
    async fn test_open_file_not_found() {
        let state = create_test_state();
        let state = State::new(state);

        let result = open_file(
            "/nonexistent/file.txt".to_string(),
            state,
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to open file"));
    }

    #[tokio::test]
    async fn test_open_file_with_language_detection() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("main.rs");
        fs::write(&test_file, "fn main() {}").expect("Failed to write test file");

        let state = create_test_state();
        let state = State::new(state);

        let result = open_file(
            test_file.to_string_lossy().to_string(),
            state,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.language, Some("rust".to_string()));
    }

    #[tokio::test]
    async fn test_save_file_success() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("save_test.txt");

        let result = save_file(
            test_file.to_string_lossy().to_string(),
            "Test content".to_string(),
        )
        .await;

        assert!(result.is_ok());
        assert!(test_file.exists());

        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "Test content");
    }

    #[tokio::test]
    async fn test_save_file_creates_parent_directories() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("nested/dir/test.txt");

        let result = save_file(
            test_file.to_string_lossy().to_string(),
            "Nested content".to_string(),
        )
        .await;

        assert!(result.is_ok());
        assert!(test_file.exists());

        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "Nested content");
    }

    #[tokio::test]
    async fn test_save_file_overwrites_existing() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("overwrite.txt");
        fs::write(&test_file, "Original").expect("Failed to write test file");

        let result = save_file(
            test_file.to_string_lossy().to_string(),
            "Updated".to_string(),
        )
        .await;

        assert!(result.is_ok());

        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "Updated");
    }

    #[tokio::test]
    async fn test_save_file_atomic_write() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("atomic.txt");

        save_file(
            test_file.to_string_lossy().to_string(),
            "Atomic write".to_string(),
        )
        .await
        .unwrap();

        // Ensure temp file is cleaned up
        let temp_file = test_file.with_extension(".tmp");
        assert!(!temp_file.exists());
    }

    #[tokio::test]
    async fn test_read_directory_success() {
        let temp_dir = setup_test_dir();

        // Create test files and directories
        fs::write(temp_dir.path().join("file1.txt"), "").unwrap();
        fs::write(temp_dir.path().join("file2.rs"), "").unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        let result = read_directory(temp_dir.path().to_string_lossy().to_string()).await;

        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 3);

        // Verify sorting: directories first, then files, alphabetically
        assert!(entries[0].is_directory || entries[0].name < entries[1].name);
    }

    #[tokio::test]
    async fn test_read_directory_not_a_directory() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("not_a_dir.txt");
        fs::write(&test_file, "").unwrap();

        let result = read_directory(test_file.to_string_lossy().to_string()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not a directory"));
    }

    #[tokio::test]
    async fn test_read_directory_shallow_recursion() {
        let temp_dir = setup_test_dir();

        // Create nested structure
        fs::create_dir(temp_dir.path().join("dir1")).unwrap();
        fs::create_dir(temp_dir.path().join("dir1/dir2")).unwrap();
        fs::write(temp_dir.path().join("dir1/file.txt"), "").unwrap();

        let result = read_directory(temp_dir.path().to_string_lossy().to_string()).await;

        assert!(result.is_ok());
        let entries = result.unwrap();

        // Find dir1
        let dir1 = entries.iter().find(|e| e.name == "dir1").unwrap();
        assert!(dir1.is_directory);
        assert!(dir1.children.is_some());

        let children = dir1.children.as_ref().unwrap();
        assert!(children.len() > 0);
    }

    #[tokio::test]
    async fn test_create_file_success() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("new_file.txt");

        let result = create_file(test_file.to_string_lossy().to_string()).await;

        assert!(result.is_ok());
        assert!(test_file.exists());

        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "");
    }

    #[tokio::test]
    async fn test_create_file_with_parent_dirs() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("nested/path/file.txt");

        let result = create_file(test_file.to_string_lossy().to_string()).await;

        assert!(result.is_ok());
        assert!(test_file.exists());
    }

    #[tokio::test]
    async fn test_create_directory_success() {
        let temp_dir = setup_test_dir();
        let test_dir = temp_dir.path().join("new_directory");

        let result = create_directory(test_dir.to_string_lossy().to_string()).await;

        assert!(result.is_ok());
        assert!(test_dir.exists());
        assert!(test_dir.is_dir());
    }

    #[tokio::test]
    async fn test_create_directory_nested() {
        let temp_dir = setup_test_dir();
        let test_dir = temp_dir.path().join("nested/deep/directory");

        let result = create_directory(test_dir.to_string_lossy().to_string()).await;

        assert!(result.is_ok());
        assert!(test_dir.exists());
        assert!(test_dir.is_dir());
    }

    #[tokio::test]
    async fn test_delete_file_success() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("delete_me.txt");
        fs::write(&test_file, "Delete me").unwrap();

        let result = delete_path(test_file.to_string_lossy().to_string()).await;

        assert!(result.is_ok());
        assert!(!test_file.exists());
    }

    #[tokio::test]
    async fn test_delete_directory_success() {
        let temp_dir = setup_test_dir();
        let test_dir = temp_dir.path().join("delete_dir");
        fs::create_dir(&test_dir).unwrap();
        fs::write(test_dir.join("file.txt"), "").unwrap();

        let result = delete_path(test_dir.to_string_lossy().to_string()).await;

        assert!(result.is_ok());
        assert!(!test_dir.exists());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_path() {
        let result = delete_path("/nonexistent/path".to_string()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to delete"));
    }

    #[tokio::test]
    async fn test_rename_file_success() {
        let temp_dir = setup_test_dir();
        let old_file = temp_dir.path().join("old_name.txt");
        let new_file = temp_dir.path().join("new_name.txt");

        fs::write(&old_file, "Content").unwrap();

        let result = rename_path(
            old_file.to_string_lossy().to_string(),
            new_file.to_string_lossy().to_string(),
        )
        .await;

        assert!(result.is_ok());
        assert!(!old_file.exists());
        assert!(new_file.exists());

        let content = fs::read_to_string(&new_file).unwrap();
        assert_eq!(content, "Content");
    }

    #[tokio::test]
    async fn test_rename_directory_success() {
        let temp_dir = setup_test_dir();
        let old_dir = temp_dir.path().join("old_dir");
        let new_dir = temp_dir.path().join("new_dir");

        fs::create_dir(&old_dir).unwrap();
        fs::write(old_dir.join("file.txt"), "Content").unwrap();

        let result = rename_path(
            old_dir.to_string_lossy().to_string(),
            new_dir.to_string_lossy().to_string(),
        )
        .await;

        assert!(result.is_ok());
        assert!(!old_dir.exists());
        assert!(new_dir.exists());
        assert!(new_dir.join("file.txt").exists());
    }

    #[tokio::test]
    async fn test_rename_creates_parent_dirs() {
        let temp_dir = setup_test_dir();
        let old_file = temp_dir.path().join("file.txt");
        let new_file = temp_dir.path().join("nested/path/file.txt");

        fs::write(&old_file, "Content").unwrap();

        let result = rename_path(
            old_file.to_string_lossy().to_string(),
            new_file.to_string_lossy().to_string(),
        )
        .await;

        assert!(result.is_ok());
        assert!(new_file.exists());
    }

    #[tokio::test]
    async fn test_file_node_serialization() {
        let node = FileNode {
            name: "test.txt".to_string(),
            path: "/path/to/test.txt".to_string(),
            is_directory: false,
            children: None,
        };

        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"name\":\"test.txt\""));
        assert!(json.contains("\"isDirectory\":false"));
    }

    #[tokio::test]
    async fn test_directory_sorting() {
        let temp_dir = setup_test_dir();

        // Create files and directories in mixed order
        fs::write(temp_dir.path().join("zzz.txt"), "").unwrap();
        fs::create_dir(temp_dir.path().join("aaa_dir")).unwrap();
        fs::write(temp_dir.path().join("mmm.txt"), "").unwrap();
        fs::create_dir(temp_dir.path().join("zzz_dir")).unwrap();

        let result = read_directory(temp_dir.path().to_string_lossy().to_string()).await;

        assert!(result.is_ok());
        let entries = result.unwrap();

        // First two should be directories (alphabetically sorted)
        assert!(entries[0].is_directory);
        assert_eq!(entries[0].name, "aaa_dir");
        assert!(entries[1].is_directory);
        assert_eq!(entries[1].name, "zzz_dir");

        // Next two should be files (alphabetically sorted)
        assert!(!entries[2].is_directory);
        assert_eq!(entries[2].name, "mmm.txt");
        assert!(!entries[3].is_directory);
        assert_eq!(entries[3].name, "zzz.txt");
    }

    #[tokio::test]
    async fn test_concurrent_file_operations() {
        let temp_dir = setup_test_dir();

        // Spawn multiple concurrent save operations
        let mut handles = vec![];

        for i in 0..10 {
            let path = temp_dir.path().join(format!("concurrent_{}.txt", i));
            let path_str = path.to_string_lossy().to_string();
            let content = format!("Content {}", i);

            let handle = tokio::spawn(async move {
                save_file(path_str, content).await
            });

            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }

        // Verify all files were created
        for i in 0..10 {
            let path = temp_dir.path().join(format!("concurrent_{}.txt", i));
            assert!(path.exists());
            let content = fs::read_to_string(&path).unwrap();
            assert_eq!(content, format!("Content {}", i));
        }
    }

    #[tokio::test]
    async fn test_large_file_handling() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("large_file.txt");

        // Create a large content (1MB)
        let large_content = "x".repeat(1_000_000);

        let result = save_file(
            test_file.to_string_lossy().to_string(),
            large_content.clone(),
        )
        .await;

        assert!(result.is_ok());

        let state = create_test_state();
        let state = State::new(state);

        let result = open_file(
            test_file.to_string_lossy().to_string(),
            state,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.content.len(), 1_000_000);
    }

    #[tokio::test]
    async fn test_unicode_filenames() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("ファイル.txt"); // Japanese characters

        let result = save_file(
            test_file.to_string_lossy().to_string(),
            "Unicode content".to_string(),
        )
        .await;

        assert!(result.is_ok());
        assert!(test_file.exists());

        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "Unicode content");
    }

    #[tokio::test]
    async fn test_special_characters_in_content() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("special.txt");

        let special_content = "Line 1\nLine 2\tTabbed\r\nWindows line\0Null byte";

        let result = save_file(
            test_file.to_string_lossy().to_string(),
            special_content.to_string(),
        )
        .await;

        assert!(result.is_ok());

        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, special_content);
    }
}
