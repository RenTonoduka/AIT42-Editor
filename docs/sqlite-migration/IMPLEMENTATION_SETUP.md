# SQLite Migration - Implementation Setup Guide

**Version**: 1.0.0
**Date**: 2025-01-13

This document provides step-by-step instructions for setting up the SQLite migration implementation.

---

## 1. Cargo Dependencies

### Add SQLx to Cargo.toml

```toml
# src-tauri/Cargo.toml

[dependencies]
# Existing dependencies...
sqlx = { version = "0.7", features = [
    "runtime-tokio-rustls",  # Async runtime with TLS
    "sqlite",                # SQLite driver
    "macros",                # Compile-time query checking
    "migrate",               # Migration support
] }
sha2 = "0.10"               # For workspace hashing (already present)
chrono = "0.4"              # For timestamp handling (already present)
thiserror = "1.0"           # For error handling

# Development dependencies
[dev-dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "macros", "migrate"] }
```

### Feature Flags

```toml
[features]
default = ["sqlite"]
sqlite = ["sqlx"]           # SQLite storage (new)
json-storage = []           # Legacy JSON storage (fallback)
terminal = ["ait42-tui"]    # Existing feature
```

---

## 2. SQLx CLI Setup

### Install SQLx CLI

```bash
# Install SQLx CLI for migration management
cargo install sqlx-cli --no-default-features --features sqlite
```

### Create Database URL Environment Variable

```bash
# Add to .env (create if not exists)
echo "DATABASE_URL=sqlite:$HOME/.ait42/sessions.db" > src-tauri/.env
```

### Initialize Database

```bash
cd src-tauri

# Create migrations directory
sqlx migrate add initial_schema

# This creates: migrations/<timestamp>_initial_schema.sql
```

---

## 3. Migration Files Setup

### Create Migration: Initial Schema

```bash
# migrations/20250113000001_initial_schema.sql
# Copy schema from API_DESIGN.md section 2
```

**Full migration file**:

```sql
-- migrations/20250113000001_initial_schema.sql
-- SQLite version: 3.40+

PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

-- ===================================
-- Core Tables
-- ===================================

CREATE TABLE IF NOT EXISTS workspaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_path TEXT NOT NULL UNIQUE,
    workspace_hash TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_accessed_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK(workspace_path <> '')
) STRICT;

CREATE INDEX idx_workspaces_path ON workspaces(workspace_path);
CREATE INDEX idx_workspaces_hash ON workspaces(workspace_hash);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    workspace_id INTEGER NOT NULL,
    type TEXT NOT NULL,
    task TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,
    model TEXT,
    timeout_seconds INTEGER,
    preserve_worktrees INTEGER,
    winner_id INTEGER,
    total_duration INTEGER,
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

CREATE TABLE IF NOT EXISTS session_runtime_mix (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    runtime TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    UNIQUE(session_id, runtime)
) STRICT;

CREATE INDEX idx_runtime_mix_session ON session_runtime_mix(session_id);

CREATE TABLE IF NOT EXISTS instances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    instance_id INTEGER NOT NULL,
    worktree_path TEXT NOT NULL,
    branch TEXT NOT NULL,
    agent_name TEXT NOT NULL,
    status TEXT NOT NULL,
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
    UNIQUE(session_id, instance_id),
    CHECK(instance_id >= 0)
) STRICT;

CREATE INDEX idx_instances_session ON instances(session_id);
CREATE INDEX idx_instances_status ON instances(status);
CREATE INDEX idx_instances_session_instance ON instances(session_id, instance_id);

CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    instance_id INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    CHECK(role IN ('user', 'assistant', 'system')),
    CHECK(id <> ''),
    CHECK(content <> '')
) STRICT;

CREATE INDEX idx_chat_messages_session ON chat_messages(session_id);
CREATE INDEX idx_chat_messages_timestamp ON chat_messages(timestamp);
CREATE INDEX idx_chat_messages_role ON chat_messages(role);

CREATE TABLE IF NOT EXISTS migration_metadata (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    schema_version INTEGER NOT NULL DEFAULT 1,
    migrated_at TEXT NOT NULL DEFAULT (datetime('now')),
    json_backup_path TEXT,
    migration_status TEXT NOT NULL DEFAULT 'pending',
    last_updated_at TEXT NOT NULL DEFAULT (datetime('now'))
) STRICT;

INSERT OR IGNORE INTO migration_metadata (id, schema_version, migration_status)
VALUES (1, 1, 'pending');

-- ===================================
-- Views
-- ===================================

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

### Run Migrations

```bash
cd src-tauri

# Run migrations
sqlx migrate run

# Verify migrations
sqlx migrate info
```

---

## 4. Compile-Time Query Checking

### Prepare Database for Offline Mode

SQLx can check queries at compile time without a live database connection.

```bash
cd src-tauri

