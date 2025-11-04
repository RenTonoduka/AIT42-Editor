import React, { useEffect, useRef, useState } from 'react';
import { GitBranch, Trophy } from 'lucide-react';

export const CompetitiveFlowDiagram: React.FC = () => {
  const [isVisible, setIsVisible] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  // IntersectionObserver でビューポート内にある場合のみアニメーション
  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => setIsVisible(entry.isIntersecting),
      { threshold: 0.1 }
    );

    if (ref.current) {
      observer.observe(ref.current);
    }

    return () => observer.disconnect();
  }, []);

  return (
    <div ref={ref} className="flex flex-col gap-4 p-6 bg-gradient-to-r from-purple-50 to-pink-50 rounded-lg border border-purple-200">
      <h3 className="text-sm font-semibold text-gray-800 mb-2">
        競争モードの仕組み
      </h3>

      <div className="flex items-center justify-center gap-4">
        {/* Single Task */}
        <div className="flex flex-col items-center">
          <div className="w-20 h-20 rounded-lg bg-purple-600 text-white flex items-center justify-center font-bold shadow-lg">
            <span className="text-sm text-center leading-tight">
              同一<br />タスク
            </span>
          </div>
        </div>

        {/* Branching */}
        <div className="flex flex-col gap-2">
          <GitBranch className="text-purple-400" size={24} />
          <span className="text-xs text-gray-500">並列実行</span>
        </div>

        {/* Multiple Agents */}
        <div className="flex flex-col gap-2">
          <div className={`w-12 h-12 rounded-lg bg-pink-500 text-white flex items-center justify-center font-bold shadow-md ${isVisible ? 'animate-pulse' : ''}`}>
            A
          </div>
          <div className={`w-12 h-12 rounded-lg bg-pink-500 text-white flex items-center justify-center font-bold shadow-md ${isVisible ? 'animate-pulse' : ''}`} style={{ animationDelay: '0.2s' }}>
            B
          </div>
          <div className={`w-12 h-12 rounded-lg bg-pink-500 text-white flex items-center justify-center font-bold shadow-md ${isVisible ? 'animate-pulse' : ''}`} style={{ animationDelay: '0.4s' }}>
            C
          </div>
        </div>

        {/* Evaluation */}
        <div className="flex flex-col items-center gap-2">
          <div className="text-purple-400 text-sm font-semibold">評価</div>
          <div className="h-12 border-l-2 border-purple-300" />
        </div>

        {/* Winner */}
        <div className="flex flex-col items-center">
          <div className="w-16 h-16 rounded-lg bg-gradient-to-br from-purple-600 to-pink-600 text-white flex items-center justify-center shadow-lg">
            <Trophy size={28} />
          </div>
          <span className="text-xs text-gray-600 mt-2 font-semibold">最良結果</span>
        </div>
      </div>

      <p className="text-xs text-gray-600 text-center mt-2">
        同じタスクを複数のエージェントが並列実行し、最も優れた結果を選択
      </p>
    </div>
  );
};
