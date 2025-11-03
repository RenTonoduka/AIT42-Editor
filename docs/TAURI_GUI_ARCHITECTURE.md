# AIT42-Editor Tauri GUI Architecture

## Executive Summary

This document outlines the comprehensive architecture for converting AIT42-Editor from a Terminal UI (TUI) application to a modern Tauri-based GUI application. The conversion will preserve the existing core editor logic (`ait42-core`) while replacing the Ratatui-based TUI with a React + TypeScript frontend.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Directory Structure](#directory-structure)
3. [System Architecture](#system-architecture)
4. [Component Mapping](#component-mapping)
5. [Tauri Bridge API](#tauri-bridge-api)
6. [Frontend Architecture](#frontend-architecture)
7. [Theme System](#theme-system)
8. [Migration Strategy](#migration-strategy)
9. [Implementation Phases](#implementation-phases)
10. [Testing Strategy](#testing-strategy)

---

## Architecture Overview

### Design Principles

- **Preserve Core Logic**: `ait42-core` remains unchanged
- **Clean Separation**: Frontend (React) communicates with backend (Rust) via Tauri commands
- **Type Safety**: End-to-end TypeScript/Rust type definitions
- **Performance**: Leverage WebView GPU acceleration for rendering
- **Extensibility**: Plugin architecture for future extensions

### Technology Stack

**Backend (Rust):**
- Tauri 1.5+ (application framework)
- ait42-core (existing editor logic)
- ait42-lsp (LSP client)
- tokio (async runtime)

**Frontend (JavaScript):**
- React 18+ (UI framework)
- TypeScript 5+ (type safety)
- Monaco Editor (code editor component)
- xterm.js (terminal emulator)
- Zustand (state management)
- TailwindCSS (styling)
- Vite (build tool)

---

## Directory Structure

```
AIT42-Editor/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── ait42-core/              # [UNCHANGED] Core editor logic
│   ├── ait42-lsp/               # [UNCHANGED] LSP client
│   ├── ait42-config/            # [UNCHANGED] Configuration
│   ├── ait42-fs/                # [UNCHANGED] File system operations
│   ├── ait42-ait42/             # [UNCHANGED] AI agent integration
│   ├── ait42-tui/               # [DEPRECATED] Keep for reference
│   └── ait42-tauri/             # [NEW] Tauri backend
│       ├── Cargo.toml
│       ├── tauri.conf.json      # Tauri configuration
│       ├── icons/               # App icons
│       ├── src/
│       │   ├── main.rs          # Tauri entry point
│       │   ├── commands/        # Tauri command handlers
│       │   │   ├── mod.rs
│       │   │   ├── editor.rs    # Editor operations
│       │   │   ├── file.rs      # File operations
│       │   │   ├── terminal.rs  # Terminal operations
│       │   │   └── lsp.rs       # LSP operations
│       │   ├── state.rs         # Application state
│       │   ├── events.rs        # Event emission
│       │   └── bridge.rs        # Frontend-backend bridge
│       └── capabilities/        # Tauri capabilities/permissions
│
├── frontend/                     # [NEW] React frontend
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   ├── index.html
│   ├── public/                  # Static assets
│   └── src/
│       ├── main.tsx             # Entry point
│       ├── App.tsx              # Root component
│       ├── vite-env.d.ts
│       │
│       ├── components/          # React components
│       │   ├── Editor/
│       │   │   ├── Editor.tsx           # Monaco editor wrapper
│       │   │   ├── EditorContainer.tsx  # Editor with tab context
│       │   │   └── LineNumbers.tsx
│       │   ├── TabBar/
│       │   │   ├── TabBar.tsx           # Tab bar
│       │   │   ├── Tab.tsx              # Single tab
│       │   │   └── TabCloseButton.tsx
│       │   ├── Sidebar/
│       │   │   ├── Sidebar.tsx          # File explorer
│       │   │   ├── FileTree.tsx         # Recursive tree
│       │   │   ├── FileItem.tsx         # File/folder item
│       │   │   └── FileIcon.tsx         # File type icons
│       │   ├── Terminal/
│       │   │   ├── Terminal.tsx         # xterm.js wrapper
│       │   │   └── TerminalInput.tsx    # Input bar
│       │   ├── StatusBar/
│       │   │   ├── StatusBar.tsx        # Bottom status bar
│       │   │   ├── CursorPosition.tsx
│       │   │   ├── FileInfo.tsx
│       │   │   └── AgentStatus.tsx
│       │   ├── CommandPalette/
│       │   │   └── CommandPalette.tsx   # Ctrl+P command palette
│       │   └── Layout/
│       │       ├── AppLayout.tsx        # Main layout
│       │       ├── SplitView.tsx        # Resizable panels
│       │       └── Panel.tsx
│       │
│       ├── hooks/               # Custom React hooks
│       │   ├── useEditor.ts     # Editor state management
│       │   ├── useFiles.ts      # File operations
│       │   ├── useTerminal.ts   # Terminal state
│       │   ├── useLsp.ts        # LSP integration
│       │   ├── useKeyBindings.ts # Keyboard shortcuts
│       │   └── useTheme.ts      # Theme management
│       │
│       ├── store/               # Zustand stores
│       │   ├── editorStore.ts   # Editor state
│       │   ├── tabStore.ts      # Tab management
│       │   ├── terminalStore.ts # Terminal state
│       │   ├── sidebarStore.ts  # Sidebar state
│       │   └── settingsStore.ts # User settings
│       │
│       ├── api/                 # Tauri API wrappers
│       │   ├── editor.ts        # Editor commands
│       │   ├── file.ts          # File operations
│       │   ├── terminal.ts      # Terminal commands
│       │   ├── lsp.ts           # LSP commands
│       │   └── types.ts         # TypeScript types
│       │
│       ├── styles/              # Global styles
│       │   ├── globals.css      # Global CSS
│       │   ├── editor.css       # Editor-specific styles
│       │   ├── terminal.css     # Terminal styles
│       │   └── themes/          # Theme definitions
│       │       ├── cursor-dark.ts   # Cursor dark theme
│       │       └── theme.ts         # Theme interface
│       │
│       └── utils/               # Utility functions
│           ├── colors.ts        # Color utilities
│           ├── formatting.ts    # Text formatting
│           └── shortcuts.ts     # Keyboard shortcuts
│
├── docs/
│   ├── TAURI_GUI_ARCHITECTURE.md  # This document
│   ├── API_REFERENCE.md           # Tauri command API
│   └── COMPONENT_GUIDE.md         # Frontend component guide
│
└── README.md
```

---

## System Architecture

### High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        Tauri Application                     │
│                                                              │
│  ┌────────────────────────┐      ┌─────────────────────┐   │
│  │   Frontend (WebView)   │      │   Backend (Rust)    │   │
│  │                        │      │                      │   │
│  │  ┌──────────────────┐  │      │  ┌───────────────┐  │   │
│  │  │   React App      │  │◄────►│  │ Tauri Core    │  │   │
│  │  │                  │  │ IPC  │  │               │  │   │
│  │  │  - Monaco Editor │  │      │  │ - Commands    │  │   │
│  │  │  - xterm.js      │  │      │  │ - Events      │  │   │
│  │  │  - Components    │  │      │  │ - State       │  │   │
│  │  └──────────────────┘  │      │  └───────┬───────┘  │   │
│  │                        │      │          │          │   │
│  │  ┌──────────────────┐  │      │  ┌───────▼───────┐  │   │
│  │  │   Zustand Store  │  │      │  │  ait42-core   │  │   │
│  │  │                  │  │      │  │               │  │   │
│  │  │  - Editor State  │  │      │  │  - Buffer     │  │   │
│  │  │  - Tab State     │  │      │  │  - Cursor     │  │   │
│  │  │  - Terminal      │  │      │  │  - Command    │  │   │
│  │  └──────────────────┘  │      │  └───────┬───────┘  │   │
│  │                        │      │          │          │   │
│  └────────────────────────┘      │  ┌───────▼───────┐  │   │
│                                   │  │  ait42-lsp    │  │   │
│                                   │  │               │  │   │
│                                   │  │  - LSP Client │  │   │
│                                   │  │  - Diagnostics│  │   │
│                                   │  └───────────────┘  │   │
│                                   │                      │   │
│                                   │  ┌───────────────┐  │   │
│                                   │  │ Terminal Exec │  │   │
│                                   │  │               │  │   │
│                                   │  │  - Command    │  │   │
│                                   │  │  - Output     │  │   │
│                                   │  └───────────────┘  │   │
│                                   └─────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Communication Flow

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│   Frontend   │  IPC    │    Tauri     │  Fn     │   ait42-     │
│   (React)    │────────►│   Commands   │────────►│     core     │
│              │         │              │         │              │
│  Monaco      │◄────────│   Events     │◄────────│   Buffer     │
│  Editor      │  emit   │              │  notify │   Manager    │
└──────────────┘         └──────────────┘         └──────────────┘
```

---

## Component Mapping

### TUI to GUI Component Mapping

| TUI Component (Ratatui) | GUI Component (React) | Technology |
|------------------------|----------------------|------------|
| `EditorWidget` | `<Editor>` | Monaco Editor |
| `TabBar` | `<TabBar>` | React Component |
| `Sidebar` | `<Sidebar>` | React Component + react-arborist |
| `TerminalPanel` | `<Terminal>` | xterm.js |
| `StatusLine` | `<StatusBar>` | React Component |
| `CommandPalette` | `<CommandPalette>` | React Component |
| `EditorState` | Zustand Store | State Management |
| `TerminalExecutor` | Backend Service | Rust (unchanged) |
| `CursorTheme` | CSS Variables | TailwindCSS |

### Visual Comparison

**TUI Layout:**
```
┌─────────────────────────────────────────────────────────┐
│ Tab1 │ Tab2 │ Tab3                                  [×] │ TabBar
├───────┬─────────────────────────────────────────────────┤
│ 📁    │ src/main.rs                               1:1 │
│ > src │                                                 │
│   main│ fn main() {                                     │
│   lib │     println!("Hello");                          │
│ 📄 doc│ }                                               │ Editor
├───────┴─────────────────────────────────────────────────┤
│ $ cargo build                                          │
│ Compiling ait42 v0.1.0                                 │ Terminal
│ Finished dev [unoptimized] target(s) in 2.15s          │
└─────────────────────────────────────────────────────────┘
│ NORMAL | UTF-8 | Rust | Ln 1, Col 1 | Agent: Idle    │ StatusBar
└─────────────────────────────────────────────────────────┘
```

**GUI Layout (Tauri):**
```
┌─────────────────────────────────────────────────────────┐
│ ≡  Tab1 ×  Tab2 ×  Tab3 ×  [+]                    [□][×]│ Title Bar
├───────┬─────────────────────────────────────────────────┤
│ 📁 src│ src/main.rs                               ●  1:1│
│  › 📄 │                                                 │
│  › 📄 │ fn main() {                                     │
│ 📁 doc│     println!("Hello");                          │
│  › 📄 │ }                                               │ Monaco
│       │                                                 │ Editor
├───────┴─────────────────────────────────────────────────┤
│ ▶ cargo build                                    [↻][×]│ Terminal
│ Compiling ait42 v0.1.0                                 │ (xterm.js)
│ Finished dev [unoptimized] target(s) in 2.15s          │
└─────────────────────────────────────────────────────────┘
│ Rust • UTF-8 • Ln 1, Col 1 • 100% •   Agent: Idle   │ Status Bar
└─────────────────────────────────────────────────────────┘
```

---

## Tauri Bridge API

### Tauri Commands (Rust → Frontend)

#### File Operations

```rust
// crates/ait42-tauri/src/commands/file.rs

#[tauri::command]
pub async fn open_file(path: String, state: State<'_, AppState>) -> Result<FileData, String> {
    // Open file using ait42-core
    // Returns file content, metadata
}

#[tauri::command]
pub async fn save_file(
    buffer_id: String,
    path: Option<String>,
    state: State<'_, AppState>
) -> Result<(), String> {
    // Save buffer to file
}

#[tauri::command]
pub async fn close_file(buffer_id: String, state: State<'_, AppState>) -> Result<(), String> {
    // Close buffer
}

#[tauri::command]
pub async fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    // List directory contents
}

#[tauri::command]
pub async fn watch_file(path: String, state: State<'_, AppState>) -> Result<(), String> {
    // Start watching file for external changes
}
```

#### Editor Operations

```rust
// crates/ait42-tauri/src/commands/editor.rs

#[tauri::command]
pub async fn insert_text(
    buffer_id: String,
    position: usize,
    text: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    // Insert text at position
}

#[tauri::command]
pub async fn delete_text(
    buffer_id: String,
    start: usize,
    end: usize,
    state: State<'_, AppState>
) -> Result<(), String> {
    // Delete text range
}

#[tauri::command]
pub async fn undo(buffer_id: String, state: State<'_, AppState>) -> Result<(), String> {
    // Undo last operation
}

#[tauri::command]
pub async fn redo(buffer_id: String, state: State<'_, AppState>) -> Result<(), String> {
    // Redo operation
}

#[tauri::command]
pub async fn get_buffer_content(
    buffer_id: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    // Get full buffer content
}
```

#### Terminal Operations

```rust
// crates/ait42-tauri/src/commands/terminal.rs

#[tauri::command]
pub async fn execute_command(
    command: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    // Execute terminal command
}

#[tauri::command]
pub async fn get_terminal_output(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    // Get terminal output buffer
}

#[tauri::command]
pub async fn clear_terminal(state: State<'_, AppState>) -> Result<(), String> {
    // Clear terminal output
}

#[tauri::command]
pub async fn kill_terminal_process(state: State<'_, AppState>) -> Result<(), String> {
    // Kill running terminal process
}
```

#### LSP Operations

```rust
// crates/ait42-tauri/src/commands/lsp.rs

#[tauri::command]
pub async fn initialize_lsp(
    language: String,
    root_path: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    // Initialize LSP server
}

#[tauri::command]
pub async fn get_completions(
    buffer_id: String,
    line: usize,
    column: usize,
    state: State<'_, AppState>
) -> Result<Vec<CompletionItem>, String> {
    // Get code completions
}

#[tauri::command]
pub async fn get_diagnostics(
    buffer_id: String,
    state: State<'_, AppState>
) -> Result<Vec<Diagnostic>, String> {
    // Get diagnostics (errors, warnings)
}

#[tauri::command]
pub async fn goto_definition(
    buffer_id: String,
    line: usize,
    column: usize,
    state: State<'_, AppState>
) -> Result<Location, String> {
    // Go to definition
}
```

### Events (Backend → Frontend)

```rust
// crates/ait42-tauri/src/events.rs

pub enum EditorEvent {
    FileChanged { buffer_id: String, content: String },
    FileSaved { buffer_id: String, path: String },
    FileExternallyModified { path: String },
    BufferDirty { buffer_id: String, is_dirty: bool },

    TerminalOutput { lines: Vec<String> },
    TerminalProcessExit { exit_code: i32 },

    LspDiagnostics { buffer_id: String, diagnostics: Vec<Diagnostic> },
    LspInitialized { language: String },

    AgentStatusChanged { status: AgentStatus },
    AgentMessage { message: String },
}

// Emit events to frontend
pub fn emit_event(app: &AppHandle, event: EditorEvent) {
    app.emit_all("editor-event", event).unwrap();
}
```

### TypeScript API Wrappers

```typescript
// frontend/src/api/editor.ts

import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

export interface FileData {
  content: string;
  path: string;
  language: string;
  encoding: string;
}

export async function openFile(path: string): Promise<FileData> {
  return await invoke('open_file', { path });
}

export async function saveFile(
  bufferId: string,
  path?: string
): Promise<void> {
  return await invoke('save_file', { bufferId, path });
}

export async function insertText(
  bufferId: string,
  position: number,
  text: string
): Promise<void> {
  return await invoke('insert_text', { bufferId, position, text });
}

export function onFileChanged(
  callback: (event: FileChangedEvent) => void
) {
  return listen('editor-event', (event) => {
    if (event.payload.type === 'FileChanged') {
      callback(event.payload);
    }
  });
}
```

---

## Frontend Architecture

### Component Hierarchy

```
<App>
  ├── <AppLayout>
  │   ├── <TitleBar>
  │   ├── <TabBar>
  │   │   └── <Tab> (multiple)
  │   ├── <SplitView orientation="horizontal">
  │   │   ├── <Sidebar>
  │   │   │   └── <FileTree>
  │   │   │       └── <FileItem> (recursive)
  │   │   └── <SplitView orientation="vertical">
  │   │       ├── <EditorContainer>
  │   │       │   └── <Editor> (Monaco)
  │   │       └── <Terminal> (xterm.js)
  │   ├── <StatusBar>
  │   │   ├── <CursorPosition>
  │   │   ├── <FileInfo>
  │   │   └── <AgentStatus>
  │   └── <CommandPalette>
  └── <ThemeProvider>
```

### State Management (Zustand)

```typescript
// frontend/src/store/editorStore.ts

import create from 'zustand';

interface EditorState {
  // Active buffer
  activeBufferId: string | null;

  // Buffer map
  buffers: Map<string, BufferState>;

  // Cursor positions per buffer
  cursors: Map<string, CursorPosition>;

  // Actions
  setActiveBuffer: (id: string) => void;
  updateBuffer: (id: string, content: string) => void;
  setCursor: (id: string, position: CursorPosition) => void;
}

export const useEditorStore = create<EditorState>((set) => ({
  activeBufferId: null,
  buffers: new Map(),
  cursors: new Map(),

  setActiveBuffer: (id) => set({ activeBufferId: id }),

  updateBuffer: (id, content) => set((state) => {
    const newBuffers = new Map(state.buffers);
    newBuffers.set(id, { ...newBuffers.get(id), content });
    return { buffers: newBuffers };
  }),

  setCursor: (id, position) => set((state) => {
    const newCursors = new Map(state.cursors);
    newCursors.set(id, position);
    return { cursors: newCursors };
  }),
}));
```

```typescript
// frontend/src/store/tabStore.ts

interface TabState {
  tabs: Tab[];
  activeTabIndex: number;

  addTab: (tab: Tab) => void;
  closeTab: (index: number) => void;
  setActiveTab: (index: number) => void;
}

export const useTabStore = create<TabState>((set) => ({
  tabs: [],
  activeTabIndex: 0,

  addTab: (tab) => set((state) => ({
    tabs: [...state.tabs, tab],
    activeTabIndex: state.tabs.length
  })),

  closeTab: (index) => set((state) => ({
    tabs: state.tabs.filter((_, i) => i !== index),
    activeTabIndex: Math.max(0, state.activeTabIndex - 1)
  })),

  setActiveTab: (index) => set({ activeTabIndex: index }),
}));
```

### Key Components

#### Editor Component (Monaco)

```typescript
// frontend/src/components/Editor/Editor.tsx

import React, { useRef, useEffect } from 'react';
import * as monaco from 'monaco-editor';
import { useEditorStore } from '../../store/editorStore';
import { insertText, deleteText } from '../../api/editor';

export const Editor: React.FC<{ bufferId: string }> = ({ bufferId }) => {
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const buffer = useEditorStore(state => state.buffers.get(bufferId));

  useEffect(() => {
    if (!containerRef.current) return;

    // Initialize Monaco Editor
    editorRef.current = monaco.editor.create(containerRef.current, {
      value: buffer?.content || '',
      language: buffer?.language || 'plaintext',
      theme: 'cursor-dark',
      automaticLayout: true,
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
    });

    // Listen for content changes
    editorRef.current.onDidChangeModelContent((e) => {
      const changes = e.changes;
      for (const change of changes) {
        if (change.text) {
          // Insert
          insertText(bufferId, change.rangeOffset, change.text);
        } else {
          // Delete
          deleteText(bufferId, change.rangeOffset, change.rangeLength);
        }
      }
    });

    return () => editorRef.current?.dispose();
  }, [bufferId]);

  // Update editor content when buffer changes
  useEffect(() => {
    if (editorRef.current && buffer) {
      const model = editorRef.current.getModel();
      if (model && model.getValue() !== buffer.content) {
        model.setValue(buffer.content);
      }
    }
  }, [buffer?.content]);

  return <div ref={containerRef} className="w-full h-full" />;
};
```

#### Terminal Component (xterm.js)

```typescript
// frontend/src/components/Terminal/Terminal.tsx

import React, { useRef, useEffect } from 'react';
import { Terminal as XTerm } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { executeCommand, onTerminalOutput } from '../../api/terminal';
import 'xterm/css/xterm.css';

export const Terminal: React.FC = () => {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const inputBuffer = useRef<string>('');

  useEffect(() => {
    if (!terminalRef.current) return;

    // Initialize xterm.js
    const xterm = new XTerm({
      theme: {
        background: '#1A1A1A',
        foreground: '#CCCCCC',
        cursor: '#FFFFFF',
      },
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      cursorBlink: true,
    });

    const fitAddon = new FitAddon();
    xterm.loadAddon(fitAddon);

    xterm.open(terminalRef.current);
    fitAddon.fit();

    xtermRef.current = xterm;
    fitAddonRef.current = fitAddon;

    // Handle user input
    xterm.onData((data) => {
      if (data === '\r') {
        // Enter pressed
        xterm.write('\r\n');
        executeCommand(inputBuffer.current);
        inputBuffer.current = '';
      } else if (data === '\x7F') {
        // Backspace
        if (inputBuffer.current.length > 0) {
          inputBuffer.current = inputBuffer.current.slice(0, -1);
          xterm.write('\b \b');
        }
      } else {
        // Regular character
        inputBuffer.current += data;
        xterm.write(data);
      }
    });

    // Listen for terminal output from backend
    const unlisten = onTerminalOutput((lines) => {
      for (const line of lines) {
        xterm.writeln(line);
      }
    });

    return () => {
      unlisten.then(fn => fn());
      xterm.dispose();
    };
  }, []);

  return <div ref={terminalRef} className="w-full h-full" />;
};
```

#### File Tree Component

```typescript
// frontend/src/components/Sidebar/FileTree.tsx

import React from 'react';
import { useFileTreeStore } from '../../store/fileTreeStore';
import { FileItem } from './FileItem';
import { openFile } from '../../api/file';

export const FileTree: React.FC = () => {
  const { rootPath, tree, expandedDirs } = useFileTreeStore();

  const handleFileClick = async (path: string) => {
    const fileData = await openFile(path);
    // Add tab and open in editor
  };

  return (
    <div className="file-tree">
      {tree.map(item => (
        <FileItem
          key={item.path}
          item={item}
          level={0}
          onFileClick={handleFileClick}
        />
      ))}
    </div>
  );
};
```

---

## Theme System

### Cursor Theme Translation

The existing Cursor theme from `crates/ait42-tui/src/themes/cursor.rs` will be translated to CSS variables and Monaco theme.

#### CSS Variables

```css
/* frontend/src/styles/themes/cursor-dark.css */

:root[data-theme="cursor-dark"] {
  /* Base Colors */
  --bg-primary: #1E1E1E;
  --bg-secondary: #252525;
  --bg-deep: #1A1A1A;
  --bg-elevated: #2D2D2D;

  --fg-primary: #CCCCCC;
  --fg-secondary: #858585;
  --fg-tertiary: #606060;

  /* Accent */
  --accent-primary: #007ACC;
  --accent-hover: #148EE0;

  /* Borders */
  --border-primary: #3E3E42;
  --border-active: #007ACC;
  --border-inactive: #333333;

  /* Semantic */
  --color-success: #10B981;
  --color-info: #3B82F6;
  --color-warning: #F59E0B;
  --color-error: #EF4444;

  /* Syntax */
  --syntax-keyword: #C586C0;
  --syntax-type: #4EC9B0;
  --syntax-function: #DCDCAA;
  --syntax-string: #CE9178;
  --syntax-number: #B5CEA8;
  --syntax-comment: #6A9955;
  --syntax-variable: #9CDCFE;
}
```

#### Monaco Editor Theme

```typescript
// frontend/src/styles/themes/cursor-dark.ts

import * as monaco from 'monaco-editor';

export const cursorDarkTheme: monaco.editor.IStandaloneThemeData = {
  base: 'vs-dark',
  inherit: true,
  rules: [
    { token: 'comment', foreground: '6A9955' },
    { token: 'keyword', foreground: 'C586C0' },
    { token: 'type', foreground: '4EC9B0' },
    { token: 'function', foreground: 'DCDCAA' },
    { token: 'string', foreground: 'CE9178' },
    { token: 'number', foreground: 'B5CEA8' },
    { token: 'variable', foreground: '9CDCFE' },
    { token: 'operator', foreground: 'D4D4D4' },
  ],
  colors: {
    'editor.background': '#1E1E1E',
    'editor.foreground': '#CCCCCC',
    'editor.lineHighlightBackground': '#2D2D2D',
    'editorCursor.foreground': '#FFFFFF',
    'editor.selectionBackground': '#264F78',
    'editor.inactiveSelectionBackground': '#3A3D41',
    'editorLineNumber.foreground': '#858585',
    'editorLineNumber.activeForeground': '#CCCCCC',
  }
};

// Register theme
monaco.editor.defineTheme('cursor-dark', cursorDarkTheme);
```

---

## Migration Strategy

### Phase 0: Preparation

**Goal**: Set up Tauri infrastructure without affecting TUI

**Tasks**:
1. Create `crates/ait42-tauri/` crate
2. Add Tauri dependencies to workspace
3. Initialize frontend project with Vite + React
4. Set up basic Tauri window
5. Configure build system

**Deliverables**:
- Empty Tauri app that launches
- React dev server running
- No TUI changes

---

### Phase 1: File Operations

**Goal**: Basic file open/save through Tauri

**Backend**:
1. Implement `open_file` command
2. Implement `save_file` command
3. Implement `list_directory` command
4. Connect to `ait42-core` Buffer

**Frontend**:
1. Create basic file tree component
2. Implement Monaco Editor integration
3. Wire up file open/save
4. Add single tab support

**Success Criteria**:
- Can open file from sidebar
- Can edit in Monaco
- Can save changes
- Single buffer works end-to-end

---

### Phase 2: Multi-Tab & Editor Operations

**Goal**: Full editor functionality with tabs

**Backend**:
1. Implement `insert_text`, `delete_text` commands
2. Implement `undo`, `redo` commands
3. Add buffer state management
4. Implement cursor position sync

**Frontend**:
1. Create TabBar component
2. Implement tab switching
3. Add tab close functionality
4. Multiple editors with state preservation

**Success Criteria**:
- Can open multiple files in tabs
- Tab switching works
- Undo/redo works
- Cursor position persists per tab

---

### Phase 3: Terminal Integration

**Goal**: Embedded terminal with command execution

**Backend**:
1. Integrate `TerminalExecutor` from TUI
2. Implement `execute_command` command
3. Implement output streaming via events
4. Add process management

**Frontend**:
1. Integrate xterm.js
2. Implement terminal input handling
3. Display command output
4. Add terminal panel resizing

**Success Criteria**:
- Can execute shell commands
- Output displays in real-time
- Terminal history works
- Can resize terminal panel

---

### Phase 4: LSP Integration

**Goal**: Code intelligence features

**Backend**:
1. Connect `ait42-lsp` to Tauri
2. Implement completion command
3. Implement diagnostics command
4. Add go-to-definition

**Frontend**:
1. Configure Monaco LSP integration
2. Display completions
3. Show diagnostics (errors/warnings)
4. Implement go-to-definition navigation

**Success Criteria**:
- Autocomplete works for Rust
- Errors show in editor
- Can jump to definitions
- Hover info displays

---

### Phase 5: UI Polish & Theme

**Goal**: Professional UI matching Cursor theme

**Tasks**:
1. Implement Cursor dark theme
2. Add status bar
3. Add command palette (Ctrl+P)
4. Implement keyboard shortcuts
5. Add loading states
6. Error handling UI

**Success Criteria**:
- UI matches Cursor aesthetic
- All keyboard shortcuts work
- Smooth animations
- Responsive layout

---

### Phase 6: Advanced Features

**Goal**: Feature parity with TUI + extras

**Tasks**:
1. File watchers (external changes)
2. Settings panel
3. Split editor views
4. Search & replace
5. Git integration (optional)
6. Plugin system (future)

---

## Implementation Phases

### Timeline Estimate

| Phase | Duration | Dependencies | Risk Level |
|-------|----------|--------------|------------|
| Phase 0: Preparation | 2-3 days | None | Low |
| Phase 1: File Operations | 5-7 days | Phase 0 | Medium |
| Phase 2: Multi-Tab | 4-5 days | Phase 1 | Medium |
| Phase 3: Terminal | 3-4 days | Phase 1 | Low |
| Phase 4: LSP | 7-10 days | Phase 1, 2 | High |
| Phase 5: UI Polish | 5-7 days | All previous | Low |
| Phase 6: Advanced | 10-14 days | All previous | Medium |

**Total Estimated Time**: 6-8 weeks

---

## Testing Strategy

### Backend Testing

```rust
// crates/ait42-tauri/tests/commands_test.rs

#[tokio::test]
async fn test_open_file_command() {
    let state = create_test_state();
    let result = open_file("test.txt".to_string(), state).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_insert_text_command() {
    // Test insert operation
}
```

### Frontend Testing

```typescript
// frontend/src/components/Editor/Editor.test.tsx

import { render, screen } from '@testing-library/react';
import { Editor } from './Editor';

describe('Editor', () => {
  it('renders Monaco editor', () => {
    render(<Editor bufferId="test" />);
    expect(screen.getByRole('textbox')).toBeInTheDocument();
  });

  it('syncs content with backend', async () => {
    // Test content synchronization
  });
});
```

### E2E Testing

```typescript
// e2e/basic-workflow.spec.ts

import { test, expect } from '@playwright/test';

test('open file and edit', async ({ page }) => {
  await page.goto('/');

  // Click file in sidebar
  await page.click('[data-testid="file-item-main.rs"]');

  // Edit content
  const editor = page.locator('.monaco-editor');
  await editor.type('// New comment');

  // Save file
  await page.keyboard.press('Control+S');

  // Verify saved
  await expect(page.locator('[data-testid="tab-modified"]')).not.toBeVisible();
});
```

---

## Migration Checklist

### Pre-Migration

- [ ] Backup current TUI codebase
- [ ] Document all TUI features
- [ ] Review ait42-core API
- [ ] Set up Tauri development environment
- [ ] Create frontend boilerplate

### Phase 0: Infrastructure

- [ ] Create `ait42-tauri` crate
- [ ] Configure Cargo.toml
- [ ] Set up tauri.conf.json
- [ ] Initialize React + Vite project
- [ ] Configure TailwindCSS
- [ ] Set up Monaco Editor
- [ ] Set up xterm.js
- [ ] Test basic Tauri window launch

### Phase 1: File Operations

- [ ] Implement `open_file` command
- [ ] Implement `save_file` command
- [ ] Implement `close_file` command
- [ ] Implement `list_directory` command
- [ ] Create FileTree component
- [ ] Create Editor component (Monaco)
- [ ] Wire up file open flow
- [ ] Wire up file save flow
- [ ] Test file operations end-to-end

### Phase 2: Multi-Tab Editor

- [ ] Implement TabBar component
- [ ] Implement Tab component
- [ ] Create tab store (Zustand)
- [ ] Implement tab switching
- [ ] Implement tab close
- [ ] Implement `insert_text` command
- [ ] Implement `delete_text` command
- [ ] Implement `undo` command
- [ ] Implement `redo` command
- [ ] Test multi-tab workflow

### Phase 3: Terminal

- [ ] Port TerminalExecutor to Tauri
- [ ] Implement `execute_command` command
- [ ] Implement `get_terminal_output` command
- [ ] Implement `clear_terminal` command
- [ ] Create Terminal component (xterm.js)
- [ ] Implement terminal input handling
- [ ] Implement output display
- [ ] Add terminal resize
- [ ] Test command execution

### Phase 4: LSP Integration

- [ ] Connect ait42-lsp to Tauri
- [ ] Implement `initialize_lsp` command
- [ ] Implement `get_completions` command
- [ ] Implement `get_diagnostics` command
- [ ] Implement `goto_definition` command
- [ ] Configure Monaco LSP client
- [ ] Test Rust language support
- [ ] Add diagnostic display
- [ ] Test go-to-definition

### Phase 5: Theme & Polish

- [ ] Convert Cursor theme to CSS
- [ ] Create Monaco theme
- [ ] Implement StatusBar component
- [ ] Implement CommandPalette
- [ ] Add keyboard shortcuts
- [ ] Implement loading states
- [ ] Add error boundaries
- [ ] Polish animations
- [ ] Test responsive layout

### Phase 6: Advanced Features

- [ ] Implement file watchers
- [ ] Create settings panel
- [ ] Add split editor views
- [ ] Implement search & replace
- [ ] Add Git integration (optional)
- [ ] Document plugin API

### Post-Migration

- [ ] Performance testing
- [ ] Memory leak testing
- [ ] Cross-platform testing (macOS, Linux, Windows)
- [ ] Update documentation
- [ ] Create migration guide
- [ ] Archive TUI codebase
- [ ] Release beta version

---

## Performance Considerations

### Backend

- Use tokio for async operations
- Implement debouncing for frequent operations (e.g., typing)
- Buffer size limits to prevent memory exhaustion
- Lazy loading for large files
- Incremental LSP updates

### Frontend

- Virtual scrolling for file tree (large directories)
- Monaco Web Workers for syntax highlighting
- Debounced Tauri command calls
- Memoized React components
- Code splitting for lazy loading

### IPC Optimization

- Batch multiple operations into single command
- Use events for real-time updates (not polling)
- Minimize payload size (send deltas, not full content)
- Implement request/response caching

---

## Security Considerations

- Sanitize all file paths (prevent directory traversal)
- Validate terminal commands (prevent shell injection)
- Use Tauri's permission system
- Sandbox terminal execution
- Validate LSP server binaries
- Content Security Policy for WebView

---

## Deployment & Distribution

### Build Configuration

```json
// crates/ait42-tauri/tauri.conf.json
{
  "build": {
    "beforeBuildCommand": "cd frontend && npm run build",
    "beforeDevCommand": "cd frontend && npm run dev",
    "devPath": "http://localhost:5173",
    "distDir": "../frontend/dist"
  },
  "package": {
    "productName": "AIT42 Editor",
    "version": "0.1.0"
  },
  "tauri": {
    "bundle": {
      "identifier": "com.ait42.editor",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "targets": ["dmg", "deb", "appimage", "msi"]
    }
  }
}
```

### Release Process

1. Run tests: `cargo test --all`
2. Build frontend: `cd frontend && npm run build`
3. Build Tauri app: `cargo tauri build`
4. Package for distribution (DMG, DEB, MSI)
5. Sign binaries (macOS notarization, Windows signing)
6. Create GitHub release with artifacts

---

## Future Enhancements

### Short-term (3-6 months)

- Multiple cursors support
- Minimap enhancements
- Integrated debugger
- Git blame annotations
- Workspace management

### Long-term (6-12 months)

- Plugin marketplace
- Cloud sync (settings, themes)
- Collaboration features (Live Share)
- AI code generation (GPT integration)
- Remote development (SSH)

---

## Conclusion

This architecture provides a clear path to converting AIT42-Editor from a TUI to a modern GUI application using Tauri. The phased approach minimizes risk by building incrementally, while the separation of concerns (core logic vs. UI) ensures maintainability.

**Key Success Factors**:
1. Preserve `ait42-core` without changes (proven stable)
2. Use Tauri's IPC for clean frontend-backend separation
3. Leverage Monaco Editor and xterm.js (battle-tested components)
4. Maintain Cursor theme aesthetic
5. Implement comprehensive testing at each phase

**Next Steps**:
1. Review and approve architecture
2. Set up development environment
3. Begin Phase 0 implementation
4. Regular progress reviews after each phase

---

## References

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Monaco Editor API](https://microsoft.github.io/monaco-editor/api/index.html)
- [xterm.js Documentation](https://xtermjs.org/)
- [React Documentation](https://react.dev/)
- [Zustand Documentation](https://github.com/pmndrs/zustand)
- [AIT42 Core Documentation](../crates/ait42-core/README.md)

---

**Document Version**: 1.0
**Last Updated**: 2025-11-03
**Author**: Claude (Sonnet 4.5)
**Status**: Draft - Awaiting Review
