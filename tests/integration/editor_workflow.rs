//! End-to-end editor workflow tests

use ait42_core::{Buffer, EditorState, InsertCommand, Mode};
use tempfile::TempDir;

#[test]
fn test_complete_editing_session() {
    let mut state = EditorState::new();

    // Start with empty buffer
    let buffer = Buffer::new();
    let id = state.open_buffer(buffer);

    // Verify initial state
    assert_eq!(state.buffer_count(), 1);
    assert_eq!(state.active_buffer_id(), Some(id));
    assert!(state.mode.is_normal());

    // Enter insert mode and type
    state.mode.enter_insert();
    assert!(state.mode.is_insert());

    let cmd = Box::new(InsertCommand::new(id, 0, "fn main() {}\n"));
    state.execute_command(cmd).unwrap();

    // Exit insert mode
    state.mode.exit_insert();
    assert!(state.mode.is_normal());

    // Verify content
    assert_eq!(state.active_buffer().unwrap().to_string(), "fn main() {}\n");
    assert!(state.active_buffer().unwrap().is_dirty());
}

#[test]
fn test_multiple_buffer_workflow() {
    let mut state = EditorState::new();

    // Open multiple buffers
    let buf1 = Buffer::from_string("Buffer 1 content".to_string(), Some("txt".to_string()));
    let buf2 = Buffer::from_string("Buffer 2 content".to_string(), Some("md".to_string()));
    let buf3 = Buffer::from_string("Buffer 3 content".to_string(), None);

    let id1 = state.open_buffer(buf1);
    let id2 = state.open_buffer(buf2);
    let id3 = state.open_buffer(buf3);

    assert_eq!(state.buffer_count(), 3);

    // Switch between buffers
    state.switch_buffer(id2).unwrap();
    assert_eq!(state.active_buffer_id(), Some(id2));
    assert_eq!(
        state.active_buffer().unwrap().language(),
        Some("md")
    );

    state.switch_buffer(id1).unwrap();
    assert_eq!(state.active_buffer_id(), Some(id1));

    // Close buffer
    state.close_buffer(id2, false).unwrap();
    assert_eq!(state.buffer_count(), 2);
}

#[test]
fn test_save_and_reload_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");

    let mut state = EditorState::new();

    // Create and edit buffer
    let buffer = Buffer::from_string("// Comment\nfn test() {}".to_string(), Some("rs".to_string()));
    let id = state.open_buffer(buffer);

    // Make it dirty
    let cmd = Box::new(InsertCommand::new(id, 0, "// Header\n"));
    state.execute_command(cmd).unwrap();

    // Save
    state.get_buffer_mut(id).unwrap().save_as(&file_path).unwrap();
    assert!(!state.active_buffer().unwrap().is_dirty());

    // Close and reopen
    state.close_buffer(id, false).unwrap();

    let new_buffer = Buffer::from_file(&file_path).unwrap();
    let new_id = state.open_buffer(new_buffer);

    assert_eq!(
        state.get_buffer(new_id).unwrap().to_string(),
        "// Header\n// Comment\nfn test() {}"
    );
}

#[test]
fn test_undo_redo_workflow() {
    let mut state = EditorState::new();
    let buffer = Buffer::new();
    let id = state.open_buffer(buffer);

    // Execute series of commands
    let cmd1 = Box::new(InsertCommand::new(id, 0, "First\n"));
    state.execute_command(cmd1).unwrap();

    let cmd2 = Box::new(InsertCommand::new(id, 6, "Second\n"));
    state.execute_command(cmd2).unwrap();

    let cmd3 = Box::new(InsertCommand::new(id, 13, "Third\n"));
    state.execute_command(cmd3).unwrap();

    assert_eq!(
        state.active_buffer().unwrap().to_string(),
        "First\nSecond\nThird\n"
    );

    // Undo twice
    state.undo().unwrap();
    state.undo().unwrap();

    assert_eq!(
        state.active_buffer().unwrap().to_string(),
        "First\n"
    );

    // Redo once
    state.redo().unwrap();

    assert_eq!(
        state.active_buffer().unwrap().to_string(),
        "First\nSecond\n"
    );

    // Execute new command (clears redo stack)
    let cmd4 = Box::new(InsertCommand::new(id, 13, "New\n"));
    state.execute_command(cmd4).unwrap();

    // Can't redo anymore
    let result = state.redo();
    assert!(result.is_err() || !result.unwrap());
}

#[test]
fn test_cursor_and_editing_workflow() {
    let mut state = EditorState::new();
    let buffer = Buffer::from_string("Line 1\nLine 2\nLine 3".to_string(), None);
    let id = state.open_buffer(buffer);

    // Get cursor
    let cursor = state.cursor().unwrap();
    assert_eq!(cursor.pos(), 0);

    // Move cursor
    let buffer_ref = state.active_buffer().unwrap();
    let mut cursor_mut = state.cursor_mut().unwrap();
    cursor_mut.move_down(buffer_ref, 1);

    // Verify cursor moved
    let (line, _) = buffer_ref.pos_to_line_col(cursor_mut.pos());
    assert_eq!(line, 1);
}

