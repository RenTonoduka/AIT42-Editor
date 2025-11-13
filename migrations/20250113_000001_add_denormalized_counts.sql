-- Add denormalized count columns for performance optimization
-- SQLx migration: Up

-- Note: SQLx tracks migrations, so this typically runs once.
-- However, we make it defensive for manual re-runs.

-- SQLite doesn't support IF NOT EXISTS for ALTER TABLE ADD COLUMN,
-- but we can check and conditionally execute using a workaround.
-- Since SQLx tracks migrations, we rely on that mechanism primarily.

-- Add count columns
-- These will fail silently if columns already exist (for manual re-runs)
ALTER TABLE sessions ADD COLUMN instance_count INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN message_count INTEGER DEFAULT 0;

-- Backfill existing data (safe to run multiple times due to WHERE clause)
-- Only update rows where counts are not set
UPDATE sessions
SET instance_count = (
    SELECT COUNT(*) FROM instances WHERE instances.session_id = sessions.id
)
WHERE instance_count = 0;

UPDATE sessions
SET message_count = (
    SELECT COUNT(*) FROM chat_messages WHERE chat_messages.session_id = sessions.id
)
WHERE message_count = 0;

-- Create triggers to maintain instance_count
-- DROP IF EXISTS makes this idempotent
DROP TRIGGER IF EXISTS update_instance_count_insert;
CREATE TRIGGER update_instance_count_insert
AFTER INSERT ON instances
FOR EACH ROW
BEGIN
    UPDATE sessions
    SET instance_count = COALESCE(instance_count, 0) + 1
    WHERE id = NEW.session_id;
END;

DROP TRIGGER IF EXISTS update_instance_count_delete;
CREATE TRIGGER update_instance_count_delete
AFTER DELETE ON instances
FOR EACH ROW
BEGIN
    UPDATE sessions
    SET instance_count = COALESCE(instance_count, 0) - 1
    WHERE id = OLD.session_id;
END;

-- Create triggers to maintain message_count
DROP TRIGGER IF EXISTS update_message_count_insert;
CREATE TRIGGER update_message_count_insert
AFTER INSERT ON chat_messages
FOR EACH ROW
BEGIN
    UPDATE sessions
    SET message_count = COALESCE(message_count, 0) + 1
    WHERE id = NEW.session_id;
END;

DROP TRIGGER IF EXISTS update_message_count_delete;
CREATE TRIGGER update_message_count_delete
AFTER DELETE ON chat_messages
FOR EACH ROW
BEGIN
    UPDATE sessions
    SET message_count = COALESCE(message_count, 0) - 1
    WHERE id = OLD.session_id;
END;
