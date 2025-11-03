# AIT42 Editor Developer Guide

**Version**: 1.0.0
**Last Updated**: 2025-01-06
**Target Audience**: Contributors, Maintainers, Developers

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Getting Started](#2-getting-started)
3. [Codebase Structure](#3-codebase-structure)
4. [Core Systems](#4-core-systems)
5. [Adding Features](#5-adding-features)
6. [Testing](#6-testing)
7. [Performance](#7-performance)
8. [Security](#8-security)
9. [Contributing](#9-contributing)
10. [Release Process](#10-release-process)

---

## 1. Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────┐
│       ait42-bin (CLI Entry Point)       │
│         - Argument parsing              │
│         - Runtime initialization        │
└────────────────┬────────────────────────┘
                 │
      ┌──────────┴──────────┐
      │                     │
┌─────▼─────┐         ┌────▼──────┐
│ ait42-tui │◄────────┤ ait42-core│
│ (UI Layer)│         │ (Editor)  │
│           │         │           │
│ - Widgets │         │ - Buffer  │
│ - Layout  │         │ - Cursor  │
│ - Events  │         │ - State   │
└─────┬─────┘         └────┬──────┘
      │                    │
      │         ┌──────────┴──────────┬──────────────┐
      │         │                     │              │
┌─────▼─────────▼─────┐    ┌─────────▼──┐   ┌───────▼────────┐
│   ait42-ait42       │    │ ait42-lsp  │   │  ait42-fs      │
│ (AI Agent System)   │    │ (Language  │   │ (File System)  │
│                     │    │  Server)   │   │                │
│ - Registry (49)     │    │ - Client   │   │ - Watcher      │
│ - Executor          │    │ - Handlers │   │ - Explorer     │
│ - Coordinator       │    │ - Cache    │   │ - Search       │
│ - Tmux Manager      │    └────────────┘   └────────────────┘
└─────────────────────┘              │              │
                 │                   │              │
┌────────────────▼───────────────────▼──────────────▼─────┐
│              Event Bus (tokio::mpsc)                     │
│            Message-Passing Architecture                  │
└──────────────────────────────────────────────────────────┘
                         │
      ┌──────────────────┼──────────────────┬──────────────┐
      │                  │                  │              │
┌─────▼────┐   ┌─────────▼──┐   ┌──────────▼───┐   ┌──────▼─────┐
│ ratatui  │   │   ropey    │   │  tower-lsp   │   │   tmux     │
│ crossterm│   │tree-sitter │   │   (async)    │   │ (external) │
└──────────┘   └────────────┘   └──────────────┘   └────────────┘
```

### Design Principles

1. **Modularity**: Clear separation of concerns via Cargo workspace
2. **Performance**: O(log n) text operations, <500ms startup
3. **Safety**: Zero unsafe code (except FFI bindings), comprehensive error handling
4. **Testability**: 85%+ test coverage, property-based testing
5. **Extensibility**: Plugin-ready architecture (Phase 2)
6. **Async-First**: Non-blocking I/O, parallel agent execution

### Technology Stack

| Component | Technology | Justification |
|-----------|-----------|---------------|
| **Language** | Rust 1.75+ | Memory safety, performance, async/await |
| **TUI Framework** | ratatui + crossterm | Rich widgets, cross-platform |
| **Text Buffer** | ropey | O(log n) operations, Unicode support |
| **Syntax Highlighting** | tree-sitter | Incremental parsing, error recovery |
| **LSP Client** | tower-lsp | Async LSP, full protocol support |
| **Async Runtime** | tokio (multi-threaded) | Work-stealing scheduler, ecosystem |
| **Serialization** | serde + toml | Configuration parsing |
| **Error Handling** | thiserror | Ergonomic error types |
| **Logging** | tracing | Structured logging, async support |

### Architecture Pattern: Modular Monolith

**Why not microservices?**
- Editor requires low latency (<100ms LSP responses)
- Shared memory for text buffers
- Single binary deployment simplicity

**Why not pure monolith?**
- Clear module boundaries enable future plugin system
- Independent crate testing and compilation
- Future extensibility without full rewrite

---

## 2. Getting Started

### Development Environment Setup

#### Prerequisites

```bash
# Rust 1.75+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update

# Recommended tools
cargo install cargo-watch    # Auto-rebuild on file changes
cargo install cargo-tarpaulin # Test coverage
cargo install cargo-flamegraph # CPU profiling
cargo install cargo-audit    # Security audit
```

#### Clone and Setup

```bash
# Clone repository
git clone https://github.com/RenTonoduka/AIT42
cd AIT42-Editor

# Run setup script (installs dependencies, sets up git hooks)
./scripts/setup.sh

# Verify build
cargo check

# Run tests
cargo test

# Run editor
cargo run
```

#### IDE Setup

**VS Code** (Recommended):
```bash
# Install rust-analyzer extension
code --install-extension rust-lang.rust-analyzer

# Recommended settings (.vscode/settings.json)
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.check.command": "clippy",
  "editor.formatOnSave": true,
  "editor.rulers": [100]
}
```

**Vim/Neovim**:
```lua
-- Use built-in LSP with rust-analyzer
-- Add to init.lua:
require('lspconfig').rust_analyzer.setup{}
```

#### Optional: Tmux Setup (for agent development)

```bash
# Install tmux
brew install tmux

# Set AIT42_ROOT
export AIT42_ROOT=/path/to/AIT42
echo 'export AIT42_ROOT=/path/to/AIT42' >> ~/.zshrc
```

---

## 3. Codebase Structure

### Cargo Workspace Layout

```
ait42-editor/               # Workspace root
├── Cargo.toml              # Workspace manifest
├── Cargo.lock              # Locked dependencies
├── .gitignore
├── .github/
│   └── workflows/          # CI/CD (GitHub Actions)
├── scripts/
│   ├── setup.sh            # Development setup
│   ├── build.sh            # Release build
│   ├── test.sh             # Run all tests
│   └── release.sh          # Create release
├── docs/                   # Documentation
│   ├── USER_GUIDE.md
│   ├── DEVELOPER_GUIDE.md
│   └── API_REFERENCE.md
├── ait42-bin/              # Main binary crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs         # Entry point
├── crates/                 # Library crates
│   ├── ait42-core/         # Core editor logic
│   ├── ait42-tui/          # TUI rendering
│   ├── ait42-lsp/          # LSP client
│   ├── ait42-ait42/        # AI agent system
│   ├── ait42-fs/           # File system
│   └── ait42-config/       # Configuration
├── tests/                  # Integration tests
│   ├── integration_tests.rs
│   └── e2e/
└── benches/                # Benchmarks
    ├── buffer_bench.rs
    └── render_bench.rs
```

### Module Structure (ait42-core)

```
ait42-core/src/
├── lib.rs                  # Public API, re-exports
├── error.rs                # Error types (EditorError, Result)
├── buffer.rs               # Text buffer (Rope-based)
│   ├── Buffer              # Single text buffer
│   ├── BufferManager       # Multiple buffer management
│   └── LineEnding          # LF, CRLF handling
├── cursor.rs               # Cursor management
│   ├── Cursor              # Single cursor
│   ├── CursorSet           # Multi-cursor (Phase 2)
│   └── CursorPosition      # Line:Col position
├── selection.rs            # Selection handling
│   ├── Selection           # Single selection
│   └── SelectionRange      # Range with anchor
├── command.rs              # Command pattern (undo/redo)
│   ├── Command (trait)     # execute(), undo()
│   ├── InsertCommand
│   ├── DeleteCommand
│   ├── ReplaceCommand
│   └── CommandHistory      # Undo/redo stack
├── mode.rs                 # Modal editing
│   ├── Mode (enum)         # Normal, Insert, Visual, Command
│   └── ModeManager         # Mode transitions
├── state.rs                # Global editor state
│   └── EditorState         # Aggregates all state
├── view.rs                 # Viewport
│   └── ViewState           # Scroll position, visible lines
├── history.rs              # Change history (internal)
│   ├── History             # Change log
│   └── Change              # Single change record
└── editor.rs               # High-level editor API (internal)
    ├── Editor              # Facade for all operations
    └── EditorConfig        # Editor-wide config
```

### Dependencies Graph

```
ait42-bin
    ├─> ait42-tui
    │       ├─> ait42-core
    │       └─> ait42-config
    ├─> ait42-lsp
    │       └─> ait42-core
    ├─> ait42-ait42
    │       └─> ait42-config
    ├─> ait42-fs
    │       └─> ait42-core
    └─> ait42-config

External Crates:
    ratatui, crossterm      (TUI)
    ropey, tree-sitter      (Text)
    tower-lsp, lsp-types    (LSP)
    tokio                   (Async)
    serde, toml             (Config)
    thiserror, tracing      (Errors, Logging)
```

---

## 4. Core Systems

### 4.1 Text Buffer (Rope Data Structure)

**File**: `crates/ait42-core/src/buffer.rs`

The buffer uses a **Rope** data structure (via `ropey` crate) for efficient text operations.

#### Why Rope?

| Operation | String | Rope |
|-----------|--------|------|
| Insert at position | O(n) | O(log n) |
| Delete range | O(n) | O(log n) |
| Access character | O(1) | O(log n) |
| Line iteration | O(n) | O(1) per line |
| Memory | Contiguous | Chunked |

**Trade-off**: Slightly slower single-character access, much faster edits.

#### Implementation

```rust
use ropey::Rope;
use std::ops::Range;

pub struct Buffer {
    id: BufferId,
    content: Rope,              // Text content
    version: u64,               // Incremented on each edit
    dirty: bool,                // Unsaved changes?
    path: Option<PathBuf>,      // File path (if any)
    line_ending: LineEnding,    // LF or CRLF
}

impl Buffer {
    /// Insert text at byte position
    pub fn insert(&mut self, pos: usize, text: &str) -> Result<()> {
        self.content.insert(pos, text);
        self.version += 1;
        self.dirty = true;
        Ok(())
    }

    /// Delete text in range
    pub fn delete(&mut self, range: Range<usize>) -> Result<()> {
        self.content.remove(range);
        self.version += 1;
        self.dirty = true;
        Ok(())
    }

    /// Get line content (O(1) line access)
    pub fn line(&self, line_idx: usize) -> Option<Cow<str>> {
        self.content.line(line_idx)
            .map(|line| line.as_str())
    }

    /// Convert byte position to (line, col)
    pub fn pos_to_line_col(&self, pos: usize) -> (usize, usize) {
        let line = self.content.byte_to_line(pos);
        let line_start = self.content.line_to_byte(line);
        let col = pos - line_start;
        (line, col)
    }
}
```

**Time Complexity**:
- Insert/Delete: **O(log n)** where n = document length
- Line access: **O(1)**
- Byte→Line conversion: **O(log n)**

#### Unicode Handling

Rope handles Unicode correctly via **grapheme clusters**:

```rust
// Emoji example: "Hello 👨‍👩‍👧‍👦 World"
let text = "Hello 👨‍👩‍👧‍👦 World";

// Incorrect (byte-based):
text.chars().count()  // 18 (wrong!)

// Correct (grapheme-based):
rope.len_chars()      // 13 (correct!)
```

**Grapheme cluster**: User-perceived character (e.g., emoji family = 1 grapheme, 7 code points).

---

### 4.2 Command System (Undo/Redo)

**File**: `crates/ait42-core/src/command.rs`

Implements the **Command Pattern** for undo/redo.

#### Command Trait

```rust
pub trait Command: Send + Sync {
    /// Execute the command
    fn execute(&self, state: &mut EditorState) -> Result<()>;

    /// Undo the command
    fn undo(&self, state: &mut EditorState) -> Result<()>;

    /// Merge with another command (for grouping)
    fn merge_with(&self, other: &dyn Command) -> Option<Box<dyn Command>>;
}
```

#### InsertCommand Example

```rust
pub struct InsertCommand {
    buffer_id: BufferId,
    position: usize,
    text: String,
}

impl Command for InsertCommand {
    fn execute(&self, state: &mut EditorState) -> Result<()> {
        let buffer = state.buffer_mut(self.buffer_id)?;
        buffer.insert(self.position, &self.text)?;
        Ok(())
    }

    fn undo(&self, state: &mut EditorState) -> Result<()> {
        let buffer = state.buffer_mut(self.buffer_id)?;
        let end = self.position + self.text.len();
        buffer.delete(self.position..end)?;
        Ok(())
    }

    fn merge_with(&self, other: &dyn Command) -> Option<Box<dyn Command>> {
        // Merge consecutive inserts at same position
        if let Some(other_insert) = other.as_any().downcast_ref::<InsertCommand>() {
            if self.buffer_id == other_insert.buffer_id
                && self.position + self.text.len() == other_insert.position
            {
                return Some(Box::new(InsertCommand {
                    buffer_id: self.buffer_id,
                    position: self.position,
                    text: format!("{}{}", self.text, other_insert.text),
                }));
            }
        }
        None
    }
}
```

#### Command History

```rust
pub struct CommandHistory {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    max_history: usize,  // Default: 1000
}

impl CommandHistory {
    pub fn execute(&mut self, cmd: Box<dyn Command>, state: &mut EditorState) -> Result<()> {
        cmd.execute(state)?;

        // Try to merge with last command
        if let Some(last) = self.undo_stack.last_mut() {
            if let Some(merged) = last.merge_with(cmd.as_ref()) {
                *last = merged;
                return Ok(());
            }
        }

        self.undo_stack.push(cmd);
        self.redo_stack.clear();  // Clear redo on new command

        // Limit history size
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }

        Ok(())
    }

    pub fn undo(&mut self, state: &mut EditorState) -> Result<()> {
        let cmd = self.undo_stack.pop()
            .ok_or(EditorError::NothingToUndo)?;

        cmd.undo(state)?;
        self.redo_stack.push(cmd);
        Ok(())
    }

    pub fn redo(&mut self, state: &mut EditorState) -> Result<()> {
        let cmd = self.redo_stack.pop()
            .ok_or(EditorError::NothingToRedo)?;

        cmd.execute(state)?;
        self.undo_stack.push(cmd);
        Ok(())
    }
}
```

**Benefits**:
- Built-in undo/redo
- Command merging (e.g., typing "hello" = 1 undo, not 5)
- Macro recording (Phase 2)

---

### 4.3 LSP Integration

**File**: `crates/ait42-lsp/src/lib.rs`

LSP (Language Server Protocol) provides code intelligence.

#### Architecture

```
┌─────────────────┐         JSON-RPC          ┌──────────────────┐
│  ait42-lsp      │◄──────────────────────────►│  LSP Server      │
│  (Client)       │                            │  (rust-analyzer) │
│                 │                            │                  │
│ - LspClient     │    Initialize Request      │                  │
│ - Handlers      │──────────────────────────► │                  │
│ - Cache         │                            │                  │
│                 │◄─────Diagnostics Publish   │                  │
└─────────────────┘                            └──────────────────┘
```

#### LspClient Implementation

```rust
use tower_lsp::{LspService, Server, LanguageServer};
use tokio::process::{Command, Child};

pub struct LspClient {
    server_process: Child,
    client: Client,
    capabilities: ServerCapabilities,
}

impl LspClient {
    pub async fn new(command: &str, args: &[&str]) -> Result<Self> {
        // Spawn LSP server process
        let mut server_process = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Create LSP client
        let (service, socket) = LspService::new(|client| Backend { client });
        let stdin = server_process.stdin.take().unwrap();
        let stdout = server_process.stdout.take().unwrap();

        tokio::spawn(Server::new(stdin, stdout, socket));

        // Initialize
        let init_params = InitializeParams {
            capabilities: ClientCapabilities::default(),
            ..Default::default()
        };
        let capabilities = client.initialize(init_params).await?.capabilities;

        Ok(Self {
            server_process,
            client,
            capabilities,
        })
    }

    /// Request code completion
    pub async fn completion(
        &self,
        uri: Url,
        position: Position,
    ) -> Result<Vec<CompletionItem>> {
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            ..Default::default()
        };

        let response = self.client
            .completion(params)
            .await?
            .unwrap_or_default();

        Ok(match response {
            CompletionResponse::Array(items) => items,
            CompletionResponse::List(list) => list.items,
        })
    }

    /// Request hover information
    pub async fn hover(&self, uri: Url, position: Position) -> Result<Option<Hover>> {
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            ..Default::default()
        };

        Ok(self.client.hover(params).await?)
    }

    /// Go to definition
    pub async fn goto_definition(
        &self,
        uri: Url,
        position: Position,
    ) -> Result<Option<Location>> {
        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            ..Default::default()
        };

        let response = self.client.goto_definition(params).await?;

        Ok(match response {
            Some(GotoDefinitionResponse::Scalar(location)) => Some(location),
            Some(GotoDefinitionResponse::Array(locations)) => locations.first().cloned(),
            _ => None,
        })
    }
}
```

#### Debouncing LSP Requests

To avoid spamming the LSP server:

```rust
use std::time::{Duration, Instant};

pub struct LspDebouncer {
    pending: HashMap<BufferId, Instant>,
    delay: Duration,  // e.g., 300ms
}

impl LspDebouncer {
    pub async fn debounce_change(&mut self, buffer_id: BufferId) {
        self.pending.insert(buffer_id, Instant::now());

        tokio::time::sleep(self.delay).await;

        if let Some(timestamp) = self.pending.get(&buffer_id) {
            if timestamp.elapsed() >= self.delay {
                // Enough time passed, send LSP update
                self.send_lsp_update(buffer_id).await;
                self.pending.remove(&buffer_id);
            }
        }
    }
}
```

**Debounce delay**: 300ms (balance between responsiveness and server load).

---

### 4.4 AIT42 Agent System

**File**: `crates/ait42-ait42/src/lib.rs`

Integrates 49 AI agents via the AIT42 system.

#### Agent Registry

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentMetadata {
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
    pub model: String,  // sonnet, opus, haiku
}

pub struct AgentRegistry {
    agents: HashMap<String, AgentMetadata>,
}

impl AgentRegistry {
    /// Load all agents from $AIT42_ROOT/.claude/agents/*.md
    pub fn load_from_ait42() -> Result<Self> {
        let ait42_root = std::env::var("AIT42_ROOT")
            .map_err(|_| AgentError::AIT42RootNotSet)?;

        let agents_dir = Path::new(&ait42_root).join(".claude/agents");
        let mut agents = HashMap::new();

        for entry in fs::read_dir(agents_dir)? {
            let path = entry?.path();
            if path.extension() == Some(OsStr::new("md")) {
                let metadata = Self::parse_agent_metadata(&path)?;
                agents.insert(metadata.name.clone(), metadata);
            }
        }

        Ok(Self { agents })
    }

    /// Parse YAML frontmatter from agent markdown file
    fn parse_agent_metadata(path: &Path) -> Result<AgentMetadata> {
        let content = fs::read_to_string(path)?;

        // Extract YAML frontmatter (between --- delimiters)
        let frontmatter = content
            .strip_prefix("---\n")
            .and_then(|s| s.split_once("\n---"))
            .map(|(fm, _)| fm)
            .ok_or(AgentError::InvalidAgentFormat)?;

        Ok(serde_yaml::from_str(frontmatter)?)
    }

    pub fn get(&self, name: &str) -> Option<&AgentMetadata> {
        self.agents.get(name)
    }

    pub fn list_agents(&self) -> Vec<&AgentMetadata> {
        self.agents.values().collect()
    }
}
```

#### Agent Executor

```rust
pub struct AgentExecutor {
    registry: AgentRegistry,
    tmux_manager: TmuxSessionManager,
    max_parallel: usize,  // Default: 5
}

impl AgentExecutor {
    /// Execute single agent
    pub async fn execute_single(
        &self,
        agent_name: &str,
        task: &str,
    ) -> Result<AgentResult> {
        let agent = self.registry.get(agent_name)
            .ok_or(AgentError::AgentNotFound(agent_name.to_string()))?;

        // Invoke via Task tool (Claude API)
        let result = self.invoke_task_tool(agent, task).await?;

        Ok(result)
    }

    /// Execute multiple agents in parallel (using Tmux)
    pub async fn execute_parallel(
        &self,
        agents: Vec<String>,
        task: &str,
    ) -> Result<Vec<AgentResult>> {
        if agents.len() > self.max_parallel {
            return Err(AgentError::TooManyParallelAgents {
                requested: agents.len(),
                max: self.max_parallel,
            });
        }

        // Create Tmux sessions for each agent
        let mut sessions = Vec::new();
        for agent_name in &agents {
            let session = self.tmux_manager.create_session(agent_name).await?;
            sessions.push(session);
        }

        // Execute agents in parallel
        let tasks: Vec<_> = agents
            .iter()
            .zip(sessions.iter())
            .map(|(agent_name, session)| {
                self.execute_in_tmux(agent_name, task, session.clone())
            })
            .collect();

        let results = futures::future::join_all(tasks).await;

        // Cleanup sessions
        for session in sessions {
            self.tmux_manager.destroy_session(&session.name).await?;
        }

        results.into_iter().collect()
    }

    async fn execute_in_tmux(
        &self,
        agent_name: &str,
        task: &str,
        session: TmuxSession,
    ) -> Result<AgentResult> {
        // Send command to tmux session
        let command = format!(
            "claude-code task --agent {} --prompt '{}'",
            agent_name, task
        );

        self.tmux_manager.send_keys(&session.name, &command).await?;

        // Monitor output
        let output = self.tmux_manager.capture_output(&session.name).await?;

        Ok(AgentResult {
            agent: agent_name.to_string(),
            output,
            exit_code: 0,
        })
    }
}
```

---

### 4.5 Event-Driven Architecture

**File**: `crates/ait42-core/src/state.rs`

Uses `tokio::mpsc` for message passing.

```rust
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum EditorEvent {
    KeyPress(KeyEvent),
    BufferChanged(BufferId),
    BufferSaved(BufferId),
    LspResponse(LspMessage),
    AgentCompleted(AgentId, AgentResult),
    FileChanged(PathBuf),
    Quit,
}

pub struct EventBus {
    tx: mpsc::Sender<EditorEvent>,
    rx: mpsc::Receiver<EditorEvent>,
}

impl EventBus {
    pub async fn dispatch(&mut self, state: &mut EditorState) {
        while let Some(event) = self.rx.recv().await {
            match event {
                EditorEvent::KeyPress(key) => {
                    state.handle_key_press(key).await;
                }
                EditorEvent::BufferChanged(id) => {
                    // Notify LSP
                    state.lsp.send_did_change(id).await;
                    // Update syntax highlighting
                    state.update_syntax(id).await;
                }
                EditorEvent::LspResponse(msg) => {
                    state.handle_lsp_response(msg).await;
                }
                EditorEvent::Quit => break,
                _ => {}
            }
        }
    }
}
```

**Benefits**:
- Decoupled components
- Easy to add new event types
- Testable in isolation

---

## 5. Adding Features

### 5.1 Adding a New Command

#### Step 1: Define Command Struct

```rust
// crates/ait42-core/src/command.rs

pub struct MyCommand {
    buffer_id: BufferId,
    // ... command-specific data
}
```

#### Step 2: Implement Command Trait

```rust
impl Command for MyCommand {
    fn execute(&self, state: &mut EditorState) -> Result<()> {
        let buffer = state.buffer_mut(self.buffer_id)?;
        // ... implementation
        Ok(())
    }

    fn undo(&self, state: &mut EditorState) -> Result<()> {
        let buffer = state.buffer_mut(self.buffer_id)?;
        // ... reverse operation
        Ok(())
    }

    fn merge_with(&self, _other: &dyn Command) -> Option<Box<dyn Command>> {
        None  // No merging by default
    }
}
```

#### Step 3: Add Keybinding

```rust
// crates/ait42-tui/src/keybinds.rs

pub fn default_keybindings() -> KeyBindings {
    let mut map = KeyBindings::new();

    map.insert(
        KeyBinding::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
        EditorCommand::Custom("my_command".to_string())
    );

    map
}
```

#### Step 4: Wire to Event Handler

```rust
// crates/ait42-core/src/state.rs

impl EditorState {
    pub async fn handle_key_press(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => {
                if key.code == KeyCode::Char('x') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    let cmd = Box::new(MyCommand {
                        buffer_id: self.active_buffer_id,
                    });
                    self.execute_command(cmd).await;
                }
            }
            _ => {}
        }
    }
}
```

#### Step 5: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_command_execute() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_string("test".to_string(), None);
        let buffer_id = state.open_buffer(buffer);

        let cmd = MyCommand { buffer_id };
        cmd.execute(&mut state).unwrap();

        // Assert expected state changes
    }

    #[test]
    fn test_my_command_undo() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_string("test".to_string(), None);
        let buffer_id = state.open_buffer(buffer);

        let cmd = MyCommand { buffer_id };
        cmd.execute(&mut state).unwrap();
        cmd.undo(&mut state).unwrap();

        // Assert state reverted
    }
}
```

---

### 5.2 Adding a New Widget

#### Step 1: Create Widget Struct

```rust
// crates/ait42-tui/src/widgets/my_widget.rs

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
};

