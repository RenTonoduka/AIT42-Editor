/**
 * Debug Store - Manages debugging state
 *
 * Uses Zustand for lightweight state management
 */

import { create } from 'zustand';

/**
 * Debug configuration
 */
export interface DebugConfiguration {
  name: string;
  type: string; // 'rust', 'node', 'python', etc.
  request: 'launch' | 'attach';
  program?: string;
  args?: string[];
  cwd?: string;
  env?: Record<string, string>;
}

/**
 * Breakpoint information
 */
export interface Breakpoint {
  id: string;
  filePath: string;
  line: number;
  enabled: boolean;
  condition?: string;
}

/**
 * Debug session state
 */
export type DebugState = 'stopped' | 'running' | 'paused' | 'terminated';

/**
 * Debug store state and actions
 */
interface DebugStore {
  /** Current debug state */
  state: DebugState;
  /** Active debug configuration */
  activeConfig: DebugConfiguration | null;
  /** Available debug configurations */
  configurations: DebugConfiguration[];
  /** Breakpoints by file path */
  breakpoints: Map<string, Breakpoint[]>;
  /** Whether debug panel is visible */
  showDebugPanel: boolean;
  /** Current debug output lines */
  outputLines: string[];

  // Actions
  /** Start debugging with configuration */
  startDebugging: (config: DebugConfiguration) => void;
  /** Stop debugging session */
  stopDebugging: () => void;
  /** Pause execution */
  pauseDebugging: () => void;
  /** Continue execution */
  continueDebugging: () => void;
  /** Step over */
  stepOver: () => void;
  /** Step into */
  stepInto: () => void;
  /** Step out */
  stepOut: () => void;

  /** Add debug configuration */
  addConfiguration: (config: DebugConfiguration) => void;
  /** Remove debug configuration */
  removeConfiguration: (name: string) => void;
  /** Get configuration by name */
  getConfiguration: (name: string) => DebugConfiguration | undefined;

  /** Toggle breakpoint */
  toggleBreakpoint: (filePath: string, line: number) => void;
  /** Remove breakpoint */
  removeBreakpoint: (id: string) => void;
  /** Get breakpoints for file */
  getFileBreakpoints: (filePath: string) => Breakpoint[];
  /** Get all breakpoints */
  getAllBreakpoints: () => Breakpoint[];

  /** Add debug output line */
  addOutputLine: (line: string) => void;
  /** Clear debug output */
  clearOutput: () => void;

  /** Show debug panel */
  showDebug: () => void;
  /** Hide debug panel */
  hideDebug: () => void;
  /** Toggle debug panel */
  toggleDebugPanel: () => void;
}

/**
 * Generate unique breakpoint ID
 */
function generateBreakpointId(filePath: string, line: number): string {
  return `${filePath}:${line}`;
}

/**
 * Debug store
 */
export const useDebugStore = create<DebugStore>((set, get) => ({
  state: 'stopped',
  activeConfig: null,
  configurations: [],
  breakpoints: new Map(),
  showDebugPanel: false,
  outputLines: [],

  startDebugging: (config: DebugConfiguration) => {
    set({
      state: 'running',
      activeConfig: config,
      outputLines: [`Starting debug session: ${config.name}...`],
    });
    // TODO: Implement actual DAP connection
    console.log('[Debug] Starting debugging with config:', config);
  },

  stopDebugging: () => {
    const currentConfig = get().activeConfig;
    set((state) => ({
      state: 'terminated',
      activeConfig: null,
      outputLines: [
        ...state.outputLines,
        `Debug session terminated: ${currentConfig?.name || 'unknown'}`,
      ],
    }));
    console.log('[Debug] Stopping debugging');
  },

  pauseDebugging: () => {
    set((state) => ({
      state: 'paused',
      outputLines: [...state.outputLines, 'Execution paused'],
    }));
    console.log('[Debug] Paused');
  },

  continueDebugging: () => {
    set((state) => ({
      state: 'running',
      outputLines: [...state.outputLines, 'Continuing execution...'],
    }));
    console.log('[Debug] Continue');
  },

  stepOver: () => {
    console.log('[Debug] Step over');
  },

  stepInto: () => {
    console.log('[Debug] Step into');
  },

  stepOut: () => {
    console.log('[Debug] Step out');
  },

  addConfiguration: (config: DebugConfiguration) => {
    set((state) => ({
      configurations: [...state.configurations, config],
    }));
  },

  removeConfiguration: (name: string) => {
    set((state) => ({
      configurations: state.configurations.filter((c) => c.name !== name),
    }));
  },

  getConfiguration: (name: string) => {
    return get().configurations.find((c) => c.name === name);
  },

  toggleBreakpoint: (filePath: string, line: number) => {
    set((state) => {
      const newBreakpoints = new Map(state.breakpoints);
      const fileBreakpoints = newBreakpoints.get(filePath) || [];

      // Check if breakpoint already exists at this line
      const existingIndex = fileBreakpoints.findIndex((bp) => bp.line === line);

      if (existingIndex >= 0) {
        // Remove existing breakpoint
        fileBreakpoints.splice(existingIndex, 1);
      } else {
        // Add new breakpoint
        const newBreakpoint: Breakpoint = {
          id: generateBreakpointId(filePath, line),
          filePath,
          line,
          enabled: true,
        };
        fileBreakpoints.push(newBreakpoint);
      }

      if (fileBreakpoints.length === 0) {
        newBreakpoints.delete(filePath);
      } else {
        newBreakpoints.set(filePath, fileBreakpoints);
      }

      return { breakpoints: newBreakpoints };
    });
  },

  removeBreakpoint: (id: string) => {
    set((state) => {
      const newBreakpoints = new Map(state.breakpoints);

      for (const [filePath, breakpoints] of newBreakpoints) {
        const filtered = breakpoints.filter((bp) => bp.id !== id);
        if (filtered.length === 0) {
          newBreakpoints.delete(filePath);
        } else {
          newBreakpoints.set(filePath, filtered);
        }
      }

      return { breakpoints: newBreakpoints };
    });
  },

  getFileBreakpoints: (filePath: string) => {
    return get().breakpoints.get(filePath) || [];
  },

  getAllBreakpoints: () => {
    const allBreakpoints: Breakpoint[] = [];
    for (const breakpoints of get().breakpoints.values()) {
      allBreakpoints.push(...breakpoints);
    }
    return allBreakpoints;
  },

  addOutputLine: (line: string) => {
    set((state) => ({
      outputLines: [...state.outputLines, line],
    }));
  },

  clearOutput: () => {
    set({ outputLines: [] });
  },

  showDebug: () => {
    set({ showDebugPanel: true });
  },

  hideDebug: () => {
    set({ showDebugPanel: false });
  },

  toggleDebugPanel: () => {
    set((state) => ({ showDebugPanel: !state.showDebugPanel }));
  },
}));
