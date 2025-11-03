//! Theme System
//!
//! Provides color schemes and styling for the editor UI.

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// Editor theme with colors and styles
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub background: Color,
    pub foreground: Color,
    pub cursor: Color,
    pub selection: Style,
    pub line_number: Style,
    pub line_number_active: Style,
    pub statusline_normal: Style,
    pub statusline_insert: Style,
    pub statusline_visual: Style,
    pub statusline_command: Style,
    pub border: Style,
    pub search_match: Style,
    pub comment: Style,
    pub keyword: Style,
    pub string: Style,
    pub number: Style,
    pub function: Style,
}

impl Theme {
    /// Create default Monokai-inspired theme
    pub fn monokai() -> Self {
        Self {
            name: "Monokai".to_string(),
            background: Color::Rgb(39, 40, 34),
            foreground: Color::Rgb(248, 248, 242),
            cursor: Color::Rgb(253, 151, 31),
            selection: Style::default()
                .bg(Color::Rgb(73, 72, 62))
                .fg(Color::Rgb(248, 248, 242)),
            line_number: Style::default()
                .fg(Color::Rgb(90, 90, 90))
                .add_modifier(Modifier::DIM),
            line_number_active: Style::default()
                .fg(Color::Rgb(248, 248, 242))
                .add_modifier(Modifier::BOLD),
            statusline_normal: Style::default()
                .bg(Color::Rgb(102, 217, 239))
                .fg(Color::Rgb(39, 40, 34))
                .add_modifier(Modifier::BOLD),
            statusline_insert: Style::default()
                .bg(Color::Rgb(166, 226, 46))
                .fg(Color::Rgb(39, 40, 34))
                .add_modifier(Modifier::BOLD),
            statusline_visual: Style::default()
                .bg(Color::Rgb(253, 151, 31))
                .fg(Color::Rgb(39, 40, 34))
                .add_modifier(Modifier::BOLD),
            statusline_command: Style::default()
                .bg(Color::Rgb(174, 129, 255))
                .fg(Color::Rgb(39, 40, 34))
                .add_modifier(Modifier::BOLD),
            border: Style::default().fg(Color::Rgb(117, 113, 94)),
            search_match: Style::default()
                .bg(Color::Rgb(253, 151, 31))
                .fg(Color::Rgb(39, 40, 34))
                .add_modifier(Modifier::BOLD),
            comment: Style::default()
                .fg(Color::Rgb(117, 113, 94))
                .add_modifier(Modifier::ITALIC),
            keyword: Style::default()
                .fg(Color::Rgb(249, 38, 114))
                .add_modifier(Modifier::BOLD),
            string: Style::default().fg(Color::Rgb(230, 219, 116)),
            number: Style::default().fg(Color::Rgb(174, 129, 255)),
            function: Style::default().fg(Color::Rgb(166, 226, 46)),
        }
    }

