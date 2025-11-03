/**
 * Tauri command bindings for communication with Rust backend
 */
import { invoke } from '@tauri-apps/api/tauri';

/**
 * Response from open_file command
 */
export interface OpenFileResponse {
  bufferId: string;
  content: string;
  path: string;
  language: string | null;
}

/**
 * File node structure from backend
 */
export interface FileNode {
  name: string;
  path: string;
  isDirectory: boolean;
  children: FileNode[] | null;
}

/**
 * Type-safe Tauri command wrappers
 */
export const tauriApi = {
  /**
   * Open a file and return its content with buffer info
   */
  async openFile(path: string): Promise<OpenFileResponse> {
    try {
      const response = await invoke<OpenFileResponse>('open_file', { path });
      return response;
    } catch (error) {
      throw new Error(`Failed to open file: ${error}`);
    }
  },

  /**
   * Save file content to disk
   */
  async saveFile(path: string, content: string): Promise<void> {
    try {
      await invoke('save_file', { path, content });
    } catch (error) {
      throw new Error(`Failed to save file: ${error}`);
    }
  },

  /**
   * Read directory contents
   */
  async readDirectory(path: string): Promise<FileNode[]> {
    try {
      const nodes = await invoke<FileNode[]>('read_directory', { path });
      return nodes;
    } catch (error) {
      throw new Error(`Failed to read directory: ${error}`);
    }
  },

  /**
   * Create a new file
   */
  async createFile(path: string): Promise<void> {
    try {
      await invoke('create_file', { path });
    } catch (error) {
      throw new Error(`Failed to create file: ${error}`);
    }
  },

  /**
   * Create a new directory
   */
  async createDirectory(path: string): Promise<void> {
    try {
      await invoke('create_directory', { path });
    } catch (error) {
      throw new Error(`Failed to create directory: ${error}`);
    }
  },

  /**
   * Delete a file or directory
   */
  async deletePath(path: string): Promise<void> {
    try {
      await invoke('delete_path', { path });
    } catch (error) {
      throw new Error(`Failed to delete: ${error}`);
    }
  },

  /**
   * Rename/move a file or directory
   */
  async renamePath(oldPath: string, newPath: string): Promise<void> {
    try {
      await invoke('rename_path', { oldPath, newPath });
    } catch (error) {
      throw new Error(`Failed to rename: ${error}`);
    }
  },
};
