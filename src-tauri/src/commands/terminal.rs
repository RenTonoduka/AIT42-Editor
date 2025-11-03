//! Terminal Commands
//!
//! Tauri commands for terminal operations: execute commands, get output, etc.

#![cfg(feature = "terminal")]

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// Execute a terminal command
///
/// # Arguments
/// * `command` - Command string to execute
/// * `state` - Application state
///
/// # Returns
/// * `Ok(output)` - Command output (captured immediately after execution)
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn execute_command(
    command: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Execute command - need to drop lock before await
    {
        let mut terminal = state
            .terminal
            .lock()
            .map_err(|e| format!("Failed to lock terminal: {}", e))?;

        terminal
            .execute(&command)
            .await
            .map_err(|e| format!("Failed to execute command: {}", e))?;
    }

    // Get output in separate scope
    let output = {
        let terminal = state
            .terminal
            .lock()
            .map_err(|e| format!("Failed to lock terminal: {}", e))?;

        terminal
            .get_output()
            .iter()
            .cloned()
            .collect::<Vec<String>>()
            .join("\n")
    };

    Ok(output)
}

/// Get terminal output buffer
///
/// Returns all buffered terminal output lines.
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// * `Ok(lines)` - Array of output lines
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn get_terminal_output(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let terminal = state
        .terminal
        .lock()
        .map_err(|e| format!("Failed to lock terminal: {}", e))?;

    Ok(terminal.get_output().to_vec())
}

/// Get last N lines of terminal output
///
/// # Arguments
/// * `lines` - Number of lines to get
/// * `state` - Application state
///
/// # Returns
/// * `Ok(lines)` - Array of output lines
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn get_terminal_tail(
    lines: usize,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let terminal = state
        .terminal
        .lock()
        .map_err(|e| format!("Failed to lock terminal: {}", e))?;

    Ok(terminal.get_output_tail(lines).to_vec())
}

/// Clear terminal output buffer
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// * `Ok(())` - Terminal cleared successfully
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn clear_terminal(state: State<'_, AppState>) -> Result<(), String> {
    let mut terminal = state
        .terminal
        .lock()
        .map_err(|e| format!("Failed to lock terminal: {}", e))?;

    terminal.clear();

    Ok(())
}

/// Get current working directory
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// * `Ok(path)` - Current working directory path
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn get_current_directory(state: State<'_, AppState>) -> Result<String, String> {
    let terminal = state
        .terminal
        .lock()
        .map_err(|e| format!("Failed to lock terminal: {}", e))?;

    Ok(terminal.current_dir().to_string_lossy().to_string())
}

/// Set current working directory
///
/// # Arguments
/// * `path` - New working directory path
/// * `state` - Application state
///
/// # Returns
/// * `Ok(())` - Working directory changed successfully
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn set_current_directory(
    path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut terminal = state
        .terminal
        .lock()
        .map_err(|e| format!("Failed to lock terminal: {}", e))?;

    let path_buf = std::path::PathBuf::from(&path);

    if !path_buf.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    terminal.set_current_dir(path_buf);

    Ok(())
}

/// Get command history
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// * `Ok(history)` - Array of command strings (most recent first)
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn get_command_history(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let terminal = state
        .terminal
        .lock()
        .map_err(|e| format!("Failed to lock terminal: {}", e))?;

    // Reverse to get most recent first
    let history: Vec<String> = terminal
        .history()
        .iter()
        .rev()
        .cloned()
        .collect();

    Ok(history)
}

/// Get terminal info
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// * `Ok(info)` - Terminal information
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn get_terminal_info(state: State<'_, AppState>) -> Result<TerminalInfo, String> {
    let terminal = state
        .terminal
        .lock()
        .map_err(|e| format!("Failed to lock terminal: {}", e))?;

    Ok(TerminalInfo {
        current_dir: terminal.current_dir().to_string_lossy().to_string(),
        output_lines: terminal.get_output().len(),
        history_size: terminal.history().len(),
        timeout_seconds: terminal.timeout().as_secs(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalInfo {
    pub current_dir: String,
    pub output_lines: usize,
    pub history_size: usize,
    pub timeout_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ait42_core::{Editor, EditorConfig, EditorState};
    use ait42_tui::terminal_executor::TerminalExecutor;
    use std::sync::{Arc, Mutex};

    fn create_test_state() -> AppState {
        let editor = Editor::new(EditorConfig::default()).unwrap();
        let editor_state = EditorState::new();
        let terminal = TerminalExecutor::new(std::env::current_dir().unwrap());

        AppState {
            editor: Arc::new(Mutex::new(editor)),
            editor_state: Arc::new(Mutex::new(editor_state)),
            terminal: Arc::new(Mutex::new(terminal)),
        }
    }

    #[tokio::test]
    async fn test_execute_echo() {
        let state = create_test_state();

        let result = execute_command("echo test".to_string(), State::from(&state))
            .await
            .unwrap();

        assert!(result.contains("test"));
    }

    #[tokio::test]
    async fn test_terminal_history() {
        let state = create_test_state();

        // Execute some commands
        let _ = execute_command("echo hello".to_string(), State::from(&state)).await;
        let _ = execute_command("echo world".to_string(), State::from(&state)).await;

        // Get history
        let history = get_command_history(State::from(&state))
            .await
            .unwrap();

        assert_eq!(history.len(), 2);
        assert_eq!(history[0], "echo world"); // Most recent first
        assert_eq!(history[1], "echo hello");
    }

    #[tokio::test]
    async fn test_clear_terminal() {
        let state = create_test_state();

        // Execute command to generate output
        let _ = execute_command("echo test".to_string(), State::from(&state)).await;

        // Verify output exists
        let output = get_terminal_output(State::from(&state))
            .await
            .unwrap();
        assert!(!output.is_empty());

        // Clear terminal
        clear_terminal(State::from(&state)).await.unwrap();

        // Verify output cleared
        let output = get_terminal_output(State::from(&state))
            .await
            .unwrap();
        assert!(output.is_empty());
    }
}
