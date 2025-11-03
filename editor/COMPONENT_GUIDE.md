# AIT42 Editor - Component Implementation Guide

## Overview

Complete React component structure for AIT42 Editor with TypeScript, Zustand state management, and Cursor dark theme.

## Component Hierarchy

```
App (src/App.tsx)
└── Layout (components/Layout/Layout.tsx)
    ├── Sidebar (components/Sidebar/Sidebar.tsx)
    │   └── FileTree (components/Sidebar/FileTree.tsx)
    │       └── FileTreeItem (inline component)
    ├── EditorArea (components/Editor/EditorArea.tsx)
    │   ├── TabBar (components/Editor/TabBar.tsx)
    │   │   └── Tab (components/Editor/Tab.tsx)
    │   └── MonacoEditor (components/Editor/MonacoEditor.tsx)
    ├── Terminal (components/Terminal/Terminal.tsx)
    └── CommandPalette (components/CommandPalette/CommandPalette.tsx)
```

## State Management

### Zustand Stores

#### 1. Editor Store (`store/editorStore.ts`)

**Purpose**: Manage open tabs and editor content

**State**:
```typescript
{
  tabs: EditorTab[];
  activeTabId: string | null;
}
```

**Actions**:
- `addTab(tab)`: Add new tab or activate existing
- `removeTab(id)`: Close tab with confirmation if dirty
- `setActiveTab(id)`: Switch active tab
- `updateTabContent(id, content)`: Update and mark dirty
- `markTabClean(id)`: Mark as saved

**Usage**:
```typescript
const { tabs, activeTab, addTab, updateContent } = useEditor();
```

#### 2. File Tree Store (`store/fileTreeStore.ts`)

**Purpose**: Manage file system navigation state

**State**:
```typescript
{
  rootPath: string;
  nodes: FileNode[];
  selectedPath: string | null;
  expandedPaths: Set<string>;
}
```

**Actions**:
- `setRootPath(path)`: Set workspace root
- `setNodes(nodes)`: Update tree structure
- `setSelectedPath(path)`: Highlight selection
- `toggleExpanded(path)`: Expand/collapse folder
- `updateNodeChildren(path, children)`: Load subdirectory

## Custom Hooks

### 1. useTauri (`hooks/useTauri.ts`)

**Purpose**: Type-safe Tauri IPC wrapper

**API**:
```typescript
const { invoke } = useTauri();
const result = await invoke<Type>('command_name', { args });
```

**Commands**:
- File System: `read_directory`, `read_file`, `write_file`
- Agents: `list_agents`, `run_agent`, `run_coordinator`
- Tmux: `list_tmux_sessions`, `capture_tmux_output`

### 2. useFileSystem (`hooks/useFileSystem.ts`)

**Purpose**: High-level file operations

**API**:
```typescript
const { readDirectory, readFile, writeFile, loading, error } = useFileSystem();

// Example
const nodes = await readDirectory('/path/to/dir');
const content = await readFile('/path/to/file.ts');
await writeFile('/path/to/file.ts', 'new content');
```

### 3. useEditor (`hooks/useEditor.ts`)

**Purpose**: Editor operations (combines store + file system)

**API**:
```typescript
const {
  tabs,
  activeTab,
  openFile,
  saveFile,
  saveAllFiles,
  closeFile,
  updateContent,
} = useEditor();

// Example
await openFile('/path/to/file.ts', 'file.ts');
await saveFile(); // Save active tab
await closeFile(tabId); // Prompts if dirty
```

## Component Details

### Layout Component

**File**: `components/Layout/Layout.tsx`

**Features**:
- Resizable panels (react-resizable-panels)
- Keyboard shortcuts
- Panel visibility toggles

**Keyboard Shortcuts**:
- `Cmd/Ctrl + P`: Open command palette
- `Cmd/Ctrl + B`: Toggle sidebar
- `Cmd/Ctrl + J`: Toggle terminal
- `Cmd/Ctrl + S`: Save file

**Panel Configuration**:
```typescript
<PanelGroup direction="horizontal">
  <Panel defaultSize={20} minSize={15} maxSize={40}>
    <Sidebar />
  </Panel>
  <Panel minSize={30}>
    <PanelGroup direction="vertical">
      <Panel defaultSize={70}>
        <EditorArea />
      </Panel>
      <Panel defaultSize={30}>
        <Terminal />
      </Panel>
    </PanelGroup>
  </Panel>
</PanelGroup>
```

### Sidebar Component

**File**: `components/Sidebar/Sidebar.tsx`

**Features**:
- File tree navigation
- Action buttons (new file, new folder, refresh)
- Collapsible header

**Actions**:
- New File: Create file in selected directory
- New Folder: Create subdirectory
- Refresh: Reload file tree

### FileTree Component

**File**: `components/Sidebar/FileTree.tsx`

**Features**:
- Recursive directory rendering
- Lazy loading of subdirectories
- File type icons
- Expand/collapse animation
- Selection highlighting

