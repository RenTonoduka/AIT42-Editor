-- Initial schema creation for AIT42 Session History Database
-- SQLx migration: Up

-- Workspaces table to store workspace metadata
CREATE TABLE IF NOT EXISTS workspaces (
    hash TEXT PRIMARY KEY NOT NULL,
    path TEXT NOT NULL,
    last_accessed TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Index for workspace path lookups
CREATE INDEX IF NOT EXISTS idx_workspaces_path ON workspaces(path);
CREATE INDEX IF NOT EXISTS idx_workspaces_last_accessed ON workspaces(last_accessed DESC);

-- ---

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_hash TEXT NOT NULL,
    session_type TEXT NOT NULL CHECK (session_type IN ('competition', 'ensemble', 'debate', 'integration')),
    task TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('running', 'completed', 'failed', 'paused')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,
    model TEXT,
    timeout_seconds INTEGER CHECK (timeout_seconds IS NULL OR timeout_seconds > 0),
    preserve_worktrees INTEGER DEFAULT 0 CHECK (preserve_worktrees IN (0, 1)),
    winner_id INTEGER,
    runtime_mix TEXT,  -- JSON array: ["claude", "gemini", "codex"]
    total_duration INTEGER CHECK (total_duration IS NULL OR total_duration >= 0),
    total_files_changed INTEGER,
    total_lines_added INTEGER,
    total_lines_deleted INTEGER,
    FOREIGN KEY (workspace_hash) REFERENCES workspaces(hash) ON DELETE CASCADE
);

-- Indexes for sessions table
CREATE INDEX IF NOT EXISTS idx_sessions_workspace_hash ON sessions(workspace_hash);
CREATE INDEX IF NOT EXISTS idx_sessions_type ON sessions(session_type);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_sessions_workspace_status ON sessions(workspace_hash, status);
CREATE INDEX IF NOT EXISTS idx_sessions_workspace_created ON sessions(workspace_hash, created_at DESC);

-- ---

CREATE TABLE IF NOT EXISTS instances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    instance_id INTEGER NOT NULL,
    worktree_path TEXT NOT NULL,
    branch TEXT NOT NULL,
    agent_name TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('running', 'completed', 'failed', 'timeout')),
    tmux_session_id TEXT NOT NULL,
    output TEXT,
    start_time TEXT,
    end_time TEXT,
    files_changed INTEGER,
    lines_added INTEGER,
    lines_deleted INTEGER,
    runtime TEXT CHECK (runtime IS NULL OR runtime IN ('claude', 'gemini', 'codex')),
    model TEXT,
    runtime_label TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    UNIQUE (session_id, instance_id)
);

-- Indexes for instances table
CREATE INDEX IF NOT EXISTS idx_instances_session_id ON instances(session_id);
CREATE INDEX IF NOT EXISTS idx_instances_status ON instances(status);
CREATE INDEX IF NOT EXISTS idx_instances_session_instance ON instances(session_id, instance_id);
CREATE INDEX IF NOT EXISTS idx_instances_tmux ON instances(tmux_session_id);

-- ---

CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    instance_id INTEGER,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL CHECK (LENGTH(content) <= 10000),
    timestamp TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE SET NULL
);

-- Indexes for chat_messages table
CREATE INDEX IF NOT EXISTS idx_chat_messages_session_id ON chat_messages(session_id);
CREATE INDEX IF NOT EXISTS idx_chat_messages_timestamp ON chat_messages(timestamp);
CREATE INDEX IF NOT EXISTS idx_chat_messages_session_time ON chat_messages(session_id, timestamp);

-- ---

-- Enable foreign key constraints (must be set for each connection in SQLite)
PRAGMA foreign_keys = ON;

-- Enable WAL mode for better concurrency
PRAGMA journal_mode = WAL;

-- Set synchronous mode to NORMAL for better performance
PRAGMA synchronous = NORMAL;

-- Increase cache size to 64 MB
PRAGMA cache_size = -64000;

-- Enable memory-mapped I/O for faster reads (256 MB)
PRAGMA mmap_size = 268435456;

-- Enable auto-vacuum (incremental mode)
PRAGMA auto_vacuum = INCREMENTAL;
