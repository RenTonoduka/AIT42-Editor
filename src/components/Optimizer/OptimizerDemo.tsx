/**
 * OptimizerDemo Component - Demo page for task optimizer
 *
 * Provides sample tasks, side-by-side comparison, and export functionality.
 *
 * @module components/Optimizer/OptimizerDemo
 * @version 1.6.0
 */

import React, { useState } from 'react';
import {
  Sparkles,
  Download,
  Copy,
  Check,
  Zap,
  Code,
  Database,
  Globe,
  Lock,
  ShoppingCart,
} from 'lucide-react';
import { TaskAnalyzer } from './TaskAnalyzer';
import type { OptimizerDemoProps } from '@/types/optimizer';
import type { OptimizerState } from '@/types/optimizer';

// ============================================================================
// Sample Tasks
// ============================================================================

interface SampleTaskItem {
  id: string;
  title: string;
  description: string;
  icon: React.ComponentType<{ className?: string }>;
  expectedComplexity: string;
}

const SAMPLE_TASKS: SampleTaskItem[] = [
  {
    id: 'auth',
    title: 'ユーザー認証',
    description: 'JWT ベースのユーザー認証システムをログイン、ログアウト、トークン更新機能付きで実装',
    icon: Lock,
    expectedComplexity: '線形',
  },
  {
    id: 'ecommerce',
    title: 'EC チェックアウト',
    description: 'カート管理、決済統合 (Stripe)、注文処理を含む EC チェックアウトフローを構築',
    icon: ShoppingCart,
    expectedComplexity: '二乗',
  },
  {
    id: 'api',
    title: 'REST API',
    description: 'CRUD 操作、認証、ページネーション機能を持つブログプラットフォーム用 REST API を作成',
    icon: Code,
    expectedComplexity: '線形',
  },
  {
    id: 'database',
    title: 'データベース移行',
    description: 'MySQL から PostgreSQL へ 50 以上のテーブルをデータ変換とインデックス作成を含めて移行',
    icon: Database,
    expectedComplexity: '二乗',
  },
  {
    id: 'landing',
    title: 'ランディングページ',
    description: 'ヒーローセクション、機能、testimonials、お問い合わせフォームを含むレスポンシブランディングページを構築',
    icon: Globe,
    expectedComplexity: '対数',
  },
  {
    id: 'microservices',
    title: 'マイクロサービスアーキテクチャ',
    description: '5 つ以上のサービス、API ゲートウェイ、サービス検出、メッセージキューを含むマイクロサービスアーキテクチャを設計・実装',
    icon: Zap,
    expectedComplexity: '指数',
  },
];

// ============================================================================
// Component
// ============================================================================

/**
 * OptimizerDemo Component
 *
 * **Features**:
 * - Sample tasks for quick testing
 * - Side-by-side input → output view
 * - Export results as JSON
 * - Copy to clipboard functionality
 * - Responsive grid layout
 *
 * @example
 * ```tsx
 * <OptimizerDemo />
 * ```
 */
