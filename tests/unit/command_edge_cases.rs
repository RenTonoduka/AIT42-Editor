//! Additional Command System Edge Case Tests
//!
//! Comprehensive test coverage for command execution, undo/redo,
//! command merging, and history management.

use ait42_core::buffer::Buffer;
use ait42_core::command::{Command, CommandHistory, DeleteCommand, InsertCommand, ReplaceCommand};

#[cfg(test)]
mod command_execution_tests {
    use super::*;

    #[test]
    fn test_insert_command_at_start() {
        let mut buffer = Buffer::from_string("World".to_string(), None);
        let mut cmd = InsertCommand::new(buffer.id(), 0, "Hello ");

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello World");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "World");
    }

    #[test]
    fn test_insert_command_at_end() {
        let mut buffer = Buffer::from_string("Hello".to_string(), None);
        let len = buffer.len_bytes();
        let mut cmd = InsertCommand::new(buffer.id(), len, " World");

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello World");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello");
    }

    #[test]
    fn test_insert_command_empty_string() {
        let mut buffer = Buffer::from_string("Test".to_string(), None);
        let mut cmd = InsertCommand::new(buffer.id(), 2, "");

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Test"); // Unchanged

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Test");
    }

    #[test]
    fn test_insert_command_multibyte() {
        let mut buffer = Buffer::new();
        let mut cmd = InsertCommand::new(buffer.id(), 0, "こんにちは");

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "こんにちは");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "");
    }

    #[test]
    fn test_delete_command_beginning() {
        let mut buffer = Buffer::from_string("Hello World".to_string(), None);
        let mut cmd = DeleteCommand::new(buffer.id(), 0..6);

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "World");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello World");
    }

    #[test]
    fn test_delete_command_end() {
        let mut buffer = Buffer::from_string("Hello World".to_string(), None);
        let mut cmd = DeleteCommand::new(buffer.id(), 5..11);

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello World");
    }

    #[test]
    fn test_delete_command_empty_range() {
        let mut buffer = Buffer::from_string("Test".to_string(), None);
        let mut cmd = DeleteCommand::new(buffer.id(), 2..2);

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Test"); // Unchanged

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Test");
    }

    #[test]
    fn test_delete_command_entire_buffer() {
        let mut buffer = Buffer::from_string("Delete me".to_string(), None);
        let len = buffer.len_bytes();
        let mut cmd = DeleteCommand::new(buffer.id(), 0..len);

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Delete me");
    }

    #[test]
    fn test_replace_command_same_length() {
        let mut buffer = Buffer::from_string("Hello World".to_string(), None);
        let mut cmd = ReplaceCommand::new(buffer.id(), 0..5, "Goodbye".to_string());

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Goodbye World");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello World");
    }

    #[test]
    fn test_replace_command_shorter() {
        let mut buffer = Buffer::from_string("Hello World".to_string(), None);
        let mut cmd = ReplaceCommand::new(buffer.id(), 0..5, "Hi".to_string());

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hi World");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello World");
    }

    #[test]
    fn test_replace_command_longer() {
        let mut buffer = Buffer::from_string("Hi World".to_string(), None);
        let mut cmd = ReplaceCommand::new(buffer.id(), 0..2, "Hello".to_string());

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello World");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hi World");
    }

    #[test]
    fn test_replace_with_empty_string() {
        let mut buffer = Buffer::from_string("Hello World".to_string(), None);
        let mut cmd = ReplaceCommand::new(buffer.id(), 5..11, "".to_string());

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello World");
    }

    #[test]
    fn test_command_description() {
        let buffer = Buffer::new();
        let insert = InsertCommand::new(buffer.id(), 0, "test");
        let delete = DeleteCommand::new(buffer.id(), 0..1);
        let replace = ReplaceCommand::new(buffer.id(), 0..1, "x".to_string());

        assert_eq!(insert.description(), "Insert text");
        assert_eq!(delete.description(), "Delete text");
        assert_eq!(replace.description(), "Replace text");
    }

    #[test]
    fn test_command_can_undo() {
        let buffer = Buffer::new();
        let cmd = InsertCommand::new(buffer.id(), 0, "test");

        assert!(cmd.can_undo());
    }
}

