//! View State
//!
//! Manages viewport information and scrolling.

/// View state for managing viewport
#[derive(Debug, Clone)]
pub struct ViewState {
    /// First visible line (scroll offset)
    scroll_offset: usize,
    /// Viewport height in lines
    viewport_height: usize,
    /// Viewport width in columns
    viewport_width: usize,
}

impl ViewState {
    /// Create new view state
    pub fn new(viewport_height: usize, viewport_width: usize) -> Self {
        Self {
            scroll_offset: 0,
            viewport_height,
            viewport_width,
        }
    }

    /// Get scroll offset (first visible line)
    #[inline]
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get viewport height
    #[inline]
    pub fn viewport_height(&self) -> usize {
        self.viewport_height
    }

    /// Get viewport width
    #[inline]
    pub fn viewport_width(&self) -> usize {
        self.viewport_width
    }

    /// Set viewport dimensions
    pub fn set_viewport_size(&mut self, height: usize, width: usize) {
        self.viewport_height = height;
        self.viewport_width = width;
    }

    /// Get last visible line
    #[inline]
    pub fn last_visible_line(&self) -> usize {
        self.scroll_offset + self.viewport_height
    }

    /// Check if line is visible
    #[inline]
    pub fn is_line_visible(&self, line: usize) -> bool {
        line >= self.scroll_offset && line < self.last_visible_line()
    }

    /// Ensure cursor line is visible
    ///
    /// Adjusts scroll offset to make the cursor line visible.
    pub fn ensure_cursor_visible(&mut self, cursor_line: usize) {
        // Cursor above viewport - scroll up
        if cursor_line < self.scroll_offset {
            self.scroll_offset = cursor_line;
        }
        // Cursor below viewport - scroll down
        else if cursor_line >= self.last_visible_line() {
            self.scroll_offset = cursor_line.saturating_sub(self.viewport_height - 1);
        }
    }

    /// Scroll up by number of lines
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Scroll down by number of lines
    pub fn scroll_down(&mut self, lines: usize, max_line: usize) {
        let new_offset = self.scroll_offset + lines;
        let max_offset = max_line.saturating_sub(self.viewport_height);
        self.scroll_offset = new_offset.min(max_offset);
    }

    /// Scroll to specific line
    pub fn scroll_to(&mut self, line: usize, max_line: usize) {
        let max_offset = max_line.saturating_sub(self.viewport_height);
        self.scroll_offset = line.min(max_offset);
    }

    /// Scroll to top of buffer
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to bottom of buffer
    pub fn scroll_to_bottom(&mut self, max_line: usize) {
        self.scroll_offset = max_line.saturating_sub(self.viewport_height);
    }

    /// Page up (scroll up by viewport height)
    pub fn page_up(&mut self) {
        self.scroll_up(self.viewport_height);
    }

    /// Page down (scroll down by viewport height)
    pub fn page_down(&mut self, max_line: usize) {
        self.scroll_down(self.viewport_height, max_line);
    }

    /// Center cursor line in viewport
    pub fn center_cursor(&mut self, cursor_line: usize, max_line: usize) {
        let target_offset = cursor_line.saturating_sub(self.viewport_height / 2);
        let max_offset = max_line.saturating_sub(self.viewport_height);
        self.scroll_offset = target_offset.min(max_offset);
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self::new(24, 80) // Default terminal size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_state_creation() {
        let view = ViewState::new(24, 80);
        assert_eq!(view.scroll_offset(), 0);
        assert_eq!(view.viewport_height(), 24);
        assert_eq!(view.viewport_width(), 80);
    }

    #[test]
    fn test_view_state_visibility() {
        let view = ViewState::new(10, 80);
        assert!(view.is_line_visible(0));
        assert!(view.is_line_visible(9));
        assert!(!view.is_line_visible(10));
    }

    #[test]
    fn test_ensure_cursor_visible_scroll_down() {
        let mut view = ViewState::new(10, 80);
        view.ensure_cursor_visible(15);

        // Should scroll down to make line 15 visible
        assert!(view.is_line_visible(15));
        assert_eq!(view.scroll_offset(), 6); // 15 - 10 + 1
    }

    #[test]
    fn test_ensure_cursor_visible_scroll_up() {
        let mut view = ViewState::new(10, 80);
        view.scroll_offset = 10;

        view.ensure_cursor_visible(5);

        // Should scroll up to make line 5 visible
        assert!(view.is_line_visible(5));
        assert_eq!(view.scroll_offset(), 5);
    }

    #[test]
    fn test_scroll_up_down() {
        let mut view = ViewState::new(10, 80);

        view.scroll_down(5, 100);
        assert_eq!(view.scroll_offset(), 5);

        view.scroll_up(3);
        assert_eq!(view.scroll_offset(), 2);
    }

    #[test]
    fn test_page_up_down() {
        let mut view = ViewState::new(10, 80);

        view.page_down(100);
        assert_eq!(view.scroll_offset(), 10);

        view.page_up();
        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn test_center_cursor() {
        let mut view = ViewState::new(10, 80);
        view.center_cursor(20, 100);

        // Cursor at line 20 should be centered
        // Offset = 20 - 10/2 = 15
        assert_eq!(view.scroll_offset(), 15);
        assert!(view.is_line_visible(20));
    }
}
