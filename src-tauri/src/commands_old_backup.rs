use std::path::PathBuf;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileTreeNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileTreeNode>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: String,
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

/// Open and read a file
#[tauri::command]
pub async fn open_file(
    path: String,
    state: State<'_, Arc<AppState>>,
) -> Result<FileContent, String> {
    let path_buf = PathBuf::from(&path);

    // Read file content
    let content = tokio::fs::read_to_string(&path_buf)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Detect language from file extension
    let language = path_buf
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_string());

    Ok(FileContent {
        path,
        content,
        language,
    })
}

/// Save file content
#[tauri::command]
pub async fn save_file(
    path: String,
    content: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    tokio::fs::write(&path, content)
        .await
        .map_err(|e| format!("Failed to save file: {}", e))?;

    Ok(())
}

/// Create a new file
#[tauri::command]
pub async fn create_file(
    path: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);

    // Create parent directories if they don't exist
    if let Some(parent) = path_buf.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    // Create empty file
    tokio::fs::write(&path_buf, "")
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    Ok(())
}

/// Get file tree for a directory
#[tauri::command]
pub async fn get_file_tree(
    root_path: String,
    state: State<'_, Arc<AppState>>,
) -> Result<FileTreeNode, String> {
    let path = PathBuf::from(&root_path);

    fn build_tree(path: &PathBuf) -> Result<FileTreeNode, String> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| format!("Failed to get metadata: {}", e))?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let path_str = path.to_string_lossy().to_string();

        if metadata.is_dir() {
            let mut children = Vec::new();

            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if let Ok(child) = build_tree(&entry.path()) {
                        children.push(child);
                    }
                }
            }

            children.sort_by(|a, b| {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                }
            });

            Ok(FileTreeNode {
                name,
                path: path_str,
                is_dir: true,
                children: Some(children),
            })
        } else {
            Ok(FileTreeNode {
                name,
                path: path_str,
                is_dir: false,
                children: None,
            })
        }
    }

    build_tree(&path)
}

/// Search files by pattern
#[tauri::command]
pub async fn search_files(
    root_path: String,
    pattern: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<String>, String> {
    // Placeholder implementation
    // TODO: Integrate with ait42-fs search functionality
    Ok(vec![])
}

/// Format document using LSP
#[tauri::command]
pub async fn format_document(
    path: String,
    content: String,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    // Placeholder implementation
    // TODO: Integrate with ait42-lsp formatting
    Ok(content)
}

/// Get diagnostics for a file
#[tauri::command]
pub async fn get_diagnostics(
    path: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<Diagnostic>, String> {
    // Placeholder implementation
    // TODO: Integrate with ait42-lsp diagnostics
    Ok(vec![])
}

/// Get completions at cursor position
#[tauri::command]
pub async fn get_completions(
    path: String,
    line: u32,
    character: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<CompletionItem>, String> {
    // Placeholder implementation
    // TODO: Integrate with ait42-lsp completions
    Ok(vec![])
}

/// Go to definition
#[tauri::command]
pub async fn goto_definition(
    path: String,
    line: u32,
    character: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<Option<FileContent>, String> {
    // Placeholder implementation
    // TODO: Integrate with ait42-lsp goto_definition
    Ok(None)
}
