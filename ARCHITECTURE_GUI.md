# AIT42 Editor GUI Architecture

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Interface                          │
│                     (React + TypeScript)                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │  Monaco  │  │   File   │  │ Terminal │  │  Agent   │       │
│  │  Editor  │  │   Tree   │  │   UI     │  │  Panel   │       │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              Tauri IPC (JSON-RPC over IPC)               │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ▲ │
                              │ │ Commands / Events
                              │ ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Tauri Backend (Rust)                       │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                      App State                            │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐         │  │
│  │  │   Editor   │  │    LSP     │  │  Terminal  │         │  │
│  │  │  Instance  │  │  Clients   │  │  Executor  │         │  │
│  │  └────────────┘  └────────────┘  └────────────┘         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   Tauri Commands                          │  │
│  │  • File Ops    • Editor Ops    • Terminal Ops            │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ▲ │
                              │ │
                              │ ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Core AIT42 Crates                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌──────────┐ │
│  │  ait42-    │  │  ait42-    │  │  ait42-    │  │  ait42-  │ │
│  │   core     │  │    lsp     │  │    fs      │  │  config  │ │
│  └────────────┘  └────────────┘  └────────────┘  └──────────┘ │
│                                                                  │
│  ┌────────────┐  ┌────────────┐                                │
│  │  ait42-    │  │  ait42-    │                                │
│  │   tui      │  │   ait42    │                                │
│  └────────────┘  └────────────┘                                │
└─────────────────────────────────────────────────────────────────┘
```

## Component Breakdown

### Frontend Layer (React + TypeScript)

#### 1. Monaco Editor Component
- Code editing with syntax highlighting
- LSP integration for completions, diagnostics
- Multi-language support
- Custom themes

**Location**: `src/components/EditorPane.tsx`

#### 2. File Tree Component
- Recursive directory listing
- File icons based on type
- Context menu (create, delete, rename)
- Drag-and-drop support

**Location**: `src/components/FileTree.tsx`

#### 3. Terminal Component
- xterm.js integration
- PTY communication via Tauri
- Multiple terminal tabs
- Scrollback buffer

**Location**: `src/components/Terminal.tsx`

#### 4. Agent Panel
- AI agent chat interface
- Inline code suggestions
- Accept/reject workflow
- Agent status indicator

**Location**: `src/components/AgentPanel.tsx`

### IPC Layer (Tauri Commands)

Commands are organized into groups:

#### File Operations
```rust
open_file(path: String) -> FileContent
save_file(path: String, content: String) -> ()
create_file(path: String) -> ()
create_directory(path: String) -> ()
delete_path(path: String) -> ()
rename_path(old_path: String, new_path: String) -> ()
read_directory(path: String) -> Vec<FileTreeNode>
```

#### Editor Operations
```rust
insert_text(buffer_id: Uuid, position: Position, text: String) -> ()
delete_text(buffer_id: Uuid, range: Range) -> ()
replace_text(buffer_id: Uuid, range: Range, text: String) -> ()
undo(buffer_id: Uuid) -> ()
redo(buffer_id: Uuid) -> ()
get_buffer_content(buffer_id: Uuid) -> String
get_buffer_info(buffer_id: Uuid) -> BufferInfo
close_buffer(buffer_id: Uuid) -> ()
list_buffers() -> Vec<BufferInfo>
```

#### Terminal Operations
```rust
execute_command(command: String) -> CommandResult
get_terminal_output(id: Uuid) -> String
get_terminal_tail(id: Uuid, lines: usize) -> String
clear_terminal() -> ()
get_current_directory() -> PathBuf
set_current_directory(path: PathBuf) -> ()
get_command_history() -> Vec<String>
get_terminal_info() -> TerminalInfo
```

### Backend Layer (Tauri + Rust)

#### Application State
```rust
pub struct AppState {
    pub editor: Arc<Mutex<Editor>>,
    pub editor_state: Arc<Mutex<EditorState>>,
    pub buffer_manager: Mutex<BufferManager>,
    pub config: Mutex<Config>,
    pub lsp_clients: Mutex<Vec<LspClient>>,
    #[cfg(feature = "terminal")]
    pub terminal: Arc<Mutex<TerminalExecutor>>,
}
```

**Features**:
- Thread-safe state management (Arc<Mutex<T>>)
- Undo/redo history
- Buffer lifecycle management
- LSP client pooling
- Terminal session management

### Core Crates Integration

#### ait42-core
- `Editor`: Core editing logic
- `EditorState`: Undo/redo, mode management
- `BufferManager`: Buffer lifecycle
- `TextBuffer`: Rope-based text storage

#### ait42-lsp
- `LspClient`: Language server communication
- Protocol implementation (LSP 3.17)
- Diagnostics, completions, hover, goto definition

#### ait42-fs
- File watching
- Directory traversal
- Search and indexing

#### ait42-config
- Configuration management
- User preferences
- Theme definitions

#### ait42-tui (optional)
- `TerminalExecutor`: Shell command execution
- PTY management
- Command history

## Data Flow

### Opening a File

```
1. User clicks file in File Tree
   ↓
