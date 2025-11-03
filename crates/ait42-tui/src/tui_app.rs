//! TUI Application
//!
//! Main application loop coordinating events, rendering, and state.

use crate::{
    event::{EditorEvent, EventLoop},
    keybinds::{EditorCommand, KeyBinding, KeyMap, Mode},
    layout::LayoutConfig,
    renderer::Renderer,
    theme::Theme,
    widgets::editor::ViewState,
};
use ait42_core::{Buffer, Cursor, Editor, EditorConfig};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use std::{path::PathBuf, time::Duration};
use tracing::{debug, error, info};

/// Tab information
#[derive(Debug, Clone)]
pub struct Tab {
    /// Tab title
    pub title: String,
    /// File path (if any)
    pub path: Option<PathBuf>,
    /// Buffer associated with this tab
    pub buffer: Buffer,
    /// Is modified
    pub is_modified: bool,
}

impl Tab {
    /// Create a new tab
    pub fn new(title: String, path: Option<PathBuf>, buffer: Buffer) -> Self {
        Self {
            title,
            path,
            buffer,
            is_modified: false,
        }
    }
}

/// Sidebar item
#[derive(Debug, Clone)]
pub struct SidebarItem {
    /// Item name
    pub name: String,
    /// Item path
    pub path: PathBuf,
    /// Is directory
    pub is_dir: bool,
    /// Is expanded (for directories)
    pub is_expanded: bool,
    /// Indentation level
    pub level: usize,
}

/// Which panel currently has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    /// Editor panel
    Editor,
    /// Sidebar panel
    Sidebar,
    /// Terminal panel
    Terminal,
}

/// Editor state
pub struct EditorState {
    /// Core editor
    editor: Editor,
    /// Current buffer
    buffer: Buffer,
    /// Cursor
    cursor: Cursor,
    /// View state (scrolling)
    view: ViewState,
    /// Current mode
    mode: Mode,
    /// Command palette input
    command_input: String,
    /// Show command palette
    show_command_palette: bool,
    /// Running flag
    running: bool,

    // Phase 10b: Multi-panel state
    /// Open tabs
    tabs: Vec<Tab>,
    /// Active tab index
    active_tab_index: usize,
    /// Sidebar visibility
    sidebar_visible: bool,
    /// Sidebar items
    sidebar_items: Vec<SidebarItem>,
    /// Sidebar selected index
    sidebar_selected: usize,
    /// Terminal visibility
    terminal_visible: bool,
    /// Terminal scroll offset
    terminal_scroll: usize,
    /// Currently focused panel
    focused_panel: FocusedPanel,
}

impl EditorState {
    /// Create new editor state
    pub fn new(config: EditorConfig) -> Result<Self> {
        let editor = Editor::new(config)?;
        let buffer = Buffer::new();
        let cursor = Cursor::default();
        let view = ViewState::new();

        // Create initial tab
        let initial_tab = Tab::new(
            "untitled".to_string(),
            None,
            buffer.clone(),
        );

        Ok(Self {
            editor,
            buffer,
            cursor,
            view,
            mode: Mode::Normal,
            command_input: String::new(),
            show_command_palette: false,
            running: true,
            // Phase 10b: Initialize multi-panel state
            tabs: vec![initial_tab],
            active_tab_index: 0,
            sidebar_visible: true,
            sidebar_items: Vec::new(),
            sidebar_selected: 0,
            terminal_visible: false,
            terminal_scroll: 0,
            focused_panel: FocusedPanel::Editor,
        })
    }

    /// Load file into buffer
    pub fn load_file(&mut self, path: std::path::PathBuf) -> Result<()> {
        self.buffer = Buffer::from_file(&path)?;
        self.cursor = Cursor::default();
        self.view = ViewState::new();
        Ok(())
    }