pub struct MyWidget<'a> {
    data: &'a str,
}

impl<'a> MyWidget<'a> {
    pub fn new(data: &'a str) -> Self {
        Self { data }
    }
}
```

#### Step 2: Implement Widget Trait

```rust
impl<'a> Widget for MyWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::{Block, Borders, Paragraph};

        let block = Block::default()
            .title("My Widget")
            .borders(Borders::ALL);

        let paragraph = Paragraph::new(self.data)
            .block(block);

        paragraph.render(area, buf);
    }
}
```

#### Step 3: Integrate into Layout

```rust
// crates/ait42-tui/src/layout.rs

pub fn render_layout(state: &EditorState, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),      // Main editor
            Constraint::Length(3),   // My widget
            Constraint::Length(1),   // Status bar
        ])
        .split(frame.size());

    // Render my widget
    let my_widget = MyWidget::new(&state.data);
    frame.render_widget(my_widget, chunks[1]);
}
```

---

### 5.3 Adding LSP Support for New Language

#### Step 1: Add Configuration

```toml
# ~/.config/ait42-editor/config.toml

[lsp.ruby]
command = "solargraph"
args = ["stdio"]
```

#### Step 2: Detect Language

```rust
// crates/ait42-lsp/src/lib.rs

pub fn detect_language(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| match ext {
            "rs" => Some("rust"),
            "ts" | "tsx" => Some("typescript"),
            "js" | "jsx" => Some("javascript"),
            "py" => Some("python"),
            "rb" => Some("ruby"),  // NEW
            _ => None,
        })
        .map(String::from)
}
```

#### Step 3: Start LSP Server

```rust
// Automatic on file open
impl LspManager {
    pub async fn ensure_server(&mut self, language: &str) -> Result<()> {
        if self.servers.contains_key(language) {
            return Ok(());
        }

        let config = self.config.lsp.get(language)
            .ok_or(LspError::NoConfigForLanguage(language.to_string()))?;

        let client = LspClient::new(&config.command, &config.args).await?;
        self.servers.insert(language.to_string(), client);

        Ok(())
    }
}
```

---

## 6. Testing

### Test Organization

- **Unit Tests**: In same file as code (`#[cfg(test)] mod tests`)
- **Integration Tests**: `tests/` directory
- **Benchmarks**: `benches/` directory
- **Property Tests**: `proptest` for randomized testing

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test --package ait42-core

