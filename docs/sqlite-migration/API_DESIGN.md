# SQLite Migration - Data Access API Design

**Version**: 1.0.0
**Date**: 2025-01-13
**Status**: Design Phase

---

## Executive Summary

**API Style**: Tauri Command-based (existing pattern maintained)
**Database**: SQLite with SQLx (compile-time checked queries)
**Migration Strategy**: Gradual migration with backward compatibility layer
**Key Design Decisions**:

1. **Maintain existing Tauri command signatures** - Zero breaking changes for frontend
2. **Use SQLx for type safety** - Compile-time SQL validation prevents runtime errors
3. **Phased migration** - Run JSON and SQLite in parallel during transition
4. **Workspace-scoped sessions** - Continue per-workspace isolation pattern

---

## 1. Existing API Analysis

### Current Architecture (JSON-based)

**Storage Location**: `~/.ait42/sessions/{workspace_hash}.json`

**Data Structures**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeSession {
    pub id: String,
    pub r#type: String,              // competition | ensemble | debate
    pub task: String,
    pub status: String,              // running | completed | failed | paused
    pub created_at: String,          // ISO8601
    pub updated_at: String,          // ISO8601
    pub completed_at: Option<String>,
    pub instances: Vec<WorktreeInstance>,
    pub chat_history: Vec<ChatMessage>,
    pub model: Option<String>,
    pub timeout_seconds: Option<u32>,
    pub preserve_worktrees: Option<bool>,
    pub winner_id: Option<u32>,
    pub runtime_mix: Option<Vec<String>>,
    pub total_duration: Option<u64>,
    pub total_files_changed: Option<u32>,
    pub total_lines_added: Option<u32>,
    pub total_lines_deleted: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub runtime: Option<String>,
    pub model: Option<String>,
    pub runtime_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,                // user | assistant | system
    pub content: String,
    pub timestamp: String,
    pub instance_id: Option<u32>,
}
```

**Existing Tauri Commands**:
```rust
#[tauri::command]
async fn create_session(state: State<'_, AppState>, workspace_path: String, session: WorktreeSession) -> Result<WorktreeSession, String>

#[tauri::command]
async fn update_session(state: State<'_, AppState>, workspace_path: String, session: WorktreeSession) -> Result<WorktreeSession, String>

#[tauri::command]
async fn get_session(state: State<'_, AppState>, workspace_path: String, session_id: String) -> Result<WorktreeSession, String>

#[tauri::command]
async fn get_all_sessions(state: State<'_, AppState>, workspace_path: String) -> Result<Vec<WorktreeSession>, String>

#[tauri::command]
async fn delete_session(state: State<'_, AppState>, workspace_path: String, session_id: String) -> Result<(), String>

#[tauri::command]
async fn add_chat_message(state: State<'_, AppState>, workspace_path: String, session_id: String, message: ChatMessage) -> Result<WorktreeSession, String>

#[tauri::command]
async fn update_instance_status(state: State<'_, AppState>, workspace_path: String, session_id: String, instance_id: u32, new_status: String) -> Result<WorktreeSession, String>
```

**Frontend Integration**:
```typescript
// src/services/tauri.ts
const tauriApi = {
  async createSession(workspacePath: string, session: WorktreeSession): Promise<WorktreeSession>
  async updateSession(workspacePath: string, session: WorktreeSession): Promise<WorktreeSession>
  async getSession(workspacePath: string, sessionId: string): Promise<WorktreeSession>
  async getAllSessions(workspacePath: string): Promise<WorktreeSession[]>
  async deleteSession(workspacePath: string, sessionId: string): Promise<void>
  async addChatMessage(workspacePath: string, sessionId: string, message: ChatMessage): Promise<WorktreeSession>
  async updateInstanceStatus(workspacePath: string, sessionId: string, instanceId: number, newStatus: string): Promise<WorktreeSession>
}

// src/store/sessionHistoryStore.ts - Zustand store
interface SessionHistoryStore {
  sessions: WorktreeSession[];
  activeSessionId: string | null;
  workspacePath: string;
  isLoading: boolean;
  error: string | null;

  loadSessions: () => Promise<void>;
  createSession: (session: WorktreeSession) => Promise<void>;
  updateSession: (session: WorktreeSession) => Promise<void>;
  // ... other actions
}
```

---

## 2. SQLite Schema Design

### Database Location

**Primary**: `~/.ait42/sessions.db` (centralized database)
**Alternative**: `~/.ait42/sessions/{workspace_hash}.db` (per-workspace isolation)

**Recommendation**: Use centralized database with workspace_path indexing for better query performance and easier maintenance.

### Schema Definition

```sql
-- SQLite version: 3.40+
-- Enable WAL mode for concurrent reads
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

-- ===================================
-- Core Tables
-- ===================================

-- Workspaces: Track workspace metadata
CREATE TABLE IF NOT EXISTS workspaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_path TEXT NOT NULL UNIQUE,
    workspace_hash TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_accessed_at TEXT NOT NULL DEFAULT (datetime('now')),

    -- Index for fast workspace lookups
    CHECK(workspace_path <> '')
) STRICT;

CREATE INDEX idx_workspaces_path ON workspaces(workspace_path);
CREATE INDEX idx_workspaces_hash ON workspaces(workspace_hash);

-- Sessions: Main session data
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,                     -- UUID from frontend
    workspace_id INTEGER NOT NULL,
    type TEXT NOT NULL,                      -- competition | ensemble | debate
    task TEXT NOT NULL,
    status TEXT NOT NULL,                    -- running | completed | failed | paused
    created_at TEXT NOT NULL,                -- ISO8601 timestamp
    updated_at TEXT NOT NULL,                -- ISO8601 timestamp
    completed_at TEXT,                       -- ISO8601 timestamp
    model TEXT,
    timeout_seconds INTEGER,
    preserve_worktrees INTEGER,              -- SQLite boolean (0/1)
    winner_id INTEGER,
    total_duration INTEGER,                  -- milliseconds
    total_files_changed INTEGER,
    total_lines_added INTEGER,
    total_lines_deleted INTEGER,

    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    CHECK(type IN ('competition', 'ensemble', 'debate')),
    CHECK(status IN ('running', 'completed', 'failed', 'paused')),
    CHECK(id <> ''),
    CHECK(task <> '')
) STRICT;

CREATE INDEX idx_sessions_workspace ON sessions(workspace_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_type ON sessions(type);
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);
CREATE INDEX idx_sessions_updated_at ON sessions(updated_at DESC);

-- Runtime Mix: Many-to-many relationship for session runtimes
CREATE TABLE IF NOT EXISTS session_runtime_mix (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    runtime TEXT NOT NULL,

    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    UNIQUE(session_id, runtime)
) STRICT;

CREATE INDEX idx_runtime_mix_session ON session_runtime_mix(session_id);

-- Instances: Worktree instances within a session
CREATE TABLE IF NOT EXISTS instances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    instance_id INTEGER NOT NULL,           -- Instance number within session
    worktree_path TEXT NOT NULL,
    branch TEXT NOT NULL,
    agent_name TEXT NOT NULL,
    status TEXT NOT NULL,
    tmux_session_id TEXT NOT NULL,
    output TEXT,
    start_time TEXT,                        -- ISO8601
    end_time TEXT,                          -- ISO8601
    files_changed INTEGER,
    lines_added INTEGER,
    lines_deleted INTEGER,
    runtime TEXT,
    model TEXT,
    runtime_label TEXT,

    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    UNIQUE(session_id, instance_id),
    CHECK(instance_id >= 0)
) STRICT;

CREATE INDEX idx_instances_session ON instances(session_id);
CREATE INDEX idx_instances_status ON instances(status);
CREATE INDEX idx_instances_session_instance ON instances(session_id, instance_id);

-- Chat Messages: Chat history for sessions
CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY,                     -- UUID from frontend
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,                      -- user | assistant | system
    content TEXT NOT NULL,
    timestamp TEXT NOT NULL,                 -- ISO8601
    instance_id INTEGER,                     -- Optional link to instance

    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    CHECK(role IN ('user', 'assistant', 'system')),
    CHECK(id <> ''),
    CHECK(content <> '')
) STRICT;

CREATE INDEX idx_chat_messages_session ON chat_messages(session_id);
CREATE INDEX idx_chat_messages_timestamp ON chat_messages(timestamp);
CREATE INDEX idx_chat_messages_role ON chat_messages(role);

-- ===================================
-- Migration Metadata
-- ===================================

-- Track migration status and version
CREATE TABLE IF NOT EXISTS migration_metadata (
    id INTEGER PRIMARY KEY CHECK (id = 1),   -- Singleton table
    schema_version INTEGER NOT NULL DEFAULT 1,
    migrated_at TEXT NOT NULL DEFAULT (datetime('now')),
    json_backup_path TEXT,
    migration_status TEXT NOT NULL DEFAULT 'pending', -- pending | in_progress | completed | failed
    last_updated_at TEXT NOT NULL DEFAULT (datetime('now'))
) STRICT;

-- Initialize migration metadata
INSERT OR IGNORE INTO migration_metadata (id, schema_version, migration_status)
VALUES (1, 1, 'pending');

-- ===================================
-- Views for Complex Queries
-- ===================================

-- Complete session view with aggregated data
CREATE VIEW IF NOT EXISTS session_complete AS
SELECT
    s.id,
    s.workspace_id,
    w.workspace_path,
    s.type,
    s.task,
    s.status,
    s.created_at,
    s.updated_at,
    s.completed_at,
    s.model,
    s.timeout_seconds,
    s.preserve_worktrees,
    s.winner_id,
    s.total_duration,
    s.total_files_changed,
    s.total_lines_added,
    s.total_lines_deleted,
    COUNT(DISTINCT i.id) as instance_count,
    COUNT(DISTINCT cm.id) as message_count,
    GROUP_CONCAT(DISTINCT srm.runtime) as runtime_mix
FROM sessions s
JOIN workspaces w ON s.workspace_id = w.id
LEFT JOIN instances i ON s.id = i.session_id
LEFT JOIN chat_messages cm ON s.id = cm.session_id
LEFT JOIN session_runtime_mix srm ON s.id = srm.session_id
GROUP BY s.id;
```

### Schema Migrations

**Migration Files**: `src-tauri/migrations/`

```sql
-- migrations/001_initial_schema.sql
-- (Schema from above)

-- migrations/002_add_performance_indexes.sql (Future)
CREATE INDEX IF NOT EXISTS idx_sessions_composite
ON sessions(workspace_id, status, created_at DESC);

-- migrations/003_add_full_text_search.sql (Future)
CREATE VIRTUAL TABLE IF NOT EXISTS sessions_fts
USING fts5(id, task, content='sessions', content_rowid='rowid');
```

---

## 3. Tauri Command Design (SQLite Version)

### Principle: Zero Breaking Changes

**Strategy**: Keep exact same command signatures, only change internal implementation.

### Database Connection Pool

```rust
// src-tauri/src/db/mod.rs
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use sqlx::ConnectOptions;
use std::path::PathBuf;
use std::time::Duration;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Initialize database with connection pool
    pub async fn new() -> Result<Self, sqlx::Error> {
        let db_path = get_database_path()?;

        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_url = format!("sqlite:{}", db_path.display());

        // Configure SQLite options
        let connect_options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(5))
            .disable_statement_logging(); // Enable in debug mode only

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect_with(connect_options)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

fn get_database_path() -> Result<PathBuf, std::io::Error> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Home directory not found"
        ))?;

    Ok(home_dir.join(".ait42").join("sessions.db"))
}
```

### AppState Integration

```rust
// src-tauri/src/state.rs (updated)
use crate::db::Database;