    /// Create Solarized Dark theme
    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            background: Color::Rgb(0, 43, 54),
            foreground: Color::Rgb(131, 148, 150),
            cursor: Color::Rgb(220, 50, 47),
            selection: Style::default()
                .bg(Color::Rgb(7, 54, 66))
                .fg(Color::Rgb(131, 148, 150)),
            line_number: Style::default()
                .fg(Color::Rgb(88, 110, 117))
                .add_modifier(Modifier::DIM),
            line_number_active: Style::default()
                .fg(Color::Rgb(147, 161, 161))
                .add_modifier(Modifier::BOLD),
            statusline_normal: Style::default()
                .bg(Color::Rgb(38, 139, 210))
                .fg(Color::Rgb(0, 43, 54))
                .add_modifier(Modifier::BOLD),
            statusline_insert: Style::default()
                .bg(Color::Rgb(133, 153, 0))
                .fg(Color::Rgb(0, 43, 54))
                .add_modifier(Modifier::BOLD),
            statusline_visual: Style::default()
                .bg(Color::Rgb(203, 75, 22))
                .fg(Color::Rgb(0, 43, 54))
                .add_modifier(Modifier::BOLD),
            statusline_command: Style::default()
                .bg(Color::Rgb(108, 113, 196))
                .fg(Color::Rgb(0, 43, 54))
                .add_modifier(Modifier::BOLD),
            border: Style::default().fg(Color::Rgb(7, 54, 66)),
            search_match: Style::default()
                .bg(Color::Rgb(181, 137, 0))
                .fg(Color::Rgb(0, 43, 54))
                .add_modifier(Modifier::BOLD),
            comment: Style::default()
                .fg(Color::Rgb(88, 110, 117))
                .add_modifier(Modifier::ITALIC),
            keyword: Style::default()
                .fg(Color::Rgb(133, 153, 0))
                .add_modifier(Modifier::BOLD),
            string: Style::default().fg(Color::Rgb(42, 161, 152)),
            number: Style::default().fg(Color::Rgb(108, 113, 196)),
            function: Style::default().fg(Color::Rgb(38, 139, 210)),
        }
    }

    /// Create Gruvbox theme
    pub fn gruvbox() -> Self {
        Self {
            name: "Gruvbox".to_string(),
            background: Color::Rgb(40, 40, 40),
            foreground: Color::Rgb(235, 219, 178),
            cursor: Color::Rgb(254, 128, 25),
            selection: Style::default()
                .bg(Color::Rgb(60, 56, 54))
                .fg(Color::Rgb(235, 219, 178)),
            line_number: Style::default()
                .fg(Color::Rgb(124, 111, 100))
                .add_modifier(Modifier::DIM),
            line_number_active: Style::default()
                .fg(Color::Rgb(250, 189, 47))
                .add_modifier(Modifier::BOLD),
            statusline_normal: Style::default()
                .bg(Color::Rgb(152, 151, 26))
                .fg(Color::Rgb(40, 40, 40))
                .add_modifier(Modifier::BOLD),
            statusline_insert: Style::default()
                .bg(Color::Rgb(184, 187, 38))
                .fg(Color::Rgb(40, 40, 40))
                .add_modifier(Modifier::BOLD),
            statusline_visual: Style::default()
                .bg(Color::Rgb(215, 153, 33))
                .fg(Color::Rgb(40, 40, 40))
                .add_modifier(Modifier::BOLD),
            statusline_command: Style::default()
                .bg(Color::Rgb(177, 98, 134))
                .fg(Color::Rgb(40, 40, 40))
                .add_modifier(Modifier::BOLD),
            border: Style::default().fg(Color::Rgb(80, 73, 69)),
            search_match: Style::default()
                .bg(Color::Rgb(215, 153, 33))
                .fg(Color::Rgb(40, 40, 40))
                .add_modifier(Modifier::BOLD),
            comment: Style::default()
                .fg(Color::Rgb(146, 131, 116))
                .add_modifier(Modifier::ITALIC),
            keyword: Style::default()
                .fg(Color::Rgb(251, 73, 52))
                .add_modifier(Modifier::BOLD),
            string: Style::default().fg(Color::Rgb(184, 187, 38)),
            number: Style::default().fg(Color::Rgb(211, 134, 155)),
            function: Style::default().fg(Color::Rgb(142, 192, 124)),
        }
    }

    /// Get theme by name
    pub fn by_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "monokai" => Some(Self::monokai()),
            "solarized-dark" => Some(Self::solarized_dark()),
            "gruvbox" => Some(Self::gruvbox()),
            _ => None,
        }
    }

    /// List available theme names
    pub fn available_themes() -> Vec<&'static str> {
        vec!["monokai", "solarized-dark", "gruvbox"]
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::monokai()
    }
}

/// Theme configuration from user settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: Option<String>,
    pub custom_colors: Option<CustomColors>,
}

/// Custom color overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomColors {
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub cursor: Option<String>,
}

impl Theme {
    /// Create theme from configuration
    pub fn from_config(config: &ThemeConfig) -> Self {
        let mut theme = if let Some(name) = &config.name {
            Self::by_name(name).unwrap_or_default()
        } else {
            Self::default()
        };

        // Apply custom color overrides
        if let Some(custom) = &config.custom_colors {
            if let Some(bg) = &custom.background {
                if let Some(color) = parse_color(bg) {
                    theme.background = color;
                }
            }
            if let Some(fg) = &custom.foreground {
                if let Some(color) = parse_color(fg) {
                    theme.foreground = color;
                }
            }
            if let Some(cursor) = &custom.cursor {
                if let Some(color) = parse_color(cursor) {
                    theme.cursor = color;
                }
            }
        }

        theme
    }
}

/// Parse color from hex string (#RRGGBB)
fn parse_color(s: &str) -> Option<Color> {
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, "Monokai");
    }

    #[test]
    fn test_theme_by_name() {
        assert!(Theme::by_name("monokai").is_some());
        assert!(Theme::by_name("solarized-dark").is_some());
        assert!(Theme::by_name("gruvbox").is_some());
        assert!(Theme::by_name("nonexistent").is_none());
    }

    #[test]
    fn test_parse_color() {
        let color = parse_color("#FF0000");
        assert_eq!(color, Some(Color::Rgb(255, 0, 0)));

        let color = parse_color("#00FF00");
        assert_eq!(color, Some(Color::Rgb(0, 255, 0)));

        let color = parse_color("invalid");
        assert_eq!(color, None);
    }

    #[test]
    fn test_theme_from_config() {
        let config = ThemeConfig {
            name: Some("gruvbox".to_string()),
            custom_colors: Some(CustomColors {
                background: Some("#000000".to_string()),
                foreground: None,
                cursor: None,
            }),
        };

        let theme = Theme::from_config(&config);
        assert_eq!(theme.background, Color::Rgb(0, 0, 0));
    }

    #[test]
    fn test_available_themes() {
        let themes = Theme::available_themes();
        assert_eq!(themes.len(), 3);
        assert!(themes.contains(&"monokai"));
    }
}
