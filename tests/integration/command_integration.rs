//! Integration tests for Command system and undo/redo

use ait42_core::buffer::Buffer;
use ait42_core::command::{
    Command, CommandHistory, DeleteCommand, InsertCommand, ReplaceCommand,
};

#[test]
fn test_command_undo_redo_chain() {
    let mut buffer = Buffer::new();
    let mut history = CommandHistory::new();

    // Execute chain of commands
    let mut cmd1 = InsertCommand::new(buffer.id(), 0, "Hello");
    cmd1.execute(&mut buffer).unwrap();
    history.push(Box::new(cmd1));

    let mut cmd2 = InsertCommand::new(buffer.id(), 5, " World");
    cmd2.execute(&mut buffer).unwrap();
    history.push(Box::new(cmd2));

    let mut cmd3 = InsertCommand::new(buffer.id(), 11, "!");
    cmd3.execute(&mut buffer).unwrap();
    history.push(Box::new(cmd3));

    assert_eq!(buffer.to_string(), "Hello World!");

    // Undo all
    history.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hello World");

    history.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hello");

    history.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "");

    // Redo all
    history.redo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hello");

    history.redo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hello World");

    history.redo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hello World!");
}

#[test]
fn test_command_history_clears_redo_on_new_command() {
    let mut buffer = Buffer::new();
    let mut history = CommandHistory::new();

    // Execute and undo
    let mut cmd1 = InsertCommand::new(buffer.id(), 0, "A");
    cmd1.execute(&mut buffer).unwrap();
    history.push(Box::new(cmd1));

    history.undo(&mut buffer).unwrap();
    assert!(history.can_redo());

    // Execute new command - should clear redo
    let mut cmd2 = InsertCommand::new(buffer.id(), 0, "B");
    cmd2.execute(&mut buffer).unwrap();
    history.push(Box::new(cmd2));

    assert!(!history.can_redo());
}

#[test]
fn test_command_history_max_capacity() {
    let mut buffer = Buffer::new();
    let mut history = CommandHistory::with_capacity(3);

    // Add 5 commands (more than capacity)
    for i in 0..5 {
        let mut cmd = InsertCommand::new(buffer.id(), 0, format!("{}", i));
        cmd.execute(&mut buffer).unwrap();
        history.push(Box::new(cmd));
    }

    // Should only keep last 3
    assert_eq!(history.undo_len(), 3);

    // Undo all available
    history.undo(&mut buffer).unwrap();
    history.undo(&mut buffer).unwrap();
    history.undo(&mut buffer).unwrap();

    // No more undo available
    assert!(!history.can_undo());
}

#[test]
fn test_insert_command_merge() {
    let mut buffer = Buffer::new();
    let mut history = CommandHistory::new();

    // Insert "H" at position 0
    let mut cmd1 = InsertCommand::new(buffer.id(), 0, "H");
    cmd1.execute(&mut buffer).unwrap();
    history.push(Box::new(cmd1));

    // Insert "e" at position 1 - should merge
    let mut cmd2 = InsertCommand::new(buffer.id(), 1, "e");
    cmd2.execute(&mut buffer).unwrap();
    history.push(Box::new(cmd2));

    // Should still have only 1 command due to merge
    assert_eq!(history.undo_len(), 1);

    // Undo should remove both chars
    history.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "");
}

#[test]
fn test_delete_command_saves_text() {
    let mut buffer = Buffer::from_string("Hello World".to_string(), None);
    let mut cmd = DeleteCommand::new(buffer.id(), 5..11);

    cmd.execute(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hello");

    // Undo should restore exact text
    cmd.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hello World");
}

#[test]
fn test_replace_command_preserves_old_text() {
    let mut buffer = Buffer::from_string("Hello World".to_string(), None);
    let mut cmd = ReplaceCommand::new(buffer.id(), 0..5, "Hi".to_string());

    cmd.execute(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hi World");

    cmd.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hello World");
}

#[test]
fn test_command_on_empty_buffer() {
    let mut buffer = Buffer::new();

    let mut cmd = InsertCommand::new(buffer.id(), 0, "First");
    cmd.execute(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "First");

    cmd.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "");
    assert!(buffer.is_empty());
}

#[test]
fn test_command_delete_entire_buffer() {
    let mut buffer = Buffer::from_string("Delete me!".to_string(), None);
    let len = buffer.len_bytes();

    let mut cmd = DeleteCommand::new(buffer.id(), 0..len);
    cmd.execute(&mut buffer).unwrap();
    assert!(buffer.is_empty());

    cmd.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Delete me!");
}

#[test]
fn test_command_replace_entire_buffer() {
    let mut buffer = Buffer::from_string("Old content".to_string(), None);
    let len = buffer.len_bytes();

    let mut cmd = ReplaceCommand::new(buffer.id(), 0..len, "New content".to_string());
    cmd.execute(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "New content");

    cmd.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Old content");
}

#[test]
fn test_command_insert_at_various_positions() {
    let mut buffer = Buffer::from_string("ac".to_string(), None);

    // Insert in middle
    let mut cmd1 = InsertCommand::new(buffer.id(), 1, "b");
    cmd1.execute(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "abc");

    // Insert at start
    let mut cmd2 = InsertCommand::new(buffer.id(), 0, "0");
    cmd2.execute(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "0abc");

    // Insert at end
    let mut cmd3 = InsertCommand::new(buffer.id(), 4, "!");
    cmd3.execute(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "0abc!");

    // Undo all in reverse
    cmd3.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "0abc");

    cmd2.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "abc");

    cmd1.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "ac");
}

