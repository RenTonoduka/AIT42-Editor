//! Tab Bar Widget
//!
//! Displays the tab bar with open files, active tab highlighting, and window controls.

use crate::theme::Theme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::Span,
    widgets::Widget,
};

/// Represents a single tab in the tab bar
#[derive(Debug, Clone)]
pub struct Tab {
    /// Tab title (typically the file name)
    pub title: String,
    /// Whether the file has been modified
    pub modified: bool,
}

impl Tab {
    /// Create a new tab
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            modified: false,
        }
    }

    /// Set modified status
    pub fn modified(mut self, modified: bool) -> Self {
        self.modified = modified;
        self
    }

    /// Get display width of the tab
    fn display_width(&self) -> usize {
        // " title * " or " title  "
        let base = self.title.len() + 3; // space + title + space
        if self.modified {
            base + 2 // + " *"
        } else {
            base
        }
    }

    /// Truncate title if needed to fit max_width
    fn truncated_title(&self, max_width: usize) -> String {
        let suffix = if self.modified { " * " } else { "  " };
        let suffix_len = suffix.len() + 1; // +1 for leading space

        if max_width <= suffix_len {
            return String::from("…");
        }

        let available = max_width - suffix_len;
        if self.title.len() <= available {
            self.title.clone()
        } else if available > 3 {
            format!("{}…", &self.title[..available - 1])
        } else {
            String::from("…")
        }
    }
}

/// Tab bar widget that displays open tabs and window controls
pub struct TabBar<'a> {
    tabs: &'a [Tab],
    active_index: usize,
    theme: &'a Theme,
    show_controls: bool,
}

impl<'a> TabBar<'a> {
    /// Create a new tab bar
    pub fn new(tabs: &'a [Tab], active_index: usize, theme: &'a Theme) -> Self {
        Self {
            tabs,
            active_index,
            theme,
            show_controls: true,
        }
    }

    /// Hide window control buttons
    pub fn hide_controls(mut self) -> Self {
        self.show_controls = false;
        self
    }

    /// Render window controls (× □ ─)
    fn render_controls(&self, area: Rect, buf: &mut Buffer) {
        if !self.show_controls || area.width < 8 {
            return;
        }

        let controls = " ─ □ × ";
        let x = area.right().saturating_sub(controls.len() as u16);
        let y = area.y;

        let style = Style::default()
            .fg(self.theme.comment.fg.unwrap_or(self.theme.foreground))
            .add_modifier(Modifier::BOLD);

        if x >= area.left() {
            buf.set_string(x, y, controls, style);
        }
    }

    /// Calculate maximum width for each tab
    fn calculate_tab_width(&self, area: Rect) -> usize {
        let controls_width = if self.show_controls { 8 } else { 0 };
        let available_width = area.width.saturating_sub(controls_width) as usize;

        if self.tabs.is_empty() {
            return 0;
        }

        // Try to fit all tabs first
        let total_width: usize = self.tabs.iter().map(|t| t.display_width()).sum();

        if total_width <= available_width {
            // All tabs fit, no need to truncate
            return usize::MAX;
        }

        // Distribute available width evenly
        let tab_count = self.tabs.len();
        let max_width = available_width / tab_count;

        // Minimum width for a tab is 5 characters (" … * ")
        max_width.max(5)
    }
}

