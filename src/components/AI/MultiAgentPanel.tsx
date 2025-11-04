/**
 * MultiAgentPanel - Parallel Multi-Claude Code Development
 *
 * Manages multiple Claude Code instances working in parallel
 * on different aspects of a project (Frontend, Backend, Testing, etc.)
 */

import React, { useState, useEffect, useRef } from 'react';
import { Users, Play, Pause, Square, Trash2, Plus, Code2, CheckCircle, XCircle, Clock, Terminal, GitBranch } from 'lucide-react';
import { tauriApi, AgentExecutionResponse, TmuxSession, WorktreeInfo } from '@/services/tauri';

export interface ClaudeCodeInstance {
  id: string;
  name: string;
  role: string; // "Frontend", "Backend", "Testing", etc.
  task: string;
  status: 'idle' | 'running' | 'completed' | 'failed' | 'paused';
  output: string[];
  startTime?: number;
  endTime?: number;
  executionId?: string;
  tmuxSessionId?: string; // For tmux-based execution
  worktreePath?: string; // For git worktree-based execution
  worktreeBranch?: string; // Branch name for the worktree
  metrics?: {
    linesOfCode?: number;
    filesModified?: number;
    testsAdded?: number;
    coveragePercent?: number;
  };
}

export interface MultiAgentPanelProps {
  /** Whether the panel is visible */
  isVisible: boolean;
  /** Callback when panel should close */
  onClose: () => void;
}

/**
 * MultiAgentPanel component
 */
