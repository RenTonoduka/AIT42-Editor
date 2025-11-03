//! Multi-buffer and concurrent operation tests

use ait42_core::{Buffer, BufferManager, EditorState, InsertCommand};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_ten_buffers_workflow() {
    let mut state = EditorState::new();

    // Open 10 buffers
    let mut ids = Vec::new();
    for i in 0..10 {
        let buffer = Buffer::from_string(format!("Buffer {}", i), None);
        ids.push(state.open_buffer(buffer));
    }

    assert_eq!(state.buffer_count(), 10);

    // Edit each buffer
    for (i, id) in ids.iter().enumerate() {
        state.switch_buffer(*id).unwrap();
        let cmd = Box::new(InsertCommand::new(*id, 0, format!("Header {}\n", i)));
        state.execute_command(cmd).unwrap();
    }

    // Verify all buffers modified
    for (i, id) in ids.iter().enumerate() {
        let buffer = state.get_buffer(*id).unwrap();
        assert!(buffer.to_string().contains(&format!("Header {}", i)));
        assert!(buffer.is_dirty());
    }

    // Close half
    for id in ids.iter().take(5) {
        state.close_buffer(*id, true).unwrap();
    }

    assert_eq!(state.buffer_count(), 5);
}

#[test]
fn test_buffer_manager_iteration() {
    let mut manager = BufferManager::new();

    let id1 = manager.new_buffer(Some("rs".to_string()));
    let id2 = manager.new_buffer(Some("py".to_string()));
    let id3 = manager.new_buffer(Some("js".to_string()));

    // Get all IDs
    let ids = manager.buffer_ids();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
    assert!(ids.contains(&id3));
}

#[test]
fn test_concurrent_buffer_modifications() {
    let mut manager = BufferManager::new();

    let id1 = manager.new_buffer(None);
    let id2 = manager.new_buffer(None);

    // Modify both buffers
    manager.get_mut(id1).unwrap().insert(0, "Buffer 1").unwrap();
    manager.get_mut(id2).unwrap().insert(0, "Buffer 2").unwrap();

    // Both should be dirty
    let dirty = manager.dirty_buffers();
    assert_eq!(dirty.len(), 2);

    // Save one
    manager.get_mut(id1).unwrap().mark_clean();

    // Only one dirty now
    let dirty = manager.dirty_buffers();
    assert_eq!(dirty.len(), 1);
    assert!(dirty.contains(&id2));
}

#[test]
fn test_buffer_manager_active_switching() {
    let mut manager = BufferManager::new();

    let id1 = manager.new_buffer(None);
    let id2 = manager.new_buffer(None);
    let id3 = manager.new_buffer(None);

    // First buffer is active by default
    assert_eq!(manager.active_buffer_id(), Some(id1));

    // Switch through all buffers
    manager.switch_to(id2).unwrap();
    assert_eq!(manager.active_buffer_id(), Some(id2));

    manager.switch_to(id3).unwrap();
    assert_eq!(manager.active_buffer_id(), Some(id3));

    manager.switch_to(id1).unwrap();
    assert_eq!(manager.active_buffer_id(), Some(id1));
}

#[test]
fn test_buffer_manager_close_active() {
    let mut manager = BufferManager::new();

    let id1 = manager.new_buffer(None);
    let id2 = manager.new_buffer(None);
    let id3 = manager.new_buffer(None);

    // Make id2 active
    manager.switch_to(id2).unwrap();
    assert_eq!(manager.active_buffer_id(), Some(id2));

    // Close active buffer
    manager.close(id2, false).unwrap();

    // Should switch to first available buffer
    let active = manager.active_buffer_id();
    assert!(active.is_some());
    assert!(active == Some(id1) || active == Some(id3));
}

#[test]
fn test_multi_file_save_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BufferManager::new();

    // Create multiple files
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    let file3 = temp_dir.path().join("file3.txt");

    fs::write(&file1, "Content 1").unwrap();
    fs::write(&file2, "Content 2").unwrap();
    fs::write(&file3, "Content 3").unwrap();

    // Open all files
    let id1 = manager.open_file(&file1).unwrap();
    let id2 = manager.open_file(&file2).unwrap();
    let id3 = manager.open_file(&file3).unwrap();

    // Modify all
    manager.get_mut(id1).unwrap().insert(9, " modified").unwrap();
    manager.get_mut(id2).unwrap().insert(9, " modified").unwrap();
    manager.get_mut(id3).unwrap().insert(9, " modified").unwrap();

    // Save all
    manager.save(id1).unwrap();
    manager.save(id2).unwrap();
    manager.save(id3).unwrap();

    // Verify files on disk
    assert_eq!(fs::read_to_string(&file1).unwrap(), "Content 1 modified");
    assert_eq!(fs::read_to_string(&file2).unwrap(), "Content 2 modified");
    assert_eq!(fs::read_to_string(&file3).unwrap(), "Content 3 modified");
}

