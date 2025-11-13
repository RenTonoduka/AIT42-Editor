# SQLite Migration - Implementation Guide

## Quick Start

This guide provides concrete code examples for implementing the SQLite migration.

### Prerequisites

```bash
# Install SQLx CLI for migrations
cargo install sqlx-cli --no-default-features --features sqlite

# Create database and run migrations
cd crates/ait42-session
sqlx database create
sqlx migrate run
```

---

## Phase 1: Foundation - Code Examples

### 1.1 Create Crate Structure

```bash
# Create new crate
cargo new --lib crates/ait42-session

# Create directory structure
mkdir -p crates/ait42-session/src/{db,models,repository,migration}
mkdir -p crates/ait42-session/migrations
mkdir -p crates/ait42-session/tests/fixtures
```

### 1.2 Cargo.toml Configuration

```toml
# crates/ait42-session/Cargo.toml
[package]
name = "ait42-session"
version = "0.1.0"
edition = "2021"
description = "Session data management with SQLite persistence"

[dependencies]
# Database
sqlx = { version = "0.7.3", features = [
    "runtime-tokio-rustls",
    "sqlite",
    "macros",
    "migrate",
    "chrono",
    "uuid"
] }

# Async runtime (shared with workspace)
tokio = { workspace = true }
async-trait = { workspace = true }

# Serialization
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }

# Logging
tracing = { workspace = true }

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1.6", features = ["serde", "v4"] }

[dev-dependencies]
tempfile = { workspace = true }
tokio-test = "0.4"
```

### 1.3 Database Schema Migration

```sql
-- migrations/20250113_001_initial_schema.sql
-- Session storage schema

CREATE TABLE IF NOT EXISTS workspaces (
    hash TEXT PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    last_accessed TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    workspace_hash TEXT NOT NULL,
    session_type TEXT NOT NULL CHECK(session_type IN ('competition', 'ensemble', 'debate')),
    task TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('running', 'completed', 'failed', 'paused')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,
    model TEXT,
    timeout_seconds INTEGER,
    preserve_worktrees INTEGER, -- SQLite boolean (0/1)
    winner_id INTEGER,
    runtime_mix TEXT, -- JSON array
    total_duration INTEGER,
    total_files_changed INTEGER,
    total_lines_added INTEGER,
    total_lines_deleted INTEGER,
    integration_phase TEXT CHECK(integration_phase IN ('pending', 'in_progress', 'completed')),
    integration_instance_id INTEGER,
    FOREIGN KEY (workspace_hash) REFERENCES workspaces(hash) ON DELETE CASCADE
);

CREATE INDEX idx_sessions_workspace ON sessions(workspace_hash);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_type ON sessions(session_type);
CREATE INDEX idx_sessions_created ON sessions(created_at DESC);
CREATE INDEX idx_sessions_updated ON sessions(updated_at DESC);

CREATE TABLE IF NOT EXISTS instances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    instance_id INTEGER NOT NULL,
    worktree_path TEXT NOT NULL,
    branch TEXT NOT NULL,
    agent_name TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('idle', 'running', 'completed', 'failed', 'paused', 'archived')),
    tmux_session_id TEXT NOT NULL,
    output TEXT, -- Can be large
    start_time TEXT,
    end_time TEXT,
    files_changed INTEGER,
    lines_added INTEGER,
    lines_deleted INTEGER,
    runtime TEXT,
    model TEXT,
    runtime_label TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    UNIQUE(session_id, instance_id)
);

CREATE INDEX idx_instances_session ON instances(session_id);
CREATE INDEX idx_instances_status ON instances(status);

CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    instance_id INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE INDEX idx_messages_session ON chat_messages(session_id);
CREATE INDEX idx_messages_timestamp ON chat_messages(timestamp DESC);
```

### 1.4 Domain Models

```rust
// crates/ait42-session/src/models/mod.rs
pub mod session;
pub mod instance;
pub mod message;

pub use session::*;
pub use instance::*;
pub use message::*;
```

