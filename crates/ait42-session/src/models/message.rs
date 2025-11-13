use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Chat message in a session
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    #[sqlx(skip)]
    pub session_id: Option<String>,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub instance_id: Option<u32>,
}

/// Database row representation for chat message queries
#[derive(Debug, Clone, FromRow)]
pub struct ChatMessageRow {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub instance_id: Option<i64>,
}

impl From<ChatMessageRow> for ChatMessage {
    fn from(row: ChatMessageRow) -> Self {
        Self {
            id: row.id,
            session_id: Some(row.session_id),
            role: row.role,
            content: row.content,
            timestamp: row.timestamp,
            instance_id: row.instance_id.map(|v| v as u32),
        }
    }
}
