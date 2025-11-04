# ディベートモード Phase 1 - React Frontend実装仕様書

**Version**: 1.0.0
**Date**: 2025-11-04
**Author**: Claude Code (Frontend Specialist)

## 目次

1. [コンポーネント構成](#1-コンポーネント構成)
2. [型定義](#2-型定義)
3. [Tauri API拡張](#3-tauri-api拡張)
4. [State管理](#4-state管理)
5. [DebateDialog実装](#5-debatedialog実装)
6. [RoleSelector実装](#6-roleselector実装)
7. [DebateStatusPanel実装](#7-debatestatuspanel実装)
8. [DebateFlowDiagram実装](#8-debateflowdiagram実装)
9. [UIモックアップ](#9-uiモックアップ)
10. [レスポンシブ対応](#10-レスポンシブ対応)
11. [アクセシビリティ](#11-アクセシビリティ)
12. [パフォーマンス最適化](#12-パフォーマンス最適化)
13. [エラーハンドリング](#13-エラーハンドリング)
14. [テスト戦略](#14-テスト戦略)
15. [実装チェックリスト](#15-実装チェックリスト)

---

## 1. コンポーネント構成

```
src/
├── components/
│   └── AI/
│       ├── DebateDialog.tsx           # メインダイアログ (600行)
│       ├── DebateStatusPanel.tsx      # 進捗表示パネル (250行)
│       ├── RoleSelector.tsx           # ロール選択UI (200行)
│       ├── DebateFlowDiagram.tsx      # フロー図 (150行)
│       ├── ModeIndicator.tsx          # 既存 (流用)
│       └── ModeTooltip.tsx            # 既存 (流用)
├── services/
│   └── tauri.ts                       # Tauri API (150行追加)
├── store/
│   └── debateStore.ts                 # State管理 (350行)
└── types/
    └── debate.ts                      # 型定義 (250行)
```

**推定LOC**: 1,350行 (新規実装のみ)
**推定工数**: 2週間 (1名)

---

## 2. 型定義

### src/types/debate.ts

```typescript
/**
 * Debate Mode Type Definitions
 *
 * Phase 1: 3-round structured debate system
 */

export type ClaudeModel = 'sonnet' | 'haiku' | 'opus';

/**
 * Debate execution request (Frontend → Rust)
 */
export interface DebateRequest {
  /** タスク説明 */
  task: string;

  /** ロール定義（3つ固定） */
  roles: RoleDefinition[];

  /** Claude モデル選択 */
  model: ClaudeModel;

  /** 全体タイムアウト（デフォルト: 2400秒 = 40分） */
  timeoutSeconds: number;

  /** 完了後もworktreeを保持 */
  preserveWorktrees: boolean;
}

/**
 * ロール定義
 */
export interface RoleDefinition {
  /** ロールID（internal識別子） */
  id: string;

  /** ロール表示名 */
  name: string;

  /** システムプロンプト（ロールの振る舞い定義） */
  systemPrompt: string;

  /** Lucide icon name */
  icon: string;
}

/**
 * ディベート実行結果 (Rust → Frontend)
 */
export interface DebateResult {
  /** ディベートID（UUID） */
  debateId: string;

  /** 初期ステータス */
  status: DebateStatus;

  /** 開始メッセージ */
  message: string;
}

/**
 * ディベートステータス（Tagged Union）
 */
export type DebateStatus =
  | { type: 'started' }
  | { type: 'round1InProgress'; currentRole: string }
  | { type: 'round2InProgress'; currentRole: string }
  | { type: 'round3InProgress'; currentRole: string }
  | { type: 'completed'; finalResult: string; completionTimeMs: number }
  | { type: 'failed'; error: string; failedAtRound: number };

/**
 * ラウンド出力（各ロールの結果）
 */
export interface RoundOutput {
  /** ロールID */
  roleId: string;

  /** ロール名 */
  roleName: string;

  /** 提案内容（Markdown） */
  proposal: string;

  /** 実行タイムスタンプ */
  timestamp: string;

  /** 実行時間（秒） */
  durationSecs: number;

  /** 使用トークン数 */
  tokensUsed: number;

  /** 使用モデル */
  model: string;
}

/**
 * ディベート詳細ステータス（ポーリング取得）
 */
export interface DebateDetailedStatus {
  debateId: string;
  status: DebateStatus;
  currentRound: number;
  totalRounds: number;
  roundOutputs: Map<number, RoundOutput[]>; // roundNumber -> outputs
  startedAt: string;
  updatedAt: string;
  completedAt?: string;
}

/**
 * プリセットロール定義
 */
export const ROLE_PRESETS: Record<string, PresetDefinition> = {
  'tech-stack': {
    id: 'tech-stack',
    label: '技術スタック選定',
    description: '最適な技術選択を多角的に検討',
    emoji: '🏗️',
    roles: [
      {
        id: 'architect',
        name: 'Technical Architect',
        systemPrompt: `あなたは経験豊富なテクニカルアーキテクトです。
長期的視点、保守性、スケーラビリティ、技術的負債回避を最優先に考えます。
5年後を見据えた設計判断を行い、堅牢性とエンタープライズレベルの品質を重視します。`,
        icon: 'Building2',
      },
      {
        id: 'pragmatist',
        name: 'Pragmatist',
        systemPrompt: `あなたは実践的なプラグマティストです。
実装可能性、現実的な制約（予算・納期・チームスキル）、ROI、技術選定リスクを重視します。
理想論ではなく「実際に動くもの」を最短で届けることを優先します。`,
        icon: 'Wrench',
      },
      {
        id: 'innovator',
        name: 'Innovator',
        systemPrompt: `あなたは革新的なイノベーターです。
最新技術、革新的アプローチ、パフォーマンス最適化、開発者体験向上を追求します。
既存の枠にとらわれず、新しい可能性を探求し、技術的な競争優位性を重視します。`,
        icon: 'Sparkles',
      },
    ],
  },

  'security-review': {
    id: 'security-review',
    label: 'セキュリティレビュー',
    description: 'OWASP準拠の包括的なセキュリティ分析',
    emoji: '🛡️',
    roles: [
      {
        id: 'security-architect',
        name: 'Security Architect',
        systemPrompt: `あなたはセキュリティアーキテクトです。
OWASP Top 10、設計レベルのセキュリティ、Defense in Depth、Zero Trust原則を適用します。
認証・認可・暗号化・監査ログ・データ保護を網羅的に検討します。`,
        icon: 'Shield',
      },
      {
        id: 'pen-tester',
        name: 'Penetration Tester',
        systemPrompt: `あなたは実践的なペネトレーションテスターです。
実際の攻撃シナリオ、脆弱性検証、エクスプロイト可能性、攻撃面分析を行います。
SQLi、XSS、CSRF、認証バイパス、権限昇格などを実践的に検証します。`,
        icon: 'Bug',
      },
      {
        id: 'compliance',
        name: 'Compliance Expert',
        systemPrompt: `あなたはコンプライアンス専門家です。
GDPR、PCI DSS、HIPAA、個人情報保護法などの法的要件準拠を検証します。
データ保持期間、同意管理、監査証跡、インシデント対応計画をチェックします。`,
        icon: 'FileText',
      },
    ],
  },

  'code-review': {
    id: 'code-review',
    label: 'コードレビュー',
    description: '多角的な視点からコード品質を評価',
    emoji: '🔍',
    roles: [
      {
        id: 'maintainability',
        name: 'Maintainability Expert',
        systemPrompt: `あなたは保守性の専門家です。
可読性、命名規則、コメント品質、関数分割、SOLID原則、DRY原則を重視します。
6ヶ月後に別の開発者が読んでも理解できるコードを目指します。`,
        icon: 'FileCode',
      },
      {
        id: 'performance',
        name: 'Performance Expert',
        systemPrompt: `あなたはパフォーマンス専門家です。
時間計算量、空間計算量、N+1問題、キャッシング戦略、並列処理を分析します。
計測可能なパフォーマンス改善提案を行います。`,
        icon: 'Zap',
      },
      {
        id: 'testing',
        name: 'Testing Expert',
        systemPrompt: `あなたはテスト専門家です。
テスト容易性、テストカバレッジ、エッジケース、モック設計、統合テスト戦略を評価します。
テスタビリティを高める設計改善提案を行います。`,
        icon: 'CheckCircle',
      },
    ],
  },
};

/**
 * プリセット定義
 */
export interface PresetDefinition {
  id: string;
  label: string;
  description: string;
  emoji: string;
  roles: RoleDefinition[];
}

/**
 * モデル情報
 */
export const MODEL_INFO: Record<ClaudeModel, ModelInfo> = {
  sonnet: {
    label: 'Sonnet 4.5',
    description: 'バランス型：速度と品質の最適バランス',
    emoji: '⚡',
    costPer1M: 3.0, // USD
    speedRating: 4,
    qualityRating: 5,
  },
  haiku: {
    label: 'Haiku 3.5',
    description: '高速型：最速の応答速度',
    emoji: '🚀',
    costPer1M: 0.8,
    speedRating: 5,
    qualityRating: 3,
  },
  opus: {
    label: 'Opus 4',
    description: '高品質型：最高の出力品質',
    emoji: '💎',
    costPer1M: 15.0,
    speedRating: 2,
    qualityRating: 5,
  },
};

export interface ModelInfo {
  label: string;
  description: string;
  emoji: string;
  costPer1M: number; // USD per 1M tokens
  speedRating: number; // 1-5
  qualityRating: number; // 1-5
}
```

---

## 3. Tauri API拡張

### src/services/tauri.ts (既存ファイルに追加)

```typescript
/**
 * Debate Mode API (added to existing tauriApi object)
 */

// Import types
import { DebateRequest, DebateResult, DebateDetailedStatus } from '@/types/debate';

// Add to tauriApi object:
export const tauriApi = {
  // ... existing methods ...

  // ===== Debate Mode Commands =====

  /**
   * Start a new debate session
   *
   * @param request - Debate configuration
   * @returns DebateResult with debateId
   */
  async executeDebate(request: DebateRequest): Promise<DebateResult> {
    try {
      const result = await invoke<DebateResult>('execute_debate', { request });
      return result;
    } catch (error) {
      throw new Error(`Failed to start debate: ${error}`);
    }
  },

  /**
   * Get current debate status (polling)
   *
   * @param debateId - Debate UUID
   * @returns Current status with round outputs
   */
  async getDebateStatus(debateId: string): Promise<DebateDetailedStatus> {
    try {
      const status = await invoke<DebateDetailedStatus>('get_debate_status', { debateId });
      return status;
    } catch (error) {
      throw new Error(`Failed to get debate status: ${error}`);
    }
  },

  /**
   * Cancel a running debate
   *
   * @param debateId - Debate UUID
   * @param cleanupWorktrees - Whether to remove worktrees (default: true)
   */
  async cancelDebate(debateId: string, cleanupWorktrees: boolean = true): Promise<void> {
    try {
      await invoke('cancel_debate', { debateId, cleanupWorktrees });
    } catch (error) {
      throw new Error(`Failed to cancel debate: ${error}`);
    }
  },

  /**
   * Get debate results (final output)
   *
   * @param debateId - Debate UUID
   * @returns Final result with all round outputs
   */
  async getDebateResults(debateId: string): Promise<DebateDetailedStatus> {
    try {
      const result = await invoke<DebateDetailedStatus>('get_debate_results', { debateId });
      return result;
    } catch (error) {
      throw new Error(`Failed to get debate results: ${error}`);
    }
  },
};
```

---

## 4. State管理

### src/store/debateStore.ts

```typescript
import { create } from 'zustand';
import { DebateRequest, DebateResult, DebateDetailedStatus, RoundOutput } from '@/types/debate';
import { tauriApi } from '@/services/tauri';

interface DebateStore {
  // State
  /** 現在アクティブなディベートID */
  activeDebateId: string | null;

  /** ディベート詳細ステータス */
  debateStatus: DebateDetailedStatus | null;

  /** ステータスポーリング中フラグ */
  isPolling: boolean;

  /** ポーリング間隔ID */
  pollingIntervalId: NodeJS.Timeout | null;

  /** エラーメッセージ */
  error: string | null;

  // Actions
  /** ディベート開始 */
  startDebate: (request: DebateRequest) => Promise<DebateResult>;

  /** ステータスポーリング開始 */
  startPolling: (debateId: string) => void;

  /** ステータスポーリング停止 */
  stopPolling: () => void;

  /** ディベートキャンセル */
  cancelDebate: (debateId: string, cleanupWorktrees?: boolean) => Promise<void>;

  /** ストアリセット */
  reset: () => void;

  /** エラーをクリア */
  clearError: () => void;
}

export const useDebateStore = create<DebateStore>((set, get) => ({
  // Initial State
  activeDebateId: null,
  debateStatus: null,
  isPolling: false,
  pollingIntervalId: null,
  error: null,

  // Actions
  startDebate: async (request: DebateRequest) => {
    set({ error: null });

    try {
      const result = await tauriApi.executeDebate(request);

      set({
        activeDebateId: result.debateId,
        debateStatus: {
          debateId: result.debateId,
          status: result.status,
          currentRound: 0,
          totalRounds: 3,
          roundOutputs: new Map(),
          startedAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        },
      });

      // 自動ポーリング開始
      get().startPolling(result.debateId);

      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      set({ error: errorMessage });
      throw error;
    }
  },

  startPolling: (debateId: string) => {
    const { stopPolling } = get();

    // 既存のポーリングを停止
    stopPolling();

    // ポーリング関数
    const poll = async () => {
      try {
        const status = await tauriApi.getDebateStatus(debateId);

        set({
          debateStatus: status,
          updatedAt: new Date().toISOString(),
        });

        // 完了or失敗したらポーリング停止
        if (status.status.type === 'completed' || status.status.type === 'failed') {
          get().stopPolling();
        }
      } catch (error) {
        console.error('Polling error:', error);
        // エラーでもポーリングは継続（次回成功する可能性）
      }
    };

    // 初回即座に実行
    poll();

    // 5秒ごとにポーリング
    const intervalId = setInterval(poll, 5000);

    set({ isPolling: true, pollingIntervalId: intervalId });
  },

  stopPolling: () => {
    const { pollingIntervalId } = get();

    if (pollingIntervalId) {
      clearInterval(pollingIntervalId);
      set({ isPolling: false, pollingIntervalId: null });
    }
  },

  cancelDebate: async (debateId: string, cleanupWorktrees = true) => {
    set({ error: null });

    try {
      await tauriApi.cancelDebate(debateId, cleanupWorktrees);

      // ポーリング停止
      get().stopPolling();

      // ステート更新
      set({
        activeDebateId: null,
        debateStatus: null,
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      set({ error: errorMessage });
      throw error;
    }
  },

  reset: () => {
    const { stopPolling } = get();

    stopPolling();

    set({
      activeDebateId: null,
      debateStatus: null,
      isPolling: false,
      pollingIntervalId: null,
      error: null,
    });
  },

  clearError: () => {
    set({ error: null });
  },
}));
```

---

## 5. DebateDialog実装

### src/components/AI/DebateDialog.tsx

```typescript
/**
 * DebateDialog - Claude Code Debate Mode
 *
 * 3-round structured debate dialog for multi-perspective task analysis
 */

import React, { useState, useEffect } from 'react';
import { MessageSquare, X, Settings as SettingsIcon, Code2, Clock } from 'lucide-react';
import { RoleSelector } from './RoleSelector';
import { DebateFlowDiagram } from './DebateFlowDiagram';
import { ModeIndicator } from './ModeIndicator';
import { ModeTooltip } from './ModeTooltip';
import { ROLE_PRESETS, MODEL_INFO, ClaudeModel } from '@/types/debate';
import { useDebateStore } from '@/store/debateStore';

export interface DebateDialogProps {
  /** ダイアログ表示状態 */
  isOpen: boolean;

  /** ダイアログを閉じるコールバック */
  onClose: () => void;

  /** ディベート開始時のコールバック */
  onStart?: (debateId: string, task: string) => void;
}

export const DebateDialog: React.FC<DebateDialogProps> = ({
  isOpen,
  onClose,
  onStart,
}) => {
  // State
  const [task, setTask] = useState('');
  const [selectedPreset, setSelectedPreset] = useState<string>('tech-stack');
  const [selectedModel, setSelectedModel] = useState<ClaudeModel>('sonnet');
  const [timeoutSeconds, setTimeoutSeconds] = useState(2400); // 40分
  const [preserveWorktrees, setPreserveWorktrees] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [isStarting, setIsStarting] = useState(false);

  // Store
  const startDebate = useDebateStore((state) => state.startDebate);
  const error = useDebateStore((state) => state.error);
  const clearError = useDebateStore((state) => state.clearError);

  // Reset state when dialog opens
  useEffect(() => {
    if (isOpen) {
      setTask('');
      setSelectedPreset('tech-stack');
      setSelectedModel('sonnet');
      setTimeoutSeconds(2400);
      setPreserveWorktrees(false);
      setShowAdvanced(false);
      setIsStarting(false);
      clearError();
    }
  }, [isOpen, clearError]);

  // ESC key handler
  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen && !isStarting) {
        onClose();
      }
    };

    window.addEventListener('keydown', handleEsc);
    return () => window.removeEventListener('keydown', handleEsc);
  }, [isOpen, isStarting, onClose]);

  const handleStart = async () => {
    if (!task.trim()) {
      alert('タスクを入力してください');
      return;
    }

    const preset = ROLE_PRESETS[selectedPreset];
    if (!preset) {
      alert('ロールプリセットが見つかりません');
      return;
    }

    setIsStarting(true);
    clearError();

    try {
      const result = await startDebate({
        task: task.trim(),
        roles: preset.roles,
        model: selectedModel,
        timeoutSeconds,
        preserveWorktrees,
      });

      console.log('Debate started:', result);

      if (onStart) {
        onStart(result.debateId, task.trim());
      }

      // ダイアログを閉じる
      onClose();
    } catch (error) {
      console.error('Failed to start debate:', error);
      // エラーはストアに保存されているので、ここでは何もしない
    } finally {
      setIsStarting(false);
    }
  };

  if (!isOpen) return null;

  const preset = ROLE_PRESETS[selectedPreset];
  const modelInfo = MODEL_INFO[selectedModel];

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-labelledby="debate-dialog-title"
    >
      <div
        className="w-full max-w-3xl max-h-[85vh] bg-editor-elevated border border-editor-border rounded-xl shadow-2xl overflow-hidden flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center gap-3 px-6 py-4 border-b border-editor-border bg-editor-surface">
          <MessageSquare size={24} className="text-orange-600" />
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-1">
              <h2 id="debate-dialog-title" className="text-lg font-semibold text-text-primary">
                Claude Code ディベート
              </h2>
              <ModeIndicator mode="debate" />
              <ModeTooltip mode="debate" />
            </div>
            <p className="text-xs text-text-tertiary">
              3つの視点から多角的に検討して意思決定の質を向上
            </p>
          </div>
          <button
            onClick={onClose}
            className="p-1 hover:bg-editor-border/30 rounded transition-colors"
            title="閉じる (Esc)"
            aria-label="ダイアログを閉じる"
          >
            <X size={20} className="text-text-tertiary" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          {/* Error Display */}
          {error && (
            <div className="p-4 bg-red-500/10 border border-red-500/30 rounded-lg text-sm text-red-400">
              <strong>エラー:</strong> {error}
            </div>
          )}

          {/* Flow Diagram */}
          <DebateFlowDiagram />

          {/* Task Input */}
          <div>
            <label htmlFor="debate-task" className="block text-sm font-medium text-text-primary mb-2">
              タスク説明
            </label>
            <textarea
              id="debate-task"
              value={task}
              onChange={(e) => setTask(e.target.value)}
              placeholder="検討したいタスクを入力してください...&#10;例: 'Next.js vs Astro でブログを作るべきか？'"
              className="w-full px-4 py-3 bg-editor-bg text-text-primary placeholder-text-tertiary border border-editor-border rounded-lg focus:outline-none focus:ring-2 focus:ring-orange-600/50 resize-none"
              rows={4}
              aria-required="true"
            />
          </div>

          {/* Role Preset Selection */}
          <RoleSelector
            selectedPreset={selectedPreset}
            onSelectPreset={setSelectedPreset}
          />

          {/* Model Selection */}
          <div>
            <label className="block text-sm font-medium text-text-primary mb-3">
              <Code2 size={16} className="inline mr-2" />
              Claude モデル
            </label>
            <div className="grid grid-cols-3 gap-3">
              {(['sonnet', 'haiku', 'opus'] as ClaudeModel[]).map((model) => {
                const isSelected = selectedModel === model;
                const info = MODEL_INFO[model];

                return (
                  <button
                    key={model}
                    onClick={() => setSelectedModel(model)}
                    className={`p-4 rounded-lg border-2 transition-all text-left ${
                      isSelected
                        ? 'border-orange-600 bg-orange-600/10 shadow-glow-sm'
                        : 'border-editor-border hover:border-editor-border/60 bg-editor-surface'
                    }`}
                    aria-pressed={isSelected}
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
              aria-expanded={showAdvanced}
            >
              <SettingsIcon size={16} />
              詳細設定を{showAdvanced ? '非表示' : '表示'}
            </button>

            {showAdvanced && (
              <div className="mt-4 p-4 bg-editor-bg rounded-lg border border-editor-border space-y-4">
                {/* Timeout */}
                <div>
                  <label htmlFor="debate-timeout" className="block text-xs font-medium text-text-secondary mb-2">
                    <Clock size={14} className="inline mr-1" />
                    タイムアウト（全体の制限時間）
                  </label>
                  <div className="flex items-center gap-3">
                    <input
                      id="debate-timeout"
                      type="number"
                      min="600"
                      max="7200"
                      step="600"
                      value={timeoutSeconds}
                      onChange={(e) => {
                        const value = parseInt(e.target.value, 10);
                        setTimeoutSeconds(isNaN(value) ? 2400 : value);
                      }}
                      className="flex-1 px-3 py-2 bg-editor-surface text-text-primary border border-editor-border rounded focus:outline-none focus:ring-2 focus:ring-orange-600/50"
                    />
                    <span className="text-sm text-text-tertiary">
                      = {Math.floor(timeoutSeconds / 60)} 分
                    </span>
                  </div>
                </div>

                {/* Preserve Worktrees */}
                <div className="flex items-start gap-3">
                  <input
                    id="debate-preserve"
                    type="checkbox"
                    checked={preserveWorktrees}
                    onChange={(e) => setPreserveWorktrees(e.target.checked)}
                    className="mt-1"
                  />
                  <div className="flex-1">
                    <label htmlFor="debate-preserve" className="text-xs font-medium text-text-secondary">
                      完了後もworktreeを保持
                    </label>
                    <div className="text-xs text-text-tertiary mt-1">
                      ディベート完了後もGit worktreeと出力を保持し、後で確認できるようにします
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
                {preset.roles.length}つのロール × {modelInfo.label} で実行準備完了
              </>
            )}
          </div>
          <div className="flex gap-3">
            <button
              onClick={onClose}
              disabled={isStarting}
              className="px-4 py-2 text-text-secondary hover:text-text-primary transition-colors disabled:opacity-50"
            >
              キャンセル
            </button>
            <button
              onClick={handleStart}
              disabled={!task.trim() || isStarting}
              className="px-6 py-2 bg-gradient-to-r from-orange-600 to-red-600 hover:from-orange-500 hover:to-red-500 disabled:from-editor-border disabled:to-editor-border disabled:text-text-tertiary text-white font-semibold rounded-lg transition-all shadow-glow-sm hover:shadow-glow-md"
              aria-busy={isStarting}
            >
              {isStarting ? '起動中...' : '💬 ディベート開始'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
```

---

## 6. RoleSelector実装

### src/components/AI/RoleSelector.tsx

```typescript
import React from 'react';
import * as Icons from 'lucide-react';
import { ROLE_PRESETS } from '@/types/debate';

interface RoleSelectorProps {
  selectedPreset: string;
  onSelectPreset: (preset: string) => void;
}

export const RoleSelector: React.FC<RoleSelectorProps> = ({
  selectedPreset,
  onSelectPreset,
}) => {
  const presets = Object.values(ROLE_PRESETS);

  return (
    <div>
      <label className="block text-sm font-medium text-text-primary mb-3">
        ロール選択
      </label>
      <div className="space-y-3">
        {presets.map((preset) => {
          const isSelected = selectedPreset === preset.id;

          return (
            <button
              key={preset.id}
              onClick={() => onSelectPreset(preset.id)}
              className={`w-full p-4 rounded-lg border-2 transition-all text-left ${
                isSelected
                  ? 'border-orange-600 bg-orange-600/10'
                  : 'border-editor-border hover:border-editor-border/60 bg-editor-surface'
              }`}
              aria-pressed={isSelected}
            >
              <div className="flex items-center gap-2 mb-2">
                <span className="text-xl">{preset.emoji}</span>
                <div className="font-semibold text-sm text-text-primary">
                  {preset.label}
                </div>
              </div>
              <div className="text-xs text-text-tertiary mb-3">
                {preset.description}
              </div>

              {/* Role badges */}
              <div className="flex flex-wrap gap-2">
                {preset.roles.map((role) => {
                  // Dynamically get Lucide icon component
                  const IconComponent = Icons[role.icon as keyof typeof Icons] as React.FC<{ size: number }>;

                  return (
                    <div
                      key={role.id}
                      className="flex items-center gap-1.5 px-2 py-1 rounded bg-editor-bg text-text-secondary text-xs"
                    >
                      {IconComponent && <IconComponent size={14} />}
                      {role.name}
                    </div>
                  );
                })}
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
};
```

---

## 7. DebateStatusPanel実装

### src/components/AI/DebateStatusPanel.tsx

```typescript
import React, { useEffect } from 'react';
import { Loader2, CheckCircle2, XCircle, Clock, MessageSquare } from 'lucide-react';
import { useDebateStore } from '@/store/debateStore';

export const DebateStatusPanel: React.FC = () => {
  const { activeDebateId, debateStatus, isPolling, error } = useDebateStore();

  if (!activeDebateId || !debateStatus) {
    return null;
  }

  const { status, currentRound, totalRounds } = debateStatus;

  const getRoundProgress = () => {
    switch (status.type) {
      case 'started':
        return { current: 0, total: 3, label: '開始準備中...', percentage: 0 };
      case 'round1InProgress':
        return { current: 1, total: 3, label: 'Round 1: 独立提案', percentage: 33 };
      case 'round2InProgress':
        return { current: 2, total: 3, label: 'Round 2: 批判的分析', percentage: 66 };
      case 'round3InProgress':
        return { current: 3, total: 3, label: 'Round 3: 合意形成', percentage: 100 };
      case 'completed':
        return { current: 3, total: 3, label: '✅ 完了', percentage: 100 };
      case 'failed':
        return { current: status.failedAtRound, total: 3, label: '❌ 失敗', percentage: (status.failedAtRound / 3) * 100 };
      default:
        return { current: 0, total: 3, label: '不明', percentage: 0 };
    }
  };

  const progress = getRoundProgress();

  return (
    <div className="bg-editor-surface border border-editor-border rounded-lg p-4 shadow-md">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          {status.type === 'completed' ? (
            <CheckCircle2 size={20} className="text-green-400" />
          ) : status.type === 'failed' ? (
            <XCircle size={20} className="text-red-400" />
          ) : (
            <Loader2 size={20} className="text-blue-400 animate-spin" />
          )}
          <span className="font-semibold text-text-primary">{progress.label}</span>
        </div>
        <span className="text-sm text-text-tertiary">
          {progress.current} / {progress.total} ラウンド
        </span>
      </div>

      {/* Progress Bar */}
      <div className="w-full bg-editor-bg rounded-full h-2 mb-2 overflow-hidden">
        <div
          className="bg-gradient-to-r from-orange-600 to-red-600 h-2 rounded-full transition-all duration-500"
          style={{ width: `${progress.percentage}%` }}
          role="progressbar"
          aria-valuenow={progress.percentage}
          aria-valuemin={0}
          aria-valuemax={100}
        />
      </div>

      {/* Current Role */}
      {(status.type === 'round1InProgress' || status.type === 'round2InProgress' || status.type === 'round3InProgress') && (
        <div className="mt-2 flex items-center gap-2 text-sm text-text-secondary">
          <MessageSquare size={16} />
          <span>現在: {status.currentRole}</span>
        </div>
      )}

      {/* Error Message */}
      {status.type === 'failed' && (
        <div className="mt-3 p-3 bg-red-500/10 border border-red-500/30 rounded text-sm text-red-400">
          <strong>エラー:</strong> {status.error}
        </div>
      )}

      {/* Completion Message */}
      {status.type === 'completed' && (
        <div className="mt-3 p-3 bg-green-500/10 border border-green-500/30 rounded text-sm text-green-400">
          ディベートが完了しました。MultiAgentパネルで結果を確認してください。
        </div>
      )}

      {/* Store Error */}
      {error && (
        <div className="mt-3 p-3 bg-red-500/10 border border-red-500/30 rounded text-sm text-red-400">
          <strong>システムエラー:</strong> {error}
        </div>
      )}
    </div>
  );
};
```

---

## 8. DebateFlowDiagram実装

### src/components/AI/DebateFlowDiagram.tsx

```typescript
import React from 'react';
import { ArrowRight, MessageSquare, Users, Trophy } from 'lucide-react';

export const DebateFlowDiagram: React.FC = () => {
  const rounds = [
    {
      number: 1,
      label: '独立提案',
      description: '各ロールが独立して提案',
      icon: MessageSquare,
      color: 'text-blue-400',
    },
    {
      number: 2,
      label: '批判的分析',
      description: '他の提案を批判的に検証',
      icon: Users,
      color: 'text-purple-400',
    },
    {
      number: 3,
      label: '合意形成',
      description: '最終的な統合案を作成',
      icon: Trophy,
      color: 'text-orange-400',
    },
  ];

  return (
    <div className="bg-editor-bg rounded-lg border border-editor-border p-4">
      <h3 className="text-sm font-semibold text-text-primary mb-4">ディベートフロー</h3>

      <div className="flex items-center justify-between">
        {rounds.map((round, index) => {
          const Icon = round.icon;

          return (
            <React.Fragment key={round.number}>
              <div className="flex-1 text-center">
                <div className={`inline-flex items-center justify-center w-12 h-12 rounded-full bg-editor-surface border-2 border-editor-border ${round.color} mb-2`}>
                  <Icon size={20} />
                </div>
                <div className="text-xs font-semibold text-text-primary mb-1">
                  Round {round.number}
                </div>
                <div className="text-xs text-text-secondary mb-1">
                  {round.label}
                </div>
                <div className="text-xs text-text-tertiary">
                  {round.description}
                </div>
              </div>

              {index < rounds.length - 1 && (
                <div className="flex-shrink-0 px-3">
                  <ArrowRight size={20} className="text-text-tertiary" />
                </div>
              )}
            </React.Fragment>
          );
        })}
      </div>
    </div>
  );
};
```

---

## 9. UIモックアップ

### 9.1 DebateDialog Layout (ASCII Art)

```
┌────────────────────────────────────────────────────────────┐
│ 💬 Claude Code ディベート                 [Debate] [?] [×] │
│ 3つの視点から多角的に検討して意思決定の質を向上            │
├────────────────────────────────────────────────────────────┤
│                                                            │
│ ┌────────────────────────────────────────────────────┐   │
│ │ [Round 1: 独立提案] → [Round 2: 批判的分析] → [Round 3: 合意形成] │
│ └────────────────────────────────────────────────────┘   │
│                                                            │
│ タスク説明                                                 │
│ ┌──────────────────────────────────────────────────┐     │
│ │ Next.js vs Astro でブログを作るべきか？          │     │
│ │ SEO、パフォーマンス、開発体験を考慮したい        │     │
│ │                                                  │     │
│ └──────────────────────────────────────────────────┘     │
│                                                            │
│ ロール選択                                                 │
│ ┌──────────────────────────────────────────────────┐     │
│ │ ● 🏗️ 技術スタック選定 (推奨)                      │     │
│ │   最適な技術選択を多角的に検討                   │     │
│ │   [🏗️ Architect] [🔧 Pragmatist] [✨ Innovator]  │     │
│ └──────────────────────────────────────────────────┘     │
│ ┌──────────────────────────────────────────────────┐     │
│ │ ○ 🛡️ セキュリティレビュー                         │     │
│ │   OWASP準拠の包括的な分析                        │     │
│ │   [🛡️ Security] [🐛 PenTest] [📄 Compliance]     │     │
│ └──────────────────────────────────────────────────┘     │
│ ┌──────────────────────────────────────────────────┐     │
│ │ ○ 🔍 コードレビュー                               │     │
│ │   多角的な視点からコード品質を評価               │     │
│ │   [📄 Maintain] [⚡ Perf] [✓ Test]              │     │
│ └──────────────────────────────────────────────────┘     │
│                                                            │
│ Claude モデル                                              │
│ ┌───────────┐ ┌───────────┐ ┌───────────┐            │
│ │ ⚡ Sonnet  │ │ 🚀 Haiku  │ │ 💎 Opus   │            │
│ │ 4.5       │ │ 3.5       │ │ 4         │            │
│ │ バランス  │ │ 高速      │ │ 高品質    │            │
│ └───────────┘ └───────────┘ └───────────┘            │
│                                                            │
│ [詳細設定 ▼]                                               │
│                                                            │
│ ────────────────────────────────────────────────────────  │
│ 3つのロール × Sonnet 4.5 で実行準備完了                    │
│                         [キャンセル] [💬 ディベート開始]    │
└────────────────────────────────────────────────────────────┘
```

### 9.2 DebateStatusPanel Layout

```
┌────────────────────────────────────────────────────┐
│ 🔄 Round 2: 批判的分析            2 / 3 ラウンド   │
├────────────────────────────────────────────────────┤
│ ████████████████░░░░░░░░ 66%                       │
│ 💬 現在: Pragmatist                                │
└────────────────────────────────────────────────────┘
```

---

## 10. レスポンシブ対応

### 10.1 ブレークポイント

```typescript
// Tailwind CSS breakpoints (既存設定を流用)
// sm: 640px
// md: 768px
// lg: 1024px
// xl: 1280px

// DebateDialog: lg以下では幅90%, xl以上では max-w-3xl
// RoleSelector: sm以下では1列、md以上では1列（全幅）
// ModelSelector: 常に3列（sm以下では横スクロール可能）
```

### 10.2 モバイル最適化

```typescript
// Mobile-specific optimizations (useEffect hook)
useEffect(() => {
  const isMobile = window.innerWidth < 768;

  if (isMobile) {
    // タスク入力: rows=3 (デスクトップは4)
    // 詳細設定: デフォルトで閉じる
    // モデル選択: 横スクロール
    setShowAdvanced(false);
  }
}, []);
```

### 10.3 Tailwind Responsive Classes

```tsx
<div className="grid grid-cols-1 md:grid-cols-3 gap-3">
  {/* Mobile: 1列, Desktop: 3列 */}
</div>

<div className="max-w-3xl max-h-[85vh] overflow-y-auto">
  {/* モバイルでは画面に収まるように */}
</div>
```

---

## 11. アクセシビリティ

### 11.1 キーボード操作

```typescript
// ESC: ダイアログ閉じる
useEffect(() => {
  const handleEsc = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && isOpen && !isStarting) {
      onClose();
    }
  };
  window.addEventListener('keydown', handleEsc);
  return () => window.removeEventListener('keydown', handleEsc);
}, [isOpen, isStarting, onClose]);

// Tab: フォーカス移動（自動）
// Enter: 送信（textarea内ではShift+Enter）
```

### 11.2 ARIA属性

```tsx
<div
  role="dialog"
  aria-modal="true"
  aria-labelledby="debate-dialog-title"
>
  <h2 id="debate-dialog-title">Claude Code ディベート</h2>

  <textarea
    id="debate-task"
    aria-required="true"
    aria-label="タスク説明を入力"
  />

  <button
    aria-pressed={isSelected}
    aria-busy={isStarting}
  >
    ディベート開始
  </button>

  <div
    role="progressbar"
    aria-valuenow={66}
    aria-valuemin={0}
    aria-valuemax={100}
  />
</div>
```

### 11.3 スクリーンリーダー対応

```tsx
// すべてのインタラクティブ要素にaria-label
<button aria-label="ダイアログを閉じる">
  <X size={20} />
</button>

// フォーカス可能な要素の順序
tabIndex={0} // 自然な順序
tabIndex={-1} // フォーカス不可
```

---

## 12. パフォーマンス最適化

### 12.1 メモ化

```typescript
import { memo, useMemo, useCallback } from 'react';

// Component memoization
export const DebateDialog = memo(({ isOpen, onClose, onStart }) => {
  // ...
});

// Callback memoization
const handleStart = useCallback(async () => {
  // ...
}, [task, selectedPreset, selectedModel, timeoutSeconds, preserveWorktrees, startDebate]);

// Value memoization
const preset = useMemo(() => ROLE_PRESETS[selectedPreset], [selectedPreset]);
```

### 12.2 遅延ロード

```typescript
// 大きなコンポーネントは動的import (必要に応じて)
const DebateFlowDiagram = lazy(() => import('./DebateFlowDiagram'));

// ...

<Suspense fallback={<div className="h-24 bg-editor-bg animate-pulse rounded" />}>
  <DebateFlowDiagram />
</Suspense>
```

### 12.3 ポーリング最適化

```typescript
// 5秒ごとのポーリング（バックエンド負荷とリアルタイム性のバランス）
const intervalId = setInterval(poll, 5000);

// 完了時に即座にポーリング停止
if (status.type === 'completed' || status.type === 'failed') {
  get().stopPolling();
}

// コンポーネントアンマウント時にクリーンアップ
useEffect(() => {
  return () => {
    stopPolling();
  };
}, []);
```

---

## 13. エラーハンドリング

### 13.1 Error Boundary

```typescript
class DebateErrorBoundary extends React.Component {
  state = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('DebateDialog Error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="p-6 bg-red-500/10 border border-red-500/30 rounded">
          <h3 className="text-lg font-semibold text-red-400 mb-2">エラーが発生しました</h3>
          <pre className="text-sm text-red-300 overflow-auto">{this.state.error?.message}</pre>
          <button
            onClick={() => this.setState({ hasError: false, error: null })}
            className="mt-4 px-4 py-2 bg-red-600 text-white rounded"
          >
            リトライ
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

// Usage
<DebateErrorBoundary>
  <DebateDialog ... />
</DebateErrorBoundary>
```

### 13.2 API Error Handling

```typescript
// Tauri API wrapper (src/services/tauri.ts)
async executeDebate(request: DebateRequest): Promise<DebateResult> {
  try {
    const result = await invoke<DebateResult>('execute_debate', { request });
    return result;
  } catch (error) {
    // エラーを詳細なメッセージに変換
    if (error instanceof Error) {
      throw new Error(`Failed to start debate: ${error.message}`);
    } else {
      throw new Error(`Failed to start debate: ${String(error)}`);
    }
  }
}

// Store error handling
startDebate: async (request: DebateRequest) => {
  set({ error: null });

  try {
    const result = await tauriApi.executeDebate(request);
    set({ activeDebateId: result.debateId });
    return result;
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    set({ error: errorMessage });
    throw error; // 再スロー（呼び出し側でハンドル）
  }
}
```

---

## 14. テスト戦略

### 14.1 ユニットテスト (Vitest)

```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { DebateDialog } from './DebateDialog';
import { useDebateStore } from '@/store/debateStore';

describe('DebateDialog', () => {
  beforeEach(() => {
    // Reset store before each test
    useDebateStore.getState().reset();
  });

  it('renders when open', () => {
    render(<DebateDialog isOpen={true} onClose={() => {}} />);
    expect(screen.getByText('Claude Code ディベート')).toBeInTheDocument();
  });

  it('does not render when closed', () => {
    const { container } = render(<DebateDialog isOpen={false} onClose={() => {}} />);
    expect(container.firstChild).toBeNull();
  });

  it('calls onStart with correct parameters', async () => {
    const mockOnStart = vi.fn();
    const mockStartDebate = vi.fn().mockResolvedValue({
      debateId: 'test-id',
      status: { type: 'started' },
      message: 'Started',
    });

    // Mock store
    vi.spyOn(useDebateStore, 'getState').mockReturnValue({
      startDebate: mockStartDebate,
      error: null,
      clearError: () => {},
    });

    render(<DebateDialog isOpen={true} onClose={() => {}} onStart={mockOnStart} />);

    // タスク入力
    const textarea = screen.getByLabelText('タスク説明');
    fireEvent.change(textarea, { target: { value: 'Test task' } });

    // ロール選択（デフォルトで tech-stack が選択済み）
    // モデル選択（デフォルトで sonnet が選択済み）

    // 開始ボタンクリック
    const startButton = screen.getByText('💬 ディベート開始');
    fireEvent.click(startButton);

    await waitFor(() => {
      expect(mockStartDebate).toHaveBeenCalledWith({
        task: 'Test task',
        roles: expect.any(Array),
        model: 'sonnet',
        timeoutSeconds: 2400,
        preserveWorktrees: false,
      });
      expect(mockOnStart).toHaveBeenCalledWith('test-id', 'Test task');
    });
  });

  it('validates empty task', () => {
    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {});

    render(<DebateDialog isOpen={true} onClose={() => {}} />);

    const startButton = screen.getByText('💬 ディベート開始');
    fireEvent.click(startButton);

    expect(alertSpy).toHaveBeenCalledWith('タスクを入力してください');
  });

  it('handles ESC key', () => {
    const mockOnClose = vi.fn();

    render(<DebateDialog isOpen={true} onClose={mockOnClose} />);

    fireEvent.keyDown(window, { key: 'Escape' });

    expect(mockOnClose).toHaveBeenCalled();
  });
});

describe('DebateStatusPanel', () => {
  it('does not render when no active debate', () => {
    vi.spyOn(useDebateStore, 'getState').mockReturnValue({
      activeDebateId: null,
      debateStatus: null,
    });

    const { container } = render(<DebateStatusPanel />);
    expect(container.firstChild).toBeNull();
  });

  it('renders progress for round 2', () => {
    vi.spyOn(useDebateStore, 'getState').mockReturnValue({
      activeDebateId: 'test-id',
      debateStatus: {
        status: { type: 'round2InProgress', currentRole: 'Pragmatist' },
        currentRound: 2,
        totalRounds: 3,
      },
    });

    render(<DebateStatusPanel />);

    expect(screen.getByText('Round 2: 批判的分析')).toBeInTheDocument();
    expect(screen.getByText('2 / 3 ラウンド')).toBeInTheDocument();
    expect(screen.getByText('現在: Pragmatist')).toBeInTheDocument();
  });
});
```

### 14.2 統合テスト (Playwright)

```typescript
import { test, expect } from '@playwright/test';

test.describe('Debate Mode E2E', () => {
  test('complete debate flow', async ({ page }) => {
    await page.goto('http://localhost:5173');

    // ディベートボタンクリック
    await page.click('button:has-text("💬 討論")');

    // ダイアログが表示される
    await expect(page.locator('h2:has-text("Claude Code ディベート")')).toBeVisible();

    // タスク入力
    await page.fill('textarea#debate-task', 'Next.js vs Astro for blog');

    // ロール選択（デフォルトで tech-stack が選択済み）
    await expect(page.locator('button[aria-pressed="true"]:has-text("技術スタック選定")')).toBeVisible();

    // モデル選択（デフォルトで sonnet が選択済み）
    await expect(page.locator('button[aria-pressed="true"]:has-text("Sonnet 4.5")')).toBeVisible();

    // 開始ボタンクリック
    await page.click('button:has-text("💬 ディベート開始")');

    // ステータスパネルが表示される
    await expect(page.locator('text=Round 1: 独立提案')).toBeVisible({ timeout: 10000 });

    // 進捗が進む（Round 2まで待つ）
    await expect(page.locator('text=Round 2: 批判的分析')).toBeVisible({ timeout: 600000 }); // 10分

    // 完了を待つ（最大40分）
    await expect(page.locator('text=✅ 完了')).toBeVisible({ timeout: 2400000 }); // 40分
  });

  test('cancel debate', async ({ page }) => {
    await page.goto('http://localhost:5173');

    // ディベート開始
    await page.click('button:has-text("💬 討論")');
    await page.fill('textarea#debate-task', 'Test task');
    await page.click('button:has-text("💬 ディベート開始")');

    // ステータスパネル表示
    await expect(page.locator('text=Round 1: 独立提案')).toBeVisible({ timeout: 10000 });

    // キャンセルボタンクリック（TODO: キャンセルUIを追加する必要がある）
    // await page.click('button:has-text("キャンセル")');

    // ステータスパネルが消える
    // await expect(page.locator('text=Round 1: 独立提案')).not.toBeVisible();
  });
});
```

---

## 15. 実装チェックリスト

### Phase 1: 基本実装 (Week 1-2)

- [ ] **型定義** (1日)
  - [ ] `src/types/debate.ts` 作成
  - [ ] `DebateRequest`, `DebateResult`, `DebateStatus` 定義
  - [ ] `ROLE_PRESETS`, `MODEL_INFO` 定義
  - [ ] TypeScript strict mode でエラー0件

- [ ] **Tauri API拡張** (1日)
  - [ ] `src/services/tauri.ts` に `executeDebate`, `getDebateStatus`, `cancelDebate` 追加
  - [ ] エラーハンドリング実装
  - [ ] 型安全性確認

- [ ] **State管理** (2日)
  - [ ] `src/store/debateStore.ts` 作成
  - [ ] Zustand store 実装
  - [ ] ポーリングロジック実装
  - [ ] エラーハンドリング

- [ ] **DebateDialog** (3日)
  - [ ] `src/components/AI/DebateDialog.tsx` 作成
  - [ ] タスク入力、ロール選択、モデル選択UI
  - [ ] 詳細設定（タイムアウト、worktree保持）
  - [ ] ESCキー、アクセシビリティ対応

- [ ] **RoleSelector** (1日)
  - [ ] `src/components/AI/RoleSelector.tsx` 作成
  - [ ] プリセット選択UI
  - [ ] アイコン表示（Lucide React）

- [ ] **DebateStatusPanel** (1日)
  - [ ] `src/components/AI/DebateStatusPanel.tsx` 作成
  - [ ] プログレスバー、ステータス表示
  - [ ] エラー表示

- [ ] **DebateFlowDiagram** (1日)
  - [ ] `src/components/AI/DebateFlowDiagram.tsx` 作成
  - [ ] 3ラウンドフロー図

### Phase 2: 統合とテスト (Week 3-4)

- [ ] **MultiAgentPanel統合** (2日)
  - [ ] ディベート結果を MultiAgentPanel で表示
  - [ ] ラウンド別出力表示
  - [ ] worktree情報表示

- [ ] **レスポンシブ対応** (1日)
  - [ ] モバイル（sm）、タブレット（md）、デスクトップ（lg+）
  - [ ] タッチ操作テスト

- [ ] **アクセシビリティ** (1日)
  - [ ] キーボード操作確認
  - [ ] ARIA属性確認
  - [ ] スクリーンリーダーテスト

- [ ] **ユニットテスト** (2日)
  - [ ] `DebateDialog.test.tsx`
  - [ ] `DebateStatusPanel.test.tsx`
  - [ ] `RoleSelector.test.tsx`
  - [ ] カバレッジ >= 80%

- [ ] **統合テスト** (2日)
  - [ ] Playwright E2E テスト
  - [ ] 完全なディベートフロー
  - [ ] エラーケース

- [ ] **パフォーマンス最適化** (1日)
  - [ ] React.memo 適用
  - [ ] useCallback / useMemo 最適化
  - [ ] Lighthouse Performance Score >= 90

- [ ] **ドキュメント** (1日)
  - [ ] コンポーネントJSDoc
  - [ ] README更新
  - [ ] 使用方法ガイド

---

## 成功条件

**テスト**:
- [ ] すべてのテストがパス
- [ ] カバレッジ >= 80%

**型安全性**:
- [ ] TypeScript strict mode でエラー0件
- [ ] ESLint エラー0件

**パフォーマンス**:
- [ ] Lighthouse Performance Score >= 90
- [ ] Lighthouse Accessibility Score >= 90

**アクセシビリティ**:
- [ ] WCAG 2.1 AA 準拠
- [ ] キーボード操作完全対応
- [ ] スクリーンリーダー対応

**実装品質**:
- [ ] エラーハンドリング完全実装
- [ ] レスポンシブ対応（Mobile First）
- [ ] コードレビューパス（>= 90/100）

---

## 推定工数

| タスク | 工数 | 担当 |
|--------|------|------|
| 型定義 + Tauri API | 2日 | Frontend Dev |
| State管理 | 2日 | Frontend Dev |
| DebateDialog | 3日 | Frontend Dev |
| RoleSelector + StatusPanel + FlowDiagram | 3日 | Frontend Dev |
| 統合 + テスト | 4日 | Frontend Dev |
| **合計** | **14日** | **1名** |

**余裕をもって2週間（10営業日）**で完了可能。

---

## 次のステップ

1. **Backend実装** (Rust側)
   - `execute_debate` コマンド実装
   - `get_debate_status` ポーリングAPI
   - worktree管理、tmux管理

2. **Phase 2 拡張**
   - カスタムロール作成UI
   - ディベート履歴管理
   - 結果比較ツール

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-04
**Author**: Claude Code (Frontend Specialist)