#[test]
fn test_command_delete_edge_cases() {
    let mut buffer = Buffer::from_string("abcdef".to_string(), None);

    // Delete single character
    let mut cmd1 = DeleteCommand::new(buffer.id(), 0..1);
    cmd1.execute(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "bcdef");

    cmd1.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "abcdef");

    // Delete empty range (should be no-op)
    let mut cmd2 = DeleteCommand::new(buffer.id(), 2..2);
    cmd2.execute(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "abcdef");
}

#[test]
fn test_command_replace_with_longer_text() {
    let mut buffer = Buffer::from_string("Hi".to_string(), None);

    let mut cmd = ReplaceCommand::new(buffer.id(), 0..2, "Hello World".to_string());
    cmd.execute(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hello World");

    cmd.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hi");
}

#[test]
fn test_command_replace_with_shorter_text() {
    let mut buffer = Buffer::from_string("Hello World".to_string(), None);

    let mut cmd = ReplaceCommand::new(buffer.id(), 0..11, "Hi".to_string());
    cmd.execute(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hi");

    cmd.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hello World");
}

#[test]
fn test_command_history_clear() {
    let mut buffer = Buffer::new();
    let mut history = CommandHistory::new();

    // Add commands
    for i in 0..5 {
        let mut cmd = InsertCommand::new(buffer.id(), 0, format!("{}", i));
        cmd.execute(&mut buffer).unwrap();
        history.push(Box::new(cmd));
    }

    assert!(history.can_undo());
    assert_eq!(history.undo_len(), 5);

    history.clear();

    assert!(!history.can_undo());
    assert!(!history.can_redo());
    assert_eq!(history.undo_len(), 0);
    assert_eq!(history.redo_len(), 0);
}

#[test]
fn test_command_undo_beyond_history() {
    let mut buffer = Buffer::new();
    let mut history = CommandHistory::new();

    // Try undo with empty history
    let result = history.undo(&mut buffer).unwrap();
    assert!(!result); // Should return false, not error
}

#[test]
fn test_command_redo_beyond_history() {
    let mut buffer = Buffer::new();
    let mut history = CommandHistory::new();

    // Try redo with empty history
    let result = history.redo(&mut buffer).unwrap();
    assert!(!result);
}

#[test]
fn test_command_complex_workflow() {
    let mut buffer = Buffer::new();
    let mut history = CommandHistory::new();

    // Simulate typical editing workflow

    // Type "Hello"
    let mut cmd1 = InsertCommand::new(buffer.id(), 0, "Hello");
    cmd1.execute(&mut buffer).unwrap();
    history.push(Box::new(cmd1));

    // Add space
    let mut cmd2 = InsertCommand::new(buffer.id(), 5, " ");
    cmd2.execute(&mut buffer).unwrap();
    history.push(Box::new(cmd2));

    // Type "World"
    let mut cmd3 = InsertCommand::new(buffer.id(), 6, "World");
    cmd3.execute(&mut buffer).unwrap();
    history.push(Box::new(cmd3));

    assert_eq!(buffer.to_string(), "Hello World");

    // Oops, undo "World"
    history.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hello ");

    // Type "Everyone" instead
    let mut cmd4 = InsertCommand::new(buffer.id(), 6, "Everyone");
    cmd4.execute(&mut buffer).unwrap();
    history.push(Box::new(cmd4));

    assert_eq!(buffer.to_string(), "Hello Everyone");

    // Can't redo "World" anymore
    assert!(!history.can_redo());
}

#[test]
fn test_command_unicode_operations() {
    let mut buffer = Buffer::from_string("Hello 世界".to_string(), None);

    // Delete Japanese characters
    let hello_len = "Hello ".len();
    let world_len = "世界".len();

    let mut cmd = DeleteCommand::new(buffer.id(), hello_len..(hello_len + world_len));
    cmd.execute(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hello ");

    // Undo should restore exact UTF-8
    cmd.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Hello 世界");
}

#[test]
fn test_command_multiline_operations() {
    let mut buffer = Buffer::from_string("Line 1\nLine 2\nLine 3".to_string(), None);

    // Delete middle line
    let line1_len = "Line 1\n".len();
    let line2_len = "Line 2\n".len();

    let mut cmd = DeleteCommand::new(buffer.id(), line1_len..(line1_len + line2_len));
    cmd.execute(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Line 1\nLine 3");

    cmd.undo(&mut buffer).unwrap();
    assert_eq!(buffer.to_string(), "Line 1\nLine 2\nLine 3");
}
