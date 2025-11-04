/**
 * MultiAgentPanel - Parallel Multi-Claude Code Development
 *
 * Manages multiple Claude Code instances working in parallel
 * on different aspects of a project (Frontend, Backend, Testing, etc.)
 */

import React, { useState, useEffect } from 'react';
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

  // Start single instance
  const startInstance = async (id: string) => {
    const instance = instances.find((inst) => inst.id === id);
    if (!instance || !instance.task) return;

    // Update status to running
    setInstances(
      instances.map((inst) =>
        inst.id === id
          ? { ...inst, status: 'running', startTime: Date.now(), output: [`Starting ${inst.role}...`] }
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
                output: [...inst.output, `Agent ${agentName} started`, `Execution ID: ${response.executionId}`],
              }
            : inst
        )
      );

      // Simulate progress updates (in real implementation, poll for status)
      setTimeout(() => {
        setInstances((prev) =>
          prev.map((inst) =>
            inst.id === id
              ? {
                  ...inst,
                  status: 'completed',
                  endTime: Date.now(),
                  output: [...inst.output, 'Task completed successfully!'],
                }
              : inst
          )
        );
      }, 5000);
    } catch (error) {
      setInstances(
        instances.map((inst) =>
          inst.id === id
            ? {
                ...inst,
                status: 'failed',
                endTime: Date.now(),
                output: [...inst.output, `Error: ${error}`],
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
    setInstances(
      instances.map((inst) =>
        inst.id === id ? { ...inst, status: 'paused' } : inst
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
        <label className="block text-xs font-medium text-text-secondary mb-2">
          Global Project Task
        </label>
        <input
          type="text"
          value={globalTask}
          onChange={(e) => setGlobalTask(e.target.value)}
          placeholder="e.g., Build a full-stack e-commerce application"
          className="w-full px-3 py-2 bg-editor-surface text-text-primary placeholder-text-tertiary border border-editor-border rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/50"
        />
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
        <button
          onClick={addInstance}
          className="flex items-center gap-2 px-3 py-2 text-sm text-accent-primary hover:bg-accent-primary/10 rounded transition-colors"
        >
          <Plus size={16} />
          Add Claude Instance
        </button>
        <button
          onClick={startAll}
          disabled={instances.every((inst) => !inst.task || inst.status !== 'idle')}
          className="px-4 py-2 bg-gradient-to-r from-accent-primary to-accent-secondary hover:from-accent-secondary hover:to-accent-primary disabled:from-editor-border disabled:to-editor-border disabled:text-text-tertiary text-white text-sm font-semibold rounded-lg transition-all"
        >
          <Play size={16} className="inline mr-2" />
          Start All Instances
        </button>
      </div>
    </div>
  );
};
