//! Editor Widget
//!
//! Renders the main text editing area with cursor, selection, and line numbers.

use crate::theme::Theme;
use ait42_core::{Buffer, Cursor, Selection};
use ratatui::{
    buffer::Buffer as RatatuiBuffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use unicode_width::UnicodeWidthStr;

/// View state for scrolling
#[derive(Debug, Clone, Default)]
pub struct ViewState {
    /// First visible line (0-indexed)
    pub scroll_line: usize,
    /// Horizontal scroll offset
    pub scroll_col: usize,
}

impl ViewState {
    /// Create new view state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update scroll to keep cursor visible
    pub fn update_scroll(&mut self, cursor_line: usize, cursor_col: usize, viewport: Rect) {
        let visible_lines = viewport.height as usize;
        let visible_cols = viewport.width as usize;

        // Vertical scrolling
        if cursor_line < self.scroll_line {
            self.scroll_line = cursor_line;
        } else if cursor_line >= self.scroll_line + visible_lines {
            self.scroll_line = cursor_line - visible_lines + 1;
        }

        // Horizontal scrolling
        if cursor_col < self.scroll_col {
            self.scroll_col = cursor_col;
        } else if cursor_col >= self.scroll_col + visible_cols {
            self.scroll_col = cursor_col - visible_cols + 1;
        }
    }
}

/// Editor widget for rendering text buffer
pub struct EditorWidget<'a> {
    buffer: &'a Buffer,
    cursor: &'a Cursor,
    selection: Option<&'a Selection>,
    view: &'a ViewState,
    theme: &'a Theme,
    show_line_numbers: bool,
}

impl<'a> EditorWidget<'a> {
    /// Create new editor widget
    pub fn new(
        buffer: &'a Buffer,
        cursor: &'a Cursor,
        view: &'a ViewState,
        theme: &'a Theme,
    ) -> Self {
        Self {
            buffer,
            cursor,
            selection: None,
            view,
            theme,
            show_line_numbers: true,
        }
    }

    /// Set selection
    pub fn selection(mut self, selection: &'a Selection) -> Self {
        self.selection = Some(selection);
        self
    }

    /// Set whether to show line numbers
    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Render line numbers in separate area
    pub fn render_line_numbers(&self, area: Rect, buf: &mut RatatuiBuffer) {
        if area.width < 3 {
            return;
        }

        let line_count = self.buffer.line_count();
        let cursor_line = self.cursor.position().line;
        let start_line = self.view.scroll_line;
        let end_line = (start_line + area.height as usize).min(line_count);

        for (i, line_num) in (start_line..end_line).enumerate() {
            let y = area.y + i as u16;
            if y >= area.y + area.height {
                break;
            }

            let is_cursor_line = line_num == cursor_line;
            let style = if is_cursor_line {
                self.theme.line_number_active
            } else {
                self.theme.line_number
            };

            let line_str = format!("{:>4} ", line_num + 1);
            buf.set_string(area.x, y, line_str, style);
        }
    }
}

impl<'a> Widget for EditorWidget<'a> {
    fn render(self, area: Rect, buf: &mut RatatuiBuffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Clear the area with background color
        let bg_style = Style::default().bg(self.theme.background);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf.get_mut(x, y).set_style(bg_style);
            }
        }

        let content = self.buffer.content();
        let lines: Vec<&str> = content.lines().collect();
        let line_count = lines.len().max(1);

        let cursor_pos = self.cursor.position();
        let start_line = self.view.scroll_line;
        let end_line = (start_line + area.height as usize).min(line_count);

        // Render visible lines
        for (i, line_idx) in (start_line..end_line).enumerate() {
            let y = area.y + i as u16;
            if y >= area.y + area.height {
                break;
            }

            let line_text = if line_idx < lines.len() {
                lines[line_idx]
            } else {
                ""
            };

            // Apply horizontal scrolling
            let visible_text = if self.view.scroll_col < line_text.len() {
                &line_text[self.view.scroll_col..]
            } else {
                ""
            };

            // Truncate to fit viewport
            let max_width = area.width as usize;
            let display_text = truncate_to_width(visible_text, max_width);

            // Apply syntax highlighting (basic for now, Phase 2 will add proper highlighting)
            let style = Style::default().fg(self.theme.foreground);
            buf.set_string(area.x, y, display_text, style);

            // Render cursor on current line
            if line_idx == cursor_pos.line {
                let cursor_col = cursor_pos.col.saturating_sub(self.view.scroll_col);
                if cursor_col < area.width as usize {
                    let cursor_x = area.x + cursor_col as u16;
                    if cursor_x < area.right() {
                        // Set cursor background
                        buf.get_mut(cursor_x, y)
                            .set_bg(self.theme.cursor)
                            .set_fg(self.theme.background)
                            .set_style(Style::default().add_modifier(Modifier::BOLD));
                    }
                }
            }
        }

        // Render empty lines indicator (~)
        for i in end_line..start_line + area.height as usize {
            let y = area.y + (i - start_line) as u16;
            if y < area.bottom() {
                let style = self.theme.line_number;
                buf.set_string(area.x, y, "~", style);
            }
        }
    }
}

