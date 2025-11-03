//! Mode System
//!
//! Implements Vim-style modal editing.

/// Editor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Normal mode (navigation and commands)
    Normal,
    /// Insert mode (text insertion)
    Insert,
    /// Visual mode (text selection)
    Visual,
    /// Command mode (Ex commands like :w, :q)
    Command,
}

impl Mode {
    /// Get mode indicator string for status bar
    pub fn indicator(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
            Mode::Command => "COMMAND",
        }
    }

    /// Check if mode allows text insertion
    pub fn allows_insert(&self) -> bool {
        matches!(self, Mode::Insert)
    }

    /// Check if mode allows selection
    pub fn allows_selection(&self) -> bool {
        matches!(self, Mode::Visual)
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}

/// Mode manager for mode transitions
#[derive(Debug, Clone)]
pub struct ModeManager {
    current: Mode,
    previous: Option<Mode>,
}

impl ModeManager {
    /// Create new mode manager in Normal mode
    pub fn new() -> Self {
        Self {
            current: Mode::Normal,
            previous: None,
        }
    }

    /// Get current mode
    #[inline]
    pub fn current(&self) -> Mode {
        self.current
    }

    /// Get previous mode
    #[inline]
    pub fn previous(&self) -> Option<Mode> {
        self.previous
    }

    /// Switch to new mode
    ///
    /// Stores previous mode for potential restoration.
    pub fn switch_to(&mut self, mode: Mode) {
        if self.current != mode {
            self.previous = Some(self.current);
            self.current = mode;
        }
    }

    /// Enter Insert mode
    pub fn enter_insert(&mut self) {
        self.switch_to(Mode::Insert);
    }

    /// Exit Insert mode (return to Normal)
    pub fn exit_insert(&mut self) {
        if self.current == Mode::Insert {
            self.switch_to(Mode::Normal);
        }
    }

    /// Enter Visual mode
    pub fn enter_visual(&mut self) {
        self.switch_to(Mode::Visual);
    }

    /// Exit Visual mode (return to Normal)
    pub fn exit_visual(&mut self) {
        if self.current == Mode::Visual {
            self.switch_to(Mode::Normal);
        }
    }

    /// Enter Command mode
    pub fn enter_command(&mut self) {
        self.switch_to(Mode::Command);
    }

    /// Exit Command mode (return to Normal)
    pub fn exit_command(&mut self) {
        if self.current == Mode::Command {
            self.switch_to(Mode::Normal);
        }
    }

    /// Return to previous mode
    pub fn return_to_previous(&mut self) {
        if let Some(prev) = self.previous {
            self.current = prev;
            self.previous = None;
        }
    }

    /// Check if in Normal mode
    #[inline]
    pub fn is_normal(&self) -> bool {
        self.current == Mode::Normal
    }

    /// Check if in Insert mode
    #[inline]
    pub fn is_insert(&self) -> bool {
        self.current == Mode::Insert
    }

    /// Check if in Visual mode
    #[inline]
    pub fn is_visual(&self) -> bool {
        self.current == Mode::Visual
    }

    /// Check if in Command mode
    #[inline]
    pub fn is_command(&self) -> bool {
        self.current == Mode::Command
    }
}

impl Default for ModeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_indicator() {
        assert_eq!(Mode::Normal.indicator(), "NORMAL");
        assert_eq!(Mode::Insert.indicator(), "INSERT");
        assert_eq!(Mode::Visual.indicator(), "VISUAL");
        assert_eq!(Mode::Command.indicator(), "COMMAND");
    }

    #[test]
    fn test_mode_manager_switch() {
        let mut mgr = ModeManager::new();
        assert_eq!(mgr.current(), Mode::Normal);

        mgr.enter_insert();
        assert_eq!(mgr.current(), Mode::Insert);
        assert_eq!(mgr.previous(), Some(Mode::Normal));

        mgr.exit_insert();
        assert_eq!(mgr.current(), Mode::Normal);
    }

    #[test]
    fn test_mode_manager_previous() {
        let mut mgr = ModeManager::new();

        mgr.enter_insert();
        mgr.enter_visual();

        assert_eq!(mgr.current(), Mode::Visual);
        assert_eq!(mgr.previous(), Some(Mode::Insert));

        mgr.return_to_previous();
        assert_eq!(mgr.current(), Mode::Insert);
    }

    #[test]
    fn test_mode_checks() {
        let mut mgr = ModeManager::new();

        assert!(mgr.is_normal());
        assert!(!mgr.is_insert());

        mgr.enter_insert();
        assert!(!mgr.is_normal());
        assert!(mgr.is_insert());
    }
}
