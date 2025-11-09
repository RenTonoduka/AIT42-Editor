/**
 * Session History Management - Persistent storage for worktree sessions
 * Inspired by Vibe Kanban's SQLite-based persistence
 *
 * Sessions are now workspace-specific, stored in ~/.ait42/sessions/{workspace_hash}.json
 */

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
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

/// Generate a stable hash from workspace path for file naming
/// Uses SHA256 to create a deterministic identifier
///
/// Normalizes paths before hashing to ensure consistency:
/// - Trailing slashes are removed (/path/to/project and /path/to/project/ produce same hash)
/// - Symbolic links are resolved to their targets
/// - Relative paths are converted to absolute paths
///
/// Falls back to cleaned path if canonicalization fails (e.g., path doesn't exist yet)
fn workspace_hash(workspace_path: &str) -> String {
    use std::path::Path;

    // Attempt to canonicalize the path (resolves symlinks, converts to absolute path)
    let normalized_path = match fs::canonicalize(Path::new(workspace_path)) {
        Ok(canonical) => canonical.to_string_lossy().to_string(),
        Err(_) => {
            // If canonicalization fails (e.g., path doesn't exist), clean the path
            // Remove trailing slashes for consistency
            workspace_path.trim_end_matches('/').to_string()
        }
    };

    let mut hasher = Sha256::new();
    hasher.update(normalized_path.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string() // Use first 16 chars for readability
}

/// Get path to sessions storage file for a specific workspace
/// Uses user's home directory to avoid read-only file system errors in macOS app bundles
/// Format: ~/.ait42/sessions/{workspace_hash}.json
///
/// Fallback behavior:
/// - Primary: Uses home directory (~/.ait42/sessions/)
/// - Fallback: Uses /tmp directory if home directory cannot be determined
///   (This may cause session data loss on system reboot, but prevents crashes)
fn get_sessions_file_path(_state: &AppState, workspace_path: &str) -> PathBuf {
    // Use home directory instead of working directory to avoid read-only issues
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let hash = workspace_hash(workspace_path);
    home_dir
        .join(".ait42")
        .join("sessions")
        .join(format!("{}.json", hash))
}

/// Ensure .ait42/sessions directory exists in user's home directory
fn ensure_storage_dir(_state: &AppState) -> Result<(), String> {
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let sessions_dir = home_dir.join(".ait42").join("sessions");

    if !sessions_dir.exists() {
        fs::create_dir_all(&sessions_dir).map_err(|e| {
            format!("Failed to create .ait42/sessions directory at {:?}: {}", sessions_dir, e)
        })?;
    }

    Ok(())
}

/// Load all sessions from disk for a specific workspace
fn load_sessions(state: &AppState, workspace_path: &str) -> Result<Vec<WorktreeSession>, String> {
    let sessions_file = get_sessions_file_path(state, workspace_path);

    if !sessions_file.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&sessions_file).map_err(|e| e.to_string())?;

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse sessions: {}", e))
}

/// Save all sessions to disk for a specific workspace
fn save_sessions(state: &AppState, workspace_path: &str, sessions: &[WorktreeSession]) -> Result<(), String> {
    ensure_storage_dir(state)?;

    let sessions_file = get_sessions_file_path(state, workspace_path);
    let content = serde_json::to_string_pretty(sessions).map_err(|e| e.to_string())?;

    fs::write(&sessions_file, content).map_err(|e| e.to_string())?;

    Ok(())
}

/// Create a new session
#[tauri::command]
pub async fn create_session(
    state: State<'_, AppState>,
    workspace_path: String,
    session: WorktreeSession,
) -> Result<WorktreeSession, String> {
    tracing::info!("Creating new session: {} for workspace: {}", session.id, workspace_path);

    let mut sessions = load_sessions(&state, &workspace_path)?;
    sessions.push(session.clone());
    save_sessions(&state, &workspace_path, &sessions)?;

    Ok(session)
}

/// Update an existing session
#[tauri::command]
pub async fn update_session(
    state: State<'_, AppState>,
    workspace_path: String,
    session: WorktreeSession,
) -> Result<WorktreeSession, String> {
    tracing::info!("Updating session: {} for workspace: {}", session.id, workspace_path);

    let mut sessions = load_sessions(&state, &workspace_path)?;

    if let Some(existing) = sessions.iter_mut().find(|s| s.id == session.id) {
        *existing = session.clone();
    } else {
        return Err(format!("Session {} not found", session.id));
    }

    save_sessions(&state, &workspace_path, &sessions)?;

    Ok(session)
}

/// Get a specific session by ID
#[tauri::command]
pub async fn get_session(
    state: State<'_, AppState>,
    workspace_path: String,
    session_id: String,
) -> Result<WorktreeSession, String> {
    tracing::info!("Fetching session: {} for workspace: {}", session_id, workspace_path);

    let sessions = load_sessions(&state, &workspace_path)?;

    sessions
        .into_iter()
        .find(|s| s.id == session_id)
        .ok_or_else(|| format!("Session {} not found", session_id))
}

/// Get all sessions for a specific workspace
#[tauri::command]
pub async fn get_all_sessions(
    state: State<'_, AppState>,
    workspace_path: String,
) -> Result<Vec<WorktreeSession>, String> {
    tracing::info!("Fetching all sessions for workspace: {}", workspace_path);

    load_sessions(&state, &workspace_path)
}

/// Delete a session
#[tauri::command]
pub async fn delete_session(
    state: State<'_, AppState>,
    workspace_path: String,
    session_id: String,
) -> Result<(), String> {
    tracing::info!("Deleting session: {} for workspace: {}", session_id, workspace_path);

    let mut sessions = load_sessions(&state, &workspace_path)?;
    sessions.retain(|s| s.id != session_id);
    save_sessions(&state, &workspace_path, &sessions)?;

    Ok(())
}

/// Add chat message to a session
#[tauri::command]
pub async fn add_chat_message(
    state: State<'_, AppState>,
    workspace_path: String,
    session_id: String,
    message: ChatMessage,
) -> Result<WorktreeSession, String> {
    tracing::info!(
        "Adding chat message to session {} for workspace {}: {:?}",
        session_id,
        workspace_path,
        message.id
    );

    let mut sessions = load_sessions(&state, &workspace_path)?;

    if let Some(session) = sessions.iter_mut().find(|s| s.id == session_id) {
        session.chat_history.push(message);
        session.updated_at = chrono::Utc::now().to_rfc3339();
        let result = session.clone();
        save_sessions(&state, &workspace_path, &sessions)?;
        Ok(result)
    } else {
        Err(format!("Session {} not found", session_id))
    }
}

/// Update instance status within a session
#[tauri::command]
pub async fn update_instance_status(
    state: State<'_, AppState>,
    workspace_path: String,
    session_id: String,
    instance_id: u32,
    new_status: String,
) -> Result<WorktreeSession, String> {
    tracing::info!(
        "Updating instance {} status in session {} to {} for workspace: {}",
        instance_id,
        session_id,
        new_status,
        workspace_path
    );

    let mut sessions = load_sessions(&state, &workspace_path)?;

    if let Some(session) = sessions.iter_mut().find(|s| s.id == session_id) {
        if let Some(instance) = session
            .instances
            .iter_mut()
            .find(|i| i.instance_id == instance_id)
        {
            instance.status = new_status;
            session.updated_at = chrono::Utc::now().to_rfc3339();
            let result = session.clone();
            save_sessions(&state, &workspace_path, &sessions)?;
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
