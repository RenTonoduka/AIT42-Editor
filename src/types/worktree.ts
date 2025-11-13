/**
 * Worktree Session Management Types
 * Inspired by Vibe Kanban's task-centric approach
 */

export type SessionType = 'competition' | 'ensemble' | 'debate';
export type SessionStatus = 'running' | 'completed' | 'failed' | 'paused';
export type InstanceStatus = 'idle' | 'running' | 'completed' | 'failed' | 'paused' | 'archived';
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

  // Competition evaluation fields
  testsPassed?: number;
  testsFailed?: number;
  codeComplexity?: number;    // 0-100 (lower is better)
  executionTime?: number;      // seconds
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
 * Evaluation metrics for Competition mode
 */
export interface EvaluationMetrics {
  filesChanged: number;
  linesAdded: number;
  linesDeleted: number;
  executionTime: number;      // seconds
  testsPassed?: number;        // Optional: from test results
  testsFailed?: number;        // Optional: from test results
  codeComplexity?: number;     // Optional: 0-100 (lower is better)
  successRate: number;         // 0-100 (percentage)
}

/**
 * Evaluation score for each instance in Competition mode
 */
export interface EvaluationScore {
  instanceId: number;
  agentName: string;
  runtime: string;
  totalScore: number;          // 0-100
  metrics: EvaluationMetrics;
  rank: number;                // 1 = best, 2 = second, etc.
  isRecommended: boolean;      // Top 3 recommended
  scoreBreakdown: {
    testScore: number;         // 0-40 points
    complexityScore: number;   // 0-30 points
    efficiencyScore: number;   // 0-20 points
    changeScore: number;       // 0-10 points
  };
}

/**
 * Competition evaluation result
 */
export interface CompetitionEvaluation {
  competitionId: string;
  evaluatedAt: string;
  scores: EvaluationScore[];
  recommendedWinnerId: number; // Top ranked instance
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
  evaluation?: CompetitionEvaluation;   // Competition evaluation result

  // Statistics
  totalDuration?: number; // seconds
  totalFilesChanged?: number;
  totalLinesAdded?: number;
  totalLinesDeleted?: number;

  // Ensemble mode specific: 統合フェーズの状態
  integrationPhase?: 'pending' | 'in_progress' | 'completed';
  // 統合AIのinstance ID
  integrationInstanceId?: number;
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
