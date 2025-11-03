//! Editor Commands
//!
//! Tauri commands for text editing operations: insert, delete, undo, redo, etc.

use ait42_core::{BufferId, Command, DeleteCommand, InsertCommand, ReplaceCommand};
use serde::{Deserialize, Serialize};
use std::ops::Range;
use tauri::State;
use uuid::Uuid;

use crate::state::AppState;

/// Position in buffer (line, column)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

/// Text range (start, end)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

impl From<TextRange> for Range<usize> {
    fn from(r: TextRange) -> Self {
        r.start..r.end
    }
}

/// Insert text at byte position
///
/// # Arguments
/// * `buffer_id` - Buffer ID (UUID string)
/// * `position` - Byte offset position
/// * `text` - Text to insert
/// * `state` - Application state
///
/// # Returns
/// * `Ok(())` - Text inserted successfully
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn insert_text(
    buffer_id: String,
    position: usize,
    text: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let buffer_id = Uuid::parse_str(&buffer_id)
        .map_err(|e| format!("Invalid buffer ID: {}", e))?;

    let mut editor = state
        .editor
        .lock()
        .map_err(|e| format!("Failed to lock editor: {}", e))?;

    // Get buffer
    let buffer = editor
        .buffers_mut()
        .get_mut(buffer_id)
        .ok_or_else(|| format!("Buffer not found: {}", buffer_id))?;

    // Insert text
    buffer
        .insert(position, &text)
        .map_err(|e| format!("Failed to insert text: {}", e))?;

    Ok(())
}

/// Delete text in range
///
/// # Arguments
/// * `buffer_id` - Buffer ID (UUID string)
/// * `range` - Range to delete (start, end byte offsets)
/// * `state` - Application state
///
/// # Returns
/// * `Ok(())` - Text deleted successfully
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn delete_text(
    buffer_id: String,
    range: TextRange,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let buffer_id = Uuid::parse_str(&buffer_id)
        .map_err(|e| format!("Invalid buffer ID: {}", e))?;

    let mut editor = state
        .editor
        .lock()
        .map_err(|e| format!("Failed to lock editor: {}", e))?;

    // Get buffer
    let buffer = editor
        .buffers_mut()
        .get_mut(buffer_id)
        .ok_or_else(|| format!("Buffer not found: {}", buffer_id))?;

    // Delete text
    buffer
        .delete(range.into())
        .map_err(|e| format!("Failed to delete text: {}", e))?;

    Ok(())
}

/// Replace text in range
///
/// # Arguments
/// * `buffer_id` - Buffer ID (UUID string)
/// * `range` - Range to replace (start, end byte offsets)
/// * `text` - Replacement text
/// * `state` - Application state
///
/// # Returns
/// * `Ok(())` - Text replaced successfully
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn replace_text(
    buffer_id: String,
    range: TextRange,
    text: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let buffer_id = Uuid::parse_str(&buffer_id)
        .map_err(|e| format!("Invalid buffer ID: {}", e))?;

    let mut editor = state
        .editor
        .lock()
        .map_err(|e| format!("Failed to lock editor: {}", e))?;

    // Get buffer
    let buffer = editor
        .buffers_mut()
        .get_mut(buffer_id)
        .ok_or_else(|| format!("Buffer not found: {}", buffer_id))?;

    // Replace text
    buffer
        .replace(range.into(), &text)
        .map_err(|e| format!("Failed to replace text: {}", e))?;

    Ok(())
}

/// Undo last edit
///
/// # Arguments
/// * `buffer_id` - Buffer ID (UUID string)
/// * `state` - Application state
///
/// # Returns
/// * `Ok(())` - Undo successful
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn undo(
    buffer_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let buffer_id = Uuid::parse_str(&buffer_id)
        .map_err(|e| format!("Invalid buffer ID: {}", e))?;

    let mut editor_state = state
        .editor_state
        .lock()
        .map_err(|e| format!("Failed to lock editor state: {}", e))?;

    // Undo operation
    editor_state
        .undo()
        .map_err(|e| format!("Failed to undo: {}", e))?;

    Ok(())
}

/// Redo last undone edit
///
/// # Arguments
/// * `buffer_id` - Buffer ID (UUID string)
/// * `state` - Application state
///
/// # Returns
/// * `Ok(())` - Redo successful
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn redo(
    buffer_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let buffer_id = Uuid::parse_str(&buffer_id)
        .map_err(|e| format!("Invalid buffer ID: {}", e))?;

    let mut editor_state = state
        .editor_state
        .lock()
        .map_err(|e| format!("Failed to lock editor state: {}", e))?;

    // Redo operation
    editor_state
        .redo()
        .map_err(|e| format!("Failed to redo: {}", e))?;

    Ok(())
}