export const MultiAgentPanel: React.FC<MultiAgentPanelProps> = ({
  isVisible,
  onClose,
}) => {
  const [instances, setInstances] = useState<ClaudeCodeInstance[]>([
    {
      id: '1',
      name: 'Claude Frontend',
      role: 'Frontend Developer',
      task: '',
      status: 'idle',
      output: [],
    },
    {
      id: '2',
      name: 'Claude Backend',
      role: 'Backend Developer',
      task: '',
      status: 'idle',
      output: [],
    },
    {
      id: '3',
      name: 'Claude Tester',
      role: 'Test Engineer',
      task: '',
      status: 'idle',
      output: [],
    },
  ]);

  const [globalTask, setGlobalTask] = useState('');
  const [showComparison, setShowComparison] = useState(false);
  const [desiredInstanceCount, setDesiredInstanceCount] = useState(3);
  const [useTmuxMode, setUseTmuxMode] = useState(true); // Use tmux by default for AIT42 integration

  // Track active polling intervals for cleanup
  const pollingIntervalsRef = useRef<Map<string, NodeJS.Timeout>>(new Map());

  // Cleanup polling intervals on unmount
  useEffect(() => {
    return () => {
      // Clear all active polling intervals when component unmounts
      pollingIntervalsRef.current.forEach((interval) => clearInterval(interval));
      pollingIntervalsRef.current.clear();
    };
  }, []);

  // Decompose global task into subtasks for each role
  const decomposeTask = (globalTaskDescription: string, role: string): string => {
    if (!globalTaskDescription.trim()) return '';

    const taskTemplates: Record<string, (task: string) => string> = {
      'Frontend Developer': (task) =>
        `Build the frontend UI for: ${task}\n- Create React components with TypeScript\n- Implement responsive design\n- Add proper state management\n- Ensure accessibility (WCAG)`,
      'Backend Developer': (task) =>
        `Implement backend services for: ${task}\n- Design and implement REST/GraphQL APIs\n- Set up database schemas and queries\n- Implement business logic and validation\n- Add error handling and logging`,
      'Test Engineer': (task) =>
        `Create comprehensive tests for: ${task}\n- Write unit tests with high coverage\n- Implement integration tests\n- Add E2E test scenarios\n- Set up CI/CD test automation`,
      'DevOps Engineer': (task) =>
        `Set up DevOps infrastructure for: ${task}\n- Configure CI/CD pipelines\n- Set up deployment automation\n- Implement monitoring and logging\n- Configure security scanning`,
      'Security Specialist': (task) =>
        `Perform security analysis for: ${task}\n- Conduct threat modeling\n- Implement OWASP best practices\n- Add authentication/authorization\n- Perform vulnerability scanning`,
      'Database Designer': (task) =>
        `Design database architecture for: ${task}\n- Design normalized schema\n- Optimize queries and indexes\n- Plan migration strategy\n- Implement data validation`,
    };

    const template = taskTemplates[role];
    return template ? template(globalTaskDescription) : globalTaskDescription;
  };

  // Auto-distribute tasks when global task changes
  const handleDistributeTasks = () => {
    if (!globalTask.trim()) return;

    setInstances((prev) =>
      prev.map((inst) => ({
        ...inst,
        task: decomposeTask(globalTask, inst.role),
      }))
    );
  };

  // Add new Claude Code instance
  const addInstance = () => {
    const newInstance: ClaudeCodeInstance = {
      id: Date.now().toString(),
      name: `Claude ${instances.length + 1}`,
      role: 'Developer',
      task: '',
      status: 'idle',
      output: [],
    };
    setInstances([...instances, newInstance]);
  };

  // Quick setup: Create specific number of instances
  const quickSetup = (count: number) => {
    const roles = [
      'Frontend Developer',
      'Backend Developer',
      'Test Engineer',
      'DevOps Engineer',
      'Security Specialist',
      'Database Designer',
    ];

    const newInstances: ClaudeCodeInstance[] = [];
    for (let i = 0; i < count; i++) {
      const role = roles[i % roles.length];
      newInstances.push({
        id: `${Date.now()}-${i}`,
        name: `Claude ${i + 1}`,
        role,
        task: '',
        status: 'idle',
        output: [],
      });
    }
    setInstances(newInstances);
    setDesiredInstanceCount(count);
  };

  // Remove instance
  const removeInstance = (id: string) => {
    setInstances(instances.filter((inst) => inst.id !== id));
  };

  // Update instance task
  const updateInstanceTask = (id: string, task: string) => {
    setInstances(
      instances.map((inst) => (inst.id === id ? { ...inst, task } : inst))
    );
  };

  // Update instance role
  const updateInstanceRole = (id: string, role: string) => {
    setInstances(
      instances.map((inst) => (inst.id === id ? { ...inst, role } : inst))
    );
  };

  // Poll for agent output and status
  const pollAgentStatus = async (id: string, executionId: string) => {
    const maxPolls = 120; // 2 minutes max (120 * 1s)
    let pollCount = 0;

    const cleanup = (intervalId: NodeJS.Timeout) => {
      clearInterval(intervalId);
      pollingIntervalsRef.current.delete(id);
    };

    const pollInterval = setInterval(async () => {
      try {
        const output = await tauriApi.getAgentOutput(executionId);

        setInstances((prev) =>
          prev.map((inst) => {
            if (inst.id === id) {
              const newOutput = output.output ? [...inst.output, ...output.output.split('\n').filter(Boolean)] : inst.output;

              // Check if execution is complete
              if (output.status === 'completed') {
                cleanup(pollInterval);
                return {
                  ...inst,
                  status: 'completed',
                  endTime: Date.now(),
                  output: [...newOutput, '✅ Task completed successfully!'],
                };
              } else if (output.status === 'failed') {
                cleanup(pollInterval);
                return {
                  ...inst,
                  status: 'failed',
                  endTime: Date.now(),
                  output: [...newOutput, `❌ Task failed: ${output.error || 'Unknown error'}`],
                };
              }

              return { ...inst, output: newOutput };
            }
            return inst;
          })
        );

        pollCount++;
        if (pollCount >= maxPolls) {
          cleanup(pollInterval);
          setInstances((prev) =>
            prev.map((inst) =>
              inst.id === id
                ? {
                    ...inst,
                    status: 'failed',
                    endTime: Date.now(),
                    output: [...inst.output, '⏱️ Execution timeout'],
                  }
                : inst
            )
          );
        }
      } catch (error) {
        console.error('Failed to poll agent status:', error);
      }
    }, 1000); // Poll every second

    // Register interval for cleanup
    pollingIntervalsRef.current.set(id, pollInterval);
  };

  // Poll for tmux session output
  const pollTmuxStatus = async (id: string, sessionId: string) => {
    const maxPolls = 120; // 2 minutes max (120 * 1s)
    let pollCount = 0;
    let lastOutputLength = 0;

    const cleanup = (intervalId: NodeJS.Timeout) => {
      clearInterval(intervalId);
      pollingIntervalsRef.current.delete(id);
    };

    const pollInterval = setInterval(async () => {
      try {
        const output = await tauriApi.captureTmuxOutput(sessionId);
        const outputLines = output.split('\n').filter(Boolean);

        // Only update if there's new output
        if (outputLines.length > lastOutputLength) {
          const newLines = outputLines.slice(lastOutputLength);
          lastOutputLength = outputLines.length;

          setInstances((prev) =>
            prev.map((inst) => {
              if (inst.id === id) {
                return {
                  ...inst,
                  output: [...inst.output, ...newLines],
                };
              }
              return inst;
            })
          );
        }

        // Check if session is still running
        const sessions = await tauriApi.listTmuxSessions();
        const isRunning = sessions.some((s) => s.sessionId === sessionId);

        if (!isRunning) {
          cleanup(pollInterval);
          setInstances((prev) =>
            prev.map((inst) =>
              inst.id === id
                ? {
                    ...inst,
                    status: 'completed',
                    endTime: Date.now(),
                    output: [...inst.output, '✅ Tmux session completed'],
                  }
                : inst
            )
          );
        }

        pollCount++;
        if (pollCount >= maxPolls) {
          cleanup(pollInterval);
          setInstances((prev) =>
            prev.map((inst) =>
              inst.id === id
                ? {
                    ...inst,
                    status: 'completed',
                    endTime: Date.now(),
                    output: [...inst.output, '⏱️ Polling timeout - session may still be running'],
                  }
                : inst
            )
          );
        }
      } catch (error) {
        console.error('Failed to poll tmux status:', error);
      }
    }, 1000); // Poll every second

    // Register interval for cleanup
    pollingIntervalsRef.current.set(id, pollInterval);
  };

  // Start single instance
  const startInstance = async (id: string) => {
    const instance = instances.find((inst) => inst.id === id);
    if (!instance || !instance.task) return;

    // Update status to running
    setInstances(
      instances.map((inst) =>
        inst.id === id
          ? {
              ...inst,
              status: 'running',
              startTime: Date.now(),
              output: [useTmuxMode ? `🚀 Starting ${inst.role} in Tmux...` : `🚀 Starting ${inst.role}...`]
            }
          : inst
      )
    );

    try {
      // Execute agent based on role
      const agentMap: Record<string, string> = {
        'Frontend Developer': 'frontend-developer',
        'Backend Developer': 'backend-developer',
        'Test Engineer': 'test-generator',
        'DevOps Engineer': 'devops-engineer',
        'Security Specialist': 'security-scanner',
        'Database Designer': 'database-designer',
      };

      const agentName = agentMap[instance.role] || 'code-reviewer';

      if (useTmuxMode) {
        const timestamp = Date.now();
        const worktreeBasename = `ait42-${instance.role.toLowerCase().replace(/\s+/g, '-')}-${timestamp}`;
        const worktreePath = `../ait42-worktrees/${worktreeBasename}`;
        const worktreeBranch = `ait42/${instance.role.toLowerCase().replace(/\s+/g, '-')}/${timestamp}`;

        try {
          // 1. Create dedicated git worktree for this agent
          const worktree = await tauriApi.createWorktree(worktreePath, worktreeBranch, true);

          // Update instance with worktree info
          setInstances(
            instances.map((inst) =>
              inst.id === id
                ? {
                    ...inst,
                    worktreePath: worktree.path,
                    worktreeBranch: worktree.branch,
                    output: [
                      ...inst.output,
                      `📁 Worktree created: ${worktree.path}`,
                      `🌿 Branch: ${worktree.branch}`,
                      `📌 Commit: ${worktree.commit.substring(0, 7)}`,
                    ],
                  }
                : inst
            )
          );

          // 2. Create tmux session for agent execution
          const session = await tauriApi.createTmuxSession({
            agentName,
            task: instance.task,
            context: globalTask,
          });

          // Update instance with tmux session ID
          setInstances(
            instances.map((inst) =>
              inst.id === id
                ? {
                    ...inst,
                    tmuxSessionId: session.sessionId,
                    output: [
                      ...inst.output,
                      `🎬 Tmux session created: ${session.sessionId}`,
                      `🤖 Agent "${agentName}" running in isolated environment`,
                      `🔧 Working directory: ${worktree.path}`,
                      `📊 Use 'tmux attach -t ${session.sessionId}' to view live output`,
                    ],
                  }
                : inst
            )
          );

          // Start polling for tmux output
          await pollTmuxStatus(id, session.sessionId);
        } catch (worktreeError) {
          // Handle worktree creation error
          setInstances(
            instances.map((inst) =>
              inst.id === id
                ? {
                    ...inst,
                    status: 'failed',
                    endTime: Date.now(),
                    output: [...inst.output, `❌ Failed to create worktree: ${worktreeError}`],
                  }
                : inst
            )
          );
          return;
        }
      } else {
        // Regular execution mode
        const response = await tauriApi.executeAgent({
          agentName,
          task: instance.task,
          context: globalTask,
        });

        // Update instance with execution ID
        setInstances(
          instances.map((inst) =>
            inst.id === id
              ? {
                  ...inst,
                  executionId: response.executionId,
                  output: [...inst.output, `🤖 Agent "${agentName}" started`, `📋 Execution ID: ${response.executionId}`],
                }
              : inst
          )
        );

        // Start polling for status and output
        await pollAgentStatus(id, response.executionId);
      }
    } catch (error) {
      setInstances(
        instances.map((inst) =>
          inst.id === id
            ? {
                ...inst,
                status: 'failed',
                endTime: Date.now(),
                output: [...inst.output, `❌ Error: ${error}`],
              }
            : inst
        )
      );
    }
  };

  // Start all instances in parallel
  const startAll = async () => {
    const runnableInstances = instances.filter(
      (inst) => inst.task && inst.status === 'idle'
    );

    for (const instance of runnableInstances) {
      await startInstance(instance.id);
    }
  };

  // Stop instance
  const stopInstance = async (id: string) => {
    const instance = instances.find((inst) => inst.id === id);

    // Clear polling interval if exists
    const interval = pollingIntervalsRef.current.get(id);
    if (interval) {
      clearInterval(interval);
      pollingIntervalsRef.current.delete(id);
    }

    // Kill tmux session if exists
    if (instance?.tmuxSessionId) {
      try {
        await tauriApi.killTmuxSession(instance.tmuxSessionId);
      } catch (error) {
        console.error('Failed to kill tmux session:', error);
      }
    }

    // Remove worktree if exists
    if (instance?.worktreePath) {
      try {
        await tauriApi.removeWorktree(instance.worktreePath, true);
        console.log(`Removed worktree: ${instance.worktreePath}`);
      } catch (error) {
        console.error('Failed to remove worktree:', error);
      }
    }

    setInstances(
      instances.map((inst) =>
        inst.id === id ? { ...inst, status: 'paused', endTime: Date.now() } : inst
      )
    );
  };

  // Get status icon
  const getStatusIcon = (status: ClaudeCodeInstance['status']) => {
    switch (status) {
      case 'idle':
        return <Clock size={16} className="text-text-tertiary" />;
      case 'running':
        return <Play size={16} className="text-blue-400 animate-pulse" />;
      case 'completed':
        return <CheckCircle size={16} className="text-green-400" />;
      case 'failed':
        return <XCircle size={16} className="text-red-400" />;
      case 'paused':
        return <Pause size={16} className="text-yellow-400" />;
    }
  };

  // Get status color
  const getStatusColor = (status: ClaudeCodeInstance['status']) => {
    switch (status) {
      case 'idle':
        return 'text-text-tertiary';
      case 'running':
        return 'text-blue-400';
      case 'completed':
        return 'text-green-400';
      case 'failed':
        return 'text-red-400';
      case 'paused':
        return 'text-yellow-400';
    }
  };

  // Calculate duration
  const getDuration = (instance: ClaudeCodeInstance): string => {
    if (!instance.startTime) return '-';
    const endTime = instance.endTime || Date.now();
    const duration = Math.floor((endTime - instance.startTime) / 1000);
    return `${duration}s`;
  };

  if (!isVisible) return null;

  return (
    <div className="fixed inset-y-0 right-0 w-[600px] bg-editor-elevated border-l border-editor-border shadow-2xl z-40 flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-editor-border bg-editor-surface">
        <div className="flex items-center gap-2">
          <Users size={20} className="text-accent-primary" />
          <h2 className="text-sm font-semibold text-text-primary">
            Multi-Agent Parallel Development
          </h2>
          <div className="ml-2 px-2 py-0.5 bg-accent-primary/20 text-accent-primary text-xs rounded-full font-medium">
            {instances.length} instances
          </div>
        </div>
        <button
          onClick={onClose}
          className="p-1 hover:bg-editor-border/30 rounded transition-colors"
        >
          <Square size={16} className="text-text-tertiary" />
        </button>
      </div>

      {/* Instance Count Control */}
      <div className="px-4 py-3 border-b border-editor-border bg-editor-surface">
        <div className="flex items-center gap-3">
          <div className="flex-1">
            <label className="block text-xs font-medium text-text-secondary mb-1">
              Instance Count
            </label>
            <div className="flex items-center gap-2">
              <input
                type="number"
                min="1"
                max="20"
                value={desiredInstanceCount}
                onChange={(e) => setDesiredInstanceCount(parseInt(e.target.value, 10) || 1)}
                className="w-20 px-2 py-1 text-sm bg-editor-bg text-text-primary border border-editor-border rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/50"
              />
              <button
                onClick={() => quickSetup(desiredInstanceCount)}
                className="px-3 py-1 text-xs bg-accent-primary hover:bg-accent-secondary text-white rounded transition-colors"
              >
                Apply
              </button>
            </div>
          </div>
          <div className="flex-1">
            <label className="block text-xs font-medium text-text-secondary mb-1">
              Quick Presets
            </label>
            <div className="flex gap-2">
              <button
                onClick={() => quickSetup(3)}
                className="px-3 py-1 text-xs bg-editor-bg hover:bg-editor-border text-text-primary border border-editor-border rounded transition-colors"
                title="3 instances (Frontend, Backend, Tester)"
              >
                Small (3)
              </button>
              <button
                onClick={() => quickSetup(6)}
                className="px-3 py-1 text-xs bg-editor-bg hover:bg-editor-border text-text-primary border border-editor-border rounded transition-colors"
                title="6 instances (Full stack team)"
              >
                Medium (6)
              </button>
              <button
                onClick={() => quickSetup(10)}
                className="px-3 py-1 text-xs bg-editor-bg hover:bg-editor-border text-text-primary border border-editor-border rounded transition-colors"
                title="10 instances (Large team)"
              >
                Large (10)
              </button>
            </div>
          </div>
        </div>
        <div className="text-xs text-text-tertiary mt-2">
          💡 Quick setup will replace all current instances with the specified count
        </div>
      </div>

      {/* Execution Mode Toggle */}
      <div className="px-4 py-3 border-b border-editor-border bg-editor-surface">
        <div className="flex items-center justify-between">
          <div>
            <label className="block text-xs font-medium text-text-secondary mb-1">
              Execution Mode
            </label>
            <p className="text-xs text-text-tertiary">
              {useTmuxMode
                ? '🎬 Tmux: Isolated sessions with full terminal access'
                : '🚀 Standard: Direct agent execution'}
            </p>
          </div>
          <button
            onClick={() => setUseTmuxMode(!useTmuxMode)}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
              useTmuxMode ? 'bg-accent-primary' : 'bg-editor-border'
            }`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                useTmuxMode ? 'translate-x-6' : 'translate-x-1'
              }`}
            />
          </button>
        </div>
        {useTmuxMode && (
          <div className="mt-2 p-2 bg-accent-primary/10 border border-accent-primary/30 rounded text-xs text-text-secondary">
            <div className="flex items-start gap-2">
              <Terminal size={14} className="text-accent-primary mt-0.5 flex-shrink-0" />
              <div className="flex-1">
                <strong className="text-accent-primary">AIT42 Tmux + Worktree Integration:</strong>
                <ul className="mt-1 space-y-1 list-disc list-inside">
                  <li>Each agent runs in <strong>isolated tmux session</strong></li>
                  <li>Each agent works in <strong>dedicated git worktree</strong></li>
                  <li>Access live output: <code className="px-1 bg-editor-bg rounded">tmux attach -t ait42-{'{agent}'}</code></li>
                  <li>Maximum {instances.length} parallel sessions</li>
                  <li>Worktree path: <code className="px-1 bg-editor-bg rounded">../ait42-worktrees/</code></li>
                  <li>Branch naming: <code className="px-1 bg-editor-bg rounded">ait42/{'{role}'}/{'{timestamp}'}</code></li>
                </ul>
              </div>
              <GitBranch size={14} className="text-accent-primary mt-0.5 flex-shrink-0" />
            </div>
          </div>
        )}
      </div>

      {/* Global Task */}
      <div className="px-4 py-3 border-b border-editor-border bg-editor-bg">
        <div className="flex items-center justify-between mb-2">
          <label className="block text-xs font-medium text-text-secondary">
            Global Project Task
          </label>
          <button
            onClick={handleDistributeTasks}
            disabled={!globalTask.trim() || instances.length === 0}
            className="px-3 py-1 text-xs text-accent-primary hover:text-accent-secondary disabled:text-text-tertiary disabled:cursor-not-allowed transition-colors"
            title="Auto-distribute tasks to all instances based on their roles"
          >
            📋 Distribute Tasks
          </button>
        </div>
        <textarea
          value={globalTask}
          onChange={(e) => setGlobalTask(e.target.value)}
          placeholder="e.g., Build a full-stack e-commerce application with user authentication, product catalog, and payment integration"
          className="w-full px-3 py-2 bg-editor-surface text-text-primary placeholder-text-tertiary border border-editor-border rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/50 resize-none"
          rows={3}
        />
        <div className="text-xs text-text-tertiary mt-1">
          💡 Tip: Click "Distribute Tasks" to automatically generate role-specific subtasks from the global task
        </div>
      </div>

      {/* Instances List */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {instances.map((instance) => (
          <div
            key={instance.id}
            className="bg-editor-surface border border-editor-border rounded-lg p-4 space-y-3"
          >
            {/* Instance Header */}
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-2">
                {getStatusIcon(instance.status)}
                <div>
                  <div className="text-sm font-medium text-text-primary">
                    {instance.name}
                  </div>
                  <div className={`text-xs ${getStatusColor(instance.status)}`}>
                    {instance.status} • {getDuration(instance)}
                  </div>
                </div>
              </div>
              <div className="flex gap-1">
                {instance.status === 'idle' && (
                  <button
                    onClick={() => startInstance(instance.id)}
                    disabled={!instance.task}
                    className="p-1 text-green-400 hover:bg-green-400/10 rounded disabled:opacity-30 disabled:cursor-not-allowed"
                    title="Start"
                  >
                    <Play size={16} />
                  </button>
                )}
                {instance.status === 'running' && (
                  <button
                    onClick={() => stopInstance(instance.id)}
                    className="p-1 text-yellow-400 hover:bg-yellow-400/10 rounded"
                    title="Pause"
                  >
                    <Pause size={16} />
                  </button>
                )}
                <button
                  onClick={() => removeInstance(instance.id)}
                  className="p-1 text-red-400 hover:bg-red-400/10 rounded"
                  title="Remove"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            </div>

            {/* Role */}
            <div>
              <label className="block text-xs text-text-tertiary mb-1">Role</label>
              <select
                value={instance.role}
                onChange={(e) => updateInstanceRole(instance.id, e.target.value)}
                disabled={instance.status !== 'idle'}
                className="w-full px-2 py-1 text-xs bg-editor-bg text-text-primary border border-editor-border rounded focus:outline-none focus:ring-1 focus:ring-accent-primary/50 disabled:opacity-50"
              >
                <option value="Frontend Developer">Frontend Developer</option>
                <option value="Backend Developer">Backend Developer</option>
                <option value="Test Engineer">Test Engineer</option>
                <option value="DevOps Engineer">DevOps Engineer</option>
                <option value="Security Specialist">Security Specialist</option>
                <option value="Database Designer">Database Designer</option>
              </select>
            </div>

            {/* Task */}
            <div>
              <label className="block text-xs text-text-tertiary mb-1">Specific Task</label>
              <textarea
                value={instance.task}
                onChange={(e) => updateInstanceTask(instance.id, e.target.value)}
                disabled={instance.status !== 'idle'}
                placeholder={`What should ${instance.name} work on?`}
                className="w-full px-2 py-1 text-xs bg-editor-bg text-text-primary placeholder-text-tertiary border border-editor-border rounded focus:outline-none focus:ring-1 focus:ring-accent-primary/50 resize-none disabled:opacity-50"
                rows={2}
              />
            </div>

            {/* Output */}
            {instance.output.length > 0 && (
              <div className="mt-2">
                <label className="block text-xs text-text-tertiary mb-1">Output</label>
                <div className="bg-editor-bg border border-editor-border rounded p-2 max-h-32 overflow-y-auto">
                  {instance.output.map((line, idx) => (
                    <div key={idx} className="text-xs text-text-secondary font-mono">
                      {line}
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Footer Actions */}
      <div className="flex items-center justify-between px-4 py-3 border-t border-editor-border bg-editor-surface">
        <div className="flex gap-2">
          <button
            onClick={addInstance}
            className="flex items-center gap-2 px-3 py-2 text-sm text-accent-primary hover:bg-accent-primary/10 rounded transition-colors"
          >
            <Plus size={16} />
            Add Claude Instance
          </button>
          <button
            onClick={() => setShowComparison(!showComparison)}
            disabled={instances.filter((i) => i.status === 'completed' || i.status === 'failed').length < 2}
            className="flex items-center gap-2 px-3 py-2 text-sm text-blue-400 hover:bg-blue-400/10 rounded transition-colors disabled:text-text-tertiary disabled:cursor-not-allowed"
            title="Compare results from completed instances"
          >
            <Code2 size={16} />
            {showComparison ? 'Hide' : 'Compare'} Results
          </button>
        </div>
        <button
          onClick={startAll}
          disabled={instances.every((inst) => !inst.task || inst.status !== 'idle')}
          className="px-4 py-2 bg-gradient-to-r from-accent-primary to-accent-secondary hover:from-accent-secondary hover:to-accent-primary disabled:from-editor-border disabled:to-editor-border disabled:text-text-tertiary text-white text-sm font-semibold rounded-lg transition-all"
        >
          <Play size={16} className="inline mr-2" />
          Start All Instances
        </button>
      </div>

      {/* Comparison Panel */}
      {showComparison && (
        <div className="border-t border-editor-border bg-editor-bg max-h-96 overflow-y-auto">
          <div className="p-4">
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-sm font-semibold text-text-primary">Results Comparison</h3>
              <button
                onClick={() => setShowComparison(false)}
                className="p-1 hover:bg-editor-border/30 rounded transition-colors"
              >
                <XCircle size={16} className="text-text-tertiary" />
              </button>
            </div>

            <div className="grid grid-cols-2 gap-3">
              {instances
                .filter((inst) => inst.status === 'completed' || inst.status === 'failed')
                .map((instance) => (
                  <div
                    key={instance.id}
                    className="bg-editor-surface border border-editor-border rounded-lg p-3"
                  >
                    {/* Instance Header */}
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        {getStatusIcon(instance.status)}
                        <div className="text-sm font-medium text-text-primary">
                          {instance.name}
                        </div>
                      </div>
                      <div className="text-xs text-text-tertiary">
                        {getDuration(instance)}
                      </div>
                    </div>

                    {/* Role & Status */}
                    <div className="text-xs text-text-secondary mb-2">
                      {instance.role} • {instance.status}
                    </div>

                    {/* Metrics */}
                    {instance.metrics && (
                      <div className="grid grid-cols-2 gap-2 mb-2 p-2 bg-editor-bg rounded">
                        {instance.metrics.linesOfCode !== undefined && (
                          <div className="text-xs">
                            <span className="text-text-tertiary">Lines:</span>{' '}
                            <span className="text-text-primary font-medium">
                              {instance.metrics.linesOfCode}
                            </span>
                          </div>
                        )}
                        {instance.metrics.filesModified !== undefined && (
                          <div className="text-xs">
                            <span className="text-text-tertiary">Files:</span>{' '}
                            <span className="text-text-primary font-medium">
                              {instance.metrics.filesModified}
                            </span>
                          </div>
                        )}
                        {instance.metrics.testsAdded !== undefined && (
                          <div className="text-xs">
                            <span className="text-text-tertiary">Tests:</span>{' '}
                            <span className="text-text-primary font-medium">
                              {instance.metrics.testsAdded}
                            </span>
                          </div>
                        )}
                        {instance.metrics.coveragePercent !== undefined && (
                          <div className="text-xs">
                            <span className="text-text-tertiary">Coverage:</span>{' '}
                            <span className="text-text-primary font-medium">
                              {instance.metrics.coveragePercent}%
                            </span>
                          </div>
                        )}
                      </div>
                    )}

                    {/* Output Summary */}
                    <div className="text-xs text-text-tertiary">
                      {instance.output.length} output lines
                    </div>

                    {/* Last output line */}
                    {instance.output.length > 0 && (
                      <div className="mt-2 p-2 bg-editor-bg rounded text-xs font-mono text-text-secondary truncate">
                        {instance.output[instance.output.length - 1]}
                      </div>
                    )}
                  </div>
                ))}
            </div>

            {/* Summary Statistics */}
            <div className="mt-4 p-3 bg-editor-surface border border-editor-border rounded-lg">
              <div className="text-xs font-semibold text-text-secondary mb-2">Summary</div>
              <div className="grid grid-cols-4 gap-4 text-xs">
                <div>
                  <div className="text-text-tertiary">Completed</div>
                  <div className="text-lg font-bold text-green-400">
                    {instances.filter((i) => i.status === 'completed').length}
                  </div>
                </div>
                <div>
                  <div className="text-text-tertiary">Failed</div>
                  <div className="text-lg font-bold text-red-400">
                    {instances.filter((i) => i.status === 'failed').length}
                  </div>
                </div>
                <div>
                  <div className="text-text-tertiary">Running</div>
                  <div className="text-lg font-bold text-blue-400">
                    {instances.filter((i) => i.status === 'running').length}
                  </div>
                </div>
                <div>
                  <div className="text-text-tertiary">Avg Duration</div>
                  <div className="text-lg font-bold text-text-primary">
                    {(() => {
                      const completed = instances.filter(
                        (i) => (i.status === 'completed' || i.status === 'failed') && i.startTime && i.endTime
                      );
                      if (completed.length === 0) return '-';
                      const avgMs =
                        completed.reduce((sum, i) => sum + ((i.endTime || 0) - (i.startTime || 0)), 0) /
                        completed.length;
                      return `${Math.floor(avgMs / 1000)}s`;
                    })()}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
