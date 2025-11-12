/**
 * Worktree Store
 *
 * Manages worktree state: list, selection, and file tree
 */
import { create } from 'zustand';
import { worktreeApi, type WorktreeInfo, type FileNode } from '@/services/worktree';

/**
 * Worktree store state and actions
 */
interface WorktreeStore {
  /** All worktrees for current competition */
  worktrees: WorktreeInfo[];
  /** Currently selected worktree ID */
  selectedWorktree: string | null;
  /** File tree for selected worktree */
  fileTree: FileNode[];
  /** Currently selected file path */
  selectedFile: string | null;
  /** Content of selected file */
  fileContent: string | null;
  /** Search query for filtering files */
  searchQuery: string;
  /** Loading state */
  isLoading: boolean;
  /** Loading state for file content */
  isLoadingFile: boolean;
  /** Error message */
  error: string | null;

  // Actions
  /** Fetch worktrees for a specific competition */
  fetchWorktrees: (competitionId: string) => Promise<void>;
  /** Manually set worktrees (for session integration) */
  setWorktrees: (worktrees: WorktreeInfo[]) => void;
  /** Select a worktree by ID */
  selectWorktree: (id: string) => void;
  /** Load file tree for selected worktree */
  loadFileTree: (worktreePath: string) => Promise<void>;
  /** Select a file and load its content */
  selectFile: (filePath: string) => Promise<void>;
  /** Set search query for filtering */
  setSearchQuery: (query: string) => void;
  /** Delete a worktree by ID */
  deleteWorktree: (id: string) => Promise<void>;
  /** Reset store to initial state */
  reset: () => void;
}

/**
 * Initial state
 */
const initialState = {
  worktrees: [],
  selectedWorktree: null,
  fileTree: [],
  selectedFile: null,
  fileContent: null,
  searchQuery: '',
  isLoading: false,
  isLoadingFile: false,
  error: null,
};

/**
 * Worktree store
 */
export const useWorktreeStore = create<WorktreeStore>((set, get) => ({
  ...initialState,

  fetchWorktrees: async (competitionId: string) => {
    set({ isLoading: true, error: null });

    try {
      const worktrees = await worktreeApi.listWorktrees(competitionId);
      set({ worktrees, isLoading: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to fetch worktrees',
        isLoading: false,
      });
    }
  },

  setWorktrees: (worktrees: WorktreeInfo[]) => {
    console.log('[worktreeStore] setWorktrees called with:', worktrees.length, 'worktrees');
    set({ worktrees });
  },

  selectWorktree: (id: string) => {
    const { worktrees } = get();
    const worktree = worktrees.find((w) => w.id === id);

    if (worktree) {
      set({ selectedWorktree: id });
      // Auto-load file tree when selecting
      get().loadFileTree(worktree.path);
    }
  },

  loadFileTree: async (worktreePath: string) => {
    set({ isLoading: true, error: null });

    try {
      const fileTree = await worktreeApi.getWorktreeFiles(worktreePath, 3);
      set({ fileTree, isLoading: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to load file tree',
        isLoading: false,
      });
    }
  },

  selectFile: async (filePath: string) => {
    set({ selectedFile: filePath, isLoadingFile: true, error: null });

    try {
      // Use Tauri fs API to read file
      const { readTextFile } = await import('@tauri-apps/api/fs');
      const content = await readTextFile(filePath);

      set({ fileContent: content, isLoadingFile: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to load file content',
        fileContent: null,
        isLoadingFile: false,
      });
    }
  },

  setSearchQuery: (query: string) => {
    set({ searchQuery: query });
  },

  deleteWorktree: async (id: string) => {
    set({ isLoading: true, error: null });

    try {
      await worktreeApi.deleteWorktree(id);

      // Remove from local state
      const { worktrees, selectedWorktree } = get();
      const newWorktrees = worktrees.filter((w) => w.id !== id);

      set({
        worktrees: newWorktrees,
        selectedWorktree: selectedWorktree === id ? null : selectedWorktree,
        fileTree: selectedWorktree === id ? [] : get().fileTree,
        isLoading: false,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to delete worktree',
        isLoading: false,
      });
    }
  },

  reset: () => {
    set(initialState);
  },
}));