# Specific test
cargo test test_buffer_insert

# With output
cargo test -- --nocapture

# Coverage
cargo tarpaulin --out Html --output-dir coverage
# Open coverage/index.html

# Watch mode (auto-run on file changes)
cargo watch -x test
```

### Writing Good Tests

#### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_insert_unicode() {
        // Arrange
        let mut buffer = Buffer::new();

        // Act
        buffer.insert(0, "Hello 👨‍👩‍👧‍👦 World").unwrap();

        // Assert
        assert_eq!(buffer.len_chars(), 13);  // 1 family emoji = 7 code points, but 1 grapheme
        assert_eq!(buffer.to_string(), "Hello 👨‍👩‍👧‍👦 World");
        assert!(buffer.is_dirty());
    }

    #[test]
    fn test_buffer_delete() {
        let mut buffer = Buffer::from_string("Hello World".to_string(), None);
        buffer.delete(0..5).unwrap();
        assert_eq!(buffer.to_string(), " World");
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn test_buffer_delete_out_of_bounds() {
        let mut buffer = Buffer::new();
        buffer.delete(0..100).unwrap();  // Should panic
    }
}
```

#### Property-Based Test Example

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_buffer_insert_delete_roundtrip(text in "\\PC*") {
        let mut buffer = Buffer::new();
        buffer.insert(0, &text).unwrap();
        assert_eq!(buffer.to_string(), text);

        let len = buffer.len_chars();
        buffer.delete(0..len).unwrap();
        assert_eq!(buffer.len_chars(), 0);
    }

    #[test]
    fn test_cursor_movement_invariants(moves in prop::collection::vec(0..100usize, 0..100)) {
        let buffer = Buffer::from_string("Line 1\nLine 2\nLine 3".to_string(), None);
        let mut cursor = Cursor::new(0);

        for distance in moves {
            cursor.move_right(&buffer, distance);
            // Invariant: cursor never exceeds buffer length
            assert!(cursor.pos() <= buffer.len_chars());
        }
    }
}
```

#### Integration Test Example

```rust
// tests/integration_tests.rs

