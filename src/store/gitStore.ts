/**
 * Git Store
 *
 * Manages Git state including repository status, branches, and commits
 */
import { create } from 'zustand';

/**
 * Git file status
 */
export interface GitFileStatus {
  path: string;
  status: string; // 'modified', 'added', 'deleted', 'untracked', 'renamed'
}

/**
 * Git status information
 */
export interface GitStatus {
  branch: string;
  ahead: number;
  behind: number;
  files: GitFileStatus[];
}

/**
 * Git commit information
 */
export interface GitCommit {
  sha: string;
  author: string;
  email: string;
  message: string;
  timestamp: number;
}

/**
 * Git store state and actions
 */
interface GitStore {
  // State
  status: GitStatus | null;
  branches: string[];
  commits: GitCommit[];
  showGitPanel: boolean;
  isLoading: boolean;
  error: string | null;

  // Status operations
  fetchStatus: () => Promise<void>;
  setStatus: (status: GitStatus) => void;

  // Branch operations
  fetchBranches: () => Promise<void>;
  setBranches: (branches: string[]) => void;
  checkoutBranch: (branch: string) => Promise<void>;
  createBranch: (name: string) => Promise<void>;

  // Commit operations
  fetchCommits: (limit?: number) => Promise<void>;
  setCommits: (commits: GitCommit[]) => void;
  addFiles: (files: string[]) => Promise<void>;
  resetFiles: (files: string[]) => Promise<void>;
  commit: (message: string) => Promise<string>;

  // Remote operations
  push: (remote?: string, branch?: string) => Promise<void>;
  pull: (remote?: string, branch?: string) => Promise<void>;

  // UI state
  showGit: () => void;
  hideGit: () => void;
  toggleGitPanel: () => void;

  // Error handling
  setError: (error: string | null) => void;
  clearError: () => void;
}

/**
 * Create Git store
 */
export const useGitStore = create<GitStore>((set, get) => ({
  // Initial state
  status: null,
  branches: [],
  commits: [],
  showGitPanel: false,
  isLoading: false,
  error: null,

  // Status operations
  fetchStatus: async () => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      const status = await tauriApi.gitStatus();
      set({ status, isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to fetch Git status';
      set({ error: message, isLoading: false });
      console.error('[Git Store] Failed to fetch status:', error);
    }
  },

  setStatus: (status: GitStatus) => {
    set({ status });
  },

  // Branch operations
  fetchBranches: async () => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      const branches = await tauriApi.gitBranches();
      set({ branches, isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to fetch branches';
      set({ error: message, isLoading: false });
      console.error('[Git Store] Failed to fetch branches:', error);
    }
  },

  setBranches: (branches: string[]) => {
    set({ branches });
  },

  checkoutBranch: async (branch: string) => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      await tauriApi.gitCheckout(branch);
      // Refresh status and branches
      await get().fetchStatus();
      await get().fetchBranches();
      set({ isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to checkout branch';
      set({ error: message, isLoading: false });
      console.error('[Git Store] Failed to checkout branch:', error);
      throw error;
    }
  },

  createBranch: async (name: string) => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      await tauriApi.gitCreateBranch(name);
      // Refresh branches
      await get().fetchBranches();
      set({ isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to create branch';
      set({ error: message, isLoading: false });
      console.error('[Git Store] Failed to create branch:', error);
      throw error;
    }
  },

  // Commit operations
  fetchCommits: async (limit?: number) => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      const commits = await tauriApi.gitLog(limit);
      set({ commits, isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to fetch commits';
      set({ error: message, isLoading: false });
      console.error('[Git Store] Failed to fetch commits:', error);
    }
  },

  setCommits: (commits: GitCommit[]) => {
    set({ commits });
  },

  addFiles: async (files: string[]) => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      await tauriApi.gitAdd(files);
      // Refresh status
      await get().fetchStatus();
      set({ isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to add files';
      set({ error: message, isLoading: false });
      console.error('[Git Store] Failed to add files:', error);
      throw error;
    }
  },

  resetFiles: async (files: string[]) => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      await tauriApi.gitReset(files);
      // Refresh status
      await get().fetchStatus();
      set({ isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to reset files';
      set({ error: message, isLoading: false });
      console.error('[Git Store] Failed to reset files:', error);
      throw error;
    }
  },

  commit: async (message: string) => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      const sha = await tauriApi.gitCommit(message);
      // Refresh status and commits
      await get().fetchStatus();
      await get().fetchCommits();
      set({ isLoading: false });
      return sha;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to commit';
      set({ error: message, isLoading: false });
      console.error('[Git Store] Failed to commit:', error);
      throw error;
    }
  },

  // Remote operations
  push: async (remote?: string, branch?: string) => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      await tauriApi.gitPush(remote, branch);
      // Refresh status
      await get().fetchStatus();
      set({ isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to push';
      set({ error: message, isLoading: false });
      console.error('[Git Store] Failed to push:', error);
      throw error;
    }
  },

  pull: async (remote?: string, branch?: string) => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      await tauriApi.gitPull(remote, branch);
      // Refresh status and commits
      await get().fetchStatus();
      await get().fetchCommits();
      set({ isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to pull';
      set({ error: message, isLoading: false });
      console.error('[Git Store] Failed to pull:', error);
      throw error;
    }
  },

  // UI state
  showGit: () => {
    set({ showGitPanel: true });
  },

  hideGit: () => {
    set({ showGitPanel: false });
  },

  toggleGitPanel: () => {
    set((state) => ({ showGitPanel: !state.showGitPanel }));
  },

  // Error handling
  setError: (error: string | null) => {
    set({ error });
  },

  clearError: () => {
    set({ error: null });
  },
}));
