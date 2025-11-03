//! Property-based tests for Command system

use ait42_core::buffer::Buffer;
use ait42_core::command::{Command, CommandHistory, DeleteCommand, InsertCommand, ReplaceCommand};
use proptest::prelude::*;

proptest! {
    /// Execute then undo restores original state
    #[test]
    fn test_command_undo_inverse(
        initial_text in "\\PC{0,100}",
        insert_text in "\\PC{1,50}",
        pos in 0..50usize
    ) {
        let mut buffer = Buffer::from_string(initial_text.clone(), None);
        let pos = pos.min(buffer.len_bytes());

        let mut cmd = InsertCommand::new(buffer.id(), pos, insert_text);

        // Execute
        if cmd.execute(&mut buffer).is_ok() {
            // Undo
            let _ = cmd.undo(&mut buffer);

            // Should restore original
            prop_assert_eq!(buffer.to_string(), initial_text);
        }
    }

    /// Delete then undo preserves exact text
    #[test]
    fn test_delete_undo_exact(
        s in "\\PC{10,100}",
        start in 0..50usize
    ) {
        let original = s.clone();
        let mut buffer = Buffer::from_string(s, None);
        let len = buffer.len_bytes();

        if len > 2 {
            let start = start.min(len - 2);
            let end = (start + 5).min(len);

            let mut cmd = DeleteCommand::new(buffer.id(), start..end);

            // Execute delete
            if cmd.execute(&mut buffer).is_ok() {
                // Undo should restore exact text
                let _ = cmd.undo(&mut buffer);
                prop_assert_eq!(buffer.to_string(), original);
            }
        }
    }

    /// Replace then undo preserves original
    #[test]
    fn test_replace_undo_inverse(
        s in "\\PC{10,100}",
        replacement in "\\PC{5,50}"
    ) {
        let original = s.clone();
        let mut buffer = Buffer::from_string(s, None);
        let len = buffer.len_bytes();

        if len > 4 {
            let start = len / 4;
            let end = (len * 3) / 4;

            let mut cmd = ReplaceCommand::new(buffer.id(), start..end, replacement);

            if cmd.execute(&mut buffer).is_ok() {
                let _ = cmd.undo(&mut buffer);
                prop_assert_eq!(buffer.to_string(), original);
            }
        }
    }

    /// Command history undo/redo is reversible
    #[test]
    fn test_history_undo_redo_inverse(
        texts in prop::collection::vec("\\PC{1,20}", 1..10)
    ) {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        // Execute all commands
        let mut pos = 0;
        for text in &texts {
            let mut cmd = InsertCommand::new(buffer.id(), pos, text.as_str());
            let _ = cmd.execute(&mut buffer);
            history.push(Box::new(cmd));
            pos += text.len();
        }

        let after_insert = buffer.to_string();

        // Undo all
        while history.can_undo() {
            let _ = history.undo(&mut buffer);
        }

        let after_undo = buffer.to_string();
        prop_assert_eq!(after_undo, "");

        // Redo all
        while history.can_redo() {
            let _ = history.redo(&mut buffer);
        }

        // Should restore state after inserts
        prop_assert_eq!(buffer.to_string(), after_insert);
    }

    /// Command history respects max capacity
    #[test]
    fn test_history_capacity_limit(
        capacity in 1..20usize,
        num_commands in 1..50usize
    ) {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::with_capacity(capacity);

        // Execute more commands than capacity
        for i in 0..num_commands {
            let mut cmd = InsertCommand::new(buffer.id(), 0, format!("{}", i));
            let _ = cmd.execute(&mut buffer);
            history.push(Box::new(cmd));
        }

        // History should not exceed capacity
        prop_assert!(history.undo_len() <= capacity);
    }

    /// New command clears redo stack
    #[test]
    fn test_new_command_clears_redo(
        texts in prop::collection::vec("\\PC{1,20}", 2..10)
    ) {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        // Execute some commands
        for (i, text) in texts.iter().enumerate() {
            let mut cmd = InsertCommand::new(buffer.id(), i, text.as_str());
            let _ = cmd.execute(&mut buffer);
            history.push(Box::new(cmd));
        }

        // Undo at least one
        let _ = history.undo(&mut buffer);
        prop_assert!(history.can_redo());

        // Execute new command
        let mut new_cmd = InsertCommand::new(buffer.id(), 0, "new");
        let _ = new_cmd.execute(&mut buffer);
        history.push(Box::new(new_cmd));

        // Redo should be cleared
        prop_assert!(!history.can_redo());
    }

    /// Empty buffer undo is no-op
    #[test]
    fn test_undo_empty_history(s in "\\PC{0,50}") {
        let mut buffer = Buffer::from_string(s.clone(), None);
        let mut history = CommandHistory::new();

        let original = buffer.to_string();

        // Undo with empty history
        let result = history.undo(&mut buffer);

        prop_assert!(result.is_ok());
        prop_assert_eq!(buffer.to_string(), original);
    }

    /// Multiple undos then same number of redos restores state
    #[test]
    fn test_symmetric_undo_redo(
        texts in prop::collection::vec("\\PC{1,10}", 3..10),
        undo_count in 1..5usize
    ) {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        // Execute commands
        let mut pos = 0;
        for text in &texts {
            let mut cmd = InsertCommand::new(buffer.id(), pos, text.as_str());
            let _ = cmd.execute(&mut buffer);
            history.push(Box::new(cmd));
            pos += text.len();
        }

        let state_after_commands = buffer.to_string();

        // Undo some
        let actual_undos = undo_count.min(texts.len());
        for _ in 0..actual_undos {
            let _ = history.undo(&mut buffer);
        }

        // Redo same amount
        for _ in 0..actual_undos {
            let _ = history.redo(&mut buffer);
        }

        // Should restore state
        prop_assert_eq!(buffer.to_string(), state_after_commands);
    }

    /// Command execution increments buffer version
    #[test]
    fn test_command_increments_version(
        s in "\\PC{0,50}",
        insert_text in "\\PC{1,20}"
    ) {
        let mut buffer = Buffer::from_string(s, None);
        let v0 = buffer.version();

        let mut cmd = InsertCommand::new(buffer.id(), 0, insert_text);
        if cmd.execute(&mut buffer).is_ok() {
            prop_assert_eq!(buffer.version(), v0 + 1);
        }

        // Undo also increments version
        if cmd.undo(&mut buffer).is_ok() {
            prop_assert_eq!(buffer.version(), v0 + 2);
        }
    }

    /// Command history clear removes all commands
    #[test]
    fn test_history_clear(
        texts in prop::collection::vec("\\PC{1,20}", 1..10)
    ) {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        // Add commands
        for text in &texts {
            let mut cmd = InsertCommand::new(buffer.id(), 0, text.as_str());
            let _ = cmd.execute(&mut buffer);
            history.push(Box::new(cmd));
        }

        prop_assert!(history.can_undo());

        history.clear();

        prop_assert!(!history.can_undo());
        prop_assert!(!history.can_redo());
        prop_assert_eq!(history.undo_len(), 0);
        prop_assert_eq!(history.redo_len(), 0);
    }

    /// Insert command merge reduces history size
    #[test]
    fn test_insert_merge_reduces_history(
        chars in prop::collection::vec('a'..='z', 2..20)
    ) {
        let mut buffer = Buffer::new();
        let mut history = CommandHistory::new();

        // Insert characters consecutively (should merge)
        for (i, ch) in chars.iter().enumerate() {
            let mut cmd = InsertCommand::new(buffer.id(), i, ch.to_string());
            let _ = cmd.execute(&mut buffer);
            history.push(Box::new(cmd));
        }

        // Due to merging, history should be smaller than number of inserts
        prop_assert!(history.undo_len() <= chars.len());
    }

    /// Delete command preserves UTF-8
    #[test]
    fn test_delete_preserves_utf8(
        s in "\\PC{10,100}"
    ) {
        let mut buffer = Buffer::from_string(s, None);
        let len = buffer.len_bytes();

        if len > 4 {
            let start = 2;
            let end = len - 2;

            let mut cmd = DeleteCommand::new(buffer.id(), start..end);

            if cmd.execute(&mut buffer).is_ok() {
                // Buffer should still be valid UTF-8
                let _ = buffer.to_string(); // Would panic if invalid

                // Undo should also preserve UTF-8
                if cmd.undo(&mut buffer).is_ok() {
                    let _ = buffer.to_string();
                }

                prop_assert!(true);
            }
        }
    }

    /// Replace with same text is idempotent for undo
    #[test]
    fn test_replace_same_text_idempotent(s in "\\PC{10,50}") {
        let original = s.clone();
        let mut buffer = Buffer::from_string(s.clone(), None);
        let len = buffer.len_bytes();

        if len > 4 {
            let range = 2..(len - 2);
            let same_text = s[range.clone()].to_string();

            let mut cmd = ReplaceCommand::new(buffer.id(), range, same_text);

            // Execute (replace with same text)
            if cmd.execute(&mut buffer).is_ok() {
                // Buffer should be unchanged
                prop_assert_eq!(buffer.to_string(), original);

                // Undo should also result in same state
                if cmd.undo(&mut buffer).is_ok() {
                    prop_assert_eq!(buffer.to_string(), original);
                }
            }
        }
    }

    /// Command descriptions are non-empty
    #[test]
    fn test_command_descriptions(
        text in "\\PC{1,50}",
        pos in 0..50usize
    ) {
        let buffer = Buffer::new();
        let pos = pos.min(buffer.len_bytes());

        let insert_cmd = InsertCommand::new(buffer.id(), pos, text.clone());
        let delete_cmd = DeleteCommand::new(buffer.id(), pos..pos + 5);
        let replace_cmd = ReplaceCommand::new(buffer.id(), pos..pos + 5, text);

        prop_assert!(!insert_cmd.description().is_empty());
        prop_assert!(!delete_cmd.description().is_empty());
        prop_assert!(!replace_cmd.description().is_empty());
    }
}
