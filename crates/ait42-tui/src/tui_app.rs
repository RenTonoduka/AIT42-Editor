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
use std::time::Duration;
use tracing::{debug, error, info};

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
}

impl EditorState {
    /// Create new editor state
    pub fn new(config: EditorConfig) -> Result<Self> {
        let editor = Editor::new(config)?;
        let buffer = Buffer::new();
        let cursor = Cursor::default();
        let view = ViewState::new();

        Ok(Self {
            editor,
            buffer,
            cursor,
            view,
            mode: Mode::Normal,
            command_input: String::new(),
            show_command_palette: false,
            running: true,
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
            self.state.view.update_scroll(
                cursor_pos.line,
                cursor_pos.col,
                size,
            );

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
        // Special handling for Ctrl+C to quit
        if key.code == KeyCode::Char('c') &&
           key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
            self.state.running = false;
            return Ok(());
        }

        // Look up command in keymap
        let key_binding = KeyBinding::from_key_event(key);
        if let Some(command) = self.keybinds.lookup(self.state.mode, key_binding.clone()) {
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

        state.execute_command(&EditorCommand::EnterInsertMode).unwrap();
        assert_eq!(state.mode, Mode::Insert);

        state.execute_command(&EditorCommand::EnterNormalMode).unwrap();
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

        state.execute_command(&EditorCommand::OpenCommandPalette).unwrap();
        assert!(state.show_command_palette);

        state.execute_command(&EditorCommand::OpenCommandPalette).unwrap();
        assert!(!state.show_command_palette);
    }
}
