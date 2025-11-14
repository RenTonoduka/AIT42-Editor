use async_trait::async_trait;
use std::sync::Arc;

use crate::db::{queries, DbPool};
use crate::error::Result;
use crate::models::{compute_workspace_hash, ChatMessage, WorktreeSession};
use crate::repository::SessionRepository;

/// SQLite implementation of SessionRepository
pub struct SqliteSessionRepository {
    pool: Arc<DbPool>,
}

impl SqliteSessionRepository {
    /// Create new repository with given database pool
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Create repository with custom database path
    pub async fn with_path(db_path: impl AsRef<std::path::Path>) -> Result<Self> {
        let pool = DbPool::new(db_path).await?;
        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Create repository with default database path
    pub async fn new_default() -> Result<Self> {
        let pool = DbPool::new_default().await?;
        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Get reference to database pool
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}

#[async_trait]
impl SessionRepository for SqliteSessionRepository {
    async fn create_session(&self, session: WorktreeSession) -> Result<WorktreeSession> {
        // Ensure workspace_hash is set
        if session.workspace_hash.is_none() {
            return Err(crate::error::SessionError::Validation(
                "workspace_hash must be set before creating session".to_string(),
            ));
        }

        queries::create_session(self.pool.pool(), session).await
    }

    async fn update_session(&self, session: WorktreeSession) -> Result<WorktreeSession> {
        queries::update_session(self.pool.pool(), &session).await?;

        // Return updated session
        let workspace_hash = session
            .workspace_hash
            .as_ref()
            .ok_or_else(|| {
                crate::error::SessionError::Validation("workspace_hash is required".to_string())
            })?;

        queries::get_session(self.pool.pool(), workspace_hash, &session.id).await
    }

    async fn get_session(&self, workspace_path: &str, session_id: &str) -> Result<WorktreeSession> {
        let workspace_hash = compute_workspace_hash(workspace_path);
        queries::get_session(self.pool.pool(), &workspace_hash, session_id).await
    }

    async fn get_all_sessions(&self, workspace_path: &str) -> Result<Vec<WorktreeSession>> {
        let workspace_hash = compute_workspace_hash(workspace_path);
        queries::get_all_sessions(self.pool.pool(), &workspace_hash).await
    }

    async fn delete_session(&self, workspace_path: &str, session_id: &str) -> Result<()> {
        let workspace_hash = compute_workspace_hash(workspace_path);
        queries::delete_session(self.pool.pool(), &workspace_hash, session_id).await
    }

    async fn add_chat_message(&self, session_id: &str, message: ChatMessage) -> Result<()> {
        queries::add_chat_message(self.pool.pool(), session_id, &message).await
    }

    async fn update_instance_status(
        &self,
        session_id: &str,
        instance_id: u32,
        new_status: &str,
    ) -> Result<()> {
        queries::update_instance_status(self.pool.pool(), session_id, instance_id, new_status).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_repository_creation() {
        let db_file = NamedTempFile::new().unwrap();
        let repo = SqliteSessionRepository::with_path(db_file.path()).await;
        assert!(repo.is_ok());
    }
}
