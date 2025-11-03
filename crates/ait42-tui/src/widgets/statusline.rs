//! Status Line Widget
//!
//! Displays editor state, cursor position, and file information.

use crate::{keybinds::Mode, theme::Theme};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};
use std::path::Path;

/// Status line widget
pub struct StatusLine<'a> {
    mode: Mode,
    file_path: Option<&'a Path>,
    dirty: bool,
    cursor_pos: (usize, usize), // (line, col)
    file_type: Option<&'a str>,
    total_lines: usize,
    theme: &'a Theme,
}

impl<'a> StatusLine<'a> {
    /// Create new status line
    pub fn new(
        mode: Mode,
        cursor_pos: (usize, usize),
        total_lines: usize,
        theme: &'a Theme,
    ) -> Self {
        Self {
            mode,
            file_path: None,
            dirty: false,
            cursor_pos,
            file_type: None,
            total_lines,
            theme,
        }
    }

    /// Set file path
    pub fn file_path(mut self, path: &'a Path) -> Self {
        self.file_path = Some(path);
        self
    }

    /// Set dirty flag
    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    /// Set file type
    pub fn file_type(mut self, file_type: &'a str) -> Self {
        self.file_type = Some(file_type);
        self
    }

    /// Get mode style
    fn mode_style(&self) -> Style {
        match self.mode {
            Mode::Normal => self.theme.statusline_normal,
            Mode::Insert => self.theme.statusline_insert,
            Mode::Visual => self.theme.statusline_visual,
            Mode::Command => self.theme.statusline_command,
        }
    }

    /// Format left side of status line
    fn left_section(&self) -> Vec<Span<'a>> {
        let mut spans = Vec::new();

        // Mode indicator
        let mode_text = format!(" {} ", self.mode.as_str());
        spans.push(Span::styled(mode_text, self.mode_style()));

        // Spacing
        spans.push(Span::raw(" "));

        // File path or [No Name]
        let file_name = if let Some(path) = self.file_path {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("[No Name]")
        } else {
            "[No Name]"
        };

        spans.push(Span::styled(
            file_name,
            Style::default().fg(self.theme.foreground),
        ));

        // Dirty indicator
        if self.dirty {
            spans.push(Span::styled(
                " [+]",
                Style::default().fg(self.theme.keyword.fg.unwrap()),
            ));
        }

        spans
    }

    /// Format right side of status line
    fn right_section(&self) -> Vec<Span<'a>> {
        let mut spans = Vec::new();

        // File type
        if let Some(ft) = self.file_type {
            spans.push(Span::styled(
                ft,
                Style::default().fg(self.theme.comment.fg.unwrap()),
            ));
            spans.push(Span::raw(" │ "));
        }

        // Cursor position and percentage
        let line = self.cursor_pos.0 + 1; // 1-indexed for display
        let col = self.cursor_pos.1 + 1;
        let percentage = if self.total_lines > 0 {
            (line * 100) / self.total_lines
        } else {
            0
        };

        let position_text = format!(" {}:{} {}% ", line, col, percentage);
        spans.push(Span::styled(
            position_text,
            Style::default().fg(self.theme.foreground),
        ));

        spans
    }
}

impl<'a> Widget for StatusLine<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Clear the line with background
        let bg_style = Style::default().bg(self.theme.background);
        for x in area.left()..area.right() {
            buf.get_mut(x, area.y).set_style(bg_style);
        }

        // Render left section
        let left_spans = self.left_section();
        let mut x_offset = area.x;

        for span in &left_spans {
            let width = span.content.len() as u16;
            if x_offset + width > area.right() {
                break;
            }
            buf.set_span(x_offset, area.y, span, width);
            x_offset += width;
        }

        // Render right section
        let right_spans = self.right_section();
        let right_width: usize = right_spans.iter().map(|s| s.content.len()).sum();

        if right_width < area.width as usize {
            let right_x = area.right().saturating_sub(right_width as u16);
            let mut x_offset = right_x;

            for span in &right_spans {
                let width = span.content.len() as u16;
                if x_offset + width > area.right() {
                    break;
                }
                buf.set_span(x_offset, area.y, span, width);
                x_offset += width;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_status_line_creation() {
        let theme = Theme::default();
        let status = StatusLine::new(Mode::Normal, (0, 0), 100, &theme);

        assert_eq!(status.mode, Mode::Normal);
        assert!(!status.dirty);
    }

    #[test]
    fn test_status_line_with_file() {
        let theme = Theme::default();
        let path = PathBuf::from("/tmp/test.rs");
        let status = StatusLine::new(Mode::Insert, (10, 5), 100, &theme)
            .file_path(&path)
            .dirty(true)
            .file_type("rust");

        assert_eq!(status.cursor_pos, (10, 5));
        assert!(status.dirty);
        assert_eq!(status.file_type, Some("rust"));
    }

    #[test]
    fn test_mode_styles() {
        let theme = Theme::default();

        let normal = StatusLine::new(Mode::Normal, (0, 0), 1, &theme);
        assert_eq!(normal.mode_style(), theme.statusline_normal);

        let insert = StatusLine::new(Mode::Insert, (0, 0), 1, &theme);
        assert_eq!(insert.mode_style(), theme.statusline_insert);

        let visual = StatusLine::new(Mode::Visual, (0, 0), 1, &theme);
        assert_eq!(visual.mode_style(), theme.statusline_visual);
    }

    #[test]
    fn test_cursor_position_display() {
        let theme = Theme::default();
        let status = StatusLine::new(Mode::Normal, (0, 0), 100, &theme);

        // Line and col are 0-indexed internally, but displayed as 1-indexed
        assert_eq!(status.cursor_pos, (0, 0));

        let status2 = StatusLine::new(Mode::Normal, (99, 49), 100, &theme);
        assert_eq!(status2.cursor_pos, (99, 49));
    }
}
