/**
 * FileTree Store - Manages file tree state
 *
 * Handles directory tree state, expansion, and selection.
 */
import { create } from 'zustand';

export interface FileNode {
  name: string;
  path: string;
  isDirectory: boolean;
  children: FileNode[] | null;
}

interface FileTreeState {
  rootPath: string | null;
  tree: FileNode[];
  expandedPaths: Set<string>;
  selectedPath: string | null;
  loading: boolean;
  error: string | null;
}

interface FileTreeActions {
  setRootPath: (path: string) => void;
  setTree: (tree: FileNode[]) => void;
  toggleExpand: (path: string) => void;
  expandPath: (path: string) => void;
  selectPath: (path: string | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

type FileTreeStore = FileTreeState & FileTreeActions;

const initialState: FileTreeState = {
  rootPath: null,
  tree: [],
  expandedPaths: new Set(),
  selectedPath: null,
  loading: false,
  error: null,
};

export const useFileTreeStore = create<FileTreeStore>((set, get) => ({
  ...initialState,

  setRootPath: (path: string) => {
    set({ rootPath: path });
  },

  setTree: (tree: FileNode[]) => {
    set({ tree, error: null });
  },

  toggleExpand: (path: string) => {
    const { expandedPaths } = get();
    const newExpanded = new Set(expandedPaths);

    if (newExpanded.has(path)) {
      newExpanded.delete(path);
    } else {
      newExpanded.add(path);
    }

    set({ expandedPaths: newExpanded });
  },

  expandPath: (path: string) => {
    const { expandedPaths } = get();
    const newExpanded = new Set(expandedPaths);

    // Expand all parent directories
    const parts = path.split('/').filter(Boolean);
    let currentPath = '';

    for (let i = 0; i < parts.length - 1; i++) {
      currentPath += '/' + parts[i];
      newExpanded.add(currentPath);
    }

    set({ expandedPaths: newExpanded });
  },

  selectPath: (path: string | null) => {
    set({ selectedPath: path });
  },

  setLoading: (loading: boolean) => {
    set({ loading });
  },

  setError: (error: string | null) => {
    set({ error });
  },

  reset: () => {
    set(initialState);
  },
}));