#[test]
fn test_mode_transitions_workflow() {
    let mut state = EditorState::new();
    let buffer = Buffer::new();
    state.open_buffer(buffer);

    // Normal mode by default
    assert!(state.mode.is_normal());

    // Enter insert mode
    state.mode.enter_insert();
    assert!(state.mode.is_insert());

    // Exit back to normal
    state.mode.exit_insert();
    assert!(state.mode.is_normal());

    // Enter visual mode
    state.mode.enter_visual();
    assert!(state.mode.is_visual());

    // Exit visual
    state.mode.exit_visual();
    assert!(state.mode.is_normal());

    // Enter command mode
    state.mode.enter_command();
    assert!(state.mode.is_command());

    // Exit command
    state.mode.exit_command();
    assert!(state.mode.is_normal());
}

#[test]
fn test_dirty_buffer_close_prevention() {
    let mut state = EditorState::new();
    let buffer = Buffer::new();
    let id = state.open_buffer(buffer);

    // Make buffer dirty
    let cmd = Box::new(InsertCommand::new(id, 0, "unsaved"));
    state.execute_command(cmd).unwrap();

    // Try to close without force - should fail
    let result = state.close_buffer(id, false);
    assert!(result.is_err());
    assert_eq!(state.buffer_count(), 1);

    // Force close should work
    let result = state.close_buffer(id, true);
    assert!(result.is_ok());
    assert_eq!(state.buffer_count(), 0);
}

#[test]
fn test_selection_workflow() {
    let mut state = EditorState::new();
    let buffer = Buffer::from_string("Select this text".to_string(), None);
    state.open_buffer(buffer);

    // Start selection
    let cursor_mut = state.cursor_mut().unwrap();
    cursor_mut.start_selection();
    cursor_mut.set_pos(6); // Select "Select"

    assert!(cursor_mut.has_selection());
    assert_eq!(cursor_mut.selection(), Some(0..6));

    // Clear selection
    cursor_mut.clear_selection();
    assert!(!cursor_mut.has_selection());
}

#[test]
fn test_empty_buffer_workflow() {
    let mut state = EditorState::new();

    // Start with no buffers
    assert_eq!(state.buffer_count(), 0);
    assert!(state.active_buffer().is_none());
    assert!(state.cursor().is_none());

    // Open first buffer
    let buffer = Buffer::new();
    let id = state.open_buffer(buffer);

    assert_eq!(state.buffer_count(), 1);
    assert!(state.active_buffer().is_some());
    assert!(state.cursor().is_some());

    // Close only buffer
    state.close_buffer(id, false).unwrap();

    assert_eq!(state.buffer_count(), 0);
    assert!(state.active_buffer().is_none());
}

#[test]
fn test_view_state_workflow() {
    let mut state = EditorState::new();
    let buffer = Buffer::from_string("Line 1\n".repeat(100), None);
    let id = state.open_buffer(buffer);

    // Get view state
    if let Some(view) = state.view_state(id) {
        assert_eq!(view.scroll_offset, 0);
        assert_eq!(view.viewport_height, 0); // Default
    }

    // Update view state
    if let Some(view) = state.view_state_mut(id) {
        view.scroll_offset = 10;
        view.viewport_height = 24;
    }

    // Verify update
    if let Some(view) = state.view_state(id) {
        assert_eq!(view.scroll_offset, 10);
        assert_eq!(view.viewport_height, 24);
    }
}

#[test]
fn test_rapid_edit_workflow() {
    let mut state = EditorState::new();
    let buffer = Buffer::new();
    let id = state.open_buffer(buffer);

    // Simulate rapid typing
    for i in 0..100 {
        let cmd = Box::new(InsertCommand::new(id, i, "x"));
        state.execute_command(cmd).unwrap();
    }

    assert_eq!(state.active_buffer().unwrap().to_string().len(), 100);

    // Undo many times
    for _ in 0..50 {
        state.undo().unwrap();
    }

    assert_eq!(state.active_buffer().unwrap().to_string().len(), 50);
}

#[test]
fn test_buffer_switch_preserves_state() {
    let mut state = EditorState::new();

    // Create two buffers with different content
    let buf1 = Buffer::from_string("Buffer 1".to_string(), None);
    let buf2 = Buffer::from_string("Buffer 2".to_string(), None);

    let id1 = state.open_buffer(buf1);
    let id2 = state.open_buffer(buf2);

    // Edit first buffer
    state.switch_buffer(id1).unwrap();
    let cmd = Box::new(InsertCommand::new(id1, 8, " modified"));
    state.execute_command(cmd).unwrap();

    // Switch to second buffer
    state.switch_buffer(id2).unwrap();
    assert_eq!(state.active_buffer().unwrap().to_string(), "Buffer 2");

    // Switch back - should preserve modifications
    state.switch_buffer(id1).unwrap();
    assert_eq!(
        state.active_buffer().unwrap().to_string(),
        "Buffer 1 modified"
    );
}