    /// Handle editor command
    fn execute_command(&mut self, command: &EditorCommand) -> Result<()> {
        use EditorCommand::*;

        match command {
            // Mode transitions
            EnterInsertMode => {
                self.mode = Mode::Insert;
                debug!("Entered insert mode");
            }
            EnterVisualMode => {
                self.mode = Mode::Visual;
                debug!("Entered visual mode");
            }
            EnterCommandMode => {
                self.mode = Mode::Command;
                self.show_command_palette = true;
                self.command_input.clear();
                debug!("Entered command mode");
            }
            EnterNormalMode => {
                self.mode = Mode::Normal;
                self.show_command_palette = false;
                debug!("Entered normal mode");
            }

            // Cursor movement
            MoveLeft => self.move_cursor_left(),
            MoveRight => self.move_cursor_right(),
            MoveUp => self.move_cursor_up(),
            MoveDown => self.move_cursor_down(),
            MoveLineStart => self.move_cursor_line_start(),
            MoveLineEnd => self.move_cursor_line_end(),
            MoveWordForward => self.move_cursor_word_forward(),
            MoveWordBackward => self.move_cursor_word_backward(),

            // Editing
            InsertChar(ch) => self.insert_char(*ch),
            InsertNewline => self.insert_newline(),
            DeleteChar => self.delete_char(),
            Backspace => self.backspace(),

            // Commands
            OpenCommandPalette => {
                self.show_command_palette = !self.show_command_palette;
                self.command_input.clear();
            }
            Save => self.save_buffer()?,
            Quit => self.quit(),
            ForceQuit => self.force_quit(),

            // Phase 10b: Tab management
            NewTab => self.new_tab("untitled".to_string())?,
            CloseTab => self.close_tab(self.active_tab_index)?,
            NextTab => self.next_tab(),
            PrevTab => self.prev_tab(),
            SwitchTab(index) => self.switch_tab(*index)?,

            // Phase 10b: Panel visibility and focus
            ToggleSidebar => self.toggle_sidebar(),
            ToggleTerminal => self.toggle_terminal(),
            FocusSidebar => self.focus_sidebar(),
            FocusEditor => self.focus_editor(),
            FocusTerminal => self.focus_terminal(),
            FocusNextPanel => self.focus_next_panel(),

            // Phase 10b: Sidebar navigation
            SidebarMoveUp => self.sidebar_move_up(),
            SidebarMoveDown => self.sidebar_move_down(),
            SidebarSelect => self.sidebar_select()?,
            SidebarToggleExpand => self.sidebar_toggle_expand(),

            _ => debug!("Unimplemented command: {:?}", command),
        }

        Ok(())
    }

    // Cursor movement implementations
    fn move_cursor_left(&mut self) {
        let pos = self.cursor.position(&self.buffer);
        if pos.col > 0 {
            let _ = self.cursor.move_to(&self.buffer, pos.line, pos.col - 1);
        }
    }

    fn move_cursor_right(&mut self) {
        let pos = self.cursor.position(&self.buffer);
        // TODO: Check line length
        let _ = self.cursor.move_to(&self.buffer, pos.line, pos.col + 1);
    }

    fn move_cursor_up(&mut self) {
        let pos = self.cursor.position(&self.buffer);
        if pos.line > 0 {
            let _ = self.cursor.move_to(&self.buffer, pos.line - 1, pos.col);
        }
    }

    fn move_cursor_down(&mut self) {
        let pos = self.cursor.position(&self.buffer);
        if pos.line < self.buffer.len_lines().saturating_sub(1) {
            let _ = self.cursor.move_to(&self.buffer, pos.line + 1, pos.col);
        }
    }

    fn move_cursor_line_start(&mut self) {
        let pos = self.cursor.position(&self.buffer);
        let _ = self.cursor.move_to(&self.buffer, pos.line, 0);
    }

    fn move_cursor_line_end(&mut self) {
        let pos = self.cursor.position(&self.buffer);
        // TODO: Get actual line length
        let _ = self.cursor.move_to(&self.buffer, pos.line, 100);
    }

    fn move_cursor_word_forward(&mut self) {
        // TODO: Implement word-based movement
        self.move_cursor_right();
    }

    fn move_cursor_word_backward(&mut self) {
        // TODO: Implement word-based movement
        self.move_cursor_left();
    }

    // Editing operations
    fn insert_char(&mut self, ch: char) {
        if self.mode == Mode::Insert {
            let pos = self.cursor.position(&self.buffer);
            // TODO: Implement actual insertion
            debug!("Insert char '{}' at {}:{}", ch, pos.line, pos.col);
            self.move_cursor_right();
        } else if self.mode == Mode::Command {
            self.command_input.push(ch);
        }
    }

    fn insert_newline(&mut self) {
        if self.mode == Mode::Insert {
            // TODO: Implement newline insertion
            self.move_cursor_down();
            self.move_cursor_line_start();
        }
    }

