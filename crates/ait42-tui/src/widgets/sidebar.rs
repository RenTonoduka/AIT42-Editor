//! Sidebar Widget
//!
//! Displays a file tree hierarchy with expand/collapse controls and file type icons.

use crate::theme::Theme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};
use std::path::{Path, PathBuf};

/// File entry type
#[derive(Debug, Clone, PartialEq)]
pub enum FileEntryType {
    File,
    Directory,
}

/// A single entry in the file tree
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// File or directory path
    pub path: PathBuf,
    /// Entry type (file or directory)
    pub entry_type: FileEntryType,
    /// Indentation level (0 for root)
    pub level: usize,
    /// Whether this directory is expanded (only relevant for directories)
    pub expanded: bool,
}

impl FileEntry {
    /// Create a new file entry
    pub fn new(path: PathBuf, entry_type: FileEntryType, level: usize) -> Self {
        Self {
            path,
            entry_type,
            level,
            expanded: false,
        }
    }

    /// Create a file entry
    pub fn file(path: PathBuf, level: usize) -> Self {
        Self::new(path, FileEntryType::File, level)
    }

    /// Create a directory entry
    pub fn directory(path: PathBuf, level: usize) -> Self {
        Self::new(path, FileEntryType::Directory, level)
    }

    /// Set expanded status
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Get the file name for display
    fn display_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string()
    }

    /// Get the icon/emoji for this entry
    fn icon(&self) -> &'static str {
        match self.entry_type {
            FileEntryType::Directory => {
                if self.expanded {
                    "▼"
                } else {
                    "▶"
                }
            }
            FileEntryType::File => {
                // Get file extension-based icon
                if let Some(ext) = self.path.extension().and_then(|e| e.to_str()) {
                    match ext {
                        "rs" => "🦀",
                        "toml" => "⚙",
                        "md" => "📝",
                        "json" => "{}",
                        "yaml" | "yml" => "📋",
                        "txt" => "📄",
                        "js" | "ts" => "📜",
                        "py" => "🐍",
                        "go" => "🐹",
                        "html" => "🌐",
                        "css" => "🎨",
                        "png" | "jpg" | "jpeg" | "gif" => "🖼",
                        _ => "📄",
                    }
                } else {
                    "📄"
                }
            }
        }
    }
}

/// File tree structure
#[derive(Debug, Clone)]
pub struct FileTree {
    /// Root directory path
    pub root: PathBuf,
    /// List of file entries
    pub entries: Vec<FileEntry>,
}

impl FileTree {
    /// Create a new file tree
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            entries: Vec::new(),
        }
    }

    /// Add an entry to the tree
    pub fn add_entry(&mut self, entry: FileEntry) {
        self.entries.push(entry);
    }

    /// Create a simple example tree for testing
    pub fn example() -> Self {
        let mut tree = Self::new(PathBuf::from("/project"));

        tree.add_entry(FileEntry::directory(PathBuf::from("/project/src"), 0).expanded(true));
        tree.add_entry(FileEntry::file(PathBuf::from("/project/src/main.rs"), 1));
        tree.add_entry(FileEntry::file(PathBuf::from("/project/src/lib.rs"), 1));
        tree.add_entry(FileEntry::directory(PathBuf::from("/project/tests"), 0).expanded(false));
        tree.add_entry(FileEntry::file(PathBuf::from("/project/Cargo.toml"), 0));
        tree.add_entry(FileEntry::file(PathBuf::from("/project/README.md"), 0));

        tree
    }
}

/// Sidebar widget displaying a file tree
pub struct Sidebar<'a> {
    file_tree: &'a FileTree,
    selected_index: usize,
    theme: &'a Theme,
    scroll_offset: usize,
    show_header: bool,
}

impl<'a> Sidebar<'a> {
    /// Create a new sidebar
    pub fn new(file_tree: &'a FileTree, selected_index: usize, theme: &'a Theme) -> Self {
        Self {
            file_tree,
            selected_index,
            theme,
            scroll_offset: 0,
            show_header: true,
        }
    }

    /// Set scroll offset for viewing entries beyond visible area
    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    /// Hide the "FILES" header
    pub fn hide_header(mut self) -> Self {
        self.show_header = false;
        self
    }

    /// Render the header
    fn render_header(&self, area: Rect, buf: &mut Buffer) -> u16 {
        if !self.show_header || area.height < 2 {
            return 0;
        }

        let header = " FILES ";
        let style = Style::default()
            .fg(self.theme.foreground)
            .add_modifier(Modifier::BOLD);

        buf.set_string(area.x, area.y, header, style);

        // Draw separator line
        let separator_style = Style::default().fg(self.theme.border.fg.unwrap_or(self.theme.foreground));
        for x in area.left()..area.right() {
            buf.get_mut(x, area.y + 1).set_char('─').set_style(separator_style);
        }

        2 // Header takes 2 lines
    }

