/**
 * AIT42 Editor - File System Hook
 */

import { useState, useCallback } from 'react';
import { useTauri, TauriCommands } from './useTauri';
import { FileNode } from '../types';

interface UseFileSystemReturn {
  loading: boolean;
  error: string | null;
  readDirectory: (path: string) => Promise<FileNode[]>;
  readFile: (path: string) => Promise<string>;
  writeFile: (path: string, content: string) => Promise<void>;
  createDirectory: (path: string) => Promise<void>;
  deleteFile: (path: string) => Promise<void>;
  renameFile: (oldPath: string, newPath: string) => Promise<void>;
}

/**
 * Custom hook for file system operations via Tauri
 *
 * Provides async file operations with loading and error states
 */
export function useFileSystem(): UseFileSystemReturn {
  const { invoke } = useTauri();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Read directory contents
   */
  const readDirectory = useCallback(async (path: string): Promise<FileNode[]> => {
    setLoading(true);
    setError(null);
    try {
      const nodes = await invoke<FileNode[]>(TauriCommands.READ_DIRECTORY, { path });
      return nodes;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to read directory';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [invoke]);

  /**
   * Read file contents
   */
  const readFile = useCallback(async (path: string): Promise<string> => {
    setLoading(true);
    setError(null);
    try {
      const content = await invoke<string>(TauriCommands.READ_FILE, { path });
      return content;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to read file';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [invoke]);

  /**
   * Write file contents
   */
  const writeFile = useCallback(async (path: string, content: string): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await invoke<void>(TauriCommands.WRITE_FILE, { path, content });
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to write file';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [invoke]);

  /**
   * Create directory
   */
  const createDirectory = useCallback(async (path: string): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await invoke<void>(TauriCommands.CREATE_DIRECTORY, { path });
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create directory';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [invoke]);

  /**
   * Delete file or directory
   */
  const deleteFile = useCallback(async (path: string): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await invoke<void>(TauriCommands.DELETE_FILE, { path });
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete file';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [invoke]);

  /**
   * Rename file or directory
   */
  const renameFile = useCallback(async (oldPath: string, newPath: string): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await invoke<void>(TauriCommands.RENAME_FILE, { oldPath, newPath });
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to rename file';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [invoke]);

  return {
    loading,
    error,
    readDirectory,
    readFile,
    writeFile,
    createDirectory,
    deleteFile,
    renameFile,
  };
}