**File Icons**:
```typescript
{
  'ts': '📘',
  'tsx': '⚛️',
  'js': '📜',
  'rs': '🦀',
  'py': '🐍',
  'go': '🐹',
  'json': '📋',
  // ... more
}
```

**Interaction**:
- Click folder: Expand/collapse
- Click file: Open in editor
- Hover: Highlight background

### EditorArea Component

**File**: `components/Editor/EditorArea.tsx`

**Features**:
- Tab bar for open files
- Monaco editor integration
- Welcome screen when empty

**Welcome Screen**:
- App title and description
- Keyboard shortcuts reference
- Feature highlights

### TabBar & Tab Components

**Files**:
- `components/Editor/TabBar.tsx`
- `components/Editor/Tab.tsx`

**Features**:
- Multiple open files
- Active tab highlighting
- Modified indicator (dot)
- Close button
- Horizontal scrolling

**Tab Layout**:
```
[Icon] [Filename] [●] [×]
   ↓       ↓       ↓   ↓
 Emoji   Name   Dirty Close
```

### MonacoEditor Component

**File**: `components/Editor/MonacoEditor.tsx`

**Features**:
- Full Monaco editor instance
- Cursor dark theme
- Syntax highlighting
- Auto-save on Cmd/Ctrl+S
- Content change tracking

**Theme Configuration**:
```typescript
monaco.editor.defineTheme('cursor-dark', {
  base: 'vs-dark',
  inherit: true,
  rules: [
    { token: 'keyword', foreground: '569CD6' },
    { token: 'string', foreground: 'CE9178' },
    { token: 'comment', foreground: '6A9955' },
    // ...
  ],
  colors: {
    'editor.background': '#1E1E1E',
    'editor.foreground': '#CCCCCC',
    // ...
  },
});
```

**Editor Options**:
- Font: Fira Code with ligatures
- Font size: 14px
- Tab size: 2 spaces
- Minimap: Enabled
- Line numbers: On
- Bracket pair colorization: Enabled

### Terminal Component

**File**: `components/Terminal/Terminal.tsx`

**Features**:
- xterm.js integration
- Cursor dark theme colors
- Resizable with fit addon
- Web links addon
- Multiple terminal instances (future)

**Terminal Colors**:
```typescript
theme: {
  background: '#1E1E1E',
  foreground: '#CCCCCC',
  black: '#000000',
  red: '#CD3131',
  green: '#0DBC79',
  // ...
}
```

**Actions**:
- New Terminal: Create additional instance
- Split Terminal: Vertical/horizontal split
- Clear Terminal: Clear output

### CommandPalette Component

**File**: `components/CommandPalette/CommandPalette.tsx`

**Features**:
- Fuzzy command search
- Keyboard navigation (↑/↓)
- Command categories
- Keybinding display
- Recent commands (future)

**Commands**:
```typescript
{
  id: 'file.open',
  label: 'Open File',
  description: 'Open a file from the workspace',
  category: 'File',
  keybinding: 'Cmd+O',
  action: () => { /* ... */ }
}
```

**Navigation**:
- `↑/↓`: Navigate commands
- `Enter`: Execute command
- `Escape`: Close palette
- Type: Filter commands

## Styling System

### CSS Variables (cursor-theme.css)

**Background Colors**:
```css
--cursor-bg-primary: #1E1E1E;
--cursor-bg-secondary: #252526;
--cursor-bg-tertiary: #2D2D30;
```

**Foreground Colors**:
```css
--cursor-fg-primary: #CCCCCC;
--cursor-fg-secondary: #858585;
--cursor-fg-muted: #6A6A6A;
```

**Accent Colors**:
```css
--cursor-accent: #007ACC;
--cursor-accent-hover: #0098FF;
```

**Status Colors**:
```css
--cursor-success: #89D185;
--cursor-warning: #CCA700;
--cursor-error: #F48771;
--cursor-info: #75BEFF;
```

### CSS Modules

Each component has scoped styles:

```
Layout.module.css
Sidebar.module.css
Editor.module.css
Terminal.module.css
CommandPalette.module.css
```

**Benefits**:
- No style conflicts
- Clear component boundaries
- Easy to maintain
- Type-safe class names

### Utility Classes (global.css)

```css
.flex, .flex-col, .items-center
.gap-1, .gap-2, .gap-4
.p-2, .p-4, .m-0
.w-full, .h-full
.text-primary, .text-secondary
.bg-primary, .bg-secondary
.rounded, .rounded-md, .rounded-lg
```

## Type Definitions

### File Types (`types/index.ts`)

```typescript
interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileNode[];
  expanded?: boolean;
}

interface EditorTab {
  id: string;
  path: string;
  name: string;
  content: string;
  language: string;
  isDirty: boolean;
  isActive: boolean;
}

interface Agent {
  id: string;
  name: string;
  description: string;
  category: string;
}

interface TmuxSession {
  name: string;
  windows: number;
  created: string;
  status: 'active' | 'inactive';
}
```

## Testing Strategy

### Unit Tests

**Test Files**:
```
EditorArea.test.tsx
TabBar.test.tsx
FileTree.test.tsx
Terminal.test.tsx
```

