/**
 * MultiAgentPanel - Parallel Multi-Claude Code Development
 *
 * Manages multiple Claude Code instances working in parallel
 * on different aspects of a project (Frontend, Backend, Testing, etc.)
 */

import React, { useState, useEffect, useRef } from 'react';
import { Users, Play, Pause, Square, Trash2, Plus, Code2, CheckCircle, XCircle, Clock } from 'lucide-react';
import { tauriApi, AgentExecutionResponse } from '@/services/tauri';

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

  // Start single instance
  const startInstance = async (id: string) => {
    const instance = instances.find((inst) => inst.id === id);
    if (!instance || !instance.task) return;

    // Update status to running
    setInstances(
      instances.map((inst) =>
        inst.id === id
          ? { ...inst, status: 'running', startTime: Date.now(), output: [`🚀 Starting ${inst.role}...`] }
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
  const stopInstance = (id: string) => {
    // Clear polling interval if exists
    const interval = pollingIntervalsRef.current.get(id);
    if (interval) {
      clearInterval(interval);
      pollingIntervalsRef.current.delete(id);
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
        </div>
        <button
          onClick={onClose}
          className="p-1 hover:bg-editor-border/30 rounded transition-colors"
        >
          <Square size={16} className="text-text-tertiary" />
        </button>
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