#[test]
fn test_buffer_manager_save_as() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BufferManager::new();

    // Create buffer without file
    let id = manager.new_buffer(None);
    manager.get_mut(id).unwrap().insert(0, "New file content").unwrap();

    // Save as new file
    let file_path = temp_dir.path().join("new_file.txt");
    manager.save_as(id, &file_path).unwrap();

    // Verify
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "New file content");
    assert!(!manager.get(id).unwrap().is_dirty());
    assert_eq!(manager.get(id).unwrap().path(), Some(file_path.as_path()));
}

#[test]
fn test_buffer_manager_invalid_operations() {
    let mut manager = BufferManager::new();
    let id = manager.new_buffer(None);

    // Try to operate on non-existent buffer
    use uuid::Uuid;
    let fake_id = Uuid::new_v4();

    assert!(manager.get(fake_id).is_none());
    assert!(manager.get_mut(fake_id).is_none());
    assert!(manager.switch_to(fake_id).is_err());
    assert!(manager.close(fake_id, false).is_err());
}

#[test]
fn test_large_number_of_buffers() {
    let mut manager = BufferManager::new();

    // Open 100 buffers
    let mut ids = Vec::new();
    for i in 0..100 {
        let id = manager.new_buffer(None);
        manager.get_mut(id).unwrap()
            .insert(0, &format!("Buffer {}", i))
            .unwrap();
        ids.push(id);
    }

    assert_eq!(manager.len(), 100);

    // Verify all buffers exist
    for (i, id) in ids.iter().enumerate() {
        let buffer = manager.get(*id).unwrap();
        assert!(buffer.to_string().contains(&format!("Buffer {}", i)));
    }

    // Close all
    let dirty = manager.close_all(true).unwrap();
    assert_eq!(dirty.len(), 0);
    assert!(manager.is_empty());
}

#[test]
fn test_buffer_close_order() {
    let mut manager = BufferManager::new();

    let id1 = manager.new_buffer(None);
    let id2 = manager.new_buffer(None);
    let id3 = manager.new_buffer(None);

    // Close in different order
    manager.close(id2, false).unwrap();
    assert_eq!(manager.len(), 2);

    manager.close(id1, false).unwrap();
    assert_eq!(manager.len(), 1);

    manager.close(id3, false).unwrap();
    assert_eq!(manager.len(), 0);
}

#[test]
fn test_buffer_active_after_close_all_but_one() {
    let mut manager = BufferManager::new();

    let id1 = manager.new_buffer(None);
    let id2 = manager.new_buffer(None);
    let id3 = manager.new_buffer(None);

    manager.switch_to(id2).unwrap();

    // Close all except id2
    manager.close(id1, false).unwrap();
    manager.close(id3, false).unwrap();

    // id2 should still be active
    assert_eq!(manager.active_buffer_id(), Some(id2));
    assert_eq!(manager.len(), 1);
}

#[test]
fn test_multiple_dirty_buffers_workflow() {
    let mut manager = BufferManager::new();

    // Create 5 buffers, make 3 dirty
    let ids: Vec<_> = (0..5).map(|_| manager.new_buffer(None)).collect();

    manager.get_mut(ids[0]).unwrap().insert(0, "dirty").unwrap();
    manager.get_mut(ids[2]).unwrap().insert(0, "dirty").unwrap();
    manager.get_mut(ids[4]).unwrap().insert(0, "dirty").unwrap();

    let dirty = manager.dirty_buffers();
    assert_eq!(dirty.len(), 3);
    assert!(dirty.contains(&ids[0]));
    assert!(dirty.contains(&ids[2]));
    assert!(dirty.contains(&ids[4]));

    // Try to close all without force
    let prevented = manager.close_all(false).unwrap();
    assert_eq!(prevented.len(), 3);
    assert_eq!(manager.len(), 5); // Nothing closed

    // Force close all
    manager.close_all(true).unwrap();
    assert!(manager.is_empty());
}

#[test]
fn test_buffer_reopen_after_close() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("reopen.txt");
    fs::write(&file_path, "Original content").unwrap();

    let mut manager = BufferManager::new();

    // Open, modify, close without saving
    let id1 = manager.open_file(&file_path).unwrap();
    manager.get_mut(id1).unwrap().insert(16, " modified").unwrap();
    manager.close(id1, true).unwrap(); // Force close without save

    // Reopen - should have original content
    let id2 = manager.open_file(&file_path).unwrap();
    assert_eq!(
        manager.get(id2).unwrap().to_string(),
        "Original content"
    );
}

#[test]
fn test_buffer_manager_empty_operations() {
    let manager = BufferManager::new();

    assert!(manager.is_empty());
    assert_eq!(manager.len(), 0);
    assert!(manager.active_buffer_id().is_none());
    assert!(manager.active().is_none());
    assert_eq!(manager.buffer_ids().len(), 0);
    assert_eq!(manager.dirty_buffers().len(), 0);
}

#[test]
fn test_rapid_buffer_open_close() {
    let mut manager = BufferManager::new();

    // Rapidly open and close buffers
    for _ in 0..50 {
        let id = manager.new_buffer(None);
        manager.get_mut(id).unwrap().insert(0, "test").unwrap();
        manager.close(id, true).unwrap();
    }

    assert!(manager.is_empty());
}
