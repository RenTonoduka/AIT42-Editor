-- Add denormalized count columns for performance optimization
-- SQLx migration: Up

-- Add count columns
ALTER TABLE sessions ADD COLUMN instance_count INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN message_count INTEGER DEFAULT 0;

-- Backfill existing data
UPDATE sessions SET instance_count = (
    SELECT COUNT(*) FROM instances WHERE instances.session_id = sessions.id
);

UPDATE sessions SET message_count = (
    SELECT COUNT(*) FROM chat_messages WHERE chat_messages.session_id = sessions.id
);

-- Create triggers to maintain instance_count

CREATE TRIGGER update_instance_count_insert
AFTER INSERT ON instances
FOR EACH ROW
BEGIN
    UPDATE sessions
    SET instance_count = instance_count + 1
    WHERE id = NEW.session_id;
END;

CREATE TRIGGER update_instance_count_delete
AFTER DELETE ON instances
FOR EACH ROW
BEGIN
    UPDATE sessions
    SET instance_count = instance_count - 1
    WHERE id = OLD.session_id;
END;

-- Create triggers to maintain message_count

CREATE TRIGGER update_message_count_insert
AFTER INSERT ON chat_messages
FOR EACH ROW
BEGIN
    UPDATE sessions
    SET message_count = message_count + 1
    WHERE id = NEW.session_id;
END;

CREATE TRIGGER update_message_count_delete
AFTER DELETE ON chat_messages
FOR EACH ROW
BEGIN
    UPDATE sessions
    SET message_count = message_count - 1
    WHERE id = OLD.session_id;
END;
