/**
 * Tauri command bindings for communication with Rust backend
 */
import { invoke } from '@tauri-apps/api/core';
import type {
  FileContent,
  FileTreeNode,
  Diagnostic,
  CompletionItem,
} from '@/types';

export const tauriCommands = {
  /**
   * Open and read a file
   */
  async openFile(path: string): Promise<FileContent> {
    return invoke('open_file', { path });
  },

  /**
   * Save file content
   */
  async saveFile(path: string, content: string): Promise<void> {
    return invoke('save_file', { path, content });
  },

  /**
   * Create a new file
   */
  async createFile(path: string): Promise<void> {
    return invoke('create_file', { path });
  },

  /**
   * Get file tree for a directory
   */
  async getFileTree(rootPath: string): Promise<FileTreeNode> {
    return invoke('get_file_tree', { rootPath });
  },

  /**
   * Search files by pattern
   */
  async searchFiles(rootPath: string, pattern: string): Promise<string[]> {
    return invoke('search_files', { rootPath, pattern });
  },

  /**
   * Format document using LSP
   */
  async formatDocument(path: string, content: string): Promise<string> {
    return invoke('format_document', { path, content });
  },

  /**
   * Get diagnostics for a file
   */
  async getDiagnostics(path: string): Promise<Diagnostic[]> {
    return invoke('get_diagnostics', { path });
  },

  /**
   * Get completions at cursor position
   */
  async getCompletions(
    path: string,
    line: number,
    character: number,
  ): Promise<CompletionItem[]> {
    return invoke('get_completions', { path, line, character });
  },

  /**
   * Go to definition
   */
  async gotoDefinition(
    path: string,
    line: number,
    character: number,
  ): Promise<FileContent | null> {
    return invoke('goto_definition', { path, line, character });
  },
};
