/**
 * AIT42 Editor - Tauri Integration Hook
 */

import { invoke } from '@tauri-apps/api/tauri';

/**
 * Custom hook for Tauri command invocations
 *
 * Provides type-safe wrappers around Tauri IPC commands
 */
export function useTauri() {
  /**
   * Invoke a Tauri command with type safety
   */
  const invokeCommand = async <T>(
    command: string,
    args?: Record<string, unknown>
  ): Promise<T> => {
    try {
      const result = await invoke<T>(command, args);
      return result;
    } catch (error) {
      console.error(`Tauri command '${command}' failed:`, error);
      throw error;
    }
  };

  return {
    invoke: invokeCommand,
  };
}

/**
 * Type-safe Tauri commands
 */
export const TauriCommands = {
  // File System
  READ_DIRECTORY: 'read_directory',
  READ_FILE: 'read_file',
  WRITE_FILE: 'write_file',
  CREATE_DIRECTORY: 'create_directory',
  DELETE_FILE: 'delete_file',
  RENAME_FILE: 'rename_file',

  // Agent Management
  LIST_AGENTS: 'list_agents',
  RUN_AGENT: 'run_agent',
  RUN_COORDINATOR: 'run_coordinator',

  // Tmux Session Management
  LIST_TMUX_SESSIONS: 'list_tmux_sessions',
  CAPTURE_TMUX_OUTPUT: 'capture_tmux_output',
  KILL_TMUX_SESSION: 'kill_tmux_session',
  ATTACH_TMUX_SESSION: 'attach_tmux_session',

  // Terminal
  SPAWN_SHELL: 'spawn_shell',
  SEND_SHELL_INPUT: 'send_shell_input',
  RESIZE_TERMINAL: 'resize_terminal',
} as const;
