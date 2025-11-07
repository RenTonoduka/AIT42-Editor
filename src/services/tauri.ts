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
 * Plugin information
 */
export interface PluginInfo {
  manifest: {
    id: string;
    name: string;
    version: string;
    author: string;
    description: string;
    entry_point: string;
    dependencies: string[];
    permissions: string[];
  };
  state: 'Installed' | 'Enabled' | 'Disabled' | 'Error';
  install_path: string;
  error?: string;
}

/**
 * AIT42 Agent information
 */
export interface AgentInfo {
  name: string;
  description: string;
  category: string;
  tools: string[];
}

/**
 * Agent execution request
 */
export interface AgentExecutionRequest {
  agentName: string;
  task: string;
  context?: string;
}

/**
 * Agent execution response
 */
export interface AgentExecutionResponse {
  executionId: string;
  agentName: string;
  status: 'started' | 'running' | 'completed' | 'failed';
  output?: string;
  error?: string;
}

/**
 * Parallel execution request
 */
export interface ParallelExecutionRequest {
  agents: string[];
  task: string;
  context?: string;
}

/**
 * Tmux session information
 */
export interface TmuxSession {
  sessionId: string;
  agentName: string;
  status: 'running' | 'completed' | 'failed';
  createdAt: string;
}

/**
 * Tmux execution request
 */
export interface TmuxExecutionRequest {
  agentName: string;
  task: string;
  context?: string;
}

/**
 * Git Worktree information
 */
export interface WorktreeInfo {
  path: string;
  branch: string;
  commit: string;
  isBare: boolean;
  isDetached: boolean;
}

/**
 * Claude Code Competition Request
 */
export interface ClaudeCodeCompetitionRequest {
  task: string;
  instanceCount: number;  // 2-10
  model: string;  // "sonnet", "haiku", "opus"
  timeoutSeconds: number;  // default: 300
  preserveWorktrees: boolean;  // keep worktrees after completion
}

/**
 * Claude Code Analysis Request (Meta-Analysis)
 */
export interface ClaudeCodeAnalysisRequest {
  task: string;
  model: string;  // "sonnet", "haiku", "opus"
  timeoutSeconds: number;  // default: 120
}

/**
 * Claude Code Analysis Response
 */
export interface ClaudeCodeAnalysisResponse {
  analysisId: string;
  complexityClass: string;  // "Logarithmic", "Linear", "Quadratic", "Exponential"
  recommendedSubtasks: number;
  recommendedInstances: number;
  confidence: number;  // 0.0-1.0
  reasoning: string;
  rawOutput: string;
  status: string;  // "completed", "failed", "timeout"
}

/**
 * Claude Code Competition Result
 */
export interface ClaudeCodeCompetitionResult {
  competitionId: string;
  instances: ClaudeCodeInstanceResult[];
  startedAt: string;
  completedAt: string | null;
  status: string;  // "running", "completed", "failed"
}

/**
 * Individual Claude Code Instance Result
 */
export interface ClaudeCodeInstanceResult {
  instanceId: string;
  instanceNumber: number;
  worktreePath: string;
  tmuxSessionId: string;
  status: string;  // "starting", "running", "completed", "failed", "timeout"
  output: string;
  error: string | null;
  executionTimeMs: number;
  startedAt: string;
  completedAt: string | null;
}

/**
 * Role definition for debate participants
 */
export interface RoleDefinition {
  id: string;
  name: string;
  systemPrompt: string;
}

/**
 * Debate execution request
 */
export interface DebateRequest {
  task: string;
  roles: RoleDefinition[];  // 3 roles (Architect, Pragmatist, Innovator)
  model: string;  // "sonnet", "haiku", "opus"
  timeoutSeconds: number;  // Per-round timeout (default: 800s)
  preserveWorktrees: boolean;  // Keep worktrees after completion
}

/**
 * Debate execution result
 */
export interface DebateResult {
  debateId: string;
  status: string;  // "started", "round_1", "round_2", "round_3", "completed", "failed"
  message: string;
}

/**
 * Round output (result from one agent in one round)
 */
export interface RoundOutput {
  round: number;
  roleId: string;
  roleName: string;
  output: string;
  status: string;  // "running", "completed", "failed"
  startedAt: string;
  completedAt: string | null;
  executionTimeMs: number;
}

/**
 * Debate status (complete state)
 */