2. FileTree.tsx calls tauriCommands.openFile(path)
   ↓
3. Tauri IPC invokes open_file command
   ↓
4. Backend reads file via ait42-fs
   ↓
5. Creates buffer in Editor
   ↓
6. Returns FileContent { path, content, language }
   ↓
7. EditorPane displays content in Monaco
```

### LSP Integration

```
1. User types in Monaco Editor
   ↓
2. Monaco triggers completion request
   ↓
3. Frontend calls tauriCommands.getCompletions(path, line, char)
   ↓
4. Backend forwards request to LSP client
   ↓
5. LSP server (e.g., rust-analyzer) processes request
   ↓
6. Returns completion items
   ↓
7. Monaco displays completion list
```

### Terminal Execution

```
1. User types command in Terminal UI
   ↓
2. Terminal.tsx calls tauriCommands.executeCommand(cmd)
   ↓
3. Backend spawns PTY process via TerminalExecutor
   ↓
4. Streams output back to frontend
   ↓
5. xterm.js renders output in terminal
```

## Security Model

### Content Security Policy
```
default-src 'self'
script-src 'self' 'unsafe-inline'
style-src 'self' 'unsafe-inline'
img-src 'self' data: https:
font-src 'self' data:
```

### File System Access
- Scoped to `$HOME/**` via Tauri allowlist
- All file operations validated
- No arbitrary file system access from frontend

### IPC Security
- All commands explicitly whitelisted
- Type validation via serde
- No dynamic code execution

## Performance Considerations

### Virtual Scrolling
- Large files handled via Monaco's built-in virtualization
- File tree lazy-loads directories

### Worker Threads
- File search runs in background thread
- LSP communication is async
- Terminal I/O is non-blocking

### Caching
- LSP diagnostics cached per file
- File tree cached with file watcher updates
- Configuration cached in memory

## Build Targets

### Development
- Frontend: Vite dev server (hot reload)
- Backend: Debug build (fast compilation)
- DevTools enabled

### Production
- Frontend: Optimized bundle (tree-shaking)
- Backend: Release build (LTO, stripped)
- Single binary per platform

### Platforms
- **macOS**: .app bundle + .dmg installer
- **Windows**: .exe + .msi installer
- **Linux**: .deb, .AppImage

## Configuration

### User Settings
```json
{
  "editor": {
    "fontSize": 14,
    "tabSize": 4,
    "insertSpaces": true,
    "theme": "vs-dark"
  },
  "terminal": {
    "shell": "/bin/zsh",
    "fontSize": 12
  },
  "lsp": {
    "rust": {
      "server": "rust-analyzer",
      "enabled": true
    }
  }
}
```

**Location**: `~/.config/ait42/config.json`

## Extension Points

### Future Plugin System
- Plugin API for custom commands
- Language server registration
- Custom themes and snippets
- UI extensions (panels, menus)

## Metrics

- **Frontend Bundle Size**: ~500KB (gzipped)
- **Backend Binary Size**: ~10MB (release, stripped)
- **Memory Usage**: ~50MB (idle), ~200MB (active)
- **Startup Time**: <1s (macOS M1)

## References

- [Tauri Architecture](https://tauri.app/v1/concepts/architecture)
- [Monaco Editor Architecture](https://microsoft.github.io/monaco-editor/docs.html)
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