#[cfg(test)]
mod command_merging_tests {
    use super::*;

    #[test]
    fn test_insert_command_merge_consecutive() {
        let buffer = Buffer::new();
        let mut cmd1 = InsertCommand::new(buffer.id(), 0, "Hello");
        let cmd2 = InsertCommand::new(buffer.id(), 5, " World");

        // cmd2 position is consecutive to cmd1 end
        assert!(cmd1.merge_with(&cmd2));

        // Now cmd1 should contain both texts
        let mut test_buffer = Buffer::new();
        cmd1.execute(&mut test_buffer).unwrap();
        assert_eq!(test_buffer.to_string(), "Hello World");
    }

    #[test]
    fn test_insert_command_merge_non_consecutive() {
        let buffer = Buffer::new();
        let mut cmd1 = InsertCommand::new(buffer.id(), 0, "Hello");
        let cmd2 = InsertCommand::new(buffer.id(), 10, "World"); // Gap

        assert!(!cmd1.merge_with(&cmd2));
    }

    #[test]
    fn test_insert_command_no_merge_different_type() {
        let buffer = Buffer::new();
        let mut insert = InsertCommand::new(buffer.id(), 0, "test");
        let delete = DeleteCommand::new(buffer.id(), 0..1);

        assert!(!insert.merge_with(&delete));
    }

    #[test]
    fn test_delete_command_no_merge() {
        let buffer = Buffer::new();
        let mut delete1 = DeleteCommand::new(buffer.id(), 0..5);
        let delete2 = DeleteCommand::new(buffer.id(), 5..10);

        // Delete commands don't implement merging
        assert!(!delete1.merge_with(&delete2));
    }

    #[test]
    fn test_replace_command_no_merge() {
        let buffer = Buffer::new();
        let mut replace1 = ReplaceCommand::new(buffer.id(), 0..5, "a".to_string());
        let replace2 = ReplaceCommand::new(buffer.id(), 5..10, "b".to_string());

        // Replace commands don't implement merging
        assert!(!replace1.merge_with(&replace2));
    }
}

#[cfg(test)]
mod command_history_tests {
    use super::*;