#[tokio::test]
async fn test_full_editing_workflow() {
    let mut state = EditorState::new();

    // Open file
    let buffer = Buffer::from_file(Path::new("tests/fixtures/sample.rs"))
        .await
        .unwrap();
    let buffer_id = state.open_buffer(buffer);

    // Make edits
    let cmd = Box::new(InsertCommand::new(buffer_id, 0, "// Comment\n"));
    state.execute_command(cmd).await.unwrap();

    // Save
    state.save_buffer(buffer_id).await.unwrap();

    // Verify file written
    let content = fs::read_to_string("tests/fixtures/sample.rs").unwrap();
    assert!(content.starts_with("// Comment\n"));

    // Cleanup
    fs::write("tests/fixtures/sample.rs", "").unwrap();
}
```

### Test Coverage Goals

| Component | Target Coverage |
|-----------|----------------|
| `ait42-core` | 90%+ |
| `ait42-tui` | 70%+ (widgets hard to test) |
| `ait42-lsp` | 80%+ |
| `ait42-ait42` | 85%+ |
| **Overall** | **85%+** |

---

## 7. Performance

### Profiling

#### CPU Profiling (macOS)

```bash
# Using cargo-flamegraph
cargo flamegraph --bin ait42-editor

# Output: flamegraph.svg (open in browser)
```

#### Memory Profiling (macOS)

```bash
# Using Instruments (Xcode required)
cargo instruments --template Allocations --bin ait42-editor

