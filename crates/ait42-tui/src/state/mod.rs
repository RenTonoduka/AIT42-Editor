//! State Management for AIT42 TUI Editor
//!
//! This module provides state structures for managing the editor's
//! UI state, including tabs, cursor position, panel visibility, focus,
//! and terminal command execution.

use std::path::PathBuf;
use crate::terminal_executor::TerminalExecutor;
use anyhow::Result;

/// Represents which panel is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    /// File explorer sidebar
    Sidebar,
    /// Main text editor
    Editor,
    /// Integrated terminal
    Terminal,
}

impl Default for FocusedPanel {
    fn default() -> Self {
        Self::Editor
    }
}

/// Tab information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    /// Display title
    pub title: String,
    /// File path (if applicable)
    pub file_path: Option<PathBuf>,
    /// Whether content has been modified
    pub modified: bool,
}

impl Tab {
    /// Create a new tab with the given title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            file_path: None,
            modified: false,
        }
    }

    /// Set the file path for this tab
    pub fn with_file_path(mut self, path: PathBuf) -> Self {
        self.file_path = Some(path);
        self
    }

    /// Mark this tab as modified
    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }
}

/// Cursor position in the editor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPosition {
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number (0-indexed)
    pub column: usize,
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self { line: 0, column: 0 }
    }
}

impl CursorPosition {
    /// Create a new cursor position
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// View state (scroll offset)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewState {
    /// First visible line
    pub scroll_offset: usize,
}

impl Default for ViewState {
    fn default() -> Self {
        Self { scroll_offset: 0 }
    }
}

/// Text buffer content
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBuffer {
    /// Lines of text
    pub lines: Vec<String>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }
}

impl TextBuffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Create buffer from lines
    pub fn from_lines(lines: Vec<String>) -> Self {
        Self { lines }
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get line at index
    pub fn get_line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|s| s.as_str())
    }
}

/// Complete editor state
#[derive(Debug)]
pub struct EditorState {
    /// Open tabs
    pub tabs: Vec<Tab>,
    /// Active tab index
    pub active_tab_index: usize,
    /// Current text buffer
    pub buffer: TextBuffer,
    /// Cursor position
    pub cursor: CursorPosition,
    /// View state (scroll)
    pub view: ViewState,
    /// Sidebar visibility
    pub sidebar_visible: bool,
    /// Sidebar selected item index
    pub sidebar_selected: usize,
    /// Terminal visibility
    pub terminal_visible: bool,
    /// Terminal scroll offset
    pub terminal_scroll: usize,
    /// Currently focused panel
    pub focused_panel: FocusedPanel,
    /// Terminal command executor
    terminal_executor: TerminalExecutor,
    /// Terminal input buffer
    terminal_input: String,
    /// Terminal cursor position
    terminal_cursor_pos: usize,
    /// Terminal history navigation index
    terminal_history_index: Option<usize>,
    /// Working directory
    working_dir: PathBuf,
}

impl Default for EditorState {
    fn default() -> Self {
        let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            tabs: vec![Tab::new("Untitled")],
            active_tab_index: 0,
            buffer: TextBuffer::default(),
            cursor: CursorPosition::default(),
            view: ViewState::default(),
            sidebar_visible: true,
            sidebar_selected: 0,
            terminal_visible: true,
            terminal_scroll: 0,
            focused_panel: FocusedPanel::Editor,
            terminal_executor: TerminalExecutor::new(working_dir.clone()),
            terminal_input: String::new(),
            terminal_cursor_pos: 0,
            terminal_history_index: None,
            working_dir,
        }
    }
}

