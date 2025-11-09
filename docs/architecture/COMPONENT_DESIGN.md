# AIT42 Editor - Component Design Specification

**Version**: 1.0.0
**Date**: 2025-01-06
**Status**: Design Phase

---

## Table of Contents

1. [Component Overview](#component-overview)
2. [Core Editor Components](#core-editor-components)
3. [TUI Components](#tui-components)
4. [LSP Components](#lsp-components)
5. [AIT42 Integration Components](#ait42-integration-components)
6. [File System Components](#file-system-components)
7. [Configuration Components](#configuration-components)
8. [Component Interfaces](#component-interfaces)
9. [State Management](#state-management)
10. [Testing Strategy](#testing-strategy)

---

## Component Overview

### Dependency Graph

```
ait42-bin (main binary)
    │
    ├─► ait42-core (core editor logic)
    │   └─► Dependencies: ropey, tree-sitter
    │
    ├─► ait42-tui (TUI rendering)
    │   ├─► ait42-core
    │   └─► Dependencies: ratatui, crossterm
    │
    ├─► ait42-lsp (LSP client)
    │   ├─► ait42-core
    │   └─► Dependencies: tower-lsp, tokio
    │
    ├─► ait42-ait42 (AIT42 integration)
    │   ├─► ait42-core
    │   └─► Dependencies: serde_yaml, tokio
    │
    ├─► ait42-fs (file system)
    │   ├─► ait42-core
    │   └─► Dependencies: notify, ignore
    │
    └─► ait42-config (configuration)
        └─► Dependencies: serde, toml
```

### Component Communication

```
┌──────────────────────────────────────────────────────────┐
│                     Event Bus (mpsc)                     │
│              Centralized Message Passing                 │
└────┬─────────┬─────────┬─────────┬─────────┬────────────┘
     │         │         │         │         │
┌────▼────┐ ┌─▼─────┐ ┌─▼─────┐ ┌─▼──────┐ ┌▼───────────┐
│  Core   │ │  TUI  │ │  LSP  │ │ AIT42  │ │    FS      │
│ Editor  │ │       │ │       │ │        │ │            │
└────┬────┘ └───┬───┘ └───┬───┘ └───┬────┘ └─────┬──────┘
     │          │         │         │            │
     └──────────┴─────────┴─────────┴────────────┘
                       │
                 Shared State
               (EditorContext)
```

---

## Core Editor Components

### 1. Text Buffer (`ait42-core/buffer`)

#### 1.1 `TextBuffer`

**Responsibility**: Immutable text storage and manipulation

```rust
use ropey::Rope;

pub struct TextBuffer {
    /// Rope-based text storage (efficient for large files)
    rope: Rope,

    /// File path (None for unsaved buffers)
    path: Option<PathBuf>,

    /// Language ID for syntax highlighting
    language: Language,

    /// Modification state
    dirty: bool,

    /// Line ending style (LF, CRLF, CR)
    line_ending: LineEnding,

    /// File encoding (UTF-8, UTF-16, etc.)
    encoding: Encoding,
}

impl TextBuffer {
    /// Create new empty buffer
    pub fn new(language: Language) -> Self;

    /// Load from file
    pub fn from_file(path: &Path) -> Result<Self>;

    /// Insert text at position
    pub fn insert(&mut self, pos: usize, text: &str) -> Result<()>;

    /// Delete range
    pub fn delete(&mut self, range: Range<usize>) -> Result<()>;

    /// Get line by index
    pub fn line(&self, index: usize) -> Option<&str>;

    /// Get char count
    pub fn len_chars(&self) -> usize;

    /// Get line count
    pub fn len_lines(&self) -> usize;

    /// Get byte offset from line/column
    pub fn offset(&self, line: usize, col: usize) -> Option<usize>;

    /// Get line/column from byte offset
    pub fn position(&self, offset: usize) -> (usize, usize);

    /// Get slice of text
    pub fn slice(&self, range: Range<usize>) -> String;

    /// Mark as clean (after save)
    pub fn mark_clean(&mut self);

    /// Check if modified
    pub fn is_dirty(&self) -> bool;
}
```

**Performance**:
- Insert: O(log n)
- Delete: O(log n)
- Line access: O(log n)
- Position conversion: O(log n)

---

#### 1.2 `BufferManager`

**Responsibility**: Manage multiple open buffers

```rust
pub struct BufferManager {
    /// Map of buffer ID to buffer
    buffers: HashMap<BufferId, TextBuffer>,

    /// Currently active buffer
    active_buffer: Option<BufferId>,

    /// Buffer ID counter
    next_id: BufferId,

    /// LRU cache for recently used buffers
    lru: LruCache<BufferId, ()>,
}

impl BufferManager {
    /// Create new buffer
    pub fn new_buffer(&mut self, language: Language) -> BufferId;

    /// Open file as buffer
    pub fn open_file(&mut self, path: &Path) -> Result<BufferId>;

    /// Get buffer by ID
    pub fn get(&self, id: BufferId) -> Option<&TextBuffer>;

    /// Get mutable buffer by ID
    pub fn get_mut(&mut self, id: BufferId) -> Option<&mut TextBuffer>;

    /// Close buffer
    pub fn close(&mut self, id: BufferId) -> Result<()>;

    /// Save buffer to file
    pub fn save(&mut self, id: BufferId) -> Result<()>;

    /// Save buffer to new path
    pub fn save_as(&mut self, id: BufferId, path: &Path) -> Result<()>;

    /// Switch active buffer
    pub fn switch_to(&mut self, id: BufferId) -> Result<()>;

    /// Get active buffer ID
    pub fn active_buffer_id(&self) -> Option<BufferId>;

    /// List all buffer IDs
    pub fn buffer_ids(&self) -> Vec<BufferId>;

    /// Get dirty buffers (unsaved changes)
    pub fn dirty_buffers(&self) -> Vec<BufferId>;
}
```

---

#### 1.3 `UndoTree`

**Responsibility**: Undo/redo history with tree structure

```rust
pub struct UndoTree {
    /// Tree of edit history
    nodes: Vec<UndoNode>,

    /// Current position in tree
    current: NodeId,

    /// Root node
    root: NodeId,
}

struct UndoNode {
    /// Parent node (None for root)
    parent: Option<NodeId>,

    /// Child branches
    children: Vec<NodeId>,

    /// Edit operation
    operation: EditOperation,

    /// Timestamp
    timestamp: Instant,
}

enum EditOperation {
    Insert { pos: usize, text: String },
    Delete { pos: usize, text: String },
    Replace { pos: usize, old: String, new: String },
    Composite(Vec<EditOperation>),  // For grouped edits
}

impl UndoTree {
    /// Create new undo tree
    pub fn new() -> Self;

    /// Record edit operation
    pub fn push(&mut self, op: EditOperation);

    /// Undo current operation
    pub fn undo(&mut self) -> Option<&EditOperation>;

    /// Redo operation
    pub fn redo(&mut self) -> Option<&EditOperation>;

    /// Get undo history (linear path from current to root)
    pub fn history(&self) -> Vec<&EditOperation>;

    /// Can undo?
    pub fn can_undo(&self) -> bool;

    /// Can redo?
    pub fn can_redo(&self) -> bool;
}
```

**Benefits**:
- Tree structure allows branching undo (Phase 2)
- Efficient for large edit sequences
- Timestamp-based time-travel (Phase 2)

---

### 2. Cursor Management (`ait42-core/cursor`)

#### 2.1 `Cursor`

**Responsibility**: Single cursor position and selection

```rust
#[derive(Clone, Copy, Debug)]
pub struct Cursor {
    /// Cursor position (byte offset)
    pos: usize,

    /// Anchor for selection (None = no selection)
    anchor: Option<usize>,

    /// Preferred column (for vertical movement)
    preferred_col: Option<usize>,
}

impl Cursor {
    /// Create new cursor at position
    pub fn new(pos: usize) -> Self;

    /// Move cursor
    pub fn move_to(&mut self, pos: usize);

    /// Start selection
    pub fn start_selection(&mut self);

    /// End selection
    pub fn end_selection(&mut self) -> Option<Range<usize>>;

    /// Get selection range (if any)
    pub fn selection(&self) -> Option<Range<usize>>;

    /// Has selection?
    pub fn has_selection(&self) -> bool;

    /// Move left by chars
    pub fn move_left(&mut self, buffer: &TextBuffer, count: usize);

    /// Move right by chars
    pub fn move_right(&mut self, buffer: &TextBuffer, count: usize);

    /// Move up by lines
    pub fn move_up(&mut self, buffer: &TextBuffer, count: usize);

    /// Move down by lines
    pub fn move_down(&mut self, buffer: &TextBuffer, count: usize);

    /// Move to line start
    pub fn move_to_line_start(&mut self, buffer: &TextBuffer);

    /// Move to line end
    pub fn move_to_line_end(&mut self, buffer: &TextBuffer);
}
```

---

#### 2.2 `CursorSet` (Phase 2: Multi-cursor)

**Responsibility**: Manage multiple cursors

```rust
pub struct CursorSet {
    /// Primary cursor (always exists)
    primary: Cursor,

    /// Secondary cursors (Phase 2)
    secondary: Vec<Cursor>,
}

impl CursorSet {
    /// Create new cursor set
    pub fn new(pos: usize) -> Self;

    /// Add cursor
    pub fn add_cursor(&mut self, pos: usize);

    /// Remove cursor
    pub fn remove_cursor(&mut self, index: usize);

    /// Get all cursors
    pub fn cursors(&self) -> impl Iterator<Item = &Cursor>;

    /// Apply operation to all cursors
    pub fn apply<F>(&mut self, buffer: &TextBuffer, f: F)
    where
        F: Fn(&mut Cursor, &TextBuffer);
}
```

---

### 3. Modal Editing (`ait42-core/mode`)

#### 3.1 `Mode` Trait

**Responsibility**: Define behavior for different editing modes

```rust
pub trait Mode: Send + Sync {
    /// Handle key press
    fn handle_key(
        &self,
        key: KeyEvent,
        ctx: &mut EditorContext,
    ) -> Result<ModeTransition>;

    /// Render mode indicator
    fn indicator(&self) -> &str;

    /// On enter mode
    fn on_enter(&self, ctx: &mut EditorContext) -> Result<()> {
        Ok(())
    }

    /// On exit mode
    fn on_exit(&self, ctx: &mut EditorContext) -> Result<()> {
        Ok(())
    }
}

pub enum ModeTransition {
    /// Stay in current mode
    Stay,

    /// Switch to new mode
    Switch(Box<dyn Mode>),

    /// Exit editor
    Exit,
}
```

---

#### 3.2 `NormalMode`

**Responsibility**: Vim normal mode (navigation, commands)

```rust
pub struct NormalMode {
    /// Pending command (e.g., "d" waiting for motion)
    pending: Option<char>,

    /// Count prefix (e.g., "5j" = 5)
    count: Option<usize>,
}

impl Mode for NormalMode {
    fn handle_key(&self, key: KeyEvent, ctx: &mut EditorContext) -> Result<ModeTransition> {
        match key.code {
            // Mode switches
            KeyCode::Char('i') => Ok(ModeTransition::Switch(Box::new(InsertMode::new()))),
            KeyCode::Char('v') => Ok(ModeTransition::Switch(Box::new(VisualMode::new()))),
            KeyCode::Char(':') => Ok(ModeTransition::Switch(Box::new(CommandMode::new()))),

            // Navigation
            KeyCode::Char('h') => {
                ctx.cursor_mut().move_left(ctx.active_buffer(), self.count.unwrap_or(1));
                Ok(ModeTransition::Stay)
            }
            KeyCode::Char('j') => {
                ctx.cursor_mut().move_down(ctx.active_buffer(), self.count.unwrap_or(1));
                Ok(ModeTransition::Stay)
            }
            KeyCode::Char('k') => {
                ctx.cursor_mut().move_up(ctx.active_buffer(), self.count.unwrap_or(1));
                Ok(ModeTransition::Stay)
            }
            KeyCode::Char('l') => {
                ctx.cursor_mut().move_right(ctx.active_buffer(), self.count.unwrap_or(1));
                Ok(ModeTransition::Stay)
            }

            // Commands
            KeyCode::Char('d') => {
                if self.pending == Some('d') {
                    // dd = delete line
                    ctx.delete_line()?;
                    Ok(ModeTransition::Stay)
                } else {
                    // Wait for motion
                    self.pending = Some('d');
                    Ok(ModeTransition::Stay)
                }
            }

            // Numbers (count prefix)
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let digit = c.to_digit(10).unwrap() as usize;
                self.count = Some(self.count.unwrap_or(0) * 10 + digit);
                Ok(ModeTransition::Stay)
            }

            _ => Ok(ModeTransition::Stay),
        }
    }

    fn indicator(&self) -> &str {
        "NORMAL"
    }
}
```

---

#### 3.3 `InsertMode`

**Responsibility**: Text insertion

```rust
pub struct InsertMode;

impl Mode for InsertMode {
    fn handle_key(&self, key: KeyEvent, ctx: &mut EditorContext) -> Result<ModeTransition> {
        match key.code {
            KeyCode::Esc => Ok(ModeTransition::Switch(Box::new(NormalMode::new()))),

            KeyCode::Char(c) => {
                ctx.insert_char(c)?;
                Ok(ModeTransition::Stay)
            }

            KeyCode::Backspace => {
                ctx.delete_backward()?;
                Ok(ModeTransition::Stay)
            }

            KeyCode::Enter => {
                ctx.insert_newline()?;
                Ok(ModeTransition::Stay)
            }

            _ => Ok(ModeTransition::Stay),
        }
    }

    fn indicator(&self) -> &str {
        "INSERT"
    }
}
```

---

#### 3.4 `CommandMode`

**Responsibility**: Execute commands (`:w`, `:q`, etc.)

```rust
pub struct CommandMode {
    /// Current command input
    input: String,

    /// Command history
    history: Vec<String>,

    /// History cursor
    history_index: usize,
}

impl Mode for CommandMode {
    fn handle_key(&self, key: KeyEvent, ctx: &mut EditorContext) -> Result<ModeTransition> {
        match key.code {
            KeyCode::Esc => Ok(ModeTransition::Switch(Box::new(NormalMode::new()))),

            KeyCode::Enter => {
                self.execute_command(&self.input, ctx)?;
                self.history.push(self.input.clone());
                Ok(ModeTransition::Switch(Box::new(NormalMode::new())))
            }

            KeyCode::Char(c) => {
                self.input.push(c);
                Ok(ModeTransition::Stay)
            }

            KeyCode::Backspace => {
                self.input.pop();
                Ok(ModeTransition::Stay)
            }

            _ => Ok(ModeTransition::Stay),
        }
    }

    fn indicator(&self) -> &str {
        &format!(":{}", self.input)
    }
}

impl CommandMode {
    fn execute_command(&self, cmd: &str, ctx: &mut EditorContext) -> Result<()> {
        match cmd {
            "w" | "write" => ctx.save_buffer()?,
            "q" | "quit" => ctx.quit()?,
            "wq" => {
                ctx.save_buffer()?;
                ctx.quit()?;
            }
            "q!" => ctx.force_quit()?,
            _ => {
                // Try custom command
                if let Some(agent_cmd) = cmd.strip_prefix("agent ") {
                    ctx.execute_agent(agent_cmd)?;
                } else {
                    return Err(Error::UnknownCommand(cmd.to_string()));
                }
            }
        }
        Ok(())
    }
}
```

---

### 4. Command System (`ait42-core/command`)

#### 4.1 `Command` Trait

**Responsibility**: Executable and undoable operations

```rust
pub trait Command: Send + Sync {
    /// Execute command
    fn execute(&self, ctx: &mut EditorContext) -> Result<()>;

    /// Undo command
    fn undo(&self, ctx: &mut EditorContext) -> Result<()>;

    /// Get command description
    fn description(&self) -> &str;
}
```

---

#### 4.2 Built-in Commands

```rust
/// Insert text command
pub struct InsertTextCommand {
    buffer_id: BufferId,
    pos: usize,
    text: String,
}

impl Command for InsertTextCommand {
    fn execute(&self, ctx: &mut EditorContext) -> Result<()> {
        ctx.buffers.get_mut(self.buffer_id)?
            .insert(self.pos, &self.text)?;
        Ok(())
    }

    fn undo(&self, ctx: &mut EditorContext) -> Result<()> {
        ctx.buffers.get_mut(self.buffer_id)?
            .delete(self.pos..self.pos + self.text.len())?;
        Ok(())
    }

    fn description(&self) -> &str {
        "Insert text"
    }
}

/// Delete text command
pub struct DeleteTextCommand {
    buffer_id: BufferId,
    range: Range<usize>,
    deleted_text: String,  // For undo
}

impl Command for DeleteTextCommand {
    fn execute(&self, ctx: &mut EditorContext) -> Result<()> {
        let buffer = ctx.buffers.get_mut(self.buffer_id)?;
        self.deleted_text = buffer.slice(self.range.clone());
        buffer.delete(self.range.clone())?;
        Ok(())
    }

    fn undo(&self, ctx: &mut EditorContext) -> Result<()> {
        ctx.buffers.get_mut(self.buffer_id)?
            .insert(self.range.start, &self.deleted_text)?;
        Ok(())
    }

    fn description(&self) -> &str {
        "Delete text"
    }
}

/// Save file command
pub struct SaveFileCommand {
    buffer_id: BufferId,
}

impl Command for SaveFileCommand {
    fn execute(&self, ctx: &mut EditorContext) -> Result<()> {
        ctx.buffers.save(self.buffer_id)?;
        ctx.send_event(EditorEvent::FileSaved(self.buffer_id));
        Ok(())
    }

    fn undo(&self, _ctx: &mut EditorContext) -> Result<()> {
        // Cannot undo save
        Err(Error::CannotUndo("File save".into()))
    }

    fn description(&self) -> &str {
        "Save file"
    }
}
```

---

## TUI Components

### 1. Editor Widget (`ait42-tui/widgets/editor.rs`)

**Responsibility**: Main text editing area rendering

```rust
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

pub struct EditorWidget<'a> {
    buffer: &'a TextBuffer,
    cursor: &'a Cursor,
    theme: &'a ColorScheme,
    line_numbers: bool,
    syntax_highlights: Option<&'a Vec<HighlightRange>>,
}

impl<'a> EditorWidget<'a> {
    pub fn new(
        buffer: &'a TextBuffer,
        cursor: &'a Cursor,
        theme: &'a ColorScheme,
    ) -> Self {
        Self {
            buffer,
            cursor,
            theme,
            line_numbers: true,
            syntax_highlights: None,
        }
    }

    pub fn line_numbers(mut self, enabled: bool) -> Self {
        self.line_numbers = enabled;
        self
    }

    pub fn syntax_highlights(mut self, highlights: &'a Vec<HighlightRange>) -> Self {
        self.syntax_highlights = Some(highlights);
        self
    }

    fn render_line(&self, line_idx: usize, line_text: &str, area: Rect) -> Line<'a> {
        let mut spans = Vec::new();

        // Line number
        if self.line_numbers {
            spans.push(Span::styled(
                format!("{:>4} ", line_idx + 1),
                Style::default().fg(Color::DarkGray),
            ));
        }

        // Syntax highlighting
        if let Some(highlights) = self.syntax_highlights {
            let line_highlights: Vec<_> = highlights.iter()
                .filter(|h| h.line == line_idx)
                .collect();

            if line_highlights.is_empty() {
                // No highlights, plain text
                spans.push(Span::raw(line_text));
            } else {
                // Apply highlights
                let mut last_end = 0;
                for highlight in line_highlights {
                    // Plain text before highlight
                    if highlight.start > last_end {
                        spans.push(Span::raw(&line_text[last_end..highlight.start]));
                    }

                    // Highlighted text
                    spans.push(Span::styled(
                        &line_text[highlight.start..highlight.end],
                        self.theme.style_for(highlight.kind),
                    ));

                    last_end = highlight.end;
                }

                // Remaining text
                if last_end < line_text.len() {
                    spans.push(Span::raw(&line_text[last_end..]));
                }
            }
        } else {
            spans.push(Span::raw(line_text));
        }

        Line::from(spans)
    }
}

impl<'a> Widget for EditorWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::default()
            .borders(Borders::NONE);

        let inner_area = block.inner(area);

        // Calculate visible range
        let first_line = 0;  // TODO: Scroll offset
        let visible_lines = inner_area.height as usize;

        let lines: Vec<Line> = (first_line..first_line + visible_lines)
            .filter_map(|i| {
                self.buffer.line(i).map(|text| self.render_line(i, text, inner_area))
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block);

        paragraph.render(area, buf);
    }
}
```

---

### 2. Status Bar (`ait42-tui/widgets/status_bar.rs`)

**Responsibility**: Display mode, cursor position, file info

```rust
pub struct StatusBar<'a> {
    mode: &'a str,
    file_name: Option<&'a str>,
    cursor_pos: (usize, usize),  // (line, col)
    file_type: &'a str,
    dirty: bool,
}

impl<'a> StatusBar<'a> {
    pub fn new(
        mode: &'a str,
        file_name: Option<&'a str>,
        cursor_pos: (usize, usize),
        file_type: &'a str,
        dirty: bool,
    ) -> Self {
        Self {
            mode,
            file_name,
            cursor_pos,
            file_type,
            dirty,
        }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let left_text = format!(
            " {} | {} {}",
            self.mode,
            self.file_name.unwrap_or("[No Name]"),
            if self.dirty { "[+]" } else { "" }
        );

        let right_text = format!(
            "{}:{}  {} ",
            self.cursor_pos.0,
            self.cursor_pos.1,
            self.file_type
        );

        // Render left-aligned text
        let left_span = Span::styled(
            left_text,
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

        // Render right-aligned text
        let right_span = Span::styled(
            right_text,
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White),
        );

        // Calculate padding
        let padding_len = area.width
            .saturating_sub(left_span.content.len() as u16)
            .saturating_sub(right_span.content.len() as u16);

        let line = Line::from(vec![
            left_span,
            Span::raw(" ".repeat(padding_len as usize)),
            right_span,
        ]);

        line.render(area, buf);
    }
}
```

---

### 3. Command Palette (`ait42-tui/widgets/command_palette.rs`)

**Responsibility**: Agent selection, file search

```rust
pub struct CommandPalette<'a> {
    items: Vec<PaletteItem>,
    selected: usize,
    filter: &'a str,
    theme: &'a ColorScheme,
}

pub struct PaletteItem {
    pub title: String,
    pub description: String,
    pub kind: PaletteItemKind,
}

pub enum PaletteItemKind {
    Agent(String),      // Agent name
    File(PathBuf),      // File path
    Command(String),    // Editor command
}

impl<'a> CommandPalette<'a> {
    pub fn new(items: Vec<PaletteItem>, filter: &'a str, theme: &'a ColorScheme) -> Self {
        Self {
            items,
            selected: 0,
            filter,
            theme,
        }
    }

    fn filtered_items(&self) -> Vec<&PaletteItem> {
        if self.filter.is_empty() {
            self.items.iter().collect()
        } else {
            self.items.iter()
                .filter(|item| {
                    item.title.to_lowercase().contains(&self.filter.to_lowercase())
                        || item.description.to_lowercase().contains(&self.filter.to_lowercase())
                })
                .collect()
        }
    }
}

impl<'a> Widget for CommandPalette<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::default()
            .title(" Select Agent / File ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);

        // Filter input at top
        let filter_line = Line::from(vec![
            Span::raw("> "),
            Span::styled(
                self.filter,
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]);

        // Render items
        let filtered_items = self.filtered_items();
        let items: Vec<Line> = filtered_items.iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected {
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let icon = match item.kind {
                    PaletteItemKind::Agent(_) => "🤖",
                    PaletteItemKind::File(_) => "📄",
                    PaletteItemKind::Command(_) => "⚙️",
                };

                Line::from(vec![
                    Span::raw(format!("{} ", icon)),
                    Span::styled(format!("{:<30}", item.title), style),
                    Span::styled(item.description.clone(), Style::default().fg(Color::DarkGray)),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(items)
            .block(block);

        paragraph.render(area, buf);
    }
}
```

---

## LSP Components

### 1. LSP Client (`ait42-lsp/client.rs`)

**Responsibility**: Async communication with LSP servers

```rust
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tokio::sync::mpsc;

pub struct LspClient {
    /// Map of language ID to server handle
    servers: HashMap<String, ServerHandle>,

    /// Event sender to editor
    event_tx: mpsc::Sender<EditorEvent>,
}

struct ServerHandle {
    /// Language ID (e.g., "rust", "typescript")
    language: String,

    /// LSP client (tower-lsp)
    client: Box<dyn LanguageServer>,

    /// Server process handle
    process: Child,
}

impl LspClient {
    pub async fn new(event_tx: mpsc::Sender<EditorEvent>) -> Result<Self> {
        Ok(Self {
            servers: HashMap::new(),
            event_tx,
        })
    }

    /// Start LSP server for language
    pub async fn start_server(&mut self, language: &str, config: &LspConfig) -> Result<()> {
        let command = config.command_for(language)?;

        // Spawn LSP server process
        let process = Command::new(&command.binary)
            .args(&command.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Create tower-lsp client
        let (client, _server) = tower_lsp::LspService::new(|client| {
            EditorLspService {
                client,
                event_tx: self.event_tx.clone(),
            }
        });

        let handle = ServerHandle {
            language: language.to_string(),
            client: Box::new(client),
            process,
        };

        self.servers.insert(language.to_string(), handle);

        // Initialize server
        self.initialize_server(language).await?;

        Ok(())
    }

    async fn initialize_server(&mut self, language: &str) -> Result<()> {
        let handle = self.servers.get(language)?;

        let params = InitializeParams {
            capabilities: ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    completion: Some(CompletionClientCapabilities {
                        dynamic_registration: Some(true),
                        completion_item: Some(CompletionItemCapability {
                            snippet_support: Some(true),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    hover: Some(HoverClientCapabilities {
                        dynamic_registration: Some(true),
                        content_format: Some(vec![MarkupKind::Markdown]),
                    }),
                    definition: Some(GotoCapability {
                        dynamic_registration: Some(true),
                        link_support: Some(true),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        handle.client.initialize(params).await?;
        handle.client.initialized(InitializedParams {}).await;

        Ok(())
    }

    /// Request completion
    pub async fn completion(
        &self,
        buffer: &TextBuffer,
        pos: Position,
    ) -> Result<Vec<CompletionItem>> {
        let language = buffer.language.to_string();
        let handle = self.servers.get(&language)?;

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: buffer.path.as_ref().unwrap().to_uri(),
                },
                position: pos,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        };

        let response = handle.client.completion(params).await?;

        match response {
            Some(CompletionResponse::Array(items)) => Ok(items),
            Some(CompletionResponse::List(list)) => Ok(list.items),
            None => Ok(Vec::new()),
        }
    }

    /// Notify text document opened
    pub async fn did_open(&self, buffer: &TextBuffer) -> Result<()> {
        let language = buffer.language.to_string();
        let handle = self.servers.get(&language)?;

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: buffer.path.as_ref().unwrap().to_uri(),
                language_id: language,
                version: 0,
                text: buffer.rope.to_string(),
            },
        };

        handle.client.did_open(params).await;
        Ok(())
    }

    /// Notify text document changed
    pub async fn did_change(&self, buffer: &TextBuffer, changes: Vec<TextDocumentContentChangeEvent>) -> Result<()> {
        let language = buffer.language.to_string();
        let handle = self.servers.get(&language)?;

        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: buffer.path.as_ref().unwrap().to_uri(),
                version: buffer.version,
            },
            content_changes: changes,
        };

        handle.client.did_change(params).await;
        Ok(())
    }
}
```

---

### 2. Completion Handler (`ait42-lsp/handlers/completion.rs`)

**Responsibility**: Handle completion requests and display UI

```rust
pub struct CompletionHandler {
    /// Current completion items
    items: Vec<CompletionItem>,

    /// Selected item index
    selected: usize,

    /// Trigger position
    trigger_pos: Position,

    /// Is completion popup visible?
    visible: bool,
}

impl CompletionHandler {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            trigger_pos: Position::new(0, 0),
            visible: false,
        }
    }

    pub async fn request_completion(
        &mut self,
        lsp: &LspClient,
        buffer: &TextBuffer,
        pos: Position,
    ) -> Result<()> {
        self.items = lsp.completion(buffer, pos).await?;
        self.selected = 0;
        self.trigger_pos = pos;
        self.visible = !self.items.is_empty();
        Ok(())
    }

    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + self.items.len() - 1) % self.items.len();
        }
    }

    pub fn accept_completion(&mut self, buffer: &mut TextBuffer) -> Result<()> {
        if let Some(item) = self.items.get(self.selected) {
            let insert_text = item.insert_text.as_ref()
                .or(item.text_edit.as_ref().map(|e| &e.new_text))
                .unwrap_or(&item.label);

            let pos = self.trigger_pos;
            buffer.insert(pos.into(), insert_text)?;

            self.hide();
        }
        Ok(())
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.items.clear();
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn items(&self) -> &[CompletionItem] {
        &self.items
    }

    pub fn selected(&self) -> usize {
        self.selected
    }
}
```

---

## AIT42 Integration Components

### 1. Agent Loader (`ait42-ait42/agent/loader.rs`)

**Responsibility**: Load agent metadata from `.claude/agents/*.md`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentMetadata {
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
    pub model: String,
    pub prompt: String,
}

pub struct AgentLoader {
    /// Loaded agents
    agents: HashMap<String, AgentMetadata>,

    /// Agent directory path
    agent_dir: PathBuf,
}

impl AgentLoader {
    pub fn new(agent_dir: PathBuf) -> Self {
        Self {
            agents: HashMap::new(),
            agent_dir,
        }
    }

    /// Load all agents from directory
    pub fn load_all(&mut self) -> Result<()> {
        for entry in fs::read_dir(&self.agent_dir)? {
            let path = entry?.path();
            if path.extension() == Some(OsStr::new("md")) {
                let metadata = self.parse_agent_file(&path)?;
                self.agents.insert(metadata.name.clone(), metadata);
            }
        }
        Ok(())
    }

    fn parse_agent_file(&self, path: &Path) -> Result<AgentMetadata> {
        let content = fs::read_to_string(path)?;

        // Parse YAML frontmatter
        let (frontmatter, body) = self.split_frontmatter(&content)?;

        let mut metadata: AgentMetadata = serde_yaml::from_str(&frontmatter)?;
        metadata.prompt = body.trim().to_string();

        Ok(metadata)
    }

    fn split_frontmatter(&self, content: &str) -> Result<(String, String)> {
        let lines: Vec<&str> = content.lines().collect();

        if lines.first() != Some(&"---") {
            return Err(Error::InvalidAgentFile("Missing frontmatter".into()));
        }

        let end_index = lines[1..]
            .iter()
            .position(|&line| line == "---")
            .ok_or_else(|| Error::InvalidAgentFile("Unclosed frontmatter".into()))?
            + 1;

        let frontmatter = lines[1..end_index].join("\n");
        let body = lines[end_index + 1..].join("\n");

        Ok((frontmatter, body))
    }

    pub fn get(&self, name: &str) -> Option<&AgentMetadata> {
        self.agents.get(name)
    }

    pub fn all(&self) -> impl Iterator<Item = &AgentMetadata> {
        self.agents.values()
    }

    pub fn search(&self, query: &str) -> Vec<&AgentMetadata> {
        self.agents.values()
            .filter(|agent| {
                agent.name.to_lowercase().contains(&query.to_lowercase())
                    || agent.description.to_lowercase().contains(&query.to_lowercase())
            })
            .collect()
    }
}
```

---

### 2. Tmux Session Manager (`ait42-ait42/tmux/session_manager.rs`)

**Responsibility**: Create, attach, destroy tmux sessions for agents

```rust
use tokio::process::Command;

#[derive(Clone, Debug)]
pub struct TmuxSession {
    pub name: String,
    pub agent_id: AgentId,
    pub created_at: Instant,
    pub status: SessionStatus,
}

#[derive(Clone, Debug)]
pub enum SessionStatus {
    Creating,
    Running,
    Completed,
    Failed(String),
}

pub struct TmuxSessionManager {
    sessions: HashMap<AgentId, TmuxSession>,
    next_id: AgentId,
}

impl TmuxSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            next_id: 0,
        }
    }

    /// Create new tmux session for agent
    pub async fn create_session(&mut self, agent_name: &str) -> Result<TmuxSession> {
        let agent_id = self.next_id;
        self.next_id += 1;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let session_name = format!("ait42-{}-{}", agent_name, timestamp);

        let output = Command::new("tmux")
            .arg("new-session")
            .arg("-d")  // Detached
            .arg("-s").arg(&session_name)
            .arg("-x").arg("200")  // Width
            .arg("-y").arg("50")   // Height
            .output()
            .await?;

        if !output.status.success() {
            return Err(Error::TmuxError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let session = TmuxSession {
            name: session_name,
            agent_id,
            created_at: Instant::now(),
            status: SessionStatus::Running,
        };

        self.sessions.insert(agent_id, session.clone());
        Ok(session)
    }

    /// Execute command in tmux session
    pub async fn execute_in_session(
        &self,
        session: &TmuxSession,
        command: &str,
    ) -> Result<()> {
        let output = Command::new("tmux")
            .arg("send-keys")
            .arg("-t").arg(&session.name)
            .arg(command)
            .arg("C-m")  // Enter key
            .output()
            .await?;

        if !output.status.success() {
            return Err(Error::TmuxError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(())
    }

    /// Capture output from tmux session
    pub async fn capture_output(&self, session: &TmuxSession) -> Result<String> {
        let output = Command::new("tmux")
            .arg("capture-pane")
            .arg("-t").arg(&session.name)
            .arg("-p")  // Print to stdout
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Destroy tmux session
    pub async fn destroy_session(&mut self, agent_id: AgentId) -> Result<()> {
        if let Some(session) = self.sessions.remove(&agent_id) {
            let output = Command::new("tmux")
                .arg("kill-session")
                .arg("-t").arg(&session.name)
                .output()
                .await?;

            if !output.status.success() {
                // Session might already be dead, that's ok
                eprintln!("Warning: Could not kill session: {}", session.name);
            }
        }
        Ok(())
    }

    /// List all active sessions
    pub fn active_sessions(&self) -> Vec<&TmuxSession> {
        self.sessions.values()
            .filter(|s| matches!(s.status, SessionStatus::Running))
            .collect()
    }

    /// Get session by agent ID
    pub fn get_session(&self, agent_id: AgentId) -> Option<&TmuxSession> {
        self.sessions.get(&agent_id)
    }
}
```

---

## State Management

### Global Editor State

```rust
pub struct EditorContext {
    /// Buffer manager
    pub buffers: BufferManager,

    /// Currently active mode
    pub mode: Box<dyn Mode>,

    /// Cursor set (multi-cursor support)
    pub cursors: CursorSet,

    /// Undo history per buffer
    pub undo_trees: HashMap<BufferId, UndoTree>,

    /// LSP client
    pub lsp: LspClient,

    /// AIT42 agent registry
    pub agents: AgentLoader,

    /// Tmux session manager
    pub tmux: TmuxSessionManager,

    /// Configuration
    pub config: Config,

    /// Event bus
    pub event_tx: mpsc::Sender<EditorEvent>,

    /// UI state
    pub ui_state: UiState,
}

pub struct UiState {
    /// Command palette open?
    pub command_palette_open: bool,

    /// Agent palette open?
    pub agent_palette_open: bool,

    /// File tree visible?
    pub file_tree_visible: bool,

    /// Tmux panel visible?
    pub tmux_panel_visible: bool,

    /// Current theme
    pub theme: ColorScheme,
}

impl EditorContext {
    pub fn new(config: Config) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel(100);

        Ok(Self {
            buffers: BufferManager::new(),
            mode: Box::new(NormalMode::new()),
            cursors: CursorSet::new(0),
            undo_trees: HashMap::new(),
            lsp: LspClient::new(event_tx.clone())?,
            agents: AgentLoader::new(Path::new(".claude/agents").to_path_buf()),
            tmux: TmuxSessionManager::new(),
            config,
            event_tx,
            ui_state: UiState::default(),
        })
    }

    pub fn active_buffer(&self) -> Option<&TextBuffer> {
        self.buffers.active_buffer_id()
            .and_then(|id| self.buffers.get(id))
    }

    pub fn active_buffer_mut(&mut self) -> Option<&mut TextBuffer> {
        self.buffers.active_buffer_id()
            .and_then(move |id| self.buffers.get_mut(id))
    }
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_insert() {
        let mut buffer = TextBuffer::new(Language::Rust);
        buffer.insert(0, "Hello").unwrap();
        assert_eq!(buffer.slice(0..5), "Hello");
    }

    #[test]
    fn test_buffer_delete() {
        let mut buffer = TextBuffer::new(Language::Rust);
        buffer.insert(0, "Hello World").unwrap();
        buffer.delete(5..11).unwrap();
        assert_eq!(buffer.slice(0..5), "Hello");
    }

    #[test]
    fn test_undo_redo() {
        let mut buffer = TextBuffer::new(Language::Rust);
        let mut undo_tree = UndoTree::new();

        buffer.insert(0, "Hello").unwrap();
        undo_tree.push(EditOperation::Insert {
            pos: 0,
            text: "Hello".to_string(),
        });

        // Undo
        if let Some(op) = undo_tree.undo() {
            // Apply inverse operation
            buffer.delete(0..5).unwrap();
        }

        assert_eq!(buffer.len_chars(), 0);

        // Redo
        if let Some(op) = undo_tree.redo() {
            buffer.insert(0, "Hello").unwrap();
        }

        assert_eq!(buffer.slice(0..5), "Hello");
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_lsp_completion() {
        let mut ctx = EditorContext::new(Config::default()).unwrap();

        // Open Rust file
        let buffer_id = ctx.buffers.open_file("test.rs").await.unwrap();

        // Start LSP server
        ctx.lsp.start_server("rust", &ctx.config.lsp).await.unwrap();

        // Request completion
        let buffer = ctx.buffers.get(buffer_id).unwrap();
        let items = ctx.lsp.completion(buffer, Position::new(0, 5)).await.unwrap();

        assert!(!items.is_empty());
    }

    #[tokio::test]
    async fn test_agent_execution() {
        let mut ctx = EditorContext::new(Config::default()).unwrap();

        // Load agents
        ctx.agents.load_all().unwrap();

        // Get backend-developer agent
        let agent = ctx.agents.get("backend-developer").unwrap();

        // Create tmux session
        let session = ctx.tmux.create_session(&agent.name).await.unwrap();

        // Execute command
        ctx.tmux.execute_in_session(&session, "echo 'Hello from agent'").await.unwrap();

        // Wait a bit
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Capture output
        let output = ctx.tmux.capture_output(&session).await.unwrap();

        assert!(output.contains("Hello from agent"));

        // Cleanup
        ctx.tmux.destroy_session(session.agent_id).await.unwrap();
    }
}
```

---

**End of Component Design Document**

Generated by: system-architect agent
Date: 2025-01-06
Version: 1.0.0