pub struct AppState {
    // ... existing fields ...

    /// SQLite database connection pool
    pub database: Arc<Database>,
}

impl AppState {
    pub async fn new(working_dir: PathBuf) -> anyhow::Result<Self> {
        // ... existing initialization ...

        // Initialize database
        let database = Database::new().await?;

        Ok(Self {
            // ... existing fields ...
            database: Arc::new(database),
        })
    }
}
```

### Command Implementations

```rust
// src-tauri/src/commands/session_history_sqlite.rs
use sqlx::{Row, QueryBuilder};
use tauri::State;
use tracing::{info, warn, error};

use crate::state::AppState;
use crate::db::{models::*, queries};

// ===================================
// Session CRUD Operations
// ===================================

/// Create a new session
#[tauri::command]
pub async fn create_session(
    state: State<'_, AppState>,
    workspace_path: String,
    session: WorktreeSession,
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

    let pool = state.database.pool();

    // Begin transaction for atomicity
    let mut tx = pool.begin().await.map_err(|e| {
        error!("Failed to start transaction: {}", e);
        format!("Database error: {}", e)
    })?;

    // 1. Ensure workspace exists (upsert)
    let workspace_id = queries::upsert_workspace(&mut tx, &workspace_path).await?;

    // 2. Insert session
    queries::insert_session(&mut tx, workspace_id, &session).await?;

    // 3. Insert instances
    for instance in &session.instances {
        queries::insert_instance(&mut tx, &session.id, instance).await?;
    }

    // 4. Insert chat messages
    for message in &session.chat_history {
        queries::insert_chat_message(&mut tx, &session.id, message).await?;
    }

    // 5. Insert runtime mix
    if let Some(runtime_mix) = &session.runtime_mix {
        for runtime in runtime_mix {
            queries::insert_runtime_mix(&mut tx, &session.id, runtime).await?;
        }
    }

    // Commit transaction
    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        format!("Database error: {}", e)
    })?;

    info!("Successfully created session {}", session.id);
    Ok(session)
}