impl EditorState {
    /// Create a new editor state
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the active tab
    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab_index)
    }

    /// Get mutable reference to active tab
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_tab_index)
    }

    /// Add a new tab
    pub fn add_tab(&mut self, tab: Tab) {
        self.tabs.push(tab);
        self.active_tab_index = self.tabs.len() - 1;
    }

    /// Close the active tab
    pub fn close_active_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab_index);
            if self.active_tab_index >= self.tabs.len() {
                self.active_tab_index = self.tabs.len() - 1;
            }
        }
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
        }
    }

    /// Switch to previous tab
    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_index = if self.active_tab_index == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab_index - 1
            };
        }
    }

    /// Toggle sidebar visibility
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
    }

    /// Toggle terminal visibility
    pub fn toggle_terminal(&mut self) {
        self.terminal_visible = !self.terminal_visible;
    }

    /// Focus next panel (cycle: Editor -> Sidebar -> Terminal -> Editor)
    pub fn focus_next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Editor => {
                if self.sidebar_visible {
                    FocusedPanel::Sidebar
                } else if self.terminal_visible {
                    FocusedPanel::Terminal
                } else {
                    FocusedPanel::Editor
                }
            }
            FocusedPanel::Sidebar => {
                if self.terminal_visible {
                    FocusedPanel::Terminal
                } else {
                    FocusedPanel::Editor
                }
            }
            FocusedPanel::Terminal => FocusedPanel::Editor,
        };
    }

    /// Focus previous panel
    pub fn focus_prev_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Editor => {
                if self.terminal_visible {
                    FocusedPanel::Terminal
                } else if self.sidebar_visible {
                    FocusedPanel::Sidebar
                } else {
                    FocusedPanel::Editor
                }
            }
            FocusedPanel::Terminal => {
                if self.sidebar_visible {
                    FocusedPanel::Sidebar
                } else {
                    FocusedPanel::Editor
                }
            }
            FocusedPanel::Sidebar => FocusedPanel::Editor,
        };
    }

    // ==========================================
    // Terminal Methods
    // ==========================================

    /// Execute a terminal command
    pub async fn execute_terminal_command(&mut self, command: String) -> Result<()> {
        self.terminal_executor.execute(&command).await?;
        Ok(())
    }

    /// Submit the current terminal input
    pub async fn submit_terminal_input(&mut self) -> Result<()> {
        let command = self.terminal_input.clone();
        self.terminal_input.clear();
        self.terminal_cursor_pos = 0;
        self.terminal_history_index = None;

        self.execute_terminal_command(command).await
    }

    /// Get terminal output lines
    pub fn terminal_output(&self) -> &[String] {
        self.terminal_executor.get_output()
    }

    /// Get last N lines of terminal output
    pub fn terminal_output_tail(&self, n: usize) -> &[String] {
        self.terminal_executor.get_output_tail(n)
    }

    /// Clear terminal output
    pub fn clear_terminal(&mut self) {
        self.terminal_executor.clear();
    }

    /// Get terminal input buffer
    pub fn terminal_input(&self) -> &str {
        &self.terminal_input
    }

    /// Get terminal cursor position
    pub fn terminal_cursor_pos(&self) -> usize {
        self.terminal_cursor_pos
    }

    /// Append character to terminal input
    pub fn terminal_input_char(&mut self, ch: char) {
        self.terminal_input.insert(self.terminal_cursor_pos, ch);
        self.terminal_cursor_pos += 1;
    }

    /// Delete character before cursor (backspace)
    pub fn terminal_backspace(&mut self) {
        if self.terminal_cursor_pos > 0 {
            self.terminal_cursor_pos -= 1;
            self.terminal_input.remove(self.terminal_cursor_pos);
        }
    }

    /// Delete character at cursor
    pub fn terminal_delete(&mut self) {
        if self.terminal_cursor_pos < self.terminal_input.len() {
            self.terminal_input.remove(self.terminal_cursor_pos);
        }
    }

    /// Move terminal cursor left
    pub fn terminal_cursor_left(&mut self) {
        if self.terminal_cursor_pos > 0 {
            self.terminal_cursor_pos -= 1;
        }
    }

    /// Move terminal cursor right
    pub fn terminal_cursor_right(&mut self) {
        if self.terminal_cursor_pos < self.terminal_input.len() {
            self.terminal_cursor_pos += 1;
        }
    }

    /// Move terminal cursor to start
    pub fn terminal_cursor_home(&mut self) {
        self.terminal_cursor_pos = 0;
    }

    /// Move terminal cursor to end
    pub fn terminal_cursor_end(&mut self) {
        self.terminal_cursor_pos = self.terminal_input.len();
    }

    /// Navigate terminal history up (older commands)
    pub fn terminal_history_up(&mut self) {
        let history_len = self.terminal_executor.history().len();
        if history_len == 0 {
            return;
        }

        let new_index = match self.terminal_history_index {
            None => 0,
            Some(idx) if idx + 1 < history_len => idx + 1,
            Some(idx) => idx,
        };

        self.terminal_history_index = Some(new_index);
        if let Some(cmd) = self.terminal_executor.history_entry(new_index) {
            self.terminal_input = cmd.to_string();
            self.terminal_cursor_pos = self.terminal_input.len();
        }
    }

    /// Navigate terminal history down (newer commands)
    pub fn terminal_history_down(&mut self) {
        if let Some(idx) = self.terminal_history_index {
            if idx > 0 {
                let new_index = idx - 1;
                self.terminal_history_index = Some(new_index);
                if let Some(cmd) = self.terminal_executor.history_entry(new_index) {
                    self.terminal_input = cmd.to_string();
                    self.terminal_cursor_pos = self.terminal_input.len();
                }
            } else {
                self.terminal_history_index = None;
                self.terminal_input.clear();
                self.terminal_cursor_pos = 0;
            }
        }
    }

    /// Get working directory
    pub fn working_dir(&self) -> &PathBuf {
        &self.working_dir
    }

    /// Set working directory
    pub fn set_working_dir(&mut self, dir: PathBuf) {
        self.working_dir = dir.clone();
        self.terminal_executor.set_current_dir(dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = EditorState::default();
        assert_eq!(state.tabs.len(), 1);
        assert_eq!(state.active_tab_index, 0);
        assert_eq!(state.focused_panel, FocusedPanel::Editor);
        assert!(state.sidebar_visible);
        assert!(state.terminal_visible);
    }

    #[test]
    fn test_tab_navigation() {
        let mut state = EditorState::new();
        state.add_tab(Tab::new("File 1"));
        state.add_tab(Tab::new("File 2"));

        assert_eq!(state.tabs.len(), 3);
        assert_eq!(state.active_tab_index, 2);

        state.prev_tab();
        assert_eq!(state.active_tab_index, 1);

        state.next_tab();
        assert_eq!(state.active_tab_index, 2);
    }

    #[test]
    fn test_panel_focus_cycling() {
        let mut state = EditorState::new();

        assert_eq!(state.focused_panel, FocusedPanel::Editor);

        state.focus_next_panel();
        assert_eq!(state.focused_panel, FocusedPanel::Sidebar);

        state.focus_next_panel();
        assert_eq!(state.focused_panel, FocusedPanel::Terminal);

        state.focus_next_panel();
        assert_eq!(state.focused_panel, FocusedPanel::Editor);
    }

    #[test]
    fn test_toggle_visibility() {
        let mut state = EditorState::new();

        assert!(state.sidebar_visible);
        state.toggle_sidebar();
        assert!(!state.sidebar_visible);

        assert!(state.terminal_visible);
        state.toggle_terminal();
        assert!(!state.terminal_visible);
    }
}
