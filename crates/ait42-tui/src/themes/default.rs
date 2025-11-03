//! Default fallback theme for AIT42 Editor
//!
//! Provides a simple, high-contrast theme that works reliably
//! across all terminal types, including those with limited color support.

use ratatui::style::Color;
use super::theme::Theme;

/// Default fallback theme
///
/// Features:
/// - Simple 16-color ANSI palette
/// - Maximum compatibility
/// - High contrast for readability
/// - Works on any terminal
#[derive(Debug, Clone, Copy)]
pub struct DefaultTheme;

impl DefaultTheme {
    /// Create a new Default theme instance
    pub fn new() -> Self {
        Self
    }

    /// Get the theme name
    pub fn name(&self) -> &'static str {
        "Default"
    }

    /// Get theme description
    pub fn description(&self) -> &'static str {
        "Simple, high-contrast theme with maximum terminal compatibility"
    }

    /// Get theme version
    pub fn version(&self) -> &'static str {
        "1.0.0"
    }
}

impl Default for DefaultTheme {
    fn default() -> Self {
        Self::new()
    }
}

impl Theme for DefaultTheme {
    // ==========================================
    // Base Colors - Using ANSI colors
    // ==========================================

    fn background(&self) -> Color {
        Color::Black
    }

    fn background_light(&self) -> Color {
        Color::DarkGray
    }

    fn background_deep(&self) -> Color {
        Color::Black
    }

    fn background_lighter(&self) -> Color {
        Color::Gray
    }

    fn foreground(&self) -> Color {
        Color::White
    }

    fn foreground_dim(&self) -> Color {
        Color::Gray
    }

    fn foreground_dimmer(&self) -> Color {
        Color::DarkGray
    }

    // ==========================================
    // Tab Colors
    // ==========================================

    fn tab_active_bg(&self) -> Color {
        Color::DarkGray
    }

    fn tab_active_fg(&self) -> Color {
        Color::White
    }

    fn tab_inactive_bg(&self) -> Color {
        Color::Black
    }

    fn tab_inactive_fg(&self) -> Color {
        Color::Gray
    }

    // ==========================================
    // Sidebar Colors
    // ==========================================

    fn sidebar_bg(&self) -> Color {
        Color::Black
    }

    fn sidebar_fg(&self) -> Color {
        Color::White
    }

    fn sidebar_hover_bg(&self) -> Color {
        Color::DarkGray
    }

    fn sidebar_active_bg(&self) -> Color {
        Color::Gray
    }

    // ==========================================
    // Terminal Colors
    // ==========================================

    fn terminal_selection(&self) -> Color {
        Color::Blue
    }

    // ==========================================
    // Border Colors
    // ==========================================

    fn border(&self) -> Color {
        Color::DarkGray
    }

    fn border_active(&self) -> Color {
        Color::Blue
    }

    // ==========================================
    // Status Bar Colors
    // ==========================================

    fn status_bar_bg(&self) -> Color {
        Color::Blue
    }

    fn status_bar_fg(&self) -> Color {
        Color::White
    }

    // ==========================================
    // Semantic Colors
    // ==========================================

    fn success(&self) -> Color {
        Color::Green
    }

    fn info(&self) -> Color {
        Color::Blue
    }

    fn warning(&self) -> Color {
        Color::Yellow
    }

    fn error(&self) -> Color {
        Color::Red
    }

    // ==========================================
    // Agent Status Colors
    // ==========================================

    fn agent_running(&self) -> Color {
        Color::Blue
    }

    fn agent_success(&self) -> Color {
        Color::Green
    }

    fn agent_error(&self) -> Color {
        Color::Red
    }

    fn agent_idle(&self) -> Color {
        Color::Gray
    }

    // ==========================================
    // Syntax Highlighting Colors
    // ==========================================

    fn syntax_keyword(&self) -> Color {
        Color::Magenta
    }

    fn syntax_type(&self) -> Color {
        Color::Cyan
    }

    fn syntax_function(&self) -> Color {
        Color::Yellow
    }

    fn syntax_string(&self) -> Color {
        Color::Green
    }

    fn syntax_number(&self) -> Color {
        Color::LightGreen
    }

    fn syntax_comment(&self) -> Color {
        Color::DarkGray
    }

    fn syntax_variable(&self) -> Color {
        Color::LightBlue
    }

    fn syntax_constant(&self) -> Color {
        Color::LightMagenta
    }

    fn syntax_operator(&self) -> Color {
        Color::White
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme_basic_colors() {
        let theme = DefaultTheme::new();

        // Test basic colors are ANSI
        assert_eq!(theme.background(), Color::Black);
        assert_eq!(theme.foreground(), Color::White);
    }

    #[test]
    fn test_theme_metadata() {
        let theme = DefaultTheme::new();
        assert_eq!(theme.name(), "Default");
        assert!(!theme.description().is_empty());
        assert_eq!(theme.version(), "1.0.0");
    }
}