/// Get buffer content
///
/// # Arguments
/// * `buffer_id` - Buffer ID (UUID string)
/// * `state` - Application state
///
/// # Returns
/// * `Ok(content)` - Buffer content as string
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn get_buffer_content(
    buffer_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let buffer_id = Uuid::parse_str(&buffer_id)
        .map_err(|e| format!("Invalid buffer ID: {}", e))?;

    let editor = state
        .editor
        .lock()
        .map_err(|e| format!("Failed to lock editor: {}", e))?;

    // Get buffer
    let buffer = editor
        .buffers()
        .get(buffer_id)
        .ok_or_else(|| format!("Buffer not found: {}", buffer_id))?;

    Ok(buffer.to_string())
}

/// Get buffer info
///
/// # Arguments
/// * `buffer_id` - Buffer ID (UUID string)
/// * `state` - Application state
///
/// # Returns
/// * `Ok(info)` - Buffer information
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn get_buffer_info(
    buffer_id: String,
    state: State<'_, AppState>,
) -> Result<BufferInfo, String> {
    let buffer_id = Uuid::parse_str(&buffer_id)
        .map_err(|e| format!("Invalid buffer ID: {}", e))?;

    let editor = state
        .editor
        .lock()
        .map_err(|e| format!("Failed to lock editor: {}", e))?;

    // Get buffer
    let buffer = editor
        .buffers()
        .get(buffer_id)
        .ok_or_else(|| format!("Buffer not found: {}", buffer_id))?;

    Ok(BufferInfo {
        buffer_id: buffer_id.to_string(),
        path: buffer.path().map(|p| p.to_string_lossy().to_string()),
        language: buffer.language().map(|s| s.to_string()),
        is_dirty: buffer.is_dirty(),
        line_count: buffer.len_lines(),
        char_count: buffer.len_chars(),
        byte_count: buffer.len_bytes(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferInfo {
    pub buffer_id: String,
    pub path: Option<String>,
    pub language: Option<String>,
    pub is_dirty: bool,
    pub line_count: usize,
    pub char_count: usize,
    pub byte_count: usize,
}

/// Close buffer
///
/// # Arguments
/// * `buffer_id` - Buffer ID (UUID string)
/// * `force` - Force close even if dirty
/// * `state` - Application state
///
/// # Returns
/// * `Ok(())` - Buffer closed successfully
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn close_buffer(
    buffer_id: String,
    force: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let buffer_id = Uuid::parse_str(&buffer_id)
        .map_err(|e| format!("Invalid buffer ID: {}", e))?;

    let mut editor = state
        .editor
        .lock()
        .map_err(|e| format!("Failed to lock editor: {}", e))?;

    // Close buffer
    editor
        .buffers_mut()
        .close(buffer_id, force)
        .map_err(|e| format!("Failed to close buffer: {}", e))?;

    Ok(())
}

/// List all open buffers
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// * `Ok(buffers)` - List of buffer IDs
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn list_buffers(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let editor = state
        .editor
        .lock()
        .map_err(|e| format!("Failed to lock editor: {}", e))?;

    let buffer_ids: Vec<String> = editor
        .buffers()
        .buffer_ids()
        .into_iter()
        .map(|id| id.to_string())
        .collect();

    Ok(buffer_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ait42_core::{Buffer, Editor, EditorConfig, EditorState};
    use std::sync::{Arc, Mutex};

    fn create_test_state() -> AppState {
        let editor = Editor::new(EditorConfig::default()).unwrap();
        let editor_state = EditorState::new();
        let terminal = ait42_tui::terminal_executor::TerminalExecutor::new(
            std::env::current_dir().unwrap()
        );

        AppState {
            editor: Arc::new(Mutex::new(editor)),
            editor_state: Arc::new(Mutex::new(editor_state)),
            terminal: Arc::new(Mutex::new(terminal)),
        }
    }

    #[tokio::test]
    async fn test_insert_and_delete_text() {
        let state = create_test_state();

        // Create buffer
        let buffer = Buffer::from_string("Hello World".to_string(), None);
        let buffer_id = buffer.id().to_string();

        {
            let mut editor = state.editor.lock().unwrap();
            editor.buffers_mut().add_buffer(buffer);
        }

        // Insert text
        let result = insert_text(
            buffer_id.clone(),
            5,
            ", Rust".to_string(),
            State::from(&state),
        )
        .await;
        assert!(result.is_ok());

        // Verify content
        let content = get_buffer_content(buffer_id.clone(), State::from(&state))
            .await
            .unwrap();
        assert_eq!(content, "Hello, Rust World");

        // Delete text
        let result = delete_text(
            buffer_id.clone(),
            TextRange { start: 5, end: 11 },
            State::from(&state),
        )
        .await;
        assert!(result.is_ok());

        // Verify content
        let content = get_buffer_content(buffer_id.clone(), State::from(&state))
            .await
            .unwrap();
        assert_eq!(content, "Hello World");
    }
}
