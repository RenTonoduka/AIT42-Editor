/**
 * MultiAgentPanel - Parallel Multi-Claude Code Development
 *
 * Manages multiple Claude Code instances working in parallel
 * on different aspects of a project (Frontend, Backend, Testing, etc.)
 */

import React, { useState, useEffect, useRef } from 'react';
import { Users, Play, Pause, Square, Trash2, Plus, Code2, CheckCircle, XCircle, Clock, Terminal, GitBranch, Sparkles } from 'lucide-react';
import { tauriApi, AgentExecutionResponse, TmuxSession, WorktreeInfo } from '@/services/tauri';
import { ModeIndicator } from './ModeIndicator';
import { CollaborativeFlowDiagram } from './CollaborativeFlowDiagram';
import { ModeTooltip } from './ModeTooltip';

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
  const [ensembleResult, setEnsembleResult] = useState<string>(''); // Integrated result from all instances
  const [isIntegrating, setIsIntegrating] = useState(false); // Integration in progress

  // Track active polling intervals for cleanup
  const pollingIntervalsRef = useRef<Map<string, NodeJS.Timeout>>(new Map());
  const hasIntegratedRef = useRef<boolean>(false); // Prevent duplicate integration

  // Cleanup polling intervals on unmount
  useEffect(() => {
    return () => {
      // Clear all active polling intervals when component unmounts
      pollingIntervalsRef.current.forEach((interval) => clearInterval(interval));
      pollingIntervalsRef.current.clear();
    };
  }, []);

  // Auto-integrate results when all instances complete (Ensemble Mode)
  useEffect(() => {
    // Check if all instances have completed (successfully or with failure)
    const allCompleted = instances.length > 0 && instances.every(
      (inst) => inst.status === 'completed' || inst.status === 'failed'
    );

    // Check if at least one instance succeeded
    const hasSuccess = instances.some((inst) => inst.status === 'completed');

    // Trigger integration if:
    // 1. All instances are done
    // 2. At least one succeeded
    // 3. Haven't integrated yet
    // 4. Not currently integrating
    if (allCompleted && hasSuccess && !hasIntegratedRef.current && !isIntegrating) {
      hasIntegratedRef.current = true;
      performEnsembleIntegration();
    }
  }, [instances, isIntegrating]);

  // Perform ensemble integration - merge results from all instances
  const performEnsembleIntegration = async () => {
    setIsIntegrating(true);

    try {
      console.log('🔮 Starting Ensemble Integration...');

      // Collect outputs from all completed instances
      const completedInstances = instances.filter((inst) => inst.status === 'completed');

      if (completedInstances.length === 0) {
        setEnsembleResult('⚠️ No instances completed successfully. Cannot perform integration.');
        setIsIntegrating(false);
        return;
      }

      // Build integration context
      const integrationContext = {
        task: globalTask,
        instanceCount: completedInstances.length,
        results: completedInstances.map((inst) => ({
          role: inst.role,
          name: inst.name,
          output: inst.output.join('\n'),
          duration: inst.endTime && inst.startTime ? inst.endTime - inst.startTime : 0,
        })),
      };

      // Call integration AI agent (using code-reviewer or a dedicated integration agent)
      const integrationPrompt = `
# Ensemble Integration Task

## Global Task
${globalTask}

## Individual Results

${completedInstances.map((inst, idx) => `
### Instance ${idx + 1}: ${inst.name} (${inst.role})
**Duration**: ${inst.endTime && inst.startTime ? Math.floor((inst.endTime - inst.startTime) / 1000) : '?'}s
**Output**:
\`\`\`
${inst.output.slice(-20).join('\n')}
\`\`\`
`).join('\n')}

## Your Task
Analyze all ${completedInstances.length} results and synthesize the best integrated solution that:
1. Combines the strengths of each approach
2. Identifies and resolves conflicts
3. Provides a unified, coherent recommendation
4. Highlights key insights from each instance