export interface DebateStatus {
  debateId: string;
  currentRound: number;  // 1, 2, or 3
  totalRounds: number;  // Always 3
  status: string;  // "started", "round_1", "round_2", "round_3", "completed", "failed"
  roundOutputs: RoundOutput[];
  worktreePath: string;
  contextFiles: string[];
  startedAt: string;
  completedAt: string | null;
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

  // ===== Plugin Commands =====

  /**
   * List all plugins
   */
  async listPlugins(): Promise<PluginInfo[]> {
    try {
      const plugins = await invoke<PluginInfo[]>('list_plugins');
      return plugins;
    } catch (error) {
      throw new Error(`Failed to list plugins: ${error}`);
    }
  },

  /**
   * Get plugin by ID
   */
  async getPlugin(pluginId: string): Promise<PluginInfo | null> {
    try {
      const plugin = await invoke<PluginInfo | null>('get_plugin', { pluginId });
      return plugin;
    } catch (error) {
      throw new Error(`Failed to get plugin: ${error}`);
    }
  },

  /**
   * Enable a plugin
   */
  async enablePlugin(pluginId: string): Promise<void> {
    try {
      await invoke('enable_plugin', { pluginId });
    } catch (error) {
      throw new Error(`Failed to enable plugin: ${error}`);
    }
  },

  /**
   * Disable a plugin
   */
  async disablePlugin(pluginId: string): Promise<void> {
    try {
      await invoke('disable_plugin', { pluginId });
    } catch (error) {
      throw new Error(`Failed to disable plugin: ${error}`);
    }
  },

  /**
   * Install a plugin from path
   */
  async installPlugin(sourcePath: string): Promise<string> {
    try {
      const pluginId = await invoke<string>('install_plugin', { sourcePath });
      return pluginId;
    } catch (error) {
      throw new Error(`Failed to install plugin: ${error}`);
    }
  },

  /**
   * Uninstall a plugin
   */
  async uninstallPlugin(pluginId: string): Promise<void> {
    try {
      await invoke('uninstall_plugin', { pluginId });
    } catch (error) {
      throw new Error(`Failed to uninstall plugin: ${error}`);
    }
  },

  // ============================================================================
  // AIT42 Agent Operations
  // ============================================================================

  /**
   * List all available AI agents
   */
  async listAgents(): Promise<AgentInfo[]> {
    try {
      const agents = await invoke<AgentInfo[]>('list_agents');
      return agents;
    } catch (error) {
      throw new Error(`Failed to list agents: ${error}`);
    }
  },

  /**
   * Get information about a specific agent
   */
  async getAgentInfo(agentName: string): Promise<AgentInfo> {
    try {
      const agent = await invoke<AgentInfo>('get_agent_info', { agentName });
      return agent;
    } catch (error) {
      throw new Error(`Failed to get agent info: ${error}`);
    }
  },

  /**
   * Execute a single AI agent
   */
  async executeAgent(request: AgentExecutionRequest): Promise<AgentExecutionResponse> {
    try {
      const response = await invoke<AgentExecutionResponse>('execute_agent', { request });
      return response;
    } catch (error) {
      throw new Error(`Failed to execute agent: ${error}`);
    }
  },

  /**
   * Execute multiple AI agents in parallel
   */
  async executeParallel(request: ParallelExecutionRequest): Promise<AgentExecutionResponse[]> {
    try {
      const responses = await invoke<AgentExecutionResponse[]>('execute_parallel', { request });
      return responses;
    } catch (error) {
      throw new Error(`Failed to execute agents in parallel: ${error}`);
    }
  },

  /**
   * Get output from a running or completed agent execution
   */
  async getAgentOutput(executionId: string): Promise<AgentExecutionResponse> {
    try {
      const response = await invoke<AgentExecutionResponse>('get_agent_output', { executionId });
      return response;
    } catch (error) {
      throw new Error(`Failed to get agent output: ${error}`);
    }
  },

  /**
   * Cancel a running agent execution
   */
  async cancelAgentExecution(executionId: string): Promise<void> {
    try {
      await invoke('cancel_agent_execution', { executionId });
    } catch (error) {
      throw new Error(`Failed to cancel agent execution: ${error}`);
    }
  },