# Generate sqlx-data.json for offline compile-time checking
cargo sqlx prepare
```

This creates `sqlx-data.json` containing query metadata.

**Add to .gitignore**:
```gitignore
# src-tauri/.gitignore
.env
*.db
*.db-shm
*.db-wal
```

**Commit sqlx-data.json** to version control:
```bash
git add sqlx-data.json
git commit -m "Add SQLx query metadata for offline builds"
```

### CI/CD Configuration

```yaml
# .github/workflows/rust.yml
name: Rust CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install SQLx CLI
        run: cargo install sqlx-cli --no-default-features --features sqlite

      - name: Setup Database
        working-directory: src-tauri
        run: |
          sqlx database create
          sqlx migrate run

      - name: Check SQLx queries
        working-directory: src-tauri
        run: cargo sqlx prepare --check

      - name: Run tests
        working-directory: src-tauri
        run: cargo test
```

---

## 5. Directory Structure Setup

### Create Database Module

```bash
cd src-tauri/src

# Create database module directory
mkdir -p db

# Create module files
touch db/mod.rs
touch db/connection.rs
touch db/queries.rs
touch db/models.rs
touch db/error.rs
touch db/migration.rs
```

### Module File Templates

**db/mod.rs**:
```rust
pub mod connection;
pub mod queries;
pub mod models;
pub mod error;
pub mod migration;

pub use connection::Database;
pub use error::DatabaseError;
```

**db/error.rs**:
```rust
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

**db/connection.rs**:
```rust
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let db_path = get_database_path()?;

        info!("Initializing database at {:?}", db_path);

        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Configure SQLite options
        let connect_options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(5));

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect_with(connect_options)
            .await?;

        // Run migrations
        info!("Running database migrations");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        info!("Database initialized successfully");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_initialization() {
        let db = Database::new().await;
        assert!(db.is_ok());
    }
}
```

**db/models.rs**:
```rust
// Re-export existing types from commands module
pub use crate::commands::session_history::{
    WorktreeSession,
    WorktreeInstance,
    ChatMessage,
};

// Additional database-specific types if needed
#[derive(Debug, Clone)]
pub struct WorkspaceRecord {
    pub id: i64,
    pub workspace_path: String,
    pub workspace_hash: String,
    pub created_at: String,
    pub last_accessed_at: String,
}
```

---

## 6. Update main.rs

### Integrate Database into AppState

```rust
// src-tauri/src/main.rs

mod db; // Add database module

use db::Database;

#[tokio::main]
async fn main() {
    // ... existing initialization ...

    // Initialize database
    let database = Database::new()
        .await
        .expect("Failed to initialize database");

    let app_state = AppState {
        // ... existing fields ...
        database: Arc::new(database),
    };

    // ... rest of main ...
}
```

### Update state.rs

```rust
// src-tauri/src/state.rs

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

---

## 7. SQLx Query Examples

### Basic Query with sqlx::query!

```rust
// Compile-time checked query
let session = sqlx::query!(
    r#"
    SELECT id, type, task, status, created_at
    FROM sessions
    WHERE id = ?
    "#,
    session_id
)
.fetch_one(pool)
.await?;

// Type-safe access (fields are known at compile time)
println!("Session type: {}", session.type_);
```

### Query with Custom Type Mapping

```rust
// Use query_as! for custom structs
let sessions = sqlx::query_as!(
    WorktreeSession,
    r#"
    SELECT
        id,
        type as "type_",
        task,
        status,
        created_at,
        updated_at,
        completed_at
    FROM sessions
    WHERE workspace_id = ?
    "#,
    workspace_id
)
.fetch_all(pool)
.await?;
```

### Transaction Example

```rust
use sqlx::{Acquire, SqliteConnection};

async fn create_session_tx(
    pool: &SqlitePool,
    session: &WorktreeSession,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Insert session
    sqlx::query!(
        "INSERT INTO sessions (id, workspace_id, type, task, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        session.id,
        workspace_id,
        session.r#type,
        session.task,
        session.status,
        session.created_at,
        session.updated_at
    )
    .execute(&mut *tx)
    .await?;

    // Insert instances
    for instance in &session.instances {
        sqlx::query!(
            "INSERT INTO instances (session_id, instance_id, worktree_path, branch, agent_name, status, tmux_session_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            session.id,
            instance.instance_id,
            instance.worktree_path,
            instance.branch,
            instance.agent_name,
            instance.status,
            instance.tmux_session_id
        )
        .execute(&mut *tx)
        .await?;
    }

    // Commit transaction
    tx.commit().await?;

    Ok(())
}
```

### Dynamic Query Building

For complex queries with optional filters:

```rust
use sqlx::QueryBuilder;

