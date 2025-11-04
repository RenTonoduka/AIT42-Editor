/**
 * Editor Store - Manages editor tabs and content
 *
 * Uses Zustand for lightweight state management
 */

import { create } from 'zustand';
import { tauriApi } from '@/services/tauri';

/**
 * Represents an open editor tab
 */
export interface EditorTab {
  /** Unique tab identifier */
  id: string;
  /** Full file path */
  path: string;
  /** File name (for display) */
  name: string;
  /** Current editor content */
  content: string;
  /** Language identifier (typescript, rust, etc) */
  language: string;
  /** Whether content has unsaved changes */
  isDirty: boolean;
  /** Whether this is the active tab */
  isActive: boolean;
}

/**
 * Editor store state and actions
 */
interface EditorStore {
  /** All open tabs */
  tabs: EditorTab[];
  /** Currently active tab ID */
  activeTabId: string | null;

  // Actions
  /** Add a new tab from file path */
  addTab: (path: string) => Promise<void>;
  /** Close a tab by ID */
  closeTab: (id: string) => void;
  /** Set active tab */
  setActiveTab: (id: string) => void;
  /** Update tab content */
  updateTabContent: (id: string, content: string) => void;
  /** Save tab content to file */
  saveTab: (id: string) => Promise<void>;
  /** Save all dirty tabs */
  saveAllTabs: () => Promise<void>;
  /** Close all tabs */
  closeAllTabs: () => void;
  /** Get tab by ID */
  getTabById: (id: string) => EditorTab | undefined;
  /** Get active tab */
  getActiveTab: () => EditorTab | undefined;
}

/**
 * Generate a unique tab ID from file path
 */
function generateTabId(path: string): string {
  return `tab-${path.replace(/[^a-zA-Z0-9]/g, '-')}`;
}

/**
 * Extract file name from path
 */
function getFileName(path: string): string {
  return path.split('/').pop() || path;
}

/**
 * Detect language from file extension
 */
function detectLanguage(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase();

  const languageMap: Record<string, string> = {
    // JavaScript/TypeScript
    js: 'javascript',
    jsx: 'javascript',
    ts: 'typescript',
    tsx: 'typescript',
    mjs: 'javascript',
    cjs: 'javascript',

    // Rust
    rs: 'rust',

    // Python
    py: 'python',
    pyw: 'python',

    // Web
    html: 'html',
    htm: 'html',
    css: 'css',
    scss: 'scss',
    sass: 'sass',
    less: 'less',

    // Markdown
    md: 'markdown',
    mdx: 'markdown',

    // JSON
    json: 'json',
    jsonc: 'json',

    // YAML
    yaml: 'yaml',
    yml: 'yaml',

    // Shell
    sh: 'shell',
    bash: 'shell',
    zsh: 'shell',

    // C/C++
    c: 'c',
    cpp: 'cpp',
    cc: 'cpp',
    cxx: 'cpp',
    h: 'c',
    hpp: 'cpp',

    // Go
    go: 'go',

    // Java
    java: 'java',

    // PHP
    php: 'php',

    // Ruby
    rb: 'ruby',

    // SQL
    sql: 'sql',

    // XML
    xml: 'xml',

    // Dockerfile
    dockerfile: 'dockerfile',

    // TOML
    toml: 'toml',

    // Other
    txt: 'plaintext',
  };

  return languageMap[ext || ''] || 'plaintext';
}

/**
 * Editor store
 */
export const useEditorStore = create<EditorStore>((set, get) => ({
  tabs: [],
  activeTabId: null,

  addTab: async (path: string) => {
    const { tabs } = get();
    const id = generateTabId(path);

    // Check if tab already exists
    const existingTab = tabs.find((t) => t.id === id);
    if (existingTab) {
      // Just activate existing tab - OPTIMIZED: only update isActive for changed tabs
      set({
        tabs: tabs.map((t) =>
          t.isActive !== (t.id === id)
            ? { ...t, isActive: t.id === id }
            : t
        ),
        activeTabId: id,
      });
      return;
    }

    try {
      // Load file content from Tauri
      const fileContent = await tauriApi.openFile(path);

      const newTab: EditorTab = {
        id,
        path,
        name: getFileName(path),
        content: fileContent.content,
        language: fileContent.language ?? detectLanguage(path),
        isDirty: false,
        isActive: true,
      };

      // Add new tab and deactivate others - OPTIMIZED: only update active tab
      set({
        tabs: [
          ...tabs.map((t) =>
            t.isActive
              ? { ...t, isActive: false }
              : t
          ),
          newTab
        ],
        activeTabId: id,
      });
    } catch (error) {
      console.error('Failed to open file:', error);
      throw error;
    }
  },

  closeTab: (id: string) => {
    const { tabs, activeTabId } = get();
    const closingIndex = tabs.findIndex((t) => t.id === id);

    if (closingIndex === -1) return;

    const newTabs = tabs.filter((t) => t.id !== id);

    // Determine new active tab
    let newActiveTabId = activeTabId;

    if (activeTabId === id && newTabs.length > 0) {
      // Activate next tab (or previous if closing last)
      const nextIndex = Math.min(closingIndex, newTabs.length - 1);
      newActiveTabId = newTabs[nextIndex].id;
      newTabs[nextIndex].isActive = true;
    } else {
      newActiveTabId = newTabs.length > 0 ? newTabs[0].id : null;
    }

    set({
      tabs: newTabs,
      activeTabId: newActiveTabId,
    });
  },

  setActiveTab: (id: string) => {
    const { tabs, activeTabId } = get();

    // No-op if already active
    if (activeTabId === id) return;

    // OPTIMIZED: only update tabs that actually changed
    set({
      tabs: tabs.map((t) =>
        t.isActive !== (t.id === id)
          ? { ...t, isActive: t.id === id }
          : t
      ),
      activeTabId: id,
    });
  },

  updateTabContent: (id: string, content: string) => {
    const { tabs } = get();
    const tab = tabs.find((t) => t.id === id);

    // No-op if content hasn't changed
    if (!tab || tab.content === content) return;

    // OPTIMIZED: only create new array if content actually changed
    set({
      tabs: tabs.map((t) =>
        t.id === id
          ? {
              ...t,
              content,
              isDirty: true, // Simplified: if content changed, it's dirty
            }
          : t
      ),
    });
  },

  saveTab: async (id: string) => {
    const { tabs } = get();
    const tab = tabs.find((t) => t.id === id);

    if (!tab) return;

    try {
      await tauriApi.saveFile(tab.path, tab.content);

      // Mark as not dirty
      set({
        tabs: tabs.map((t) => (t.id === id ? { ...t, isDirty: false } : t)),
      });
    } catch (error) {
      console.error('Failed to save file:', error);
      throw error;
    }
  },

  saveAllTabs: async () => {
    const { tabs } = get();
    const dirtyTabs = tabs.filter((t) => t.isDirty);

    await Promise.all(
      dirtyTabs.map((tab) => {
        return tauriApi.saveFile(tab.path, tab.content).catch((error: unknown) => {
          console.error(`Failed to save ${tab.path}:`, error);
        });
      })
    );

    // Mark all tabs as not dirty
    set({
      tabs: tabs.map((t) => ({ ...t, isDirty: false })),
    });
  },

  closeAllTabs: () => {
    set({
      tabs: [],
      activeTabId: null,
    });
  },

  getTabById: (id: string) => {
    return get().tabs.find((t) => t.id === id);
  },

  getActiveTab: () => {
    const { tabs, activeTabId } = get();
    return tabs.find((t) => t.id === activeTabId);
  },
}));
