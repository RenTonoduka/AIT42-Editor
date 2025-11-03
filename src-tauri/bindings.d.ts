/**
 * TypeScript Type Definitions for AIT42 Editor Tauri Commands
 *
 * These types correspond to the Rust backend commands defined in:
 * - src-tauri/src/commands/file.rs
 * - src-tauri/src/commands/editor.rs
 * - src-tauri/src/commands/terminal.rs
 */

// ============================================================================
// File Operations
// ============================================================================

export interface FileNode {
  name: string;
  path: string;
  isDirectory: boolean;
  children?: FileNode[];
}

export interface OpenFileResponse {
  bufferId: string;
  content: string;
  path: string;
  language?: string;
}

export interface FileCommands {
  /**
   * Open a file and return its content
   * @param path - File path to open
   * @returns File content with buffer info
   */
  openFile(path: string): Promise<OpenFileResponse>;

  /**
   * Save file content to disk
   * @param path - File path to save to
   * @param content - File content to write
   */
  saveFile(path: string, content: string): Promise<void>;

  /**
   * Read directory contents
   * @param path - Directory path to read
   * @returns List of file nodes
   */
  readDirectory(path: string): Promise<FileNode[]>;

  /**
   * Create a new file
   * @param path - File path to create
   */
  createFile(path: string): Promise<void>;

  /**
   * Create a new directory
   * @param path - Directory path to create
   */
  createDirectory(path: string): Promise<void>;

  /**
   * Delete a file or directory
   * @param path - Path to delete
   */
  deletePath(path: string): Promise<void>;

  /**
   * Rename/move a file or directory
   * @param oldPath - Current path
   * @param newPath - New path
   */
  renamePath(oldPath: string, newPath: string): Promise<void>;
}

// ============================================================================
// Editor Operations
// ============================================================================

export interface Position {
  line: number;
  col: number;
}

export interface TextRange {
  start: number;
  end: number;
}

export interface BufferInfo {
  bufferId: string;
  path?: string;
  language?: string;
  isDirty: boolean;
  lineCount: number;
  charCount: number;
  byteCount: number;
}

export interface EditorCommands {
  /**
   * Insert text at byte position
   * @param bufferId - Buffer ID (UUID string)
   * @param position - Byte offset position
   * @param text - Text to insert
   */
  insertText(bufferId: string, position: number, text: string): Promise<void>;

  /**
   * Delete text in range
   * @param bufferId - Buffer ID (UUID string)
   * @param range - Range to delete (start, end byte offsets)
   */
  deleteText(bufferId: string, range: TextRange): Promise<void>;

  /**
   * Replace text in range
   * @param bufferId - Buffer ID (UUID string)
   * @param range - Range to replace (start, end byte offsets)
   * @param text - Replacement text
   */
  replaceText(bufferId: string, range: TextRange, text: string): Promise<void>;

  /**
   * Undo last edit
   * @param bufferId - Buffer ID (UUID string)
   */
  undo(bufferId: string): Promise<void>;

  /**
   * Redo last undone edit
   * @param bufferId - Buffer ID (UUID string)
   */
  redo(bufferId: string): Promise<void>;

  /**
   * Get buffer content
   * @param bufferId - Buffer ID (UUID string)
   * @returns Buffer content as string
   */
  getBufferContent(bufferId: string): Promise<string>;

  /**
   * Get buffer info
   * @param bufferId - Buffer ID (UUID string)
   * @returns Buffer information
   */
  getBufferInfo(bufferId: string): Promise<BufferInfo>;

  /**
   * Close buffer
   * @param bufferId - Buffer ID (UUID string)
   * @param force - Force close even if dirty
   */
  closeBuffer(bufferId: string, force: boolean): Promise<void>;

  /**
   * List all open buffers
   * @returns List of buffer IDs
   */
  listBuffers(): Promise<string[]>;
}

// ============================================================================
// Terminal Operations
// ============================================================================

export interface TerminalInfo {
  currentDir: string;
  outputLines: number;
  historySize: number;
  timeoutSeconds: number;
}

export interface TerminalCommands {
  /**
   * Execute a terminal command
   * @param command - Command string to execute
   * @returns Command output (captured immediately after execution)
   */
  executeCommand(command: string): Promise<string>;

  /**
   * Get terminal output buffer
   * @returns Array of output lines
   */
  getTerminalOutput(): Promise<string[]>;

  /**
   * Get last N lines of terminal output
   * @param lines - Number of lines to get
   * @returns Array of output lines
   */
  getTerminalTail(lines: number): Promise<string[]>;

  /**
   * Clear terminal output buffer
   */
  clearTerminal(): Promise<void>;

  /**
   * Get current working directory
   * @returns Current working directory path
   */
  getCurrentDirectory(): Promise<string>;

  /**
   * Set current working directory
   * @param path - New working directory path
   */
  setCurrentDirectory(path: string): Promise<void>;

  /**
   * Get command history
   * @returns Array of command strings (most recent first)
   */
  getCommandHistory(): Promise<string[]>;

  /**
   * Get terminal info
   * @returns Terminal information
   */
  getTerminalInfo(): Promise<TerminalInfo>;
}

// ============================================================================
// Combined API
// ============================================================================

export interface AIT42API extends FileCommands, EditorCommands, TerminalCommands {}

// Type augmentation for window.__TAURI__
declare global {
  interface Window {
    __TAURI__: {
      invoke<T = any>(cmd: string, args?: Record<string, any>): Promise<T>;
    };
  }
}