```rust
// crates/ait42-session/src/models/session.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeSession {
    pub id: String,
    pub workspace_hash: String,
    #[sqlx(rename = "session_type")]
    pub r#type: String,
    pub task: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub model: Option<String>,
    pub timeout_seconds: Option<i64>,
    #[sqlx(rename = "preserve_worktrees")]
    pub preserve_worktrees: Option<bool>,
    pub winner_id: Option<i64>,
    pub runtime_mix: Option<String>, // JSON-encoded Vec<String>
    pub total_duration: Option<i64>,
    pub total_files_changed: Option<i64>,
    pub total_lines_added: Option<i64>,
    pub total_lines_deleted: Option<i64>,
    pub integration_phase: Option<String>,
    pub integration_instance_id: Option<i64>,

    // Note: instances and chat_history are loaded separately via JOINs
    #[sqlx(skip)]
    pub instances: Vec<WorktreeInstance>,
    #[sqlx(skip)]
    pub chat_history: Vec<ChatMessage>,
}

impl WorktreeSession {
    pub fn new(
        id: String,
        workspace_hash: String,
        session_type: String,
        task: String,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id,
            workspace_hash,
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
            integration_phase: None,
            integration_instance_id: None,
            instances: Vec::new(),
            chat_history: Vec::new(),
        }
    }
}
```

```rust
// crates/ait42-session/src/models/instance.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeInstance {
    #[sqlx(skip)]
    pub instance_id: i64,
    pub session_id: String,
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
```

```rust
// crates/ait42-session/src/models/message.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub instance_id: Option<i64>,
}
```

### 1.5 Database Connection Pool

```rust
// crates/ait42-session/src/db/connection.rs
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

pub struct DbPool {
    pool: SqlitePool,
}

impl DbPool {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // Parse connection options
        let mut options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(30));

        // Disable logging for connection setup
        options.disable_statement_logging();

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(10))
            .connect_with(options)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        // Optimize pragmas
        sqlx::query("PRAGMA cache_size = -8000") // 8MB cache
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA temp_store = MEMORY")
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn close(self) {
        self.pool.close().await;
    }

    pub async fn optimize(&self) -> Result<(), sqlx::Error> {
        sqlx::query("PRAGMA optimize").execute(&self.pool).await?;
        sqlx::query("PRAGMA incremental_vacuum(100)").execute(&self.pool).await?;
        Ok(())
    }

    pub async fn verify_integrity(&self) -> Result<bool, sqlx::Error> {
        let result: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&self.pool)
            .await?;
        Ok(result == "ok")
    }
}
```

### 1.6 Repository Trait Definition

```rust
// crates/ait42-session/src/repository/mod.rs
use crate::models::{ChatMessage, WorktreeInstance, WorktreeSession};
use async_trait::async_trait;

pub mod sqlite;

pub use sqlite::SqliteSessionRepository;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Create a new session
    async fn create_session(&self, session: WorktreeSession) -> Result<WorktreeSession, RepositoryError>;

    /// Update an existing session
    async fn update_session(&self, session: WorktreeSession) -> Result<WorktreeSession, RepositoryError>;

    /// Get a specific session by ID
    async fn get_session(&self, workspace_hash: &str, session_id: &str) -> Result<WorktreeSession, RepositoryError>;

    /// Get all sessions for a workspace
    async fn get_all_sessions(&self, workspace_hash: &str) -> Result<Vec<WorktreeSession>, RepositoryError>;

    /// Delete a session
    async fn delete_session(&self, workspace_hash: &str, session_id: &str) -> Result<(), RepositoryError>;

    /// Add chat message to session
    async fn add_chat_message(&self, session_id: &str, message: ChatMessage) -> Result<(), RepositoryError>;

    /// Update instance status
    async fn update_instance_status(
        &self,
        session_id: &str,
        instance_id: i64,
        new_status: &str,
    ) -> Result<(), RepositoryError>;

    /// Search sessions by filters
    async fn search_sessions(
        &self,
        workspace_hash: &str,
        filters: SessionFilters,
    ) -> Result<Vec<WorktreeSession>, RepositoryError>;
}

#[derive(Debug, Clone, Default)]
pub struct SessionFilters {
    pub session_type: Option<Vec<String>>,
    pub status: Option<Vec<String>>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub search_query: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
```

### 1.7 SQLite Repository Implementation

