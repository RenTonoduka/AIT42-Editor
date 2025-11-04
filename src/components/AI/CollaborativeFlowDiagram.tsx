import React, { useEffect, useRef, useState } from 'react';
import { ArrowRight, CheckCircle, Sparkles } from 'lucide-react';

export const CollaborativeFlowDiagram: React.FC = () => {
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
    <div ref={ref} className="flex flex-col gap-4 p-6 bg-gradient-to-r from-purple-50 to-blue-50 rounded-lg border border-purple-200">
      <h3 className="text-sm font-semibold text-gray-800 mb-2 flex items-center gap-2">
        <Sparkles size={16} className="text-purple-600" />
        アンサンブルモードの仕組み
      </h3>

      <div className="flex items-center justify-center gap-6">
        {/* Left: Parallel Agents */}
        <div className="flex flex-col gap-3">
          {/* Agent 1 */}
          <div className="flex items-center gap-2">
            <div className={`w-14 h-14 rounded-lg bg-blue-500 text-white flex items-center justify-center font-bold shadow-md ${isVisible ? 'animate-pulse' : ''}`}>
              A
            </div>
            <span className="text-xs text-gray-600">同じタスク</span>
          </div>

          {/* Agent 2 */}
          <div className="flex items-center gap-2">
            <div className={`w-14 h-14 rounded-lg bg-green-500 text-white flex items-center justify-center font-bold shadow-md ${isVisible ? 'animate-pulse' : ''}`} style={{ animationDelay: '0.15s' }}>
              B
            </div>
            <span className="text-xs text-gray-600">同じタスク</span>
          </div>

          {/* Agent 3 */}
          <div className="flex items-center gap-2">
            <div className={`w-14 h-14 rounded-lg bg-teal-500 text-white flex items-center justify-center font-bold shadow-md ${isVisible ? 'animate-pulse' : ''}`} style={{ animationDelay: '0.3s' }}>
              C
            </div>
            <span className="text-xs text-gray-600">同じタスク</span>
          </div>
        </div>

        {/* Middle: Arrows indicating parallel execution */}
        <div className="flex flex-col items-center justify-center gap-2">
          <ArrowRight className={`text-purple-400 ${isVisible ? 'animate-pulse' : ''}`} size={24} />
          <ArrowRight className={`text-purple-400 ${isVisible ? 'animate-pulse' : ''}`} size={24} style={{ animationDelay: '0.15s' }} />
          <ArrowRight className={`text-purple-400 ${isVisible ? 'animate-pulse' : ''}`} size={24} style={{ animationDelay: '0.3s' }} />
        </div>

        {/* Right: Integration AI */}
        <div className="flex flex-col items-center">
          <div className="w-20 h-20 rounded-xl bg-gradient-to-br from-purple-600 to-blue-600 text-white flex items-center justify-center shadow-2xl">
            <Sparkles size={32} />
          </div>
          <span className="text-xs text-gray-700 mt-2 font-bold">統合AI</span>
        </div>

        {/* Arrow to final result */}
        <ArrowRight className={`text-blue-500 ${isVisible ? 'animate-bounce' : ''}`} size={24} style={{ animationDelay: '0.5s' }} />

        {/* Final Result */}
        <div className="flex flex-col items-center">
          <div className="w-16 h-16 rounded-lg bg-gradient-to-br from-green-500 to-emerald-600 text-white flex items-center justify-center shadow-lg">
            <CheckCircle size={28} />
          </div>
          <span className="text-xs text-gray-700 mt-2 font-semibold">最終結果</span>
        </div>
      </div>

      <p className="text-xs text-gray-600 text-center mt-2 leading-relaxed">
        <strong className="text-purple-700">並列実行:</strong> 各エージェントが<strong>同時に</strong>同じタスクに取り組み、
        異なる視点から解決策を生成 → <strong className="text-purple-700">統合AI</strong>が全結果を分析・統合
      </p>
    </div>
  );
};
