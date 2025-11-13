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
  Folder,
} from 'lucide-react';
import WorktreeExplorer from '@/components/Worktree/WorktreeExplorer';
import { listen, emit } from '@tauri-apps/api/event'; // 🔥 NEW: Tauri event system
import { AgentRuntime } from '@/types/worktree';
import { getRuntimeDefinition } from '@/config/runtimes';
import { useSessionHistoryStore } from '@/store/sessionHistoryStore'; // 🔥 NEW: For session updates
import { tauriApi } from '@/services/tauri'; // 🔥 NEW: For integration phase

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
  runtime?: AgentRuntime;
  model?: string;
  runtimeLabel?: string;
  runtimeEmoji?: string;
}

export interface MultiAgentPanelProps {
  instances: ClaudeCodeInstance[];
  competitionId?: string;
  workspacePath?: string; // 🔥 NEW: For session history updates
}

const MultiAgentPanel: React.FC<MultiAgentPanelProps> = ({ instances, competitionId, workspacePath }) => {
  const [expandedLogs, setExpandedLogs] = useState<Set<string>>(new Set());
  const [ensembleMode, setEnsembleMode] = useState(false);
  const [activeTab, setActiveTab] = useState<'output' | 'worktrees'>('output');
  const [localInstances, setLocalInstances] = useState<ClaudeCodeInstance[]>(instances); // 🔥 NEW: Local state for instances with outputs
  const [sessionUpdated, setSessionUpdated] = useState(false); // 🔥 NEW: Track if session was updated to completed

  // 🔥 NEW: Sync local instances when prop changes
  useEffect(() => {
    setLocalInstances(instances);
  }, [instances]);

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

  // 🔥 NEW: Handshake protocol - Listen for competition-output events
  useEffect(() => {
    if (!competitionId) return; // Only listen if we have a competition ID

    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      try {
        // STEP 1: Register listener
        console.log(`[MultiAgentPanel] Registering listener for competition ${competitionId} at`, Date.now());
        unlisten = await listen<{
          instance: number;
          output: string;
          status?: 'completed' | 'failed';
          error?: string;
        }>('competition-output', (event) => {
          try {
            const { instance, output, status } = event.payload;
            console.log(`[MultiAgentPanel] Received competition-output: instance=${instance}, output_len=${output.length}, status=${status}`);

            setLocalInstances((prev) =>
              prev.map((inst, idx) => {
                // Match by instance number (1-indexed from backend, 0-indexed in array)
                if (idx + 1 === instance) {
                  return {
                    ...inst,
                    output: (inst.output || '') + output,
                    status: status === 'completed' ? 'completed' : status === 'failed' ? 'failed' : inst.status,
                    endTime: status ? Date.now() : inst.endTime,
                  };
                }
                return inst;
              })
            );
          } catch (err) {
            console.error('[MultiAgentPanel] Error handling competition-output event:', err);
          }
        });

        // STEP 2: Send ready signal to backend
        console.log(`[MultiAgentPanel] Listener registered, emitting ready signal for competition ${competitionId}`);
        await emit('competition-listener-ready', { competitionId });
        console.log(`[MultiAgentPanel] Ready signal sent for competition ${competitionId}`);
      } catch (error) {
        console.error('[MultiAgentPanel] Failed to setup competition listener:', error);
      }
    };

    setupListener();

    return () => {
      if (unlisten) {
        console.log(`[MultiAgentPanel] Cleaning up listener for competition ${competitionId}`);
        unlisten();
      }
    };
  }, [competitionId]);

  // 🔥 NEW: Auto-update session status when all instances complete
  const { updateSession, getSession } = useSessionHistoryStore();

  // 🔥 NEW: Helper function to identify integration instances
  const isIntegrationInstance = (instance: ClaudeCodeInstance) => {
    return (
      instance.agentName?.includes('Integration') ||
      instance.agentName?.includes('統合') ||
      instance.id?.includes('integration')
    );
  };

  // 🔥 NEW: Start integration phase
  const startIntegrationPhase = async () => {
    if (!competitionId || !workspacePath) {
      console.warn('[MultiAgentPanel] Cannot start integration: missing competitionId or workspacePath');
      return;
    }

    try {
      console.log('[MultiAgentPanel] Starting integration phase...');

      // 1. Get current session from store
      const session = await getSession(competitionId);
      if (!session) {
        console.warn('[MultiAgentPanel] Session not found:', competitionId);
        return;
      }

      // 2. Request integration phase from backend
      const result = await tauriApi.startIntegrationPhase({
        sessionId: competitionId,
        workspacePath,
        instanceCount: localInstances.length,
        originalTask: session.task,
      });

      console.log('[MultiAgentPanel] Integration phase started:', result);

      // 3. Create integration instance object
      const integrationInstance = {
        instanceId: result.integrationInstanceId,
        worktreePath: result.worktreePath,
        branch: `integration-${competitionId.substring(0, 8)}`,
        agentName: '🔄 Integration Agent',
        status: 'running' as const,
        tmuxSessionId: result.tmuxSessionId,
        startTime: result.startedAt,
        output: '',
      };

      // 4. Update session with integration instance
      const updatedSession = {
        ...session,
        integrationPhase: 'in_progress' as const,
        integrationInstanceId: result.integrationInstanceId,
        instances: [...session.instances, integrationInstance],
        updatedAt: new Date().toISOString(),
      };

      await updateSession(updatedSession);

      // 5. Update local state
      setLocalInstances((prev) => [
        ...prev,
        {
          id: `integration-${competitionId}`,
          agentName: '🔄 Integration Agent',
          task: session.task,
          status: 'running',
          output: '',
          startTime: result.startedAt,
          tmuxSessionId: result.tmuxSessionId,
          worktreePath: result.worktreePath,
          worktreeBranch: `integration-${competitionId.substring(0, 8)}`,
        },
      ]);

      console.log('[MultiAgentPanel] ✅ Integration phase started successfully');
    } catch (error) {
      console.error('[MultiAgentPanel] Failed to start integration phase:', error);
    }
  };

  // 🔥 NEW: Auto-start integration phase for Ensemble mode
  useEffect(() => {
    if (!competitionId || !workspacePath) return;
    if (localInstances.length === 0) return;

    const checkAndStartIntegration = async () => {
      try {
        // Get current session to check type
        const session = await getSession(competitionId);
        if (!session) return;

        // Only for Ensemble mode
        const isEnsemble = session.type === 'ensemble';
        if (!isEnsemble) return;

        // Check if integration phase already started
        const hasIntegrationStarted =
          session.integrationPhase === 'in_progress' ||
          session.integrationPhase === 'completed';
        if (hasIntegrationStarted) return;

        // Check if there's already an integration instance
        const hasIntegrationInstance = localInstances.some(isIntegrationInstance);
        if (hasIntegrationInstance) return;

        // Check if all non-integration instances are completed or failed
        const nonIntegrationInstances = localInstances.filter(
          (inst) => !isIntegrationInstance(inst)
        );
        const allCompleted = nonIntegrationInstances.every(
          (inst) => inst.status === 'completed' || inst.status === 'failed'
        );

        if (allCompleted && nonIntegrationInstances.length > 0) {
          console.log('[MultiAgentPanel] 🔥 All instances completed, starting integration phase...');
          await startIntegrationPhase();
        }
      } catch (error) {
        console.error('[MultiAgentPanel] Error in integration phase check:', error);
      }
    };

    checkAndStartIntegration();
  }, [localInstances, competitionId, workspacePath]);

  // Auto-update session status when all instances complete (including integration)
  useEffect(() => {
    if (!competitionId || !workspacePath || sessionUpdated) return;
    if (localInstances.length === 0) return;

    // Check if all instances are completed or failed
    const allCompleted = localInstances.every(
      (inst) => inst.status === 'completed' || inst.status === 'failed'
    );

    if (allCompleted) {
      console.log('[MultiAgentPanel] All instances completed, updating session status...');

      const updateSessionStatus = async () => {
        try {
          // Get current session from store
          const session = await getSession(competitionId);
          if (!session) {
            console.warn('[MultiAgentPanel] Session not found:', competitionId);
            return;
          }

          // Check if integration instance completed
          const integrationInstance = localInstances.find(isIntegrationInstance);
          const isIntegrationCompleted = integrationInstance?.status === 'completed';

          // Update session status and instances with output
          const updatedSession = {
            ...session,
            status: 'completed' as const,
            completedAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            // If integration completed, mark integration phase as completed
            integrationPhase: isIntegrationCompleted ? ('completed' as const) : session.integrationPhase,
            instances: session.instances.map((inst, idx) => {
              const localInst = localInstances[idx];
              return {
                ...inst,
                status: localInst?.status || inst.status,
                output: localInst?.output || inst.output,
                endTime: localInst?.endTime
                  ? (typeof localInst.endTime === 'string'
                      ? localInst.endTime
                      : new Date(localInst.endTime).toISOString())
                  : inst.endTime,
              };
            }),
          };

          await updateSession(updatedSession);
          setSessionUpdated(true);
          console.log('[MultiAgentPanel] ✅ Session status updated to completed');
        } catch (error) {
          console.error('[MultiAgentPanel] Failed to update session:', error);
        }
      };

      updateSessionStatus();
    }
  }, [localInstances, competitionId, workspacePath, sessionUpdated, updateSession, getSession]);

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

  const renderRuntimeBadge = (instance: ClaudeCodeInstance) => {
    if (!instance.runtime) {
      return null;
    }

    let runtimeLabel = instance.runtimeLabel;
    let runtimeEmoji = instance.runtimeEmoji;
    try {
      if (!runtimeLabel || !runtimeEmoji) {
        const def = getRuntimeDefinition(instance.runtime);
        runtimeLabel = runtimeLabel ?? def.label;
        runtimeEmoji = runtimeEmoji ?? def.emoji;
      }
    } catch (error) {
      console.warn('[MultiAgentPanel] Unknown runtime metadata', instance.runtime, error);
    }

    return (
      <span className="px-2 py-1 bg-gray-900/60 border border-gray-700 rounded-full text-xs font-semibold text-gray-200 flex items-center gap-1">
        {runtimeEmoji && <span>{runtimeEmoji}</span>}
        {runtimeLabel || instance.runtime}
        {instance.model && <span className="text-gray-400">({instance.model})</span>}
      </span>
    );
  };

  // Empty state
  if (localInstances.length === 0) {
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
    <div className="h-full bg-gray-900 flex flex-col">
      {/* Header */}
      <div className="bg-gray-800 border-b border-gray-700 px-6 py-4">
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
                    Ensemble Mode: {localInstances.length} agents collaborating
                  </>
                ) : (
                  <>Active Tasks: {localInstances.length}</>
                )}
              </p>
            </div>
          </div>

          {/* Overall Statistics */}
          <div className="flex items-center space-x-6">
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-400">
                {localInstances.filter((i) => i.status === 'running').length}
              </div>
              <div className="text-xs text-gray-400">Running</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-green-400">
                {localInstances.filter((i) => i.status === 'completed').length}
              </div>
              <div className="text-xs text-gray-400">Completed</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-red-400">
                {localInstances.filter((i) => i.status === 'failed').length}
              </div>
              <div className="text-xs text-gray-400">Failed</div>
            </div>
          </div>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="bg-gray-800 border-b border-gray-700 px-6 flex space-x-2">
        <button
          onClick={() => setActiveTab('output')}
          className={`
            px-4 py-2 text-sm font-medium transition-colors flex items-center space-x-2
            ${
              activeTab === 'output'
                ? 'text-blue-400 border-b-2 border-blue-400'
                : 'text-gray-400 hover:text-gray-300'
            }
          `}
        >
          <Terminal className="w-4 h-4" />
          <span>Output</span>
        </button>

        <button
          onClick={() => setActiveTab('worktrees')}
          className={`
            px-4 py-2 text-sm font-medium transition-colors flex items-center space-x-2
            ${
              activeTab === 'worktrees'
                ? 'text-blue-400 border-b-2 border-blue-400'
                : 'text-gray-400 hover:text-gray-300'
            }
          `}
        >
          <Folder className="w-4 h-4" />
          <span>Worktrees</span>
        </button>
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-y-auto">
        {activeTab === 'output' && (
          /* Agent Cards Grid */
          <div className="p-6 space-y-6">
        {localInstances.map((instance) => {
          const isIntegration = isIntegrationInstance(instance);
          return (
          <div
            key={instance.id}
            className={`
              rounded-lg border-2 overflow-hidden transition-all duration-300 hover:shadow-2xl hover:scale-[1.01]
              ${isIntegration ? 'bg-purple-900/20 border-purple-500' : `bg-gray-800 ${getStatusColor(instance.status)}`}
            `}
          >
            {/* 🔥 NEW: Integration Badge */}
            {isIntegration && (
              <div className="bg-gradient-to-r from-purple-600 to-pink-600 px-4 py-2 flex items-center justify-center space-x-2">
                <Activity className="w-5 h-5 text-white animate-pulse" />
                <span className="text-sm font-bold text-white uppercase tracking-wider">
                  🔄 統合フェーズ - Integration Phase
                </span>
              </div>
            )}

            {/* Card Header */}
            <div className="p-6 border-b border-gray-700">
              <div className="flex items-start justify-between">
                {/* Left: Agent Info */}
                <div className="flex-1">
                  <div className="flex items-center space-x-3 mb-3">
                    {getStatusIcon(instance.status)}
                    <h3 className={`text-xl font-bold ${isIntegration ? 'text-purple-300' : 'text-white'}`}>
                      {instance.agentName || 'Unnamed Agent'}
                    </h3>
                    <span
                      className={`px-3 py-1 rounded-full text-xs font-semibold uppercase ${getStatusColor(
                        instance.status
                      )}`}
                    >
                      {instance.status}
                    </span>
                    {renderRuntimeBadge(instance)}
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
          );
        })}
          </div>
        )}

        {activeTab === 'worktrees' && competitionId && (
          <WorktreeExplorer competitionId={competitionId} />
        )}

        {activeTab === 'worktrees' && !competitionId && (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <Folder className="w-16 h-16 text-gray-600 mx-auto mb-4" />
              <h3 className="text-xl font-semibold text-gray-400 mb-2">
                No Competition ID
              </h3>
              <p className="text-gray-500 text-sm">
                Start a competition to view worktrees
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default MultiAgentPanel;
