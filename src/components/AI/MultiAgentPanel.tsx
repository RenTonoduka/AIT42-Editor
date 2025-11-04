import React, { useState, useEffect } from 'react';
import {
  Users,
  GitBranch,
  FolderTree,
  Terminal,
  Clock,
  CheckCircle2,
  XCircle,
  Loader2,
  ChevronDown,
  ChevronUp,
  Activity,
  Cpu,
  MemoryStick,
  FileCode,
} from 'lucide-react';

export interface ClaudeCodeInstance {
  id: string;
  agentName?: string;
  task: string;
  status: 'idle' | 'running' | 'completed' | 'failed' | 'paused';
  output?: string;
  startTime?: number | string;
  endTime?: number | string;
  tmuxSessionId?: string;
  worktreePath?: string;
  worktreeBranch?: string;
}

export interface MultiAgentPanelProps {
  instances: ClaudeCodeInstance[];
}

const MultiAgentPanel: React.FC<MultiAgentPanelProps> = ({ instances }) => {
  const [expandedLogs, setExpandedLogs] = useState<Set<string>>(new Set());
  const [ensembleMode, setEnsembleMode] = useState(false);

  // Toggle log expansion for specific instance
  const toggleLog = (instanceId: string) => {
    const newExpanded = new Set(expandedLogs);
    if (newExpanded.has(instanceId)) {
      newExpanded.delete(instanceId);
    } else {
      newExpanded.add(instanceId);
    }
    setExpandedLogs(newExpanded);
  };

  // Auto-detect ensemble mode
  useEffect(() => {
    if (instances.length >= 3) {
      setEnsembleMode(true);
    } else {
      setEnsembleMode(false);
    }
  }, [instances.length]);

  // Status color mapping
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running':
        return 'text-blue-400 bg-blue-500/10 border-blue-500/30';
      case 'completed':
        return 'text-green-400 bg-green-500/10 border-green-500/30';
      case 'failed':
        return 'text-red-400 bg-red-500/10 border-red-500/30';
      default:
        return 'text-gray-400 bg-gray-500/10 border-gray-500/30';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'running':
        return <Loader2 className="w-5 h-5 animate-spin" />;
      case 'completed':
        return <CheckCircle2 className="w-5 h-5" />;
      case 'failed':
        return <XCircle className="w-5 h-5" />;
      default:
        return <Clock className="w-5 h-5" />;
    }
  };

  // Empty state
  if (instances.length === 0) {
    return (
      <div className="flex items-center justify-center h-full bg-gray-900">
        <div className="text-center">
          <Users className="w-16 h-16 text-gray-600 mx-auto mb-4" />
          <h3 className="text-xl font-semibold text-gray-400 mb-2">
            No Active Agents
          </h3>
          <p className="text-gray-500 text-sm">
            Multi-agent tasks will appear here when started
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full bg-gray-900 overflow-y-auto">
      {/* Header */}
      <div className="sticky top-0 z-10 bg-gray-800 border-b border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <Users className="w-6 h-6 text-purple-400" />
            <div>
              <h2 className="text-xl font-bold text-white">
                Multi-Agent Execution Dashboard
              </h2>
              <p className="text-sm text-gray-400 mt-1">
                {ensembleMode ? (
                  <>
                    <Activity className="w-4 h-4 inline-block mr-1" />
                    Ensemble Mode: {instances.length} agents collaborating
                  </>
                ) : (
                  <>Active Tasks: {instances.length}</>
                )}
              </p>
            </div>
          </div>

          {/* Overall Statistics */}
          <div className="flex items-center space-x-6">
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-400">
                {instances.filter((i) => i.status === 'running').length}
              </div>
              <div className="text-xs text-gray-400">Running</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-green-400">
                {instances.filter((i) => i.status === 'completed').length}
              </div>
              <div className="text-xs text-gray-400">Completed</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-red-400">
                {instances.filter((i) => i.status === 'failed').length}
              </div>
              <div className="text-xs text-gray-400">Failed</div>
            </div>
          </div>
        </div>
      </div>

      {/* Agent Cards Grid */}
      <div className="p-6 space-y-6">
        {instances.map((instance) => (
          <div
            key={instance.id}
            className={`bg-gray-800 rounded-lg border-2 ${getStatusColor(
              instance.status
            )} overflow-hidden transition-all duration-300 hover:shadow-2xl hover:scale-[1.01]`}
          >
            {/* Card Header */}
            <div className="p-6 border-b border-gray-700">
              <div className="flex items-start justify-between">
                {/* Left: Agent Info */}
                <div className="flex-1">
                  <div className="flex items-center space-x-3 mb-3">
                    {getStatusIcon(instance.status)}
                    <h3 className="text-xl font-bold text-white">
                      {instance.agentName || 'Unnamed Agent'}
                    </h3>
                    <span
                      className={`px-3 py-1 rounded-full text-xs font-semibold uppercase ${getStatusColor(
                        instance.status
                      )}`}
                    >
                      {instance.status}
                    </span>
                  </div>

                  {/* Task Description */}
                  <div className="bg-gray-900/50 rounded-lg p-4 mb-4">
                    <div className="flex items-start space-x-2">
                      <FileCode className="w-5 h-5 text-blue-400 mt-0.5 flex-shrink-0" />
                      <div className="flex-1">
                        <div className="text-sm text-gray-400 mb-1">Task:</div>
                        <p className="text-base text-gray-200 leading-relaxed">
                          {instance.task || 'No task description'}
                        </p>
                      </div>
                    </div>
                  </div>

                  {/* Worktree & Branch Information */}
                  <div className="grid grid-cols-2 gap-4">
                    {/* Worktree Path */}
                    <div className="bg-gray-900/30 rounded-lg p-3">
                      <div className="flex items-center space-x-2 mb-2">
                        <FolderTree className="w-4 h-4 text-yellow-400" />
                        <span className="text-xs font-semibold text-gray-400 uppercase">
                          Worktree
                        </span>
                      </div>
                      <div className="text-sm font-mono text-yellow-300 break-all">
                        {instance.worktreePath || 'Unknown worktree'}
                      </div>
                    </div>

                    {/* Branch */}
                    <div className="bg-gray-900/30 rounded-lg p-3">
                      <div className="flex items-center space-x-2 mb-2">
                        <GitBranch className="w-4 h-4 text-green-400" />
                        <span className="text-xs font-semibold text-gray-400 uppercase">
                          Branch
                        </span>
                      </div>
                      <div className="text-sm font-mono text-green-300">
                        {instance.worktreeBranch || 'Unknown branch'}
                      </div>
                    </div>
                  </div>
                </div>

                {/* Right: Metrics */}
                <div className="ml-6 space-y-3">
                  {/* Tmux Session */}
                  {instance.tmuxSessionId && (
                    <div className="bg-gray-900/50 rounded-lg px-3 py-2">
                      <div className="flex items-center space-x-2">
                        <Terminal className="w-4 h-4 text-purple-400" />
                        <div>
                          <div className="text-xs text-gray-400">Tmux Session</div>
                          <div className="text-xs font-mono text-purple-300">
                            {instance.tmuxSessionId}
                          </div>
                        </div>
                      </div>
                    </div>
                  )}

                  {/* Duration */}
                  {instance.startTime && (
                    <div className="bg-gray-900/50 rounded-lg px-3 py-2">
                      <div className="flex items-center space-x-2">
                        <Clock className="w-4 h-4 text-blue-400" />
                        <div>
                          <div className="text-xs text-gray-400">Duration</div>
                          <div className="text-xs font-mono text-blue-300">
                            {new Date(
                              Date.now() - new Date(instance.startTime).getTime()
                            ).toISOString().substr(11, 8)}
                          </div>
                        </div>
                      </div>
                    </div>
                  )}

                  {/* CPU Usage (placeholder) */}
                  <div className="bg-gray-900/50 rounded-lg px-3 py-2">
                    <div className="flex items-center space-x-2">
                      <Cpu className="w-4 h-4 text-orange-400" />
                      <div>
                        <div className="text-xs text-gray-400">CPU</div>
                        <div className="text-xs font-mono text-orange-300">
                          {Math.floor(Math.random() * 60 + 20)}%
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* Memory Usage (placeholder) */}
                  <div className="bg-gray-900/50 rounded-lg px-3 py-2">
                    <div className="flex items-center space-x-2">
                      <MemoryStick className="w-4 h-4 text-cyan-400" />
                      <div>
                        <div className="text-xs text-gray-400">Memory</div>
                        <div className="text-xs font-mono text-cyan-300">
                          {Math.floor(Math.random() * 300 + 100)} MB
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* Execution Log */}
            <div className="border-t border-gray-700">
              <button
                onClick={() => toggleLog(instance.id)}
                className="w-full px-6 py-3 flex items-center justify-between hover:bg-gray-700/30 transition-colors"
              >
                <div className="flex items-center space-x-2">
                  <Terminal className="w-4 h-4 text-gray-400" />
                  <span className="text-sm font-semibold text-gray-300">
                    Execution Log
                  </span>
                  {instance.output && (
                    <span className="text-xs text-gray-500">
                      ({instance.output.split('\n').length} lines)
                    </span>
                  )}
                </div>
                {expandedLogs.has(instance.id) ? (
                  <ChevronUp className="w-5 h-5 text-gray-400" />
                ) : (
                  <ChevronDown className="w-5 h-5 text-gray-400" />
                )}
              </button>

              {expandedLogs.has(instance.id) && (
                <div className="px-6 pb-6">
                  <div className="bg-black/50 rounded-lg p-4 max-h-96 overflow-y-auto">
                    <pre className="text-xs text-green-400 font-mono whitespace-pre-wrap">
                      {instance.output || 'No output yet...'}
                    </pre>
                  </div>
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default MultiAgentPanel;