/// Update an existing session
#[tauri::command]
pub async fn update_session(
    state: State<'_, AppState>,
    workspace_path: String,
    session: WorktreeSession,
) -> Result<WorktreeSession, String> {
    info!("Updating session {} for workspace {}", session.id, workspace_path);

    // Validation
    if workspace_path.trim().is_empty() {
        return Err("Workspace path cannot be empty".to_string());
    }

    let pool = state.database.pool();
    let mut tx = pool.begin().await.map_err(|e| format!("Database error: {}", e))?;

    // 1. Verify workspace exists
    let workspace_id = queries::get_workspace_id(&mut tx, &workspace_path).await
        .ok_or_else(|| "Workspace not found".to_string())?;

    // 2. Update session
    queries::update_session(&mut tx, workspace_id, &session).await?;

    // 3. Delete and re-insert instances (simpler than diffing)
    queries::delete_instances(&mut tx, &session.id).await?;
    for instance in &session.instances {
        queries::insert_instance(&mut tx, &session.id, instance).await?;
    }

    // 4. Delete and re-insert chat messages
    queries::delete_chat_messages(&mut tx, &session.id).await?;
    for message in &session.chat_history {
        queries::insert_chat_message(&mut tx, &session.id, message).await?;
    }

    // 5. Delete and re-insert runtime mix
    queries::delete_runtime_mix(&mut tx, &session.id).await?;
    if let Some(runtime_mix) = &session.runtime_mix {
        for runtime in runtime_mix {
            queries::insert_runtime_mix(&mut tx, &session.id, runtime).await?;
        }
    }

    tx.commit().await.map_err(|e| format!("Database error: {}", e))?;

    info!("Successfully updated session {}", session.id);
    Ok(session)
}

/// Get a specific session by ID
#[tauri::command]
pub async fn get_session(
    state: State<'_, AppState>,
    workspace_path: String,
    session_id: String,
) -> Result<WorktreeSession, String> {
    info!("Fetching session {} for workspace {}", session_id, workspace_path);

    if workspace_path.trim().is_empty() {
        return Err("Workspace path cannot be empty".to_string());
    }

    let pool = state.database.pool();

    queries::get_session_by_id(pool, &workspace_path, &session_id)
        .await
        .ok_or_else(|| format!("Session {} not found", session_id))
}

/// Get all sessions for a workspace
#[tauri::command]
pub async fn get_all_sessions(
    state: State<'_, AppState>,
    workspace_path: String,
) -> Result<Vec<WorktreeSession>, String> {
    info!("Fetching all sessions for workspace {}", workspace_path);

    if workspace_path.trim().is_empty() {
        warn!("Empty workspace path - returning empty array");
        return Ok(Vec::new());
    }

    let pool = state.database.pool();

    queries::get_all_sessions(pool, &workspace_path)
        .await
        .map_err(|e| format!("Database error: {}", e))
}

/// Delete a session
#[tauri::command]
pub async fn delete_session(
    state: State<'_, AppState>,
    workspace_path: String,
    session_id: String,
) -> Result<(), String> {
    info!("Deleting session {} for workspace {}", session_id, workspace_path);

    if workspace_path.trim().is_empty() {
        return Err("Workspace path cannot be empty".to_string());
    }

    let pool = state.database.pool();

    queries::delete_session(pool, &workspace_path, &session_id)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    info!("Successfully deleted session {}", session_id);
    Ok(())
}

// ===================================
// Specialized Update Operations
// ===================================

/// Add chat message to a session
#[tauri::command]
pub async fn add_chat_message(
    state: State<'_, AppState>,
    workspace_path: String,
    session_id: String,
    message: ChatMessage,
) -> Result<WorktreeSession, String> {
    info!("Adding chat message to session {}", session_id);

    if workspace_path.trim().is_empty() {
        return Err("Workspace path cannot be empty".to_string());
    }

    let pool = state.database.pool();
    let mut tx = pool.begin().await.map_err(|e| format!("Database error: {}", e))?;

    // 1. Insert chat message
    queries::insert_chat_message(&mut tx, &session_id, &message).await?;

    // 2. Update session's updated_at timestamp
    queries::touch_session(&mut tx, &session_id).await?;

    tx.commit().await.map_err(|e| format!("Database error: {}", e))?;

    // 3. Return updated session
    get_session(state, workspace_path, session_id).await
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
    info!("Updating instance {} status to {} in session {}", instance_id, new_status, session_id);

    if workspace_path.trim().is_empty() {
        return Err("Workspace path cannot be empty".to_string());
    }

    let pool = state.database.pool();
    let mut tx = pool.begin().await.map_err(|e| format!("Database error: {}", e))?;

    // 1. Update instance status
    queries::update_instance_status(&mut tx, &session_id, instance_id, &new_status).await?;

    // 2. Update session's updated_at timestamp
    queries::touch_session(&mut tx, &session_id).await?;

    tx.commit().await.map_err(|e| format!("Database error: {}", e))?;

    // 3. Return updated session
    get_session(state, workspace_path, session_id).await
}

// ===================================
// Migration Operation
// ===================================

