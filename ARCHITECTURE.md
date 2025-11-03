# AIT42 Editor - System Architecture

**Version**: 1.0.0
**Date**: 2025-01-06
**Status**: Design Phase
**Technology**: Rust + ratatui + crossterm + AIT42 Integration

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Architecture Overview](#architecture-overview)
3. [System Components](#system-components)
4. [Data Flow](#data-flow)
5. [Technology Stack Justification](#technology-stack-justification)
6. [Design Patterns](#design-patterns)
7. [Performance Architecture](#performance-architecture)
8. [Security Architecture](#security-architecture)
9. [Scalability Strategy](#scalability-strategy)

---

## Executive Summary

AIT42 Editor is a macOS-native terminal-based code editor with deep integration to the AIT42 multi-agent system. Built on Rust for performance and safety, it provides:

- **Sub-500ms startup time** through efficient binary architecture
- **49 AI agents** accessible via command palette
- **LSP integration** for intelligent code completion
- **Tmux session management** for parallel agent execution
- **rope-based text buffer** for efficient large file handling

### Key Architectural Decisions

| Decision | Rationale |
|----------|-----------|
| **Rust + ratatui** | Native performance, memory safety, rich TUI widgets |
| **Cargo workspace** | Modular architecture, independent compilation units |
| **Async/await (tokio)** | Non-blocking LSP, agent communication, file I/O |
| **rope data structure** | O(log n) edits, efficient large file handling |
| **Message-passing architecture** | Event-driven, decoupled components |
| **Plugin-ready design** | Phase 2 extensibility via stable interfaces |

---

## Architecture Overview

### High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         AIT42 Editor                                │
│                        (Main Binary)                                │
└──────────────┬──────────────────────────────────────────────────────┘
               │
    ┌──────────┴──────────────────────────────────────┐
    │                                                  │
┌───▼────┐  ┌──────┐  ┌──────┐  ┌────────┐  ┌────────▼─────┐
│  TUI   │  │ Core │  │ LSP  │  │  File  │  │    AIT42     │
│ Layer  │  │Editor│  │Client│  │ System │  │  Integration │
└───┬────┘  └──┬───┘  └──┬───┘  └───┬────┘  └────────┬─────┘
    │          │         │          │                 │
    │          │         │          │                 │
┌───▼──────────▼─────────▼──────────▼─────────────────▼─────┐
│                    Event Bus (tokio mpsc)                  │
│              Message-Passing Architecture                  │
└────────────────────────────────────────────────────────────┘
    │          │         │          │                 │
┌───▼────┐  ┌──▼───┐  ┌──▼───┐  ┌───▼────┐  ┌────────▼─────┐
│ratatui │  │ rope │  │tower-│  │  std   │  │     tmux     │
│crosstrm│  │tree- │  │ lsp  │  │  fs    │  │   (external) │
│        │  │sitter│  │      │  │        │  │              │
└────────┘  └──────┘  └──────┘  └────────┘  └──────────────┘
```

### Architecture Pattern: **Modular Monolith with Event-Driven Core**

**Why not microservices?**
- Editor requires low latency (sub-100ms LSP responses)
- Shared memory for text buffers
- Single binary deployment simplicity

**Why not pure monolith?**
- Clear module boundaries enable Phase 2 plugin system
- Independent crate testing and compilation
- Future extensibility without rewrite

---

## System Components

### Component Breakdown

```
ait42-editor/               # Cargo workspace root
├── crates/
│   ├── ait42-core/         # Core editor logic
│   │   ├── buffer/         # Text buffer (rope-based)
│   │   ├── cursor/         # Cursor management, selections
│   │   ├── mode/           # Vim-style modal editing
│   │   ├── command/        # Command parsing, execution
│   │   └── state/          # Global editor state
│   │
│   ├── ait42-tui/          # TUI rendering layer
│   │   ├── widgets/        # Custom ratatui widgets
│   │   ├── layout/         # Layout management
│   │   ├── theme/          # Color schemes, styling
│   │   └── input/          # Keyboard/mouse input handling
│   │
│   ├── ait42-lsp/          # LSP client integration
│   │   ├── client/         # LSP client implementation
│   │   ├── handlers/       # LSP message handlers
│   │   ├── completion/     # Completion UI integration
│   │   └── diagnostics/    # Error/warning display
│   │
│   ├── ait42-ait42/        # AIT42 agent integration
│   │   ├── agent/          # Agent definitions loader
│   │   ├── coordinator/    # Coordinator communication
│   │   ├── tmux/           # Tmux session management
│   │   └── ui/             # Agent execution UI
│   │
│   ├── ait42-fs/           # File system operations
│   │   ├── watcher/        # File change detection
│   │   ├── explorer/       # File tree navigation
│   │   └── search/         # File content search
│   │
│   └── ait42-config/       # Configuration management
│       ├── parser/         # TOML config parser
│       ├── schema/         # Config validation
│       └── defaults/       # Default settings
│
├── ait42-bin/              # Main binary crate
│   └── main.rs             # Application entry point
│
└── tests/
    ├── integration/        # Integration tests
    └── e2e/                # End-to-end tests
```

### Module Responsibilities

#### 1. `ait42-core` - Core Editor Engine

**Responsibility**: Text manipulation, cursor management, editing modes

**Key Components**:
- **Buffer**: rope-based text storage
  - `TextBuffer` - immutable rope operations
  - `BufferManager` - multiple buffer management
  - `UndoTree` - undo/redo history

- **Cursor**: Multi-cursor support
  - `Cursor` - position, selection range
  - `CursorSet` - multiple cursors (Phase 2)

- **Mode**: Vim-style modal editing
  - `NormalMode`, `InsertMode`, `VisualMode`, `CommandMode`
  - `ModeManager` - mode transitions

**Dependencies**: `ropey`, `tree-sitter`

**Performance Goals**:
- Insert character: <1ms
- Large file load (10MB): <200ms
- Undo operation: <5ms

---

#### 2. `ait42-tui` - TUI Rendering Layer

**Responsibility**: Terminal UI rendering, layout, theming

**Key Components**:
- **Widgets**:
  - `EditorWidget` - main text editing area
  - `StatusBar` - mode, cursor position, file info
  - `CommandPalette` - agent selection, file search
  - `SidePanel` - file tree, agent status
  - `TmuxPanel` - tmux session viewer

- **Layout**:
  - `LayoutManager` - responsive layout calculation
  - `SplitManager` - horizontal/vertical splits (Phase 2)

- **Theme**:
  - `ColorScheme` - syntax highlighting colors
  - `ThemeManager` - theme switching

**Dependencies**: `ratatui`, `crossterm`

**Performance Goals**:
- Render frame: <16ms (60 FPS)
- Syntax highlighting: <50ms for visible area

---

#### 3. `ait42-lsp` - LSP Client

**Responsibility**: Language Server Protocol integration

**Key Components**:
- **Client**:
  - `LspClient` - async LSP communication
  - `ServerManager` - start/stop LSP servers

- **Handlers**:
  - `CompletionHandler` - auto-completion
  - `HoverHandler` - hover documentation
  - `GotoDefinitionHandler` - jump to definition
  - `DiagnosticsHandler` - errors/warnings

- **Integration**:
  - `CompletionUI` - completion popup
  - `DiagnosticsUI` - inline error display

**Dependencies**: `tower-lsp`, `serde_json`, `tokio`

**Performance Goals**:
- Completion response: <100ms
- Goto definition: <50ms
- Diagnostics update: <200ms

---

#### 4. `ait42-ait42` - AIT42 Integration

**Responsibility**: 49 AI agents, Coordinator, Tmux management

**Key Components**:
- **Agent**:
  - `AgentLoader` - load `.claude/agents/*.md`
  - `AgentRegistry` - 49 agents + Coordinator
  - `AgentMetadata` - name, description, tools

- **Coordinator**:
  - `CoordinatorClient` - communicate with Coordinator agent
  - `AgentSelector` - auto-select optimal agents

- **Tmux**:
  - `TmuxSessionManager` - create/attach/destroy sessions
  - `TmuxMonitor` - monitor agent execution
  - `SessionUI` - display session status

- **UI**:
  - `AgentPalette` - command palette for agent selection
  - `AgentStatusPanel` - running agents display
  - `TmuxViewer` - integrated tmux output viewer

**Dependencies**: Custom (read YAML frontmatter), `tmux` (external)

**Performance Goals**:
- Agent metadata load: <50ms
- Tmux session creation: <200ms
- Coordinator response: <1s (network dependent)

---

#### 5. `ait42-fs` - File System

**Responsibility**: File operations, watching, search

**Key Components**:
- **Watcher**:
  - `FileWatcher` - detect external file changes
  - `AutoReload` - prompt user for reload

- **Explorer**:
  - `FileTree` - directory tree structure
  - `FileTreeUI` - file tree widget

- **Search**:
  - `FileSearch` - fuzzy file name search
  - `ContentSearch` - ripgrep integration (Phase 2)

**Dependencies**: `notify`, `ignore` (gitignore support)

**Performance Goals**:
- File tree load: <100ms (1000 files)
- File watch detection: <50ms
- Fuzzy search: <50ms

---

#### 6. `ait42-config` - Configuration

**Responsibility**: User configuration, defaults, validation

**Key Components**:
- **Parser**:
  - `ConfigParser` - parse `config.toml`
  - `Validator` - validate config schema

- **Schema**:
  - `EditorConfig` - editor settings
  - `KeybindingConfig` - custom keybindings
  - `AIT42Config` - AIT42 integration settings
  - `LspConfig` - LSP server settings

- **Defaults**:
  - `DefaultConfig` - built-in defaults

**Dependencies**: `serde`, `toml`

**Config File Example**:
```toml
[editor]
theme = "monokai"
tab_size = 4
auto_save = true
line_numbers = true

[keybindings]
command_palette = "Ctrl+P"
save = "Ctrl+S"
agent_palette = "Ctrl+Shift+A"

[ait42]
coordinator_enabled = true
tmux_parallel_max = 5
auto_tmux = true  # Auto-use tmux for parallel/long tasks

[lsp]
rust = "rust-analyzer"
typescript = "typescript-language-server"
python = "pyright"

[appearance]
show_whitespace = false
highlight_current_line = true
```

---

## Data Flow

### 1. User Input Flow

```
User Input (keyboard)
    ↓
crossterm::event::read()
    ↓
ait42-tui::input::InputHandler
    ↓
Event::KeyPress(key)
    ↓
Event Bus (tokio::mpsc)
    ↓
┌──────────────┬────────────────┬───────────────┐
│              │                │               │
▼              ▼                ▼               ▼
NormalMode  InsertMode   CommandMode     AgentPalette
│              │                │               │
▼              ▼                ▼               ▼
Command    Buffer::insert()  execute_cmd()  spawn_agent()
│              │                │               │
▼              ▼                ▼               ▼
ait42-core  ait42-core    ait42-core      ait42-ait42
│              │                │               │
└──────────────┴────────────────┴───────────────┘
                    ↓
            State Update
                    ↓
            ait42-tui::render()
                    ↓
            ratatui::Terminal
```

### 2. LSP Communication Flow

```
Text Change Event
    ↓
ait42-core::Buffer::insert()
    ↓
Event::BufferChanged
    ↓
ait42-lsp::LspClient
    ↓
LSP: textDocument/didChange (async)
    ↓
Language Server (external process)
    ↓
LSP Response: publishDiagnostics
    ↓
ait42-lsp::DiagnosticsHandler
    ↓
Event::DiagnosticsUpdated
    ↓
ait42-core::State::diagnostics
    ↓
ait42-tui::DiagnosticsWidget
    ↓
Render inline errors
```

### 3. AIT42 Agent Execution Flow

```
User: Ctrl+Shift+A (Agent Palette)
    ↓
ait42-tui::AgentPalette::open()
    ↓
Display 49 agents (fuzzy searchable)
    ↓
User selects: "backend-developer"
    ↓
ait42-ait42::AgentSelector::execute()
    ↓
Check: Parallel execution needed?
    ├─ Yes → ait42-ait42::TmuxSessionManager::create()
    │            ↓
    │        tmux new-session -d "ait42-backend-dev-{timestamp}"
    │            ↓
    │        Execute Task tool in tmux session
    │            ↓
    │        ait42-ait42::TmuxMonitor::attach()
    │            ↓
    │        Display real-time output in TmuxPanel
    │
    └─ No → Direct execution
                ↓
            Invoke Task tool via AIT42 system
                ↓
            Display output in status bar
```

### 4. File Save Flow

```
User: Ctrl+S
    ↓
ait42-core::Command::Save
    ↓
ait42-core::BufferManager::save()
    ↓
ait42-fs::write_file()
    ↓
Atomic write (write to .tmp, rename)
    ↓
Success → Update buffer state (clean)
    ↓
LSP: textDocument/didSave
    ↓
Auto-save to swap file (.swp) - clear
```

---

## Technology Stack Justification

### Core Language: **Rust**

**Reasons**:
1. **Performance**: Native speed, zero-cost abstractions
2. **Memory Safety**: No segfaults, data races prevented at compile time
3. **Concurrency**: Fearless async/await with tokio
4. **Rich Ecosystem**: ratatui, crossterm, ropey, tower-lsp

**Alternatives Considered**:
- ❌ **Go**: Garbage collection pauses unacceptable for editor
- ❌ **C++**: Memory safety concerns, slower development
- ✅ **Rust**: Best balance of performance, safety, productivity

---

### TUI Framework: **ratatui + crossterm**

**Reasons**:
1. **Rich Widgets**: Pre-built components (List, Table, Block)
2. **Cross-platform**: macOS, Linux, Windows (future)
3. **Active Development**: Well-maintained, frequent updates
4. **Performance**: Efficient rendering, minimal redraws

**Alternatives Considered**:
- ❌ **cursive**: Less flexible layout system
- ❌ **termion**: Lower-level, more boilerplate
- ✅ **ratatui + crossterm**: Best ecosystem, documentation

---

### Text Buffer: **ropey**

**Reasons**:
1. **Efficiency**: O(log n) inserts/deletes
2. **Large Files**: Handles 100MB+ files smoothly
3. **Unicode Support**: Proper grapheme cluster handling
4. **Line Indexing**: Fast line-based operations

**Alternatives Considered**:
- ❌ **String**: O(n) inserts, memory inefficient
- ❌ **Gap buffer**: Poor worst-case performance
- ✅ **Rope**: Industry standard for text editors

---

### Syntax Highlighting: **tree-sitter**

**Reasons**:
1. **Incremental Parsing**: Reparse only changed sections
2. **Language Support**: 50+ languages
3. **Error Recovery**: Continues parsing on syntax errors
4. **Performance**: Fast enough for real-time highlighting

**Alternatives Considered**:
- ❌ **Regex-based**: Inaccurate, slow on large files
- ❌ **Custom parser**: Too much effort
- ✅ **tree-sitter**: Industry standard (used by GitHub, Neovim)

---

### LSP Client: **tower-lsp**

**Reasons**:
1. **Async/Await**: Non-blocking LSP communication
2. **Full LSP Support**: LSP 3.17 specification
3. **Battle-Tested**: Used in production LSP servers
4. **Tokio Integration**: Seamless async runtime integration

**Alternatives Considered**:
- ❌ **lsp-types only**: Too low-level, reinvent protocol handling
- ❌ **Custom implementation**: Too complex
- ✅ **tower-lsp**: Best async LSP support

---

### Async Runtime: **tokio**

**Reasons**:
1. **Multi-threaded**: Parallel LSP, file I/O, agent execution
2. **Mature**: Production-grade, used by Discord, AWS
3. **Ecosystem**: Works with tower-lsp, notify, etc.
4. **Performance**: Efficient work-stealing scheduler

**Alternatives Considered**:
- ❌ **async-std**: Smaller ecosystem
- ❌ **smol**: Lightweight but fewer integrations
- ✅ **tokio**: Industry standard

---

## Design Patterns

### 1. **Event-Driven Architecture (Message Passing)**

**Pattern**: Central event bus using `tokio::mpsc`

**Implementation**:
```rust
// Event types
enum EditorEvent {
    KeyPress(KeyEvent),
    BufferChanged(BufferId),
    LspResponse(LspMessage),
    AgentCompleted(AgentId),
    FileChanged(PathBuf),
}

// Event bus
struct EventBus {
    tx: mpsc::Sender<EditorEvent>,
    rx: mpsc::Receiver<EditorEvent>,
}

// Components subscribe to events
impl EventBus {
    async fn dispatch(&mut self) {
        while let Some(event) = self.rx.recv().await {
            match event {
                EditorEvent::KeyPress(key) => {
                    self.handle_key_press(key).await;
                }
                EditorEvent::BufferChanged(id) => {
                    self.notify_lsp_change(id).await;
                }
                // ... other events
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

### 2. **Command Pattern (Vim-style Commands)**

**Pattern**: Commands as first-class objects

**Implementation**:
```rust
trait Command {
    fn execute(&self, ctx: &mut EditorContext) -> Result<()>;
    fn undo(&self, ctx: &mut EditorContext) -> Result<()>;
}

struct InsertTextCommand {
    buffer_id: BufferId,
    position: usize,
    text: String,
}

impl Command for InsertTextCommand {
    fn execute(&self, ctx: &mut EditorContext) -> Result<()> {
        ctx.buffers.get_mut(self.buffer_id)?
            .insert(self.position, &self.text)?;
        Ok(())
    }

    fn undo(&self, ctx: &mut EditorContext) -> Result<()> {
        ctx.buffers.get_mut(self.buffer_id)?
            .delete(self.position, self.text.len())?;
        Ok(())
    }
}
```

**Benefits**:
- Built-in undo/redo
- Command history
- Macro recording (Phase 2)

---

### 3. **Strategy Pattern (Modal Editing)**

**Pattern**: Different behaviors for different modes

**Implementation**:
```rust
trait Mode {
    fn handle_key(&self, key: KeyEvent, ctx: &mut EditorContext) -> Result<Transition>;
    fn render(&self, ctx: &EditorContext) -> String;
}

struct NormalMode;
struct InsertMode;
struct VisualMode;

impl Mode for NormalMode {
    fn handle_key(&self, key: KeyEvent, ctx: &mut EditorContext) -> Result<Transition> {
        match key {
            KeyEvent::Char('i') => Ok(Transition::SwitchMode(Box::new(InsertMode))),
            KeyEvent::Char('v') => Ok(Transition::SwitchMode(Box::new(VisualMode))),
            KeyEvent::Char('d') => {
                ctx.execute_command(DeleteLineCommand::new())?;
                Ok(Transition::Stay)
            }
            _ => Ok(Transition::Stay),
        }
    }
}
```

**Benefits**:
- Clean mode separation
- Easy to add new modes
- Mode-specific keybindings

---

### 4. **Observer Pattern (File Watching)**

**Pattern**: File system changes trigger updates

**Implementation**:
```rust
use notify::{Watcher, RecursiveMode};

struct FileWatcher {
    watcher: RecommendedWatcher,
    tx: mpsc::Sender<EditorEvent>,
}

impl FileWatcher {
    fn watch_file(&mut self, path: PathBuf) -> Result<()> {
        let tx = self.tx.clone();
        self.watcher.watch(&path, RecursiveMode::NonRecursive)?;

        // On file change, send event
        // (handled by notify callback)
        Ok(())
    }
}
```

**Benefits**:
- Auto-reload on external changes
- Real-time file tree updates

---

### 5. **Facade Pattern (LSP Integration)**

**Pattern**: Simplify complex LSP protocol

**Implementation**:
```rust
pub struct LspFacade {
    client: LspClient,
    servers: HashMap<String, ServerHandle>,
}

impl LspFacade {
    // Simple interface hiding LSP complexity
    pub async fn get_completion(&self, buffer: &Buffer, pos: Position)
        -> Result<Vec<CompletionItem>>
    {
        let language = detect_language(&buffer.path)?;
        let server = self.servers.get(&language)?;

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: buffer.path.to_uri(),
                },
                position: pos,
            },
            // ... other params
        };

        let response = server.completion(params).await?;
        Ok(response.items)
    }
}
```

**Benefits**:
- Hide LSP protocol complexity
- Single entry point for all LSP operations
- Easy to mock for testing

---

### 6. **Factory Pattern (Agent Creation)**

**Pattern**: Dynamically create agents from metadata

**Implementation**:
```rust
struct AgentFactory {
    registry: AgentRegistry,
}

impl AgentFactory {
    fn create_agent(&self, name: &str) -> Result<Agent> {
        let metadata = self.registry.get(name)?;

        Agent {
            name: metadata.name.clone(),
            description: metadata.description.clone(),
            tools: metadata.tools.clone(),
            prompt: metadata.prompt.clone(),
        }
    }

    fn load_all_agents(&mut self) -> Result<()> {
        let agent_dir = Path::new(".claude/agents");
        for entry in fs::read_dir(agent_dir)? {
            let path = entry?.path();
            if path.extension() == Some("md") {
                let metadata = self.parse_agent_metadata(&path)?;
                self.registry.register(metadata);
            }
        }
        Ok(())
    }
}
```

**Benefits**:
- Dynamic agent loading
- Hot-reload agent definitions (Phase 2)
- Easy agent registration

---

## Performance Architecture

### 1. Async/Await Strategy

**Tokio Runtime Configuration**:
```rust
// main.rs
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    // Editor runs on async runtime
    let editor = Editor::new().await?;
    editor.run().await?;
    Ok(())
}
```

**Task Allocation**:
- **Main Thread**: UI rendering (must be fast, <16ms)
- **LSP Thread**: LSP communication (async I/O)
- **File I/O Thread**: File operations (async)
- **Agent Thread**: AIT42 agent execution (async)

---

### 2. Memory Management for Large Files

**Lazy Loading Strategy**:
```rust
struct Buffer {
    rope: Rope,              // Full text in memory
    visible_range: Range,    // Only this range is syntax highlighted
    syntax_tree: Option<Tree>, // Incremental parsing
}

impl Buffer {
    fn load_large_file(path: &Path) -> Result<Self> {
        // Use memory-mapped file for huge files (>100MB)
        if file_size > 100 * 1024 * 1024 {
            Self::from_mmap(path)
        } else {
            Self::from_string(fs::read_to_string(path)?)
        }
    }

    fn update_visible_range(&mut self, range: Range) {
        self.visible_range = range;
        // Only highlight visible lines
        self.update_syntax_highlighting(range);
    }
}
```

**Benefits**:
- Sub-1s load time for 100MB files
- Constant memory usage regardless of file size

---

### 3. UI Rendering Optimization

**Differential Rendering**:
```rust
struct TerminalRenderer {
    last_frame: Option<Frame>,
}

impl TerminalRenderer {
    fn render(&mut self, new_frame: Frame) {
        // Only redraw changed areas
        if let Some(last) = &self.last_frame {
            let diff = new_frame.diff(last);
            self.apply_diff(diff);
        } else {
            self.draw_full(new_frame);
        }
        self.last_frame = Some(new_frame);
    }
}
```

**Render Budget**:
- Target: 60 FPS (16ms per frame)
- Syntax highlighting: Max 8ms
- Layout calculation: Max 4ms
- Terminal write: Max 4ms

---

### 4. LSP Response Optimization

**Debouncing Strategy**:
```rust
struct LspDebouncer {
    pending: HashMap<BufferId, Instant>,
    delay: Duration,
}

impl LspDebouncer {
    async fn debounce_change(&mut self, buffer_id: BufferId) {
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

**Configuration**:
- Debounce delay: 300ms (balance responsiveness vs. LSP load)
- Completion trigger delay: 100ms

---

## Security Architecture

### 1. File Permission Handling

**Strategy**: Respect macOS file permissions, no privilege escalation

```rust
use std::os::unix::fs::PermissionsExt;

impl FileSystem {
    fn can_write(&self, path: &Path) -> Result<bool> {
        let metadata = fs::metadata(path)?;
        let permissions = metadata.permissions();

        // Check user write permission
        Ok(permissions.mode() & 0o200 != 0)
    }

    fn safe_write(&self, path: &Path, content: &str) -> Result<()> {
        if !self.can_write(path)? {
            return Err(Error::PermissionDenied(path.to_path_buf()));
        }

        // Atomic write
        let tmp_path = path.with_extension(".tmp");
        fs::write(&tmp_path, content)?;
        fs::rename(tmp_path, path)?;

        Ok(())
    }
}
```

---

### 2. Agent Execution Sandboxing (Tmux Sessions)

**Strategy**: Isolate agent execution in separate tmux sessions

```rust
struct TmuxSessionManager {
    sessions: HashMap<AgentId, TmuxSession>,
}

impl TmuxSessionManager {
    async fn create_session(&mut self, agent: Agent) -> Result<TmuxSession> {
        let session_name = format!("ait42-{}-{}", agent.name, timestamp());

        // Create isolated tmux session
        Command::new("tmux")
            .arg("new-session")
            .arg("-d")  // Detached
            .arg("-s").arg(&session_name)
            .output()
            .await?;

        // Agent runs in this isolated environment
        let session = TmuxSession {
            name: session_name,
            agent_id: agent.id,
            created_at: Instant::now(),
        };

        self.sessions.insert(agent.id, session.clone());
        Ok(session)
    }

    async fn destroy_session(&mut self, agent_id: AgentId) -> Result<()> {
        if let Some(session) = self.sessions.remove(&agent_id) {
            Command::new("tmux")
                .arg("kill-session")
                .arg("-t").arg(&session.name)
                .output()
                .await?;
        }
        Ok(())
    }
}
```

**Benefits**:
- Agent failures don't crash editor
- Multiple agents can run in parallel safely
- Easy to inspect agent execution (tmux attach)

---

### 3. Configuration File Validation

**Strategy**: Validate all user input, provide safe defaults

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Config {
    #[serde(default = "default_theme")]
    theme: String,

    #[serde(default = "default_tab_size")]
    tab_size: usize,

    #[serde(default)]
    keybindings: HashMap<String, String>,
}

impl Config {
    fn validate(&self) -> Result<()> {
        // Validate tab size
        if self.tab_size == 0 || self.tab_size > 8 {
            return Err(Error::InvalidConfig("tab_size must be 1-8".into()));
        }

        // Validate theme exists
        if !BUILTIN_THEMES.contains(&self.theme.as_str()) {
            return Err(Error::InvalidConfig(format!("Unknown theme: {}", self.theme)));
        }

        // Validate keybindings are parseable
        for (name, binding) in &self.keybindings {
            KeyBinding::parse(binding)
                .map_err(|_| Error::InvalidConfig(format!("Invalid keybinding: {}", name)))?;
        }

        Ok(())
    }
}

fn default_theme() -> String { "monokai".to_string() }
fn default_tab_size() -> usize { 4 }
```

---

### 4. Safe External Command Execution

**Strategy**: Whitelist allowed commands, validate inputs

```rust
enum AllowedCommand {
    Tmux,
    Git,  // Phase 2
}

impl AllowedCommand {
    fn execute(&self, args: &[&str]) -> Result<Output> {
        // Validate args (no shell injection)
        for arg in args {
            if arg.contains(';') || arg.contains('|') || arg.contains('&') {
                return Err(Error::UnsafeCommand("Shell metacharacters not allowed".into()));
            }
        }

        let binary = match self {
            AllowedCommand::Tmux => "tmux",
            AllowedCommand::Git => "git",
        };

        Command::new(binary)
            .args(args)
            .output()
            .await
    }
}
```

---

## Scalability Strategy

### 1. Horizontal Scaling (Multiple Agents)

**Current Design**: Support 5 parallel agent executions (tmux limit)

**Implementation**:
```rust
struct AgentPool {
    max_parallel: usize,  // Default: 5
    running: Vec<AgentExecution>,
    queue: VecDeque<AgentTask>,
}

impl AgentPool {
    async fn execute(&mut self, task: AgentTask) -> Result<()> {
        if self.running.len() >= self.max_parallel {
            // Queue for later
            self.queue.push_back(task);
        } else {
            // Execute immediately
            self.spawn_agent(task).await?;
        }
        Ok(())
    }

    async fn on_agent_complete(&mut self, agent_id: AgentId) {
        self.running.retain(|exec| exec.id != agent_id);

        // Start next queued task
        if let Some(task) = self.queue.pop_front() {
            let _ = self.spawn_agent(task).await;
        }
    }
}
```

---

### 2. Caching Strategy

**LSP Response Caching**:
```rust
struct LspCache {
    completions: LruCache<CacheKey, Vec<CompletionItem>>,
    hover_docs: LruCache<Position, Hover>,
}

impl LspCache {
    fn get_completion(&self, key: &CacheKey) -> Option<&Vec<CompletionItem>> {
        self.completions.get(key)
    }

    fn insert_completion(&mut self, key: CacheKey, items: Vec<CompletionItem>) {
        self.completions.put(key, items);
    }

    fn invalidate_buffer(&mut self, buffer_id: BufferId) {
        self.completions.retain(|k, _| k.buffer_id != buffer_id);
        self.hover_docs.clear();
    }
}
```

**Syntax Highlighting Cache**:
```rust
struct SyntaxCache {
    highlighted_lines: HashMap<LineIndex, HighlightedLine>,
}

impl SyntaxCache {
    fn get_line(&self, index: LineIndex) -> Option<&HighlightedLine> {
        self.highlighted_lines.get(&index)
    }

    fn invalidate_range(&mut self, range: Range<LineIndex>) {
        self.highlighted_lines.retain(|k, _| !range.contains(k));
    }
}
```

---

### 3. Load Balancing (Agent Execution)

**Strategy**: Distribute agents across tmux sessions evenly

```rust
struct LoadBalancer {
    sessions: Vec<TmuxSession>,
}

impl LoadBalancer {
    fn select_session(&self) -> &TmuxSession {
        // Round-robin or least-loaded
        self.sessions.iter()
            .min_by_key(|s| s.agent_count)
            .unwrap()
    }
}
```

---

## Success Metrics

### Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Startup Time** | <500ms | Time to render first frame |
| **LSP Completion** | <100ms | Request to display |
| **LSP Goto Definition** | <50ms | Request to jump |
| **File Load (1MB)** | <50ms | Open to first render |
| **File Load (10MB)** | <200ms | Open to first render |
| **File Load (100MB)** | <1s | Open to first render |
| **Memory (Idle)** | <50MB | Resident set size |
| **Memory (10 files)** | <150MB | Resident set size |
| **Memory (LSP active)** | <200MB | Resident set size |
| **Render Frame** | <16ms | 60 FPS target |
| **Agent Spawn** | <200ms | Tmux session creation |

---

## Next Steps

1. **Review this architecture** → Get stakeholder approval
2. **Create detailed component designs** → See `COMPONENT_DESIGN.md`
3. **Define Cargo workspace** → See `CARGO_WORKSPACE.md`
4. **Implement prototype** → Core editor + TUI only (Week 3)
5. **Integrate LSP** → Week 4
6. **Integrate AIT42** → Week 6-7
7. **QA & Optimization** → Week 8
8. **Documentation & Release** → Week 9-10

---

**End of Architecture Document**

Generated by: AIT42 Coordinator
Date: 2025-01-06
Version: 1.0.0
