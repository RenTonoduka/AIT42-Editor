/**
 * Worktree Session Management Types
 * Inspired by Vibe Kanban's task-centric approach
 */

export type SessionType = 'competition' | 'ensemble' | 'debate';
export type SessionStatus = 'running' | 'completed' | 'failed' | 'paused';
export type InstanceStatus = 'idle' | 'running' | 'completed' | 'failed' | 'paused';
export type AgentRuntime = 'claude' | 'codex' | 'gemini';

export interface RuntimeAllocation {
  runtime: AgentRuntime;
  count: number;
  model?: string;
}

/**
 * Individual worktree instance within a session
 */
export interface WorktreeInstance {
  instanceId: number;
  worktreePath: string;
  branch: string;
  agentName: string;
  status: InstanceStatus;
  tmuxSessionId: string;
  output?: string;
  startTime?: string;
  endTime?: string;
  filesChanged?: number;
  linesAdded?: number;
  linesDeleted?: number;
  runtime?: AgentRuntime;
  model?: string;
  runtimeLabel?: string;
}

/**
 * Chat message for interactive communication with Claude Code
 */
export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  instanceId?: number; // Which instance this message is for
}

/**
 * Complete worktree session (Competition/Ensemble/Debate)
 * Persisted to disk for historical access
 */
export interface WorktreeSession {
  id: string; // competition/ensemble/debate ID
  type: SessionType;
  task: string;
  status: SessionStatus;
  createdAt: string;
  updatedAt: string;
  completedAt?: string;

  // Instances
  instances: WorktreeInstance[];

  // Interactive chat history
  chatHistory: ChatMessage[];

  // Metadata
  model?: string; // sonnet/haiku/opus
  timeoutSeconds?: number;
  preserveWorktrees?: boolean;
  runtimeMix?: AgentRuntime[]; // Order of runtimes launched

  // Results (for Competition mode)
  winnerId?: number;

  // Statistics
  totalDuration?: number; // seconds
  totalFilesChanged?: number;
  totalLinesAdded?: number;
  totalLinesDeleted?: number;
}

/**
 * Kanban column definition
 */
export interface KanbanColumn {
  id: string;
  title: string;
  status: SessionStatus;
  sessions: WorktreeSession[];
}

/**
 * Filter and sort options for session list
 */
export interface SessionFilters {
  type?: SessionType[];
  status?: SessionStatus[];
  dateFrom?: string;
  dateTo?: string;
  searchQuery?: string;
}

export interface SessionSortOptions {
  field: 'createdAt' | 'updatedAt' | 'duration' | 'filesChanged';
  direction: 'asc' | 'desc';
}
