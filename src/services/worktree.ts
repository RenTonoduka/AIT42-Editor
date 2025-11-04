/**
 * Worktree API Service
 *
 * Provides type-safe wrappers for worktree-related Tauri commands
 */
import { invoke } from '@tauri-apps/api/tauri';

/**
 * Worktree information structure
 */
export interface WorktreeInfo {
  id: string;
  path: string;
  branch: string;
  status: string;
  created_at: string;
  changed_files: number;
}

/**
 * File node structure for worktree file tree
 */
export interface FileNode {
  name: string;
  path: string;
  is_directory: boolean;
  children?: FileNode[];
  git_status?: string;
}

/**
 * Worktree API operations
 */
export const worktreeApi = {
  /**
   * List all worktrees for a specific competition
   *
   * @param competitionId - Competition ID to filter worktrees
   * @returns Array of worktree information
   */
  listWorktrees: async (competitionId: string): Promise<WorktreeInfo[]> => {
    try {
      const worktrees = await invoke<WorktreeInfo[]>('list_worktrees', { competitionId });
      return worktrees;
    } catch (error) {
      throw new Error(`Failed to list worktrees: ${error}`);
    }
  },

  /**
   * Get file tree for a specific worktree
   *
   * @param worktreePath - Absolute path to worktree
   * @param maxDepth - Maximum directory depth (default: 3)
   * @returns Array of file nodes (root level)
   */
  getWorktreeFiles: async (worktreePath: string, maxDepth: number = 3): Promise<FileNode[]> => {
    try {
      const files = await invoke<FileNode[]>('get_worktree_files', { worktreePath, maxDepth });
      return files;
    } catch (error) {
      throw new Error(`Failed to get worktree files: ${error}`);
    }
  },

  /**
   * Delete a worktree
   *
   * @param worktreeId - Worktree ID to delete
   */
  deleteWorktree: async (worktreeId: string): Promise<void> => {
    try {
      await invoke('delete_worktree', { worktreeId });
    } catch (error) {
      throw new Error(`Failed to delete worktree: ${error}`);
    }
  },
};