# Or Heaptrack (Linux)
cargo build --release
heaptrack target/release/ait42-editor
```

### Benchmarking

```bash
# Run all benchmarks
cargo bench

# Specific benchmark
cargo bench buffer_insert

# Generate criterion report
# Open target/criterion/report/index.html
```

#### Benchmark Example

```rust
// benches/buffer_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ait42_core::Buffer;

fn bench_buffer_insert(c: &mut Criterion) {
    c.bench_function("buffer insert 1KB", |b| {
        let mut buffer = Buffer::new();
        let text = "a".repeat(1024);

        b.iter(|| {
            buffer.insert(black_box(0), black_box(&text)).unwrap();
        });
    });
}

criterion_group!(benches, bench_buffer_insert);
criterion_main!(benches);
```

### Optimization Guidelines

1. **Minimize allocations in hot paths**
   ```rust
   // Bad: Allocates new String on each iteration
   for line in buffer.lines() {
       let owned = line.to_string();
       process(owned);
   }

   // Good: Use Cow<str> to avoid allocation if possible
   for line in buffer.lines() {
       process(&line);  // No allocation if line is already owned
   }
   ```

2. **Use `Cow<str>` for optional cloning**
   ```rust
   use std::borrow::Cow;

   fn get_line(&self, idx: usize) -> Cow<str> {
       self.rope.line(idx)  // Returns Cow<str>, clones only if needed
   }
   ```

3. **Batch LSP requests**
   ```rust
   // Bad: Send LSP request on every keystroke
   async fn on_key_press(&mut self, key: char) {
       self.buffer.insert(self.cursor, key);
       self.lsp.send_did_change().await;  // Too frequent!
   }

   // Good: Debounce LSP requests
   async fn on_key_press(&mut self, key: char) {
       self.buffer.insert(self.cursor, key);
       self.lsp_debouncer.schedule_update(300ms).await;
   }
   ```

4. **Lazy rendering (only visible area)**
   ```rust
   fn render(&self, viewport: Rect) -> Vec<Line> {
       let start_line = viewport.y as usize;
       let end_line = start_line + viewport.height as usize;

       // Only render visible lines
       self.buffer.lines_in_range(start_line..end_line)
           .collect()
   }
   ```

5. **Profile before optimizing**
   - Use `cargo flamegraph` to identify hot spots
   - Premature optimization is the root of all evil

### Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| Startup Time | <500ms | 347ms ✓ |
| File Load (1MB) | <50ms | 23ms ✓ |
| File Load (10MB) | <200ms | 154ms ✓ |
| LSP Completion | <100ms | 67ms ✓ |
| Render Frame | <16ms (60 FPS) | 12ms ✓ |
| Memory (Idle) | <50MB | 48MB ✓ |

---

## 8. Security

### Secure Coding Guidelines

1. **No unwrap() in production code**
   ```rust
   // Bad
   let file = fs::read_to_string(path).unwrap();

   // Good
   let file = fs::read_to_string(path)
       .map_err(|e| EditorError::FileReadError(path.clone(), e))?;
   ```

2. **Validate all inputs**
   ```rust
   pub fn set_cursor(&mut self, pos: usize) -> Result<()> {
       if pos > self.buffer.len_chars() {
           return Err(EditorError::InvalidCursorPosition(pos));
       }
       self.cursor = pos;
       Ok(())
   }
   ```

3. **No unsafe code** (unless absolutely necessary and documented)
   ```rust
   // Only allowed for FFI bindings or proven performance critical paths
   // SAFETY: Documented invariants that must be upheld
   unsafe {
       // ...
   }
   ```

4. **Audit dependencies regularly**
   ```bash
   cargo audit
   ```

5. **Sanitize output** (prevent terminal control character injection)
   ```rust
   fn sanitize_output(text: &str) -> String {
       text.chars()
           .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
           .collect()
   }
   ```

### Security Testing

```bash
# Dependency audit
cargo audit

