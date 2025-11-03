//! Cursor-style dark theme for AIT42 Editor
//!
//! This theme replicates the professional dark aesthetic of Cursor AI's code editor,
//! featuring high contrast, excellent readability, and modern color accents.
//!
//! Color palette inspired by VS Code Dark+ theme with Cursor's signature blue accent.

use ratatui::style::Color;
use super::theme::Theme;

/// Cursor-style dark theme
///
/// Features:
/// - Deep dark backgrounds (#1E1E1E, #252525)
/// - High contrast text (#CCCCCC)
/// - Signature Cursor blue accent (#007ACC)
/// - VS Code-compatible syntax highlighting
/// - Optimized for long coding sessions
///
/// # Examples
///
/// ```rust
/// use ait42_tui::themes::{CursorTheme, Theme};
/// use ratatui::style::Style;
///
/// let theme = CursorTheme::new();
/// let tab_style = Style::default()
///     .bg(theme.tab_active_bg())
///     .fg(theme.tab_active_fg());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct CursorTheme;

impl CursorTheme {
    /// Create a new Cursor theme instance
    pub fn new() -> Self {
        Self
    }

    /// Get the theme name
    pub fn name(&self) -> &'static str {
        "Cursor Dark"
    }

    /// Get theme description
    pub fn description(&self) -> &'static str {
        "Professional dark theme inspired by Cursor AI editor"
    }

    /// Get theme version
    pub fn version(&self) -> &'static str {
        "1.0.0"
    }
}

impl Default for CursorTheme {
    fn default() -> Self {
        Self::new()
    }
}

impl Theme for CursorTheme {
    // ==========================================
    // Base Colors
    // ==========================================

    fn background(&self) -> Color {
        Color::Rgb(30, 30, 30) // #1E1E1E - Main background
    }

    fn background_light(&self) -> Color {
        Color::Rgb(37, 37, 37) // #252525 - Elevated surfaces
    }

    fn background_deep(&self) -> Color {
        Color::Rgb(26, 26, 26) // #1A1A1A - Deepest layer (terminal)
    }

    fn background_lighter(&self) -> Color {
        Color::Rgb(45, 45, 45) // #2D2D2D - Active elements
    }

    fn foreground(&self) -> Color {
        Color::Rgb(204, 204, 204) // #CCCCCC - Primary text
    }

    fn foreground_dim(&self) -> Color {
        Color::Rgb(133, 133, 133) // #858585 - Secondary text
    }

    fn foreground_dimmer(&self) -> Color {
        Color::Rgb(96, 96, 96) // #606060 - Tertiary text
    }

    // ==========================================
    // Tab Colors
    // ==========================================

    fn tab_active_bg(&self) -> Color {
        Color::Rgb(45, 45, 45) // #2D2D2D
    }

    fn tab_active_fg(&self) -> Color {
        Color::Rgb(255, 255, 255) // #FFFFFF
    }

    fn tab_inactive_bg(&self) -> Color {
        Color::Rgb(37, 37, 37) // #252525
    }

    fn tab_inactive_fg(&self) -> Color {
        Color::Rgb(133, 133, 133) // #858585
    }

    fn tab_border(&self) -> Color {
        Color::Rgb(62, 62, 66) // #3E3E42
    }

    // ==========================================
    // Sidebar Colors
    // ==========================================

    fn sidebar_bg(&self) -> Color {
        Color::Rgb(37, 37, 37) // #252525
    }

    fn sidebar_fg(&self) -> Color {
        Color::Rgb(204, 204, 204) // #CCCCCC
    }

    fn sidebar_hover_bg(&self) -> Color {
        Color::Rgb(42, 42, 42) // #2A2A2A
    }

    fn sidebar_active_bg(&self) -> Color {
        Color::Rgb(51, 51, 51) // #333333
    }

    // ==========================================
    // Terminal Colors
    // ==========================================

    fn terminal_bg(&self) -> Color {
        Color::Rgb(26, 26, 26) // #1A1A1A
    }

    fn terminal_fg(&self) -> Color {
        Color::Rgb(204, 204, 204) // #CCCCCC
    }

    fn terminal_cursor(&self) -> Color {
        Color::Rgb(255, 255, 255) // #FFFFFF
    }

    fn terminal_selection(&self) -> Color {
        Color::Rgb(38, 79, 120) // #264F78
    }

    // ==========================================
    // Border Colors
    // ==========================================

    fn border(&self) -> Color {
        Color::Rgb(62, 62, 66) // #3E3E42
    }

    fn border_active(&self) -> Color {
        Color::Rgb(0, 122, 204) // #007ACC - Cursor blue
    }

