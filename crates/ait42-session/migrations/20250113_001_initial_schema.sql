-- Session storage schema for AIT42 Editor
-- SQLite version: 3.40+

-- Enable WAL mode for concurrent reads during writes
PRAGMA journal_mode=WAL;

-- Enable foreign key constraints
PRAGMA foreign_keys=ON;

-- ===================================
-- Core Tables
-- ===================================

-- Workspaces: Track workspace metadata
CREATE TABLE IF NOT EXISTS workspaces (
    hash TEXT PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    last_accessed TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_workspaces_path ON workspaces(path);
CREATE INDEX idx_workspaces_hash ON workspaces(hash);

-- Sessions: Main session data
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
    FOREIGN KEY (workspace_hash) REFERENCES workspaces(hash) ON DELETE CASCADE,
    CHECK(id <> ''),
    CHECK(task <> '')
);

CREATE INDEX idx_sessions_workspace ON sessions(workspace_hash);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_type ON sessions(session_type);
CREATE INDEX idx_sessions_created ON sessions(created_at DESC);
CREATE INDEX idx_sessions_updated ON sessions(updated_at DESC);

-- Instances: Worktree instances within a session
CREATE TABLE IF NOT EXISTS instances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    instance_id INTEGER NOT NULL,
    worktree_path TEXT NOT NULL,
    branch TEXT NOT NULL,
    agent_name TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('idle', 'running', 'completed', 'failed', 'paused', 'archived')),
    tmux_session_id TEXT NOT NULL,
    output TEXT, -- Can be large (>100KB)
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
);

CREATE INDEX idx_instances_session ON instances(session_id);
CREATE INDEX idx_instances_status ON instances(status);
CREATE INDEX idx_instances_session_instance ON instances(session_id, instance_id);

-- Chat Messages: Chat history for sessions
CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    instance_id INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    CHECK(id <> ''),
    CHECK(content <> '')
);

CREATE INDEX idx_chat_messages_session ON chat_messages(session_id);
CREATE INDEX idx_chat_messages_timestamp ON chat_messages(timestamp DESC);
CREATE INDEX idx_chat_messages_role ON chat_messages(role);

-- ===================================
-- Migration Metadata
-- ===================================

-- Track migration status and version
CREATE TABLE IF NOT EXISTS migration_metadata (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    schema_version INTEGER NOT NULL DEFAULT 1,
    migrated_at TEXT NOT NULL DEFAULT (datetime('now')),
    json_backup_path TEXT,
    migration_status TEXT NOT NULL DEFAULT 'pending',
    last_updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Initialize migration metadata
INSERT OR IGNORE INTO migration_metadata (id, schema_version, migration_status)
VALUES (1, 1, 'pending');