impl<'a> Widget for TabBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Clear the line with background
        let bg_style = Style::default().bg(self.theme.background);
        for x in area.left()..area.right() {
            buf.get_mut(x, area.y).set_style(bg_style);
        }

        // Render window controls first (on the right)
        self.render_controls(area, buf);

        // Calculate tab width constraints
        let max_tab_width = self.calculate_tab_width(area);
        let mut x_offset = area.x;

        // Render tabs
        for (index, tab) in self.tabs.iter().enumerate() {
            if x_offset >= area.right() {
                break;
            }

            let is_active = index == self.active_index;

            // Determine tab style
            let style = if is_active {
                Style::default()
                    .bg(self.theme.selection.bg.unwrap_or(self.theme.background))
                    .fg(self.theme.foreground)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(self.theme.comment.fg.unwrap_or(self.theme.foreground))
            };

            // Build tab text
            let title = if max_tab_width == usize::MAX {
                tab.title.clone()
            } else {
                tab.truncated_title(max_tab_width)
            };

            let modified_marker = if tab.modified { " *" } else { "" };
            let tab_text = format!(" {}{} ", title, modified_marker);

            // Calculate tab width and check if it fits
            let tab_width = tab_text.len() as u16;
            if x_offset + tab_width > area.right() {
                break;
            }

            // Render the tab
            buf.set_string(x_offset, area.y, &tab_text, style);

            // Add separator if not the last visible tab
            x_offset += tab_width;
            if x_offset < area.right() && index < self.tabs.len() - 1 {
                let separator = "│";
                buf.set_string(
                    x_offset,
                    area.y,
                    separator,
                    Style::default().fg(self.theme.border.fg.unwrap_or(self.theme.foreground)),
                );
                x_offset += separator.len() as u16;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_creation() {
        let tab = Tab::new("test.rs");
        assert_eq!(tab.title, "test.rs");
        assert!(!tab.modified);
    }

    #[test]
    fn test_tab_modified() {
        let tab = Tab::new("test.rs").modified(true);
        assert!(tab.modified);
    }

    #[test]
    fn test_tab_display_width() {
        let tab = Tab::new("test.rs");
        assert_eq!(tab.display_width(), 10); // " test.rs  "

        let tab_modified = Tab::new("test.rs").modified(true);
        assert_eq!(tab_modified.display_width(), 12); // " test.rs * "
    }

    #[test]
    fn test_tab_truncation() {
        let tab = Tab::new("very_long_filename.rs");
        let truncated = tab.truncated_title(10);
        // Max width 10 - 1 (space) - 2 (suffix "  ") = 7 chars for title
        assert!(truncated.len() <= 10);
        assert!(truncated.contains('…')); // Should be truncated

        let short_tab = Tab::new("a.rs");
        let not_truncated = short_tab.truncated_title(20);
        assert_eq!(not_truncated, "a.rs");
    }

    #[test]
    fn test_tab_bar_creation() {
        let theme = Theme::default();
        let tabs = vec![
            Tab::new("file1.rs"),
            Tab::new("file2.rs").modified(true),
        ];

        let tab_bar = TabBar::new(&tabs, 0, &theme);
        assert_eq!(tab_bar.tabs.len(), 2);
        assert_eq!(tab_bar.active_index, 0);
        assert!(tab_bar.show_controls);
    }

    #[test]
    fn test_tab_bar_hide_controls() {
        let theme = Theme::default();
        let tabs = vec![Tab::new("file1.rs")];

        let tab_bar = TabBar::new(&tabs, 0, &theme).hide_controls();
        assert!(!tab_bar.show_controls);
    }

    #[test]
    fn test_calculate_tab_width() {
        let theme = Theme::default();
        let tabs = vec![
            Tab::new("file1.rs"),
            Tab::new("file2.rs"),
            Tab::new("file3.rs"),
        ];

        let tab_bar = TabBar::new(&tabs, 0, &theme);
        let area = Rect::new(0, 0, 100, 1);
        let max_width = tab_bar.calculate_tab_width(area);

        // With 100 width and 8 for controls, we have 92 / 3 = 30 per tab
        assert!(max_width >= 5); // At least minimum
    }

    #[test]
    fn test_empty_tabs() {
        let theme = Theme::default();
        let tabs: Vec<Tab> = vec![];

        let tab_bar = TabBar::new(&tabs, 0, &theme);
        let area = Rect::new(0, 0, 50, 1);
        let max_width = tab_bar.calculate_tab_width(area);

        assert_eq!(max_width, 0);
    }
}