**Example Test**:
```typescript
import { render, screen, fireEvent } from '@testing-library/react';
import { Tab } from './Tab';

describe('Tab', () => {
  it('renders tab label', () => {
    render(<Tab tab={mockTab} isActive={false} onActivate={jest.fn()} />);
    expect(screen.getByText('file.ts')).toBeInTheDocument();
  });

  it('shows dirty indicator when modified', () => {
    const dirtyTab = { ...mockTab, isDirty: true };
    render(<Tab tab={dirtyTab} isActive={false} onActivate={jest.fn()} />);
    expect(screen.getByTitle('Modified')).toBeInTheDocument();
  });

  it('calls onActivate when clicked', () => {
    const onActivate = jest.fn();
    render(<Tab tab={mockTab} isActive={false} onActivate={onActivate} />);
    fireEvent.click(screen.getByRole('tab'));
    expect(onActivate).toHaveBeenCalledTimes(1);
  });
});
```

### Integration Tests

**Scenarios**:
1. Open file from file tree → Tab appears in editor
2. Edit file content → Tab shows dirty indicator
3. Save file → Dirty indicator disappears
4. Close dirty file → Confirmation prompt
5. Search command → Filtered results

### E2E Tests (Playwright)

```typescript
test('can open and edit a file', async ({ page }) => {
  await page.goto('http://localhost:3000');

  // Click on file in tree
  await page.click('text=src/App.tsx');

  // Wait for editor to load
  await page.waitForSelector('.monaco-editor');

  // Type some text
  await page.keyboard.type('// New comment');

  // Check dirty indicator
  await expect(page.locator('.tabModified')).toBeVisible();

  // Save file
  await page.keyboard.press('Control+S');

  // Dirty indicator should disappear
  await expect(page.locator('.tabModified')).not.toBeVisible();
});
```

## Performance Optimization

### 1. React.memo

Wrap expensive components:
```typescript
export const FileTreeItem = React.memo<FileTreeItemProps>(({ node, level }) => {
  // Component logic
});
```

### 2. useCallback

Memoize callbacks:
```typescript
const handleClick = useCallback(async () => {
  await openFile(node.path, node.name);
}, [openFile, node.path, node.name]);
```

### 3. useMemo

Memoize expensive computations:
```typescript
const filteredCommands = useMemo(() => {
  return commands.filter(cmd => cmd.label.includes(search));
}, [commands, search]);
```

### 4. Code Splitting

```typescript
const Terminal = lazy(() => import('./Terminal/Terminal'));
const CommandPalette = lazy(() => import('./CommandPalette/CommandPalette'));
```

### 5. Virtual Scrolling

For large file trees:
```typescript
import { FixedSizeList } from 'react-window';

<FixedSizeList
  height={600}
  itemCount={nodes.length}
  itemSize={24}
>
  {({ index, style }) => (
    <FileTreeItem node={nodes[index]} style={style} />
  )}
</FixedSizeList>
```

## Accessibility

### WCAG 2.1 AA Compliance

**Keyboard Navigation**:
- All interactive elements focusable
- Tab order logical
- Custom keyboard shortcuts

**ARIA Attributes**:
```tsx
<div role="tab" aria-selected={isActive}>
<button aria-label="Close file">×</button>
<input aria-labelledby="search-label" />
```

**Focus Management**:
```typescript
useEffect(() => {
  inputRef.current?.focus();
}, []);
```

**Color Contrast**:
- Text on background: 7:1 (AAA)
- Interactive elements: 4.5:1 (AA)

**Screen Reader Support**:
- Semantic HTML
- Descriptive labels
- Live regions for updates

## Next Steps

### Phase 2: Tauri Backend Integration

1. **Create Tauri project**:
```bash
npm create tauri-app@latest
```

2. **Implement Rust commands**:
```rust
#[tauri::command]
async fn read_directory(path: String) -> Result<Vec<FileNode>, String>

#[tauri::command]
async fn run_agent(agent: String, task: String) -> Result<String, String>
```

3. **Connect frontend to backend**:
```typescript
const nodes = await invoke<FileNode[]>('read_directory', { path });
```

### Phase 3: Advanced Features

- LSP integration (IntelliSense, go-to-definition)
- Git integration (diff, commit, push)
- Search and replace
- Multi-cursor editing
- Debugger integration
- Extension system

## Resources

### Documentation
- [React Docs](https://react.dev)
- [Monaco Editor](https://microsoft.github.io/monaco-editor/)
- [xterm.js](https://xtermjs.org)
- [Zustand](https://docs.pmnd.rs/zustand)
- [Tauri](https://tauri.app)

### Code References
- [VS Code Source](https://github.com/microsoft/vscode)
- [Cursor](https://cursor.sh)
- [Zed Editor](https://github.com/zed-industries/zed)

## Contributing

1. Follow TypeScript strict mode
2. Write tests for new features
3. Use CSS modules for styling
4. Document complex logic
5. Keep components small and focused
6. Use custom hooks for logic reuse

## License

MIT - See LICENSE file for details.