    fn border_inactive(&self) -> Color {
        Color::Rgb(51, 51, 51) // #333333
    }

    // ==========================================
    // Status Bar Colors
    // ==========================================

    fn status_bar_bg(&self) -> Color {
        Color::Rgb(0, 122, 204) // #007ACC - Cursor signature blue
    }

    fn status_bar_fg(&self) -> Color {
        Color::Rgb(255, 255, 255) // #FFFFFF
    }

    fn status_bar_inactive(&self) -> Color {
        Color::Rgb(96, 96, 96) // #606060
    }

    // ==========================================
    // Accent Colors
    // ==========================================

    fn accent_primary(&self) -> Color {
        Color::Rgb(0, 122, 204) // #007ACC - Cursor blue
    }

    fn accent_primary_hover(&self) -> Color {
        Color::Rgb(20, 142, 224) // #148EE0
    }

    // ==========================================
    // Semantic Colors
    // ==========================================

    fn success(&self) -> Color {
        Color::Rgb(16, 185, 129) // #10B981 - Green
    }

    fn info(&self) -> Color {
        Color::Rgb(59, 130, 246) // #3B82F6 - Blue
    }

    fn warning(&self) -> Color {
        Color::Rgb(245, 158, 11) // #F59E0B - Orange
    }

    fn error(&self) -> Color {
        Color::Rgb(239, 68, 68) // #EF4444 - Red
    }

    // ==========================================
    // Agent Status Colors
    // ==========================================

    fn agent_running(&self) -> Color {
        Color::Rgb(59, 130, 246) // #3B82F6 - Blue
    }

    fn agent_success(&self) -> Color {
        Color::Rgb(16, 185, 129) // #10B981 - Green
    }

    fn agent_error(&self) -> Color {
        Color::Rgb(239, 68, 68) // #EF4444 - Red
    }

    fn agent_idle(&self) -> Color {
        Color::Rgb(133, 133, 133) // #858585 - Gray
    }

    // ==========================================
    // Syntax Highlighting Colors
    // ==========================================

    fn syntax_keyword(&self) -> Color {
        Color::Rgb(197, 134, 192) // #C586C0 - Purple
    }

    fn syntax_control(&self) -> Color {
        Color::Rgb(216, 160, 223) // #D8A0DF - Light purple
    }

    fn syntax_type(&self) -> Color {
        Color::Rgb(78, 201, 176) // #4EC9B0 - Teal
    }

    fn syntax_class(&self) -> Color {
        Color::Rgb(78, 201, 176) // #4EC9B0 - Teal
    }

    fn syntax_function(&self) -> Color {
        Color::Rgb(220, 220, 170) // #DCDCAA - Yellow
    }

    fn syntax_method(&self) -> Color {
        Color::Rgb(220, 220, 170) // #DCDCAA - Yellow
    }

    fn syntax_string(&self) -> Color {
        Color::Rgb(206, 145, 120) // #CE9178 - Orange
    }

    fn syntax_char(&self) -> Color {
        Color::Rgb(206, 145, 120) // #CE9178 - Orange
    }

    fn syntax_number(&self) -> Color {
        Color::Rgb(181, 206, 168) // #B5CEA8 - Light green
    }

    fn syntax_comment(&self) -> Color {
        Color::Rgb(106, 153, 85) // #6A9955 - Green
    }

    fn syntax_doc_comment(&self) -> Color {
        Color::Rgb(106, 153, 85) // #6A9955 - Green
    }

    fn syntax_variable(&self) -> Color {
        Color::Rgb(156, 220, 254) // #9CDCFE - Light blue
    }

    fn syntax_parameter(&self) -> Color {
        Color::Rgb(156, 220, 254) // #9CDCFE - Light blue
    }

    fn syntax_constant(&self) -> Color {
        Color::Rgb(86, 156, 214) // #569CD6 - Blue
    }

    fn syntax_enum(&self) -> Color {
        Color::Rgb(78, 201, 176) // #4EC9B0 - Teal
    }

    fn syntax_operator(&self) -> Color {
        Color::Rgb(212, 212, 212) // #D4D4D4 - Light gray
    }

    fn syntax_macro(&self) -> Color {
        Color::Rgb(189, 147, 249) // #BD93F9 - Purple
    }

    fn syntax_attribute(&self) -> Color {
        Color::Rgb(220, 220, 170) // #DCDCAA - Yellow
    }
}

// ==========================================
// Helper Functions
// ==========================================