    fn delete_char(&mut self) {
        // TODO: Implement deletion
        debug!("Delete char");
    }

    fn backspace(&mut self) {
        if self.mode == Mode::Insert {
            // TODO: Implement backspace
            self.move_cursor_left();
        } else if self.mode == Mode::Command {
            self.command_input.pop();
        }
    }

    fn save_buffer(&mut self) -> Result<()> {
        // TODO: Implement save
        info!("Saving buffer");
        Ok(())
    }

    fn quit(&mut self) {
        if self.buffer.is_dirty() {
            // TODO: Prompt for save
            info!("Buffer modified, use :q! to force quit");
        } else {
            self.running = false;
        }
    }

    fn force_quit(&mut self) {
        self.running = false;
    }

    // ==========================================
    // Phase 10b: Tab Management
    // ==========================================

    /// Create a new tab with the given title
    pub fn new_tab(&mut self, title: String) -> Result<()> {
        let buffer = Buffer::new();
        let tab = Tab::new(title, None, buffer);
        self.tabs.push(tab);
        self.active_tab_index = self.tabs.len() - 1;
        self.switch_tab(self.active_tab_index)?;
        info!("Created new tab: {}", self.tabs[self.active_tab_index].title);
        Ok(())
    }

    /// Close tab at the given index
    pub fn close_tab(&mut self, index: usize) -> Result<()> {
        if self.tabs.len() <= 1 {
            debug!("Cannot close last tab");
            return Ok(());
        }

        if index >= self.tabs.len() {
            return Ok(());
        }

        let tab = &self.tabs[index];
        if tab.is_modified {
            info!("Tab '{}' has unsaved changes", tab.title);
            // TODO: Prompt user for confirmation
        }

        self.tabs.remove(index);

        // Adjust active tab index
        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        }

