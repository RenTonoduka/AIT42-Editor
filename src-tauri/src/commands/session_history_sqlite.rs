/**
 * Session History Management - SQLite Implementation
 *
 * This module provides SQLite-based session storage as a replacement for
 * the JSON file-based storage in session_history.rs.
 *
 * During Phase 2 (Dual Write), both implementations will run in parallel.
 * During Phase 3 (SQLite Primary), this will become the primary implementation.
 */

use ait42_session::{
    ChatMessage, SessionRepository, SqliteSessionRepository, WorktreeSession,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::{debug, error, info, warn};

use crate::state::AppState;

// Re-export types from ait42-session for compatibility
pub use ait42_session::{compute_workspace_hash, WorktreeInstance};

/// Session repository wrapper in AppState
pub struct SessionRepo {
    pub repo: Arc<SqliteSessionRepository>,
}

impl SessionRepo {
    pub async fn new() -> anyhow::Result<Self> {
        let repo = SqliteSessionRepository::new_default().await?;
        Ok(Self {
            repo: Arc::new(repo),
        })
    }

    pub fn repo(&self) -> &Arc<SqliteSessionRepository> {
        &self.repo
    }
}

// ===================================
// Tauri Commands
// ===================================

/// Create a new session
#[tauri::command]
pub async fn create_session_sqlite(
    state: State<'_, AppState>,
    workspace_path: String,
    mut session: WorktreeSession,
) -> Result<WorktreeSession, String> {
    info!("Creating session {} for workspace {}", session.id, workspace_path);

    // Validation
    if workspace_path.trim().is_empty() {
        error!("Empty workspace path provided");
        return Err("Workspace path cannot be empty".to_string());
    }

    if session.id.trim().is_empty() {
        error!("Empty session ID provided");
        return Err("Session ID cannot be empty".to_string());
    }

    // Compute and set workspace hash
    let workspace_hash = compute_workspace_hash(&workspace_path);
    session.workspace_hash = Some(workspace_hash);

    // Get session repository
    let repo = state
        .session_repo
        .as_ref()
        .ok_or_else(|| "Session repository not initialized".to_string())?;

    // Create session in database
    repo.repo()
        .create_session(session)
        .await
        .map_err(|e| {
            error!("Failed to create session: {}", e);
            e.to_string()
        })
}

/// Update an existing session
#[tauri::command]
pub async fn update_session_sqlite(
    state: State<'_, AppState>,
    workspace_path: String,
    mut session: WorktreeSession,
) -> Result<WorktreeSession, String> {
    info!("Updating session {} for workspace {}", session.id, workspace_path);

    // Validation
    if workspace_path.trim().is_empty() {
        return Err("Workspace path cannot be empty".to_string());
    }

    // Ensure workspace hash is set
    if session.workspace_hash.is_none() {
        let workspace_hash = compute_workspace_hash(&workspace_path);
        session.workspace_hash = Some(workspace_hash);
    }

    // Get session repository
    let repo = state
        .session_repo
        .as_ref()
        .ok_or_else(|| "Session repository not initialized".to_string())?;

    // Update session in database
    repo.repo()
        .update_session(session)
        .await
        .map_err(|e| {
            error!("Failed to update session: {}", e);
            e.to_string()
        })
}

/// Get a specific session by ID
#[tauri::command]
pub async fn get_session_sqlite(
    state: State<'_, AppState>,
    workspace_path: String,
    session_id: String,
) -> Result<WorktreeSession, String> {
    info!("Fetching session {} for workspace {}", session_id, workspace_path);

    if workspace_path.trim().is_empty() {
        return Err("Workspace path cannot be empty".to_string());
    }

    // Get session repository
    let repo = state
        .session_repo
        .as_ref()
        .ok_or_else(|| "Session repository not initialized".to_string())?;

    // Get session from database
    repo.repo()
        .get_session(&workspace_path, &session_id)
        .await
        .map_err(|e| {
            error!("Failed to get session: {}", e);
            e.to_string()
        })
}

/// Get all sessions for a workspace
#[tauri::command]
pub async fn get_all_sessions_sqlite(
    state: State<'_, AppState>,
    workspace_path: String,
) -> Result<Vec<WorktreeSession>, String> {
    info!("Fetching all sessions for workspace {}", workspace_path);

    if workspace_path.trim().is_empty() {
        warn!("Empty workspace path - returning empty array");
        return Ok(Vec::new());
    }

    // Get session repository
    let repo = state
        .session_repo
        .as_ref()
        .ok_or_else(|| "Session repository not initialized".to_string())?;

    // Get all sessions from database
    repo.repo()
        .get_all_sessions(&workspace_path)
        .await
        .map_err(|e| {
            error!("Failed to get all sessions: {}", e);
            e.to_string()
        })
}

/// Delete a session
#[tauri::command]
pub async fn delete_session_sqlite(
    state: State<'_, AppState>,
    workspace_path: String,
    session_id: String,
) -> Result<(), String> {
    info!("Deleting session {} for workspace {}", session_id, workspace_path);

    if workspace_path.trim().is_empty() {
        return Err("Workspace path cannot be empty".to_string());
    }

    // Get session repository
    let repo = state
        .session_repo
        .as_ref()
        .ok_or_else(|| "Session repository not initialized".to_string())?;

    // Delete session from database
    repo.repo()
        .delete_session(&workspace_path, &session_id)
        .await
        .map_err(|e| {
            error!("Failed to delete session: {}", e);
            e.to_string()
        })
}

/// Add chat message to a session
#[tauri::command]
pub async fn add_chat_message_sqlite(
    state: State<'_, AppState>,
    workspace_path: String,
    session_id: String,
    mut message: ChatMessage,
) -> Result<WorktreeSession, String> {
    info!("Adding chat message to session {}", session_id);

    if workspace_path.trim().is_empty() {
        return Err("Workspace path cannot be empty".to_string());
    }

    // Get session repository
    let repo = state
        .session_repo
        .as_ref()
        .ok_or_else(|| "Session repository not initialized".to_string())?;

    // Add chat message
    repo.repo()
        .add_chat_message(&session_id, message)
        .await
        .map_err(|e| {
            error!("Failed to add chat message: {}", e);
            e.to_string()
        })?;

    // Return updated session
    repo.repo()
        .get_session(&workspace_path, &session_id)
        .await
        .map_err(|e| e.to_string())
}

/// Update instance status within a session
#[tauri::command]
pub async fn update_instance_status_sqlite(
    state: State<'_, AppState>,
    workspace_path: String,
    session_id: String,
    instance_id: u32,
    new_status: String,
) -> Result<WorktreeSession, String> {
    info!(
        "Updating instance {} status to {} in session {}",
        instance_id, new_status, session_id
    );

    if workspace_path.trim().is_empty() {
        return Err("Workspace path cannot be empty".to_string());
    }

    // Get session repository
    let repo = state
        .session_repo
        .as_ref()
        .ok_or_else(|| "Session repository not initialized".to_string())?;

    // Update instance status
    repo.repo()
        .update_instance_status(&session_id, instance_id, &new_status)
        .await
        .map_err(|e| {
            error!("Failed to update instance status: {}", e);
            e.to_string()
        })?;

    // Return updated session
    repo.repo()
        .get_session(&workspace_path, &session_id)
        .await
        .map_err(|e| e.to_string())
}

// ===================================
// Database Management Commands
// ===================================

/// Get database statistics
#[tauri::command]
pub async fn get_database_stats(state: State<'_, AppState>) -> Result<DatabaseStats, String> {
    debug!("Fetching database statistics");

    let repo = state
        .session_repo
        .as_ref()
        .ok_or_else(|| "Session repository not initialized".to_string())?;

    let pool = repo.repo().pool().pool();

    // Count sessions
    let session_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    // Count instances
    let instance_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM instances")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    // Count messages
    let message_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chat_messages")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    // Get database file size
    let db_path = repo.repo().pool().db_path();
    let db_size = std::fs::metadata(db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(DatabaseStats {
        session_count: session_count as usize,
        instance_count: instance_count as usize,
        message_count: message_count as usize,
        database_size_bytes: db_size,
        database_path: db_path.to_string_lossy().to_string(),
    })
}

/// Optimize database (run PRAGMA optimize and vacuum)
#[tauri::command]
pub async fn optimize_database(state: State<'_, AppState>) -> Result<(), String> {
    info!("Optimizing database");

    let repo = state
        .session_repo
        .as_ref()
        .ok_or_else(|| "Session repository not initialized".to_string())?;

    repo.repo()
        .pool()
        .optimize()
        .await
        .map_err(|e| {
            error!("Failed to optimize database: {}", e);
            e.to_string()
        })?;

    info!("Database optimization completed");
    Ok(())
}

/// Verify database integrity
#[tauri::command]
pub async fn verify_database_integrity(state: State<'_, AppState>) -> Result<bool, String> {
    debug!("Verifying database integrity");

    let repo = state
        .session_repo
        .as_ref()
        .ok_or_else(|| "Session repository not initialized".to_string())?;

    repo.repo()
        .pool()
        .verify_integrity()
        .await
        .map_err(|e| {
            error!("Failed to verify database integrity: {}", e);
            e.to_string()
        })
}

// ===================================
// Response Types
// ===================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStats {
    pub session_count: usize,
    pub instance_count: usize,
    pub message_count: usize,
    pub database_size_bytes: u64,
    pub database_path: String,
}
