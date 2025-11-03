# AIT42 Editor - React Frontend

AI-Powered Development Environment with 49 specialized agents, Monaco Editor, and Tmux integration.

## Features

- **Monaco Editor**: Full-featured code editor with syntax highlighting
- **Cursor Dark Theme**: Professional dark theme based on Cursor IDE
- **File Tree**: Recursive file system navigation
- **Tab Management**: Multi-file editing with dirty state tracking
- **Terminal**: Integrated xterm.js terminal
- **Command Palette**: Quick command access (Cmd/Ctrl+P)
- **Keyboard Shortcuts**: Full keyboard navigation support
- **Resizable Panels**: Customizable layout with react-resizable-panels

## Architecture

```
App
├── Layout
│   ├── Sidebar
│   │   └── FileTree
│   ├── EditorArea
│   │   ├── TabBar
│   │   │   └── Tab
│   │   └── MonacoEditor
│   └── Terminal
└── CommandPalette
```

## Tech Stack

- **React 18**: Modern React with hooks
- **TypeScript**: Full type safety
- **Zustand**: Lightweight state management
- **Monaco Editor**: VS Code editor component
- **xterm.js**: Terminal emulator
- **Vite**: Fast build tool
- **CSS Modules**: Scoped styling

## Getting Started

### Prerequisites

- Node.js 18+
- npm or yarn

### Installation

```bash
cd editor
npm install
```

### Development

```bash
npm run dev
```

Open http://localhost:3000

### Build

```bash
npm run build
```

### Type Check

```bash
npm run type-check
```

### Lint

```bash
npm run lint
```

## Project Structure

```
editor/
├── src/
│   ├── components/
│   │   ├── Layout/
│   │   │   ├── Layout.tsx
│   │   │   └── Layout.module.css
│   │   ├── Sidebar/
│   │   │   ├── Sidebar.tsx
│   │   │   ├── FileTree.tsx
│   │   │   └── Sidebar.module.css
│   │   ├── Editor/
│   │   │   ├── EditorArea.tsx
│   │   │   ├── TabBar.tsx
│   │   │   ├── Tab.tsx
│   │   │   ├── MonacoEditor.tsx
│   │   │   └── Editor.module.css
│   │   ├── Terminal/
│   │   │   ├── Terminal.tsx
│   │   │   └── Terminal.module.css
│   │   └── CommandPalette/
│   │       ├── CommandPalette.tsx
│   │       └── CommandPalette.module.css
│   ├── hooks/
│   │   ├── useTauri.ts
│   │   ├── useFileSystem.ts
│   │   └── useEditor.ts
│   ├── store/
│   │   ├── editorStore.ts
│   │   └── fileTreeStore.ts
│   ├── styles/
│   │   ├── cursor-theme.css
│   │   └── global.css
│   ├── types/
│   │   └── index.ts
│   ├── App.tsx
│   └── main.tsx
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + P` | Open command palette |
| `Cmd/Ctrl + O` | Open file |
| `Cmd/Ctrl + S` | Save file |
| `Cmd/Ctrl + K S` | Save all files |
| `Cmd/Ctrl + B` | Toggle sidebar |
| `Cmd/Ctrl + J` | Toggle terminal |
| `Escape` | Close command palette |

## State Management

### Editor Store (Zustand)

```typescript
interface EditorState {
  tabs: EditorTab[];
  activeTabId: string | null;
  addTab: (tab) => void;
  removeTab: (id) => void;
  setActiveTab: (id) => void;
  updateTabContent: (id, content) => void;
  markTabClean: (id) => void;
}
```

### File Tree Store (Zustand)

```typescript
interface FileTreeState {
  rootPath: string;
  nodes: FileNode[];
  selectedPath: string | null;
  expandedPaths: Set<string>;
  setRootPath: (path) => void;
  setNodes: (nodes) => void;
  toggleExpanded: (path) => void;
}
```

## Custom Hooks

### useTauri()

Type-safe wrapper for Tauri IPC commands.

### useFileSystem()

File system operations (read, write, create, delete).

### useEditor()

High-level editor operations (open, save, close files).

## Styling

### Cursor Theme Variables

All components use CSS variables defined in `cursor-theme.css`:

- `--cursor-bg-primary`: #1E1E1E
- `--cursor-bg-secondary`: #252526
- `--cursor-fg-primary`: #CCCCCC
- `--cursor-accent`: #007ACC
- ...and more

### CSS Modules

Each component has its own scoped CSS module for maintainability.

## Next Steps

### Phase 2: Tauri Integration

1. Create Tauri backend (`src-tauri/`)
2. Implement file system commands
3. Add agent execution commands
4. Integrate tmux session management

### Phase 3: Advanced Features

1. LSP integration (IntelliSense)
2. Git integration
3. Debugger support
4. Search and replace
5. Multi-cursor editing

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for development guidelines.

## License

MIT - See [LICENSE](../LICENSE) for details.