        self.switch_tab(self.active_tab_index)?;
        info!("Closed tab at index {}", index);
        Ok(())
    }

    /// Switch to tab at the given index
    pub fn switch_tab(&mut self, index: usize) -> Result<()> {
        if index >= self.tabs.len() {
            return Ok(());
        }

        // Save current buffer state to active tab
        if !self.tabs.is_empty() && self.active_tab_index < self.tabs.len() {
            self.tabs[self.active_tab_index].buffer = self.buffer.clone();
        }

        // Switch to new tab
        self.active_tab_index = index;
        self.buffer = self.tabs[index].buffer.clone();
        self.cursor = Cursor::default();
        self.view = ViewState::new();

        debug!("Switched to tab: {}", self.tabs[index].title);
        Ok(())
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        let next_index = (self.active_tab_index + 1) % self.tabs.len();
        let _ = self.switch_tab(next_index);
    }

    /// Switch to previous tab
    pub fn prev_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        let prev_index = if self.active_tab_index == 0 {
            self.tabs.len() - 1
        } else {
            self.active_tab_index - 1
        };
        let _ = self.switch_tab(prev_index);
    }

    // ==========================================
    // Phase 10b: Sidebar Navigation
    // ==========================================

    /// Move sidebar selection up
    pub fn sidebar_move_up(&mut self) {
        if self.sidebar_selected > 0 {
            self.sidebar_selected -= 1;
            debug!("Sidebar selection: {}", self.sidebar_selected);
        }
    }

    /// Move sidebar selection down
    pub fn sidebar_move_down(&mut self) {
        if !self.sidebar_items.is_empty()
            && self.sidebar_selected < self.sidebar_items.len() - 1
        {
            self.sidebar_selected += 1;
            debug!("Sidebar selection: {}", self.sidebar_selected);
        }
    }

    /// Select current sidebar item (open file or toggle directory)
    pub fn sidebar_select(&mut self) -> Result<()> {
        if self.sidebar_selected >= self.sidebar_items.len() {
            return Ok(());
        }

        let item = &self.sidebar_items[self.sidebar_selected].clone();

        if item.is_dir {
            // Toggle directory expansion
            self.sidebar_toggle_expand();
        } else {
            // Open file in new tab
            let buffer = Buffer::from_file(&item.path)?;
            let title = item
                .name
                .clone();
            let tab = Tab::new(title, Some(item.path.clone()), buffer);

            self.tabs.push(tab);
            self.active_tab_index = self.tabs.len() - 1;
            self.switch_tab(self.active_tab_index)?;

            info!("Opened file: {:?}", item.path);
        }

        Ok(())
    }

    /// Toggle expansion of current directory in sidebar
    pub fn sidebar_toggle_expand(&mut self) {
        if self.sidebar_selected >= self.sidebar_items.len() {
            return;
        }

        let item = &mut self.sidebar_items[self.sidebar_selected];
        if item.is_dir {
            item.is_expanded = !item.is_expanded;
            debug!(
                "Toggled directory '{}': {}",
                item.name, item.is_expanded
            );
            // TODO: Load/unload directory contents
        }
    }

    /// Load directory contents into sidebar
    pub fn sidebar_load_directory(&mut self, path: &PathBuf) -> Result<()> {
        use std::fs;

        self.sidebar_items.clear();

        let entries = fs::read_dir(path)?;
        let mut items = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = path.is_dir();

            items.push(SidebarItem {
                name,
                path,
                is_dir,
                is_expanded: false,
                level: 0,
            });
        }

        // Sort: directories first, then files
        items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        self.sidebar_items = items;
        self.sidebar_selected = 0;

        info!("Loaded {} items from {:?}", self.sidebar_items.len(), path);
        Ok(())
    }

    // ==========================================
    // Phase 10b: Panel Visibility & Focus
    // ==========================================

    /// Toggle sidebar visibility
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
        debug!("Sidebar visible: {}", self.sidebar_visible);
    }

    /// Toggle terminal visibility
    pub fn toggle_terminal(&mut self) {
        self.terminal_visible = !self.terminal_visible;
        debug!("Terminal visible: {}", self.terminal_visible);
    }

    /// Focus next panel in cycle: Editor -> Sidebar -> Terminal -> Editor
    pub fn focus_next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Editor if self.sidebar_visible => FocusedPanel::Sidebar,
            FocusedPanel::Sidebar if self.terminal_visible => FocusedPanel::Terminal,
            _ => FocusedPanel::Editor,
        };
        debug!("Focused panel: {:?}", self.focused_panel);
    }

    /// Focus the editor panel
    pub fn focus_editor(&mut self) {
        self.focused_panel = FocusedPanel::Editor;
        debug!("Focused panel: Editor");
    }

    /// Focus the sidebar panel
    pub fn focus_sidebar(&mut self) {
        if self.sidebar_visible {
            self.focused_panel = FocusedPanel::Sidebar;
            debug!("Focused panel: Sidebar");
        }
    }

    /// Focus the terminal panel
    pub fn focus_terminal(&mut self) {
        if self.terminal_visible {
            self.focused_panel = FocusedPanel::Terminal;
            debug!("Focused panel: Terminal");
        }
    }

    // ==========================================
    // Phase 10b: Getters for UI
    // ==========================================

    /// Get all tabs
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// Get active tab index
    pub fn active_tab_index(&self) -> usize {
        self.active_tab_index
    }

    /// Get sidebar visibility
    pub fn sidebar_visible(&self) -> bool {
        self.sidebar_visible
    }

    /// Get sidebar items
    pub fn sidebar_items(&self) -> &[SidebarItem] {
        &self.sidebar_items
    }

    /// Get sidebar selected index
    pub fn sidebar_selected(&self) -> usize {
        self.sidebar_selected
    }

    /// Get terminal visibility
    pub fn terminal_visible(&self) -> bool {
        self.terminal_visible
    }

    /// Get terminal scroll offset
    pub fn terminal_scroll(&self) -> usize {
        self.terminal_scroll
    }

    /// Get focused panel
    pub fn focused_panel(&self) -> FocusedPanel {
        self.focused_panel
    }
}

/// TUI Application
pub struct TuiApp {
    state: EditorState,
    renderer: Renderer,
    event_loop: EventLoop,
    keybinds: KeyMap,
    theme: Theme,
    layout_config: LayoutConfig,
}

impl TuiApp {
    /// Create new TUI application
    pub async fn new() -> Result<Self> {
        let config = EditorConfig::default();
        let state = EditorState::new(config)?;
        let renderer = Renderer::new()?;
        let event_loop = EventLoop::new(Duration::from_millis(250));
        let keybinds = KeyMap::default();
        let theme = Theme::default();
        let layout_config = LayoutConfig::default();

        Ok(Self {
            state,
            renderer,
            event_loop,
            keybinds,
            theme,
            layout_config,
        })
    }