export const OptimizerDemo: React.FC<OptimizerDemoProps> = ({
  sampleTasks,
  className = '',
}) => {
  const [selectedTask, setSelectedTask] = useState<string>('');
  const [lastResult, setLastResult] = useState<OptimizerState | null>(null);
  const [copied, setCopied] = useState(false);

  const tasks = sampleTasks?.map((desc, i) => ({
    id: `custom-${i}`,
    title: `Custom Task ${i + 1}`,
    description: desc,
    icon: Sparkles,
    expectedComplexity: 'Unknown',
  })) || SAMPLE_TASKS;

  // Handle sample task selection
  const handleSampleClick = (task: SampleTaskItem) => {
    setSelectedTask(task.description);
  };

  // Handle analysis complete
  const handleAnalysisComplete = (result: OptimizerState) => {
    setLastResult(result);
  };

  // Export results as JSON
  const handleExport = () => {
    if (!lastResult || !lastResult.optimization) return;

    const exportData = {
      timestamp: new Date().toISOString(),
      task: lastResult.taskDescription,
      complexity: lastResult.optimization.complexityClass,
      notation: lastResult.complexityInfo?.notation || '',
      subtasks: lastResult.optimization.recommendedSubtasks,
      instances: lastResult.instances?.recommendedInstances || 0,
      confidence: lastResult.optimization.confidence,
      reasoning: lastResult.optimization.reasoning,
    };

    const dataStr = JSON.stringify(exportData, null, 2);
    const dataUri = `data:application/json;charset=utf-8,${encodeURIComponent(dataStr)}`;

    const exportFileDefaultName = `task-analysis-${Date.now()}.json`;

    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();
  };

  // Copy to clipboard
  const handleCopy = async () => {
    if (!lastResult || !lastResult.optimization) return;

    const text = `Task: ${lastResult.taskDescription}
Complexity: ${lastResult.optimization.complexityClass} (${lastResult.complexityInfo?.notation || ''})
Subtasks: ${lastResult.optimization.recommendedSubtasks}
Instances: ${lastResult.instances?.recommendedInstances || 0}
Confidence: ${(lastResult.optimization.confidence * 100).toFixed(0)}%

Reasoning:
${lastResult.optimization.reasoning}`;

    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={`min-h-screen bg-gray-900 text-gray-100 p-6 ${className}`}>
      {/* Header */}
      <div className="max-w-7xl mx-auto mb-8">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <Sparkles className="w-8 h-8 text-blue-400" />
            <div>
              <h1 className="text-3xl font-bold text-white">Ω 理論タスク最適化</h1>
              <p className="text-gray-400">複雑度分析を用いたインテリジェントなサブタスク分解</p>
            </div>
          </div>

          {/* Export/Copy Buttons */}
          {lastResult && lastResult.optimization && (
            <div className="flex items-center gap-2">
              <button
                onClick={handleCopy}
                className="
                  px-3 py-2 rounded-md text-sm font-medium
                  bg-gray-800 hover:bg-gray-700 border border-gray-700
                  transition-colors flex items-center gap-2
                "
              >
                {copied ? (
                  <>
                    <Check className="w-4 h-4 text-green-400" />
                    コピー完了!
                  </>
                ) : (
                  <>
                    <Copy className="w-4 h-4" />
                    コピー
                  </>
                )}
              </button>
              <button
                onClick={handleExport}
                className="
                  px-3 py-2 rounded-md text-sm font-medium
                  bg-blue-600 hover:bg-blue-500 border border-blue-700
                  transition-colors flex items-center gap-2
                "
              >
                <Download className="w-4 h-4" />
                JSON 出力
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Main Content */}
      <div className="max-w-7xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Left Column: Sample Tasks */}
        <div>
          <div className="mb-4">
            <h2 className="text-xl font-semibold text-white mb-2">サンプルタスク</h2>
            <p className="text-sm text-gray-400">サンプルタスクをクリックして分析</p>
          </div>

          <div className="space-y-3">
            {tasks.map((task) => {
              const Icon = task.icon;
              return (
                <button
                  key={task.id}
                  onClick={() => handleSampleClick(task)}
                  className="
                    w-full p-4 bg-gray-800 border border-gray-700 rounded-lg
                    hover:bg-gray-750 hover:border-gray-600
                    transition-all duration-200
                    text-left group
                  "
                >
                  <div className="flex items-start gap-3">
                    <Icon className="w-5 h-5 text-blue-400 mt-0.5 flex-shrink-0 group-hover:scale-110 transition-transform" />
                    <div className="flex-1">
                      <div className="font-semibold text-white mb-1">{task.title}</div>
                      <div className="text-sm text-gray-400 leading-relaxed">{task.description}</div>
                      <div className="text-xs text-gray-500 mt-2">
                        予想: <span className="text-blue-400">{task.expectedComplexity}</span>
                      </div>
                    </div>
                  </div>
                </button>
              );
            })}
          </div>
        </div>

        {/* Right Column: Task Analyzer */}
        <div>
          <div className="mb-4">
            <h2 className="text-xl font-semibold text-white mb-2">タスク分析</h2>
            <p className="text-sm text-gray-400">タスクの説明を入力するか、サンプルを選択してください</p>
          </div>

          <TaskAnalyzer
            initialTask={selectedTask}
            key={selectedTask} // Force re-render when sample changes
            onAnalysisComplete={handleAnalysisComplete}
          />
        </div>
      </div>

      {/* Footer */}
      <div className="max-w-7xl mx-auto mt-8 pt-6 border-t border-gray-800">
        <div className="text-center text-sm text-gray-500">
          <p>AIT42-Editor v1.6.0 • Ω-Theory Task Optimizer</p>
          <p className="mt-1">Powered by Claude LLM and complexity analysis</p>
        </div>
      </div>
    </div>
  );
};

/**
 * Default export
 */
export default OptimizerDemo;
