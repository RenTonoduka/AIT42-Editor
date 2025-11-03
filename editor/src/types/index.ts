/**
 * AIT42 Editor - TypeScript Type Definitions
 */

// File System Types
export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileNode[];
  expanded?: boolean;
}

// Editor Types
export interface EditorTab {
  id: string;
  path: string;
  name: string;
  content: string;
  language: string;
  isDirty: boolean;
  isActive: boolean;
}

export interface EditorState {
  tabs: EditorTab[];
  activeTabId: string | null;
  addTab: (tab: Omit<EditorTab, 'id' | 'isDirty' | 'isActive'>) => void;
  removeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  updateTabContent: (id: string, content: string) => void;
  markTabClean: (id: string) => void;
}

// Agent Types
export interface Agent {
  id: string;
  name: string;
  description: string;
  category: string;
  icon?: string;
}

export interface AgentExecution {
  sessionId: string;
  agentName: string;
  task: string;
  status: 'running' | 'completed' | 'failed';
  startedAt: Date;
  output?: string;
}

// Tmux Types
export interface TmuxSession {
  name: string;
  windows: number;
  created: string;
  status: 'active' | 'inactive';
}

// Terminal Types
export interface TerminalInstance {
  id: string;
  title: string;
  cwd: string;
}

// Command Palette Types
export interface Command {
  id: string;
  label: string;
  description?: string;
  category: string;
  keybinding?: string;
  action: () => void | Promise<void>;
}

// Theme Types
export interface CursorTheme {
  background: string;
  foreground: string;
  accent: string;
  accentHover: string;
  border: string;
  surface: string;
  surfaceHover: string;
  textPrimary: string;
  textSecondary: string;
  textMuted: string;
  success: string;
  warning: string;
  error: string;
  info: string;
}
