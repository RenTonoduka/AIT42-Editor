/**
 * EnsembleDialog - Multi-runtime Ensemble Mode (Collaborative)
 */

import React, { useState, useEffect, useMemo } from 'react';
import { Sparkles, X, Settings as SettingsIcon, Cpu, Loader2 } from 'lucide-react';
import { tauriApi, MultiRuntimeCompetitionRequest } from '@/services/tauri';
import { ModeIndicator } from './ModeIndicator';
import { CollaborativeFlowDiagram } from './CollaborativeFlowDiagram';
import { ModeTooltip } from './ModeTooltip';
import { useTaskOptimizer } from '@/hooks/useTaskOptimizer';
import { RuntimeAllocation, AgentRuntime, WorktreeSession, WorktreeInstance } from '@/types/worktree';
import { RUNTIME_DEFINITIONS, getRuntimeDefinition } from '@/config/runtimes';
import { useSessionHistoryStore } from '@/store/sessionHistoryStore';

export interface EnsembleDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onStart?: (competitionId: string, allocations: RuntimeAllocation[], task: string) => void;
}

const DEFAULT_CLAUDE_INSTANCES = 3;

const buildDefaultAllocations = (): RuntimeAllocation[] =>
  RUNTIME_DEFINITIONS.map((def) => ({
    runtime: def.id,
    count: def.id === 'claude' ? DEFAULT_CLAUDE_INSTANCES : 0,
    model: def.defaultModel,
  }));

const clampRuntimeCount = (value: number) => {
  if (Number.isNaN(value)) return 0;
  return Math.max(0, Math.min(10, value));
};

const clampTimeout = (value: number) => {
  if (Number.isNaN(value)) return 300;
  return Math.max(60, Math.min(3600, value));
};

const ensureModel = (runtime: AgentRuntime, model?: string) => {
  if (model) return model;
  return getRuntimeDefinition(runtime).defaultModel;
};

