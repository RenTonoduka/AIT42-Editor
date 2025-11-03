//! Theme system for AIT42 TUI Editor
//!
//! Provides a flexible theming system with support for:
//! - Multiple color schemes (Cursor Dark, Default)
//! - Syntax highlighting colors
//! - UI element styling
//! - Runtime theme switching
//!
//! # Available Themes
//!
//! - **Cursor Dark**: Professional theme inspired by Cursor AI editor (default)
//! - **Default**: High-contrast fallback theme with ANSI colors
//!
//! # Examples
//!
//! ## Using Cursor theme
//!
//! ```rust
//! use ait42_tui::themes::{CursorTheme, Theme};
//! use ratatui::style::Style;
//!
//! let theme = CursorTheme::new();
//! let editor_style = Style::default()
//!     .bg(theme.background())
//!     .fg(theme.foreground());
//! ```
//!
//! ## Theme switching
//!
//! ```rust
//! use ait42_tui::themes::{ThemeVariant, Theme};
//!
//! let theme_variant = ThemeVariant::Cursor;
//! let theme = theme_variant.get_theme();
//! ```
//!
//! ## Syntax highlighting
//!
//! ```rust
//! use ait42_tui::themes::{CursorTheme, Theme};
//! use ratatui::style::Style;
//!
//! let theme = CursorTheme::new();
//!
//! let keyword_style = Style::default().fg(theme.syntax_keyword());
//! let string_style = Style::default().fg(theme.syntax_string());
//! let comment_style = Style::default().fg(theme.syntax_comment());
//! ```

mod theme;
mod cursor;
mod default;

pub use theme::Theme;
pub use cursor::{CursorTheme, rgb_to_ansi256};
pub use default::DefaultTheme;

/// Available theme variants
///
/// Represents all built-in themes available in AIT42 Editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeVariant {
    /// Cursor Dark - Professional theme inspired by Cursor AI (default)
    Cursor,
    /// Default - High-contrast ANSI theme for maximum compatibility
    Default,
}

impl ThemeVariant {
    /// Get a theme instance
    ///
    /// Returns a boxed trait object implementing the Theme trait.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ait42_tui::themes::ThemeVariant;
    ///
    /// let theme = ThemeVariant::Cursor.get_theme();
    /// ```
    pub fn get_theme(&self) -> Box<dyn Theme> {
        match self {
            ThemeVariant::Cursor => Box::new(CursorTheme::new()),
            ThemeVariant::Default => Box::new(DefaultTheme::new()),
        }
    }

    /// List all available theme variants
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ait42_tui::themes::ThemeVariant;
    ///
    /// for variant in ThemeVariant::all() {
    ///     println!("Theme: {}", variant.name());
    /// }
    /// ```
    pub fn all() -> &'static [ThemeVariant] {
        &[ThemeVariant::Cursor, ThemeVariant::Default]
    }

    /// Get the human-readable theme name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ait42_tui::themes::ThemeVariant;
    ///
    /// assert_eq!(ThemeVariant::Cursor.name(), "Cursor Dark");
    /// assert_eq!(ThemeVariant::Default.name(), "Default");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            ThemeVariant::Cursor => "Cursor Dark",
            ThemeVariant::Default => "Default",
        }
    }

    /// Get theme description
    pub fn description(&self) -> &'static str {
        match self {
            ThemeVariant::Cursor => "Professional dark theme inspired by Cursor AI editor",
            ThemeVariant::Default => "Simple high-contrast theme with maximum compatibility",
        }
    }

    /// Parse theme variant from string
    ///
    /// # Arguments
    ///
    /// * `s` - Theme name (case-insensitive)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ait42_tui::themes::ThemeVariant;
    ///
    /// assert_eq!(ThemeVariant::from_str("cursor"), Some(ThemeVariant::Cursor));
    /// assert_eq!(ThemeVariant::from_str("CURSOR"), Some(ThemeVariant::Cursor));
    /// assert_eq!(ThemeVariant::from_str("default"), Some(ThemeVariant::Default));
    /// assert_eq!(ThemeVariant::from_str("unknown"), None);
    /// ```
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cursor" | "cursor-dark" | "cursor_dark" => Some(ThemeVariant::Cursor),
            "default" => Some(ThemeVariant::Default),
            _ => None,
        }
    }
}

impl Default for ThemeVariant {
    /// Default theme is Cursor Dark
    fn default() -> Self {
        ThemeVariant::Cursor
    }
}

impl std::fmt::Display for ThemeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for ThemeVariant {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s).ok_or_else(|| format!("Unknown theme: {}", s))
    }
}

// ==========================================
// Presets and Utilities
// ==========================================

/// Get the default theme (Cursor Dark)
///
/// # Examples
///
/// ```rust
/// use ait42_tui::themes::default_theme;
///
/// let theme = default_theme();
/// ```
pub fn default_theme() -> Box<dyn Theme> {
    Box::new(CursorTheme::new())
}

/// Get a theme by name
///
/// Returns None if theme name is not recognized.
///
/// # Examples
///
/// ```rust
/// use ait42_tui::themes::get_theme_by_name;
///
/// let theme = get_theme_by_name("cursor").unwrap();
/// let unknown = get_theme_by_name("unknown");
/// assert!(unknown.is_none());
/// ```
pub fn get_theme_by_name(name: &str) -> Option<Box<dyn Theme>> {
    ThemeVariant::from_str(name).map(|v| v.get_theme())
}

/// List all available theme names
///
/// # Examples
///
/// ```rust
/// use ait42_tui::themes::list_theme_names;
///
/// let names = list_theme_names();
/// assert!(names.contains(&"Cursor Dark"));
/// assert!(names.contains(&"Default"));
/// ```
pub fn list_theme_names() -> Vec<&'static str> {
    ThemeVariant::all()
        .iter()
        .map(|v| v.name())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_variant_display() {
        assert_eq!(format!("{}", ThemeVariant::Cursor), "Cursor Dark");
        assert_eq!(format!("{}", ThemeVariant::Default), "Default");
    }

    #[test]
    fn test_theme_variant_from_str() {
        assert_eq!(ThemeVariant::from_str("cursor"), Some(ThemeVariant::Cursor));
        assert_eq!(ThemeVariant::from_str("CURSOR"), Some(ThemeVariant::Cursor));
        assert_eq!(ThemeVariant::from_str("cursor-dark"), Some(ThemeVariant::Cursor));
        assert_eq!(ThemeVariant::from_str("default"), Some(ThemeVariant::Default));
        assert_eq!(ThemeVariant::from_str("unknown"), None);
    }

    #[test]
    fn test_default_theme_variant() {
        assert_eq!(ThemeVariant::default(), ThemeVariant::Cursor);
    }

    #[test]
    fn test_get_theme_by_name() {
        assert!(get_theme_by_name("cursor").is_some());
        assert!(get_theme_by_name("default").is_some());
        assert!(get_theme_by_name("unknown").is_none());
    }

    #[test]
    fn test_list_theme_names() {
        let names = list_theme_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Cursor Dark"));
        assert!(names.contains(&"Default"));
    }

    #[test]
    fn test_theme_variant_all() {
        let variants = ThemeVariant::all();
        assert_eq!(variants.len(), 2);
        assert!(variants.contains(&ThemeVariant::Cursor));
        assert!(variants.contains(&ThemeVariant::Default));
    }
}
