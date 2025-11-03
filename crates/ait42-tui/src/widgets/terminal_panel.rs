//! Terminal Panel Widget
//!
//! Displays terminal output with scrolling support and prompt display.

use crate::theme::Theme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

/// Terminal panel widget for displaying command output
pub struct TerminalPanel<'a> {
    /// Output lines to display
    output: &'a [String],
    /// Scroll offset (number of lines scrolled from bottom)
    scroll_offset: usize,
    /// Theme for styling
    theme: &'a Theme,
    /// Whether to show the "TERMINAL" header
    show_header: bool,
    /// Whether to show the prompt
    show_prompt: bool,
}

impl<'a> TerminalPanel<'a> {
    /// Create a new terminal panel
    pub fn new(output: &'a [String], theme: &'a Theme) -> Self {
        Self {
            output,
            scroll_offset: 0,
            theme,
            show_header: true,
            show_prompt: true,
        }
    }

    /// Set scroll offset (0 = bottom, positive values scroll up)
    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    /// Hide the "TERMINAL" header
    pub fn hide_header(mut self) -> Self {
        self.show_header = false;
        self
    }

    /// Hide the prompt line
    pub fn hide_prompt(mut self) -> Self {
        self.show_prompt = false;
        self
    }

    /// Render the header
    fn render_header(&self, area: Rect, buf: &mut Buffer) -> u16 {
        if !self.show_header || area.height < 2 {
            return 0;
        }

        let header = " TERMINAL ";
        let style = Style::default()
            .fg(self.theme.foreground)
            .add_modifier(Modifier::BOLD);

        buf.set_string(area.x, area.y, header, style);

        // Draw separator line
        let separator_style = Style::default()
            .fg(self.theme.border.fg.unwrap_or(self.theme.foreground));
        for x in area.left()..area.right() {
            buf.get_mut(x, area.y + 1)
                .set_char('─')
                .set_style(separator_style);
        }

        2 // Header takes 2 lines
    }

    /// Render the prompt line
    fn render_prompt(&self, y: u16, area: Rect, buf: &mut Buffer) -> u16 {
        if !self.show_prompt || y >= area.bottom() {
            return 0;
        }

        let prompt = "$ ";
        let style = Style::default()
            .fg(self.theme.function.fg.unwrap_or(self.theme.foreground))
            .add_modifier(Modifier::BOLD);

        buf.set_string(area.x, y, prompt, style);

        // Show cursor after prompt
        if area.x + prompt.len() as u16 <= area.right() {
            let cursor_style = Style::default()
                .fg(self.theme.cursor)
                .add_modifier(Modifier::SLOW_BLINK);
            buf.get_mut(area.x + prompt.len() as u16, y)
                .set_char('█')
                .set_style(cursor_style);
        }

        1 // Prompt takes 1 line
    }

    /// Render a single output line
    fn render_line(&self, line: &str, y: u16, area: Rect, buf: &mut Buffer) {
        if y >= area.bottom() {
            return;
        }

        let style = Style::default().fg(self.theme.foreground);
        let max_width = area.width as usize;

        // Handle long lines - truncate with ellipsis
        let display_line = if line.len() > max_width {
            format!("{}…", &line[..max_width.saturating_sub(1)])
        } else {
            line.to_string()
        };

        buf.set_string(area.x, y, &display_line, style);
    }

    /// Calculate which lines should be visible
    fn calculate_visible_range(&self, content_height: usize) -> (usize, usize) {
        let total_lines = self.output.len();

        if total_lines == 0 {
            return (0, 0);
        }

        // If we can fit all lines, show them all
        if total_lines <= content_height {
            return (0, total_lines);
        }

        // Calculate visible range based on scroll offset
        // scroll_offset = 0 means show the most recent lines (bottom)
        // scroll_offset > 0 means scroll up
        let end_index = total_lines.saturating_sub(self.scroll_offset);
        let start_index = end_index.saturating_sub(content_height);

        (start_index, end_index)
    }
}