export const EnsembleDialog: React.FC<EnsembleDialogProps> = ({ isOpen, onClose, onStart }) => {
  const [task, setTask] = useState('');
  const [runtimeAllocations, setRuntimeAllocations] = useState<RuntimeAllocation[]>(buildDefaultAllocations);
  const [timeoutSeconds, setTimeoutSeconds] = useState(300);
  const [preserveWorktrees, setPreserveWorktrees] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [isStarting, setIsStarting] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);

  const { state: optimizerState, analyze, isAnalyzing } = useTaskOptimizer();
  const { createSession, loadSessions } = useSessionHistoryStore();

  const totalInstances = useMemo(
    () => runtimeAllocations.reduce((sum, allocation) => sum + allocation.count, 0),
    [runtimeAllocations]
  );

  const activeAllocations = useMemo(
    () => runtimeAllocations.filter((allocation) => allocation.count > 0),
    [runtimeAllocations]
  );

  const runtimeSummary = activeAllocations
    .map((allocation) => {
      const def = getRuntimeDefinition(allocation.runtime);
      return `${def.label} x ${allocation.count}`;
    })
    .join(' / ');

  useEffect(() => {
    if (isOpen) {
      setTask('');
      setRuntimeAllocations(buildDefaultAllocations());
      setTimeoutSeconds(300);
      setPreserveWorktrees(false);
      setShowAdvanced(false);
      setIsStarting(false);
      setValidationError(null);
    }
  }, [isOpen]);

  useEffect(() => {
    if (optimizerState.status === 'calculated' && optimizerState.instances) {
      setRuntimeAllocations((prev) =>
        prev.map((allocation) =>
          allocation.runtime === 'claude'
            ? { ...allocation, count: optimizerState.instances?.recommendedInstances ?? allocation.count }
            : allocation
        )
      );
    }
  }, [optimizerState]);

  useEffect(() => {
    if (!task.trim() || task.trim().length < 10) {
      return;
    }

    const debounceTimer = setTimeout(() => {
      analyze(task.trim());
    }, 1500);

    return () => clearTimeout(debounceTimer);
  }, [task, analyze]);

  const updateAllocation = (runtime: AgentRuntime, updates: Partial<RuntimeAllocation>) => {
    setRuntimeAllocations((prev) =>
      prev.map((allocation) => (allocation.runtime === runtime ? { ...allocation, ...updates } : allocation))
    );
  };

  const handleStart = async () => {
    if (!task.trim()) {
      setValidationError('タスクを入力してください');
      return;
    }

    if (totalInstances < 2 || totalInstances > 10) {
      setValidationError('合計インスタンス数は2〜10の範囲で指定してください');
      return;
    }

    if (activeAllocations.length === 0) {
      setValidationError('少なくとも1つのランタイムにインスタンスを割り当ててください');
      return;
    }

    setValidationError(null);

    try {
      const workspace = await tauriApi.getWorkspace();
      if (!workspace.is_git_repo) {
        setValidationError(
          `現在のワークスペースはGitリポジトリではありません。右上の「フォルダを開く」ボタンからGitリポジトリを選択してください。`
        );
        return;
      }
    } catch (error) {
      setValidationError(`ワークスペースの確認に失敗しました: ${error}`);
      return;
    }

    setIsStarting(true);
    try {
      const request: MultiRuntimeCompetitionRequest = {
        task: task.trim(),
        allocations: activeAllocations.map((allocation) => ({
          runtime: allocation.runtime,
          count: allocation.count,
          model: ensureModel(allocation.runtime, allocation.model),
        })),
        timeoutSeconds,
        preserveWorktrees,
        mode: 'ensemble',
      };

      const result = await tauriApi.executeMultiRuntimeCompetition(request);

      console.log('Ensemble started:', result);

      // 🔥 Create session data and save to history
      const runtimeMix = result.instances.map((inst) => inst.runtime!).filter(Boolean);
      const sessionInstances: WorktreeInstance[] = result.instances.map((inst) => ({
        instanceId: inst.instanceNumber,
        worktreePath: inst.worktreePath,
        branch: `${inst.runtime}-ensemble-${inst.instanceNumber}`,
        agentName: inst.runtime || 'unknown',
        status: 'running',
        tmuxSessionId: inst.tmuxSessionId,
        runtime: inst.runtime,
        model: inst.model,
        runtimeLabel: inst.runtime ? getRuntimeDefinition(inst.runtime).label : undefined,
        startTime: inst.startedAt,
      }));

      const session: WorktreeSession = {
        id: result.competitionId,
        type: 'ensemble',
        task: task.trim(),
        status: 'running',
        createdAt: result.startedAt,
        updatedAt: result.startedAt,
        instances: sessionInstances,
        chatHistory: [],
        timeoutSeconds,
        preserveWorktrees,
        runtimeMix: runtimeMix as any,
      };

      // Save session to database
      await createSession(session);

      // Reload sessions to update UI
      await loadSessions();

      if (onStart) {
        onStart(result.competitionId, activeAllocations, task.trim());
      }
    } catch (error) {
      console.error('Failed to start ensemble:', error);
      setValidationError(`アンサンブルの開始に失敗しました: ${error}`);
    } finally {
      setIsStarting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" onClick={onClose}>
      <div
        className="w-full max-w-3xl max-h-[85vh] bg-editor-elevated border border-editor-border rounded-xl shadow-2xl overflow-hidden flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-3 px-6 py-4 border-b border-editor-border bg-editor-surface">
          <Sparkles size={24} className="text-purple-600" />
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-1">
              <h2 className="text-lg font-semibold text-text-primary">Claude Code アンサンブル</h2>
              <ModeIndicator mode="collaborative" />
              <ModeTooltip mode="collaborative" />
            </div>
            <p className="text-xs text-text-tertiary">複数ランタイムの出力を統合して最終案を生成</p>
          </div>
          <button onClick={onClose} className="p-1 hover:bg-editor-border/30 rounded transition-colors" title="閉じる (Esc)">
            <X size={20} className="text-text-tertiary" />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          <CollaborativeFlowDiagram />

          <div>
            <label className="block text-sm font-medium text-text-primary mb-2">タスク説明</label>
            <textarea
              value={task}
              onChange={(e) => {
                setTask(e.target.value);
                if (validationError) {
                  setValidationError(null);
                }
              }}
              placeholder="各ランタイムに実行させるタスクを入力してください..."
              className="w-full px-4 py-3 bg-editor-bg text-text-primary placeholder-text-tertiary border border-editor-border rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-600/50 resize-none"
              rows={4}
            />
            {validationError && <div className="text-sm text-red-400 mt-2 px-2">{validationError}</div>}
          </div>

          {isAnalyzing && (
            <div className="flex flex-col gap-2 px-4 py-3 bg-purple-900/20 border border-purple-700/30 rounded-lg">
              <div className="flex items-center gap-3">
                <Loader2 size={16} className="animate-spin text-purple-400" />
                <span className="text-sm text-purple-300">Claude Codeがタスクを分析中...</span>
              </div>
              <span className="text-xs text-purple-400/70">分析完了を待たずに開始することもできます</span>
            </div>
          )}

          {optimizerState.status === 'calculated' && optimizerState.optimization && (
            <div className="px-4 py-3 bg-green-900/20 border border-green-700/30 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Sparkles size={16} className="text-green-400" />
                <span className="text-sm font-semibold text-green-300">
                  分析完了: {optimizerState.optimization.complexityClass} 複雑度
                </span>
              </div>
              <div className="text-xs text-green-400/80">
                推奨インスタンス数: <span className="font-bold">{optimizerState.instances?.recommendedInstances}</span>
              </div>
            </div>
          )}

          {/* Runtime Allocation */}
          <div>
            <div className="flex items-center justify-between mb-3">
              <label className="text-sm font-medium text-text-primary">
                <Cpu size={16} className="inline mr-2" />ランタイム配分
              </label>
              <span className="text-xs text-text-tertiary">合計 {totalInstances} インスタンス（2〜10）</span>
            </div>
            <div className="space-y-3">
              {runtimeAllocations.map((allocation) => {
                const def = getRuntimeDefinition(allocation.runtime);
                const isActive = allocation.count > 0;

                return (
                  <div
                    key={allocation.runtime}
                    className={`p-4 border rounded-lg bg-editor-surface ${
                      isActive ? 'border-purple-500/60 shadow-glow-sm' : 'border-editor-border'
                    }`}
                  >
                    <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                      <div className="flex items-center gap-3">
                        <span className="text-2xl" aria-hidden>
                          {def.emoji}
                        </span>
                        <div>
                          <div className="font-semibold text-text-primary">{def.label}</div>
                          <div className="text-xs text-text-tertiary">{def.description}</div>
                        </div>
                      </div>
                      <div className="flex flex-col sm:flex-row sm:items-center gap-2 min-w-[220px]">
                        <input
                          type="number"
                          min={0}
                          max={10}
                          value={allocation.count}
                          onChange={(e) =>
                            updateAllocation(allocation.runtime, { count: clampRuntimeCount(parseInt(e.target.value, 10)) })
                          }
                          className="w-full sm:w-20 px-3 py-2 bg-editor-bg text-text-primary border border-editor-border rounded focus:outline-none focus:ring-2 focus:ring-purple-600/50"
                        />
                        <select
                          value={allocation.model ?? def.defaultModel}
                          onChange={(e) => updateAllocation(allocation.runtime, { model: e.target.value })}
                          disabled={allocation.count === 0}
                          className="flex-1 px-3 py-2 bg-editor-bg text-text-primary border border-editor-border rounded focus:outline-none focus:ring-2 focus:ring-purple-600/50 disabled:opacity-50"
                        >
                          {def.modelOptions.map((model) => (
                            <option key={model} value={model}>
                              {model}
                            </option>
                          ))}
                        </select>
                      </div>
                    </div>
                    <div className="text-xs text-text-tertiary mt-2">
                      必要なAPIキー: <code className="font-mono text-text-secondary">{def.envVar}</code>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Advanced Options */}
          <div>
            <button
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="flex items-center gap-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
            >
              <SettingsIcon size={16} />
              詳細設定を{showAdvanced ? '非表示' : '表示'}
            </button>

            {showAdvanced && (
              <div className="mt-4 p-4 bg-editor-bg rounded-lg border border-editor-border space-y-4">
                <div>
                  <label className="block text-xs font-medium text-text-secondary mb-2">
                    タイムアウト（各インスタンスの制限時間）
                  </label>
                  <div className="flex items-center gap-3">
                    <input
                      type="number"
                      min="60"
                      max="3600"
                      step="60"
                      value={timeoutSeconds}
                      onChange={(e) => setTimeoutSeconds(clampTimeout(parseInt(e.target.value, 10)))}
                      className="flex-1 px-3 py-2 bg-editor-surface text-text-primary border border-editor-border rounded focus:outline-none focus:ring-2 focus:ring-purple-600/50"
                    />
                    <span className="text-sm text-text-tertiary">= {Math.floor(timeoutSeconds / 60)} 分</span>
                  </div>
                </div>

                <div className="flex items-start gap-3">
                  <input
                    type="checkbox"
                    checked={preserveWorktrees}
                    onChange={(e) => setPreserveWorktrees(e.target.checked)}
                    className="mt-1"
                  />
                  <div className="flex-1">
                    <label className="text-xs font-medium text-text-secondary">完了後もworktreeを保持</label>
                    <div className="text-xs text-text-tertiary mt-1">
                      アンサンブル完了後もGit worktreeと出力を保持し、後で検証できます
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>

        <div className="flex items-center justify-between px-6 py-4 border-t border-editor-border bg-editor-surface">
          <div className="text-sm text-text-tertiary">
            {!task.trim() ? 'タスクを入力してください' : runtimeSummary || 'ランタイムを割り当ててください'}
          </div>
          <div className="flex gap-3">
            <button onClick={onClose} className="px-4 py-2 text-text-secondary hover:text-text-primary transition-colors">
              キャンセル
            </button>
            <button
              onClick={handleStart}
              disabled={!task.trim() || isStarting}
              className="px-6 py-2 bg-gradient-to-r from-purple-500 to-purple-700 hover:from-purple-600 hover:to-purple-800 disabled:from-editor-border disabled:to-editor-border disabled:text-text-tertiary text-white font-semibold rounded-lg transition-all shadow-glow-sm hover:shadow-glow-md"
            >
              {isStarting ? '起動中...' : '🤝 アンサンブル開始'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
