/**
 * Session History Management - Persistent storage for worktree sessions
 * Inspired by Vibe Kanban's SQLite-based persistence
 */

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeInstance {
    pub instance_id: u32,
    pub worktree_path: String,
    pub branch: String,
    pub agent_name: String,
    pub status: String,
    pub tmux_session_id: String,
    pub output: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub files_changed: Option<u32>,
    pub lines_added: Option<u32>,
    pub lines_deleted: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub role: String, // user | assistant | system
    pub content: String,
    pub timestamp: String,
    pub instance_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeSession {
    pub id: String,
    pub r#type: String, // competition | ensemble | debate
    pub task: String,
    pub status: String, // running | completed | failed | paused
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub instances: Vec<WorktreeInstance>,
    pub chat_history: Vec<ChatMessage>,
    pub model: Option<String>,
    pub timeout_seconds: Option<u32>,
    pub preserve_worktrees: Option<bool>,
    pub winner_id: Option<u32>,
    pub total_duration: Option<u64>,
    pub total_files_changed: Option<u32>,
    pub total_lines_added: Option<u32>,
    pub total_lines_deleted: Option<u32>,
}

/// Get path to sessions storage file
fn get_sessions_file_path(state: &AppState) -> PathBuf {
    let working_dir = state.working_dir.blocking_lock();
    let project_root = working_dir
        .parent()
        .unwrap_or(&working_dir)
        .to_path_buf();
    project_root.join(".ait42").join("sessions.json")
}

/// Ensure .ait42 directory exists
fn ensure_storage_dir(state: &AppState) -> Result<(), String> {
    let working_dir = state.working_dir.blocking_lock();
    let project_root = working_dir
        .parent()
        .unwrap_or(&working_dir)
        .to_path_buf();
    let ait42_dir = project_root.join(".ait42");

    if !ait42_dir.exists() {
        fs::create_dir_all(&ait42_dir).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Load all sessions from disk
fn load_sessions(state: &AppState) -> Result<Vec<WorktreeSession>, String> {
    let sessions_file = get_sessions_file_path(state);

    if !sessions_file.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&sessions_file).map_err(|e| e.to_string())?;

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse sessions: {}", e))
}

/// Save all sessions to disk
fn save_sessions(state: &AppState, sessions: &[WorktreeSession]) -> Result<(), String> {
    ensure_storage_dir(state)?;

    let sessions_file = get_sessions_file_path(state);
    let content = serde_json::to_string_pretty(sessions).map_err(|e| e.to_string())?;

    fs::write(&sessions_file, content).map_err(|e| e.to_string())?;

    Ok(())
}

/// Create a new session
#[tauri::command]
pub async fn create_session(
    state: State<'_, AppState>,
    session: WorktreeSession,
) -> Result<WorktreeSession, String> {
    tracing::info!("Creating new session: {}", session.id);

    let mut sessions = load_sessions(&state)?;
    sessions.push(session.clone());
    save_sessions(&state, &sessions)?;

    Ok(session)
}

/// Update an existing session
#[tauri::command]
pub async fn update_session(
    state: State<'_, AppState>,
    session: WorktreeSession,
) -> Result<WorktreeSession, String> {
    tracing::info!("Updating session: {}", session.id);

    let mut sessions = load_sessions(&state)?;

    if let Some(existing) = sessions.iter_mut().find(|s| s.id == session.id) {
        *existing = session.clone();
    } else {
        return Err(format!("Session {} not found", session.id));
    }

    save_sessions(&state, &sessions)?;

    Ok(session)
}

/// Get a specific session by ID
#[tauri::command]
pub async fn get_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<WorktreeSession, String> {
    tracing::info!("Fetching session: {}", session_id);

    let sessions = load_sessions(&state)?;

    sessions
        .into_iter()
        .find(|s| s.id == session_id)
        .ok_or_else(|| format!("Session {} not found", session_id))
}

/// Get all sessions
#[tauri::command]
pub async fn get_all_sessions(state: State<'_, AppState>) -> Result<Vec<WorktreeSession>, String> {
    tracing::info!("Fetching all sessions");

    load_sessions(&state)
}

/// Delete a session
#[tauri::command]
pub async fn delete_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    tracing::info!("Deleting session: {}", session_id);

    let mut sessions = load_sessions(&state)?;
    sessions.retain(|s| s.id != session_id);
    save_sessions(&state, &sessions)?;

    Ok(())
}

/// Add chat message to a session
#[tauri::command]
pub async fn add_chat_message(
    state: State<'_, AppState>,
    session_id: String,
    message: ChatMessage,
) -> Result<WorktreeSession, String> {
    tracing::info!(
        "Adding chat message to session {}: {:?}",
        session_id,
        message.id
    );

    let mut sessions = load_sessions(&state)?;

    if let Some(session) = sessions.iter_mut().find(|s| s.id == session_id) {
        session.chat_history.push(message);
        session.updated_at = chrono::Utc::now().to_rfc3339();
        let result = session.clone();
        save_sessions(&state, &sessions)?;
        Ok(result)
    } else {
        Err(format!("Session {} not found", session_id))
    }
}

/// Update instance status within a session
#[tauri::command]
pub async fn update_instance_status(
    state: State<'_, AppState>,
    session_id: String,
    instance_id: u32,
    new_status: String,
) -> Result<WorktreeSession, String> {
    tracing::info!(
        "Updating instance {} status in session {} to {}",
        instance_id,
        session_id,
        new_status
    );

    let mut sessions = load_sessions(&state)?;

    if let Some(session) = sessions.iter_mut().find(|s| s.id == session_id) {
        if let Some(instance) = session
            .instances
            .iter_mut()
            .find(|i| i.instance_id == instance_id)
        {
            instance.status = new_status;
            session.updated_at = chrono::Utc::now().to_rfc3339();
            let result = session.clone();
            save_sessions(&state, &sessions)?;
            Ok(result)
        } else {
            Err(format!(
                "Instance {} not found in session {}",
                instance_id, session_id
            ))
        }
    } else {
        Err(format!("Session {} not found", session_id))
    }
}
