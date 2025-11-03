//! Command Palette Widget
//!
//! Provides fuzzy search command interface.

use crate::theme::Theme;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

/// Command item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub category: String,
}

impl Command {
    pub fn new(name: impl Into<String>, description: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            category: category.into(),
        }
    }
}

/// Fuzzy match result
#[derive(Debug, Clone)]
struct Match {
    command: Command,
    score: i64,
}

/// Command palette widget
pub struct CommandPalette<'a> {
    input: &'a str,
    commands: &'a [Command],
    selected: usize,
    theme: &'a Theme,
}

impl<'a> CommandPalette<'a> {
    /// Create new command palette
    pub fn new(input: &'a str, commands: &'a [Command], theme: &'a Theme) -> Self {
        Self {
            input,
            commands,
            selected: 0,
            theme,
        }
    }

    /// Set selected index
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Filter and score commands based on input
    fn filter_commands(&self) -> Vec<Match> {
        if self.input.is_empty() {
            // Return all commands if no input
            return self
                .commands
                .iter()
                .map(|cmd| Match {
                    command: cmd.clone(),
                    score: 0,
                })
                .collect();
        }

        let matcher = SkimMatcherV2::default();
        let mut matches: Vec<Match> = Vec::new();

        for command in self.commands {
            // Search in both name and description
            let name_score = matcher.fuzzy_match(&command.name, self.input);
            let desc_score = matcher.fuzzy_match(&command.description, self.input);

            if let Some(score) = name_score.or(desc_score) {
                matches.push(Match {
                    command: command.clone(),
                    score,
                });
            }
        }

        // Sort by score (highest first)
        matches.sort_by(|a, b| b.score.cmp(&a.score));
        matches
    }
}

impl<'a> Widget for CommandPalette<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height < 3 {
            return;
        }

        // Create border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border)
            .title(" Command Palette ");

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 2 {
            return;
        }

        // Render input box
        let input_area = Rect::new(inner.x, inner.y, inner.width, 1);
        let input_style = Style::default()
            .fg(self.theme.foreground)
            .bg(self.theme.background)
            .add_modifier(Modifier::BOLD);

        let input_text = format!("> {}", self.input);
        buf.set_string(input_area.x, input_area.y, input_text, input_style);

        // Render cursor in input
        let cursor_x = input_area.x + 2 + self.input.len() as u16;
        if cursor_x < input_area.right() {
            buf.get_mut(cursor_x, input_area.y)
                .set_bg(self.theme.cursor)
                .set_fg(self.theme.background);
        }

        // Filter and render suggestions
        let suggestions_area = Rect::new(
            inner.x,
            inner.y + 2,
            inner.width,
            inner.height.saturating_sub(2),
        );

        let matches = self.filter_commands();
        let visible_count = suggestions_area.height as usize;

        for (i, m) in matches.iter().take(visible_count).enumerate() {
            let y = suggestions_area.y + i as u16;

            let is_selected = i == self.selected;
            let style = if is_selected {
                self.theme.selection
            } else {
                Style::default().fg(self.theme.foreground)
            };

            // Format: [Category] Name - Description
            let text = format!(
                "[{}] {} - {}",
                m.command.category,
                m.command.name,
                m.command.description
            );

            // Truncate to fit
            let max_width = suggestions_area.width as usize;
            let display_text = if text.len() > max_width {
                format!("{}...", &text[..max_width.saturating_sub(3)])
            } else {
                text
            };

            buf.set_string(suggestions_area.x, y, display_text, style);
        }

        // Show count
        if !matches.is_empty() {
            let count_text = format!(" {}/{} ", self.selected + 1, matches.len());
            let count_x = area.right().saturating_sub(count_text.len() as u16 + 1);
            buf.set_string(
                count_x,
                area.y,
                count_text,
                Style::default().fg(self.theme.comment.fg.unwrap()),
            );
        }
    }
}

/// Default commands
pub fn default_commands() -> Vec<Command> {
    vec![
        Command::new("open_file", "Open file", "File"),
        Command::new("save_file", "Save current file", "File"),
        Command::new("save_as", "Save file as...", "File"),
        Command::new("close_buffer", "Close current buffer", "File"),
        Command::new("quit", "Quit editor", "File"),
        Command::new("search", "Search in file", "Edit"),
        Command::new("replace", "Find and replace", "Edit"),
        Command::new("goto_line", "Go to line", "Navigation"),
        Command::new("toggle_line_numbers", "Toggle line numbers", "View"),
        Command::new("change_theme", "Change color theme", "View"),
        Command::new("split_horizontal", "Split window horizontally", "Window"),
        Command::new("split_vertical", "Split window vertically", "Window"),
        Command::new("format_document", "Format document", "Edit"),
        Command::new("show_help", "Show help", "Help"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_commands() -> Vec<Command> {
        vec![
            Command::new("open_file", "Open a file", "File"),
            Command::new("save_file", "Save current file", "File"),
            Command::new("quit", "Quit editor", "File"),
        ]
    }

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("test", "Test command", "Test");
        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.description, "Test command");
        assert_eq!(cmd.category, "Test");
    }

    #[test]
    fn test_filter_empty_input() {
        let theme = Theme::default();
        let commands = test_commands();
        let palette = CommandPalette::new("", &commands, &theme);

        let matches = palette.filter_commands();
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_filter_with_input() {
        let theme = Theme::default();
        let commands = test_commands();
        let palette = CommandPalette::new("save", &commands, &theme);

        let matches = palette.filter_commands();
        assert!(matches.len() > 0);
        assert!(matches[0].command.name.contains("save") ||
                matches[0].command.description.contains("save"));
    }

    #[test]
    fn test_fuzzy_matching() {
        let theme = Theme::default();
        let commands = test_commands();
        let palette = CommandPalette::new("opn", &commands, &theme);

        let matches = palette.filter_commands();
        // Should match "open_file" with fuzzy search
        assert!(matches.iter().any(|m| m.command.name == "open_file"));
    }

    #[test]
    fn test_default_commands() {
        let commands = default_commands();
        assert!(commands.len() > 10);
        assert!(commands.iter().any(|c| c.name == "open_file"));
        assert!(commands.iter().any(|c| c.name == "save_file"));
    }

    #[test]
    fn test_command_palette_selected() {
        let theme = Theme::default();
        let commands = test_commands();
        let palette = CommandPalette::new("", &commands, &theme).selected(1);

        assert_eq!(palette.selected, 1);
    }
}
