//! Main Editor Implementation

use crate::{buffer::BufferManager, cursor::Cursor, history::History, Result};
use std::path::PathBuf;

/// Editor configuration
#[derive(Debug, Clone)]
pub struct EditorConfig {
    pub tab_size: usize,
    pub insert_spaces: bool,
    pub line_numbers: bool,
    pub wrap_lines: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: 4,
            insert_spaces: true,
            line_numbers: true,
            wrap_lines: false,
        }
    }
}

/// Main editor state
pub struct Editor {
    config: EditorConfig,
    buffers: BufferManager,
    cursor: Cursor,
    history: History,
}

impl Editor {
    /// Create a new editor instance
    pub fn new(config: EditorConfig) -> Result<Self> {
        Ok(Self {
            config,
            buffers: BufferManager::new(),
            cursor: Cursor::default(),
            history: History::new(),
        })
    }

    /// Open a file into a buffer
    pub async fn open_file(&mut self, path: impl Into<PathBuf>) -> Result<()> {
        // TODO: Implement file opening
        Ok(())
    }

    /// Insert text at cursor position
    pub fn insert_text(&mut self, text: &str) -> Result<()> {
        // TODO: Implement text insertion
        Ok(())
    }

    /// Get buffer manager
    pub fn buffers(&self) -> &BufferManager {
        &self.buffers
    }

    /// Get mutable buffer manager
    pub fn buffers_mut(&mut self) -> &mut BufferManager {
        &mut self.buffers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        let config = EditorConfig::default();
        let editor = Editor::new(config);
        assert!(editor.is_ok());
    }
}