/// Truncate string to fit within width (accounting for unicode)
fn truncate_to_width(s: &str, max_width: usize) -> &str {
    let mut width = 0;
    let mut byte_idx = 0;

    for (idx, ch) in s.char_indices() {
        let ch_width = ch.width().unwrap_or(0);
        if width + ch_width > max_width {
            break;
        }
        width += ch_width;
        byte_idx = idx + ch.len_utf8();
    }

    &s[..byte_idx]
}

/// Scrollbar widget
pub struct Scrollbar<'a> {
    total_lines: usize,
    visible_lines: usize,
    scroll_position: usize,
    theme: &'a Theme,
}

impl<'a> Scrollbar<'a> {
    pub fn new(
        total_lines: usize,
        visible_lines: usize,
        scroll_position: usize,
        theme: &'a Theme,
    ) -> Self {
        Self {
            total_lines,
            visible_lines,
            scroll_position,
            theme,
        }
    }
}

impl<'a> Widget for Scrollbar<'a> {
    fn render(self, area: Rect, buf: &mut RatatuiBuffer) {
        if area.width == 0 || area.height == 0 || self.total_lines <= self.visible_lines {
            return;
        }

        let scrollbar_height = area.height as usize;
        let thumb_size = (self.visible_lines * scrollbar_height / self.total_lines).max(1);
        let thumb_position = self.scroll_position * scrollbar_height / self.total_lines;

        for i in 0..scrollbar_height {
            let y = area.y + i as u16;
            let ch = if i >= thumb_position && i < thumb_position + thumb_size {
                "█"
            } else {
                "│"
            };

            buf.set_string(area.x, y, ch, self.theme.border);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ait42_core::Buffer;

    #[test]
    fn test_truncate_to_width() {
        assert_eq!(truncate_to_width("hello world", 5), "hello");
        assert_eq!(truncate_to_width("hello", 10), "hello");
        assert_eq!(truncate_to_width("", 5), "");
    }

    #[test]
    fn test_view_state_scroll() {
        let mut view = ViewState::new();
        let viewport = Rect::new(0, 0, 80, 24);

        // Scroll down
        view.update_scroll(25, 0, viewport);
        assert!(view.scroll_line > 0);

        // Scroll up
        view.update_scroll(0, 0, viewport);
        assert_eq!(view.scroll_line, 0);
    }

    #[test]
    fn test_view_state_horizontal_scroll() {
        let mut view = ViewState::new();
        let viewport = Rect::new(0, 0, 80, 24);

        // Scroll right
        view.update_scroll(0, 100, viewport);
        assert!(view.scroll_col > 0);

        // Scroll left
        view.update_scroll(0, 0, viewport);
        assert_eq!(view.scroll_col, 0);
    }

    #[test]
    fn test_editor_widget_creation() {
        let buffer = Buffer::new();
        let cursor = Cursor::default();
        let view = ViewState::new();
        let theme = Theme::default();

        let widget = EditorWidget::new(&buffer, &cursor, &view, &theme);
        assert!(widget.show_line_numbers);
    }
}
