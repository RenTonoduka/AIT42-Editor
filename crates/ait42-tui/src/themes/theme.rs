//! Theme trait definition for AIT42 TUI Editor
//!
//! Defines the interface that all themes must implement, providing
//! a consistent API for accessing colors across all UI components.

use ratatui::style::Color;

/// Theme trait for AIT42 TUI Editor
///
/// All theme implementations must provide colors for:
/// - Base colors (backgrounds, foregrounds)
/// - UI elements (tabs, sidebar, borders)
/// - Syntax highlighting
/// - Semantic colors (success, error, etc.)
/// - Agent status colors
pub trait Theme {
    // ==========================================
    // Base Colors
    // ==========================================

    /// Main background color for the editor
    fn background(&self) -> Color;

    /// Lighter background for elevated surfaces (panels, modals)
    fn background_light(&self) -> Color;

    /// Deepest background layer (terminal, deep panels)
    fn background_deep(&self) -> Color {
        self.background()
    }

    /// Even lighter background for active/hover states
    fn background_lighter(&self) -> Color {
        self.background_light()
    }

    /// Primary foreground color for main text
    fn foreground(&self) -> Color;

    /// Dimmed foreground for secondary text
    fn foreground_dim(&self) -> Color;

    /// Extra dimmed foreground for tertiary text
    fn foreground_dimmer(&self) -> Color {
        self.foreground_dim()
    }

    // ==========================================
    // Tab Colors
    // ==========================================

    /// Background color for active tab
    fn tab_active_bg(&self) -> Color;

    /// Foreground color for active tab
    fn tab_active_fg(&self) -> Color;

    /// Background color for inactive tab
    fn tab_inactive_bg(&self) -> Color;

    /// Foreground color for inactive tab
    fn tab_inactive_fg(&self) -> Color;

    /// Border color for tabs
    fn tab_border(&self) -> Color {
        self.border()
    }

    // ==========================================
    // Sidebar Colors
    // ==========================================

    /// Background color for sidebar
    fn sidebar_bg(&self) -> Color;

    /// Foreground color for sidebar text
    fn sidebar_fg(&self) -> Color;

    /// Background color for hovered sidebar items
    fn sidebar_hover_bg(&self) -> Color;

    /// Background color for active/selected sidebar items
    fn sidebar_active_bg(&self) -> Color;

    // ==========================================
    // Terminal Colors
    // ==========================================

    /// Background color for terminal
    fn terminal_bg(&self) -> Color {
        self.background_deep()
    }

    /// Foreground color for terminal text
    fn terminal_fg(&self) -> Color {
        self.foreground()
    }

    /// Cursor color for terminal
    fn terminal_cursor(&self) -> Color {
        self.foreground()
    }

    /// Selection background color for terminal
    fn terminal_selection(&self) -> Color;

    // ==========================================
    // Border Colors
    // ==========================================

    /// Default border color
    fn border(&self) -> Color;

    /// Border color for active/focused elements
    fn border_active(&self) -> Color;

    /// Border color for inactive elements
    fn border_inactive(&self) -> Color {
        self.border()
    }

    // ==========================================
    // Status Bar Colors
    // ==========================================

    /// Background color for status bar
    fn status_bar_bg(&self) -> Color;

    /// Foreground color for status bar text
    fn status_bar_fg(&self) -> Color;

    /// Color for inactive status bar elements
    fn status_bar_inactive(&self) -> Color {
        self.foreground_dim()
    }

    // ==========================================
    // Accent Colors
    // ==========================================

    /// Primary accent color (buttons, highlights)
    fn accent_primary(&self) -> Color {
        self.border_active()
    }

    /// Primary accent color on hover
    fn accent_primary_hover(&self) -> Color {
        self.accent_primary()
    }

    // ==========================================
    // Semantic Colors
    // ==========================================

    /// Success state color (green)
    fn success(&self) -> Color;

    /// Info state color (blue)
    fn info(&self) -> Color;

    /// Warning state color (yellow/orange)
    fn warning(&self) -> Color;

    /// Error state color (red)
    fn error(&self) -> Color;

    // ==========================================
    // Agent Status Colors
    // ==========================================

    /// Color for running agent status
    fn agent_running(&self) -> Color;

    /// Color for successful agent completion
    fn agent_success(&self) -> Color;

    /// Color for agent error state
    fn agent_error(&self) -> Color;

    /// Color for idle agent state
    fn agent_idle(&self) -> Color;

    // ==========================================
    // Syntax Highlighting Colors
    // ==========================================

    /// Color for language keywords (if, for, while, etc.)
    fn syntax_keyword(&self) -> Color;

    /// Color for control flow keywords (return, break, continue)
    fn syntax_control(&self) -> Color {
        self.syntax_keyword()
    }

    /// Color for type names
    fn syntax_type(&self) -> Color;

    /// Color for class names
    fn syntax_class(&self) -> Color {
        self.syntax_type()
    }

    /// Color for function names
    fn syntax_function(&self) -> Color;

    /// Color for method names
    fn syntax_method(&self) -> Color {
        self.syntax_function()
    }

    /// Color for string literals
    fn syntax_string(&self) -> Color;

    /// Color for character literals
    fn syntax_char(&self) -> Color {
        self.syntax_string()
    }

    /// Color for numeric literals
    fn syntax_number(&self) -> Color;

    /// Color for comments
    fn syntax_comment(&self) -> Color;

    /// Color for documentation comments
    fn syntax_doc_comment(&self) -> Color {
        self.syntax_comment()
    }

    /// Color for variables
    fn syntax_variable(&self) -> Color;

    /// Color for function parameters
    fn syntax_parameter(&self) -> Color {
        self.syntax_variable()
    }

    /// Color for constants
    fn syntax_constant(&self) -> Color;

    /// Color for enum variants
    fn syntax_enum(&self) -> Color {
        self.syntax_constant()
    }

    /// Color for operators (+, -, *, /, etc.)
    fn syntax_operator(&self) -> Color;

    /// Color for macros (Rust) or decorators (Python)
    fn syntax_macro(&self) -> Color {
        self.syntax_function()
    }

    /// Color for attributes (Rust) or annotations
    fn syntax_attribute(&self) -> Color {
        self.syntax_macro()
    }
}
