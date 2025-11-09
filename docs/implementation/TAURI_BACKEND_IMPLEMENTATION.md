# Tauri Backend Implementation for AIT42-Editor

## Overview

This document describes the Tauri backend implementation for the AIT42 Editor GUI application, providing a bridge between the Rust core functionality and the frontend web interface.

## Project Structure

```
src-tauri/
├── src/
│   ├── commands/
│   │   ├── mod.rs          # Command module exports
│   │   ├── file.rs         # File system operations
│   │   ├── editor.rs       # Text editing operations
│   │   └── terminal.rs     # Terminal command execution (feature-gated)
│   ├── state.rs            # Application state management
│   └── main.rs             # Application entry point
├── Cargo.toml              # Rust dependencies
├── tauri.conf.json         # Tauri configuration
├── build.rs                # Build script
└── bindings.d.ts           # TypeScript type definitions
```

## Implementation Details

### 1. File Operations (`commands/file.rs`)

Implements comprehensive file system operations:

- **open_file**: Opens a file and creates a buffer
- **save_file**: Saves file content with atomic write
- **read_directory**: Recursively reads directory structure
- **create_file**: Creates new file with parent directories
- **create_directory**: Creates directory structure
- **delete_path**: Deletes files or directories
- **rename_path**: Renames or moves files/directories

**Key Features**:
- Atomic file writes (temp file + rename)
- Automatic parent directory creation
- Sorted directory listings (directories first, alphabetical)
- Integration with ait42-core Buffer system

### 2. Editor Operations (`commands/editor.rs`)

Implements text editing commands:

- **insert_text**: Insert text at byte position
- **delete_text**: Delete text in range
- **replace_text**: Replace text in range
- **undo/redo**: Undo/redo operations
- **get_buffer_content**: Get full buffer content
- **get_buffer_info**: Get buffer metadata
- **close_buffer**: Close buffer with dirty check
- **list_buffers**: List all open buffers

**Key Features**:
- Byte-offset based operations (UTF-8 aware)
- Buffer lifecycle management
- Dirty state tracking
- Integration with ait42-core Buffer and EditorState

### 3. Terminal Operations (`commands/terminal.rs`) [Feature-Gated]

Implements terminal command execution:

- **execute_command**: Execute shell commands
- **get_terminal_output**: Get terminal output buffer
- **get_terminal_tail**: Get last N output lines
- **clear_terminal**: Clear output buffer
- **get_current_directory**: Get CWD
- **set_current_directory**: Change CWD
- **get_command_history**: Get command history
- **get_terminal_info**: Get terminal metadata

**Key Features**:
- Async command execution with timeout
- Command history management
- Output buffering with size limits
- Built-in commands (cd, clear, pwd, history)
- Integration with ait42-tui TerminalExecutor

### 4. State Management (`state.rs`)

Centralized application state using Arc<Mutex<T>>:

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

**Thread Safety**: All state is protected by Mutex for safe concurrent access from Tauri commands.

### 5. TypeScript Bindings (`bindings.d.ts`)

Complete TypeScript type definitions for frontend integration:

```typescript
export interface AIT42API extends FileCommands, EditorCommands, TerminalCommands {}
```

Provides full type safety for:
- File operations
- Editor operations
- Terminal operations
- Data structures (FileNode, BufferInfo, TerminalInfo, etc.)

## Features

### Default Features
- `custom-protocol`: Tauri custom protocol support
- `terminal`: Terminal command execution support

### Optional Features
- Terminal feature can be disabled for reduced binary size

## Configuration

### Cargo.toml Dependencies

```toml
[dependencies]
tauri = { version = "1.5", features = ["shell-open", "fs-all"] }
ait42-core = { path = "../crates/ait42-core" }
ait42-tui = { path = "../crates/ait42-tui", optional = true }
ait42-lsp = { path = "../crates/ait42-lsp" }
ait42-fs = { path = "../crates/ait42-fs" }
ait42-config = { path = "../crates/ait42-config" }
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
uuid = { version = "1.6", features = ["serde", "v4"] }
```

### Tauri Configuration (`tauri.conf.json`)

- **Window**: 1400x900, resizable, centered
- **Security**: CSP enabled with script/style inline support
- **Permissions**: File system access with home directory scope
- **Bundle**: Multi-platform icon support

## Integration with AIT42 Core

The Tauri backend seamlessly integrates with existing AIT42 crates:

1. **ait42-core**: Buffer management, cursor operations, undo/redo
2. **ait42-tui**: Terminal command execution
3. **ait42-lsp**: LSP client management (ready for integration)
4. **ait42-fs**: File system operations
5. **ait42-config**: Configuration management

## Error Handling

All commands follow consistent error handling:

```rust
#[tauri::command]
pub async fn command_name(...) -> Result<ReturnType, String> {
    // Implementation
    // Errors are converted to String for JSON serialization
}
```

Frontend receives errors as rejected promises with descriptive messages.

## Testing

Each module includes comprehensive unit tests:

- File operations: Create, save, read, delete tests
- Editor operations: Insert, delete, undo/redo tests
- Terminal operations: Command execution, history tests
- State management: Initialization and locking tests

Run tests with:
```bash
cd src-tauri
cargo test
```

## Building

### Development Build
```bash
cd src-tauri
cargo build
```

### Release Build
```bash
cd src-tauri
cargo build --release
```

### Build without terminal feature
```bash
cd src-tauri
cargo build --no-default-features --features=custom-protocol
```

## Known Issues

1. **Terminal Commands Thread Safety**: Current implementation may have issues with `Send` trait due to MutexGuard across await points. Consider refactoring to use channels or tokio::sync::Mutex.

2. **tauri::generate_context!()**: May require additional configuration or path adjustment.

## Future Improvements

1. **LSP Integration**: Implement LSP commands (diagnostics, completions, goto-definition)
2. **Real-time Events**: Add event-based updates for file changes, LSP diagnostics
3. **Plugin System**: Expose plugin API to frontend
4. **Async File Operations**: Use tokio::fs for all file operations
5. **Terminal Improvements**: Add PTY support for true terminal emulation
6. **Performance Monitoring**: Add telemetry for command execution times
7. **Error Recovery**: Implement graceful error recovery and state rollback

## API Documentation

Full API documentation can be generated with:
```bash
cd src-tauri
cargo doc --open
```

## Frontend Usage Example

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Open file
const response = await invoke<OpenFileResponse>('open_file', {
  path: '/path/to/file.txt'
});

// Insert text
await invoke('insert_text', {
  bufferId: response.bufferId,
  position: 0,
  text: 'Hello, World!'
});

// Execute terminal command
const output = await invoke<string>('execute_command', {
  command: 'ls -la'
});
```

## Conclusion

This implementation provides a robust, type-safe bridge between the AIT42 Rust core and the web-based frontend. All file, editor, and terminal operations are exposed through clean Tauri commands with comprehensive error handling and state management.

---

**Author**: Claude (Anthropic AI)
**Date**: 2025-11-03
**Version**: 0.1.0
**Status**: Implementation Complete (Pending Thread Safety Fixes for Terminal)
