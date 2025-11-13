-- Add full-text search capability for task descriptions
-- SQLx migration: Up

-- Create FTS5 virtual table for full-text search
CREATE VIRTUAL TABLE IF NOT EXISTS sessions_fts USING fts5(
    task,
    content=sessions,
    content_rowid=rowid
);

-- Populate FTS table with existing data
INSERT INTO sessions_fts(rowid, task)
SELECT rowid, task FROM sessions;

-- Create triggers to keep FTS table synchronized with sessions table

CREATE TRIGGER sessions_fts_insert AFTER INSERT ON sessions BEGIN
    INSERT INTO sessions_fts(rowid, task) VALUES (new.rowid, new.task);
END;

CREATE TRIGGER sessions_fts_delete AFTER DELETE ON sessions BEGIN
    INSERT INTO sessions_fts(sessions_fts, rowid, task) VALUES ('delete', old.rowid, old.task);
END;

CREATE TRIGGER sessions_fts_update AFTER UPDATE ON sessions BEGIN
    INSERT INTO sessions_fts(sessions_fts, rowid, task) VALUES ('delete', old.rowid, old.task);
    INSERT INTO sessions_fts(rowid, task) VALUES (new.rowid, new.task);
END;
