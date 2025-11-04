import React, { useState } from 'react';
import { HelpCircle, Sparkles, Trophy, CheckCircle } from 'lucide-react';

type ModeType = 'collaborative' | 'competitive';

interface ModeTooltipProps {
  mode: ModeType;
}

export const ModeTooltip: React.FC<ModeTooltipProps> = ({ mode }) => {
  const [isVisible, setIsVisible] = useState(false);

  const content = mode === 'collaborative' ? {
    title: 'アンサンブルモード（Ensemble）',
    icon: <Sparkles size={20} className="text-purple-600" />,
    description: '複数のエージェントが同じタスクを並列実行し、統合AIが全結果を分析・統合して最適解を生成します。',
    useCases: [
      '複雑な問題の多角的解決',
      '異なるアプローチの集合知活用',
      '高品質な実装の自動選択'
    ],
    benefits: [
      '集合知による高品質な成果',
      '統合AIによる最適解の自動生成',
      '異なる視点からの解決策を統合'
    ],
    bgColor: 'bg-purple-50',
    borderColor: 'border-purple-300'
  } : {
    title: '競争モード（Competitive）',
    icon: <Trophy size={20} className="text-yellow-600" />,
    description: '同じタスクを複数のエージェントが並列実行し、最も優れた結果を選択します。',
    useCases: [
      '最適なアルゴリズム選択',
      '複数アプローチの比較検証',
      '品質重視の実装'
    ],
    benefits: [
      '高品質な成果物',
      '複数の解決策を比較',
      'ベストプラクティスの発見'
    ],
    bgColor: 'bg-yellow-50',
    borderColor: 'border-yellow-300'
  };

  return (
    <div className="relative inline-block">
      <button
        onMouseEnter={() => setIsVisible(true)}
        onMouseLeave={() => setIsVisible(false)}
        className="p-1 hover:bg-gray-100 rounded-full transition-colors"
        aria-label="モード説明"
      >
        <HelpCircle size={16} className="text-gray-500" />
      </button>

      {isVisible && (
        <div
          className={`absolute z-50 w-80 p-4 ${content.bgColor} border-2 ${content.borderColor} rounded-lg shadow-xl top-full mt-2 left-0`}
          style={{
            animation: 'fadeIn 0.2s ease-out',
            willChange: 'opacity, transform'
          }}
        >
          {/* Header */}
          <div className="flex items-center gap-2 mb-3">
            {content.icon}
            <h4 className="font-semibold text-sm text-gray-800">
              {content.title}
            </h4>
          </div>

          {/* Description */}
          <p className="text-xs text-gray-700 mb-3 leading-relaxed">
            {content.description}
          </p>

          {/* Use Cases */}
          <div className="mb-3">
            <div className="text-xs font-semibold text-gray-700 mb-1">
              適用例:
            </div>
            <ul className="space-y-1">
              {content.useCases.map((useCase, index) => (
                <li key={index} className="text-xs text-gray-600 flex items-start gap-1">
                  <span className="text-gray-400 mt-0.5">•</span>
                  <span>{useCase}</span>
                </li>
              ))}
            </ul>
          </div>

          {/* Benefits */}
          <div>
            <div className="text-xs font-semibold text-gray-700 mb-1">
              メリット:
            </div>
            <ul className="space-y-1">
              {content.benefits.map((benefit, index) => (
                <li key={index} className="text-xs text-gray-600 flex items-start gap-1">
                  <CheckCircle size={12} className="text-green-500 mt-0.5 flex-shrink-0" />
                  <span>{benefit}</span>
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}

      <style>{`
        @keyframes fadeIn {
          from {
            opacity: 0;
            transform: translateY(-8px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }
      `}</style>
    </div>
  );
};