impl<'a> Widget for TerminalPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Clear the area with background
        let bg_style = Style::default().bg(self.theme.background);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf.get_mut(x, y).set_style(bg_style);
            }
        }

        // Render header and calculate starting position
        let mut y_offset = area.y;
        if self.show_header {
            let header_height = self.render_header(area, buf);
            y_offset += header_height;
        }

        // Calculate space for content
        let prompt_height = if self.show_prompt { 1 } else { 0 };
        let content_height = area
            .bottom()
            .saturating_sub(y_offset)
            .saturating_sub(prompt_height) as usize;

        // Determine which lines to display
        let (start_index, end_index) = self.calculate_visible_range(content_height);

        // Render output lines
        for (i, line) in self
            .output
            .iter()
            .enumerate()
            .skip(start_index)
            .take(end_index - start_index)
        {
            self.render_line(line, y_offset, area, buf);
            y_offset += 1;

            if y_offset >= area.bottom() {
                return;
            }
        }

        // Render prompt at the bottom
        if self.show_prompt {
            let prompt_y = area.bottom().saturating_sub(1);
            self.render_prompt(prompt_y, area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_panel_creation() {
        let theme = Theme::default();
        let output = vec!["line 1".to_string(), "line 2".to_string()];

        let panel = TerminalPanel::new(&output, &theme);

        assert_eq!(panel.output.len(), 2);
        assert_eq!(panel.scroll_offset, 0);
        assert!(panel.show_header);
        assert!(panel.show_prompt);
    }

    #[test]
    fn test_scroll_offset() {
        let theme = Theme::default();
        let output = vec!["line 1".to_string()];

        let panel = TerminalPanel::new(&output, &theme).scroll_offset(5);

        assert_eq!(panel.scroll_offset, 5);
    }

    #[test]
    fn test_hide_header() {
        let theme = Theme::default();
        let output = vec![];

        let panel = TerminalPanel::new(&output, &theme).hide_header();

        assert!(!panel.show_header);
    }

    #[test]
    fn test_hide_prompt() {
        let theme = Theme::default();
        let output = vec![];

        let panel = TerminalPanel::new(&output, &theme).hide_prompt();

        assert!(!panel.show_prompt);
    }

    #[test]
    fn test_calculate_visible_range_empty() {
        let theme = Theme::default();
        let output: Vec<String> = vec![];

        let panel = TerminalPanel::new(&output, &theme);
        let (start, end) = panel.calculate_visible_range(10);

        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn test_calculate_visible_range_fits_all() {
        let theme = Theme::default();
        let output = vec![
            "line 1".to_string(),
            "line 2".to_string(),
            "line 3".to_string(),
        ];

        let panel = TerminalPanel::new(&output, &theme);
        let (start, end) = panel.calculate_visible_range(10);

        assert_eq!(start, 0);
        assert_eq!(end, 3);
    }

    #[test]
    fn test_calculate_visible_range_scroll() {
        let theme = Theme::default();
        let output = vec![
            "line 1".to_string(),
            "line 2".to_string(),
            "line 3".to_string(),
            "line 4".to_string(),
            "line 5".to_string(),
        ];

        // Show bottom 2 lines (no scroll)
        let panel = TerminalPanel::new(&output, &theme);
        let (start, end) = panel.calculate_visible_range(2);
        assert_eq!(start, 3);
        assert_eq!(end, 5);

        // Scroll up by 1
        let panel_scrolled = panel.scroll_offset(1);
        let (start, end) = panel_scrolled.calculate_visible_range(2);
        assert_eq!(start, 2);
        assert_eq!(end, 4);
    }

    #[test]
    fn test_calculate_visible_range_over_scroll() {
        let theme = Theme::default();
        let output = vec!["line 1".to_string(), "line 2".to_string()];

        // Scroll beyond available content
        let panel = TerminalPanel::new(&output, &theme).scroll_offset(100);
        let (start, end) = panel.calculate_visible_range(2);

        // Should clamp to valid range
        assert_eq!(start, 0);
        assert!(end <= 2);
    }

    #[test]
    fn test_long_line_handling() {
        let theme = Theme::default();
        let long_line = "a".repeat(200);
        let output = vec![long_line];

        let panel = TerminalPanel::new(&output, &theme);

        // Verify panel is created successfully with long line
        assert_eq!(panel.output.len(), 1);
        assert!(panel.output[0].len() > 100);
    }

    #[test]
    fn test_builder_pattern() {
        let theme = Theme::default();
        let output = vec!["test".to_string()];

        let panel = TerminalPanel::new(&output, &theme)
            .scroll_offset(3)
            .hide_header()
            .hide_prompt();

        assert_eq!(panel.scroll_offset, 3);
        assert!(!panel.show_header);
        assert!(!panel.show_prompt);
    }
}
