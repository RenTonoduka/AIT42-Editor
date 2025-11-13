use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::{ChatMessage, WorktreeInstance};

/// Main session data structure
/// Compatible with existing JSON-based storage format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeSession {
    pub id: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub workspace_hash: Option<String>,
    #[serde(rename = "type")]
    pub r#type: String,
    pub task: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub model: Option<String>,
    pub timeout_seconds: Option<u32>,
    pub preserve_worktrees: Option<bool>,
    pub winner_id: Option<u32>,
    pub runtime_mix: Option<Vec<String>>,
    pub total_duration: Option<u64>,
    pub total_files_changed: Option<u32>,
    pub total_lines_added: Option<u32>,
    pub total_lines_deleted: Option<u32>,

    #[serde(default)]
    pub instances: Vec<WorktreeInstance>,
    #[serde(default)]
    pub chat_history: Vec<ChatMessage>,
}

/// Database row representation for session queries
#[derive(Debug, Clone, FromRow)]
pub struct SessionRow {
    pub id: String,
    pub workspace_hash: String,
    pub session_type: String,
    pub task: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub model: Option<String>,
    pub timeout_seconds: Option<i64>,
    pub preserve_worktrees: Option<i64>,
    pub winner_id: Option<i64>,
    pub runtime_mix: Option<String>, // JSON-encoded Vec<String>
    pub total_duration: Option<i64>,
    pub total_files_changed: Option<i64>,
    pub total_lines_added: Option<i64>,
    pub total_lines_deleted: Option<i64>,
}

impl From<SessionRow> for WorktreeSession {
    fn from(row: SessionRow) -> Self {
        let runtime_mix = row.runtime_mix
            .and_then(|s| serde_json::from_str(&s).ok());

        Self {
            id: row.id,
            workspace_hash: Some(row.workspace_hash),
            r#type: row.session_type,
            task: row.task,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
            completed_at: row.completed_at,
            model: row.model,
            timeout_seconds: row.timeout_seconds.map(|v| v as u32),
            preserve_worktrees: row.preserve_worktrees.map(|v| v != 0),
            winner_id: row.winner_id.map(|v| v as u32),
            runtime_mix,
            total_duration: row.total_duration.map(|v| v as u64),
            total_files_changed: row.total_files_changed.map(|v| v as u32),
            total_lines_added: row.total_lines_added.map(|v| v as u32),
            total_lines_deleted: row.total_lines_deleted.map(|v| v as u32),
            instances: Vec::new(),
            chat_history: Vec::new(),
        }
    }
}

impl WorktreeSession {
    /// Create a new session with default values
    pub fn new(
        id: String,
        workspace_hash: String,
        session_type: String,
        task: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            workspace_hash: Some(workspace_hash),
            r#type: session_type,
            task,
            status: "running".to_string(),
            created_at: now.clone(),
            updated_at: now,
            completed_at: None,
            model: None,
            timeout_seconds: None,
            preserve_worktrees: None,
            winner_id: None,
            runtime_mix: None,
            total_duration: None,
            total_files_changed: None,
            total_lines_added: None,
            total_lines_deleted: None,
            instances: Vec::new(),
            chat_history: Vec::new(),
        }
    }

    /// Get workspace hash, computing it if not set
    pub fn ensure_workspace_hash(&mut self, workspace_path: &str) -> &str {
        if self.workspace_hash.is_none() {
            self.workspace_hash = Some(compute_workspace_hash(workspace_path));
        }
        self.workspace_hash.as_ref().unwrap()
    }
}

/// Compute workspace hash from path (same algorithm as JSON storage)
pub fn compute_workspace_hash(workspace_path: &str) -> String {
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::path::Path;

    let normalized_path = match fs::canonicalize(Path::new(workspace_path)) {
        Ok(canonical) => canonical.to_string_lossy().to_string(),
        Err(_) => workspace_path.trim_end_matches('/').to_string(),
    };

    let mut hasher = Sha256::new();
    hasher.update(normalized_path.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}
