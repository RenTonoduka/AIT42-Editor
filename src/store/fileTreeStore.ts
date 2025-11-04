/**
 * FileTree Store - Manages file tree state
 *
 * Handles directory tree state, expansion, and selection.
 * Uses Zustand persist middleware to maintain state across hot reloads.
 */
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';

export interface FileNode {
  name: string;
  path: string;
  isDirectory: boolean;
  children: FileNode[] | null;
}

interface FileTreeState {
  rootPath: string | null;
  tree: FileNode[];
  expandedPaths: string[]; // Changed from Set to array for serialization
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
  expandedPaths: [], // Changed from Set to array
  selectedPath: null,
  loading: false,
  error: null,
};

export const useFileTreeStore = create<FileTreeStore>()(
  persist(
    (set, get) => ({
      ...initialState,

      setRootPath: (path: string) => {
        set({ rootPath: path });
      },

      setTree: (tree: FileNode[]) => {
        set({ tree, error: null });
      },

      toggleExpand: (path: string) => {
        const { expandedPaths } = get();

        if (expandedPaths.includes(path)) {
          set({ expandedPaths: expandedPaths.filter(p => p !== path) });
        } else {
          set({ expandedPaths: [...expandedPaths, path] });
        }
      },

      expandPath: (path: string) => {
        const { expandedPaths } = get();
        const newExpanded = [...expandedPaths];

        // Expand all parent directories
        const parts = path.split('/').filter(Boolean);
        let currentPath = '';

        for (let i = 0; i < parts.length - 1; i++) {
          currentPath += '/' + parts[i];
          if (!newExpanded.includes(currentPath)) {
            newExpanded.push(currentPath);
          }
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
    }),
    {
      name: 'ait42-file-tree-storage',
      storage: createJSONStorage(() => localStorage),
      // Only persist UI state (expandedPaths and selectedPath)
      partialize: (state) => ({
        expandedPaths: state.expandedPaths,
        selectedPath: state.selectedPath,
      }),
    }
  )
);