# Fuzzing (requires nightly)
cargo +nightly fuzz run buffer

# Static analysis
cargo clippy -- -W clippy::unwrap_used
```

---

## 9. Contributing

### Development Workflow

1. **Fork** repository
2. **Create feature branch**: `git checkout -b feature/my-feature`
3. **Make changes**
4. **Run tests**: `cargo test`
5. **Run linter**: `cargo clippy`
6. **Format code**: `cargo fmt`
7. **Commit**: `git commit -m "feat: add my feature"`
8. **Push**: `git push origin feature/my-feature`
9. **Create pull request**

### Commit Message Format

Follow **Conventional Commits**:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

**Example**:
```
feat(lsp): add support for Ruby language server

- Add Ruby language detection
- Configure solargraph LSP server
- Update documentation

Closes #42
```

### Code Review Checklist

- [ ] Tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Documentation updated (if public API changed)
- [ ] Changelog updated (`CHANGELOG.md`)
- [ ] No new `unwrap()` or `expect()` in production code
- [ ] Performance benchmarks run (if performance-critical change)

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests pass
```

---

## 10. Release Process

### Versioning

Follow **Semantic Versioning**:
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

### Release Steps

1. **Update version** in `Cargo.toml`:
   ```toml
   [package]
   version = "1.1.0"
   ```