  /**
   * Create a new tmux session for agent execution
   */
  async createTmuxSession(request: TmuxExecutionRequest): Promise<TmuxSession> {
    try {
      const session = await invoke<TmuxSession>('create_tmux_session', { request });
      return session;
    } catch (error) {
      throw new Error(`Failed to create tmux session: ${error}`);
    }
  },

  /**
   * List all active AIT42 tmux sessions
   */
  async listTmuxSessions(): Promise<TmuxSession[]> {
    try {
      const sessions = await invoke<TmuxSession[]>('list_tmux_sessions');
      return sessions;
    } catch (error) {
      throw new Error(`Failed to list tmux sessions: ${error}`);
    }
  },

  /**
   * Capture output from a tmux session
   */
  async captureTmuxOutput(sessionId: string): Promise<string> {
    try {
      const output = await invoke<string>('capture_tmux_output', { sessionId });
      return output;
    } catch (error) {
      throw new Error(`Failed to capture tmux output: ${error}`);
    }
  },

  /**
   * Send keys/command to a tmux session
   */
  async sendTmuxKeys(sessionId: string, keys: string): Promise<void> {
    try {
      await invoke('send_tmux_keys', { sessionId, keys });
    } catch (error) {
      throw new Error(`Failed to send tmux keys: ${error}`);
    }
  },

  /**
   * Kill a tmux session
   */
  async killTmuxSession(sessionId: string): Promise<void> {
    try {
      await invoke('kill_tmux_session', { sessionId });
    } catch (error) {
      throw new Error(`Failed to kill tmux session: ${error}`);
    }
  },

  /**
   * List all git worktrees
   */
  async listWorktrees(): Promise<WorktreeInfo[]> {
    try {
      const worktrees = await invoke<WorktreeInfo[]>('git_list_worktrees');
      return worktrees;
    } catch (error) {
      throw new Error(`Failed to list worktrees: ${error}`);
    }
  },

  /**
   * Create a new git worktree
   */
  async createWorktree(path: string, branch: string, createBranch: boolean = true): Promise<WorktreeInfo> {
    try {
      const worktree = await invoke<WorktreeInfo>('git_create_worktree', {
        path,
        branch,
        createBranch,
      });
      return worktree;
    } catch (error) {
      throw new Error(`Failed to create worktree: ${error}`);
    }
  },

  /**
   * Remove a git worktree
   */
  async removeWorktree(path: string, force: boolean = false): Promise<void> {
    try {
      await invoke('git_remove_worktree', { path, force });
    } catch (error) {
      throw new Error(`Failed to remove worktree: ${error}`);
    }
  },

  /**
   * Prune stale worktree data
   */
  async pruneWorktrees(): Promise<void> {
    try {
      await invoke('git_prune_worktrees');
    } catch (error) {
      throw new Error(`Failed to prune worktrees: ${error}`);
    }
  },

  // ===== Claude Code Competition Commands =====

  /**
   * Execute Claude Code Competition
   *
   * Creates multiple git worktrees and launches Claude Code instances in parallel
   */
  async executeClaudeCodeCompetition(
    request: ClaudeCodeCompetitionRequest
  ): Promise<ClaudeCodeCompetitionResult> {
    try {
      const result = await invoke<ClaudeCodeCompetitionResult>(
        'execute_claude_code_competition',
        { request }
      );
      return result;
    } catch (error) {
      throw new Error(`Failed to execute Claude Code competition: ${error}`);
    }
  },

  /**
   * Analyze task using Claude Code itself (meta-analysis)
   *
   * Launches Claude Code CLI to analyze the task and provide
   * complexity classification and decomposition recommendations.
   *
   * @param request - Analysis request parameters
   * @returns Analysis response with complexity class, subtask count, and reasoning
   */
  async analyzeTaskWithClaudeCode(
    request: ClaudeCodeAnalysisRequest
  ): Promise<ClaudeCodeAnalysisResponse> {
    try {
      const result = await invoke<ClaudeCodeAnalysisResponse>(
        'analyze_task_with_claude_code',
        { request }
      );
      return result;
    } catch (error) {
      throw new Error(`Failed to analyze task with Claude Code: ${error}`);
    }
  },

  /**
   * Get competition status and results
   */
  async getCompetitionStatus(
    competitionId: string
  ): Promise<ClaudeCodeCompetitionResult> {
    try {
      const result = await invoke<ClaudeCodeCompetitionResult>(
        'get_competition_status',
        { competitionId }
      );
      return result;
    } catch (error) {
      throw new Error(`Failed to get competition status: ${error}`);
    }
  },

