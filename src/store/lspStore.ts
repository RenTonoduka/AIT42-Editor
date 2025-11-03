/**
 * LSP Store - Manages LSP state
 *
 * Uses Zustand for lightweight state management
 */

import { create } from 'zustand';
import type { LspDiagnostic } from '@/services/tauri';

/**
 * File diagnostics map
 */
export interface FileDiagnostics {
  filePath: string;
  diagnostics: LspDiagnostic[];
  errorCount: number;
  warningCount: number;
  infoCount: number;
  hintCount: number;
  lastUpdated: number;
}

/**
 * LSP document version tracking
 */
export interface DocumentVersion {
  filePath: string;
  version: number;
}

/**
 * LSP store state and actions
 */
interface LspStore {
  /** Running LSP servers by language ID */
  runningServers: Set<string>;
  /** Diagnostics by file path */
  diagnostics: Map<string, FileDiagnostics>;
  /** Document versions for change tracking */
  documentVersions: Map<string, number>;
  /** Whether LSP is enabled globally */
  enabled: boolean;
  /** Whether to show diagnostics panel */
  showDiagnosticsPanel: boolean;

  // Actions
  /** Add running server */
  addRunningServer: (language: string) => void;
  /** Remove running server */
  removeRunningServer: (language: string) => void;
  /** Set running servers */
  setRunningServers: (servers: string[]) => void;
  /** Check if server is running */
  isServerRunning: (language: string) => boolean;

  /** Set diagnostics for a file */
  setFileDiagnostics: (filePath: string, diagnostics: LspDiagnostic[]) => void;
  /** Clear diagnostics for a file */
  clearFileDiagnostics: (filePath: string) => void;
  /** Get diagnostics for a file */
  getFileDiagnostics: (filePath: string) => FileDiagnostics | undefined;
  /** Get all diagnostics */
  getAllDiagnostics: () => FileDiagnostics[];
  /** Get total error count across all files */
  getTotalErrorCount: () => number;
  /** Get total warning count across all files */
  getTotalWarningCount: () => number;

  /** Get document version */
  getDocumentVersion: (filePath: string) => number;
  /** Increment document version */
  incrementDocumentVersion: (filePath: string) => number;
  /** Reset document version */
  resetDocumentVersion: (filePath: string) => void;

  /** Enable LSP */
  enableLsp: () => void;
  /** Disable LSP */
  disableLsp: () => void;
  /** Toggle diagnostics panel */
  toggleDiagnosticsPanel: () => void;
  /** Show diagnostics panel */
  showDiagnostics: () => void;
  /** Hide diagnostics panel */
  hideDiagnostics: () => void;
}

/**
 * Count diagnostics by severity
 */
function countBySeverity(diagnostics: LspDiagnostic[]) {
  let errorCount = 0;
  let warningCount = 0;
  let infoCount = 0;
  let hintCount = 0;

  for (const diag of diagnostics) {
    switch (diag.severity) {
      case 1:
        errorCount++;
        break;
      case 2:
        warningCount++;
        break;
      case 3:
        infoCount++;
        break;
      case 4:
        hintCount++;
        break;
    }
  }

  return { errorCount, warningCount, infoCount, hintCount };
}

/**
 * LSP store
 */
export const useLspStore = create<LspStore>((set, get) => ({
  runningServers: new Set(),
  diagnostics: new Map(),
  documentVersions: new Map(),
  enabled: true,
  showDiagnosticsPanel: false,

  addRunningServer: (language: string) => {
    set((state) => {
      const newServers = new Set(state.runningServers);
      newServers.add(language);
      return { runningServers: newServers };
    });
  },

  removeRunningServer: (language: string) => {
    set((state) => {
      const newServers = new Set(state.runningServers);
      newServers.delete(language);
      return { runningServers: newServers };
    });
  },

  setRunningServers: (servers: string[]) => {
    set({ runningServers: new Set(servers) });
  },

  isServerRunning: (language: string) => {
    return get().runningServers.has(language);
  },

  setFileDiagnostics: (filePath: string, diagnostics: LspDiagnostic[]) => {
    set((state) => {
      const counts = countBySeverity(diagnostics);
      const fileDiagnostics: FileDiagnostics = {
        filePath,
        diagnostics,
        ...counts,
        lastUpdated: Date.now(),
      };

      const newDiagnostics = new Map(state.diagnostics);
      newDiagnostics.set(filePath, fileDiagnostics);

      return { diagnostics: newDiagnostics };
    });
  },

  clearFileDiagnostics: (filePath: string) => {
    set((state) => {
      const newDiagnostics = new Map(state.diagnostics);
      newDiagnostics.delete(filePath);
      return { diagnostics: newDiagnostics };
    });
  },

  getFileDiagnostics: (filePath: string) => {
    return get().diagnostics.get(filePath);
  },

  getAllDiagnostics: () => {
    return Array.from(get().diagnostics.values());
  },

  getTotalErrorCount: () => {
    const allDiagnostics = Array.from(get().diagnostics.values());
    return allDiagnostics.reduce((sum, file) => sum + file.errorCount, 0);
  },

  getTotalWarningCount: () => {
    const allDiagnostics = Array.from(get().diagnostics.values());
    return allDiagnostics.reduce((sum, file) => sum + file.warningCount, 0);
  },

  getDocumentVersion: (filePath: string) => {
    return get().documentVersions.get(filePath) || 0;
  },

  incrementDocumentVersion: (filePath: string) => {
    const currentVersion = get().documentVersions.get(filePath) || 0;
    const newVersion = currentVersion + 1;

    set((state) => {
      const newVersions = new Map(state.documentVersions);
      newVersions.set(filePath, newVersion);
      return { documentVersions: newVersions };
    });

    return newVersion;
  },

  resetDocumentVersion: (filePath: string) => {
    set((state) => {
      const newVersions = new Map(state.documentVersions);
      newVersions.set(filePath, 0);
      return { documentVersions: newVersions };
    });
  },

  enableLsp: () => {
    set({ enabled: true });
  },

  disableLsp: () => {
    set({ enabled: false });
  },

  toggleDiagnosticsPanel: () => {
    set((state) => ({ showDiagnosticsPanel: !state.showDiagnosticsPanel }));
  },

  showDiagnostics: () => {
    set({ showDiagnosticsPanel: true });
  },

  hideDiagnostics: () => {
    set({ showDiagnosticsPanel: false });
  },
}));
