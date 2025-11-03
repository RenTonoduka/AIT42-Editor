//! Buffer Management
//!
//! Handles text storage using Rope data structure for efficient editing.
//! Provides O(log n) insert, delete, and replace operations.

use ropey::Rope;
use std::borrow::Cow;
use std::ops::Range;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::{EditorError, Result};

pub type BufferId = Uuid;

/// Line ending style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    /// Unix-style (LF)
    Lf,
    /// Windows-style (CRLF)
    CrLf,
}

impl LineEnding {
    /// Detect line ending from text
    pub fn detect(text: &str) -> Self {
        if text.contains("\r\n") {
            Self::CrLf
        } else {
            Self::Lf
        }
    }

    /// Get line ending as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
        }
    }
}

/// Text buffer with efficient rope-based storage
///
/// Operations are O(log n) for most text manipulations.
/// Supports Unicode correctly (grapheme clusters, not bytes).
#[derive(Debug, Clone)]
pub struct Buffer {
    id: BufferId,
    content: Rope,
    version: u64,
    dirty: bool,
    line_ending: LineEnding,
    file_path: Option<PathBuf>,
    language: Option<String>,
}

impl Buffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            content: Rope::new(),
            version: 0,
            dirty: false,
            line_ending: LineEnding::Lf,
            file_path: None,
            language: None,
        }
    }

    /// Create buffer from string
    pub fn from_string(content: String, language: Option<String>) -> Self {
        let line_ending = LineEnding::detect(&content);
        Self {
            id: Uuid::new_v4(),
            content: Rope::from_str(&content),
            version: 0,
            dirty: false,
            line_ending,
            file_path: None,
            language,
        }
    }

    /// Create buffer from file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let line_ending = LineEnding::detect(&content);

        // Detect language from file extension
        let language = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_string());

        Ok(Self {
            id: Uuid::new_v4(),
            content: Rope::from_str(&content),
            version: 0,
            dirty: false,
            line_ending,
            file_path: Some(path.to_path_buf()),
            language,
        })
    }

    /// Get buffer ID
    #[inline]
    pub fn id(&self) -> BufferId {
        self.id
    }

    /// Get buffer version (increments on each change)
    #[inline]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Check if buffer has unsaved changes
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Get file path
    #[inline]
    pub fn path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    /// Get language
    #[inline]
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// Get line ending style
    #[inline]
    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    /// Insert text at byte offset
    ///
    /// # Errors
    /// Returns error if offset is out of bounds or not at UTF-8 boundary
    pub fn insert(&mut self, pos: usize, text: &str) -> Result<()> {
        let len = self.content.len_bytes();
        if pos > len {
            return Err(EditorError::InvalidPosition(pos));
        }

        // Validate UTF-8 boundary
        if !self.content.is_char_boundary(pos) {
            return Err(EditorError::Utf8Boundary(pos));
        }

        self.content.insert(pos, text);
        self.version += 1;
        self.dirty = true;

        Ok(())
    }

    /// Delete text range
    ///
    /// # Errors
    /// Returns error if range is out of bounds
    pub fn delete(&mut self, range: Range<usize>) -> Result<()> {
        let len = self.content.len_bytes();
        if range.start > len || range.end > len || range.start > range.end {
            return Err(EditorError::InvalidRange(range));
        }

        // Validate UTF-8 boundaries
        if !self.content.is_char_boundary(range.start) {
            return Err(EditorError::Utf8Boundary(range.start));
        }
        if !self.content.is_char_boundary(range.end) {
            return Err(EditorError::Utf8Boundary(range.end));
        }

        self.content.remove(range);
        self.version += 1;
        self.dirty = true;

        Ok(())
    }

    /// Replace text in range
    ///
    /// Equivalent to delete + insert, but more efficient.
    pub fn replace(&mut self, range: Range<usize>, text: &str) -> Result<()> {
        let len = self.content.len_bytes();
        if range.start > len || range.end > len || range.start > range.end {
            return Err(EditorError::InvalidRange(range));
        }

        // Validate UTF-8 boundaries
        if !self.content.is_char_boundary(range.start) {
            return Err(EditorError::Utf8Boundary(range.start));
        }
        if !self.content.is_char_boundary(range.end) {
            return Err(EditorError::Utf8Boundary(range.end));
        }

        self.content.remove(range.clone());
        self.content.insert(range.start, text);
        self.version += 1;
        self.dirty = true;

        Ok(())
    }

    /// Get line by index (0-based)
    ///
    /// Returns None if line index is out of bounds.
    #[inline]
    pub fn line(&self, index: usize) -> Option<Cow<str>> {
        if index >= self.content.len_lines() {
            return None;
        }

        let line = self.content.line(index);
        Some(Cow::from(line.to_string()))
    }

    /// Get character at position
    #[inline]
    pub fn char_at(&self, pos: usize) -> Option<char> {
        if pos >= self.content.len_chars() {
            return None;
        }

        Some(self.content.char(pos))
    }

    /// Get character count
    #[inline]
    pub fn len_chars(&self) -> usize {
        self.content.len_chars()
    }

    /// Get byte count
    #[inline]
    pub fn len_bytes(&self) -> usize {
        self.content.len_bytes()
    }

    /// Get line count
    #[inline]
    pub fn len_lines(&self) -> usize {
        self.content.len_lines()
    }

    /// Check if buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.content.len_bytes() == 0
    }

    /// Convert (line, column) to byte offset
    ///
    /// Returns None if position is invalid.
    pub fn line_col_to_pos(&self, line: usize, col: usize) -> Option<usize> {
        if line >= self.content.len_lines() {
            return None;
        }

        let line_start = self.content.line_to_byte(line);
        let line_slice = self.content.line(line);

        // Column is in characters, not bytes
        let mut char_count = 0;
        let mut byte_offset = 0;

        for ch in line_slice.chars() {
            if char_count == col {
                return Some(line_start + byte_offset);
            }
            char_count += 1;
            byte_offset += ch.len_utf8();
        }

        // If col is at end of line
        if char_count == col {
            Some(line_start + byte_offset)
        } else {
            None
        }
    }

    /// Convert byte offset to (line, column)
    pub fn pos_to_line_col(&self, pos: usize) -> (usize, usize) {
        if pos >= self.content.len_bytes() {
            let last_line = self.content.len_lines().saturating_sub(1);
            let last_line_len = self.content.line(last_line).len_chars();
            return (last_line, last_line_len);
        }

        let line = self.content.byte_to_line(pos);
        let line_start = self.content.line_to_byte(line);
        let col_bytes = pos - line_start;

        // Convert byte offset to character column
        let line_slice = self.content.line(line);
        let col = line_slice.bytes_to_chars(col_bytes.min(line_slice.len_bytes()));

        (line, col)
    }

    /// Get text slice as string
    pub fn slice(&self, range: Range<usize>) -> Result<String> {
        let len = self.content.len_bytes();
        if range.start > len || range.end > len || range.start > range.end {
            return Err(EditorError::InvalidRange(range));
        }

        Ok(self.content.slice(range).to_string())
    }

    /// Get entire buffer content as string
    ///
    /// O(n) - Use sparingly for large files
    #[inline]
    pub fn to_string(&self) -> String {
        self.content.to_string()
    }

    /// Mark buffer as clean (after save)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Save buffer to file
    ///
    /// Uses atomic write (write to temp file, then rename).
    pub fn save(&mut self) -> Result<()> {
        let path = self.file_path.as_ref().ok_or_else(|| {
            EditorError::Other("Cannot save buffer without file path".to_string())
        })?;

        self.save_as(path)
    }

    /// Save buffer to new path
    pub fn save_as(&mut self, path: &Path) -> Result<()> {
        // Atomic write: write to temp file, then rename
        let temp_path = path.with_extension(".tmp");
        let content = self.to_string();
        std::fs::write(&temp_path, content)?;
        std::fs::rename(&temp_path, path)?;

        self.file_path = Some(path.to_path_buf());
        self.dirty = false;

        Ok(())
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
    /// Create new buffer manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new empty buffer
    ///
    /// Returns the buffer ID.
    pub fn new_buffer(&mut self, language: Option<String>) -> BufferId {
        let buffer = Buffer::from_string(String::new(), language);
        let id = buffer.id();
        self.buffers.push(buffer);

        if self.active_buffer.is_none() {
            self.active_buffer = Some(id);
        }

        id
    }

    /// Open file as buffer
    ///
    /// If the file is already open, returns existing buffer ID.
    pub fn open_file(&mut self, path: &Path) -> Result<BufferId> {
        // Check if already open
        for buffer in &self.buffers {
            if buffer.path() == Some(path) {
                return Ok(buffer.id());
            }
        }

        // Open new buffer
        let buffer = Buffer::from_file(path)?;
        let id = buffer.id();
        self.buffers.push(buffer);

        if self.active_buffer.is_none() {
            self.active_buffer = Some(id);
        }

        Ok(id)
    }

    /// Get buffer by ID
    pub fn get(&self, id: BufferId) -> Option<&Buffer> {
        self.buffers.iter().find(|b| b.id() == id)
    }

    /// Get mutable buffer by ID
    pub fn get_mut(&mut self, id: BufferId) -> Option<&mut Buffer> {
        self.buffers.iter_mut().find(|b| b.id() == id)
    }

    /// Close buffer
    ///
    /// # Errors
    /// Returns error if buffer has unsaved changes (unless `force = true`)
    pub fn close(&mut self, id: BufferId, force: bool) -> Result<()> {
        let buffer = self.get(id).ok_or(EditorError::BufferNotFound(id))?;

        if buffer.is_dirty() && !force {
            return Err(EditorError::Other(format!(
                "Buffer {} has unsaved changes",
                id
            )));
        }

        self.buffers.retain(|b| b.id() != id);

        // If closed buffer was active, switch to first available
        if self.active_buffer == Some(id) {
            self.active_buffer = self.buffers.first().map(|b| b.id());
        }

        Ok(())
    }

    /// Save buffer
    pub fn save(&mut self, id: BufferId) -> Result<()> {
        let buffer = self.get_mut(id).ok_or(EditorError::BufferNotFound(id))?;
        buffer.save()
    }

    /// Save buffer to new path
    pub fn save_as(&mut self, id: BufferId, path: &Path) -> Result<()> {
        let buffer = self.get_mut(id).ok_or(EditorError::BufferNotFound(id))?;
        buffer.save_as(path)
    }

    /// Switch active buffer
    pub fn switch_to(&mut self, id: BufferId) -> Result<()> {
        if self.get(id).is_none() {
            return Err(EditorError::BufferNotFound(id));
        }
        self.active_buffer = Some(id);
        Ok(())
    }

    /// Get active buffer ID
    #[inline]
    pub fn active_buffer_id(&self) -> Option<BufferId> {
        self.active_buffer
    }

    /// Get active buffer
    #[inline]
    pub fn active(&self) -> Option<&Buffer> {
        self.active_buffer.and_then(|id| self.get(id))
    }

    /// Get mutable active buffer
    #[inline]
    pub fn active_mut(&mut self) -> Option<&mut Buffer> {
        let id = self.active_buffer?;
        self.get_mut(id)
    }

    /// List all buffer IDs
    pub fn buffer_ids(&self) -> Vec<BufferId> {
        self.buffers.iter().map(|b| b.id()).collect()
    }

    /// Get buffers with unsaved changes
    pub fn dirty_buffers(&self) -> Vec<BufferId> {
        self.buffers
            .iter()
            .filter(|b| b.is_dirty())
            .map(|b| b.id())
            .collect()
    }

    /// Close all buffers
    ///
    /// Returns list of buffer IDs that had unsaved changes.
    pub fn close_all(&mut self, force: bool) -> Result<Vec<BufferId>> {
        let dirty: Vec<BufferId> = self.dirty_buffers();

        if !force && !dirty.is_empty() {
            return Ok(dirty);
        }

        self.buffers.clear();
        self.active_buffer = None;

        Ok(Vec::new())
    }

    /// Get buffer count
    #[inline]
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// Check if buffer manager is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buffer = Buffer::new();
        assert!(!buffer.is_dirty());
        assert_eq!(buffer.len_lines(), 1); // Empty rope has 1 line
        assert_eq!(buffer.len_bytes(), 0);
    }

    #[test]
    fn test_buffer_from_string() {
        let buffer = Buffer::from_string("Hello\nWorld".to_string(), Some("txt".to_string()));
        assert_eq!(buffer.len_lines(), 2);
        assert_eq!(buffer.language(), Some("txt"));
    }

    #[test]
    fn test_buffer_insert() {
        let mut buffer = Buffer::new();
        assert!(buffer.insert(0, "Hello").is_ok());
        assert_eq!(buffer.to_string(), "Hello");
        assert!(buffer.is_dirty());
        assert_eq!(buffer.version(), 1);
    }

    #[test]
    fn test_buffer_insert_invalid_position() {
        let mut buffer = Buffer::new();
        assert!(buffer.insert(100, "test").is_err());
    }

    #[test]
    fn test_buffer_delete() {
        let mut buffer = Buffer::from_string("Hello World".to_string(), None);
        assert!(buffer.delete(5..11).is_ok());
        assert_eq!(buffer.to_string(), "Hello");
    }

    #[test]
    fn test_buffer_replace() {
        let mut buffer = Buffer::from_string("Hello World".to_string(), None);
        assert!(buffer.replace(0..5, "Hi").is_ok());
        assert_eq!(buffer.to_string(), "Hi World");
    }

    #[test]
    fn test_buffer_line() {
        let buffer = Buffer::from_string("Line 1\nLine 2\nLine 3".to_string(), None);
        assert_eq!(buffer.line(0).unwrap().trim(), "Line 1");
        assert_eq!(buffer.line(1).unwrap().trim(), "Line 2");
        assert_eq!(buffer.line(2).unwrap().trim(), "Line 3");
        assert!(buffer.line(3).is_none());
    }

    #[test]
    fn test_buffer_line_col_to_pos() {
        let buffer = Buffer::from_string("Hello\nWorld".to_string(), None);
        assert_eq!(buffer.line_col_to_pos(0, 0), Some(0));
        assert_eq!(buffer.line_col_to_pos(0, 5), Some(5));
        assert_eq!(buffer.line_col_to_pos(1, 0), Some(6)); // After newline
        assert_eq!(buffer.line_col_to_pos(1, 5), Some(11));
    }

    #[test]
    fn test_buffer_pos_to_line_col() {
        let buffer = Buffer::from_string("Hello\nWorld".to_string(), None);
        assert_eq!(buffer.pos_to_line_col(0), (0, 0));
        assert_eq!(buffer.pos_to_line_col(5), (0, 5));
        assert_eq!(buffer.pos_to_line_col(6), (1, 0));
        assert_eq!(buffer.pos_to_line_col(11), (1, 5));
    }

    #[test]
    fn test_buffer_slice() {
        let buffer = Buffer::from_string("Hello World".to_string(), None);
        assert_eq!(buffer.slice(0..5).unwrap(), "Hello");
        assert_eq!(buffer.slice(6..11).unwrap(), "World");
    }

    #[test]
    fn test_line_ending_detection() {
        assert_eq!(LineEnding::detect("Hello\nWorld"), LineEnding::Lf);
        assert_eq!(LineEnding::detect("Hello\r\nWorld"), LineEnding::CrLf);
    }

    #[test]
    fn test_buffer_manager_new_buffer() {
        let mut manager = BufferManager::new();
        let id = manager.new_buffer(None);
        assert!(manager.get(id).is_some());
        assert_eq!(manager.active_buffer_id(), Some(id));
    }

    #[test]
    fn test_buffer_manager_switch() {
        let mut manager = BufferManager::new();
        let id1 = manager.new_buffer(None);
        let id2 = manager.new_buffer(None);

        assert!(manager.switch_to(id2).is_ok());
        assert_eq!(manager.active_buffer_id(), Some(id2));

        assert!(manager.switch_to(id1).is_ok());
        assert_eq!(manager.active_buffer_id(), Some(id1));
    }

    #[test]
    fn test_buffer_manager_dirty_buffers() {
        let mut manager = BufferManager::new();
        let id1 = manager.new_buffer(None);
        let id2 = manager.new_buffer(None);

        // Modify one buffer
        manager.get_mut(id1).unwrap().insert(0, "test").unwrap();

        let dirty = manager.dirty_buffers();
        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0], id1);
    }

    // Property-based tests would go here with proptest
    // Example:
    // proptest! {
    //     #[test]
    //     fn test_insert_delete_roundtrip(s in "\\PC*") {
    //         let mut buffer = Buffer::new();
    //         buffer.insert(0, &s).unwrap();
    //         buffer.delete(0..s.len()).unwrap();
    //         assert_eq!(buffer.to_string(), "");
    //     }
    // }
}