/// Convert RGB to ANSI 256-color palette
///
/// Useful for terminals that don't support true color (24-bit RGB).
/// Uses the standard xterm 256-color palette approximation.
///
/// # Arguments
///
/// * `r` - Red component (0-255)
/// * `g` - Green component (0-255)
/// * `b` - Blue component (0-255)
///
/// # Returns
///
/// ANSI 256-color code (0-255)
///
/// # Examples
///
/// ```rust
/// use ait42_tui::themes::cursor::rgb_to_ansi256;
///
/// // Cursor blue #007ACC
/// let ansi_blue = rgb_to_ansi256(0, 122, 204);
/// assert!(ansi_blue > 0);
/// ```
pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // Grayscale detection
    if r == g && g == b {
        if r < 8 {
            return 16; // Black
        }
        if r > 248 {
            return 231; // White
        }
        // Grayscale ramp (232-255)
        return ((r - 8) / 10) + 232;
    }

    // Color cube (16-231)
    // Convert RGB (0-255) to 6x6x6 color cube (0-5)
    let r = (r as u16 * 5 / 255) as u8;
    let g = (g as u16 * 5 / 255) as u8;
    let b = (b as u16 * 5 / 255) as u8;

    // Calculate index in 6x6x6 cube
    16 + 36 * r + 6 * g + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_theme_colors() {
        let theme = CursorTheme::new();

        // Test base colors
        assert_eq!(theme.background(), Color::Rgb(30, 30, 30));
        assert_eq!(theme.foreground(), Color::Rgb(204, 204, 204));

        // Test accent color (Cursor blue)
        assert_eq!(theme.accent_primary(), Color::Rgb(0, 122, 204));
        assert_eq!(theme.status_bar_bg(), Color::Rgb(0, 122, 204));

        // Test semantic colors
        assert_eq!(theme.success(), Color::Rgb(16, 185, 129));
        assert_eq!(theme.error(), Color::Rgb(239, 68, 68));
        assert_eq!(theme.warning(), Color::Rgb(245, 158, 11));
        assert_eq!(theme.info(), Color::Rgb(59, 130, 246));
    }

    #[test]
    fn test_theme_metadata() {
        let theme = CursorTheme::new();
        assert_eq!(theme.name(), "Cursor Dark");
        assert!(!theme.description().is_empty());
        assert_eq!(theme.version(), "1.0.0");
    }

    #[test]
    fn test_tab_colors() {
        let theme = CursorTheme::new();
        assert_eq!(theme.tab_active_bg(), Color::Rgb(45, 45, 45));
        assert_eq!(theme.tab_active_fg(), Color::Rgb(255, 255, 255));
        assert_eq!(theme.tab_inactive_bg(), Color::Rgb(37, 37, 37));
        assert_eq!(theme.tab_inactive_fg(), Color::Rgb(133, 133, 133));
    }

    #[test]
    fn test_syntax_colors() {
        let theme = CursorTheme::new();

        // Ensure syntax colors are distinct
        let keyword = theme.syntax_keyword();
        let string = theme.syntax_string();
        let comment = theme.syntax_comment();

        assert_ne!(keyword, string);
        assert_ne!(keyword, comment);
        assert_ne!(string, comment);
    }

    #[test]
    fn test_agent_colors() {
        let theme = CursorTheme::new();

        // Ensure agent status colors are distinct
        let running = theme.agent_running();
        let success = theme.agent_success();
        let error = theme.agent_error();
        let idle = theme.agent_idle();

        assert_ne!(running, success);
        assert_ne!(running, error);
        assert_ne!(success, error);
        assert_ne!(idle, running);
    }

    #[test]
    fn test_rgb_to_ansi256_grayscale() {
        // Black
        assert_eq!(rgb_to_ansi256(0, 0, 0), 16);

        // White
        assert_eq!(rgb_to_ansi256(255, 255, 255), 231);

        // Mid gray
        let gray = rgb_to_ansi256(128, 128, 128);
        assert!(gray >= 232 && gray <= 255);
    }

    #[test]
    fn test_rgb_to_ansi256_colors() {
        // Cursor blue #007ACC
        let blue = rgb_to_ansi256(0, 122, 204);
        assert!(blue >= 16 && blue < 232);

        // Should produce consistent results
        assert_eq!(rgb_to_ansi256(0, 122, 204), rgb_to_ansi256(0, 122, 204));
    }

    #[test]
    fn test_default_implementation() {
        let theme1 = CursorTheme::default();
        let theme2 = CursorTheme::new();

        assert_eq!(theme1.background(), theme2.background());
        assert_eq!(theme1.foreground(), theme2.foreground());
    }
}