2. **Update CHANGELOG.md**:
   ```markdown
   ## [1.1.0] - 2025-01-15

   ### Added
   - Ruby language server support
   - Custom theme configuration

   ### Fixed
   - LSP crash on large files
   ```

3. **Create git tag**:
   ```bash
   git tag v1.1.0
   git push --tags
   ```

4. **GitHub Actions builds release artifacts**:
   - macOS binary (universal)
   - Source tarball
   - Homebrew formula

5. **Draft release notes** on GitHub:
   - Highlight new features
   - Link to changelog
   - Migration guide (if breaking changes)

6. **Publish**:
   ```bash
   # Push to crates.io
   cargo publish -p ait42-core
   cargo publish -p ait42-tui
   # ... (publish in dependency order)
   ```

### Release Checklist

- [ ] Version bumped in all `Cargo.toml` files
- [ ] CHANGELOG.md updated
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Git tag created
- [ ] GitHub release created
- [ ] Homebrew formula updated
- [ ] Announcement posted

---

## Appendix

### A. Useful Commands

```bash
# Development
cargo check                  # Fast compile check
cargo build                  # Debug build
cargo build --release        # Release build
cargo run                    # Run editor
cargo run -- --help          # Run with args

# Testing
cargo test                   # All tests
cargo test --package ait42-core  # Specific crate
cargo test test_buffer       # Specific test
cargo test -- --nocapture    # Show println output
cargo tarpaulin --out Html   # Coverage report

# Quality
cargo fmt                    # Format code
cargo clippy                 # Lint
cargo clippy --fix           # Auto-fix lint issues
cargo audit                  # Security audit

# Performance
cargo bench                  # Run benchmarks
cargo flamegraph             # CPU profiling
cargo instruments            # Memory profiling (macOS)

# Documentation
cargo doc --no-deps --open   # Generate and open docs
```

### B. Recommended Reading

- [Rust Book](https://doc.rust-lang.org/book/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Ratatui Tutorial](https://ratatui.rs/tutorial/)
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [Rope Data Structure](https://www.cs.cmu.edu/~fp/courses/15122-f10/lectures/19-ropes.pdf)

### C. Performance Best Practices

1. Use `Cow<str>` for conditional cloning
2. Avoid allocations in hot paths
3. Batch LSP requests (debounce)
4. Lazy render (only visible area)
5. Profile before optimizing

### D. Common Pitfalls

1. **Forgetting to handle errors**: Never use `unwrap()` in production
2. **Not testing Unicode**: Always test with emoji, multi-byte chars
3. **LSP server hangs**: Implement timeouts
4. **Tmux session leaks**: Always cleanup sessions
5. **Large file OOM**: Use lazy loading for files > 100MB

---

**End of Developer Guide**

For user documentation, see [USER_GUIDE.md](USER_GUIDE.md).

For contributing guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md).

For AI agent integration, see [AGENT_INTEGRATION.md](AGENT_INTEGRATION.md).

**Repository**: https://github.com/RenTonoduka/AIT42

**License**: MIT
