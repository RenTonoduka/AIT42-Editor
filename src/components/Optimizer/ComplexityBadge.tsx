/**
 * ComplexityBadge Component - Visual indicator for Ω-theory complexity class
 *
 * Displays complexity class with color-coded badge, Ω-notation, and detailed tooltip.
 *
 * @module components/Optimizer/ComplexityBadge
 * @version 1.6.0
 */

import React, { useState, useEffect } from 'react';
import { Info, Zap, TrendingUp, Activity, AlertTriangle, Flame } from 'lucide-react';
import { getComplexityInfo, type ComplexityInfo } from '@/services/optimizer';
import type { ComplexityBadgeProps } from '@/types/optimizer';

// ============================================================================
// Complexity Color Mapping
// ============================================================================

/**
 * Get color scheme for complexity class
 *
 * **Color Strategy**:
 * - Green: Simple (Constant, Logarithmic) - 1-3 subtasks
 * - Blue: Moderate (Linear, Linearithmic) - 3-6 subtasks
 * - Yellow/Orange: Complex (Quadratic) - 5-10 subtasks
 * - Red: Very Complex (Exponential) - 8-15 subtasks
 */
function getComplexityColor(complexityClass: string): {
  bg: string;
  text: string;
  border: string;
  icon: React.ComponentType<{ className?: string }>;
} {
  const normalized = complexityClass.toLowerCase();

  if (normalized === 'constant') {
    return {
      bg: 'bg-green-500',
      text: 'text-white',
      border: 'border-green-600',
      icon: Zap,
    };
  }

  if (normalized === 'logarithmic') {
    return {
      bg: 'bg-green-600',
      text: 'text-white',
      border: 'border-green-700',
      icon: TrendingUp,
    };
  }

  if (normalized === 'linear') {
    return {
      bg: 'bg-blue-500',
      text: 'text-white',
      border: 'border-blue-600',
      icon: Activity,
    };
  }

  if (normalized === 'linearithmic') {
    return {
      bg: 'bg-blue-600',
      text: 'text-white',
      border: 'border-blue-700',
      icon: Activity,
    };
  }

  if (normalized === 'quadratic') {
    return {
      bg: 'bg-yellow-500',
      text: 'text-gray-900',
      border: 'border-yellow-600',
      icon: AlertTriangle,
    };
  }

  if (normalized === 'exponential') {
    return {
      bg: 'bg-red-500',
      text: 'text-white',
      border: 'border-red-600',
      icon: Flame,
    };
  }

  // Default fallback
  return {
    bg: 'bg-gray-500',
    text: 'text-white',
    border: 'border-gray-600',
    icon: Info,
  };
}

// ============================================================================
// Component
// ============================================================================

/**
 * ComplexityBadge Component
 *
 * **Features**:
 * - Color-coded badge (green → blue → yellow → red)
 * - Ω-notation display (e.g., "Ω(n)", "Ω(n²)")
 * - Icon representing complexity level
 * - Hover tooltip with detailed info
 * - Responsive sizing (sm, md, lg)
 * - Accessible (ARIA labels, keyboard navigation)
 *
 * @example
 * ```tsx
 * <ComplexityBadge
 *   complexityClass="Linear"
 *   size="md"
 *   showTooltip={true}
 * />
 * ```
 */
export const ComplexityBadge: React.FC<ComplexityBadgeProps> = ({
  complexityClass,
  notation,
  size = 'md',
  showTooltip = true,
  className = '',
}) => {
  const [complexityInfo, setComplexityInfo] = useState<ComplexityInfo | null>(null);
  const [showTooltipState, setShowTooltipState] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const colors = getComplexityColor(complexityClass);
  const Icon = colors.icon;

  // Fetch complexity info on mount (for tooltip)
  useEffect(() => {
    if (showTooltip) {
      setIsLoading(true);
      getComplexityInfo(complexityClass)
        .then(setComplexityInfo)
        .catch(console.error)
        .finally(() => setIsLoading(false));
    }
  }, [complexityClass, showTooltip]);

  // Size variants
  const sizeClasses = {
    sm: {
      badge: 'px-2 py-1 text-xs',
      icon: 'w-3 h-3',
      gap: 'gap-1',
    },
    md: {
      badge: 'px-3 py-1.5 text-sm',
      icon: 'w-4 h-4',
      gap: 'gap-1.5',
    },
    lg: {
      badge: 'px-4 py-2 text-base',
      icon: 'w-5 h-5',
      gap: 'gap-2',
    },
  }[size];

  return (
    <div className={`relative inline-block ${className}`}>
      {/* Badge */}
      <div
        className={`
          inline-flex items-center ${sizeClasses.gap} ${sizeClasses.badge}
          ${colors.bg} ${colors.text} border ${colors.border}
          rounded-full font-semibold shadow-md
          transition-all duration-200 hover:shadow-lg hover:scale-105
          cursor-default
        `}
        onMouseEnter={() => showTooltip && setShowTooltipState(true)}
        onMouseLeave={() => setShowTooltipState(false)}
        role="status"
        aria-label={`Complexity: ${complexityClass} ${notation || complexityInfo?.notation || ''}`}
      >
        <Icon className={sizeClasses.icon} aria-hidden="true" />
        <span>{notation || complexityInfo?.notation || `Ω(?)`}</span>
        <span className="font-bold">{complexityClass}</span>
      </div>

      {/* Tooltip */}
      {showTooltip && showTooltipState && complexityInfo && (
        <div
          className="
            absolute z-50 left-1/2 -translate-x-1/2 top-full mt-2
            w-80 p-4 bg-gray-800 border border-gray-700 rounded-lg shadow-xl
            text-sm text-gray-100
          "
          role="tooltip"
        >
          {/* Header */}
          <div className="flex items-center gap-2 mb-2 pb-2 border-b border-gray-700">
            <Icon className="w-5 h-5 text-blue-400" />
            <div>
              <div className="font-bold text-white">{complexityInfo.notation} {complexityInfo.className}</div>
              <div className="text-xs text-gray-400">Complexity Class</div>
            </div>
          </div>

          {/* Description */}
          <p className="text-gray-300 mb-3">{complexityInfo.description}</p>

          {/* Subtask Range */}
          <div className="mb-3">
            <div className="text-xs font-semibold text-gray-400 uppercase mb-1">Recommended Subtasks</div>
            <div className="text-lg font-bold text-white">{complexityInfo.subtaskRange}</div>
          </div>

          {/* Examples */}
          {complexityInfo.examples.length > 0 && (
            <div>
              <div className="text-xs font-semibold text-gray-400 uppercase mb-1">Example Use Cases</div>
              <ul className="space-y-1">
                {complexityInfo.examples.slice(0, 3).map((example, index) => (
                  <li key={index} className="text-xs text-gray-300 flex items-start gap-2">
                    <span className="text-blue-400 mt-0.5">•</span>
                    <span>{example}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* Loading state */}
          {isLoading && (
            <div className="text-xs text-gray-400 text-center py-2">Loading details...</div>
          )}
        </div>
      )}
    </div>
  );
};

/**
 * Default export
 */
export default ComplexityBadge;