```rust
// crates/ait42-session/src/repository/sqlite.rs
use crate::db::DbPool;
use crate::models::{ChatMessage, WorktreeInstance, WorktreeSession};
use crate::repository::{RepositoryError, SessionFilters, SessionRepository};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct SqliteSessionRepository {
    pool: Arc<DbPool>,
}

impl SqliteSessionRepository {
    pub async fn new(database_url: &str) -> Result<Self, RepositoryError> {
        let pool = DbPool::new(database_url).await?;
        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    pub fn pool(&self) -> &SqlitePool {
        self.pool.pool()
    }

    async fn load_instances(&self, session_id: &str) -> Result<Vec<WorktreeInstance>, RepositoryError> {
        let instances = sqlx::query_as::<_, WorktreeInstance>(
            r#"
            SELECT
                id, session_id, instance_id, worktree_path, branch,
                agent_name, status, tmux_session_id, output,
                start_time, end_time, files_changed, lines_added,
                lines_deleted, runtime, model, runtime_label
            FROM instances
            WHERE session_id = ?
            ORDER BY instance_id ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(self.pool())
        .await?;

        Ok(instances)
    }

    async fn load_messages(&self, session_id: &str) -> Result<Vec<ChatMessage>, RepositoryError> {
        let messages = sqlx::query_as::<_, ChatMessage>(
            r#"
            SELECT id, session_id, role, content, timestamp, instance_id
            FROM chat_messages
            WHERE session_id = ?
            ORDER BY timestamp ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(self.pool())
        .await?;

        Ok(messages)
    }
}

#[async_trait]
impl SessionRepository for SqliteSessionRepository {
    async fn create_session(&self, mut session: WorktreeSession) -> Result<WorktreeSession, RepositoryError> {
        let mut tx = self.pool().begin().await?;

        // Ensure workspace exists
        sqlx::query(
            r#"
            INSERT INTO workspaces (hash, path, last_accessed)
            VALUES (?, ?, ?)
            ON CONFLICT(hash) DO UPDATE SET last_accessed = excluded.last_accessed
            "#,
        )
        .bind(&session.workspace_hash)
        .bind(&session.workspace_hash) // Use hash as path for now (will be improved)
        .bind(Utc::now().to_rfc3339())
        .execute(&mut *tx)
        .await?;

        // Insert session
        let runtime_mix_json = session.runtime_mix.as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());

        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, workspace_hash, session_type, task, status,
                created_at, updated_at, completed_at, model,
                timeout_seconds, preserve_worktrees, winner_id,
                runtime_mix, total_duration, total_files_changed,
                total_lines_added, total_lines_deleted,
                integration_phase, integration_instance_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind(&session.workspace_hash)
        .bind(&session.r#type)
        .bind(&session.task)
        .bind(&session.status)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .bind(&session.completed_at)
        .bind(&session.model)
        .bind(session.timeout_seconds)
        .bind(session.preserve_worktrees)
        .bind(session.winner_id)
        .bind(runtime_mix_json)
        .bind(session.total_duration)
        .bind(session.total_files_changed)
        .bind(session.total_lines_added)
        .bind(session.total_lines_deleted)
        .bind(&session.integration_phase)
        .bind(session.integration_instance_id)
        .execute(&mut *tx)
        .await?;

        // Insert instances
        for instance in &session.instances {
            sqlx::query(
                r#"
                INSERT INTO instances (
                    session_id, instance_id, worktree_path, branch,
                    agent_name, status, tmux_session_id, output,
                    start_time, end_time, files_changed, lines_added,
                    lines_deleted, runtime, model, runtime_label
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&session.id)
            .bind(instance.instance_id)
            .bind(&instance.worktree_path)
            .bind(&instance.branch)
            .bind(&instance.agent_name)
            .bind(&instance.status)
            .bind(&instance.tmux_session_id)
            .bind(&instance.output)
            .bind(&instance.start_time)
            .bind(&instance.end_time)
            .bind(instance.files_changed)
            .bind(instance.lines_added)
            .bind(instance.lines_deleted)
            .bind(&instance.runtime)
            .bind(&instance.model)
            .bind(&instance.runtime_label)
            .execute(&mut *tx)
            .await?;
        }

        // Insert chat messages
        for message in &session.chat_history {
            sqlx::query(
                r#"
                INSERT INTO chat_messages (id, session_id, role, content, timestamp, instance_id)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&message.id)
            .bind(&session.id)
            .bind(&message.role)
            .bind(&message.content)
            .bind(&message.timestamp)
            .bind(message.instance_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(session)
    }

    async fn get_session(&self, workspace_hash: &str, session_id: &str) -> Result<WorktreeSession, RepositoryError> {
        let mut session = sqlx::query_as::<_, WorktreeSession>(
            r#"
            SELECT
                id, workspace_hash, session_type as type, task, status,
                created_at, updated_at, completed_at, model,
                timeout_seconds, preserve_worktrees, winner_id,
                runtime_mix, total_duration, total_files_changed,
                total_lines_added, total_lines_deleted,
                integration_phase, integration_instance_id
            FROM sessions
            WHERE workspace_hash = ? AND id = ?
            "#,
        )
        .bind(workspace_hash)
        .bind(session_id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| RepositoryError::NotFound(session_id.to_string()))?;

        // Load related data
        session.instances = self.load_instances(session_id).await?;
        session.chat_history = self.load_messages(session_id).await?;

        Ok(session)
    }

    async fn get_all_sessions(&self, workspace_hash: &str) -> Result<Vec<WorktreeSession>, RepositoryError> {
        let sessions = sqlx::query_as::<_, WorktreeSession>(
            r#"
            SELECT
                id, workspace_hash, session_type as type, task, status,
                created_at, updated_at, completed_at, model,
                timeout_seconds, preserve_worktrees, winner_id,
                runtime_mix, total_duration, total_files_changed,
                total_lines_added, total_lines_deleted,
                integration_phase, integration_instance_id
            FROM sessions
            WHERE workspace_hash = ?
            ORDER BY updated_at DESC
            "#,
        )
        .bind(workspace_hash)
        .fetch_all(self.pool())
        .await?;

        // Load instances and messages for each session
        let mut result = Vec::new();
        for mut session in sessions {
            session.instances = self.load_instances(&session.id).await?;
            session.chat_history = self.load_messages(&session.id).await?;
            result.push(session);
        }

        Ok(result)
    }

    async fn update_session(&self, session: WorktreeSession) -> Result<WorktreeSession, RepositoryError> {
        let mut tx = self.pool().begin().await?;

        let runtime_mix_json = session.runtime_mix.as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());

        // Update session
        sqlx::query(
            r#"
            UPDATE sessions SET
                session_type = ?, task = ?, status = ?,
                updated_at = ?, completed_at = ?, model = ?,
                timeout_seconds = ?, preserve_worktrees = ?,
                winner_id = ?, runtime_mix = ?,
                total_duration = ?, total_files_changed = ?,
                total_lines_added = ?, total_lines_deleted = ?,
                integration_phase = ?, integration_instance_id = ?
            WHERE id = ?
            "#,
        )
        .bind(&session.r#type)
        .bind(&session.task)
        .bind(&session.status)
        .bind(&session.updated_at)
        .bind(&session.completed_at)
        .bind(&session.model)
        .bind(session.timeout_seconds)
        .bind(session.preserve_worktrees)
        .bind(session.winner_id)
        .bind(runtime_mix_json)
        .bind(session.total_duration)
        .bind(session.total_files_changed)
        .bind(session.total_lines_added)
        .bind(session.total_lines_deleted)
        .bind(&session.integration_phase)
        .bind(session.integration_instance_id)
        .bind(&session.id)
        .execute(&mut *tx)
        .await?;

        // Update instances (delete and re-insert for simplicity)
        sqlx::query("DELETE FROM instances WHERE session_id = ?")
            .bind(&session.id)
            .execute(&mut *tx)
            .await?;

        for instance in &session.instances {
            sqlx::query(
                r#"
                INSERT INTO instances (
                    session_id, instance_id, worktree_path, branch,
                    agent_name, status, tmux_session_id, output,
                    start_time, end_time, files_changed, lines_added,
                    lines_deleted, runtime, model, runtime_label
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&session.id)
            .bind(instance.instance_id)
            .bind(&instance.worktree_path)
            .bind(&instance.branch)
            .bind(&instance.agent_name)
            .bind(&instance.status)
            .bind(&instance.tmux_session_id)
            .bind(&instance.output)
            .bind(&instance.start_time)
            .bind(&instance.end_time)
            .bind(instance.files_changed)
            .bind(instance.lines_added)
            .bind(instance.lines_deleted)
            .bind(&instance.runtime)
            .bind(&instance.model)
            .bind(&instance.runtime_label)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(session)
    }

    async fn delete_session(&self, workspace_hash: &str, session_id: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            "DELETE FROM sessions WHERE workspace_hash = ? AND id = ?"
        )
        .bind(workspace_hash)
        .bind(session_id)
        .execute(self.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(session_id.to_string()));
        }

        Ok(())
    }

    async fn add_chat_message(&self, session_id: &str, message: ChatMessage) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO chat_messages (id, session_id, role, content, timestamp, instance_id)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&message.id)
        .bind(session_id)
        .bind(&message.role)
        .bind(&message.content)
        .bind(&message.timestamp)
        .bind(message.instance_id)
        .execute(self.pool())
        .await?;

        // Update session timestamp
        sqlx::query("UPDATE sessions SET updated_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(session_id)
            .execute(self.pool())
            .await?;

        Ok(())
    }

    async fn update_instance_status(
        &self,
        session_id: &str,
        instance_id: i64,
        new_status: &str,
    ) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            "UPDATE instances SET status = ? WHERE session_id = ? AND instance_id = ?"
        )
        .bind(new_status)
        .bind(session_id)
        .bind(instance_id)
        .execute(self.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Instance {} in session {}",
                instance_id, session_id
            )));
        }

        // Update session timestamp
        sqlx::query("UPDATE sessions SET updated_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(session_id)
            .execute(self.pool())
            .await?;

        Ok(())
    }

    async fn search_sessions(
        &self,
        workspace_hash: &str,
        filters: SessionFilters,
    ) -> Result<Vec<WorktreeSession>, RepositoryError> {
        let mut query = String::from(
            r#"
            SELECT
                id, workspace_hash, session_type as type, task, status,
                created_at, updated_at, completed_at, model,
                timeout_seconds, preserve_worktrees, winner_id,
                runtime_mix, total_duration, total_files_changed,
                total_lines_added, total_lines_deleted,
                integration_phase, integration_instance_id
            FROM sessions
            WHERE workspace_hash = ?
            "#,
        );

        let mut conditions = Vec::new();

        if let Some(types) = &filters.session_type {
            let placeholders = types.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            conditions.push(format!("session_type IN ({})", placeholders));
        }

        if let Some(statuses) = &filters.status {
            let placeholders = statuses.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            conditions.push(format!("status IN ({})", placeholders));
        }

        if filters.date_from.is_some() {
            conditions.push("created_at >= ?".to_string());
        }

        if filters.date_to.is_some() {
            conditions.push("created_at <= ?".to_string());
        }

        if filters.search_query.is_some() {
            conditions.push("(task LIKE ? OR id LIKE ?)".to_string());
        }

        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY updated_at DESC");

        // Note: This is a simplified version. In production, use dynamic query building
        // or a query builder library to handle complex filters safely.

        let sessions = sqlx::query_as::<_, WorktreeSession>(&query)
            .bind(workspace_hash)
            .fetch_all(self.pool())
            .await?;

        // Load instances and messages for each session
        let mut result = Vec::new();
        for mut session in sessions {
            session.instances = self.load_instances(&session.id).await?;
            session.chat_history = self.load_messages(&session.id).await?;
            result.push(session);
        }

        Ok(result)
    }
}
```