async fn search_sessions(
    pool: &SqlitePool,
    workspace_id: i64,
    filters: &SessionFilters,
) -> Result<Vec<WorktreeSession>, sqlx::Error> {
    let mut query_builder = QueryBuilder::new(
        "SELECT id, type, task, status FROM sessions WHERE workspace_id = "
    );

    query_builder.push_bind(workspace_id);

    // Add optional filters
    if let Some(status) = &filters.status {
        query_builder.push(" AND status = ");
        query_builder.push_bind(status);
    }

    if let Some(session_type) = &filters.session_type {
        query_builder.push(" AND type = ");
        query_builder.push_bind(session_type);
    }

    let sessions = query_builder
        .build_query_as::<WorktreeSession>()
        .fetch_all(pool)
        .await?;

    Ok(sessions)
}
```

---

## 8. Testing Setup

### In-Memory Database for Tests

```rust
// tests/common/mod.rs
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

pub async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .expect("Failed to create test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

pub fn create_test_session(id: &str) -> WorktreeSession {
    WorktreeSession {
        id: id.to_string(),
        r#type: "competition".to_string(),
        task: "Test task".to_string(),
        status: "running".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
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
```

### Example Test

```rust
// tests/session_crud_test.rs
mod common;

use common::*;

#[tokio::test]
async fn test_create_and_retrieve_session() {
    let pool = setup_test_db().await;

    // Create workspace
    sqlx::query!(
        "INSERT INTO workspaces (workspace_path, workspace_hash) VALUES (?, ?)",
        "/test/workspace",
        "test_hash_123"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create session
    let session = create_test_session("test-session-1");
    let workspace_id = 1i64;

    sqlx::query!(
        "INSERT INTO sessions (id, workspace_id, type, task, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        session.id,
        workspace_id,
        session.r#type,
        session.task,
        session.status,
        session.created_at,
        session.updated_at
    )
    .execute(&pool)
    .await
    .unwrap();

    // Retrieve session
    let retrieved = sqlx::query!(
        "SELECT id, type, task, status FROM sessions WHERE id = ?",
        session.id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(retrieved.id, session.id);
    assert_eq!(retrieved.type_, session.r#type);
}
```

---

## 9. Environment Variables

### Development

```bash
# src-tauri/.env
DATABASE_URL=sqlite:$HOME/.ait42/sessions.db
RUST_LOG=info,sqlx=debug
```

### Production

```bash
# Production environment variables
DATABASE_URL=sqlite:$HOME/.ait42/sessions.db
RUST_LOG=info
```

---

## 10. Build Commands

### Development Build

```bash
cd src-tauri

# Build with SQLite feature (default)
cargo build

# Build with JSON storage (legacy)
cargo build --no-default-features --features json-storage
```

### Release Build

```bash
cd src-tauri

# Release build
cargo build --release

# Verify database is initialized
ls -lh ~/.ait42/sessions.db
```

### Check SQLx Queries

```bash
cd src-tauri

# Prepare query metadata (run after adding new queries)
cargo sqlx prepare

# Check queries without running (CI/CD)
cargo sqlx prepare --check
```

---

## 11. Troubleshooting

### Issue: "sqlx-data.json not found"

**Solution**:
```bash
cd src-tauri
cargo sqlx prepare
```

### Issue: "Database is locked"

**Cause**: Multiple processes accessing SQLite simultaneously

**Solution**:
- Use WAL mode (already configured)
- Increase busy_timeout (already set to 5 seconds)
- Check for orphaned processes

### Issue: Migration fails

**Solution**:
```bash
cd src-tauri

# Check migration status
sqlx migrate info

# Revert last migration
sqlx migrate revert

# Re-run migrations
sqlx migrate run
```

### Issue: Query type mismatch

**Cause**: SQLx query return type doesn't match Rust struct

**Solution**:
1. Check field names match (use `as` for aliases)
2. Use `query_as!` macro with explicit types
3. Re-run `cargo sqlx prepare`

---

## 12. Performance Tuning

### SQLite PRAGMA Settings

Already configured in schema, but can be adjusted:

```sql
-- Increase cache size (default: 2MB, increase for better performance)
PRAGMA cache_size = -20000;  -- 20MB cache

-- Enable memory-mapped I/O
PRAGMA mmap_size = 268435456; -- 256MB

-- Optimize for concurrent access
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
```

### Connection Pool Tuning

```rust
// Adjust based on workload
SqlitePoolOptions::new()
    .max_connections(10)      // Increase for high concurrency
    .min_connections(2)       // Keep warm connections
    .acquire_timeout(Duration::from_secs(5))
    .connect_with(connect_options)
    .await?
```

---

## Next Steps

1. **Complete database module implementation** (db/queries.rs)
2. **Implement session commands** (commands/session_history_sqlite.rs)
3. **Write comprehensive tests** (tests/)
4. **Run performance benchmarks**
5. **Deploy Phase 1** (parallel operation)

---

**Document Version**: 1.0.0
**Last Updated**: 2025-01-13
**Related Documents**:
- API_DESIGN.md
- MIGRATION_PLAN.md (to be created)
- TESTING_GUIDE.md (to be created)
