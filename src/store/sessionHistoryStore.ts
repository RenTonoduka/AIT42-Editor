/**
 * Session History Store
 *
 * Manages worktree session history: Competition, Ensemble, and Debate sessions
 * Inspired by Vibe Kanban's task-centric persistence model
 */
import { create } from 'zustand';
import { tauriApi } from '@/services/tauri';
import type {
  WorktreeSession,
  ChatMessage,
  SessionFilters,
  SessionSortOptions,
  SessionStatus,
} from '@/types/worktree';

/**
 * Session History store state and actions
 */
interface SessionHistoryStore {
  /** All sessions loaded from disk */
  sessions: WorktreeSession[];
  /** Currently selected/active session ID */
  activeSessionId: string | null;
  /** Loading state */
  isLoading: boolean;
  /** Error message */
  error: string | null;
  /** Filters for session list */
  filters: SessionFilters;
  /** Sort options */
  sortOptions: SessionSortOptions;

  // Actions
  /** Load all sessions from disk */
  loadSessions: () => Promise<void>;
  /** Create a new session */
  createSession: (session: WorktreeSession) => Promise<void>;
  /** Update an existing session */
  updateSession: (session: WorktreeSession) => Promise<void>;
  /** Get a specific session by ID */
  getSession: (sessionId: string) => Promise<WorktreeSession | null>;
  /** Delete a session */
  deleteSession: (sessionId: string) => Promise<void>;
  /** Set active session */
  setActiveSession: (sessionId: string | null) => void;
  /** Add chat message to active session */
  addChatMessage: (sessionId: string, message: ChatMessage) => Promise<void>;
  /** Update instance status */
  updateInstanceStatus: (
    sessionId: string,
    instanceId: number,
    newStatus: string
  ) => Promise<void>;
  /** Set filters */
  setFilters: (filters: Partial<SessionFilters>) => void;
  /** Set sort options */
  setSortOptions: (sortOptions: SessionSortOptions) => void;
  /** Get filtered and sorted sessions */
  getFilteredSessions: () => WorktreeSession[];
  /** Reset store to initial state */
  reset: () => void;
}

/**
 * Initial state
 */
const initialState = {
  sessions: [],
  activeSessionId: null,
  isLoading: false,
  error: null,
  filters: {},
  sortOptions: {
    field: 'updatedAt' as const,
    direction: 'desc' as const,
  },
};

/**
 * Session History store
 */
export const useSessionHistoryStore = create<SessionHistoryStore>((set, get) => ({
  ...initialState,

  loadSessions: async () => {
    set({ isLoading: true, error: null });

    try {
      const sessions = await tauriApi.getAllSessions();
      set({ sessions, isLoading: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to load sessions',
        isLoading: false,
      });
    }
  },

  createSession: async (session: WorktreeSession) => {
    set({ isLoading: true, error: null });

    try {
      const created = await tauriApi.createSession(session);

      // Add to local state
      const { sessions } = get();
      set({
        sessions: [...sessions, created],
        activeSessionId: created.id,
        isLoading: false,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to create session',
        isLoading: false,
      });
      throw error;
    }
  },

  updateSession: async (session: WorktreeSession) => {
    set({ isLoading: true, error: null });

    try {
      const updated = await tauriApi.updateSession(session);

      // Update local state
      const { sessions } = get();
      const newSessions = sessions.map((s) => (s.id === updated.id ? updated : s));

      set({
        sessions: newSessions,
        isLoading: false,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to update session',
        isLoading: false,
      });
      throw error;
    }
  },

  getSession: async (sessionId: string) => {
    set({ isLoading: true, error: null });

    try {
      const session = await tauriApi.getSession(sessionId);
      set({ isLoading: false });
      return session;
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to get session',
        isLoading: false,
      });
      return null;
    }
  },

  deleteSession: async (sessionId: string) => {
    set({ isLoading: true, error: null });

    try {
      await tauriApi.deleteSession(sessionId);

      // Remove from local state
      const { sessions, activeSessionId } = get();
      const newSessions = sessions.filter((s) => s.id !== sessionId);

      set({
        sessions: newSessions,
        activeSessionId: activeSessionId === sessionId ? null : activeSessionId,
        isLoading: false,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to delete session',
        isLoading: false,
      });
      throw error;
    }
  },

  setActiveSession: (sessionId: string | null) => {
    set({ activeSessionId: sessionId });
  },

  addChatMessage: async (sessionId: string, message: ChatMessage) => {
    try {
      const updated = await tauriApi.addChatMessage(sessionId, message);

      // Update local state
      const { sessions } = get();
      const newSessions = sessions.map((s) => (s.id === updated.id ? updated : s));

      set({ sessions: newSessions });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to add chat message',
      });
      throw error;
    }
  },

  updateInstanceStatus: async (
    sessionId: string,
    instanceId: number,
    newStatus: string
  ) => {
    try {
      const updated = await tauriApi.updateInstanceStatus(sessionId, instanceId, newStatus);

      // Update local state
      const { sessions } = get();
      const newSessions = sessions.map((s) => (s.id === updated.id ? updated : s));

      set({ sessions: newSessions });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to update instance status',
      });
      throw error;
    }
  },

  setFilters: (filters: Partial<SessionFilters>) => {
    const currentFilters = get().filters;
    set({ filters: { ...currentFilters, ...filters } });
  },

  setSortOptions: (sortOptions: SessionSortOptions) => {
    set({ sortOptions });
  },

  getFilteredSessions: () => {
    const { sessions, filters, sortOptions } = get();
    let filtered = [...sessions];

    // Apply type filter
    if (filters.type && filters.type.length > 0) {
      filtered = filtered.filter((s) => filters.type!.includes(s.type));
    }

    // Apply status filter
    if (filters.status && filters.status.length > 0) {
      filtered = filtered.filter((s) => filters.status!.includes(s.status));
    }

    // Apply date filters
    if (filters.dateFrom) {
      filtered = filtered.filter((s) => s.createdAt >= filters.dateFrom!);
    }
    if (filters.dateTo) {
      filtered = filtered.filter((s) => s.createdAt <= filters.dateTo!);
    }

    // Apply search query
    if (filters.searchQuery && filters.searchQuery.trim()) {
      const query = filters.searchQuery.toLowerCase();
      filtered = filtered.filter(
        (s) =>
          s.task.toLowerCase().includes(query) ||
          s.id.toLowerCase().includes(query) ||
          s.instances.some((i) => i.agentName.toLowerCase().includes(query))
      );
    }

    // Apply sorting
    filtered.sort((a, b) => {
      let aValue: string | number = 0;
      let bValue: string | number = 0;

      switch (sortOptions.field) {
        case 'createdAt':
          aValue = a.createdAt;
          bValue = b.createdAt;
          break;
        case 'updatedAt':
          aValue = a.updatedAt;
          bValue = b.updatedAt;
          break;
        case 'duration':
          aValue = a.totalDuration || 0;
          bValue = b.totalDuration || 0;
          break;
        case 'filesChanged':
          aValue = a.totalFilesChanged || 0;
          bValue = b.totalFilesChanged || 0;
          break;
      }

      if (sortOptions.direction === 'asc') {
        return aValue < bValue ? -1 : aValue > bValue ? 1 : 0;
      } else {
        return aValue > bValue ? -1 : aValue < bValue ? 1 : 0;
      }
    });

    return filtered;
  },

  reset: () => {
    set(initialState);
  },
}));
