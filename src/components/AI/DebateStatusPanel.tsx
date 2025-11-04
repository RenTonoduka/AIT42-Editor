import React, { useState, useEffect } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import {
  MessageSquare,
  Users,
  Clock,
  CheckCircle2,
  XCircle,
  Loader2,
  ChevronDown,
  ChevronUp,
  FileText,
  GitBranch,
  Calendar,
} from 'lucide-react';
import { tauriApi, DebateStatus, RoundOutput } from '@/services/tauri';

export interface DebateStatusPanelProps {
  debateId: string;
  task: string;
  onClose?: () => void;
}

const DebateStatusPanel: React.FC<DebateStatusPanelProps> = ({
  debateId,
  task,
  onClose,
}) => {
  const [status, setStatus] = useState<DebateStatus | null>(null);
  const [expandedRounds, setExpandedRounds] = useState<Set<number>>(new Set([1])); // Default expand round 1
  const [isPolling, setIsPolling] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Toggle round expansion
  const toggleRound = (round: number) => {
    const newExpanded = new Set(expandedRounds);
    if (newExpanded.has(round)) {
      newExpanded.delete(round);
    } else {
      newExpanded.add(round);
    }
    setExpandedRounds(newExpanded);
  };

  // Fetch debate status
  const fetchStatus = async () => {
    try {
      const debateStatus = await tauriApi.getDebateStatus(debateId);
      setStatus(debateStatus);
      setError(null);

      // Stop polling if debate is completed or failed
      if (debateStatus.status === 'completed' || debateStatus.status === 'failed') {
        setIsPolling(false);
      }

      // Auto-expand current round
      if (debateStatus.currentRound) {
        setExpandedRounds((prev) => new Set(prev).add(debateStatus.currentRound));
      }
    } catch (err) {
      console.error('Failed to fetch debate status:', err);
      setError(`ステータス取得エラー: ${err}`);
    }
  };

  // Poll for status updates
  useEffect(() => {
    if (!isPolling) return;

    fetchStatus(); // Initial fetch

    const interval = setInterval(() => {
      fetchStatus();
    }, 2000); // Poll every 2 seconds

    return () => clearInterval(interval);
  }, [debateId, isPolling]);

  // Listen to Tauri events for real-time updates
  useEffect(() => {
    let unlistenStatus: UnlistenFn | null = null;
    let unlistenRound: UnlistenFn | null = null;

    const setupListeners = async () => {
      try {
        // Listen for debate status updates
        unlistenStatus = await listen<DebateStatus>('debate-status', (event) => {
          if (event.payload.debateId === debateId) {
            setStatus(event.payload);

            // Stop polling if completed or failed
            if (event.payload.status === 'completed' || event.payload.status === 'failed') {
              setIsPolling(false);
            }
          }
        });

        // Listen for round output updates
        unlistenRound = await listen<RoundOutput>('debate-round-output', (event) => {
          setStatus((prevStatus) => {
            if (!prevStatus || prevStatus.debateId !== debateId) return prevStatus;

            // Update roundOutputs with new output
            const updatedOutputs = [...prevStatus.roundOutputs];
            const existingIndex = updatedOutputs.findIndex(
              (o) => o.round === event.payload.round && o.roleId === event.payload.roleId
            );

            if (existingIndex >= 0) {
              updatedOutputs[existingIndex] = event.payload;
            } else {
              updatedOutputs.push(event.payload);
            }

            return {
              ...prevStatus,
              roundOutputs: updatedOutputs,
            };
          });

          // Auto-expand the round receiving output
          setExpandedRounds((prev) => new Set(prev).add(event.payload.round));
        });
      } catch (err) {
        console.error('Failed to setup event listeners:', err);
      }
    };

    setupListeners();

    return () => {
      if (unlistenStatus) unlistenStatus();
      if (unlistenRound) unlistenRound();
    };
  }, [debateId]);

  // Status color mapping
  const getStatusColor = (statusStr: string) => {
    switch (statusStr) {
      case 'running':
      case 'round_1':
      case 'round_2':
      case 'round_3':
        return 'text-blue-400 bg-blue-500/10 border-blue-500/30';
      case 'completed':
        return 'text-green-400 bg-green-500/10 border-green-500/30';
      case 'failed':
        return 'text-red-400 bg-red-500/10 border-red-500/30';
      default:
        return 'text-gray-400 bg-gray-500/10 border-gray-500/30';
    }
  };

  const getStatusIcon = (statusStr: string) => {
    switch (statusStr) {
      case 'running':
      case 'round_1':
      case 'round_2':
      case 'round_3':
        return <Loader2 className="w-5 h-5 animate-spin" />;
      case 'completed':
        return <CheckCircle2 className="w-5 h-5" />;
      case 'failed':
        return <XCircle className="w-5 h-5" />;
      default:
        return <Clock className="w-5 h-5" />;
    }
  };

  const getRoleOutputColor = (roleOutputStatus: string) => {
    switch (roleOutputStatus) {
      case 'running':
        return 'border-l-blue-500';
      case 'completed':
        return 'border-l-green-500';
      case 'failed':
        return 'border-l-red-500';
      default:
        return 'border-l-gray-500';
    }
  };

  // Format duration
  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}min`;
  };

  // Calculate total duration
  const getTotalDuration = () => {
    if (!status) return null;
    if (!status.startedAt) return null;

    const endTime = status.completedAt ? new Date(status.completedAt) : new Date();
    const startTime = new Date(status.startedAt);
    const duration = endTime.getTime() - startTime.getTime();

    return formatDuration(duration);
  };

  // Get round outputs grouped by round
  const getRoundOutputs = (round: number): RoundOutput[] => {
    if (!status) return [];
    return status.roundOutputs.filter((o) => o.round === round).sort((a, b) => {
      // Sort by completion time or start time
      const timeA = a.completedAt || a.startedAt;
      const timeB = b.completedAt || b.startedAt;
      return new Date(timeA).getTime() - new Date(timeB).getTime();
    });
  };

  // Loading state
  if (!status && !error) {
    return (
      <div className="flex items-center justify-center h-full bg-gray-900">
        <div className="text-center">
          <Loader2 className="w-16 h-16 text-blue-400 mx-auto mb-4 animate-spin" />
          <p className="text-gray-400">Debate ステータスを読み込み中...</p>
        </div>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className="flex items-center justify-center h-full bg-gray-900">
        <div className="text-center">
          <XCircle className="w-16 h-16 text-red-400 mx-auto mb-4" />
          <p className="text-red-400 mb-2">エラーが発生しました</p>
          <p className="text-gray-500 text-sm">{error}</p>
          {onClose && (
            <button
              onClick={onClose}
              className="mt-4 px-4 py-2 bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-lg transition-colors"
            >
              閉じる
            </button>
          )}
        </div>
      </div>
    );
  }

  if (!status) return null;

  return (
    <div className="h-full bg-gray-900 flex flex-col overflow-hidden">
      {/* Header */}
      <div className="bg-gray-800 border-b border-gray-700 p-4 flex items-center justify-between">
        <div className="flex items-center space-x-3">
          <MessageSquare className="w-6 h-6 text-purple-400" />
          <div>
            <h2 className="text-xl font-bold text-white">Debate Mode</h2>
            <p className="text-sm text-gray-400 mt-1">{task}</p>
          </div>
        </div>
        <div className={`flex items-center space-x-2 px-3 py-1.5 rounded-lg border ${getStatusColor(status.status)}`}>
          {getStatusIcon(status.status)}
          <span className="font-medium capitalize">{status.status.replace('_', ' ')}</span>
        </div>
      </div>

      {/* Debate Info */}
      <div className="bg-gray-800/50 border-b border-gray-700 p-4">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="flex items-center space-x-2">
            <Users className="w-4 h-4 text-gray-400" />
            <div>
              <p className="text-xs text-gray-500">ラウンド</p>
              <p className="text-sm font-medium text-white">
                {status.currentRound} / {status.totalRounds}
              </p>
            </div>
          </div>
          <div className="flex items-center space-x-2">
            <Clock className="w-4 h-4 text-gray-400" />
            <div>
              <p className="text-xs text-gray-500">経過時間</p>
              <p className="text-sm font-medium text-white">{getTotalDuration() || '-'}</p>
            </div>
          </div>
          <div className="flex items-center space-x-2">
            <GitBranch className="w-4 h-4 text-gray-400" />
            <div>
              <p className="text-xs text-gray-500">Worktree</p>
              <p className="text-sm font-medium text-white truncate" title={status.worktreePath}>
                {status.worktreePath.split('/').pop() || '-'}
              </p>
            </div>
          </div>
          <div className="flex items-center space-x-2">
            <FileText className="w-4 h-4 text-gray-400" />
            <div>
              <p className="text-xs text-gray-500">コンテキスト</p>
              <p className="text-sm font-medium text-white">{status.contextFiles.length} files</p>
            </div>
          </div>
        </div>
      </div>

      {/* Round Progress */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {[1, 2, 3].map((roundNum) => {
          const roundOutputs = getRoundOutputs(roundNum);
          const isExpanded = expandedRounds.has(roundNum);
          const isCurrent = status.currentRound === roundNum;
          const isPast = status.currentRound > roundNum;
          const isFuture = status.currentRound < roundNum;

          return (
            <div
              key={roundNum}
              className={`bg-gray-800 rounded-lg border ${
                isCurrent ? 'border-blue-500/50' : isPast ? 'border-green-500/30' : 'border-gray-700'
              }`}
            >
              {/* Round Header */}
              <button
                onClick={() => toggleRound(roundNum)}
                className="w-full p-4 flex items-center justify-between hover:bg-gray-700/50 transition-colors rounded-t-lg"
              >
                <div className="flex items-center space-x-3">
                  <div
                    className={`w-10 h-10 rounded-full flex items-center justify-center font-bold ${
                      isCurrent
                        ? 'bg-blue-500/20 text-blue-400 border-2 border-blue-500'
                        : isPast
                        ? 'bg-green-500/20 text-green-400'
                        : 'bg-gray-700 text-gray-500'
                    }`}
                  >
                    {isPast ? <CheckCircle2 className="w-5 h-5" /> : roundNum}
                  </div>
                  <div className="text-left">
                    <h3 className="text-lg font-semibold text-white">
                      Round {roundNum}
                      {roundNum === 1 && ' - 独立提案'}
                      {roundNum === 2 && ' - 批判的分析'}
                      {roundNum === 3 && ' - コンセンサス形成'}
                    </h3>
                    <p className="text-sm text-gray-400">
                      {roundOutputs.length} / 3 roles{' '}
                      {isFuture ? '(未実行)' : isCurrent ? '(実行中)' : '(完了)'}
                    </p>
                  </div>
                </div>
                {isExpanded ? (
                  <ChevronUp className="w-5 h-5 text-gray-400" />
                ) : (
                  <ChevronDown className="w-5 h-5 text-gray-400" />
                )}
              </button>

              {/* Round Outputs */}
              {isExpanded && (
                <div className="border-t border-gray-700 p-4 space-y-3">
                  {roundOutputs.length === 0 ? (
                    <div className="text-center py-8 text-gray-500">
                      {isFuture ? 'このラウンドはまだ実行されていません' : 'ロール実行待機中...'}
                    </div>
                  ) : (
                    roundOutputs.map((output, idx) => (
                      <div
                        key={`${output.roleId}-${idx}`}
                        className={`bg-gray-900 rounded-lg p-4 border-l-4 ${getRoleOutputColor(output.status)}`}
                      >
                        {/* Role Header */}
                        <div className="flex items-center justify-between mb-2">
                          <div className="flex items-center space-x-2">
                            <Users className="w-4 h-4 text-gray-400" />
                            <span className="font-medium text-white">{output.roleName}</span>
                            <span className="text-xs text-gray-500">({output.roleId})</span>
                          </div>
                          <div className="flex items-center space-x-3 text-xs text-gray-400">
                            <div className="flex items-center space-x-1">
                              <Clock className="w-3 h-3" />
                              <span>{formatDuration(output.executionTimeMs)}</span>
                            </div>
                            {output.status === 'running' && (
                              <Loader2 className="w-4 h-4 text-blue-400 animate-spin" />
                            )}
                            {output.status === 'completed' && (
                              <CheckCircle2 className="w-4 h-4 text-green-400" />
                            )}
                            {output.status === 'failed' && <XCircle className="w-4 h-4 text-red-400" />}
                          </div>
                        </div>

                        {/* Output Content */}
                        <div className="bg-black/30 rounded p-3 mt-2">
                          <pre className="text-xs text-gray-300 whitespace-pre-wrap font-mono max-h-60 overflow-y-auto">
                            {output.output || '出力待機中...'}
                          </pre>
                        </div>

                        {/* Timestamps */}
                        <div className="flex items-center justify-between mt-2 text-xs text-gray-500">
                          <div className="flex items-center space-x-1">
                            <Calendar className="w-3 h-3" />
                            <span>開始: {new Date(output.startedAt).toLocaleTimeString('ja-JP')}</span>
                          </div>
                          {output.completedAt && (
                            <span>完了: {new Date(output.completedAt).toLocaleTimeString('ja-JP')}</span>
                          )}
                        </div>
                      </div>
                    ))
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Footer with actions */}
      {(status.status === 'completed' || status.status === 'failed') && onClose && (
        <div className="bg-gray-800 border-t border-gray-700 p-4 flex justify-end space-x-3">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg transition-colors"
          >
            閉じる
          </button>
        </div>
      )}
    </div>
  );
};

export default DebateStatusPanel;
