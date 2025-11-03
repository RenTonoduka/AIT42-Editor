//! Buffer Management
//!
//! Handles text storage using Rope data structure for efficient editing.

use ropey::Rope;
use std::path::PathBuf;
use uuid::Uuid;

pub type BufferId = Uuid;

/// Text buffer with efficient rope-based storage
#[derive(Debug)]
pub struct Buffer {
    id: BufferId,
    rope: Rope,
    file_path: Option<PathBuf>,
    modified: bool,
    language: Option<String>,
}

impl Buffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            rope: Rope::new(),
            file_path: None,
            modified: false,
            language: None,
        }
    }

    /// Create buffer from file
    pub fn from_file(path: PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let rope = Rope::from_str(&content);

        Ok(Self {
            id: Uuid::new_v4(),
            rope,
            file_path: Some(path),
            modified: false,
            language: None,
        })
    }

    /// Get buffer ID
    pub fn id(&self) -> BufferId {
        self.id
    }

    /// Get file path
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    /// Check if buffer is modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Get buffer content as string
    pub fn content(&self) -> String {
        self.rope.to_string()
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Insert text at position
    pub fn insert(&mut self, line: usize, col: usize, text: &str) {
        // TODO: Implement proper insertion logic
        self.modified = true;
    }

    /// Delete text range
    pub fn delete(&mut self, start_line: usize, start_col: usize, end_line: usize, end_col: usize) {
        // TODO: Implement proper deletion logic
        self.modified = true;
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages multiple buffers
#[derive(Debug, Default)]
pub struct BufferManager {
    buffers: Vec<Buffer>,
    active_buffer: Option<BufferId>,
}

impl BufferManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new buffer
    pub fn add(&mut self, buffer: Buffer) -> BufferId {
        let id = buffer.id();
        self.buffers.push(buffer);
        if self.active_buffer.is_none() {
            self.active_buffer = Some(id);
        }
        id
    }

    /// Get active buffer
    pub fn active(&self) -> Option<&Buffer> {
        self.active_buffer
            .and_then(|id| self.buffers.iter().find(|b| b.id() == id))
    }

    /// Get mutable active buffer
    pub fn active_mut(&mut self) -> Option<&mut Buffer> {
        let active_id = self.active_buffer?;
        self.buffers.iter_mut().find(|b| b.id() == active_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buffer = Buffer::new();
        assert!(!buffer.is_modified());
        assert_eq!(buffer.line_count(), 1);
    }

    #[test]
    fn test_buffer_manager() {
        let mut manager = BufferManager::new();
        let buffer = Buffer::new();
        let id = manager.add(buffer);
        assert!(manager.active().is_some());
    }
}
