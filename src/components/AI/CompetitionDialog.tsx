/**
 * CompetitionDialog - Claude Code Competition Mode
 *
 * Launches multiple Claude Code instances in parallel using Git worktrees
 * and compares their results for the same task.
 */

import React, { useState, useEffect } from 'react';
import { Trophy, X, Settings as SettingsIcon, Code2, Cpu } from 'lucide-react';
import { tauriApi, ClaudeCodeCompetitionRequest } from '@/services/tauri';
import { ModeIndicator } from './ModeIndicator';
import { CompetitiveFlowDiagram } from './CompetitiveFlowDiagram';
import { ModeTooltip } from './ModeTooltip';

export interface CompetitionDialogProps {
  /** Whether the dialog is visible */
  isOpen: boolean;
  /** Callback when dialog should close */
  onClose: () => void;
  /** Callback when competition starts */
  onStart?: (competitionId: string, instanceCount: number, task: string) => void;
}

type ClaudeModel = 'sonnet' | 'haiku' | 'opus';

const MODEL_INFO: Record<ClaudeModel, { label: string; description: string; emoji: string }> = {
  sonnet: {
    label: 'Sonnet 4.5',
    description: 'バランス型：速度と品質の最適バランス',
    emoji: '⚡',
  },
  haiku: {
    label: 'Haiku 3.5',
    description: '高速型：最速の応答速度',
    emoji: '🚀',
  },
  opus: {
    label: 'Opus 4',
    description: '高品質型：最高の出力品質',
    emoji: '💎',
  },
};

/**
 * CompetitionDialog component
 */
