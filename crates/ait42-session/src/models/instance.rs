use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Worktree instance within a session
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeInstance {
    #[sqlx(skip)]
    pub instance_id: u32,
    #[sqlx(skip)]
    pub session_id: Option<String>,
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
    pub runtime: Option<String>,
    pub model: Option<String>,
    pub runtime_label: Option<String>,
}

/// Database row representation for instance queries
#[derive(Debug, Clone, FromRow)]
pub struct InstanceRow {
    pub id: i64,
    pub session_id: String,
    pub instance_id: i64,
    pub worktree_path: String,
    pub branch: String,
    pub agent_name: String,
    pub status: String,
    pub tmux_session_id: String,
    pub output: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub files_changed: Option<i64>,
    pub lines_added: Option<i64>,
    pub lines_deleted: Option<i64>,
    pub runtime: Option<String>,
    pub model: Option<String>,
    pub runtime_label: Option<String>,
}

impl From<InstanceRow> for WorktreeInstance {
    fn from(row: InstanceRow) -> Self {
        Self {
            instance_id: row.instance_id as u32,
            session_id: Some(row.session_id),
            worktree_path: row.worktree_path,
            branch: row.branch,
            agent_name: row.agent_name,
            status: row.status,
            tmux_session_id: row.tmux_session_id,
            output: row.output,
            start_time: row.start_time,
            end_time: row.end_time,
            files_changed: row.files_changed.map(|v| v as u32),
            lines_added: row.lines_added.map(|v| v as u32),
            lines_deleted: row.lines_deleted.map(|v| v as u32),
            runtime: row.runtime,
            model: row.model,
            runtime_label: row.runtime_label,
        }
    }
}