    #[test]
    fn test_history_creation() {
        let history = CommandHistory::new();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_len(), 0);
        assert_eq!(history.redo_len(), 0);
    }

    #[test]
    fn test_history_with_capacity() {
        let history = CommandHistory::with_capacity(500);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_push_command() {
        let buffer = Buffer::new();
        let mut history = CommandHistory::new();

        let cmd = Box::new(InsertCommand::new(buffer.id(), 0, "test"));
        history.push(cmd);

        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_len(), 1);
    }

    #[test]
    fn test_undo_on_empty_history() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        let result = history.undo(&mut buffer).unwrap();
        assert!(!result); // Nothing to undo
        assert!(!history.can_undo());
    }

    #[test]
    fn test_redo_on_empty_history() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        let result = history.redo(&mut buffer).unwrap();
        assert!(!result); // Nothing to redo
        assert!(!history.can_redo());
    }

    #[test]
    fn test_undo_single_command() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        let mut cmd = Box::new(InsertCommand::new(buffer.id(), 0, "test"));
        cmd.execute(&mut buffer).unwrap();
        history.push(cmd);

        assert_eq!(buffer.to_string(), "test");

        let result = history.undo(&mut buffer).unwrap();
        assert!(result);
        assert_eq!(buffer.to_string(), "");
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_redo_after_undo() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        let mut cmd = Box::new(InsertCommand::new(buffer.id(), 0, "test"));
        cmd.execute(&mut buffer).unwrap();
        history.push(cmd);

        history.undo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "");

        let result = history.redo(&mut buffer).unwrap();
        assert!(result);
        assert_eq!(buffer.to_string(), "test");
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_new_command_clears_redo() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        // Execute and undo a command
        let mut cmd1 = Box::new(InsertCommand::new(buffer.id(), 0, "first"));
        cmd1.execute(&mut buffer).unwrap();
        history.push(cmd1);
        history.undo(&mut buffer).unwrap();

        assert!(history.can_redo());

        // Execute new command
        let mut cmd2 = Box::new(InsertCommand::new(buffer.id(), 0, "second"));
        cmd2.execute(&mut buffer).unwrap();
        history.push(cmd2);

        // Redo should be cleared
        assert!(!history.can_redo());
        assert_eq!(history.redo_len(), 0);
    }

    #[test]
    fn test_multiple_undo_redo() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        // Execute three commands
        for i in 0..3 {
            let mut cmd = Box::new(InsertCommand::new(buffer.id(), buffer.len_bytes(), &format!("{}", i)));
            cmd.execute(&mut buffer).unwrap();
            history.push(cmd);
        }

        assert_eq!(buffer.to_string(), "012");

        // Undo all
        history.undo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "01");
        history.undo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "0");
        history.undo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "");

        // Redo all
        history.redo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "0");
        history.redo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "01");
        history.redo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "012");
    }

    #[test]
    fn test_history_max_capacity() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::with_capacity(3);

        // Push 5 commands
        for i in 0..5 {
            let mut cmd = Box::new(InsertCommand::new(buffer.id(), buffer.len_bytes(), "x"));
            cmd.execute(&mut buffer).unwrap();
            history.push(cmd);
        }

        // Should only keep last 3
        assert_eq!(history.undo_len(), 3);
    }

    #[test]
    fn test_history_clear() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        let mut cmd = Box::new(InsertCommand::new(buffer.id(), 0, "test"));
        cmd.execute(&mut buffer).unwrap();
        history.push(cmd);

        history.clear();

        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_len(), 0);
        assert_eq!(history.redo_len(), 0);
    }

    #[test]
    fn test_command_merging_in_history() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        // Push first command
        let mut cmd1 = Box::new(InsertCommand::new(buffer.id(), 0, "Hello"));
        cmd1.execute(&mut buffer).unwrap();
        history.push(cmd1);

        // Push mergeable command
        let mut cmd2 = Box::new(InsertCommand::new(buffer.id(), 5, " World"));
        cmd2.execute(&mut buffer).unwrap();
        history.push(cmd2);

        // Should have been merged
        assert_eq!(history.undo_len(), 1);

        // Undo should undo both
        history.undo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "");
    }
}

#[cfg(test)]
mod command_error_handling_tests {
    use super::*;