    /// Run the application event loop
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting TUI application");

        while self.state.running {
            // Update view scroll to keep cursor visible
            let size = self.renderer.size()?;
            let cursor_pos = self.state.cursor.position(&self.state.buffer);
            self.state
                .view
                .update_scroll(cursor_pos.line, cursor_pos.col, size);

            // Update layout config
            self.layout_config.show_command_palette = self.state.show_command_palette;

            // Render
            self.renderer.render(
                &self.state.buffer,
                &self.state.cursor,
                &self.state.view,
                self.state.mode,
                &self.theme,
                &self.layout_config,
            )?;

            // Handle events
            if let Some(event) = self.event_loop.next().await {
                self.handle_event(event)?;
            }
        }

        info!("TUI application shutting down");
        Ok(())
    }

    /// Handle an event
    fn handle_event(&mut self, event: EditorEvent) -> Result<()> {
        match event {
            EditorEvent::Key(key) => self.handle_key(key)?,
            EditorEvent::Resize(w, h) => {
                debug!("Terminal resized: {}x{}", w, h);
            }
            EditorEvent::Tick => {
                // Periodic refresh (no-op for now)
            }
            EditorEvent::Quit => {
                self.state.running = false;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keyboard input
    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        use crate::tui_app::FocusedPanel;

        // Special handling for Ctrl+C to quit
        if key.code == KeyCode::Char('c')
            && key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            self.state.running = false;
            return Ok(());
        }

        // Look up command based on focused panel
        let key_binding = KeyBinding::from_key_event(key);
        let command = match self.state.focused_panel() {
            FocusedPanel::Sidebar => {
                // Try sidebar-specific bindings first, fall back to normal mode
                self.keybinds.lookup_sidebar(key_binding.clone())
                    .or_else(|| self.keybinds.lookup(self.state.mode, key_binding.clone()))
            }
            _ => {
                // Use mode-based bindings for editor and terminal
                self.keybinds.lookup(self.state.mode, key_binding.clone())
            }
        };

        if let Some(command) = command {
            self.state.execute_command(command)?;
        } else {
            // Handle character input in insert mode
            if self.state.mode == Mode::Insert || self.state.mode == Mode::Command {
                if let KeyCode::Char(ch) = key.code {
                    self.state.execute_command(&EditorCommand::InsertChar(ch))?;
                }
            } else {
                debug!("Unbound key: {:?}", key);
            }
        }

        Ok(())
    }

    /// Load a file
    pub fn load_file(&mut self, path: std::path::PathBuf) -> Result<()> {
        self.state.load_file(path)
    }

    /// Set theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_state_creation() {
        let config = EditorConfig::default();
        let state = EditorState::new(config);
        assert!(state.is_ok());

        let state = state.unwrap();
        assert_eq!(state.mode, Mode::Normal);
        assert!(!state.show_command_palette);
    }

    #[test]
    fn test_mode_transitions() {
        let config = EditorConfig::default();
        let mut state = EditorState::new(config).unwrap();

        state
            .execute_command(&EditorCommand::EnterInsertMode)
            .unwrap();
        assert_eq!(state.mode, Mode::Insert);

        state
            .execute_command(&EditorCommand::EnterNormalMode)
            .unwrap();
        assert_eq!(state.mode, Mode::Normal);
    }

    #[test]
    fn test_cursor_movement() {
        let config = EditorConfig::default();
        let mut state = EditorState::new(config).unwrap();

        // Add some content to the buffer
        state.buffer.insert(0, "Line 1\nLine 2\nLine 3").unwrap();

        let initial_pos = state.cursor.position(&state.buffer);
        assert_eq!(initial_pos.line, 0);
        assert_eq!(initial_pos.col, 0);

        state.execute_command(&EditorCommand::MoveRight).unwrap();
        let new_pos = state.cursor.position(&state.buffer);
        assert_eq!(new_pos.col, 1);

        state.execute_command(&EditorCommand::MoveDown).unwrap();
        let new_pos = state.cursor.position(&state.buffer);
        assert_eq!(new_pos.line, 1);
    }

    #[test]
    fn test_command_palette_toggle() {
        let config = EditorConfig::default();
        let mut state = EditorState::new(config).unwrap();

        assert!(!state.show_command_palette);

        state
            .execute_command(&EditorCommand::OpenCommandPalette)
            .unwrap();
        assert!(state.show_command_palette);

        state
            .execute_command(&EditorCommand::OpenCommandPalette)
            .unwrap();
        assert!(!state.show_command_palette);
    }

    // Phase 10b tests
    #[test]
    fn test_tab_management() {
        let config = EditorConfig::default();
        let mut state = EditorState::new(config).unwrap();

        // Initial state: 1 tab
        assert_eq!(state.tabs().len(), 1);
        assert_eq!(state.active_tab_index(), 0);

        // Create new tab
        state.new_tab("test.rs".to_string()).unwrap();
        assert_eq!(state.tabs().len(), 2);
        assert_eq!(state.active_tab_index(), 1);

        // Switch to previous tab
        state.prev_tab();
        assert_eq!(state.active_tab_index(), 0);

        // Switch to next tab
        state.next_tab();
        assert_eq!(state.active_tab_index(), 1);

        // Close tab
        state.close_tab(1).unwrap();
        assert_eq!(state.tabs().len(), 1);
    }

    #[test]
    fn test_sidebar_visibility() {
        let config = EditorConfig::default();
        let mut state = EditorState::new(config).unwrap();

        // Initial state: sidebar visible
        assert!(state.sidebar_visible());

        // Toggle off
        state.toggle_sidebar();
        assert!(!state.sidebar_visible());

        // Toggle on
        state.toggle_sidebar();
        assert!(state.sidebar_visible());
    }

    #[test]
    fn test_terminal_visibility() {
        let config = EditorConfig::default();
        let mut state = EditorState::new(config).unwrap();

        // Initial state: terminal hidden
        assert!(!state.terminal_visible());

        // Toggle on
        state.toggle_terminal();
        assert!(state.terminal_visible());

        // Toggle off
        state.toggle_terminal();
        assert!(!state.terminal_visible());
    }

    #[test]
    fn test_panel_focus() {
        let config = EditorConfig::default();
        let mut state = EditorState::new(config).unwrap();

        // Initial focus: Editor
        assert_eq!(state.focused_panel(), FocusedPanel::Editor);

        // Focus sidebar
        state.focus_sidebar();
        assert_eq!(state.focused_panel(), FocusedPanel::Sidebar);

        // Focus editor
        state.focus_editor();
        assert_eq!(state.focused_panel(), FocusedPanel::Editor);

        // Enable terminal and focus it
        state.toggle_terminal();
        state.focus_terminal();
        assert_eq!(state.focused_panel(), FocusedPanel::Terminal);
    }

    #[test]
    fn test_sidebar_navigation() {
        let config = EditorConfig::default();
        let mut state = EditorState::new(config).unwrap();

        // Load temp directory for testing
        let temp_dir = std::env::temp_dir();
        state.sidebar_load_directory(&temp_dir).ok();

        if !state.sidebar_items().is_empty() {
            let initial_selected = state.sidebar_selected();

            // Move down
            state.sidebar_move_down();
            assert!(state.sidebar_selected() > initial_selected);

            // Move up
            state.sidebar_move_up();
            assert_eq!(state.sidebar_selected(), initial_selected);
        }
    }

    #[test]
    fn test_focus_cycle() {
        let config = EditorConfig::default();
        let mut state = EditorState::new(config).unwrap();

        // Enable all panels
        assert!(state.sidebar_visible());
        state.toggle_terminal();
        assert!(state.terminal_visible());

        // Start at editor
        assert_eq!(state.focused_panel(), FocusedPanel::Editor);

        // Cycle through panels
        state.focus_next_panel();
        assert_eq!(state.focused_panel(), FocusedPanel::Sidebar);

        state.focus_next_panel();
        assert_eq!(state.focused_panel(), FocusedPanel::Terminal);

        state.focus_next_panel();
        assert_eq!(state.focused_panel(), FocusedPanel::Editor);
    }

    #[test]
    fn test_tab_closing_last_tab() {
        let config = EditorConfig::default();
        let mut state = EditorState::new(config).unwrap();

        // Should have 1 tab initially
        assert_eq!(state.tabs().len(), 1);

        // Try to close the last tab (should not close)
        state.close_tab(0).unwrap();
        assert_eq!(state.tabs().len(), 1);
    }
}