export const CompetitionDialog: React.FC<CompetitionDialogProps> = ({
  isOpen,
  onClose,
  onStart,
}) => {
  const [task, setTask] = useState('');
  const [instanceCount, setInstanceCount] = useState(3);
  const [selectedModel, setSelectedModel] = useState<ClaudeModel>('sonnet');
  const [timeoutSeconds, setTimeoutSeconds] = useState(300);
  const [preserveWorktrees, setPreserveWorktrees] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [isStarting, setIsStarting] = useState(false);

  // Reset state when dialog opens
  useEffect(() => {
    if (isOpen) {
      setTask('');
      setInstanceCount(3);
      setSelectedModel('sonnet');
      setTimeoutSeconds(300);
      setPreserveWorktrees(false);
      setShowAdvanced(false);
      setIsStarting(false);
    }
  }, [isOpen]);

  const handleStart = async () => {
    if (!task.trim()) {
      alert('タスクを入力してください');
      return;
    }

    setIsStarting(true);
    try {
      const request: ClaudeCodeCompetitionRequest = {
        task: task.trim(),
        instanceCount,
        model: selectedModel,
        timeoutSeconds,
        preserveWorktrees,
      };

      const result = await tauriApi.executeClaudeCodeCompetition(request);

      console.log('Competition started:', result);

      if (onStart) {
        // タスクとインスタンス数も渡す
        onStart(result.competitionId, instanceCount, task.trim());
      }

      // 成功後にリセット（onStart内でダイアログが閉じられる）
      setIsStarting(false);
    } catch (error) {
      console.error('Failed to start competition:', error);
      alert(`コンペティションの開始に失敗しました: ${error}`);
      setIsStarting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="w-full max-w-3xl max-h-[85vh] bg-editor-elevated border border-editor-border rounded-xl shadow-2xl overflow-hidden flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center gap-3 px-6 py-4 border-b border-editor-border bg-editor-surface">
          <Trophy size={24} className="text-accent-primary" />
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-1">
              <h2 className="text-lg font-semibold text-text-primary">
                Claude Code コンペティション
              </h2>
              <ModeIndicator mode="competitive" />
              <ModeTooltip mode="competitive" />
            </div>
            <p className="text-xs text-text-tertiary">
              複数のClaude Codeインスタンスを並列実行して結果を比較
            </p>
          </div>
          <button
            onClick={onClose}
            className="p-1 hover:bg-editor-border/30 rounded transition-colors"
            title="閉じる (Esc)"
          >
            <X size={20} className="text-text-tertiary" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          {/* Flow Diagram */}
          <CompetitiveFlowDiagram />

          {/* Task Input */}
          <div>
            <label className="block text-sm font-medium text-text-primary mb-2">
              タスク説明
            </label>
            <textarea
              value={task}
              onChange={(e) => setTask(e.target.value)}
              placeholder="各インスタンスに実行させるタスクを入力してください...&#10;例: 'ユーザー認証機能をJWTで実装してください'"
              className="w-full px-4 py-3 bg-editor-bg text-text-primary placeholder-text-tertiary border border-editor-border rounded-lg focus:outline-none focus:ring-2 focus:ring-accent-primary/50 resize-none"
              rows={4}
            />
          </div>

          {/* Instance Count Slider */}
          <div>
            <div className="flex items-center justify-between mb-3">
              <label className="text-sm font-medium text-text-primary">
                <Cpu size={16} className="inline mr-2" />
                インスタンス数
              </label>
              <span className="text-2xl font-bold text-accent-primary">{instanceCount}</span>
            </div>
            <input
              type="range"
              min="2"
              max="10"
              value={instanceCount}
              onChange={(e) => {
                const value = parseInt(e.target.value, 10);
                setInstanceCount(isNaN(value) ? 3 : value);
              }}
              className="w-full h-2 bg-editor-border rounded-lg appearance-none cursor-pointer accent-accent-primary"
            />
            <div className="flex justify-between text-xs text-text-tertiary mt-1">
              <span>2</span>
              <span>10</span>
            </div>
          </div>

          {/* Model Selection */}
          <div>
            <label className="block text-sm font-medium text-text-primary mb-3">
              <Code2 size={16} className="inline mr-2" />
              Claude モデル
            </label>
            <div className="grid grid-cols-3 gap-3">
              {(Object.keys(MODEL_INFO) as ClaudeModel[]).map((model) => {
                const info = MODEL_INFO[model];
                const isSelected = selectedModel === model;

                return (
                  <button
                    key={model}
                    onClick={() => setSelectedModel(model)}
                    className={`p-4 rounded-lg border-2 transition-all text-left ${
                      isSelected
                        ? 'border-accent-primary bg-accent-primary/10 shadow-glow-sm'
                        : 'border-editor-border hover:border-editor-border/60 bg-editor-surface'
                    }`}
                  >
                    <div className="text-2xl mb-1">{info.emoji}</div>
                    <div className="font-semibold text-sm text-text-primary mb-1">
                      {info.label}
                    </div>
                    <div className="text-xs text-text-tertiary leading-tight">
                      {info.description}
                    </div>
                  </button>
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
                {/* Timeout */}
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
                      onChange={(e) => {
                        const value = parseInt(e.target.value, 10);
                        setTimeoutSeconds(isNaN(value) ? 300 : value);
                      }}
                      className="flex-1 px-3 py-2 bg-editor-surface text-text-primary border border-editor-border rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/50"
                    />
                    <span className="text-sm text-text-tertiary">
                      = {Math.floor(timeoutSeconds / 60)} 分
                    </span>
                  </div>
                </div>

                {/* Preserve Worktrees */}
                <div className="flex items-start gap-3">
                  <input
                    type="checkbox"
                    checked={preserveWorktrees}
                    onChange={(e) => setPreserveWorktrees(e.target.checked)}
                    className="mt-1"
                  />
                  <div className="flex-1">
                    <label className="text-xs font-medium text-text-secondary">
                      完了後もworktreeを保持
                    </label>
                    <div className="text-xs text-text-tertiary mt-1">
                      コンペティション完了後もGit worktreeと出力を保持し、後で確認できるようにします
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-6 py-4 border-t border-editor-border bg-editor-surface">
          <div className="text-sm text-text-tertiary">
            {!task.trim() ? (
              'タスクを入力してください'
            ) : (
              <>
                {instanceCount} 個のインスタンス × {MODEL_INFO[selectedModel].label} で実行準備完了
              </>
            )}
          </div>
          <div className="flex gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 text-text-secondary hover:text-text-primary transition-colors"
            >
              キャンセル
            </button>
            <button
              onClick={handleStart}
              disabled={!task.trim() || isStarting}
              className="px-6 py-2 bg-gradient-to-r from-accent-primary to-accent-secondary hover:from-accent-secondary hover:to-accent-primary disabled:from-editor-border disabled:to-editor-border disabled:text-text-tertiary text-white font-semibold rounded-lg transition-all shadow-glow-sm hover:shadow-glow-md"
            >
              {isStarting ? '起動中...' : '🏆 コンペティション開始'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
