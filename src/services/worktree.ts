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
 * Diff line structure
 */
export interface DiffLine {
  line_type: 'add' | 'delete' | 'context';
  content: string;
  old_line_num?: number;
  new_line_num?: number;
}

/**
 * Diff hunk structure
 */
export interface DiffHunk {
  old_start: number;
  old_lines: number;
  new_start: number;
  new_lines: number;
  lines: DiffLine[];
}

/**
 * File diff structure
 */
export interface FileDiff {
  file_path: string;
  old_path?: string;
  change_type: 'modified' | 'added' | 'deleted' | 'renamed';
  hunks: DiffHunk[];
  additions: number;
  deletions: number;
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

  /**
   * Get git diff for a specific file in a worktree
   *
   * @param worktreePath - Absolute path to worktree
   * @param filePath - Relative path to file within worktree
   * @returns File diff information
   */
  getFileDiff: async (worktreePath: string, filePath: string): Promise<FileDiff> => {
    try {
      const diff = await invoke<FileDiff>('get_file_diff', { worktreePath, filePath });
      return diff;
    } catch (error) {
      throw new Error(`Failed to get file diff: ${error}`);
    }
  },

  /**
   * Get git diff for entire worktree
   *
   * @param worktreePath - Absolute path to worktree
   * @returns Array of file diffs
   */
  getWorktreeDiff: async (worktreePath: string): Promise<FileDiff[]> => {
    try {
      const diffs = await invoke<FileDiff[]>('get_worktree_diff', { worktreePath });
      return diffs;
    } catch (error) {
      throw new Error(`Failed to get worktree diff: ${error}`);
    }
  },
};
