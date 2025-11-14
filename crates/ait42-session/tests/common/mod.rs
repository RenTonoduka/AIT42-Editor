/// Common test utilities and helpers for ait42-session tests
///
/// This module provides:
/// - In-memory database setup
/// - Test data factories
/// - Assertion helpers
/// - Cleanup utilities

use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use std::time::Duration;

/// Create an in-memory SQLite pool for testing
///
/// This creates a fresh database for each test to ensure isolation.
/// Uses WAL mode and enables foreign keys for realistic behavior.
pub async fn create_test_pool() -> SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(3))
        .connect_with(options)
        .await
        .expect("Failed to create test pool")
}

/// Setup test database with schema migrations
pub async fn setup_test_db(pool: &SqlitePool) {
    // Create workspaces table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS workspaces (
            hash TEXT PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            last_accessed TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create workspaces table");

    // Create sessions table
    sqlx::query(
        r#"
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
            preserve_worktrees INTEGER,
            winner_id INTEGER,
            runtime_mix TEXT,
            total_duration INTEGER,
            total_files_changed INTEGER,
            total_lines_added INTEGER,
            total_lines_deleted INTEGER,
            integration_phase TEXT CHECK(integration_phase IN ('pending', 'in_progress', 'completed')),
            integration_instance_id INTEGER,
            FOREIGN KEY (workspace_hash) REFERENCES workspaces(hash) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create sessions table");

    // Create indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_workspace ON sessions(workspace_hash)")
        .execute(pool)
        .await
        .expect("Failed to create workspace index");

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status)")
        .execute(pool)
        .await
        .expect("Failed to create status index");

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_type ON sessions(session_type)")
        .execute(pool)
        .await
        .expect("Failed to create type index");

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_created ON sessions(created_at DESC)")
        .execute(pool)
        .await
        .expect("Failed to create created_at index");

    // Create instances table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS instances (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            instance_id INTEGER NOT NULL,
            worktree_path TEXT NOT NULL,
            branch TEXT NOT NULL,
            agent_name TEXT NOT NULL,
            status TEXT NOT NULL CHECK(status IN ('idle', 'running', 'completed', 'failed', 'paused', 'archived')),
            tmux_session_id TEXT NOT NULL,
            output TEXT,
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
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create instances table");

    // Create chat_messages table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS chat_messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            instance_id INTEGER,
            FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create chat_messages table");

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_session ON chat_messages(session_id)")
        .execute(pool)
        .await
        .expect("Failed to create messages session index");
}

/// Test data factory for WorktreeSession
pub mod factories {
    use chrono::Utc;
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    pub struct WorktreeSession {
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
        pub preserve_worktrees: Option<bool>,
        pub winner_id: Option<i64>,
        pub runtime_mix: Option<String>,
        pub total_duration: Option<i64>,
        pub total_files_changed: Option<i64>,
        pub total_lines_added: Option<i64>,
        pub total_lines_deleted: Option<i64>,
        pub integration_phase: Option<String>,
        pub integration_instance_id: Option<i64>,
    }

    impl WorktreeSession {
        /// Create a default test session
        pub fn default() -> Self {
            let now = Utc::now().to_rfc3339();
            Self {
                id: Uuid::new_v4().to_string(),
                workspace_hash: "workspace_1".to_string(),
                session_type: "competition".to_string(),
                task: "Test task".to_string(),
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
            }
        }

        /// Create a session with custom ID
        pub fn with_id(id: &str) -> Self {
            let mut session = Self::default();
            session.id = id.to_string();
            session
        }

        /// Create a session with custom type
        pub fn with_type(session_type: &str) -> Self {
            let mut session = Self::default();
            session.session_type = session_type.to_string();
            session
        }

        /// Create a session with custom status
        pub fn with_status(status: &str) -> Self {
            let mut session = Self::default();
            session.status = status.to_string();
            session
        }

        /// Create a session with custom workspace
        pub fn with_workspace(workspace_hash: &str) -> Self {
            let mut session = Self::default();
            session.workspace_hash = workspace_hash.to_string();
            session
        }