  /**
   * Cancel a running competition
   */
  async cancelCompetition(
    competitionId: string,
    cleanupWorktrees: boolean = true
  ): Promise<void> {
    try {
      await invoke('cancel_competition', { competitionId, cleanupWorktrees });
    } catch (error) {
      throw new Error(`Failed to cancel competition: ${error}`);
    }
  },

  // ===== Claude Code Debate Commands =====

  /**
   * Execute Claude Code Debate
   *
   * Creates a single git worktree and executes 3 rounds of debate sequentially
   */
  async executeDebate(
    request: DebateRequest
  ): Promise<DebateResult> {
    try {
      const result = await invoke<DebateResult>(
        'execute_debate',
        { request }
      );
      return result;
    } catch (error) {
      throw new Error(`Failed to execute debate: ${error}`);
    }
  },

  /**
   * Get debate status
   */
  async getDebateStatus(
    debateId: string
  ): Promise<DebateStatus> {
    try {
      const result = await invoke<DebateStatus>(
        'get_debate_status',
        { debateId }
      );
      return result;
    } catch (error) {
      throw new Error(`Failed to get debate status: ${error}`);
    }
  },

  /**
   * Cancel a running debate
   */
  async cancelDebate(
    debateId: string,
    cleanupWorktrees: boolean = true
  ): Promise<void> {
    try {
      await invoke('cancel_debate', { debateId, cleanupWorktrees });
    } catch (error) {
      throw new Error(`Failed to cancel debate: ${error}`);
    }
  },

  // ===== Session History Commands (v1.6.0) =====

  /**
   * Create a new worktree session
   */
  async createSession(session: import('@/types/worktree').WorktreeSession): Promise<import('@/types/worktree').WorktreeSession> {
    try {
      const result = await invoke<import('@/types/worktree').WorktreeSession>('create_session', { session });
      return result;
    } catch (error) {
      throw new Error(`Failed to create session: ${error}`);
    }
  },

  /**
   * Update an existing session
   */
  async updateSession(session: import('@/types/worktree').WorktreeSession): Promise<import('@/types/worktree').WorktreeSession> {
    try {
      const result = await invoke<import('@/types/worktree').WorktreeSession>('update_session', { session });
      return result;
    } catch (error) {
      throw new Error(`Failed to update session: ${error}`);
    }
  },

  /**
   * Get a specific session by ID
   */
  async getSession(sessionId: string): Promise<import('@/types/worktree').WorktreeSession> {
    try {
      const result = await invoke<import('@/types/worktree').WorktreeSession>('get_session', { sessionId });
      return result;
    } catch (error) {
      throw new Error(`Failed to get session: ${error}`);
    }
  },

  /**
   * Get all sessions
   */
  async getAllSessions(): Promise<import('@/types/worktree').WorktreeSession[]> {
    try {
      const result = await invoke<import('@/types/worktree').WorktreeSession[]>('get_all_sessions');
      return result;
    } catch (error) {
      throw new Error(`Failed to get all sessions: ${error}`);
    }
  },

  /**
   * Delete a session
   */
  async deleteSession(sessionId: string): Promise<void> {
    try {
      await invoke('delete_session', { sessionId });
    } catch (error) {
      throw new Error(`Failed to delete session: ${error}`);
    }
  },

  /**
   * Add a chat message to a session
   */
  async addChatMessage(
    sessionId: string,
    message: import('@/types/worktree').ChatMessage
  ): Promise<import('@/types/worktree').WorktreeSession> {
    try {
      const result = await invoke<import('@/types/worktree').WorktreeSession>('add_chat_message', {
        sessionId,
        message,
      });
      return result;
    } catch (error) {
      throw new Error(`Failed to add chat message: ${error}`);
    }
  },

  /**
   * Update instance status within a session
   */
  async updateInstanceStatus(
    sessionId: string,
    instanceId: number,
    newStatus: string
  ): Promise<import('@/types/worktree').WorktreeSession> {
    try {
      const result = await invoke<import('@/types/worktree').WorktreeSession>('update_instance_status', {
        sessionId,
        instanceId,
        newStatus,
      });
      return result;
    } catch (error) {
      throw new Error(`Failed to update instance status: ${error}`);
    }
  },
};
