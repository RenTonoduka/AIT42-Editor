import React, { useEffect, useRef, useState } from 'react';
import { MessageSquare, Users, CheckCircle, ArrowRight } from 'lucide-react';

export interface DebateFlowDiagramProps {
  currentRound?: number;
}

export const DebateFlowDiagram: React.FC<DebateFlowDiagramProps> = ({ currentRound = 0 }) => {
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

  const roles = [
    { id: 'architect', name: 'Architect', color: 'from-blue-500 to-blue-600' },
    { id: 'pragmatist', name: 'Pragmatist', color: 'from-green-500 to-green-600' },
    { id: 'innovator', name: 'Innovator', color: 'from-purple-500 to-purple-600' },
  ];

  const rounds = [
    { num: 1, name: '独立提案', desc: '各ロールが独自の視点を提示' },
    { num: 2, name: '批判的分析', desc: '他の提案を評価・改善' },
    { num: 3, name: 'コンセンサス', desc: '統合された最終提案' },
  ];

  return (
    <div
      ref={ref}
      className="flex flex-col gap-6 p-6 bg-gradient-to-r from-indigo-50 to-purple-50 rounded-lg border border-indigo-200"
    >
      <div className="flex items-center gap-2">
        <MessageSquare className="text-indigo-600" size={20} />
        <h3 className="text-sm font-semibold text-gray-800">協議モードの仕組み</h3>
      </div>

      {/* 3 Rounds Visualization */}
      <div className="flex items-center justify-between gap-6">
        {rounds.map((round, roundIdx) => (
          <React.Fragment key={round.num}>
            {/* Round Container */}
            <div className="flex-1 flex flex-col items-center">
              {/* Round Header */}
              <div
                className={`w-full mb-3 px-3 py-2 rounded-lg text-center transition-all ${
                  currentRound === round.num
                    ? 'bg-indigo-600 text-white shadow-lg scale-105'
                    : 'bg-white/80 text-gray-700 border border-indigo-200'
                }`}
              >
                <div className="text-xs font-semibold mb-1">Round {round.num}</div>
                <div className="text-xs font-medium">{round.name}</div>
              </div>

              {/* 3 Roles Network */}
              <div className="relative w-full h-32">
                <svg className="absolute inset-0 w-full h-full" style={{ zIndex: 0 }}>
                  {/* Connection lines for Round 2 and 3 (fully connected) */}
                  {round.num >= 2 &&
                    roles.map((role, i) =>
                      roles.slice(i + 1).map((targetRole, j) => {
                        const x1 = (i / 2) * 100 + 50;
                        const y1 = 50;
                        const x2 = ((i + j + 1) / 2) * 100 + 50;
                        const y2 = 50;
                        return (
                          <line
                            key={`${role.id}-${targetRole.id}`}
                            x1={`${x1}%`}
                            y1={`${y1}%`}
                            x2={`${x2}%`}
                            y2={`${y2}%`}
                            stroke="currentColor"
                            strokeWidth="2"
                            strokeDasharray="4"
                            className={`${
                              currentRound === round.num
                                ? 'text-indigo-400 opacity-100'
                                : 'text-indigo-200 opacity-40'
                            } transition-all`}
                            style={{
                              animation:
                                isVisible && currentRound === round.num
                                  ? 'dash 2s linear infinite'
                                  : 'none',
                            }}
                          />
                        );
                      })
                    )}
                </svg>

                {/* Role Nodes */}
                <div className="relative flex items-center justify-around h-full px-4" style={{ zIndex: 1 }}>
                  {roles.map((role, idx) => (
                    <div
                      key={role.id}
                      className={`flex flex-col items-center ${
                        isVisible && currentRound === round.num ? 'animate-pulse' : ''
                      }`}
                      style={{ animationDelay: `${idx * 0.2}s` }}
                    >
                      <div
                        className={`w-14 h-14 rounded-full bg-gradient-to-br ${role.color} text-white flex items-center justify-center shadow-lg font-bold text-sm ${
                          currentRound === round.num ? 'ring-4 ring-indigo-300' : ''
                        } transition-all`}
                      >
                        {role.name[0]}
                      </div>
                      <div className="text-xs text-gray-600 mt-1.5 font-medium">{role.name}</div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Round Description */}
              <p className="text-xs text-gray-500 text-center mt-2">{round.desc}</p>
            </div>

            {/* Arrow between rounds */}
            {roundIdx < rounds.length - 1 && (
              <div className="flex flex-col items-center justify-center">
                <ArrowRight
                  size={24}
                  className={`${
                    currentRound > round.num ? 'text-indigo-600' : 'text-indigo-300'
                  } transition-colors`}
                />
              </div>
            )}
          </React.Fragment>
        ))}
      </div>

      {/* Final Output */}
      <div className="flex items-center justify-center gap-4 pt-4 border-t border-indigo-200">
        <ArrowRight size={20} className="text-indigo-400" />
        <div className="flex items-center gap-2 px-4 py-2 bg-gradient-to-br from-indigo-600 to-purple-600 text-white rounded-lg shadow-lg">
          <CheckCircle size={20} />
          <span className="text-sm font-semibold">統合された最終提案</span>
        </div>
      </div>

      <p className="text-xs text-gray-600 text-center">
        3つの視点が3ラウンドで協議し、コンセンサスを形成
      </p>

      {/* Animation keyframes */}
      <style>{`
        @keyframes dash {
          to {
            stroke-dashoffset: -20;
          }
        }
      `}</style>
    </div>
  );
};
