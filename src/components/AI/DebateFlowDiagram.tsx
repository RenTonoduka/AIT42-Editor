import React, { useEffect, useRef, useState } from 'react';
import { MessageSquare, CheckCircle } from 'lucide-react';

export interface DebateFlowDiagramProps {
  currentRound?: number;
}

export const DebateFlowDiagram: React.FC<DebateFlowDiagramProps> = ({ currentRound = 0 }) => {
  const [isVisible, setIsVisible] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

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
    { id: 'architect', name: 'Architect', color: '#3b82f6', label: 'A' },
    { id: 'pragmatist', name: 'Pragmatist', color: '#10b981', label: 'P' },
    { id: 'innovator', name: 'Innovator', color: '#a855f7', label: 'I' },
  ];

  // SVG dimensions
  const width = 600;
  const height = 500;
  const nodeRadius = 35;
  const layerSpacing = 160;

  // Node positions for each layer (round)
  const getNodePosition = (roundNum: number, roleIndex: number) => {
    const y = roundNum * layerSpacing;
    const totalNodes = roundNum === 3 ? 1 : 3;
    const startX = width / 2 - ((totalNodes - 1) * 120) / 2;
    const x = totalNodes === 1 ? width / 2 : startX + roleIndex * 120;
    return { x, y };
  };

  // Generate edges between rounds
  const generateEdges = () => {
    const edges = [];

    // Round 1 → Round 2 (3 → 3 = 9 edges)
    for (let i = 0; i < 3; i++) {
      for (let j = 0; j < 3; j++) {
        const from = getNodePosition(1, i);
        const to = getNodePosition(2, j);
        edges.push({
          id: `r1-${i}-r2-${j}`,
          from,
          to,
          active: currentRound >= 2,
          round: 2,
        });
      }
    }

    // Round 2 → Round 3 (3 → 1 = 3 edges)
    for (let i = 0; i < 3; i++) {
      const from = getNodePosition(2, i);
      const to = getNodePosition(3, 0);
      edges.push({
        id: `r2-${i}-r3`,
        from,
        to,
        active: currentRound >= 3,
        round: 3,
      });
    }

    return edges;
  };

  const edges = generateEdges();

  // Generate curved path for edge
  const getEdgePath = (from: { x: number; y: number }, to: { x: number; y: number }) => {
    const midY = (from.y + to.y) / 2;
    return `M ${from.x},${from.y} Q ${from.x},${midY} ${(from.x + to.x) / 2},${midY} T ${to.x},${to.y}`;
  };

  return (
    <div
      ref={ref}
      className="flex flex-col gap-4 p-6 bg-gradient-to-br from-slate-50 via-indigo-50 to-purple-50 rounded-lg border border-indigo-200"
    >
      {/* Header */}
      <div className="flex items-center gap-2">
        <MessageSquare className="text-indigo-600" size={20} />
        <h3 className="text-sm font-semibold text-gray-800">ニューラルネットワーク型協議モード</h3>
      </div>

      {/* Neural Network SVG */}
      <div className="flex justify-center">
        <svg
          width={width}
          height={height}
          viewBox={`0 0 ${width} ${height}`}
          className="drop-shadow-sm"
        >
          <defs>
            {/* Gradient for active edges */}
            <linearGradient id="activeGradient" x1="0%" y1="0%" x2="0%" y2="100%">
              <stop offset="0%" stopColor="#818cf8" stopOpacity="0.8" />
              <stop offset="100%" stopColor="#c084fc" stopOpacity="0.8" />
            </linearGradient>

            {/* Arrow marker */}
            <marker
              id="arrowhead"
              markerWidth="10"
              markerHeight="10"
              refX="9"
              refY="3"
              orient="auto"
              markerUnits="strokeWidth"
            >
              <path d="M0,0 L0,6 L9,3 z" fill="#818cf8" />
            </marker>

            {/* Glow filter for active nodes */}
            <filter id="glow">
              <feGaussianBlur stdDeviation="4" result="coloredBlur" />
              <feMerge>
                <feMergeNode in="coloredBlur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>

          {/* Edges */}
          {edges.map((edge) => (
            <g key={edge.id}>
              <path
                d={getEdgePath(edge.from, edge.to)}
                fill="none"
                stroke={edge.active ? 'url(#activeGradient)' : '#e0e7ff'}
                strokeWidth={edge.active ? '3' : '2'}
                strokeOpacity={edge.active ? 1 : 0.4}
                markerEnd={edge.active ? 'url(#arrowhead)' : undefined}
                className="transition-all duration-500"
                style={{
                  strokeDasharray: edge.active && isVisible ? '8, 8' : 'none',
                  animation:
                    edge.active && isVisible && currentRound === edge.round
                      ? 'dash 2s linear infinite'
                      : 'none',
                }}
              />
            </g>
          ))}

          {/* Round 1 Nodes (Input Layer) */}
          <g>
            <text
              x={width / 2}
              y={getNodePosition(1, 0).y - 60}
              textAnchor="middle"
              className="text-xs font-semibold fill-gray-600"
            >
              Round 1: 独立提案 (Input Layer)
            </text>
            {roles.map((role, idx) => {
              const pos = getNodePosition(1, idx);
              const isActive = currentRound >= 1;
              return (
                <g key={`r1-${role.id}`}>
                  <circle
                    cx={pos.x}
                    cy={pos.y}
                    r={nodeRadius}
                    fill={role.color}
                    opacity={isActive ? 1 : 0.4}
                    filter={isActive && currentRound === 1 ? 'url(#glow)' : undefined}
                    className={`transition-all duration-500 ${
                      isActive && currentRound === 1 ? 'animate-pulse' : ''
                    }`}
                    style={{ animationDelay: `${idx * 0.15}s` }}
                  />
                  <text
                    x={pos.x}
                    y={pos.y}
                    textAnchor="middle"
                    dominantBaseline="central"
                    className="text-xl font-bold fill-white pointer-events-none"
                  >
                    {role.label}
                  </text>
                  <text
                    x={pos.x}
                    y={pos.y + nodeRadius + 18}
                    textAnchor="middle"
                    className="text-xs font-medium fill-gray-700"
                  >
                    {role.name}
                  </text>
                </g>
              );
            })}
          </g>

          {/* Round 2 Nodes (Hidden Layer - Fully Connected) */}
          <g>
            <text
              x={width / 2}
              y={getNodePosition(2, 0).y - 60}
              textAnchor="middle"
              className="text-xs font-semibold fill-gray-600"
            >
              Round 2: 批判的分析 (Hidden Layer - 全結合)
            </text>
            {roles.map((role, idx) => {
              const pos = getNodePosition(2, idx);
              const isActive = currentRound >= 2;
              return (
                <g key={`r2-${role.id}`}>
                  <circle
                    cx={pos.x}
                    cy={pos.y}
                    r={nodeRadius}
                    fill={role.color}
                    opacity={isActive ? 1 : 0.4}
                    filter={isActive && currentRound === 2 ? 'url(#glow)' : undefined}
                    className={`transition-all duration-500 ${
                      isActive && currentRound === 2 ? 'animate-pulse' : ''
                    }`}
                    style={{ animationDelay: `${idx * 0.15}s` }}
                  />
                  <text
                    x={pos.x}
                    y={pos.y}
                    textAnchor="middle"
                    dominantBaseline="central"
                    className="text-xl font-bold fill-white pointer-events-none"
                  >
                    {role.label}
                  </text>
                  <text
                    x={pos.x}
                    y={pos.y + nodeRadius + 18}
                    textAnchor="middle"
                    className="text-xs font-medium fill-gray-700"
                  >
                    {role.name}
                  </text>
                </g>
              );
            })}
          </g>

          {/* Round 3 Node (Output Layer) */}
          <g>
            <text
              x={width / 2}
              y={getNodePosition(3, 0).y - 60}
              textAnchor="middle"
              className="text-xs font-semibold fill-gray-600"
            >
              Round 3: コンセンサス形成 (Output Layer)
            </text>
            {(() => {
              const pos = getNodePosition(3, 0);
              const isActive = currentRound >= 3;
              return (
                <g>
                  <circle
                    cx={pos.x}
                    cy={pos.y}
                    r={nodeRadius + 5}
                    fill="url(#activeGradient)"
                    opacity={isActive ? 1 : 0.4}
                    filter={isActive && currentRound === 3 ? 'url(#glow)' : undefined}
                    className={`transition-all duration-500 ${
                      isActive && currentRound === 3 ? 'animate-pulse' : ''
                    }`}
                  />
                  <CheckCircle
                    x={pos.x - 16}
                    y={pos.y - 16}
                    width={32}
                    height={32}
                    className="fill-white"
                  />
                  <text
                    x={pos.x}
                    y={pos.y + nodeRadius + 25}
                    textAnchor="middle"
                    className="text-sm font-bold fill-gray-800"
                  >
                    統合された最終提案
                  </text>
                </g>
              );
            })()}
          </g>
        </svg>
      </div>

      {/* Description */}
      <p className="text-xs text-gray-600 text-center">
        3つのロールが3層のネットワークで協議し、統合されたコンセンサスを形成
      </p>

      {/* Animation keyframes */}
      <style>{`
        @keyframes dash {
          to {
            stroke-dashoffset: -16;
          }
        }
      `}</style>
    </div>
  );
};