    /// Render a single file entry
    fn render_entry(
        &self,
        entry: &FileEntry,
        y: u16,
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
    ) {
        if y >= area.bottom() {
            return;
        }

        let style = if is_selected {
            Style::default()
                .bg(self.theme.selection.bg.unwrap_or(self.theme.background))
                .fg(self.theme.foreground)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.foreground)
        };

        // Calculate indentation
        let indent = "  ".repeat(entry.level);
        let icon = entry.icon();
        let name = entry.display_name();

        // Build the full line
        let line = format!("{}{} {}", indent, icon, name);

        // Truncate if too long
        let max_width = area.width.saturating_sub(1) as usize;
        let display_line = if line.len() > max_width {
            format!("{}…", &line[..max_width.saturating_sub(1)])
        } else {
            line
        };

        // Clear the line first
        for x in area.left()..area.right() {
            buf.get_mut(x, y).set_char(' ').set_style(style);
        }

        // Render the text
        buf.set_string(area.x, y, &display_line, style);
    }
}

impl<'a> Widget for Sidebar<'a> {
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

        // Render header and get starting y position
        let mut y_offset = area.y;
        if self.show_header {
            let header_height = self.render_header(area, buf);
            y_offset += header_height;
        }

        // Calculate visible entries
        let visible_height = area.bottom().saturating_sub(y_offset) as usize;
        let start_index = self.scroll_offset;
        let end_index = (start_index + visible_height).min(self.file_tree.entries.len());

        // Render visible entries
        for (i, entry) in self
            .file_tree
            .entries
            .iter()
            .enumerate()
            .skip(start_index)
            .take(end_index - start_index)
        {
            let is_selected = i == self.selected_index;
            self.render_entry(entry, y_offset, is_selected, area, buf);
            y_offset += 1;

            if y_offset >= area.bottom() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_entry_creation() {
        let entry = FileEntry::file(PathBuf::from("/test/file.rs"), 0);
        assert_eq!(entry.entry_type, FileEntryType::File);
        assert_eq!(entry.level, 0);
        assert!(!entry.expanded);
    }

    #[test]
    fn test_directory_entry() {
        let entry = FileEntry::directory(PathBuf::from("/test/dir"), 0).expanded(true);
        assert_eq!(entry.entry_type, FileEntryType::Directory);
        assert!(entry.expanded);
    }

    #[test]
    fn test_file_entry_display_name() {
        let entry = FileEntry::file(PathBuf::from("/test/file.rs"), 0);
        assert_eq!(entry.display_name(), "file.rs");
    }

    #[test]
    fn test_file_entry_icons() {
        let rs_file = FileEntry::file(PathBuf::from("test.rs"), 0);
        assert_eq!(rs_file.icon(), "🦀");

        let md_file = FileEntry::file(PathBuf::from("test.md"), 0);
        assert_eq!(md_file.icon(), "📝");

        let dir_collapsed = FileEntry::directory(PathBuf::from("src"), 0);
        assert_eq!(dir_collapsed.icon(), "▶");

        let dir_expanded = FileEntry::directory(PathBuf::from("src"), 0).expanded(true);
        assert_eq!(dir_expanded.icon(), "▼");
    }

    #[test]
    fn test_file_tree_creation() {
        let tree = FileTree::new(PathBuf::from("/project"));
        assert_eq!(tree.root, PathBuf::from("/project"));
        assert!(tree.entries.is_empty());
    }

    #[test]
    fn test_file_tree_add_entry() {
        let mut tree = FileTree::new(PathBuf::from("/project"));
        tree.add_entry(FileEntry::file(PathBuf::from("/project/main.rs"), 0));
        assert_eq!(tree.entries.len(), 1);
    }

    #[test]
    fn test_example_tree() {
        let tree = FileTree::example();
        assert!(!tree.entries.is_empty());
        assert_eq!(tree.root, PathBuf::from("/project"));
    }

    #[test]
    fn test_sidebar_creation() {
        let theme = Theme::default();
        let tree = FileTree::example();
        let sidebar = Sidebar::new(&tree, 0, &theme);

        assert_eq!(sidebar.selected_index, 0);
        assert_eq!(sidebar.scroll_offset, 0);
        assert!(sidebar.show_header);
    }

    #[test]
    fn test_sidebar_scroll_offset() {
        let theme = Theme::default();
        let tree = FileTree::example();
        let sidebar = Sidebar::new(&tree, 0, &theme).scroll_offset(5);

        assert_eq!(sidebar.scroll_offset, 5);
    }

    #[test]
    fn test_sidebar_hide_header() {
        let theme = Theme::default();
        let tree = FileTree::example();
        let sidebar = Sidebar::new(&tree, 0, &theme).hide_header();

        assert!(!sidebar.show_header);
    }
}
