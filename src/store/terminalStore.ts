/**
 * Terminal Store - Manages terminal state
 *
 * Uses Zustand for lightweight state management
 */

import { create } from 'zustand';
import type { Terminal as XTerm } from '@xterm/xterm';

/**
 * Terminal store state and actions
 */
interface TerminalStore {
  /** Whether terminal is visible */
  isVisible: boolean;
  /** Terminal height in pixels */
  height: number;
  /** XTerm instance reference */
  xtermInstance: XTerm | null;
  /** Current working directory */
  currentDirectory: string;
  /** Command history */
  commandHistory: string[];
  /** Whether terminal is being resized */
  isResizing: boolean;

  // Actions
  /** Toggle terminal visibility */
  toggleTerminal: () => void;
  /** Show terminal */
  showTerminal: () => void;
  /** Hide terminal */
  hideTerminal: () => void;
  /** Set terminal height */
  setHeight: (height: number) => void;
  /** Set xterm instance */
  setXTermInstance: (instance: XTerm | null) => void;
  /** Set current directory */
  setCurrentDirectory: (dir: string) => void;
  /** Set command history */
  setCommandHistory: (history: string[]) => void;
  /** Set resizing state */
  setResizing: (isResizing: boolean) => void;
}

/**
 * Default terminal height
 */
const DEFAULT_TERMINAL_HEIGHT = 250;

/**
 * Minimum terminal height
 */
export const MIN_TERMINAL_HEIGHT = 100;

/**
 * Maximum terminal height (80% of viewport)
 */
export const MAX_TERMINAL_HEIGHT_RATIO = 0.8;

/**
 * Terminal store
 */
export const useTerminalStore = create<TerminalStore>((set) => ({
  isVisible: false,
  height: DEFAULT_TERMINAL_HEIGHT,
  xtermInstance: null,
  currentDirectory: '',
  commandHistory: [],
  isResizing: false,

  toggleTerminal: () => {
    set((state) => ({ isVisible: !state.isVisible }));
  },

  showTerminal: () => {
    set({ isVisible: true });
  },

  hideTerminal: () => {
    set({ isVisible: false });
  },

  setHeight: (height: number) => {
    // Clamp height to reasonable bounds
    const clampedHeight = Math.max(
      MIN_TERMINAL_HEIGHT,
      Math.min(height, window.innerHeight * MAX_TERMINAL_HEIGHT_RATIO)
    );

    set({ height: clampedHeight });
  },

  setXTermInstance: (instance: XTerm | null) => {
    set({ xtermInstance: instance });
  },

  setCurrentDirectory: (dir: string) => {
    set({ currentDirectory: dir });
  },

  setCommandHistory: (history: string[]) => {
    set({ commandHistory: history });
  },

  setResizing: (isResizing: boolean) => {
    set({ isResizing });
  },
}));
