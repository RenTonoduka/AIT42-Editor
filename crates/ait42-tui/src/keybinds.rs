//! Key Binding System
//!
//! Maps keyboard input to editor commands with Vim-like bindings.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Editor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
            Mode::Command => "COMMAND",
        }
    }
}

/// Editor commands
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorCommand {
    // Mode transitions
    EnterInsertMode,
    EnterVisualMode,
    EnterCommandMode,
    EnterNormalMode,

    // Cursor movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordForward,
    MoveWordBackward,
    MoveLineStart,
    MoveLineEnd,
    MovePageUp,
    MovePageDown,
    MoveFileStart,
    MoveFileEnd,

    // Editing
    InsertChar(char),
    InsertNewline,
    DeleteChar,
    DeleteLine,
    DeleteWord,
    Backspace,

    // Undo/Redo
    Undo,
    Redo,

    // Search
    Search,
    SearchNext,
    SearchPrevious,

    // Commands
    OpenCommandPalette,
    Save,
    Quit,
    ForceQuit,

    // Other
    Noop,
}

/// Key binding (key + modifiers)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn from_key_event(event: KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

/// Key mapping for all editor modes
#[derive(Debug, Clone)]
pub struct KeyMap {
    normal_mode: HashMap<KeyBinding, EditorCommand>,
    insert_mode: HashMap<KeyBinding, EditorCommand>,
    visual_mode: HashMap<KeyBinding, EditorCommand>,
    command_mode: HashMap<KeyBinding, EditorCommand>,
}

impl KeyMap {
    /// Create default Vim-like key bindings
    pub fn default_vim() -> Self {
        let mut normal_mode = HashMap::new();
        let mut insert_mode = HashMap::new();
        let mut visual_mode = HashMap::new();
        let mut command_mode = HashMap::new();

        // Normal mode bindings
        Self::setup_normal_mode(&mut normal_mode);
        Self::setup_insert_mode(&mut insert_mode);
        Self::setup_visual_mode(&mut visual_mode);
        Self::setup_command_mode(&mut command_mode);

        Self {
            normal_mode,
            insert_mode,
            visual_mode,
            command_mode,
        }
    }

    fn setup_normal_mode(map: &mut HashMap<KeyBinding, EditorCommand>) {
        use EditorCommand::*;
        use KeyCode::*;

        // Mode transitions
        map.insert(kb(Char('i'), NONE), EnterInsertMode);
        map.insert(kb(Char('v'), NONE), EnterVisualMode);
        map.insert(kb(Char(':'), SHIFT), EnterCommandMode);

        // Cursor movement
        map.insert(kb(Char('h'), NONE), MoveLeft);
        map.insert(kb(Char('j'), NONE), MoveDown);
        map.insert(kb(Char('k'), NONE), MoveUp);
        map.insert(kb(Char('l'), NONE), MoveRight);
        map.insert(kb(Left, NONE), MoveLeft);
        map.insert(kb(Right, NONE), MoveRight);
        map.insert(kb(Up, NONE), MoveUp);
        map.insert(kb(Down, NONE), MoveDown);

        map.insert(kb(Char('w'), NONE), MoveWordForward);
        map.insert(kb(Char('b'), NONE), MoveWordBackward);
        map.insert(kb(Char('0'), NONE), MoveLineStart);
        map.insert(kb(Char('$'), SHIFT), MoveLineEnd);
        map.insert(kb(Home, NONE), MoveLineStart);
        map.insert(kb(End, NONE), MoveLineEnd);

        map.insert(kb(Char('g'), CTRL), MoveFileStart);
        map.insert(kb(Char('G'), SHIFT), MoveFileEnd);
        map.insert(kb(PageUp, NONE), MovePageUp);
        map.insert(kb(PageDown, NONE), MovePageDown);

        // Editing
        map.insert(kb(Char('x'), NONE), DeleteChar);
        map.insert(kb(Char('d'), NONE), DeleteLine);

        // Undo/Redo
        map.insert(kb(Char('u'), NONE), Undo);
        map.insert(kb(Char('r'), CTRL), Redo);

        // Search
        map.insert(kb(Char('/'), NONE), Search);
        map.insert(kb(Char('n'), NONE), SearchNext);
        map.insert(kb(Char('N'), SHIFT), SearchPrevious);

        // Commands
        map.insert(kb(Char('p'), CTRL), OpenCommandPalette);
        map.insert(kb(Char('s'), CTRL), Save);
        map.insert(kb(Char('q'), NONE), Quit);
    }

    fn setup_insert_mode(map: &mut HashMap<KeyBinding, EditorCommand>) {
        use EditorCommand::*;
        use KeyCode::*;

        // Exit insert mode
        map.insert(kb(Esc, NONE), EnterNormalMode);

        // Navigation
        map.insert(kb(Left, NONE), MoveLeft);
        map.insert(kb(Right, NONE), MoveRight);
        map.insert(kb(Up, NONE), MoveUp);
        map.insert(kb(Down, NONE), MoveDown);

        // Editing
        map.insert(kb(Enter, NONE), InsertNewline);
        map.insert(kb(Backspace, NONE), Backspace);

        // Ctrl shortcuts
        map.insert(kb(Char('s'), CTRL), Save);
        map.insert(kb(Char('p'), CTRL), OpenCommandPalette);
    }