/// Migrate existing JSON files to SQLite
#[tauri::command]
pub async fn migrate_from_json(
    state: State<'_, AppState>,
) -> Result<MigrationReport, String> {
    info!("Starting JSON to SQLite migration");

    let pool = state.database.pool();

    crate::db::migration::migrate_json_to_sqlite(pool).await
        .map_err(|e| format!("Migration failed: {}", e))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationReport {
    pub total_workspaces: usize,
    pub total_sessions: usize,
    pub migrated_sessions: usize,
    pub failed_sessions: usize,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}
```

---

## 4. SQLx Query Implementation

### Query Module Structure

```rust
// src-tauri/src/db/queries.rs
use sqlx::{SqliteConnection, Row};
use crate::commands::session_history::*;

// ===================================
// Workspace Queries
// ===================================

/// Upsert workspace and return workspace_id
pub async fn upsert_workspace(
    conn: &mut SqliteConnection,
    workspace_path: &str,
) -> Result<i64, String> {
    let workspace_hash = compute_workspace_hash(workspace_path);

    // Use UPSERT (INSERT ... ON CONFLICT) for atomic operation
    let result = sqlx::query!(
        r#"
        INSERT INTO workspaces (workspace_path, workspace_hash, last_accessed_at)
        VALUES (?1, ?2, datetime('now'))
        ON CONFLICT(workspace_path) DO UPDATE SET
            last_accessed_at = datetime('now')
        RETURNING id
        "#,
        workspace_path,
        workspace_hash
    )
    .fetch_one(conn)
    .await
    .map_err(|e| format!("Failed to upsert workspace: {}", e))?;

    Ok(result.id)
}

/// Get workspace ID by path
pub async fn get_workspace_id(
    conn: &mut SqliteConnection,
    workspace_path: &str,
) -> Option<i64> {
    sqlx::query_scalar!(
        "SELECT id FROM workspaces WHERE workspace_path = ?",
        workspace_path
    )
    .fetch_optional(conn)
    .await
    .ok()
    .flatten()
}

// ===================================
// Session Queries
// ===================================

/// Insert new session
pub async fn insert_session(
    conn: &mut SqliteConnection,
    workspace_id: i64,
    session: &WorktreeSession,
) -> Result<(), String> {
    let preserve_worktrees = session.preserve_worktrees.map(|b| if b { 1 } else { 0 });

    sqlx::query!(
        r#"
        INSERT INTO sessions (
            id, workspace_id, type, task, status,
            created_at, updated_at, completed_at,
            model, timeout_seconds, preserve_worktrees,
            winner_id, total_duration, total_files_changed,
            total_lines_added, total_lines_deleted
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8,
            ?9, ?10, ?11,
            ?12, ?13, ?14,
            ?15, ?16
        )
        "#,
        session.id,
        workspace_id,
        session.r#type,
        session.task,
        session.status,
        session.created_at,
        session.updated_at,
        session.completed_at,
        session.model,
        session.timeout_seconds,
        preserve_worktrees,
        session.winner_id,
        session.total_duration,
        session.total_files_changed,
        session.total_lines_added,
        session.total_lines_deleted
    )
    .execute(conn)
    .await
    .map_err(|e| format!("Failed to insert session: {}", e))?;

    Ok(())
}

/// Update existing session
pub async fn update_session(
    conn: &mut SqliteConnection,
    workspace_id: i64,
    session: &WorktreeSession,
) -> Result<(), String> {
    let preserve_worktrees = session.preserve_worktrees.map(|b| if b { 1 } else { 0 });

    sqlx::query!(
        r#"
        UPDATE sessions SET
            type = ?3, task = ?4, status = ?5,
            updated_at = ?6, completed_at = ?7,
            model = ?8, timeout_seconds = ?9, preserve_worktrees = ?10,
            winner_id = ?11, total_duration = ?12, total_files_changed = ?13,
            total_lines_added = ?14, total_lines_deleted = ?15
        WHERE id = ?1 AND workspace_id = ?2
        "#,
        session.id,
        workspace_id,
        session.r#type,
        session.task,
        session.status,
        session.updated_at,
        session.completed_at,
        session.model,
        session.timeout_seconds,
        preserve_worktrees,
        session.winner_id,
        session.total_duration,
        session.total_files_changed,
        session.total_lines_added,
        session.total_lines_deleted
    )
    .execute(conn)
    .await
    .map_err(|e| format!("Failed to update session: {}", e))?;

    Ok(())
}

/// Get session by ID (with all related data)
pub async fn get_session_by_id(
    pool: &SqlitePool,
    workspace_path: &str,
    session_id: &str,
) -> Option<WorktreeSession> {
    // 1. Get session base data
    let session_row = sqlx::query!(
        r#"
        SELECT s.*
        FROM sessions s
        JOIN workspaces w ON s.workspace_id = w.id
        WHERE s.id = ?1 AND w.workspace_path = ?2
        "#,
        session_id,
        workspace_path
    )
    .fetch_optional(pool)
    .await
    .ok()??;

    // 2. Get instances
    let instances = sqlx::query_as!(
        WorktreeInstance,
        r#"
        SELECT
            instance_id, worktree_path, branch, agent_name, status,
            tmux_session_id, output, start_time, end_time,
            files_changed, lines_added, lines_deleted,
            runtime, model, runtime_label
        FROM instances
        WHERE session_id = ?1
        ORDER BY instance_id
        "#,
        session_id
    )
    .fetch_all(pool)
    .await
    .ok()?;

    // 3. Get chat messages
    let chat_history = sqlx::query_as!(
        ChatMessage,
        r#"
        SELECT id, role, content, timestamp, instance_id
        FROM chat_messages
        WHERE session_id = ?1
        ORDER BY timestamp
        "#,
        session_id
    )
    .fetch_all(pool)
    .await
    .ok()?;

    // 4. Get runtime mix
    let runtime_mix_rows: Vec<String> = sqlx::query_scalar(
        "SELECT runtime FROM session_runtime_mix WHERE session_id = ?1"
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .ok()?;

    let runtime_mix = if runtime_mix_rows.is_empty() {
        None
    } else {
        Some(runtime_mix_rows)
    };

    // 5. Construct WorktreeSession
    Some(WorktreeSession {
        id: session_row.id,
        r#type: session_row.type_,
        task: session_row.task,
        status: session_row.status,
        created_at: session_row.created_at,
        updated_at: session_row.updated_at,
        completed_at: session_row.completed_at,
        instances,
        chat_history,
        model: session_row.model,
        timeout_seconds: session_row.timeout_seconds.map(|v| v as u32),
        preserve_worktrees: session_row.preserve_worktrees.map(|v| v != 0),
        winner_id: session_row.winner_id.map(|v| v as u32),
        runtime_mix,
        total_duration: session_row.total_duration.map(|v| v as u64),
        total_files_changed: session_row.total_files_changed.map(|v| v as u32),
        total_lines_added: session_row.total_lines_added.map(|v| v as u32),
        total_lines_deleted: session_row.total_lines_deleted.map(|v| v as u32),
    })
}

/// Get all sessions for a workspace
pub async fn get_all_sessions(
    pool: &SqlitePool,
    workspace_path: &str,
) -> Result<Vec<WorktreeSession>, sqlx::Error> {
    // 1. Get all session IDs for workspace
    let session_ids: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT s.id
        FROM sessions s
        JOIN workspaces w ON s.workspace_id = w.id
        WHERE w.workspace_path = ?1
        ORDER BY s.updated_at DESC
        "#
    )
    .bind(workspace_path)
    .fetch_all(pool)
    .await?;

    // 2. Fetch each session with all related data
    let mut sessions = Vec::new();
    for session_id in session_ids {
        if let Some(session) = get_session_by_id(pool, workspace_path, &session_id).await {
            sessions.push(session);
        }
    }

    Ok(sessions)
}

/// Delete session and all related data (CASCADE handles this)
pub async fn delete_session(
    pool: &SqlitePool,
    workspace_path: &str,
    session_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM sessions
        WHERE id = ?1 AND workspace_id IN (
            SELECT id FROM workspaces WHERE workspace_path = ?2
        )
        "#,
        session_id,
        workspace_path
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ===================================
// Instance Queries
// ===================================

pub async fn insert_instance(
    conn: &mut SqliteConnection,
    session_id: &str,
    instance: &WorktreeInstance,
) -> Result<(), String> {
    sqlx::query!(
        r#"
        INSERT INTO instances (
            session_id, instance_id, worktree_path, branch,
            agent_name, status, tmux_session_id, output,
            start_time, end_time, files_changed, lines_added,
            lines_deleted, runtime, model, runtime_label
        ) VALUES (
            ?1, ?2, ?3, ?4,
            ?5, ?6, ?7, ?8,
            ?9, ?10, ?11, ?12,
            ?13, ?14, ?15, ?16
        )
        "#,
        session_id,
        instance.instance_id,
        instance.worktree_path,
        instance.branch,
        instance.agent_name,
        instance.status,
        instance.tmux_session_id,
        instance.output,
        instance.start_time,
        instance.end_time,
        instance.files_changed,
        instance.lines_added,
        instance.lines_deleted,
        instance.runtime,
        instance.model,
        instance.runtime_label
    )
    .execute(conn)
    .await
    .map_err(|e| format!("Failed to insert instance: {}", e))?;

    Ok(())
}

pub async fn delete_instances(
    conn: &mut SqliteConnection,
    session_id: &str,
) -> Result<(), String> {
    sqlx::query!("DELETE FROM instances WHERE session_id = ?", session_id)
        .execute(conn)
        .await
        .map_err(|e| format!("Failed to delete instances: {}", e))?;

    Ok(())
}

pub async fn update_instance_status(
    conn: &mut SqliteConnection,
    session_id: &str,
    instance_id: u32,
    new_status: &str,
) -> Result<(), String> {
    let rows_affected = sqlx::query!(
        r#"
        UPDATE instances
        SET status = ?3
        WHERE session_id = ?1 AND instance_id = ?2
        "#,
        session_id,
        instance_id,
        new_status
    )
    .execute(conn)
    .await
    .map_err(|e| format!("Failed to update instance status: {}", e))?
    .rows_affected();

    if rows_affected == 0 {
        return Err(format!("Instance {} not found in session {}", instance_id, session_id));
    }

    Ok(())
}

// ===================================
// Chat Message Queries
// ===================================

pub async fn insert_chat_message(
    conn: &mut SqliteConnection,
    session_id: &str,
    message: &ChatMessage,
) -> Result<(), String> {
    sqlx::query!(
        r#"
        INSERT INTO chat_messages (id, session_id, role, content, timestamp, instance_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        message.id,
        session_id,
        message.role,
        message.content,
        message.timestamp,
        message.instance_id
    )
    .execute(conn)
    .await
    .map_err(|e| format!("Failed to insert chat message: {}", e))?;

    Ok(())
}

pub async fn delete_chat_messages(
    conn: &mut SqliteConnection,
    session_id: &str,
) -> Result<(), String> {
    sqlx::query!("DELETE FROM chat_messages WHERE session_id = ?", session_id)
        .execute(conn)
        .await
        .map_err(|e| format!("Failed to delete chat messages: {}", e))?;

    Ok(())
}

// ===================================
// Runtime Mix Queries
// ===================================

pub async fn insert_runtime_mix(
    conn: &mut SqliteConnection,
    session_id: &str,
    runtime: &str,
) -> Result<(), String> {
    sqlx::query!(
        "INSERT INTO session_runtime_mix (session_id, runtime) VALUES (?1, ?2)",
        session_id,
        runtime
    )
    .execute(conn)
    .await
    .map_err(|e| format!("Failed to insert runtime mix: {}", e))?;

    Ok(())
}

pub async fn delete_runtime_mix(
    conn: &mut SqliteConnection,
    session_id: &str,
) -> Result<(), String> {
    sqlx::query!("DELETE FROM session_runtime_mix WHERE session_id = ?", session_id)
        .execute(conn)
        .await
        .map_err(|e| format!("Failed to delete runtime mix: {}", e))?;

    Ok(())
}

// ===================================
// Utility Queries
// ===================================

/// Update session's updated_at timestamp
pub async fn touch_session(
    conn: &mut SqliteConnection,
    session_id: &str,
) -> Result<(), String> {
    sqlx::query!(
        "UPDATE sessions SET updated_at = datetime('now') WHERE id = ?",
        session_id
    )
    .execute(conn)
    .await
    .map_err(|e| format!("Failed to touch session: {}", e))?;

    Ok(())
}

// ===================================
// Helper Functions
// ===================================

fn compute_workspace_hash(workspace_path: &str) -> String {
    use sha2::{Digest, Sha256};

    let normalized_path = std::fs::canonicalize(workspace_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| workspace_path.trim_end_matches('/').to_string());

    let mut hasher = Sha256::new();
    hasher.update(normalized_path.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}
```

---

## 5. Error Handling Strategy

### Error Types

```rust
// src-tauri/src/db/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(#[from] sqlx::Error),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Instance not found: session={0}, instance_id={1}")]
    InstanceNotFound(String, u32),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<DatabaseError> for String {
    fn from(err: DatabaseError) -> Self {
        err.to_string()
    }
}
```

### Error Handling Patterns

```rust
// Pattern 1: Transaction rollback on error
pub async fn create_session_safe(
    pool: &SqlitePool,
    workspace_path: &str,
    session: &WorktreeSession,
) -> Result<(), DatabaseError> {
    let mut tx = pool.begin().await?;

    match perform_operations(&mut tx, workspace_path, session).await {
        Ok(_) => {
            tx.commit().await?;
            Ok(())
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}

// Pattern 2: Graceful degradation
pub async fn get_all_sessions_safe(
    pool: &SqlitePool,
    workspace_path: &str,
) -> Vec<WorktreeSession> {
    match get_all_sessions(pool, workspace_path).await {
        Ok(sessions) => sessions,
        Err(e) => {
            tracing::error!("Failed to fetch sessions: {}", e);
            Vec::new() // Return empty array instead of error
        }
    }
}

// Pattern 3: Retry with backoff
use tokio::time::{sleep, Duration};

pub async fn execute_with_retry<F, T>(
    operation: F,
    max_retries: u32,
) -> Result<T, DatabaseError>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, sqlx::Error>> + Send>>,
{
    let mut attempts = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts >= max_retries {
                    return Err(DatabaseError::ConnectionFailed(e));
                }

                // Exponential backoff: 100ms, 200ms, 400ms, 800ms
                let delay = Duration::from_millis(100 * (2_u64.pow(attempts - 1)));
                tracing::warn!("Database operation failed (attempt {}/{}), retrying in {:?}", attempts, max_retries, delay);
                sleep(delay).await;
            }
        }
    }
}
```

---

## 6. Frontend Integration

### TypeScript Type Definitions

**No changes required** - existing types remain the same:

```typescript
// src/types/worktree.ts (unchanged)
export interface WorktreeSession {
  id: string;
  type: 'competition' | 'ensemble' | 'debate';
  task: string;
  status: 'running' | 'completed' | 'failed' | 'paused';
  createdAt: string;
  updatedAt: string;
  completedAt?: string;
  instances: WorktreeInstance[];
  chatHistory: ChatMessage[];
  model?: string;
  timeoutSeconds?: number;
  preserveWorktrees?: boolean;
  winnerId?: number;
  runtimeMix?: string[];
  totalDuration?: number;
  totalFilesChanged?: number;
  totalLinesAdded?: number;
  totalLinesDeleted?: number;
}

export interface WorktreeInstance {
  instanceId: number;
  worktreePath: string;
  branch: string;
  agentName: string;
  status: string;
  tmuxSessionId: string;
  output?: string;
  startTime?: string;
  endTime?: string;
  filesChanged?: number;
  linesAdded?: number;
  linesDeleted?: number;
  runtime?: string;
  model?: string;
  runtimeLabel?: string;
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  instanceId?: number;
}
```

### Zustand Store Integration

**No changes required** - store continues to work as-is:

```typescript
// src/store/sessionHistoryStore.ts (unchanged)
export const useSessionHistoryStore = create<SessionHistoryStore>((set, get) => ({
  // ... existing implementation ...

  // All methods remain unchanged
  loadSessions: async () => {
    const { workspacePath } = get();
    if (!workspacePath) return;

    set({ isLoading: true, error: null });

    try {
      // Same API call - backend implementation changed to SQLite
      const sessions = await tauriApi.getAllSessions(workspacePath);
      set({ sessions, isLoading: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to load sessions',
        isLoading: false,
      });
    }
  },

  // ... other methods unchanged ...
}));
```

### Compatibility Layer (Optional)

For maximum safety during migration, implement a feature flag:

```rust
// src-tauri/src/commands/mod.rs
#[cfg(feature = "sqlite")]
pub use session_history_sqlite::*;

#[cfg(not(feature = "sqlite"))]
pub use session_history_json::*;
```

```toml
# Cargo.toml
[features]
default = ["sqlite"]
sqlite = ["sqlx"]
json-storage = [] # Legacy JSON storage
```

---

## 7. Migration Strategy

### Phase 1: Preparation (Week 1)

**Tasks**:
1. Create database schema and migrations
2. Implement SQLx queries with tests
3. Add database initialization to AppState
4. Create migration tool for JSON → SQLite

**Testing**:
- Unit tests for each query function
- Integration tests with test database
- Benchmark queries vs JSON file I/O

**Success Criteria**:
- All queries pass unit tests
- Migration tool successfully converts sample data
- Performance >= JSON baseline

### Phase 2: Parallel Operation (Week 2-3)

**Strategy**: Run both storage systems in parallel

```rust
// src-tauri/src/commands/session_history_hybrid.rs
#[tauri::command]
pub async fn create_session(
    state: State<'_, AppState>,
    workspace_path: String,
    session: WorktreeSession,
) -> Result<WorktreeSession, String> {
    // Write to SQLite (primary)
    let sqlite_result = session_history_sqlite::create_session(
        state.clone(),
        workspace_path.clone(),
        session.clone()
    ).await;

    // Also write to JSON (backup)
    let json_result = session_history_json::create_session(
        state.clone(),
        workspace_path.clone(),
        session.clone()
    ).await;

    match (sqlite_result, json_result) {
        (Ok(session), Ok(_)) => {
            tracing::info!("Session saved to both SQLite and JSON");
            Ok(session)
        }
        (Ok(session), Err(json_err)) => {
            tracing::warn!("JSON save failed but SQLite succeeded: {}", json_err);
            Ok(session)
        }
        (Err(sqlite_err), Ok(_)) => {
            tracing::error!("SQLite save failed: {}", sqlite_err);
            Err(sqlite_err)
        }
        (Err(sqlite_err), Err(_)) => {
            Err(sqlite_err)
        }
    }
}
```

**Validation**:
- Compare SQLite vs JSON results on every read
- Log discrepancies for investigation
- Automatic data consistency checks

### Phase 3: SQLite Primary (Week 4)

**Cutover**:
1. Feature flag: Switch default to SQLite
2. JSON becomes read-only fallback
3. Monitor error rates and performance

```rust
#[tauri::command]
pub async fn get_all_sessions(
    state: State<'_, AppState>,
    workspace_path: String,
) -> Result<Vec<WorktreeSession>, String> {
    // Primary: Read from SQLite
    let sqlite_result = session_history_sqlite::get_all_sessions(
        state.clone(),
        workspace_path.clone()
    ).await;

    match sqlite_result {
        Ok(sessions) => Ok(sessions),
        Err(e) => {
            tracing::error!("SQLite read failed, falling back to JSON: {}", e);

            // Fallback: Read from JSON
            session_history_json::get_all_sessions(state, workspace_path).await
        }
    }
}
```

### Phase 4: JSON Deprecation (Week 5+)

**Cleanup**:
1. Remove JSON write operations
2. Add JSON import tool for legacy data
3. Remove JSON dependencies from codebase

**Migration Tool**:
```rust
#[tauri::command]
pub async fn import_json_sessions(
    state: State<'_, AppState>,
) -> Result<MigrationReport, String> {
    let sessions_dir = dirs::home_dir()
        .ok_or("Home directory not found")?
        .join(".ait42")
        .join("sessions");

    let mut report = MigrationReport {
        total_workspaces: 0,
        total_sessions: 0,
        migrated_sessions: 0,
        failed_sessions: 0,
        duration_ms: 0,
        errors: Vec::new(),
    };

    let start_time = std::time::Instant::now();

    // Iterate through all JSON files
    for entry in std::fs::read_dir(&sessions_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        report.total_workspaces += 1;

        // Read JSON file
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;

        let sessions: Vec<WorktreeSession> = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(e) => {
                report.errors.push(format!("Failed to parse {:?}: {}", path, e));
                continue;
            }
        };

        report.total_sessions += sessions.len();

        // Extract workspace_path from filename (reverse hash lookup not possible)
        // Need to store workspace_path mapping or prompt user

        for session in sessions {
            match import_session(&state, &session).await {
                Ok(_) => report.migrated_sessions += 1,
                Err(e) => {
                    report.failed_sessions += 1;
                    report.errors.push(format!("Failed to import session {}: {}", session.id, e));
                }
            }
        }
    }

    report.duration_ms = start_time.elapsed().as_millis() as u64;
    Ok(report)
}
```

---

## 8. Testing Strategy

### Unit Tests

```rust
// src-tauri/src/db/queries_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .unwrap();

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_upsert_workspace() {
        let pool = setup_test_db().await;
        let mut conn = pool.acquire().await.unwrap();

        // First insert
        let id1 = upsert_workspace(&mut conn, "/test/workspace").await.unwrap();
        assert!(id1 > 0);

        // Second insert (should return same ID)
        let id2 = upsert_workspace(&mut conn, "/test/workspace").await.unwrap();
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_create_and_get_session() {
        let pool = setup_test_db().await;
        let mut conn = pool.acquire().await.unwrap();

        // Create workspace
        let workspace_id = upsert_workspace(&mut conn, "/test/workspace").await.unwrap();

        // Create session
        let session = WorktreeSession {
            id: "test-session-123".to_string(),
            r#type: "competition".to_string(),
            task: "Test task".to_string(),
            status: "running".to_string(),
            created_at: "2025-01-13T10:00:00Z".to_string(),
            updated_at: "2025-01-13T10:00:00Z".to_string(),
            completed_at: None,
            instances: vec![],
            chat_history: vec![],
            model: Some("claude-3.5".to_string()),
            timeout_seconds: Some(3600),
            preserve_worktrees: Some(false),
            winner_id: None,
            runtime_mix: Some(vec!["claude-code".to_string()]),
            total_duration: None,
            total_files_changed: None,
            total_lines_added: None,
            total_lines_deleted: None,
        };

        insert_session(&mut conn, workspace_id, &session).await.unwrap();

        // Retrieve session
        let retrieved = get_session_by_id(&pool, "/test/workspace", "test-session-123")
            .await
            .unwrap();

        assert_eq!(retrieved.id, session.id);
        assert_eq!(retrieved.task, session.task);
        assert_eq!(retrieved.status, session.status);
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        let pool = setup_test_db().await;
        let mut tx = pool.begin().await.unwrap();

        let workspace_id = upsert_workspace(&mut tx, "/test/workspace").await.unwrap();

        let session = create_test_session("test-session-rollback");
        insert_session(&mut tx, workspace_id, &session).await.unwrap();

        // Rollback transaction
        tx.rollback().await.unwrap();

        // Session should not exist
        let retrieved = get_session_by_id(&pool, "/test/workspace", "test-session-rollback").await;
        assert!(retrieved.is_none());
    }

    fn create_test_session(id: &str) -> WorktreeSession {
        WorktreeSession {
            id: id.to_string(),
            r#type: "competition".to_string(),
            task: "Test".to_string(),
            status: "running".to_string(),
            created_at: "2025-01-13T10:00:00Z".to_string(),
            updated_at: "2025-01-13T10:00:00Z".to_string(),
            completed_at: None,
            instances: vec![],
            chat_history: vec![],
            model: None,
            timeout_seconds: None,
            preserve_worktrees: None,
            winner_id: None,
            runtime_mix: None,
            total_duration: None,
            total_files_changed: None,
            total_lines_added: None,
            total_lines_deleted: None,
        }
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs
use ait42_editor::*;

#[tokio::test]
async fn test_full_session_lifecycle() {
    let state = setup_test_app_state().await;

    let workspace_path = "/tmp/test-workspace".to_string();

    // 1. Create session
    let session = create_test_session("integration-test-1");
    let created = create_session(
        State::from(&state),
        workspace_path.clone(),
        session.clone()
    ).await.unwrap();

    assert_eq!(created.id, session.id);

    // 2. Get session
    let retrieved = get_session(
        State::from(&state),
        workspace_path.clone(),
        session.id.clone()
    ).await.unwrap();

    assert_eq!(retrieved.id, session.id);

    // 3. Update session
    let mut updated = retrieved.clone();
    updated.status = "completed".to_string();

    update_session(
        State::from(&state),
        workspace_path.clone(),
        updated.clone()
    ).await.unwrap();

    // 4. Get all sessions
    let all_sessions = get_all_sessions(
        State::from(&state),
        workspace_path.clone()
    ).await.unwrap();

    assert_eq!(all_sessions.len(), 1);
    assert_eq!(all_sessions[0].status, "completed");

    // 5. Delete session
    delete_session(
        State::from(&state),
        workspace_path.clone(),
        session.id.clone()
    ).await.unwrap();

    // 6. Verify deleted
    let deleted = get_session(
        State::from(&state),
        workspace_path.clone(),
        session.id
    ).await;

    assert!(deleted.is_err());
}
```

### Performance Tests

```rust
#[tokio::test]
async fn benchmark_session_operations() {
    let state = setup_test_app_state().await;
    let workspace_path = "/tmp/benchmark-workspace".to_string();

    // Create 1000 sessions
    let start = std::time::Instant::now();
    for i in 0..1000 {
        let session = create_test_session(&format!("bench-session-{}", i));
        create_session(
            State::from(&state),
            workspace_path.clone(),
            session
        ).await.unwrap();
    }
    let create_duration = start.elapsed();

    println!("Created 1000 sessions in {:?} ({:.2} ms/session)",
        create_duration,
        create_duration.as_millis() as f64 / 1000.0
    );

    // Read all sessions
    let start = std::time::Instant::now();
    let sessions = get_all_sessions(
        State::from(&state),
        workspace_path.clone()
    ).await.unwrap();
    let read_duration = start.elapsed();

    assert_eq!(sessions.len(), 1000);
    println!("Read 1000 sessions in {:?}", read_duration);
}
```

---

## 9. Performance Considerations

### Query Optimization

**1. Indexes**: Already defined in schema for common query patterns

**2. Connection Pooling**: Use 5 connections (balance between concurrency and overhead)

**3. Prepared Statements**: SQLx automatically caches prepared statements

**4. Batch Operations**: Use transactions for multiple inserts

```rust
// Efficient batch insert
pub async fn batch_create_sessions(
    pool: &SqlitePool,
    workspace_path: &str,
    sessions: Vec<WorktreeSession>,
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let workspace_id = upsert_workspace(&mut tx, workspace_path).await?;

    for session in sessions {
        insert_session(&mut tx, workspace_id, &session).await?;

        for instance in &session.instances {
            insert_instance(&mut tx, &session.id, instance).await?;
        }

        for message in &session.chat_history {
            insert_chat_message(&mut tx, &session.id, message).await?;
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}
```

### WAL Mode Benefits

**Write-Ahead Logging** (enabled in schema):
- Concurrent reads while writing
- Better performance for small transactions
- Automatic checkpointing

### Expected Performance

| Operation | JSON (baseline) | SQLite (estimated) | Improvement |
|-----------|----------------|-------------------|-------------|
| Create session | 5-10ms | 2-5ms | 2x faster |
| Get single session | 3-8ms | 1-3ms | 2-3x faster |
| Get all sessions (10) | 10-30ms | 5-15ms | 2x faster |
| Get all sessions (100) | 100-300ms | 20-50ms | 5-6x faster |
| Update session | 10-20ms | 3-8ms | 2-3x faster |
| Delete session | 5-15ms | 2-5ms | 2-3x faster |

---

## 10. Rollback Plan

### If Critical Issues Occur

**Step 1**: Disable SQLite feature flag
```toml
# Cargo.toml
[features]
default = ["json-storage"] # Revert to JSON
```

**Step 2**: Redeploy with JSON backend

**Step 3**: Investigate SQLite issues offline

**Step 4**: Fix and re-test before re-enabling

### Data Recovery

**JSON backups maintained** during Phase 2 (parallel operation)

**SQLite database backed up** before each migration phase:
```bash
# Backup command
sqlite3 ~/.ait42/sessions.db ".backup ~/.ait42/sessions-backup-$(date +%Y%m%d).db"
```

---

## Next Steps

### Immediate Actions

1. **Week 1**: Schema design review → Implementation of database module
2. **Week 2**: Query implementation → Unit tests → Integration tests
3. **Week 3**: Parallel operation deployment → Validation
4. **Week 4**: SQLite primary cutover → Monitoring
5. **Week 5+**: JSON deprecation → Cleanup

### Delegation

- **backend-developer**: Implement database module, queries, error handling
- **integration-tester**: Create comprehensive test suite, performance benchmarks
- **devops-engineer**: Database backup strategy, monitoring alerts
- **frontend-developer**: Monitor for API compatibility issues during migration

### Success Metrics

- **0 breaking changes** for frontend
- **≥2x performance improvement** on read operations
- **≥90% data integrity** during parallel operation phase
- **<0.1% error rate** in production after cutover

---

## Appendix: File Structure

```
src-tauri/
├── src/
│   ├── db/
│   │   ├── mod.rs              # Database module exports
│   │   ├── connection.rs       # Connection pool management
│   │   ├── queries.rs          # SQLx query functions
│   │   ├── models.rs           # Database model types
│   │   ├── migration.rs        # JSON → SQLite migration logic
│   │   └── error.rs            # Database error types
│   ├── commands/
│   │   ├── session_history.rs  # Original JSON implementation (legacy)
│   │   ├── session_history_sqlite.rs  # New SQLite implementation
│   │   └── session_history_hybrid.rs  # Parallel operation (Phase 2)
│   ├── state.rs                # AppState with Database integration
│   └── main.rs                 # Main application entry
├── migrations/
│   ├── 001_initial_schema.sql
│   ├── 002_add_performance_indexes.sql
│   └── 003_add_full_text_search.sql
├── Cargo.toml                  # Add sqlx dependencies
└── tests/
    ├── integration_test.rs
    └── performance_test.rs

docs/
└── sqlite-migration/
    ├── API_DESIGN.md           # This document
    ├── MIGRATION_PLAN.md       # Detailed migration execution plan
    └── TESTING_GUIDE.md        # Testing procedures and checklists
```

---

**Document Version**: 1.0.0
**Last Updated**: 2025-01-13
**Next Review**: 2025-01-20 (after Phase 1 completion)
