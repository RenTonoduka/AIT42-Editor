//! File System Commands
//!
//! Tauri commands for file operations: open, save, read directory, create file, etc.

use ait42_core::{Buffer, BufferId};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::State;

use crate::state::AppState;

/// File node for directory tree display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub children: Option<Vec<FileNode>>,
}

/// Open a file and return its content
///
/// # Arguments
/// * `path` - File path to open
/// * `state` - Application state
///
/// # Returns
/// * `Ok(content)` - File content as string with buffer ID
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn open_file(
    path: String,
    state: State<'_, AppState>,
) -> Result<OpenFileResponse, String> {
    let path_buf = PathBuf::from(&path);

    // Open file as buffer
    let buffer = Buffer::from_file(&path_buf).map_err(|e| format!("Failed to open file: {}", e))?;

    let content = buffer.to_string();
    let buffer_id = buffer.id().to_string();
    let language = buffer.language().map(|s| s.to_string());

    // Add buffer to state
    let mut editor = state
        .editor
        .lock()
        .map_err(|e| format!("Failed to lock editor: {}", e))?;

    editor.buffers_mut().add_buffer(buffer);

    Ok(OpenFileResponse {
        buffer_id,
        content,
        path,
        language,
    })
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenFileResponse {
    pub buffer_id: String,
    pub content: String,
    pub path: String,
    pub language: Option<String>,
}

/// Save file content to disk
///
/// # Arguments
/// * `path` - File path to save to
/// * `content` - File content to write
///
/// # Returns
/// * `Ok(())` - File saved successfully
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn save_file(path: String, content: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);

    // Create parent directories if they don't exist
    if let Some(parent) = path_buf.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directories: {}", e))?;
    }

    // Write file atomically
    let temp_path = path_buf.with_extension(".tmp");
    std::fs::write(&temp_path, content)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    std::fs::rename(&temp_path, &path_buf)
        .map_err(|e| format!("Failed to rename temp file: {}", e))?;

    Ok(())
}

/// Read directory contents
///
/// # Arguments
/// * `path` - Directory path to read
///
/// # Returns
/// * `Ok(entries)` - List of file nodes
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn read_directory(path: String) -> Result<Vec<FileNode>, String> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    let mut entries = Vec::new();

    let read_dir = std::fs::read_dir(&path_buf)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let entry_path = entry.path();
        let name = entry
            .file_name()
            .to_string_lossy()
            .to_string();

        let is_directory = entry_path.is_dir();

        // Recursively read subdirectories (limit depth to avoid performance issues)
        let children = if is_directory {
            match read_directory_shallow(entry_path.to_str().unwrap().to_string()).await {
                Ok(children) => Some(children),
                Err(_) => None, // Skip directories we can't read
            }
        } else {
            None
        };

        entries.push(FileNode {
            name,
            path: entry_path.to_string_lossy().to_string(),
            is_directory,
            children,
        });
    }

    // Sort: directories first, then files, alphabetically
    entries.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}

/// Read directory contents (shallow, no recursion)
async fn read_directory_shallow(path: String) -> Result<Vec<FileNode>, String> {
    let path_buf = PathBuf::from(&path);

    let mut entries = Vec::new();

    let read_dir = std::fs::read_dir(&path_buf)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let entry_path = entry.path();
        let name = entry
            .file_name()
            .to_string_lossy()
            .to_string();

        entries.push(FileNode {
            name,
            path: entry_path.to_string_lossy().to_string(),
            is_directory: entry_path.is_dir(),
            children: None,
        });
    }

    // Sort: directories first, then files, alphabetically
    entries.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}

/// Create a new file
///
/// # Arguments
/// * `path` - File path to create
///
/// # Returns
/// * `Ok(())` - File created successfully
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn create_file(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);

    // Create parent directories if they don't exist
    if let Some(parent) = path_buf.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directories: {}", e))?;
    }

    // Create empty file
    std::fs::write(&path_buf, "")
        .map_err(|e| format!("Failed to create file: {}", e))?;

    Ok(())
}

/// Create a new directory
///
/// # Arguments
/// * `path` - Directory path to create
///
/// # Returns
/// * `Ok(())` - Directory created successfully
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn create_directory(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);

    std::fs::create_dir_all(&path_buf)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    Ok(())
}

/// Delete a file or directory
///
/// # Arguments
/// * `path` - Path to delete
///
/// # Returns
/// * `Ok(())` - Deleted successfully
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn delete_path(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);

    if path_buf.is_dir() {
        std::fs::remove_dir_all(&path_buf)
            .map_err(|e| format!("Failed to delete directory: {}", e))?;
    } else {
        std::fs::remove_file(&path_buf)
            .map_err(|e| format!("Failed to delete file: {}", e))?;
    }

    Ok(())
}

/// Rename/move a file or directory
///
/// # Arguments
/// * `old_path` - Current path
/// * `new_path` - New path
///
/// # Returns
/// * `Ok(())` - Renamed successfully
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn rename_path(old_path: String, new_path: String) -> Result<(), String> {
    let old_path_buf = PathBuf::from(&old_path);
    let new_path_buf = PathBuf::from(&new_path);

    // Create parent directories if they don't exist
    if let Some(parent) = new_path_buf.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directories: {}", e))?;
    }

    std::fs::rename(&old_path_buf, &new_path_buf)
        .map_err(|e| format!("Failed to rename: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_save_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("ait42_test.txt");
        let path = test_file.to_string_lossy().to_string();

        // Clean up if exists
        let _ = std::fs::remove_file(&test_file);

        // Create file
        assert!(create_file(path.clone()).await.is_ok());
        assert!(test_file.exists());

        // Save content
        assert!(save_file(path.clone(), "Hello, World!".to_string()).await.is_ok());

        // Verify content
        let content = std::fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "Hello, World!");

        // Clean up
        let _ = std::fs::remove_file(&test_file);
    }

    #[tokio::test]
    async fn test_directory_operations() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("ait42_test_dir");
        let path = test_dir.to_string_lossy().to_string();

        // Clean up if exists
        let _ = std::fs::remove_dir_all(&test_dir);

        // Create directory
        assert!(create_directory(path.clone()).await.is_ok());
        assert!(test_dir.exists());
        assert!(test_dir.is_dir());

        // Read directory
        let entries = read_directory(path.clone()).await.unwrap();
        assert_eq!(entries.len(), 0); // Empty directory

        // Delete directory
        assert!(delete_path(path.clone()).await.is_ok());
        assert!(!test_dir.exists());
    }
}