    #[test]
    fn test_insert_invalid_position() {
        let mut buffer = Buffer::new();
        let mut cmd = InsertCommand::new(buffer.id(), 1000, "test");

        let result = cmd.execute(&mut buffer);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_invalid_range() {
        let mut buffer = Buffer::from_string("Test".to_string(), None);
        let mut cmd = DeleteCommand::new(buffer.id(), 0..100);

        let result = cmd.execute(&mut buffer);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_inverted_range() {
        let mut buffer = Buffer::from_string("Test".to_string(), None);
        let mut cmd = DeleteCommand::new(buffer.id(), 3..1);

        let result = cmd.execute(&mut buffer);
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_invalid_range() {
        let mut buffer = Buffer::from_string("Test".to_string(), None);
        let mut cmd = ReplaceCommand::new(buffer.id(), 0..100, "x".to_string());

        let result = cmd.execute(&mut buffer);
        assert!(result.is_err());
    }

    #[test]
    fn test_undo_without_execute_fails() {
        let mut buffer = Buffer::new();
        let mut cmd = DeleteCommand::new(buffer.id(), 0..1);

        // Undo without execute should fail (no saved text)
        let result = cmd.undo(&mut buffer);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod command_complex_workflows {
    use super::*;

    #[test]
    fn test_insert_delete_insert_workflow() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        // Insert
        let mut cmd1 = Box::new(InsertCommand::new(buffer.id(), 0, "Hello World"));
        cmd1.execute(&mut buffer).unwrap();
        history.push(cmd1);

        // Delete
        let mut cmd2 = Box::new(DeleteCommand::new(buffer.id(), 5..11));
        cmd2.execute(&mut buffer).unwrap();
        history.push(cmd2);

        // Insert again
        let mut cmd3 = Box::new(InsertCommand::new(buffer.id(), 5, " Rust"));
        cmd3.execute(&mut buffer).unwrap();
        history.push(cmd3);

        assert_eq!(buffer.to_string(), "Hello Rust");

        // Undo all
        history.undo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "Hello");
        history.undo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "Hello World");
        history.undo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "");
    }

    #[test]
    fn test_replace_undo_replace_workflow() {
        let mut buffer = Buffer::from_string("Hello World".to_string(), None);
        let mut history = CommandHistory::new();

        // First replace
        let mut cmd1 = Box::new(ReplaceCommand::new(buffer.id(), 0..5, "Hi".to_string()));
        cmd1.execute(&mut buffer).unwrap();
        history.push(cmd1);
        assert_eq!(buffer.to_string(), "Hi World");

        // Undo
        history.undo(&mut buffer).unwrap();
        assert_eq!(buffer.to_string(), "Hello World");

        // Different replace
        let mut cmd2 = Box::new(ReplaceCommand::new(buffer.id(), 6..11, "Rust".to_string()));
        cmd2.execute(&mut buffer).unwrap();
        history.push(cmd2);
        assert_eq!(buffer.to_string(), "Hello Rust");

        // Can't redo first command
        assert!(!history.can_redo());
    }

    #[test]
    fn test_mixed_commands_workflow() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        let commands: Vec<Box<dyn Command>> = vec![
            Box::new(InsertCommand::new(buffer.id(), 0, "The ")),
            Box::new(InsertCommand::new(buffer.id(), 4, "quick ")),
            Box::new(InsertCommand::new(buffer.id(), 10, "brown ")),
            Box::new(InsertCommand::new(buffer.id(), 16, "fox")),
        ];

        for mut cmd in commands {
            cmd.execute(&mut buffer).unwrap();
            history.push(cmd);
        }

        assert_eq!(buffer.to_string(), "The quick brown fox");

        // Undo twice
        history.undo(&mut buffer).unwrap();
        history.undo(&mut buffer).unwrap();

        assert!(buffer.to_string().len() < 19);

        // Redo once
        history.redo(&mut buffer).unwrap();

        assert!(history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_rapid_typing_simulation() {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        let text = "The quick brown fox jumps over the lazy dog";

        for (i, ch) in text.chars().enumerate() {
            let mut cmd = Box::new(InsertCommand::new(buffer.id(), i, &ch.to_string()));
            cmd.execute(&mut buffer).unwrap();
            history.push(cmd);
        }

        // Due to merging, should have fewer entries than characters
        assert!(history.undo_len() < text.len());
    }
}

#[cfg(test)]
mod command_utf8_tests {
    use super::*;

    #[test]
    fn test_insert_emoji_command() {
        let mut buffer = Buffer::new();
        let mut cmd = InsertCommand::new(buffer.id(), 0, "Hello 👋 World");

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello 👋 World");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "");
    }

    #[test]
    fn test_delete_multibyte_characters() {
        let mut buffer = Buffer::from_string("Hello 世界".to_string(), None);
        let start = "Hello ".len();
        let end = buffer.len_bytes();

        let mut cmd = DeleteCommand::new(buffer.id(), start..end);

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello ");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello 世界");
    }

    #[test]
    fn test_replace_ascii_with_multibyte() {
        let mut buffer = Buffer::from_string("Hello".to_string(), None);
        let mut cmd = ReplaceCommand::new(buffer.id(), 0..5, "こんにちは".to_string());

        assert!(cmd.execute(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "こんにちは");

        assert!(cmd.undo(&mut buffer).is_ok());
        assert_eq!(buffer.to_string(), "Hello");
    }
}