**Output Format**: Provide a clear, structured integration report.
`;

      console.log('📤 Sending integration request to AI...');

      // Execute integration using code-reviewer agent (or create dedicated integration agent)
      const response = await tauriApi.executeAgent({
        agentName: 'code-reviewer', // Using code-reviewer for integration analysis
        task: integrationPrompt,
        context: 'Ensemble Mode Integration',
      });

      console.log(`✅ Integration started: ${response.executionId}`);

      // Poll for integration result
      const pollIntegrationResult = async () => {
        const maxPolls = 60; // 60 seconds max
        let pollCount = 0;

        const pollInterval = setInterval(async () => {
          try {
            const output = await tauriApi.getAgentOutput(response.executionId);

            if (output.status === 'completed') {
              clearInterval(pollInterval);
              setEnsembleResult(output.output || 'Integration completed (no output)');
              setIsIntegrating(false);
              console.log('🎉 Ensemble Integration Complete!');
            } else if (output.status === 'failed') {
              clearInterval(pollInterval);
              setEnsembleResult(`❌ Integration failed: ${output.error || 'Unknown error'}`);
              setIsIntegrating(false);
            }

            pollCount++;
            if (pollCount >= maxPolls) {
              clearInterval(pollInterval);
              setEnsembleResult('⏱️ Integration timeout');
              setIsIntegrating(false);
            }
          } catch (error) {
            console.error('Failed to poll integration result:', error);
          }
        }, 1000);
      };

      await pollIntegrationResult();
    } catch (error) {
      console.error('Ensemble integration error:', error);
      setEnsembleResult(`❌ Integration error: ${error}`);
      setIsIntegrating(false);
    }
  };

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

  // Start all instances in parallel (TRUE PARALLEL EXECUTION)
  const startAll = async () => {
    try {
      const runnableInstances = instances.filter(
        (inst) => inst.task && inst.status === 'idle'
      );

      if (runnableInstances.length === 0) {
        console.warn('No runnable instances found');
        return;
      }

      console.log(`🚀 Starting ${runnableInstances.length} instances in parallel (Ensemble Mode)`);

      // Execute ALL instances in parallel using Promise.all
      const startPromises = runnableInstances.map((instance) =>
        startInstance(instance.id).catch((error) => {
          console.error(`Failed to start instance ${instance.id}:`, error);
          // Return error but don't throw - Promise.all will continue
          return { error, instanceId: instance.id };
        })
      );

      // Wait for all instances to start (not complete, just start)
      const results = await Promise.all(startPromises);

      // Log any failures
      const failures = results.filter((r) => r && typeof r === 'object' && 'error' in r);
      if (failures.length > 0) {
        console.warn(`${failures.length} instance(s) failed to start`);
      }

      console.log(`✅ All ${runnableInstances.length} instances started in parallel`);
    } catch (error) {
      console.error('Error in startAll:', error);
      // Don't close the panel, just log the error
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

  // Get status text in Japanese
  const getStatusText = (status: ClaudeCodeInstance['status']): string => {
    switch (status) {
      case 'idle':
        return 'アイドル';
      case 'running':
        return '実行中';
      case 'completed':
        return '完了';
      case 'failed':
        return '失敗';
      case 'paused':
        return '一時停止';
    }
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
            マルチエージェント並列開発
          </h2>
          <ModeIndicator mode="collaborative" />
          <ModeTooltip mode="collaborative" />
          <div className="ml-2 px-2 py-0.5 bg-accent-primary/20 text-accent-primary text-xs rounded-full font-medium">
            {instances.length} インスタンス
          </div>
        </div>
        <button
          onClick={onClose}
          className="p-1 hover:bg-editor-border/30 rounded transition-colors"
        >
          <Square size={16} className="text-text-tertiary" />
        </button>
      </div>

      {/* Flow Diagram */}
      <div className="px-4 py-3 border-b border-editor-border bg-editor-bg">
        <CollaborativeFlowDiagram />
      </div>

      {/* Instance Count Control */}
      <div className="px-4 py-3 border-b border-editor-border bg-editor-surface">
        <div className="flex items-center gap-3">
          <div className="flex-1">
            <label className="block text-xs font-medium text-text-secondary mb-1">
              インスタンス数
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
                適用
              </button>
            </div>
          </div>
          <div className="flex-1">
            <label className="block text-xs font-medium text-text-secondary mb-1">
              クイックプリセット
            </label>
            <div className="flex gap-2">
              <button
                onClick={() => quickSetup(3)}
                className="px-3 py-1 text-xs bg-editor-bg hover:bg-editor-border text-text-primary border border-editor-border rounded transition-colors"
                title="3インスタンス (Frontend, Backend, Tester)"
              >
                小 (3)
              </button>
              <button
                onClick={() => quickSetup(6)}
                className="px-3 py-1 text-xs bg-editor-bg hover:bg-editor-border text-text-primary border border-editor-border rounded transition-colors"
                title="6インスタンス (フルスタックチーム)"
              >
                中 (6)
              </button>
              <button
                onClick={() => quickSetup(10)}
                className="px-3 py-1 text-xs bg-editor-bg hover:bg-editor-border text-text-primary border border-editor-border rounded transition-colors"
                title="10インスタンス (大規模チーム)"
              >
                大 (10)
              </button>
            </div>
          </div>
        </div>
        <div className="text-xs text-text-tertiary mt-2">
          💡 クイックセットアップは現在のインスタンスをすべて指定数に置き換えます
        </div>
      </div>

      {/* Execution Mode Toggle */}
      <div className="px-4 py-3 border-b border-editor-border bg-editor-surface">
        <div className="flex items-center justify-between">
          <div>
            <label className="block text-xs font-medium text-text-secondary mb-1">
              実行モード
            </label>
            <p className="text-xs text-text-tertiary">
              {useTmuxMode
                ? '🎬 Tmux: 完全なターミナルアクセスを持つ分離セッション'
                : '🚀 標準: 直接エージェント実行'}
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
                <strong className="text-accent-primary">AIT42 Tmux + Worktree 統合:</strong>
                <ul className="mt-1 space-y-1 list-disc list-inside">
                  <li>各エージェントは<strong>分離されたtmuxセッション</strong>で実行</li>
                  <li>各エージェントは<strong>専用のgit worktree</strong>で作業</li>
                  <li>ライブ出力へアクセス: <code className="px-1 bg-editor-bg rounded">tmux attach -t ait42-{'{agent}'}</code></li>
                  <li>最大 {instances.length} 個の並列セッション</li>
                  <li>Worktreeパス: <code className="px-1 bg-editor-bg rounded">../ait42-worktrees/</code></li>
                  <li>ブランチ命名: <code className="px-1 bg-editor-bg rounded">ait42/{'{role}'}/{'{timestamp}'}</code></li>
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
            全体プロジェクトタスク
          </label>
          <button
            onClick={handleDistributeTasks}
            disabled={!globalTask.trim() || instances.length === 0}
            className="px-3 py-1 text-xs text-accent-primary hover:text-accent-secondary disabled:text-text-tertiary disabled:cursor-not-allowed transition-colors"
            title="役割に基づいてすべてのインスタンスにタスクを自動分配"
          >
            📋 タスクを分配
          </button>
        </div>
        <textarea
          value={globalTask}
          onChange={(e) => setGlobalTask(e.target.value)}
          placeholder="例: ユーザー認証、商品カタログ、決済統合を備えたフルスタックECアプリケーションを構築"
          className="w-full px-3 py-2 bg-editor-surface text-text-primary placeholder-text-tertiary border border-editor-border rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/50 resize-none"
          rows={3}
        />
        <div className="text-xs text-text-tertiary mt-1">
          💡 ヒント: 「タスクを分配」をクリックすると、全体タスクから役割別のサブタスクが自動生成されます
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
                    {getStatusText(instance.status)} • {getDuration(instance)}
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
              <label className="block text-xs text-text-tertiary mb-1">役割</label>
              <select
                value={instance.role}
                onChange={(e) => updateInstanceRole(instance.id, e.target.value)}
                disabled={instance.status !== 'idle'}
                className="w-full px-2 py-1 text-xs bg-editor-bg text-text-primary border border-editor-border rounded focus:outline-none focus:ring-1 focus:ring-accent-primary/50 disabled:opacity-50"
              >
                <option value="Frontend Developer">フロントエンド開発者</option>
                <option value="Backend Developer">バックエンド開発者</option>
                <option value="Test Engineer">テストエンジニア</option>
                <option value="DevOps Engineer">DevOpsエンジニア</option>
                <option value="Security Specialist">セキュリティスペシャリスト</option>
                <option value="Database Designer">データベース設計者</option>
              </select>
            </div>

            {/* Task */}
            <div>
              <label className="block text-xs text-text-tertiary mb-1">具体的なタスク</label>
              <textarea
                value={instance.task}
                onChange={(e) => updateInstanceTask(instance.id, e.target.value)}
                disabled={instance.status !== 'idle'}
                placeholder={`${instance.name}に何を作業させますか？`}
                className="w-full px-2 py-1 text-xs bg-editor-bg text-text-primary placeholder-text-tertiary border border-editor-border rounded focus:outline-none focus:ring-1 focus:ring-accent-primary/50 resize-none disabled:opacity-50"
                rows={2}
              />
            </div>

            {/* Output */}
            {instance.output.length > 0 && (
              <div className="mt-2">
                <label className="block text-xs text-text-tertiary mb-1">出力</label>
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
            Claude インスタンスを追加
          </button>
          <button
            onClick={() => setShowComparison(!showComparison)}
            disabled={instances.filter((i) => i.status === 'completed' || i.status === 'failed').length < 2}
            className="flex items-center gap-2 px-3 py-2 text-sm text-blue-400 hover:bg-blue-400/10 rounded transition-colors disabled:text-text-tertiary disabled:cursor-not-allowed"
            title="完了したインスタンスの結果を比較"
          >
            <Code2 size={16} />
            結果を{showComparison ? '非表示' : '比較'}
          </button>
        </div>
        <button
          onClick={startAll}
          disabled={instances.every((inst) => !inst.task || inst.status !== 'idle')}
          className="px-4 py-2 bg-gradient-to-r from-accent-primary to-accent-secondary hover:from-accent-secondary hover:to-accent-primary disabled:from-editor-border disabled:to-editor-border disabled:text-text-tertiary text-white text-sm font-semibold rounded-lg transition-all"
        >
          <Play size={16} className="inline mr-2" />
          すべてのインスタンスを開始
        </button>
      </div>

      {/* Ensemble Integration Result Panel */}
      {(ensembleResult || isIntegrating) && (
        <div className="border-t border-purple-500/30 bg-gradient-to-r from-purple-900/10 to-blue-900/10">
          <div className="p-4">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-2">
                <Sparkles size={18} className="text-purple-400" />
                <h3 className="text-sm font-semibold text-text-primary">
                  アンサンブル統合結果
                </h3>
                {isIntegrating && (
                  <span className="text-xs text-purple-400 animate-pulse">
                    統合中...
                  </span>
                )}
              </div>
              <button
                onClick={() => {
                  setEnsembleResult('');
                  hasIntegratedRef.current = false;
                }}
                className="p-1 hover:bg-editor-border/30 rounded transition-colors"
                title="統合結果をクリア"
              >
                <XCircle size={16} className="text-text-tertiary" />
              </button>
            </div>

            <div className="bg-editor-surface border border-purple-500/30 rounded-lg p-4">
              {isIntegrating ? (
                <div className="flex items-center justify-center gap-3 py-8">
                  <div className="w-5 h-5 border-2 border-purple-500 border-t-transparent rounded-full animate-spin" />
                  <span className="text-sm text-text-secondary">
                    統合AIが全インスタンスの結果を分析・統合しています...
                  </span>
                </div>
              ) : (
                <div className="prose prose-sm max-w-none text-text-primary">
                  <pre className="whitespace-pre-wrap text-xs font-mono bg-editor-bg p-3 rounded border border-editor-border overflow-x-auto">
                    {ensembleResult}
                  </pre>
                </div>
              )}
            </div>

            <div className="mt-3 p-3 bg-purple-900/10 rounded-lg border border-purple-500/20">
              <div className="text-xs text-text-secondary">
                <strong className="text-purple-400">💡 アンサンブル統合:</strong>
                <ul className="mt-1 space-y-1 list-disc list-inside">
                  <li>全インスタンスの完了後に自動的に統合AIが起動</li>
                  <li>各インスタンスの結果を分析し、強みを組み合わせた最適解を生成</li>
                  <li>矛盾点を検出・解決し、統一された推奨事項を提供</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Comparison Panel */}
      {showComparison && (
        <div className="border-t border-editor-border bg-editor-bg max-h-96 overflow-y-auto">
          <div className="p-4">
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-sm font-semibold text-text-primary">結果比較</h3>
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
                      {instance.role} • {getStatusText(instance.status)}
                    </div>

                    {/* Metrics */}
                    {instance.metrics && (
                      <div className="grid grid-cols-2 gap-2 mb-2 p-2 bg-editor-bg rounded">
                        {instance.metrics.linesOfCode !== undefined && (
                          <div className="text-xs">
                            <span className="text-text-tertiary">行数:</span>{' '}
                            <span className="text-text-primary font-medium">
                              {instance.metrics.linesOfCode}
                            </span>
                          </div>
                        )}
                        {instance.metrics.filesModified !== undefined && (
                          <div className="text-xs">
                            <span className="text-text-tertiary">ファイル数:</span>{' '}
                            <span className="text-text-primary font-medium">
                              {instance.metrics.filesModified}
                            </span>
                          </div>
                        )}
                        {instance.metrics.testsAdded !== undefined && (
                          <div className="text-xs">
                            <span className="text-text-tertiary">テスト:</span>{' '}
                            <span className="text-text-primary font-medium">
                              {instance.metrics.testsAdded}
                            </span>
                          </div>
                        )}
                        {instance.metrics.coveragePercent !== undefined && (
                          <div className="text-xs">
                            <span className="text-text-tertiary">カバレッジ:</span>{' '}
                            <span className="text-text-primary font-medium">
                              {instance.metrics.coveragePercent}%
                            </span>
                          </div>
                        )}
                      </div>
                    )}

                    {/* Output Summary */}
                    <div className="text-xs text-text-tertiary">
                      {instance.output.length} 行の出力
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
              <div className="text-xs font-semibold text-text-secondary mb-2">集計</div>
              <div className="grid grid-cols-4 gap-4 text-xs">
                <div>
                  <div className="text-text-tertiary">完了</div>
                  <div className="text-lg font-bold text-green-400">
                    {instances.filter((i) => i.status === 'completed').length}
                  </div>
                </div>
                <div>
                  <div className="text-text-tertiary">失敗</div>
                  <div className="text-lg font-bold text-red-400">
                    {instances.filter((i) => i.status === 'failed').length}
                  </div>
                </div>
                <div>
                  <div className="text-text-tertiary">実行中</div>
                  <div className="text-lg font-bold text-blue-400">
                    {instances.filter((i) => i.status === 'running').length}
                  </div>
                </div>
                <div>
                  <div className="text-text-tertiary">平均時間</div>
                  <div className="text-lg font-bold text-text-primary">
                    {(() => {
                      const completed = instances.filter(
                        (i) => (i.status === 'completed' || i.status === 'failed') && i.startTime && i.endTime
                      );
                      if (completed.length === 0) return '-';
                      const avgMs =
                        completed.reduce((sum, i) => sum + ((i.endTime || 0) - (i.startTime || 0)), 0) /
                        completed.length;
                      return `${Math.floor(avgMs / 1000)}秒`;
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
