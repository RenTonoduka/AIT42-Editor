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
 * Terminal information structure
 */
export interface TerminalInfo {
  currentDir: string;
  outputLines: number;
  historySize: number;
  timeoutSeconds: number;
}

/**
 * LSP diagnostic information
 */
export interface LspDiagnostic {
  message: string;
  severity: number; // 1=Error, 2=Warning, 3=Information, 4=Hint
  startLine: number;
  startCharacter: number;
  endLine: number;
  endCharacter: number;
  code?: string;
  source?: string;
}

/**
 * LSP completion item
 */
export interface LspCompletionItem {
  label: string;
  kind?: number;
  detail?: string;
  documentation?: string;
  insertText?: string;
  sortText?: string;
}

/**
 * LSP hover information
 */
export interface LspHoverInfo {
  contents: string;
}

/**
 * LSP location information
 */
export interface LspLocation {
  uri: string;
  startLine: number;
  startCharacter: number;
  endLine: number;
  endCharacter: number;
}

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

  // ===== Terminal Commands =====

  /**
   * Execute a terminal command
   */
  async executeCommand(command: string): Promise<string> {
    try {
      const output = await invoke<string>('execute_command', { command });
      return output;
    } catch (error) {
      throw new Error(`Failed to execute command: ${error}`);
    }
  },

  /**
   * Get all terminal output
   */
  async getTerminalOutput(): Promise<string[]> {
    try {
      const lines = await invoke<string[]>('get_terminal_output');
      return lines;
    } catch (error) {
      throw new Error(`Failed to get terminal output: ${error}`);
    }
  },

  /**
   * Get last N lines of terminal output
   */
  async getTerminalTail(lines: number): Promise<string[]> {
    try {
      const output = await invoke<string[]>('get_terminal_tail', { lines });
      return output;
    } catch (error) {
      throw new Error(`Failed to get terminal tail: ${error}`);
    }
  },

  /**
   * Clear terminal output buffer
   */
  async clearTerminal(): Promise<void> {
    try {
      await invoke('clear_terminal');
    } catch (error) {
      throw new Error(`Failed to clear terminal: ${error}`);
    }
  },

  /**
   * Get current working directory
   */
  async getCurrentDirectory(): Promise<string> {
    try {
      const path = await invoke<string>('get_current_directory');
      return path;
    } catch (error) {
      throw new Error(`Failed to get current directory: ${error}`);
    }
  },

  /**
   * Set current working directory
   */
  async setCurrentDirectory(path: string): Promise<void> {
    try {
      await invoke('set_current_directory', { path });
    } catch (error) {
      throw new Error(`Failed to set current directory: ${error}`);
    }
  },

  /**
   * Get command history
   */
  async getCommandHistory(): Promise<string[]> {
    try {
      const history = await invoke<string[]>('get_command_history');
      return history;
    } catch (error) {
      throw new Error(`Failed to get command history: ${error}`);
    }
  },

  /**
   * Get terminal info
   */
  async getTerminalInfo(): Promise<TerminalInfo> {
    try {
      const info = await invoke<TerminalInfo>('get_terminal_info');
      return info;
    } catch (error) {
      throw new Error(`Failed to get terminal info: ${error}`);
    }
  },

  // ===== LSP Commands =====

  /**
   * Start LSP server for a specific language
   */
  async startLspServer(language: string): Promise<void> {
    try {
      await invoke('start_lsp_server', { language });
    } catch (error) {
      throw new Error(`Failed to start LSP server for ${language}: ${error}`);
    }
  },

  /**
   * Stop LSP server for a specific language
   */
  async stopLspServer(language: string): Promise<void> {
    try {
      await invoke('stop_lsp_server', { language });
    } catch (error) {
      throw new Error(`Failed to stop LSP server for ${language}: ${error}`);
    }
  },

  /**
   * Get list of running LSP servers
   */
  async getRunningLspServers(): Promise<string[]> {
    try {
      const servers = await invoke<string[]>('get_running_lsp_servers');
      return servers;
    } catch (error) {
      throw new Error(`Failed to get running LSP servers: ${error}`);
    }
  },

  /**
   * Notify LSP server that a document was opened
   */
  async lspDidOpen(filePath: string, content: string, languageId: string): Promise<void> {
    try {
      await invoke('lsp_did_open', { filePath, content, languageId });
    } catch (error) {
      throw new Error(`Failed to notify LSP of file open: ${error}`);
    }
  },

  /**
   * Notify LSP server of document changes
   */
  async lspDidChange(filePath: string, content: string, version: number): Promise<void> {
    try {
      await invoke('lsp_did_change', { filePath, content, version });
    } catch (error) {
      throw new Error(`Failed to notify LSP of changes: ${error}`);
    }
  },

  /**
   * Notify LSP server that a document was saved
   */
  async lspDidSave(filePath: string, content?: string): Promise<void> {
    try {
      await invoke('lsp_did_save', { filePath, content });
    } catch (error) {
      throw new Error(`Failed to notify LSP of save: ${error}`);
    }
  },

  /**
   * Notify LSP server that a document was closed
   */
  async lspDidClose(filePath: string): Promise<void> {
    try {
      await invoke('lsp_did_close', { filePath });
    } catch (error) {
      throw new Error(`Failed to notify LSP of close: ${error}`);
    }
  },

  /**
   * Get completion suggestions at a specific position
   */
  async lspCompletion(
    filePath: string,
    line: number,
    character: number
  ): Promise<LspCompletionItem[]> {
    try {
      const completions = await invoke<LspCompletionItem[]>('lsp_completion', {
        filePath,
        line,
        character,
      });
      return completions;
    } catch (error) {
      throw new Error(`Failed to get completions: ${error}`);
    }
  },

  /**
   * Get hover information at a specific position
   */
  async lspHover(
    filePath: string,
    line: number,
    character: number
  ): Promise<LspHoverInfo | null> {
    try {
      const hover = await invoke<LspHoverInfo | null>('lsp_hover', {
        filePath,
        line,
        character,
      });
      return hover;
    } catch (error) {
      throw new Error(`Failed to get hover info: ${error}`);
    }
  },

  /**
   * Go to definition of symbol at a specific position
   */
  async lspGotoDefinition(
    filePath: string,
    line: number,
    character: number
  ): Promise<LspLocation[]> {
    try {
      const locations = await invoke<LspLocation[]>('lsp_goto_definition', {
        filePath,
        line,
        character,
      });
      return locations;
    } catch (error) {
      throw new Error(`Failed to get definition: ${error}`);
    }
  },

  /**
   * Get diagnostics for a specific file
   */
  async lspDiagnostics(filePath: string): Promise<LspDiagnostic[]> {
    try {
      const diagnostics = await invoke<LspDiagnostic[]>('lsp_diagnostics', {
        filePath,
      });
      return diagnostics;
    } catch (error) {
      throw new Error(`Failed to get diagnostics: ${error}`);
    }
  },

  // ===== Git Commands =====

  /**
   * Get Git status for current repository
   */
  async gitStatus(): Promise<GitStatus> {
    try {
      const status = await invoke<GitStatus>('git_status');
      return status;
    } catch (error) {
      throw new Error(`Failed to get Git status: ${error}`);
    }
  },

  /**
   * Stage files for commit
   */
  async gitAdd(files: string[]): Promise<void> {
    try {
      await invoke('git_add', { files });
    } catch (error) {
      throw new Error(`Failed to add files: ${error}`);
    }
  },

  /**
   * Unstage files
   */
  async gitReset(files: string[]): Promise<void> {
    try {
      await invoke('git_reset', { files });
    } catch (error) {
      throw new Error(`Failed to reset files: ${error}`);
    }
  },

  /**
   * Create a commit
   */
  async gitCommit(message: string): Promise<string> {
    try {
      const sha = await invoke<string>('git_commit', { message });
      return sha;
    } catch (error) {
      throw new Error(`Failed to commit: ${error}`);
    }
  },

  /**
   * Push to remote
   */
  async gitPush(remote?: string, branch?: string): Promise<void> {
    try {
      await invoke('git_push', { remote, branch });
    } catch (error) {
      throw new Error(`Failed to push: ${error}`);
    }
  },

  /**
   * Pull from remote
   */
  async gitPull(remote?: string, branch?: string): Promise<void> {
    try {
      await invoke('git_pull', { remote, branch });
    } catch (error) {
      throw new Error(`Failed to pull: ${error}`);
    }
  },

  /**
   * Get commit history
   */
  async gitLog(limit?: number): Promise<GitCommit[]> {
    try {
      const commits = await invoke<GitCommit[]>('git_log', { limit });
      return commits;
    } catch (error) {
      throw new Error(`Failed to get commit history: ${error}`);
    }
  },

  /**
   * Get list of branches
   */
  async gitBranches(): Promise<string[]> {
    try {
      const branches = await invoke<string[]>('git_branches');
      return branches;
    } catch (error) {
      throw new Error(`Failed to get branches: ${error}`);
    }
  },

  /**
   * Checkout branch
   */
  async gitCheckout(branch: string): Promise<void> {
    try {
      await invoke('git_checkout', { branch });
    } catch (error) {
      throw new Error(`Failed to checkout branch: ${error}`);
    }
  },

  /**
   * Create new branch
   */
  async gitCreateBranch(name: string): Promise<void> {
    try {
      await invoke('git_create_branch', { name });
    } catch (error) {
      throw new Error(`Failed to create branch: ${error}`);
    }
  },
};
