# AIT42 Editor - API Specification

**Version**: 1.0.0
**Date**: 2025-01-06
**Status**: Design Phase

---

## Table of Contents

1. [Overview](#overview)
2. [Core Editor API](#core-editor-api)
3. [Text Buffer API](#text-buffer-api)
4. [Cursor Management API](#cursor-management-api)
5. [Mode System API](#mode-system-api)
6. [Command System API](#command-system-api)
7. [LSP Client API](#lsp-client-api)
8. [AIT42 Integration API](#ait42-integration-api)
9. [TUI Rendering API](#tui-rendering-api)
10. [Configuration API](#configuration-api)
11. [Error Handling](#error-handling)
12. [Usage Examples](#usage-examples)

---

## Overview

This document defines the complete API surface for AIT42 Editor components. All APIs follow Rust conventions and prioritize:

- **Type Safety**: Leverage Rust's type system for compile-time guarantees
- **Async/Await**: Non-blocking operations for LSP, file I/O, and agent execution
- **Error Handling**: Comprehensive `Result<T, Error>` types
- **Documentation**: Rustdoc-compatible comments for all public APIs

### API Design Principles

1. **Minimal Public Surface**: Only expose what's necessary
2. **Zero-Cost Abstractions**: No runtime overhead
3. **Thread Safety**: All public APIs are `Send + Sync` where appropriate
4. **Testability**: Easy to mock and test in isolation

---

## Core Editor API

### `EditorContext`

Central state container for the entire editor.

```rust
/// Global editor context containing all subsystems
///
/// This is the primary interface for interacting with the editor.
/// It owns all major components and provides a unified API.
///
/// # Thread Safety
/// `EditorContext` is `!Send + !Sync` as it contains terminal state.
/// Use message passing via `event_tx` for cross-thread communication.
pub struct EditorContext {
    /// Buffer manager for all open files
    pub buffers: BufferManager,

    /// Current editing mode
    mode: Box<dyn Mode>,

    /// Cursor positions
    cursors: CursorSet,

    /// Undo/redo history per buffer
    undo_trees: HashMap<BufferId, UndoTree>,

    /// LSP client for code intelligence
    lsp: LspClient,

    /// AIT42 agent registry
    agents: AgentRegistry,

    /// Tmux session manager
    tmux: TmuxSessionManager,

    /// User configuration
    config: Config,

    /// Event bus sender
    event_tx: mpsc::Sender<EditorEvent>,

    /// UI state (palettes, panels, etc.)
    ui_state: UiState,
}

impl EditorContext {
    /// Create new editor context with default configuration
    ///
    /// # Errors
    /// Returns error if:
    /// - Configuration file is invalid
    /// - Agent directory is not found
    /// - LSP servers fail to initialize
    ///
    /// # Examples
    /// ```rust
    /// let ctx = EditorContext::new(Config::default())?;
    /// ```
    pub fn new(config: Config) -> Result<Self>;

    /// Run the editor event loop
    ///
    /// This is the main entry point. It blocks until the editor exits.
    ///
    /// # Errors
    /// Returns error on fatal failures (e.g., terminal initialization failure)
    ///
    /// # Examples
    /// ```rust
    /// let mut ctx = EditorContext::new(Config::default())?;
    /// ctx.run().await?;
    /// ```
    pub async fn run(&mut self) -> Result<()>;

    /// Get currently active buffer
    ///
    /// Returns `None` if no buffer is open.
    pub fn active_buffer(&self) -> Option<&TextBuffer>;

    /// Get mutable reference to active buffer
    pub fn active_buffer_mut(&mut self) -> Option<&mut TextBuffer>;

    /// Get current cursor position
    pub fn cursor(&self) -> &Cursor;

    /// Get mutable cursor
    pub fn cursor_mut(&mut self) -> &mut Cursor;

    /// Switch to different mode
    ///
    /// Triggers `on_exit()` for current mode and `on_enter()` for new mode.
    ///
    /// # Examples
    /// ```rust
    /// ctx.switch_mode(Box::new(InsertMode::new()))?;
    /// ```
    pub fn switch_mode(&mut self, mode: Box<dyn Mode>) -> Result<()>;

    /// Execute a command
    ///
    /// Commands are added to the undo history automatically.
    ///
    /// # Examples
    /// ```rust
    /// let cmd = InsertTextCommand::new(buffer_id, pos, "text");
    /// ctx.execute_command(Box::new(cmd))?;
    /// ```
    pub fn execute_command(&mut self, cmd: Box<dyn Command>) -> Result<()>;

    /// Undo last command
    ///
    /// Returns `true` if undo was successful, `false` if nothing to undo.
    pub fn undo(&mut self) -> Result<bool>;

    /// Redo last undone command
    pub fn redo(&mut self) -> Result<bool>;

    /// Send event to event bus
    ///
    /// Non-blocking. If the event queue is full, returns error.
    pub fn send_event(&self, event: EditorEvent) -> Result<()>;

    /// Quit the editor
    ///
    /// Prompts for unsaved changes if any.
    pub fn quit(&mut self) -> Result<()>;

    /// Force quit without saving
    pub fn force_quit(&mut self) -> Result<()>;
}
```

### `EditorEvent`

Event types for inter-component communication.

```rust
/// Events that flow through the editor event bus
///
/// All components communicate via events to maintain loose coupling.
#[derive(Debug, Clone)]
pub enum EditorEvent {
    /// User pressed a key
    KeyPress(KeyEvent),

    /// Buffer content changed
    BufferChanged {
        buffer_id: BufferId,
        change: TextChange,
    },

    /// Buffer was saved
    BufferSaved(BufferId),

    /// Buffer was closed
    BufferClosed(BufferId),

    /// LSP server sent response
    LspResponse {
        language: String,
        message: LspMessage,
    },

    /// LSP diagnostics updated
    DiagnosticsUpdated {
        buffer_id: BufferId,
        diagnostics: Vec<Diagnostic>,
    },

    /// Agent execution started
    AgentStarted {
        agent_id: AgentId,
        name: String,
    },

    /// Agent execution completed
    AgentCompleted {
        agent_id: AgentId,
        result: AgentResult,
    },

    /// Agent execution failed
    AgentFailed {
        agent_id: AgentId,
        error: String,
    },

    /// File changed externally
    FileChanged(PathBuf),

    /// Mode switched
    ModeChanged {
        from: String,
        to: String,
    },

    /// UI state changed
    UiStateChanged(UiStateChange),
}

/// Text change event details
#[derive(Debug, Clone)]
pub struct TextChange {
    /// Byte offset where change occurred
    pub offset: usize,

    /// Number of bytes deleted
    pub deleted: usize,

    /// Text inserted
    pub inserted: String,
}

/// UI state change types
#[derive(Debug, Clone)]
pub enum UiStateChange {
    CommandPaletteOpened,
    CommandPaletteClosed,
    AgentPaletteOpened,
    AgentPaletteClosed,
    FileTreeToggled(bool),
    TmuxPanelToggled(bool),
}
```

---

## Text Buffer API

### `TextBuffer`

Rope-based text storage with efficient operations.

```rust
/// Text buffer using rope data structure for efficient large file handling
///
/// Operations are O(log n) for most text manipulations.
/// Supports Unicode correctly (grapheme clusters, not bytes).
///
/// # Thread Safety
/// `TextBuffer` is `Send` but not `Sync`. Use message passing for
/// concurrent access.
pub struct TextBuffer {
    rope: Rope,
    path: Option<PathBuf>,
    language: Language,
    dirty: bool,
    line_ending: LineEnding,
    encoding: Encoding,
    version: u64,
}

impl TextBuffer {
    /// Create new empty buffer
    ///
    /// # Examples
    /// ```rust
    /// let buffer = TextBuffer::new(Language::Rust);
    /// ```
    pub fn new(language: Language) -> Self;

    /// Load buffer from file
    ///
    /// # Errors
    /// Returns error if:
    /// - File does not exist
    /// - No read permissions
    /// - Invalid encoding
    ///
    /// # Examples
    /// ```rust
    /// let buffer = TextBuffer::from_file(Path::new("main.rs"))?;
    /// ```
    pub fn from_file(path: &Path) -> Result<Self>;

    /// Create buffer from string
    pub fn from_string(content: String, language: Language) -> Self;

    /// Insert text at byte offset
    ///
    /// # Performance
    /// O(log n) where n is the buffer size
    ///
    /// # Errors
    /// Returns error if offset is out of bounds
    ///
    /// # Examples
    /// ```rust
    /// buffer.insert(0, "Hello, world!")?;
    /// ```
    pub fn insert(&mut self, offset: usize, text: &str) -> Result<()>;

    /// Delete text range
    ///
    /// # Performance
    /// O(log n)
    ///
    /// # Examples
    /// ```rust
    /// buffer.delete(0..5)?; // Delete first 5 bytes
    /// ```
    pub fn delete(&mut self, range: Range<usize>) -> Result<()>;

    /// Replace text in range
    ///
    /// Equivalent to delete + insert, but more efficient.
    pub fn replace(&mut self, range: Range<usize>, text: &str) -> Result<()>;

    /// Get line by index (0-based)
    ///
    /// # Performance
    /// O(log n)
    ///
    /// Returns `None` if line index is out of bounds.
    pub fn line(&self, index: usize) -> Option<Cow<str>>;

    /// Get character count
    ///
    /// # Performance
    /// O(1)
    pub fn len_chars(&self) -> usize;

    /// Get byte count
    pub fn len_bytes(&self) -> usize;

    /// Get line count
    pub fn len_lines(&self) -> usize;

    /// Convert (line, column) to byte offset
    ///
    /// # Performance
    /// O(log n)
    ///
    /// Returns `None` if position is invalid.
    ///
    /// # Examples
    /// ```rust
    /// let offset = buffer.offset(10, 5)?; // Line 10, column 5
    /// ```
    pub fn offset(&self, line: usize, col: usize) -> Option<usize>;

    /// Convert byte offset to (line, column)
    ///
    /// # Performance
    /// O(log n)
    pub fn position(&self, offset: usize) -> (usize, usize);

    /// Get text slice as string
    ///
    /// # Performance
    /// O(log n + m) where m is the length of the slice
    ///
    /// # Examples
    /// ```rust
    /// let text = buffer.slice(0..10);
    /// ```
    pub fn slice(&self, range: Range<usize>) -> String;

    /// Get entire buffer content as string
    ///
    /// # Performance
    /// O(n) - Use sparingly for large files
    pub fn to_string(&self) -> String;

    /// Mark buffer as clean (after save)
    pub fn mark_clean(&mut self);

    /// Check if buffer has unsaved changes
    pub fn is_dirty(&self) -> bool;

    /// Get buffer's file path
    pub fn path(&self) -> Option<&Path>;

    /// Get buffer's language
    pub fn language(&self) -> &Language;

    /// Get buffer version (increments on each change)
    ///
    /// Used for LSP synchronization.
    pub fn version(&self) -> u64;

    /// Save buffer to file
    ///
    /// Uses atomic write (write to temp file, then rename).
    ///
    /// # Errors
    /// Returns error if:
    /// - No write permissions
    /// - Disk full
    /// - I/O error
    pub fn save(&mut self) -> Result<()>;

    /// Save buffer to new path
    pub fn save_as(&mut self, path: &Path) -> Result<()>;
}
```

### `BufferManager`

Manage multiple open buffers.

```rust
/// Manager for multiple text buffers
///
/// Provides buffer lifecycle management (open, close, save).
/// Maintains an LRU cache for recently accessed buffers.
pub struct BufferManager {
    buffers: HashMap<BufferId, TextBuffer>,
    active_buffer: Option<BufferId>,
    next_id: BufferId,
    lru: LruCache<BufferId, ()>,
}

impl BufferManager {
    /// Create new buffer manager
    pub fn new() -> Self;

    /// Create new empty buffer
    ///
    /// Returns the buffer ID.
    pub fn new_buffer(&mut self, language: Language) -> BufferId;

    /// Open file as buffer
    ///
    /// If the file is already open, returns existing buffer ID.
    ///
    /// # Examples
    /// ```rust
    /// let id = manager.open_file(Path::new("main.rs"))?;
    /// ```
    pub fn open_file(&mut self, path: &Path) -> Result<BufferId>;

    /// Get buffer by ID
    pub fn get(&self, id: BufferId) -> Option<&TextBuffer>;

    /// Get mutable buffer by ID
    pub fn get_mut(&mut self, id: BufferId) -> Option<&mut TextBuffer>;

    /// Close buffer
    ///
    /// # Errors
    /// Returns error if buffer has unsaved changes (unless `force = true`)
    pub fn close(&mut self, id: BufferId, force: bool) -> Result<()>;

    /// Save buffer
    pub fn save(&mut self, id: BufferId) -> Result<()>;

    /// Save buffer to new path
    pub fn save_as(&mut self, id: BufferId, path: &Path) -> Result<()>;

    /// Switch active buffer
    pub fn switch_to(&mut self, id: BufferId) -> Result<()>;

    /// Get active buffer ID
    pub fn active_buffer_id(&self) -> Option<BufferId>;

    /// List all buffer IDs
    pub fn buffer_ids(&self) -> Vec<BufferId>;

    /// Get buffers with unsaved changes
    pub fn dirty_buffers(&self) -> Vec<BufferId>;

    /// Close all buffers
    ///
    /// Returns list of buffer IDs that had unsaved changes.
    pub fn close_all(&mut self, force: bool) -> Result<Vec<BufferId>>;
}

/// Buffer identifier (monotonically increasing)
pub type BufferId = u64;
```

---

## Cursor Management API

### `Cursor`

Single cursor with optional selection.

```rust
/// A cursor position with optional selection range
///
/// The cursor is always at a byte offset. When a selection is active,
/// the `anchor` defines the selection start and `pos` defines the end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    pos: usize,
    anchor: Option<usize>,
    preferred_col: Option<usize>,
}

impl Cursor {
    /// Create cursor at position
    pub fn new(pos: usize) -> Self;

    /// Get cursor position
    pub fn pos(&self) -> usize;

    /// Set cursor position
    pub fn set_pos(&mut self, pos: usize);

    /// Move cursor by character offset
    ///
    /// Handles grapheme clusters correctly.
    pub fn move_by(&mut self, buffer: &TextBuffer, offset: isize);

    /// Move cursor left
    pub fn move_left(&mut self, buffer: &TextBuffer, count: usize);

    /// Move cursor right
    pub fn move_right(&mut self, buffer: &TextBuffer, count: usize);

    /// Move cursor up
    ///
    /// Preserves preferred column across lines of different lengths.
    pub fn move_up(&mut self, buffer: &TextBuffer, count: usize);

    /// Move cursor down
    pub fn move_down(&mut self, buffer: &TextBuffer, count: usize);

    /// Move to start of line
    pub fn move_to_line_start(&mut self, buffer: &TextBuffer);

    /// Move to end of line
    pub fn move_to_line_end(&mut self, buffer: &TextBuffer);

    /// Move to start of buffer
    pub fn move_to_buffer_start(&mut self);

    /// Move to end of buffer
    pub fn move_to_buffer_end(&mut self, buffer: &TextBuffer);

    /// Start selection at current position
    pub fn start_selection(&mut self);

    /// Extend selection to current position
    pub fn extend_selection(&mut self);

    /// Clear selection
    pub fn clear_selection(&mut self);

    /// Get selection range (if any)
    ///
    /// Returns `None` if no selection is active.
    /// Range is always normalized (start < end).
    pub fn selection(&self) -> Option<Range<usize>>;

    /// Check if selection is active
    pub fn has_selection(&self) -> bool;
}
```

### `CursorSet`

Multi-cursor support (Phase 2).

```rust
/// Multiple cursors (Phase 2)
///
/// Supports Sublime Text / VS Code style multi-cursor editing.
pub struct CursorSet {
    primary: Cursor,
    secondary: Vec<Cursor>,
}

impl CursorSet {
    /// Create cursor set with single cursor
    pub fn new(pos: usize) -> Self;

    /// Add cursor at position
    ///
    /// Cursors are automatically merged if they overlap.
    pub fn add_cursor(&mut self, pos: usize);

    /// Remove cursor by index
    pub fn remove_cursor(&mut self, index: usize) -> Result<()>;

    /// Get all cursors
    pub fn cursors(&self) -> impl Iterator<Item = &Cursor>;

    /// Get mutable cursor iterator
    pub fn cursors_mut(&mut self) -> impl Iterator<Item = &mut Cursor>;

    /// Apply operation to all cursors
    ///
    /// # Examples
    /// ```rust
    /// cursors.apply(buffer, |cursor, buffer| {
    ///     cursor.move_right(buffer, 1);
    /// });
    /// ```
    pub fn apply<F>(&mut self, buffer: &TextBuffer, f: F)
    where
        F: Fn(&mut Cursor, &TextBuffer);

    /// Merge overlapping cursors
    pub fn merge_cursors(&mut self);

    /// Clear all secondary cursors
    pub fn clear_secondary(&mut self);
}
```

---

## Mode System API

### `Mode` Trait

Define behavior for different editing modes.

```rust
/// Editing mode (Vim-style modal editing)
///
/// Each mode defines how key presses are handled.
/// Modes can transition to other modes.
pub trait Mode: Send + Sync {
    /// Handle key press in this mode
    ///
    /// Returns a mode transition (stay, switch, or exit).
    ///
    /// # Examples
    /// ```rust
    /// fn handle_key(&self, key: KeyEvent, ctx: &mut EditorContext)
    ///     -> Result<ModeTransition>
    /// {
    ///     match key.code {
    ///         KeyCode::Char('i') => {
    ///             Ok(ModeTransition::Switch(Box::new(InsertMode::new())))
    ///         }
    ///         _ => Ok(ModeTransition::Stay),
    ///     }
    /// }
    /// ```
    fn handle_key(
        &mut self,
        key: KeyEvent,
        ctx: &mut EditorContext,
    ) -> Result<ModeTransition>;

    /// Get mode indicator text
    ///
    /// Displayed in status bar (e.g., "NORMAL", "INSERT").
    fn indicator(&self) -> &str;

    /// Called when entering this mode
    ///
    /// Use for mode-specific initialization.
    fn on_enter(&mut self, ctx: &mut EditorContext) -> Result<()> {
        Ok(())
    }

    /// Called when exiting this mode
    ///
    /// Use for cleanup.
    fn on_exit(&mut self, ctx: &mut EditorContext) -> Result<()> {
        Ok(())
    }
}

/// Mode transition result
pub enum ModeTransition {
    /// Stay in current mode
    Stay,

    /// Switch to new mode
    Switch(Box<dyn Mode>),

    /// Exit editor
    Exit,
}
```

### Built-in Modes

```rust
/// Normal mode (Vim-style navigation and commands)
pub struct NormalMode {
    pending: Option<char>,
    count: Option<usize>,
}

impl NormalMode {
    pub fn new() -> Self;
}

/// Insert mode (text insertion)
pub struct InsertMode;

impl InsertMode {
    pub fn new() -> Self;
}

/// Visual mode (text selection)
pub struct VisualMode {
    mode: VisualModeKind,
}

pub enum VisualModeKind {
    Character,
    Line,
    Block,
}

impl VisualMode {
    pub fn new(kind: VisualModeKind) -> Self;
}

/// Command mode (Ex commands like :w, :q)
pub struct CommandMode {
    input: String,
    history: Vec<String>,
    history_index: usize,
}

impl CommandMode {
    pub fn new() -> Self;

    /// Execute command from input
    fn execute_command(&self, cmd: &str, ctx: &mut EditorContext) -> Result<()>;
}
```

---

## Command System API

### `Command` Trait

Executable and undoable operations.

```rust
/// A command that can be executed and undone
///
/// All editor operations that modify state should be commands.
/// This enables undo/redo functionality.
pub trait Command: Send + Sync {
    /// Execute the command
    ///
    /// # Errors
    /// Returns error if command execution fails.
    fn execute(&mut self, ctx: &mut EditorContext) -> Result<()>;

    /// Undo the command
    ///
    /// Should reverse the effects of `execute()`.
    ///
    /// # Errors
    /// Returns error if undo is not possible or fails.
    fn undo(&mut self, ctx: &mut EditorContext) -> Result<()>;

    /// Get command description
    ///
    /// Used for command history display.
    fn description(&self) -> &str;

    /// Check if command can be undone
    ///
    /// Some commands (like save) cannot be undone.
    fn can_undo(&self) -> bool {
        true
    }
}
```

### Built-in Commands

```rust
/// Insert text at cursor position
pub struct InsertTextCommand {
    buffer_id: BufferId,
    pos: usize,
    text: String,
}

impl InsertTextCommand {
    pub fn new(buffer_id: BufferId, pos: usize, text: impl Into<String>) -> Self;
}

/// Delete text range
pub struct DeleteTextCommand {
    buffer_id: BufferId,
    range: Range<usize>,
    deleted_text: Option<String>, // Populated during execute
}

impl DeleteTextCommand {
    pub fn new(buffer_id: BufferId, range: Range<usize>) -> Self;
}

/// Replace text in range
pub struct ReplaceTextCommand {
    buffer_id: BufferId,
    range: Range<usize>,
    old_text: Option<String>,
    new_text: String,
}

impl ReplaceTextCommand {
    pub fn new(buffer_id: BufferId, range: Range<usize>, new_text: String) -> Self;
}

/// Save file command
pub struct SaveFileCommand {
    buffer_id: BufferId,
}

impl SaveFileCommand {
    pub fn new(buffer_id: BufferId) -> Self;
}

/// Open file command
pub struct OpenFileCommand {
    path: PathBuf,
    buffer_id: Option<BufferId>, // Populated during execute
}

impl OpenFileCommand {
    pub fn new(path: PathBuf) -> Self;
}

/// Close buffer command
pub struct CloseBufferCommand {
    buffer_id: BufferId,
    force: bool,
}

impl CloseBufferCommand {
    pub fn new(buffer_id: BufferId, force: bool) -> Self;
}
```

---

## LSP Client API

### `LspClient`

Async LSP communication.

```rust
/// LSP client for code intelligence
///
/// Manages LSP servers for different languages.
/// All operations are async and non-blocking.
pub struct LspClient {
    servers: HashMap<String, ServerHandle>,
    event_tx: mpsc::Sender<EditorEvent>,
}

impl LspClient {
    /// Create new LSP client
    pub async fn new(event_tx: mpsc::Sender<EditorEvent>) -> Result<Self>;

    /// Start LSP server for language
    ///
    /// # Errors
    /// Returns error if:
    /// - Server binary not found
    /// - Server fails to start
    /// - Initialization fails
    ///
    /// # Examples
    /// ```rust
    /// let config = LspServerConfig {
    ///     binary: "rust-analyzer".into(),
    ///     args: vec![],
    /// };
    /// lsp.start_server("rust", config).await?;
    /// ```
    pub async fn start_server(
        &mut self,
        language: &str,
        config: LspServerConfig,
    ) -> Result<()>;

    /// Stop LSP server
    pub async fn stop_server(&mut self, language: &str) -> Result<()>;

    /// Notify: Text document opened
    ///
    /// Call this when opening a file in the editor.
    pub async fn did_open(&self, buffer: &TextBuffer) -> Result<()>;

    /// Notify: Text document changed
    ///
    /// Call this after buffer modifications.
    pub async fn did_change(
        &self,
        buffer: &TextBuffer,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<()>;

    /// Notify: Text document saved
    pub async fn did_save(&self, buffer: &TextBuffer) -> Result<()>;

    /// Notify: Text document closed
    pub async fn did_close(&self, buffer: &TextBuffer) -> Result<()>;

    /// Request: Completion
    ///
    /// # Performance
    /// Target: <100ms response time
    ///
    /// # Examples
    /// ```rust
    /// let items = lsp.completion(buffer, Position::new(10, 5)).await?;
    /// ```
    pub async fn completion(
        &self,
        buffer: &TextBuffer,
        pos: Position,
    ) -> Result<Vec<CompletionItem>>;

    /// Request: Hover information
    ///
    /// Returns documentation for symbol at position.
    pub async fn hover(
        &self,
        buffer: &TextBuffer,
        pos: Position,
    ) -> Result<Option<Hover>>;

    /// Request: Go to definition
    ///
    /// # Performance
    /// Target: <50ms response time
    pub async fn goto_definition(
        &self,
        buffer: &TextBuffer,
        pos: Position,
    ) -> Result<Vec<Location>>;

    /// Request: Find references
    pub async fn find_references(
        &self,
        buffer: &TextBuffer,
        pos: Position,
    ) -> Result<Vec<Location>>;

    /// Request: Document symbols
    ///
    /// Returns all symbols in the document (functions, classes, etc.).
    pub async fn document_symbols(
        &self,
        buffer: &TextBuffer,
    ) -> Result<Vec<DocumentSymbol>>;

    /// Request: Rename symbol
    pub async fn rename(
        &self,
        buffer: &TextBuffer,
        pos: Position,
        new_name: String,
    ) -> Result<WorkspaceEdit>;

    /// Request: Code actions
    ///
    /// Returns available refactorings and quick fixes.
    pub async fn code_actions(
        &self,
        buffer: &TextBuffer,
        range: Range<usize>,
    ) -> Result<Vec<CodeAction>>;

    /// Request: Formatting
    pub async fn format_document(
        &self,
        buffer: &TextBuffer,
    ) -> Result<Vec<TextEdit>>;
}

/// LSP server configuration
#[derive(Debug, Clone)]
pub struct LspServerConfig {
    pub binary: String,
    pub args: Vec<String>,
}

/// Re-export LSP types from tower-lsp
pub use tower_lsp::lsp_types::{
    CompletionItem,
    Hover,
    Location,
    Position,
    DocumentSymbol,
    WorkspaceEdit,
    CodeAction,
    TextEdit,
    TextDocumentContentChangeEvent,
    Diagnostic,
    DiagnosticSeverity,
};
```

---

## AIT42 Integration API

### `AgentRegistry`

Load and manage AIT42 agents.

```rust
/// Registry of AIT42 agents
///
/// Loads agent metadata from `.claude/agents/*.md` files.
pub struct AgentRegistry {
    agents: HashMap<String, AgentMetadata>,
    agent_dir: PathBuf,
}

impl AgentRegistry {
    /// Create new agent registry
    ///
    /// # Examples
    /// ```rust
    /// let registry = AgentRegistry::new(Path::new(".claude/agents"))?;
    /// ```
    pub fn new(agent_dir: PathBuf) -> Result<Self>;

    /// Load all agents from directory
    ///
    /// Parses YAML frontmatter from markdown files.
    ///
    /// # Errors
    /// Returns error if:
    /// - Agent directory does not exist
    /// - Agent file has invalid frontmatter
    /// - YAML parsing fails
    pub fn load_all(&mut self) -> Result<()>;

    /// Get agent by name
    pub fn get(&self, name: &str) -> Option<&AgentMetadata>;

    /// List all agents
    pub fn all(&self) -> impl Iterator<Item = &AgentMetadata>;

    /// Search agents by name or description
    ///
    /// # Examples
    /// ```rust
    /// let results = registry.search("backend");
    /// ```
    pub fn search(&self, query: &str) -> Vec<&AgentMetadata>;

    /// Get agent count
    pub fn count(&self) -> usize;
}

/// Agent metadata from frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Agent name (unique identifier)
    pub name: String,

    /// Short description
    pub description: String,

    /// Available tools
    pub tools: Vec<String>,

    /// Model to use (e.g., "claude-3-7-sonnet-20250219")
    pub model: String,

    /// Agent prompt (body of markdown file)
    pub prompt: String,

    /// Optional: Recommended for parallel execution
    #[serde(default)]
    pub parallel_recommended: bool,

    /// Optional: Requires Coordinator
    #[serde(default)]
    pub requires_coordinator: bool,
}
```

### `TmuxSessionManager`

Manage tmux sessions for agent execution.

```rust
/// Tmux session manager for parallel agent execution
///
/// Creates isolated tmux sessions for each agent.
/// Supports up to 5 parallel sessions by default (configurable).
pub struct TmuxSessionManager {
    sessions: HashMap<AgentId, TmuxSession>,
    next_id: AgentId,
    max_parallel: usize,
}

impl TmuxSessionManager {
    /// Create new session manager
    pub fn new(max_parallel: usize) -> Self;

    /// Create new tmux session for agent
    ///
    /// # Errors
    /// Returns error if:
    /// - Tmux is not installed
    /// - Session creation fails
    /// - Max parallel limit reached
    ///
    /// # Examples
    /// ```rust
    /// let session = tmux.create_session("backend-developer").await?;
    /// ```
    pub async fn create_session(&mut self, agent_name: &str) -> Result<TmuxSession>;

    /// Execute command in tmux session
    ///
    /// Sends command as if typed in terminal, followed by Enter.
    ///
    /// # Examples
    /// ```rust
    /// tmux.execute_in_session(&session, "cargo build").await?;
    /// ```
    pub async fn execute_in_session(
        &self,
        session: &TmuxSession,
        command: &str,
    ) -> Result<()>;

    /// Capture output from tmux session
    ///
    /// Returns the visible pane content.
    pub async fn capture_output(&self, session: &TmuxSession) -> Result<String>;

    /// Attach to tmux session
    ///
    /// Opens tmux session in current terminal (blocks until detach).
    pub async fn attach(&self, session: &TmuxSession) -> Result<()>;

    /// Destroy tmux session
    ///
    /// Kills the session and cleans up resources.
    pub async fn destroy_session(&mut self, agent_id: AgentId) -> Result<()>;

    /// List all active sessions
    pub fn active_sessions(&self) -> Vec<&TmuxSession>;

    /// Get session by agent ID
    pub fn get_session(&self, agent_id: AgentId) -> Option<&TmuxSession>;

    /// Check if tmux is available
    pub fn is_tmux_available() -> bool;
}

/// Tmux session information
#[derive(Debug, Clone)]
pub struct TmuxSession {
    pub name: String,
    pub agent_id: AgentId,
    pub created_at: Instant,
    pub status: SessionStatus,
}

#[derive(Debug, Clone)]
pub enum SessionStatus {
    Creating,
    Running,
    Completed,
    Failed(String),
}

pub type AgentId = u64;
```

### `AgentExecutor`

Execute AIT42 agents.

```rust
/// Agent execution coordinator
///
/// Handles agent lifecycle: preparation, execution, monitoring, cleanup.
pub struct AgentExecutor {
    registry: AgentRegistry,
    tmux: TmuxSessionManager,
    event_tx: mpsc::Sender<EditorEvent>,
}

impl AgentExecutor {
    /// Create new agent executor
    pub fn new(
        registry: AgentRegistry,
        tmux: TmuxSessionManager,
        event_tx: mpsc::Sender<EditorEvent>,
    ) -> Self;

    /// Execute agent
    ///
    /// # Execution Strategy
    /// - If `parallel_recommended`: Use tmux session
    /// - If `requires_coordinator`: Invoke Coordinator first
    /// - Otherwise: Direct execution
    ///
    /// # Examples
    /// ```rust
    /// let agent = registry.get("backend-developer").unwrap();
    /// let result = executor.execute(agent, "Implement REST API").await?;
    /// ```
    pub async fn execute(
        &mut self,
        agent: &AgentMetadata,
        task: &str,
    ) -> Result<AgentResult>;

    /// Execute multiple agents in parallel
    ///
    /// Creates separate tmux sessions for each agent.
    ///
    /// # Examples
    /// ```rust
    /// let agents = vec![
    ///     ("backend-developer", "Implement API"),
    ///     ("frontend-developer", "Create UI"),
    /// ];
    /// let results = executor.execute_parallel(agents).await?;
    /// ```
    pub async fn execute_parallel(
        &mut self,
        tasks: Vec<(&AgentMetadata, &str)>,
    ) -> Result<Vec<AgentResult>>;

    /// Get execution status
    pub fn get_status(&self, agent_id: AgentId) -> Option<SessionStatus>;

    /// Cancel agent execution
    pub async fn cancel(&mut self, agent_id: AgentId) -> Result<()>;
}

/// Agent execution result
#[derive(Debug, Clone)]
pub struct AgentResult {
    pub agent_id: AgentId,
    pub agent_name: String,
    pub output: String,
    pub exit_code: Option<i32>,
    pub duration: Duration,
    pub success: bool,
}
```

---

## TUI Rendering API

### `EditorWidget`

Main text editing area.

```rust
/// Editor widget for text rendering
///
/// Renders buffer content with syntax highlighting, line numbers,
/// and cursor position.
pub struct EditorWidget<'a> {
    buffer: &'a TextBuffer,
    cursor: &'a Cursor,
    theme: &'a ColorScheme,
    scroll_offset: usize,
    line_numbers: bool,
    syntax_highlights: Option<&'a Vec<HighlightRange>>,
}

impl<'a> EditorWidget<'a> {
    /// Create new editor widget
    pub fn new(
        buffer: &'a TextBuffer,
        cursor: &'a Cursor,
        theme: &'a ColorScheme,
    ) -> Self;

    /// Enable/disable line numbers
    pub fn line_numbers(mut self, enabled: bool) -> Self;

    /// Set syntax highlights
    pub fn syntax_highlights(mut self, highlights: &'a Vec<HighlightRange>) -> Self;

    /// Set scroll offset
    pub fn scroll_offset(mut self, offset: usize) -> Self;
}

impl<'a> Widget for EditorWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer);
}

/// Syntax highlight range
#[derive(Debug, Clone)]
pub struct HighlightRange {
    pub line: usize,
    pub start: usize,
    pub end: usize,
    pub kind: HighlightKind,
}

#[derive(Debug, Clone, Copy)]
pub enum HighlightKind {
    Keyword,
    Function,
    Type,
    String,
    Number,
    Comment,
    Variable,
    Operator,
}
```

### `StatusBar`

Status bar widget.

```rust
/// Status bar widget
///
/// Displays mode, file info, cursor position.
pub struct StatusBar<'a> {
    mode: &'a str,
    file_name: Option<&'a str>,
    cursor_pos: (usize, usize),
    file_type: &'a str,
    dirty: bool,
    theme: &'a ColorScheme,
}

impl<'a> StatusBar<'a> {
    pub fn new(
        mode: &'a str,
        file_name: Option<&'a str>,
        cursor_pos: (usize, usize),
        file_type: &'a str,
        dirty: bool,
        theme: &'a ColorScheme,
    ) -> Self;
}

impl<'a> Widget for StatusBar<'a>;
```

### `CommandPalette`

Command palette for agent/file selection.

```rust
/// Command palette widget
///
/// Fuzzy searchable list of agents, files, and commands.
pub struct CommandPalette<'a> {
    items: Vec<PaletteItem>,
    selected: usize,
    filter: &'a str,
    theme: &'a ColorScheme,
}

impl<'a> CommandPalette<'a> {
    pub fn new(
        items: Vec<PaletteItem>,
        filter: &'a str,
        theme: &'a ColorScheme,
    ) -> Self;
}

impl<'a> Widget for CommandPalette<'a>;

/// Palette item
#[derive(Debug, Clone)]
pub struct PaletteItem {
    pub title: String,
    pub description: String,
    pub kind: PaletteItemKind,
}

#[derive(Debug, Clone)]
pub enum PaletteItemKind {
    Agent(String),
    File(PathBuf),
    Command(String),
}
```

---

## Configuration API

### `Config`

User configuration.

```rust
/// Editor configuration
///
/// Loaded from `~/.config/ait42-editor/config.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub editor: EditorConfig,
    pub keybindings: KeybindingConfig,
    pub ait42: AIT42Config,
    pub lsp: LspConfig,
    pub appearance: AppearanceConfig,
}

impl Config {
    /// Load configuration from default location
    ///
    /// Falls back to defaults if config file doesn't exist.
    pub fn load() -> Result<Self>;

    /// Load from specific path
    pub fn load_from(path: &Path) -> Result<Self>;

    /// Save configuration
    pub fn save(&self) -> Result<()>;

    /// Save to specific path
    pub fn save_to(&self, path: &Path) -> Result<()>;

    /// Validate configuration
    ///
    /// Checks for invalid values.
    pub fn validate(&self) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub theme: String,
    pub tab_size: usize,
    pub auto_save: bool,
    pub auto_save_delay: u64, // milliseconds
    pub line_numbers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    pub command_palette: String,
    pub save: String,
    pub quit: String,
    pub agent_palette: String,
    pub toggle_file_tree: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIT42Config {
    pub coordinator_enabled: bool,
    pub tmux_parallel_max: usize,
    pub auto_tmux: bool,
    pub agent_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspConfig {
    pub rust: Option<String>,
    pub typescript: Option<String>,
    pub python: Option<String>,
    pub go: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub show_whitespace: bool,
    pub highlight_current_line: bool,
    pub cursor_style: CursorStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CursorStyle {
    Block,
    Line,
    Underline,
}
```

---

## Error Handling

### `Error` Type

Unified error type for all editor operations.

```rust
/// Editor error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Buffer not found: {0}")]
    BufferNotFound(BufferId),

    #[error("Invalid buffer offset: {0}")]
    InvalidOffset(usize),

    #[error("LSP error: {0}")]
    Lsp(String),

    #[error("LSP server not found: {0}")]
    LspServerNotFound(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Tmux error: {0}")]
    Tmux(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Cannot undo: {0}")]
    CannotUndo(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("Event bus full")]
    EventBusFull,

    #[error("Unsaved changes in buffer: {0}")]
    UnsavedChanges(BufferId),
}

pub type Result<T> = std::result::Result<T, Error>;
```

---

## Usage Examples

### Complete Editor Lifecycle

```rust
use ait42_editor::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Create editor context
    let mut ctx = EditorContext::new(config).await?;

    // Load agents
    ctx.agents.load_all()?;

    // Run editor (blocks until quit)
    ctx.run().await?;

    Ok(())
}
```

### Open and Edit File

```rust
// Open file
let buffer_id = ctx.buffers.open_file(Path::new("main.rs"))?;

// Switch to insert mode
ctx.switch_mode(Box::new(InsertMode::new()))?;

// Insert text at cursor
let pos = ctx.cursor().pos();
let cmd = InsertTextCommand::new(buffer_id, pos, "Hello, world!");
ctx.execute_command(Box::new(cmd))?;

// Save file
let save_cmd = SaveFileCommand::new(buffer_id);
ctx.execute_command(Box::new(save_cmd))?;
```

### Execute AIT42 Agent

```rust
// Get agent
let agent = ctx.agents.get("backend-developer")
    .ok_or_else(|| Error::AgentNotFound("backend-developer".into()))?;

// Create tmux session
let session = ctx.tmux.create_session(&agent.name).await?;

// Execute agent task
let task = "Implement REST API for user management";
ctx.tmux.execute_in_session(&session, &format!(
    "ait42 agent {} --task '{}'",
    agent.name,
    task
)).await?;

// Monitor execution (non-blocking)
tokio::spawn(async move {
    loop {
        let output = ctx.tmux.capture_output(&session).await?;
        println!("{}", output);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
});
```

### LSP Completion

```rust
// Start LSP server
ctx.lsp.start_server("rust", LspServerConfig {
    binary: "rust-analyzer".into(),
    args: vec![],
}).await?;

// Open file
let buffer_id = ctx.buffers.open_file(Path::new("main.rs"))?;
let buffer = ctx.buffers.get(buffer_id).unwrap();

// Notify LSP
ctx.lsp.did_open(buffer).await?;

// Request completion
let pos = Position::new(10, 5);
let items = ctx.lsp.completion(buffer, pos).await?;

// Display completion items in UI
for item in items {
    println!("{}: {}", item.label, item.detail.unwrap_or_default());
}
```

### Multi-Agent Parallel Execution

```rust
let agents = vec![
    (ctx.agents.get("backend-developer").unwrap(), "Implement API"),
    (ctx.agents.get("frontend-developer").unwrap(), "Create UI"),
    (ctx.agents.get("qa-engineer").unwrap(), "Write tests"),
];

// Execute in parallel
let executor = AgentExecutor::new(
    ctx.agents.clone(),
    ctx.tmux.clone(),
    ctx.event_tx.clone(),
);

let results = executor.execute_parallel(agents).await?;

for result in results {
    println!("Agent {}: {}", result.agent_name,
        if result.success { "Success" } else { "Failed" });
    println!("Output: {}", result.output);
}
```

---

**End of API Specification**

Generated by: AIT42 Coordinator
Date: 2025-01-06
Version: 1.0.0
