//! Layout Management
//!
//! Handles dynamic layout calculation based on terminal size and UI state.

use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

/// Minimum terminal dimensions
pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;

/// Editor layout configuration
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Show line numbers
    pub show_line_numbers: bool,
    /// Line number gutter width
    pub line_number_width: u16,
    /// Show command palette
    pub show_command_palette: bool,
    /// Command palette height
    pub command_palette_height: u16,
    /// Show sidebar (file tree)
    pub show_sidebar: bool,
    /// Sidebar width
    pub sidebar_width: u16,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            line_number_width: 5,
            show_command_palette: false,
            command_palette_height: 10,
            show_sidebar: false,
            sidebar_width: 30,
        }
    }
}

/// Calculated layout areas
#[derive(Debug, Clone, Copy)]
pub struct EditorLayout {
    /// Full terminal area
    pub full: Rect,
    /// Editor content area (main text editor)
    pub editor: Rect,
    /// Line number gutter area
    pub line_numbers: Option<Rect>,
    /// Status line area (bottom bar)
    pub statusline: Rect,
    /// Command palette area (when visible)
    pub command_palette: Option<Rect>,
    /// Sidebar area (when visible)
    pub sidebar: Option<Rect>,
}

impl EditorLayout {
    /// Calculate layout based on terminal size and configuration
    ///
    /// # Arguments
    /// * `terminal_size` - Current terminal dimensions
    /// * `config` - Layout configuration
    ///
    /// # Returns
    /// Calculated layout with all UI areas
    pub fn calculate(terminal_size: Rect, config: &LayoutConfig) -> Self {
        // Validate minimum size
        if terminal_size.width < MIN_WIDTH || terminal_size.height < MIN_HEIGHT {
            // Return a minimal layout if too small
            return Self::minimal(terminal_size);
        }

        let full = terminal_size;

        // Split main areas: [sidebar?] [main content]
        let (sidebar, main_area) = if config.show_sidebar {
            let chunks = RatatuiLayout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(config.sidebar_width),
                    Constraint::Min(0),
                ])
                .split(full);
            (Some(chunks[0]), chunks[1])
        } else {
            (None, full)
        };

        // Split vertically: [editor] [command_palette?] [statusline]
        let vertical_constraints = if config.show_command_palette {
            vec![
                Constraint::Min(0),
                Constraint::Length(config.command_palette_height),
                Constraint::Length(1),
            ]
        } else {
            vec![
                Constraint::Min(0),
                Constraint::Length(1),
            ]
        };

        let vertical_chunks = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints(vertical_constraints)
            .split(main_area);

        let editor_area = vertical_chunks[0];
        let (command_palette, statusline) = if config.show_command_palette {
            (Some(vertical_chunks[1]), vertical_chunks[2])
        } else {
            (None, vertical_chunks[1])
        };

        // Split editor area horizontally: [line_numbers?] [text]
        let (line_numbers, editor) = if config.show_line_numbers {
            let chunks = RatatuiLayout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(config.line_number_width),
                    Constraint::Min(0),
                ])
                .split(editor_area);
            (Some(chunks[0]), chunks[1])
        } else {
            (None, editor_area)
        };

        Self {
            full,
            editor,
            line_numbers,
            statusline,
            command_palette,
            sidebar,
        }
    }

    /// Create minimal layout for small terminals
    fn minimal(terminal_size: Rect) -> Self {
        let chunks = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(terminal_size);

        Self {
            full: terminal_size,
            editor: chunks[0],
            line_numbers: None,
            statusline: chunks[1],
            command_palette: None,
            sidebar: None,
        }
    }

    /// Get the viewport area for editor content (text only, no decorations)
    pub fn editor_viewport(&self) -> Rect {
        self.editor
    }

    /// Check if layout has sufficient space
    pub fn is_valid(&self) -> bool {
        self.editor.width >= 20 && self.editor.height >= 5
    }

    /// Get visible line count in editor
    pub fn visible_lines(&self) -> usize {
        self.editor.height as usize
    }

    /// Get visible column count in editor
    pub fn visible_cols(&self) -> usize {
        self.editor.width as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_layout() {
        let terminal_size = Rect::new(0, 0, 100, 30);
        let config = LayoutConfig::default();

        let layout = EditorLayout::calculate(terminal_size, &config);

        assert!(layout.is_valid());
        assert!(layout.line_numbers.is_some());
        assert_eq!(layout.statusline.height, 1);
        assert!(layout.command_palette.is_none());
        assert!(layout.sidebar.is_none());
    }

    #[test]
    fn test_layout_with_command_palette() {
        let terminal_size = Rect::new(0, 0, 100, 30);
        let config = LayoutConfig {
            show_command_palette: true,
            ..Default::default()
        };

        let layout = EditorLayout::calculate(terminal_size, &config);

        assert!(layout.command_palette.is_some());
        assert_eq!(layout.command_palette.unwrap().height, 10);
    }

    #[test]
    fn test_layout_with_sidebar() {
        let terminal_size = Rect::new(0, 0, 120, 30);
        let config = LayoutConfig {
            show_sidebar: true,
            sidebar_width: 30,
            ..Default::default()
        };

        let layout = EditorLayout::calculate(terminal_size, &config);

        assert!(layout.sidebar.is_some());
        assert_eq!(layout.sidebar.unwrap().width, 30);
    }

    #[test]
    fn test_minimal_layout() {
        let terminal_size = Rect::new(0, 0, 50, 10);
        let config = LayoutConfig::default();

        let layout = EditorLayout::calculate(terminal_size, &config);

        // Should handle small terminals gracefully
        assert!(layout.full.width < MIN_WIDTH);
    }

    #[test]
    fn test_layout_without_line_numbers() {
        let terminal_size = Rect::new(0, 0, 100, 30);
        let config = LayoutConfig {
            show_line_numbers: false,
            ..Default::default()
        };

        let layout = EditorLayout::calculate(terminal_size, &config);

        assert!(layout.line_numbers.is_none());
    }

    #[test]
    fn test_visible_dimensions() {
        let terminal_size = Rect::new(0, 0, 100, 30);
        let config = LayoutConfig::default();

        let layout = EditorLayout::calculate(terminal_size, &config);

        assert!(layout.visible_lines() > 0);
        assert!(layout.visible_cols() > 0);
    }
}