        /// Create a completed session
        pub fn completed() -> Self {
            let mut session = Self::default();
            session.status = "completed".to_string();
            session.completed_at = Some(Utc::now().to_rfc3339());
            session.total_duration = Some(3600);
            session.total_files_changed = Some(10);
            session.total_lines_added = Some(150);
            session.total_lines_deleted = Some(50);
            session
        }

        /// Create N test sessions
        pub fn create_many(count: usize) -> Vec<Self> {
            (0..count).map(|i| {
                let mut session = Self::default();
                session.task = format!("Test task {}", i);
                session
            }).collect()
        }
    }

    #[derive(Debug, Clone)]
    pub struct WorktreeInstance {
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

    impl WorktreeInstance {
        pub fn default(session_id: &str, instance_id: i64) -> Self {
            Self {
                session_id: session_id.to_string(),
                instance_id,
                worktree_path: format!("/tmp/worktree_{}", instance_id),
                branch: format!("branch_{}", instance_id),
                agent_name: format!("agent_{}", instance_id),
                status: "running".to_string(),
                tmux_session_id: format!("tmux_{}", instance_id),
                output: None,
                start_time: Some(Utc::now().to_rfc3339()),
                end_time: None,
                files_changed: None,
                lines_added: None,
                lines_deleted: None,
                runtime: Some("claude".to_string()),
                model: Some("claude-3-5-sonnet-20241022".to_string()),
                runtime_label: Some("claude_default".to_string()),
            }
        }

        pub fn create_many(session_id: &str, count: usize) -> Vec<Self> {
            (0..count).map(|i| Self::default(session_id, i as i64)).collect()
        }
    }

    #[derive(Debug, Clone)]
    pub struct ChatMessage {
        pub id: String,
        pub session_id: String,
        pub role: String,
        pub content: String,
        pub timestamp: String,
        pub instance_id: Option<i64>,
    }

    impl ChatMessage {
        pub fn default(session_id: &str) -> Self {
            Self {
                id: Uuid::new_v4().to_string(),
                session_id: session_id.to_string(),
                role: "user".to_string(),
                content: "Test message".to_string(),
                timestamp: Utc::now().to_rfc3339(),
                instance_id: None,
            }
        }

        pub fn assistant(session_id: &str, content: &str) -> Self {
            let mut msg = Self::default(session_id);
            msg.role = "assistant".to_string();
            msg.content = content.to_string();
            msg
        }

        pub fn create_many(session_id: &str, count: usize) -> Vec<Self> {
            (0..count).map(|i| {
                let mut msg = Self::default(session_id);
                msg.content = format!("Message {}", i);
                msg
            }).collect()
        }
    }
}

/// Helper to insert a test workspace
pub async fn insert_test_workspace(pool: &SqlitePool, hash: &str, path: &str) {
    sqlx::query(
        "INSERT INTO workspaces (hash, path, last_accessed) VALUES (?, ?, ?)"
    )
    .bind(hash)
    .bind(path)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .expect("Failed to insert test workspace");
}

/// Helper to count sessions
pub async fn count_sessions(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions")
        .fetch_one(pool)
        .await
        .expect("Failed to count sessions")
}

/// Helper to count instances
pub async fn count_instances(pool: &SqlitePool, session_id: &str) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM instances WHERE session_id = ?")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("Failed to count instances")
}

/// Helper to count messages
pub async fn count_messages(pool: &SqlitePool, session_id: &str) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM chat_messages WHERE session_id = ?")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("Failed to count messages")
}

/// Cleanup test database (optional, in-memory DBs are auto-cleaned)
pub async fn cleanup_test_db(pool: &SqlitePool) {
    let _ = sqlx::query("DELETE FROM chat_messages").execute(pool).await;
    let _ = sqlx::query("DELETE FROM instances").execute(pool).await;
    let _ = sqlx::query("DELETE FROM sessions").execute(pool).await;
    let _ = sqlx::query("DELETE FROM workspaces").execute(pool).await;
}
