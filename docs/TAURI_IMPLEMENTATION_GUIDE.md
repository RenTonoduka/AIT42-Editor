# Tauri GUI Implementation Guide for AIT42 Editor

**Project**: AIT42-Editor
**Document Version**: 1.0.0
**Date**: 2025-11-03
**Target**: Migration from TUI to Tauri GUI Application

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Setup Instructions](#setup-instructions)
4. [Development Workflow](#development-workflow)
5. [Component Development](#component-development)
6. [Backend Development](#backend-development)
7. [Integration with Existing Code](#integration-with-existing-code)
8. [Cursor Theme Implementation](#cursor-theme-implementation)
9. [Testing](#testing)
10. [Building & Distribution](#building--distribution)
11. [Troubleshooting](#troubleshooting)
12. [Roadmap](#roadmap)

---

## 1. Overview

### Why Tauri?

**AIT42 Editor** is transitioning from a Terminal User Interface (TUI) to a modern desktop GUI application using Tauri. This migration brings:

**Benefits**:
- **Rich UI Components**: Modern web technologies (React/Vue/Svelte) for complex interfaces
- **Better User Experience**: Graphical file explorer, syntax highlighting, split views
- **Cross-Platform**: Single codebase for macOS, Windows, and Linux
- **Native Performance**: Rust backend + native webview (no Electron overhead)
- **Smaller Bundle Size**: ~3-5 MB vs Electron's ~100+ MB
- **Existing Codebase Reuse**: Keep all Rust logic (ait42-core, ait42-ait42, etc.)
- **AI Agent Integration**: Enhanced visualization of agent execution

**Comparison**:

| Feature | TUI (Current) | Tauri GUI (Target) |
|---------|---------------|-------------------|
| UI Framework | Ratatui | React/Svelte + Tauri |
| File Size | ~5 MB | ~8 MB |
| Startup Time | <100ms | <500ms |
| Platform | Terminal | Desktop (macOS/Windows/Linux) |
| UI Complexity | Limited | Rich (Monaco Editor, file tree) |
| Agent Visualization | Text-based | Graphical with animations |

### Architecture Summary

```
┌─────────────────────────────────────────────────────────────────┐
│                     Tauri Application                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Frontend (WebView)                Backend (Rust)                │
│  ┌─────────────────────┐          ┌──────────────────────┐      │
│  │ React/Svelte        │          │ Tauri Commands       │      │
│  │ - Monaco Editor     │◄────────►│ - File Operations    │      │
│  │ - File Explorer     │  IPC     │ - Agent Execution    │      │
│  │ - Command Palette   │          │ - LSP Client         │      │
│  │ - Agent Panel       │          │ - Configuration      │      │
│  └─────────────────────┘          └──────────┬───────────┘      │
│                                               │                  │
│                                    ┌──────────▼──────────────┐   │
│                                    │ Existing Rust Crates    │   │
│                                    │ - ait42-core            │   │
│                                    │ - ait42-ait42 (agents)  │   │
│                                    │ - ait42-lsp             │   │
│                                    │ - ait42-fs              │   │
│                                    │ - ait42-config          │   │
│                                    └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Prerequisites

### Required Software

#### macOS
```bash
# Xcode Command Line Tools (required)
xcode-select --install

# Rust 1.70+ (current project uses 1.91)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version  # Should be 1.70+

# Node.js 18+ (for frontend)
brew install node@18
node --version   # Should be 18+
npm --version    # Should be 9+

# Tauri CLI
npm install -g @tauri-apps/cli
# OR
cargo install tauri-cli
```

#### System Dependencies (macOS)
```bash
# WebKit development headers (usually pre-installed on macOS)
# No additional dependencies needed on macOS

# Optional: tmux (for agent execution)
brew install tmux
```

### Version Requirements

| Tool | Minimum Version | Recommended |
|------|----------------|-------------|
| Rust | 1.70 | 1.91 (project standard) |
| Node.js | 18.0 | 20.x LTS |
| npm | 9.0 | 10.x |
| Tauri CLI | 1.5 | 2.0 |
| macOS | 11 (Big Sur) | 14+ (Sonoma) |

### Verify Installation

```bash
# Check all prerequisites
rustc --version    # rust 1.91.0 or later
node --version     # v18.0.0 or later
npm --version      # 9.0.0 or later
cargo --version    # 1.70.0 or later
```

---

## 3. Setup Instructions

### Step 1: Install Tauri CLI

```bash
cd /Users/tonodukaren/Programming/AI/02_Workspace/05_Client/03_Sun/AIT42-Editor

# Option A: Install globally with npm
npm install -g @tauri-apps/cli

# Option B: Install as project dev dependency
npm install --save-dev @tauri-apps/cli

# Option C: Install with Cargo
cargo install tauri-cli
```

### Step 2: Initialize Tauri Project

```bash
# Initialize Tauri in the project
npm install @tauri-apps/api
npm install @tauri-apps/cli --save-dev

# Create Tauri configuration
npx tauri init
```

**Configuration Prompts**:
```
? What is your app name? AIT42 Editor
? What should the window title be? AIT42 - AI-Powered Code Editor
? Where are your web assets located? ../dist
? What is the url of your dev server? http://localhost:5173
? What is your frontend dev command? npm run dev
? What is your frontend build command? npm run build
```

### Step 3: Create Frontend Application

Choose your preferred frontend framework:

#### Option A: React + Vite (Recommended)

```bash
# Create React app with Vite
npm create vite@latest ait42-frontend -- --template react-ts

# Install dependencies
cd ait42-frontend
npm install

# Install additional dependencies
npm install @monaco-editor/react
npm install @tauri-apps/api
npm install @tauri-apps/plugin-fs
npm install react-icons
npm install @uiw/react-codemirror
```

#### Option B: Svelte + Vite (Lightweight)

```bash
# Create Svelte app
npm create vite@latest ait42-frontend -- --template svelte-ts

cd ait42-frontend
npm install
npm install @tauri-apps/api
npm install svelte-codemirror-editor
```

### Step 4: Project Structure

Create the following structure:

```bash
mkdir -p src-tauri/src
mkdir -p ait42-frontend/src/components
mkdir -p ait42-frontend/src/hooks
mkdir -p ait42-frontend/src/styles
```

**Final Structure**:
```
AIT42-Editor/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── ait42-core/               # Existing: Core editor logic
│   ├── ait42-tui/                # Existing: TUI (can be deprecated)
│   ├── ait42-ait42/              # Existing: Agent integration
│   ├── ait42-lsp/                # Existing: LSP client
│   ├── ait42-fs/                 # Existing: Filesystem
│   └── ait42-config/             # Existing: Configuration
├── src-tauri/                    # NEW: Tauri backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   └── src/
│       ├── main.rs
│       ├── commands/             # Tauri commands
│       │   ├── mod.rs
│       │   ├── editor.rs
│       │   ├── file.rs
│       │   └── agent.rs
│       └── state.rs              # Application state
└── ait42-frontend/               # NEW: Frontend application
    ├── package.json
    ├── vite.config.ts
    ├── index.html
    └── src/
        ├── main.tsx
        ├── App.tsx
        ├── components/
        │   ├── Editor.tsx
        │   ├── FileExplorer.tsx
        │   ├── CommandPalette.tsx
        │   ├── AgentPanel.tsx
        │   ├── StatusBar.tsx
        │   └── TabBar.tsx
        ├── hooks/
        │   ├── useEditor.ts
        │   ├── useFileSystem.ts
        │   └── useAgents.ts
        └── styles/
            ├── cursor-theme.css
            └── global.css
```

### Step 5: Configure Workspace

Update root `Cargo.toml`:

```toml
[workspace]
members = [
    "ait42-bin",
    "crates/ait42-core",
    "crates/ait42-tui",
    "crates/ait42-lsp",
    "crates/ait42-ait42",
    "crates/ait42-fs",
    "crates/ait42-config",
    "src-tauri",              # Add Tauri crate
]
resolver = "2"
```

### Step 6: Initial Tauri Configuration

Create `src-tauri/tauri.conf.json`:

```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:5173",
    "distDir": "../ait42-frontend/dist"
  },
  "package": {
    "productName": "AIT42 Editor",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "fs": {
        "all": true,
        "scope": ["$HOME/**"]
      },
      "shell": {
        "all": false,
        "execute": true,
        "open": true
      },
      "dialog": {
        "all": true
      },
      "window": {
        "all": false,
        "create": true,
        "center": true,
        "requestUserAttention": true,
        "setResizable": true,
        "setTitle": true,
        "maximize": true,
        "unmaximize": true
      }
    },
    "bundle": {
      "active": true,
      "category": "DeveloperTool",
      "copyright": "Copyright © 2025 AIT42 Team",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "identifier": "com.ait42.editor",
      "longDescription": "AI-powered code editor with 49 specialized development agents",
      "macOS": {
        "entitlements": null,
        "minimumSystemVersion": "11.0"
      },
      "resources": [],
      "shortDescription": "AI-Powered Code Editor",
      "targets": "all"
    },
    "security": {
      "csp": null
    },
    "updater": {
      "active": false
    },
    "windows": [
      {
        "fullscreen": false,
        "height": 800,
        "resizable": true,
        "title": "AIT42 Editor",
        "width": 1400,
        "minWidth": 800,
        "minHeight": 600
      }
    ]
  }
}
```

---

## 4. Development Workflow

### Running Development Server

```bash
# Terminal 1: Start Tauri dev server
cd /Users/tonodukaren/Programming/AI/02_Workspace/05_Client/03_Sun/AIT42-Editor
cargo tauri dev

# This will:
# 1. Start frontend dev server (Vite on http://localhost:5173)
# 2. Compile Rust backend
# 3. Open desktop application window
```

**First Run**:
- Compilation may take 5-10 minutes (subsequent runs: 10-30 seconds)
- Hot reload enabled for frontend changes
- Backend changes require app restart

### Development Commands

```bash
# Run dev server (hot reload)
cargo tauri dev

# Build for production
cargo tauri build

# Clean build artifacts
cargo clean
rm -rf ait42-frontend/dist

# Run frontend only (without Tauri)
cd ait42-frontend
npm run dev

# Run Rust tests
cargo test --workspace

# Run frontend tests
cd ait42-frontend
npm test

# Check Rust code
cargo clippy --workspace
cargo fmt --check
```

### Debugging

#### Frontend Debugging
```bash
# Open DevTools in Tauri window
- macOS: Cmd+Option+I
- Windows/Linux: Ctrl+Shift+I

# Or programmatically:
import { window } from '@tauri-apps/api';
window.getCurrent().openDevtools();
```

#### Backend Debugging
```rust
// Add to src-tauri/src/main.rs
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // ... rest of main
}
```

View logs:
```bash
# macOS
tail -f ~/Library/Logs/AIT42\ Editor/ait42-editor.log

# Linux
tail -f ~/.local/share/AIT42 Editor/logs/ait42-editor.log
```

---

## 5. Component Development

### Basic Tauri Component Structure

#### Backend Command (Rust)

`src-tauri/src/commands/file.rs`:
```rust
use tauri::State;
use ait42_fs::FileSystem;
use ait42_core::Buffer;

#[tauri::command]
pub async fn read_file(path: String) -> Result<String, String> {
    let fs = FileSystem::new();
    fs.read_file(&path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    let fs = FileSystem::new();
    fs.write_file(&path, content.as_bytes())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_directory(path: String) -> Result<Vec<String>, String> {
    let fs = FileSystem::new();
    fs.list_directory(&path)
        .await
        .map(|entries| entries.iter().map(|e| e.path.display().to_string()).collect())
        .map_err(|e| e.to_string())
}
```

Register commands in `src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::file::{read_file, write_file, list_directory};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            read_file,
            write_file,
            list_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### Frontend Component (React)

`ait42-frontend/src/components/Editor.tsx`:
```typescript
import { invoke } from '@tauri-apps/api/tauri';
import Editor from '@monaco-editor/react';
import { useState, useEffect } from 'react';

interface EditorProps {
  filePath: string;
}

export function CodeEditor({ filePath }: EditorProps) {
  const [content, setContent] = useState<string>('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadFile();
  }, [filePath]);

  async function loadFile() {
    try {
      setLoading(true);
      const fileContent = await invoke<string>('read_file', { path: filePath });
      setContent(fileContent);
    } catch (error) {
      console.error('Failed to load file:', error);
    } finally {
      setLoading(false);
    }
  }

  async function saveFile(newContent: string) {
    try {
      await invoke('write_file', { path: filePath, content: newContent });
      console.log('File saved successfully');
    } catch (error) {
      console.error('Failed to save file:', error);
    }
  }

  if (loading) return <div>Loading...</div>;

  return (
    <Editor
      height="100vh"
      language="rust"
      theme="cursor-dark"
      value={content}
      onChange={(value) => value && saveFile(value)}
      options={{
        fontSize: 14,
        fontFamily: 'JetBrains Mono, Menlo, Monaco, Courier New',
        minimap: { enabled: true },
        scrollBeyondLastLine: false,
      }}
    />
  );
}
```

### Styling Guidelines

Use Tailwind CSS or CSS Modules:

`ait42-frontend/tailwind.config.js`:
```javascript
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        'cursor-bg': '#1E1E1E',
        'cursor-bg-light': '#252525',
        'cursor-accent': '#007ACC',
        'cursor-text': '#CCCCCC',
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Menlo', 'Monaco', 'Courier New', 'monospace'],
      },
    },
  },
  plugins: [],
};
```

### State Management

Use React Context or Zustand:

`ait42-frontend/src/store/editorStore.ts`:
```typescript
import create from 'zustand';

interface EditorStore {
  openFiles: string[];
  activeFile: string | null;
  openFile: (path: string) => void;
  closeFile: (path: string) => void;
  setActiveFile: (path: string) => void;
}

export const useEditorStore = create<EditorStore>((set) => ({
  openFiles: [],
  activeFile: null,
  openFile: (path) => set((state) => ({
    openFiles: [...state.openFiles, path],
    activeFile: path,
  })),
  closeFile: (path) => set((state) => ({
    openFiles: state.openFiles.filter((f) => f !== path),
  })),
  setActiveFile: (path) => set({ activeFile: path }),
}));
```

---

## 6. Backend Development

### Adding New Tauri Commands

#### Step 1: Define Command

`src-tauri/src/commands/agent.rs`:
```rust
use tauri::State;
use ait42_ait42::{Coordinator, ExecutionMode, AgentExecutor};
use std::sync::Mutex;

pub struct AppState {
    executor: Mutex<AgentExecutor>,
}

#[tauri::command]
pub async fn execute_agent(
    agent_name: String,
    task: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let mut executor = state.executor.lock().unwrap();

    let results = executor
        .execute_single(&agent_name, &task)
        .await
        .map_err(|e| e.to_string())?;

    Ok(results.output)
}

#[tauri::command]
pub async fn list_agents(
    state: State<'_, AppState>
) -> Result<Vec<String>, String> {
    let executor = state.executor.lock().unwrap();
    Ok(executor.coordinator.list_agents()
        .into_iter()
        .map(|a| a.name)
        .collect())
}
```

#### Step 2: Register Command

`src-tauri/src/main.rs`:
```rust
mod commands;

use commands::agent::{execute_agent, list_agents, AppState};
use ait42_ait42::{AIT42Config, Coordinator, AgentExecutor};
use std::sync::Mutex;

fn main() {
    let config = AIT42Config::load().expect("Failed to load config");
    let coordinator = Coordinator::new(config).expect("Failed to create coordinator");
    let executor = AgentExecutor::new(coordinator);

    tauri::Builder::default()
        .manage(AppState {
            executor: Mutex::new(executor),
        })
        .invoke_handler(tauri::generate_handler![
            execute_agent,
            list_agents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Error Handling

**Rust Side**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Agent execution failed: {0}")]
    AgentError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<CommandError> for String {
    fn from(err: CommandError) -> Self {
        err.to_string()
    }
}
```

**TypeScript Side**:
```typescript
import { invoke } from '@tauri-apps/api/tauri';

async function executeAgent(agent: string, task: string) {
  try {
    const result = await invoke<string>('execute_agent', {
      agentName: agent,
      task: task,
    });
    return { success: true, data: result };
  } catch (error) {
    console.error('Agent execution failed:', error);
    return { success: false, error: String(error) };
  }
}
```

### Testing Strategies

#### Rust Backend Tests

`src-tauri/src/commands/file.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_read_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "Hello, Tauri!").unwrap();

        let content = read_file(file_path.display().to_string())
            .await
            .unwrap();
        assert_eq!(content, "Hello, Tauri!");
    }

    #[tokio::test]
    async fn test_write_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("output.txt");

        write_file(
            file_path.display().to_string(),
            "Test content".to_string()
        ).await.unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Test content");
    }
}
```

Run tests:
```bash
cargo test -p ait42-tauri
```

---

## 7. Integration with Existing Code

### Using ait42-core

The existing `ait42-core` crate contains the core editor logic (Buffer, Cursor, etc.). Integrate it seamlessly:

`src-tauri/src/commands/editor.rs`:
```rust
use ait42_core::{Buffer, BufferId, Cursor, Position};
use tauri::State;
use std::sync::Mutex;
use std::collections::HashMap;

pub struct EditorState {
    buffers: Mutex<HashMap<BufferId, Buffer>>,
}

#[tauri::command]
pub fn create_buffer(
    content: String,
    file_path: Option<String>,
    state: State<'_, EditorState>
) -> Result<String, String> {
    let mut buffers = state.buffers.lock().unwrap();
    let buffer = Buffer::new(content, file_path);
    let id = buffer.id().to_string();
    buffers.insert(buffer.id(), buffer);
    Ok(id)
}

#[tauri::command]
pub fn get_buffer_content(
    buffer_id: String,
    state: State<'_, EditorState>
) -> Result<String, String> {
    let buffers = state.buffers.lock().unwrap();
    let id: BufferId = buffer_id.parse().map_err(|e| format!("{}", e))?;

    buffers.get(&id)
        .map(|b| b.to_string())
        .ok_or_else(|| "Buffer not found".to_string())
}

#[tauri::command]
pub fn insert_text(
    buffer_id: String,
    position: (usize, usize),
    text: String,
    state: State<'_, EditorState>
) -> Result<(), String> {
    let mut buffers = state.buffers.lock().unwrap();
    let id: BufferId = buffer_id.parse().map_err(|e| format!("{}", e))?;

    let buffer = buffers.get_mut(&id)
        .ok_or_else(|| "Buffer not found".to_string())?;

    let pos = Position::new(position.0, position.1);
    buffer.insert(pos, &text);
    Ok(())
}
```

### Using ait42-ait42 (Agent System)

The agent system is already implemented. Expose it through Tauri:

`src-tauri/src/commands/agent.rs`:
```rust
use ait42_ait42::{
    Coordinator, AgentExecutor, ExecutionMode,
    commands::{AgentCommand, CommandResult}
};
use tauri::State;
use std::sync::Mutex;

pub struct AgentState {
    executor: Mutex<AgentExecutor>,
}

#[tauri::command]
pub async fn run_agent(
    agent: String,
    task: String,
    state: State<'_, AgentState>
) -> Result<String, String> {
    let cmd = AgentCommand::RunAgent { agent, task };
    let mut executor = state.executor.lock().unwrap();

    match cmd.execute(&mut executor).await {
        Ok(CommandResult::Executed(results)) => {
            Ok(results.first()
                .map(|r| r.output.clone())
                .unwrap_or_default())
        }
        Ok(_) => Err("Unexpected result type".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn run_coordinated(
    task: String,
    state: State<'_, AgentState>
) -> Result<Vec<AgentResult>, String> {
    let mut executor = state.executor.lock().unwrap();

    let results = executor
        .execute(ExecutionMode::Coordinated, &task)
        .await
        .map_err(|e| e.to_string())?;

    Ok(results.into_iter().map(|r| AgentResult {
        agent_name: r.agent_name,
        output: r.output,
        status: format!("{:?}", r.status),
        duration_ms: r.duration.as_millis() as u64,
    }).collect())
}

#[derive(serde::Serialize)]
pub struct AgentResult {
    agent_name: String,
    output: String,
    status: String,
    duration_ms: u64,
}
```

### Migrating from TUI Widgets

**Before (TUI)**:
```rust
// crates/ait42-tui/src/widgets/editor.rs
use ratatui::{Frame, layout::Rect, widgets::Block};

pub fn render_editor(frame: &mut Frame, area: Rect, buffer: &Buffer) {
    let block = Block::default()
        .title("Editor")
        .borders(Borders::ALL);

    let content = buffer.to_string();
    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}
```

**After (Tauri + React)**:
```typescript
// ait42-frontend/src/components/Editor.tsx
import Editor from '@monaco-editor/react';

export function CodeEditor({ buffer }: { buffer: Buffer }) {
  return (
    <div className="border border-gray-700 rounded">
      <div className="bg-cursor-bg-light px-4 py-2 border-b border-gray-700">
        <h3 className="text-cursor-text font-semibold">Editor</h3>
      </div>
      <Editor
        height="600px"
        language="rust"
        theme="cursor-dark"
        value={buffer.content}
        options={{
          fontSize: 14,
          minimap: { enabled: true },
        }}
      />
    </div>
  );
}
```

---

## 8. Cursor Theme Implementation

### Color Palette

Create a comprehensive color system matching the TUI Cursor theme:

`ait42-frontend/src/styles/cursor-theme.css`:
```css
:root {
  /* Background hierarchy */
  --bg-deep: #1A1A1A;
  --bg-main: #1E1E1E;
  --bg-light: #252525;
  --bg-lighter: #2D2D2D;

  /* Foreground hierarchy */
  --fg-main: #CCCCCC;
  --fg-dim: #858585;
  --fg-dimmer: #606060;

  /* Cursor blue accent */
  --accent-primary: #007ACC;
  --accent-hover: #148EE0;

  /* Semantic colors */
  --success: #10B981;
  --info: #3B82F6;
  --warning: #F59E0B;
  --error: #EF4444;

  /* Borders */
  --border: #3E3E42;
  --border-active: #007ACC;

  /* Syntax highlighting */
  --syntax-keyword: #C586C0;
  --syntax-type: #4EC9B0;
  --syntax-function: #DCDCAA;
  --syntax-string: #CE9178;
  --syntax-comment: #6A9955;
  --syntax-number: #B5CEA8;
  --syntax-variable: #9CDCFE;
}

body {
  background-color: var(--bg-main);
  color: var(--fg-main);
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

.editor-container {
  background-color: var(--bg-main);
}

.sidebar {
  background-color: var(--bg-light);
  border-right: 1px solid var(--border);
}

.tab-active {
  background-color: var(--bg-lighter);
  color: white;
}

.tab-inactive {
  background-color: var(--bg-light);
  color: var(--fg-dim);
}

.status-bar {
  background-color: var(--accent-primary);
  color: white;
  padding: 0.5rem 1rem;
}

button.primary {
  background-color: var(--accent-primary);
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 0.25rem;
  cursor: pointer;
}

button.primary:hover {
  background-color: var(--accent-hover);
}
```

### Monaco Editor Theme

`ait42-frontend/src/themes/cursor-dark.ts`:
```typescript
import { editor } from 'monaco-editor';

export const cursorDarkTheme: editor.IStandaloneThemeData = {
  base: 'vs-dark',
  inherit: true,
  rules: [
    { token: 'comment', foreground: '6A9955' },
    { token: 'keyword', foreground: 'C586C0', fontStyle: 'bold' },
    { token: 'string', foreground: 'CE9178' },
    { token: 'number', foreground: 'B5CEA8' },
    { token: 'type', foreground: '4EC9B0' },
    { token: 'function', foreground: 'DCDCAA' },
    { token: 'variable', foreground: '9CDCFE' },
  ],
  colors: {
    'editor.background': '#1E1E1E',
    'editor.foreground': '#CCCCCC',
    'editor.lineHighlightBackground': '#252525',
    'editorCursor.foreground': '#FFFFFF',
    'editorLineNumber.foreground': '#858585',
    'editorLineNumber.activeForeground': '#CCCCCC',
    'editor.selectionBackground': '#264F78',
    'editor.inactiveSelectionBackground': '#3A3D41',
    'scrollbarSlider.background': '#79797966',
    'scrollbarSlider.hoverBackground': '#646464B3',
  },
};

// Register theme
import * as monaco from 'monaco-editor';
monaco.editor.defineTheme('cursor-dark', cursorDarkTheme);
```

### CSS Variables Usage

`ait42-frontend/src/components/StatusBar.tsx`:
```typescript
export function StatusBar({ file, position, language }: StatusBarProps) {
  return (
    <div className="status-bar flex justify-between items-center">
      <div className="flex gap-4">
        <span>{file || 'No file open'}</span>
        <span>Ln {position.line}, Col {position.column}</span>
        <span>{language}</span>
      </div>
      <div className="flex gap-4">
        <span>UTF-8</span>
        <span>LF</span>
        <span>Rust</span>
      </div>
    </div>
  );
}
```

---

## 9. Testing

### Unit Tests (Rust)

```bash
# Test all workspace crates
cargo test --workspace

# Test specific crate
cargo test -p ait42-tauri

# Test with coverage
cargo tarpaulin --workspace --out Html
```

### Integration Tests (Rust)

`src-tauri/tests/integration_test.rs`:
```rust
use ait42_tauri::commands::{read_file, write_file};
use tempfile::tempdir;
use std::fs;

#[tokio::test]
async fn test_file_operations() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.txt");
    let path_str = path.display().to_string();

    // Write file
    write_file(path_str.clone(), "Hello, World!".to_string())
        .await
        .unwrap();

    // Read file
    let content = read_file(path_str).await.unwrap();
    assert_eq!(content, "Hello, World!");
}
```

### E2E Tests (Playwright)

Install Playwright:
```bash
cd ait42-frontend
npm install -D @playwright/test
npx playwright install
```

`ait42-frontend/tests/editor.spec.ts`:
```typescript
import { test, expect } from '@playwright/test';

test('should load editor', async ({ page }) => {
  await page.goto('http://localhost:5173');

  // Check if editor loads
  await expect(page.locator('.monaco-editor')).toBeVisible();
});

test('should open file', async ({ page }) => {
  await page.goto('http://localhost:5173');

  // Click open file
  await page.click('[data-testid="open-file"]');

  // Select file
  await page.fill('[data-testid="file-path"]', '/path/to/file.rs');
  await page.click('[data-testid="confirm"]');

  // Verify file content loads
  await expect(page.locator('.monaco-editor')).toContainText('fn main()');
});

test('should execute agent', async ({ page }) => {
  await page.goto('http://localhost:5173');

  // Open command palette
  await page.keyboard.press('Meta+P');

  // Type agent command
  await page.fill('[data-testid="command-input"]', 'agent run code-reviewer');
  await page.keyboard.press('Enter');

  // Wait for result
  await expect(page.locator('[data-testid="agent-output"]')).toBeVisible();
});
```

Run E2E tests:
```bash
npm run test:e2e
```

---

## 10. Building & Distribution

### Development Build

```bash
# Build for development (with debug symbols)
cargo tauri build --debug
```

### Production Build

```bash
# Build optimized release
cargo tauri build

# Output locations:
# macOS: src-tauri/target/release/bundle/macos/AIT42 Editor.app
# Linux: src-tauri/target/release/bundle/appimage/
# Windows: src-tauri/target/release/bundle/msi/
```

### macOS App Bundle

The built app is located at:
```
src-tauri/target/release/bundle/macos/AIT42 Editor.app
```

Install to Applications:
```bash
cp -r "src-tauri/target/release/bundle/macos/AIT42 Editor.app" /Applications/
```

### Code Signing (macOS)

**Prerequisites**:
- Apple Developer account
- Valid Developer ID certificate

**Configure** `tauri.conf.json`:
```json
{
  "tauri": {
    "bundle": {
      "macOS": {
        "signing": {
          "identity": "Developer ID Application: Your Name (TEAM_ID)"
        }
      }
    }
  }
}
```

**Sign and notarize**:
```bash
# Build with signing
cargo tauri build

# Notarize (requires Apple Developer account)
xcrun notarytool submit \
  "src-tauri/target/release/bundle/macos/AIT42 Editor.app.zip" \
  --apple-id "your@email.com" \
  --team-id "TEAM_ID" \
  --password "app-specific-password"
```

### Auto-updates

Install Tauri updater:
```bash
cargo install tauri-cli --features updater
```

Configure `tauri.conf.json`:
```json
{
  "tauri": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://releases.ait42.com/{{target}}/{{current_version}}"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY"
    }
  }
}
```

Generate key pair:
```bash
tauri signer generate -w ~/.tauri/myapp.key
```

---

## 11. Troubleshooting

### Common Issues

#### Issue: "Failed to load dynamic library"

**macOS**:
```bash
# Install required libraries
brew install webview

# Set library path
export DYLD_LIBRARY_PATH=/usr/local/lib:$DYLD_LIBRARY_PATH
```

#### Issue: "Command not found: cargo tauri"

```bash
# Install Tauri CLI globally
npm install -g @tauri-apps/cli

# OR add to project
npm install --save-dev @tauri-apps/cli
npx tauri dev
```

#### Issue: "Frontend dev server not starting"

```bash
# Check if port 5173 is in use
lsof -i :5173
kill -9 <PID>

# Start manually
cd ait42-frontend
npm run dev
```

#### Issue: "Rust compilation errors"

```bash
# Clean and rebuild
cargo clean
rm -rf target/
cargo build

# Update dependencies
cargo update
```

#### Issue: "Monaco Editor not loading"

```typescript
// Add to vite.config.ts
export default defineConfig({
  optimizeDeps: {
    include: ['monaco-editor'],
  },
  build: {
    commonjsOptions: {
      include: [/monaco-editor/, /node_modules/],
    },
  },
});
```

### Debug Mode

Enable verbose logging:

`src-tauri/src/main.rs`:
```rust
fn main() {
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    // ... rest of main
}
```

Run with logs:
```bash
RUST_LOG=debug cargo tauri dev 2>&1 | tee tauri.log
```

### Performance Issues

**Slow startup**:
- Reduce frontend bundle size
- Lazy load Monaco Editor
- Use code splitting

**High memory usage**:
- Limit open buffers
- Implement buffer eviction
- Use virtual scrolling for file explorer

---

## 12. Roadmap

### Phase 1: Basic GUI (Current)

**Timeline**: 2-4 weeks

**Goals**:
- ✅ Tauri project setup
- ✅ Basic file operations (read/write)
- ✅ Monaco Editor integration
- ✅ File explorer (sidebar)
- ✅ Tab bar
- ✅ Status bar
- ✅ Command palette
- ✅ Cursor dark theme

**Deliverables**:
- Working desktop app
- Basic text editing
- File management

### Phase 2: Advanced Features

**Timeline**: 4-6 weeks

**Goals**:
- LSP integration (autocomplete, diagnostics)
- Agent panel with real-time execution
- Split view editor
- Search & replace
- Git integration
- Terminal panel
- Settings UI

**Deliverables**:
- Full-featured code editor
- AI agent visualization
- Developer workflow tools

### Phase 3: AI Integration

**Timeline**: 4-8 weeks

**Goals**:
- Inline agent suggestions
- Code generation UI
- Agent execution history
- Multi-agent workflows
- Agent marketplace
- Custom agent templates

**Deliverables**:
- AI-powered coding experience
- Advanced agent orchestration
- User customization

### Phase 4: Polish & Distribution

**Timeline**: 2-4 weeks

**Goals**:
- Performance optimization
- Comprehensive testing
- Documentation
- macOS app signing
- Auto-updater
- Release to GitHub/website

**Deliverables**:
- Production-ready app
- Distribution channels
- User documentation

---

## Conclusion

This guide provides a comprehensive roadmap for migrating AIT42 Editor from a TUI to a modern Tauri GUI application. By leveraging existing Rust crates (`ait42-core`, `ait42-ait42`, etc.) and combining them with a rich web-based frontend, you'll create a powerful, cross-platform code editor with integrated AI agents.

**Key Advantages**:
- Preserve all existing Rust logic
- Modern, user-friendly interface
- Native performance
- Small bundle size (~8 MB vs Electron's ~100+ MB)
- Cross-platform (macOS, Windows, Linux)

**Next Steps**:
1. Run `cargo tauri init` to bootstrap the project
2. Choose frontend framework (React recommended)
3. Implement basic file operations
4. Integrate Monaco Editor
5. Connect AI agent system
6. Build and test

For questions or issues, refer to:
- [Tauri Documentation](https://tauri.app/v1/guides/)
- [AIT42 Integration Report](../AIT42_INTEGRATION_REPORT.md)
- [Cursor Theme Implementation](./PHASE10_CURSOR_UI_IMPLEMENTATION.md)

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-03
**Maintained By**: AIT42 Team