### 1.8 Public API (lib.rs)

```rust
// crates/ait42-session/src/lib.rs
//! AIT42 Session Management
//!
//! This crate provides session data persistence using SQLite.

pub mod db;
pub mod models;
pub mod repository;
pub mod migration;

pub use db::DbPool;
pub use models::{ChatMessage, WorktreeInstance, WorktreeSession};
pub use repository::{
    RepositoryError, SessionFilters, SessionRepository, SqliteSessionRepository,
};

// Re-export commonly used types
pub use chrono;
pub use sqlx;
pub use uuid;
```

### 1.9 Unit Tests

```rust
// crates/ait42-session/tests/integration_tests.rs
use ait42_session::*;
use chrono::Utc;
use tempfile::NamedTempFile;
use uuid::Uuid;

#[tokio::test]
async fn test_create_and_get_session() {
    let db_file = NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}", db_file.path().display());

    let repo = SqliteSessionRepository::new(&db_url).await.unwrap();

    let session = WorktreeSession::new(
        Uuid::new_v4().to_string(),
        "test_workspace".to_string(),
        "competition".to_string(),
        "Test task".to_string(),
    );

    let created = repo.create_session(session.clone()).await.unwrap();
    assert_eq!(created.id, session.id);

    let fetched = repo
        .get_session("test_workspace", &session.id)
        .await
        .unwrap();
    assert_eq!(fetched.id, session.id);
    assert_eq!(fetched.task, "Test task");
}

#[tokio::test]
async fn test_get_all_sessions() {
    let db_file = NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}", db_file.path().display());

    let repo = SqliteSessionRepository::new(&db_url).await.unwrap();

    // Create multiple sessions
    for i in 0..5 {
        let session = WorktreeSession::new(
            Uuid::new_v4().to_string(),
            "test_workspace".to_string(),
            "competition".to_string(),
            format!("Task {}", i),
        );
        repo.create_session(session).await.unwrap();
    }

    let sessions = repo.get_all_sessions("test_workspace").await.unwrap();
    assert_eq!(sessions.len(), 5);
}

#[tokio::test]
async fn test_update_session() {
    let db_file = NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}", db_file.path().display());

    let repo = SqliteSessionRepository::new(&db_url).await.unwrap();

    let mut session = WorktreeSession::new(
        Uuid::new_v4().to_string(),
        "test_workspace".to_string(),
        "competition".to_string(),
        "Original task".to_string(),
    );

    repo.create_session(session.clone()).await.unwrap();

    session.task = "Updated task".to_string();
    session.status = "completed".to_string();
    session.updated_at = Utc::now().to_rfc3339();

    let updated = repo.update_session(session.clone()).await.unwrap();
    assert_eq!(updated.task, "Updated task");
    assert_eq!(updated.status, "completed");
}

#[tokio::test]
async fn test_delete_session() {
    let db_file = NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}", db_file.path().display());

    let repo = SqliteSessionRepository::new(&db_url).await.unwrap();

    let session = WorktreeSession::new(
        Uuid::new_v4().to_string(),
        "test_workspace".to_string(),
        "competition".to_string(),
        "Test task".to_string(),
    );

    repo.create_session(session.clone()).await.unwrap();

    repo.delete_session("test_workspace", &session.id)
        .await
        .unwrap();

    let result = repo.get_session("test_workspace", &session.id).await;
    assert!(result.is_err());
}
```

---

## Phase 2: Dual Write Implementation

See main ARCHITECTURE.md document for Phase 2-4 implementation details.

---

## Development Workflow

### Running Tests

```bash
# Run all tests
cd crates/ait42-session
cargo test

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Run specific test
cargo test test_create_and_get_session
```

### Running Migrations

```bash
# Create new migration
sqlx migrate add add_full_text_search

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Generating SQLx Offline Data

```bash
# For CI/CD builds without database
cargo sqlx prepare --workspace
git add sqlx-data.json
```

---

## Troubleshooting

### Issue: "database is locked" error

**Solution**: Increase busy timeout or enable WAL mode

```rust
let options = SqliteConnectOptions::from_str(database_url)?
    .busy_timeout(Duration::from_secs(30))
    .journal_mode(SqliteJournalMode::Wal);
```

### Issue: SQLx macro compilation fails

**Solution**: Ensure database exists at compile time

```bash
sqlx database create
sqlx migrate run
cargo sqlx prepare
```

---

## Next Steps

1. Complete Phase 1 implementation
2. Write comprehensive unit tests
3. Benchmark performance
4. Proceed to Phase 2 (Dual Write)

For questions, contact the backend team lead.
