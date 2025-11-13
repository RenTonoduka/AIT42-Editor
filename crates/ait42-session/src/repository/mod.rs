use async_trait::async_trait;

use crate::error::Result;
use crate::models::{ChatMessage, WorktreeSession};

pub mod sqlite;

pub use sqlite::SqliteSessionRepository;

/// Session repository trait
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Create a new session
    async fn create_session(&self, session: WorktreeSession) -> Result<WorktreeSession>;

    /// Update existing session
    async fn update_session(&self, session: WorktreeSession) -> Result<WorktreeSession>;

    /// Get session by ID
    async fn get_session(&self, workspace_path: &str, session_id: &str) -> Result<WorktreeSession>;

    /// Get all sessions for a workspace
    async fn get_all_sessions(&self, workspace_path: &str) -> Result<Vec<WorktreeSession>>;

    /// Delete session
    async fn delete_session(&self, workspace_path: &str, session_id: &str) -> Result<()>;

    /// Add chat message to session
    async fn add_chat_message(&self, session_id: &str, message: ChatMessage) -> Result<()>;

    /// Update instance status
    async fn update_instance_status(
        &self,
        session_id: &str,
        instance_id: u32,
        new_status: &str,
    ) -> Result<()>;
}