    fn setup_visual_mode(map: &mut HashMap<KeyBinding, EditorCommand>) {
        use EditorCommand::*;
        use KeyCode::*;

        // Exit visual mode
        map.insert(kb(Esc, NONE), EnterNormalMode);

        // Movement (extends selection)
        map.insert(kb(Char('h'), NONE), MoveLeft);
        map.insert(kb(Char('j'), NONE), MoveDown);
        map.insert(kb(Char('k'), NONE), MoveUp);
        map.insert(kb(Char('l'), NONE), MoveRight);
        map.insert(kb(Left, NONE), MoveLeft);
        map.insert(kb(Right, NONE), MoveRight);
        map.insert(kb(Up, NONE), MoveUp);
        map.insert(kb(Down, NONE), MoveDown);
    }

    fn setup_command_mode(map: &mut HashMap<KeyBinding, EditorCommand>) {
        use EditorCommand::*;
        use KeyCode::*;

        // Exit command mode
        map.insert(kb(Esc, NONE), EnterNormalMode);
        map.insert(kb(Enter, NONE), EnterNormalMode);

        // Navigation
        map.insert(kb(Left, NONE), MoveLeft);
        map.insert(kb(Right, NONE), MoveRight);

        // Editing
        map.insert(kb(Backspace, NONE), Backspace);
    }

    /// Look up command for key binding in current mode
    pub fn lookup(&self, mode: Mode, key: KeyBinding) -> Option<&EditorCommand> {
        let map = match mode {
            Mode::Normal => &self.normal_mode,
            Mode::Insert => &self.insert_mode,
            Mode::Visual => &self.visual_mode,
            Mode::Command => &self.command_mode,
        };
        map.get(&key)
    }

    /// Add custom key binding
    pub fn add_binding(&mut self, mode: Mode, key: KeyBinding, command: EditorCommand) {
        let map = match mode {
            Mode::Normal => &mut self.normal_mode,
            Mode::Insert => &mut self.insert_mode,
            Mode::Visual => &mut self.visual_mode,
            Mode::Command => &mut self.command_mode,
        };
        map.insert(key, command);
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        Self::default_vim()
    }
}

/// Helper to create KeyBinding
fn kb(code: KeyCode, modifiers: KeyModifiers) -> KeyBinding {
    KeyBinding::new(code, modifiers)
}

const NONE: KeyModifiers = KeyModifiers::NONE;
const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

/// Configuration for custom key bindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindConfig {
    pub preset: Option<String>,
    pub custom: Option<HashMap<String, String>>,
}

impl KeyMap {
    /// Create from configuration
    pub fn from_config(config: &KeyBindConfig) -> Self {
        // For now, just use default
        // TODO: Implement custom binding parsing
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_mode_bindings() {
        let keymap = KeyMap::default();

        let h_key = KeyBinding::new(KeyCode::Char('h'), KeyModifiers::NONE);
        let cmd = keymap.lookup(Mode::Normal, h_key);
        assert_eq!(cmd, Some(&EditorCommand::MoveLeft));

        let j_key = KeyBinding::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let cmd = keymap.lookup(Mode::Normal, j_key);
        assert_eq!(cmd, Some(&EditorCommand::MoveDown));
    }

    #[test]
    fn test_insert_mode_bindings() {
        let keymap = KeyMap::default();

        let esc_key = KeyBinding::new(KeyCode::Esc, KeyModifiers::NONE);
        let cmd = keymap.lookup(Mode::Insert, esc_key);
        assert_eq!(cmd, Some(&EditorCommand::EnterNormalMode));
    }

    #[test]
    fn test_ctrl_bindings() {
        let keymap = KeyMap::default();

        let ctrl_p = KeyBinding::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
        let cmd = keymap.lookup(Mode::Normal, ctrl_p);
        assert_eq!(cmd, Some(&EditorCommand::OpenCommandPalette));

        let ctrl_s = KeyBinding::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        let cmd = keymap.lookup(Mode::Insert, ctrl_s);
        assert_eq!(cmd, Some(&EditorCommand::Save));
    }

    #[test]
    fn test_mode_transitions() {
        let keymap = KeyMap::default();

        let i_key = KeyBinding::new(KeyCode::Char('i'), KeyModifiers::NONE);
        let cmd = keymap.lookup(Mode::Normal, i_key);
        assert_eq!(cmd, Some(&EditorCommand::EnterInsertMode));

        let v_key = KeyBinding::new(KeyCode::Char('v'), KeyModifiers::NONE);
        let cmd = keymap.lookup(Mode::Normal, v_key);
        assert_eq!(cmd, Some(&EditorCommand::EnterVisualMode));
    }

    #[test]
    fn test_add_custom_binding() {
        let mut keymap = KeyMap::default();

        let custom_key = KeyBinding::new(KeyCode::Char('z'), KeyModifiers::CONTROL);
        keymap.add_binding(Mode::Normal, custom_key.clone(), EditorCommand::Undo);

        let cmd = keymap.lookup(Mode::Normal, custom_key);
        assert_eq!(cmd, Some(&EditorCommand::Undo));
    }

    #[test]
    fn test_mode_display() {
        assert_eq!(Mode::Normal.as_str(), "NORMAL");
        assert_eq!(Mode::Insert.as_str(), "INSERT");
        assert_eq!(Mode::Visual.as_str(), "VISUAL");
        assert_eq!(Mode::Command.as_str(), "COMMAND");
    }
}
