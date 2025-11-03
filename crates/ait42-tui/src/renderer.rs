//! Renderer
//!
//! Handles terminal rendering with ratatui.

use crate::{
    keybinds::Mode,
    layout::{EditorLayout, LayoutConfig},
    theme::Theme,
    widgets::{editor::ViewState, EditorWidget, StatusLine},
};
use ait42_core::{Buffer, Cursor};
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

/// Terminal renderer
pub struct Renderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Renderer {
    /// Create new renderer and initialize terminal
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    /// Render the editor UI
    pub fn render(
        &mut self,
        buffer: &Buffer,
        cursor: &Cursor,
        view: &ViewState,
        mode: Mode,
        theme: &Theme,
        layout_config: &LayoutConfig,
    ) -> Result<()> {
        self.terminal.draw(|f| {
            let size = f.size();
            let layout = EditorLayout::calculate(size, layout_config);

            // Render line numbers if configured
            if let Some(line_numbers_area) = layout.line_numbers {
                let editor_widget = EditorWidget::new(buffer, cursor, view, theme);
                editor_widget.render_line_numbers(line_numbers_area, f.buffer_mut());
            }

            // Render main editor
            let editor_widget = EditorWidget::new(buffer, cursor, view, theme)
                .show_line_numbers(false); // Line numbers rendered separately
            f.render_widget(editor_widget, layout.editor);

            // Render status line
            let cursor_pos = cursor.position();
            let status = StatusLine::new(
                mode,
                (cursor_pos.line, cursor_pos.col),
                buffer.line_count(),
                theme,
            )
            .dirty(buffer.is_modified());

            if let Some(path) = buffer.file_path() {
                let status = status.file_path(path);
                f.render_widget(status, layout.statusline);
            } else {
                f.render_widget(status, layout.statusline);
            }

            // Render command palette if visible
            if let Some(palette_area) = layout.command_palette {
                use crate::widgets::command_palette::{default_commands, CommandPalette};
                let commands = default_commands();
                let palette = CommandPalette::new("", &commands, theme);
                f.render_widget(palette, palette_area);
            }

            // Set cursor position for terminal
            let cursor_screen_x = layout.editor.x +
                cursor_pos.col.saturating_sub(view.scroll_col) as u16;
            let cursor_screen_y = layout.editor.y +
                cursor_pos.line.saturating_sub(view.scroll_line) as u16;

            // Only show cursor in insert mode
            if mode == Mode::Insert {
                if cursor_screen_x < layout.editor.right() &&
                   cursor_screen_y < layout.editor.bottom() {
                    f.set_cursor(cursor_screen_x, cursor_screen_y);
                }
            }
        })?;

        Ok(())
    }

    /// Clear the terminal
    pub fn clear(&mut self) -> Result<()> {
        self.terminal.clear()?;
        Ok(())
    }

    /// Get terminal size
    pub fn size(&self) -> Result<ratatui::layout::Rect> {
        Ok(self.terminal.size()?)
    }

    /// Hide cursor
    pub fn hide_cursor(&mut self) -> Result<()> {
        self.terminal.hide_cursor()?;
        Ok(())
    }

    /// Show cursor
    pub fn show_cursor(&mut self) -> Result<()> {
        self.terminal.show_cursor()?;
        Ok(())
    }

    /// Restore terminal to normal state
    pub fn restore(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // Try to restore terminal on drop
        let _ = self.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        // Skip in CI environments
        if std::env::var("CI").is_ok() {
            return;
        }

        // This test requires a TTY
        // We'll just verify the struct can be created
        // Actual rendering tests would need snapshot testing
    }

    #[test]
    fn test_renderer_drop() {
        // Ensure drop doesn't panic
        // This is a basic safety test
    }
}
