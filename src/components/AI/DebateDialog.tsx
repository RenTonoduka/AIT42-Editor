/**
 * DebateDialog - Claude Code Debate Mode (Collaborative Dialogue)
 *
 * Launches 3 role-based Claude Code agents in sequential rounds
 * for multi-perspective debate and consensus formation
 */

import React, { useState, useEffect } from 'react';
import { MessageSquare, X, Settings as SettingsIcon, Code2, Users } from 'lucide-react';
import { tauriApi, DebateRequest, RoleDefinition } from '@/services/tauri';
import { DebateFlowDiagram } from './DebateFlowDiagram';

export interface DebateDialogProps {
  /** Whether the dialog is visible */
  isOpen: boolean;
  /** Callback when dialog should close */
  onClose: () => void;
  /** Callback when debate starts */
  onStart?: (debateId: string, task: string) => void;
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
 * Role presets for different debate scenarios
 */
const ROLE_PRESETS: Record<string, { name: string; description: string; roles: RoleDefinition[] }> = {
  'tech-stack': {
    name: '技術スタック選定',
    description: 'アーキテクト、現実主義者、革新者の視点で技術選択を議論',
    roles: [
      {
        id: 'architect',
        name: 'Technical Architect',
        systemPrompt: `# あなたのロール: Technical Architect (技術アーキテクト)

あなたは佐藤太郎、15年のシステム開発経験を持つシニアアーキテクトです。

## あなたの性格と視点
- **慎重派**: 新技術より実績ある技術を選ぶ
- **長期視点**: 5年後も保守できる設計を重視
- **原則主義**: SOLID原則、デザインパターンに忠実
- **エビデンス重視**: 「なんとなく」ではなく、根拠を示す

## 重視すべき観点
1. アーキテクチャパターン
2. 技術スタックの長期保守性
3. スケーラビリティ（10倍対応可能か）
4. 保守性（新メンバーが3ヶ月で理解可能か）
5. 技術的負債の最小化`,
      },
      {
        id: 'pragmatist',
        name: 'Pragmatist',
        systemPrompt: `# あなたのロール: Pragmatist (現実主義者)

あなたは田中花子、10年の開発経験を持つテックリードです。

## あなたの性格と視点
- **現実的**: 理想より実現可能性を重視
- **チーム重視**: メンバーのスキルセットを考慮
- **予算意識**: コストパフォーマンスを常に意識
- **リスク管理**: 失敗時の影響を最小化

## 重視すべき観点
1. 実装期間（現実的なスケジュール）
2. チームスキルとの適合性
3. 予算制約（インフラコスト、ライセンス）
4. リスク（技術的負債の許容範囲）
5. 段階的アプローチの可能性`,
      },
      {
        id: 'innovator',
        name: 'Innovator',
        systemPrompt: `# あなたのロール: Innovator (革新者)

あなたは山田次郎、7年の開発経験を持つテックリードです。

## あなたの性格と視点
- **革新志向**: 最新技術で差別化を図る
- **ユーザー体験重視**: UXで圧倒的な体験を提供
- **スピード重視**: 素早いリリースとイテレーション
- **未来志向**: 3-5年先のトレンドを見据える

## 重視すべき観点
1. 差別化ポイント（競合との違い）
2. 最新技術の活用（ユーザー体験向上）
3. UX革新（直感的、魅力的）
4. バイラル性（口コミで広がる要素）
5. 未来への拡張性`,
      },
    ],
  },
  'security-review': {
    name: 'セキュリティレビュー',
    description: 'セキュリティ、開発効率、ビジネス価値の視点で議論',
    roles: [
      {
        id: 'security',
        name: 'Security Expert',
        systemPrompt: `# あなたのロール: Security Expert (セキュリティ専門家)

あなたはセキュリティを最優先で考える専門家です。

## 重視すべき観点
1. OWASP Top 10対策
2. データ保護とプライバシー
3. 認証・認可の強度
4. 脆弱性スキャンと監視
5. コンプライアンス（GDPR、個人情報保護法）`,
      },
      {
        id: 'developer',
        name: 'Developer',
        systemPrompt: `# あなたのロール: Developer (開発者)

あなたは開発効率と開発者体験を重視するエンジニアです。

## 重視すべき観点
1. 開発速度（デプロイ頻度）
2. 開発者体験（DX）
3. デバッグのしやすさ
4. テスト容易性
5. ドキュメント充実度`,
      },
      {
        id: 'business',
        name: 'Business Analyst',
        systemPrompt: `# あなたのロール: Business Analyst (ビジネスアナリスト)

あなたはビジネス価値を最大化する視点で考えます。

## 重視すべき観点
1. ROI（投資対効果）
2. Time to Market（市場投入速度）
3. ユーザー獲得コスト
4. 解約率・継続率
5. 競合優位性`,
      },
    ],
  },
  'architecture-decision': {
    name: 'アーキテクチャ決定',
    description: 'モノリス vs マイクロサービス、技術選択などの重要な意思決定',
    roles: [
      {
        id: 'architect',
        name: 'System Architect',
        systemPrompt: `# あなたのロール: System Architect (システムアーキテクト)

システム全体の構造を設計する責任者です。

## 重視すべき観点
1. スケーラビリティ設計
2. 疎結合・高凝集
3. 運用容易性
4. 障害復旧性
5. 技術的整合性`,
      },
      {
        id: 'operations',
        name: 'Operations Engineer',
        systemPrompt: `# あなたのロール: Operations Engineer (運用エンジニア)

システムを安定稼働させる責任者です。

## 重視すべき観点
1. 可観測性（ログ、メトリクス、トレース）
2. インシデント対応速度
3. 自動化（CI/CD、IaC）
4. コスト最適化
5. SLA/SLO達成`,
      },
      {
        id: 'product',
        name: 'Product Manager',
        systemPrompt: `# あなたのロール: Product Manager (プロダクトマネージャー)

プロダクトの成功を目指す責任者です。

## 重視すべき観点
1. ユーザーニーズ適合
2. リリーススピード
3. A/Bテスト容易性
4. データドリブン意思決定
5. プロダクトマーケットフィット`,
      },
    ],
  },
};

/**
 * DebateDialog component
 */
export const DebateDialog: React.FC<DebateDialogProps> = ({
  isOpen,
  onClose,
  onStart,
}) => {
  const [task, setTask] = useState('');
  const [selectedPreset, setSelectedPreset] = useState<string>('tech-stack');
  const [selectedModel, setSelectedModel] = useState<ClaudeModel>('sonnet');
  const [timeoutSeconds, setTimeoutSeconds] = useState(800);
  const [preserveWorktrees, setPreserveWorktrees] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [isStarting, setIsStarting] = useState(false);

  // Reset state when dialog opens
  useEffect(() => {
    if (isOpen) {
      setTask('');
      setSelectedPreset('tech-stack');
      setSelectedModel('sonnet');
      setTimeoutSeconds(800);
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

    // Check if workspace is a Git repository
    try {
      const workspace = await tauriApi.getWorkspace();
      if (!workspace.is_git_repo) {
        alert(
          `現在のワークスペースはGitリポジトリではありません。\n\n` +
          `ワークスペース: ${workspace.path}\n\n` +
          `右上の「フォルダを開く」ボタンからGitリポジトリを選択してください。`
        );
        return;
      }
    } catch (error) {
      alert(`ワークスペースの確認に失敗しました:\n${error}`);
      return;
    }

    const preset = ROLE_PRESETS[selectedPreset];
    if (!preset) {
      alert('ロールプリセットが見つかりません');
      return;
    }

    setIsStarting(true);
    try {
      const request: DebateRequest = {
        task: task.trim(),
        roles: preset.roles,
        model: selectedModel,
        timeoutSeconds,
        preserveWorktrees,
      };

      const result = await tauriApi.executeDebate(request);

      console.log('Debate started:', result);

      if (onStart) {
        onStart(result.debateId, task.trim());
      }

      setIsStarting(false);
      onClose();
    } catch (error) {
      console.error('Failed to start debate:', error);
      alert(`ディベートの開始に失敗しました: ${error}`);
      setIsStarting(false);
    }
  };

  if (!isOpen) return null;

  const currentPreset = ROLE_PRESETS[selectedPreset];

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="w-full max-w-4xl max-h-[85vh] bg-gray-800 border border-gray-700 rounded-xl shadow-2xl overflow-hidden flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center gap-3 px-6 py-4 border-b border-gray-700 bg-gray-850">
          <MessageSquare size={24} className="text-blue-400" />
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-1">
              <h2 className="text-lg font-semibold text-gray-100">
                Claude Code ディベート
              </h2>
              <span className="px-2 py-0.5 text-xs font-medium bg-blue-600/20 text-blue-400 rounded">
                Collaborative
              </span>
            </div>
            <p className="text-xs text-gray-400">
              3つの異なる視点（3ロール × 3ラウンド）で多角的に議論し、統合提案を作成
            </p>
          </div>
          <button
            onClick={onClose}
            className="p-1 hover:bg-gray-700 rounded transition-colors"
            title="閉じる (Esc)"
          >
            <X size={20} className="text-gray-400" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          {/* Flow Diagram */}
          <DebateFlowDiagram />

          {/* Task Input */}
          <div>
            <label className="block text-sm font-medium text-gray-200 mb-2">
              ディベートテーマ
            </label>
            <textarea
              value={task}
              onChange={(e) => setTask(e.target.value)}
              placeholder="議論したいテーマやタスクを入力してください...&#10;例: '新規プロジェクトの技術スタックを選定したい（React vs Vue、REST vs GraphQL）'"
              className="w-full px-4 py-3 bg-gray-900 text-gray-100 placeholder-gray-500 border border-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500/50 resize-none"
              rows={4}
            />
          </div>

          {/* Role Preset Selection */}
          <div>
            <label className="block text-sm font-medium text-gray-200 mb-3">
              <Users size={16} className="inline mr-2" />
              ロールプリセット
            </label>
            <div className="grid grid-cols-1 gap-3">
              {Object.entries(ROLE_PRESETS).map(([key, preset]) => {
                const isSelected = selectedPreset === key;

                return (
                  <button
                    key={key}
                    onClick={() => setSelectedPreset(key)}
                    className={`p-4 rounded-lg border-2 transition-all text-left ${
                      isSelected
                        ? 'border-blue-500 bg-blue-500/10'
                        : 'border-gray-700 hover:border-gray-600 bg-gray-850'
                    }`}
                  >
                    <div className="flex items-start justify-between mb-2">
                      <div className="font-semibold text-sm text-gray-100">
                        {preset.name}
                      </div>
                      {isSelected && (
                        <span className="text-blue-400">✓</span>
                      )}
                    </div>
                    <div className="text-xs text-gray-400 mb-3">
                      {preset.description}
                    </div>
                    <div className="flex gap-2">
                      {preset.roles.map((role) => (
                        <span
                          key={role.id}
                          className="px-2 py-1 text-xs bg-gray-700 text-gray-300 rounded"
                        >
                          {role.name}
                        </span>
                      ))}
                    </div>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Model Selection */}
          <div>
            <label className="block text-sm font-medium text-gray-200 mb-3">
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
                        ? 'border-blue-500 bg-blue-500/10'
                        : 'border-gray-700 hover:border-gray-600 bg-gray-850'
                    }`}
                  >
                    <div className="text-2xl mb-1">{info.emoji}</div>
                    <div className="font-semibold text-sm text-gray-100 mb-1">
                      {info.label}
                    </div>
                    <div className="text-xs text-gray-400 leading-tight">
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
              className="flex items-center gap-2 text-sm text-gray-300 hover:text-gray-100 transition-colors"
            >
              <SettingsIcon size={16} />
              詳細設定を{showAdvanced ? '非表示' : '表示'}
            </button>

            {showAdvanced && (
              <div className="mt-4 p-4 bg-gray-900 rounded-lg border border-gray-700 space-y-4">
                {/* Timeout */}
                <div>
                  <label className="block text-xs font-medium text-gray-300 mb-2">
                    ラウンドタイムアウト（各ラウンドの制限時間）
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
                        setTimeoutSeconds(isNaN(value) ? 800 : value);
                      }}
                      className="flex-1 px-3 py-2 bg-gray-850 text-gray-100 border border-gray-700 rounded focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                    />
                    <span className="text-sm text-gray-400">
                      = {Math.floor(timeoutSeconds / 60)} 分
                    </span>
                  </div>
                  <div className="text-xs text-gray-500 mt-1">
                    推奨: 800秒（13.3分） = 3ラウンド合計 約40分
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
                    <label className="text-xs font-medium text-gray-300">
                      完了後もworktreeを保持
                    </label>
                    <div className="text-xs text-gray-500 mt-1">
                      ディベート完了後もGit worktreeと出力を保持し、後で確認できるようにします
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* Debate Flow Explanation */}
          <div className="p-4 bg-blue-500/10 border border-blue-500/30 rounded-lg">
            <div className="text-sm font-medium text-blue-300 mb-2">
              ディベートの流れ
            </div>
            <div className="text-xs text-gray-300 space-y-1">
              <div>📝 <strong>Round 1 (独立提案)</strong>: 各ロールが独立して提案</div>
              <div>🔍 <strong>Round 2 (批判的分析)</strong>: Round 1の提案を批判的に分析</div>
              <div>🤝 <strong>Round 3 (合意形成)</strong>: 統合された最終提案を作成</div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-6 py-4 border-t border-gray-700 bg-gray-850">
          <div className="text-sm text-gray-400">
            {!task.trim() ? (
              'ディベートテーマを入力してください'
            ) : (
              <>
                {currentPreset.roles.length} ロール × 3 ラウンド × {MODEL_INFO[selectedModel].label}
              </>
            )}
          </div>
          <div className="flex gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 text-gray-300 hover:text-gray-100 transition-colors"
            >
              キャンセル
            </button>
            <button
              onClick={handleStart}
              disabled={!task.trim() || isStarting}
              className="px-6 py-2 bg-gradient-to-r from-blue-600 to-cyan-600 hover:from-blue-500 hover:to-cyan-500 disabled:from-gray-700 disabled:to-gray-700 disabled:text-gray-500 text-white font-semibold rounded-lg transition-all"
            >
              {isStarting ? '起動中...' : '💬 ディベート開始'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
