/**
 * InstanceRecommendation Component - Display Claude Code instance recommendation
 *
 * Shows optimal instance count, subtasks per instance, and resource constraints.
 *
 * @module components/Optimizer/InstanceRecommendation
 * @version 1.6.0
 */

import React from 'react';
import { Cpu, Users, Layers, AlertCircle, Info } from 'lucide-react';
import type { InstanceRecommendationProps } from '@/types/optimizer';

// ============================================================================
// Component
// ============================================================================

/**
 * InstanceRecommendation Component
 *
 * **Features**:
 * - Visual instance count display with icons
 * - Subtasks per instance breakdown
 * - Resource constraint warning
 * - Reasoning tooltip
 * - Responsive design
 *
 * @example
 * ```tsx
 * <InstanceRecommendation
 *   instances={instanceResult}
 *   recommendedSubtasks={5}
 *   complexityClass="Linear"
 * />
 * ```
 */
export const InstanceRecommendation: React.FC<InstanceRecommendationProps> = ({
  instances,
  recommendedSubtasks,
  complexityClass,
  className = '',
}) => {
  const { recommendedInstances, subtasksPerInstance, resourceConstrained, reasoning } = instances;

  // Generate instance icons (max 10 for display)
  const instanceIcons = Array.from({ length: Math.min(recommendedInstances, 10) }, (_, i) => i);
  const hasMore = recommendedInstances > 10;

  return (
    <div className={`bg-gray-800 border border-gray-700 rounded-lg p-4 ${className}`}>
      {/* Header */}
      <div className="flex items-center gap-2 mb-4">
        <Cpu className="w-5 h-5 text-purple-400" />
        <h3 className="text-base font-semibold text-white">Instance Recommendation</h3>
      </div>

      {/* Instance Count Display */}
      <div className="mb-4">
        <div className="text-sm text-gray-400 mb-2">Recommended Claude Code Instances</div>
        <div className="flex items-center gap-3">
          {/* Numeric Display */}
          <div className="text-4xl font-bold text-purple-400">{recommendedInstances}</div>

          {/* Visual Icons */}
          <div className="flex-1">
            <div className="flex flex-wrap gap-1.5">
              {instanceIcons.map((i) => (
                <Users
                  key={i}
                  className="w-6 h-6 text-purple-400"
                  strokeWidth={2}
                  aria-hidden="true"
                />
              ))}
              {hasMore && (
                <span className="text-purple-400 text-lg font-semibold ml-1">
                  +{recommendedInstances - 10}
                </span>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Subtasks Per Instance */}
      <div className="mb-4 p-3 bg-gray-900 rounded-md">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            <Layers className="w-4 h-4 text-blue-400" />
            <span className="text-sm font-medium text-gray-300">Subtasks per Instance</span>
          </div>
          <span className="text-2xl font-bold text-blue-400">
            {subtasksPerInstance.toFixed(2)}
          </span>
        </div>

        {/* Breakdown */}
        <div className="text-xs text-gray-400">
          {recommendedSubtasks} subtasks ÷ {recommendedInstances} instances = {subtasksPerInstance.toFixed(2)} each
        </div>
      </div>

      {/* Resource Constraint Warning */}
      {resourceConstrained && (
        <div className="mb-4 p-3 bg-yellow-900/20 border border-yellow-600/30 rounded-md">
          <div className="flex items-start gap-2">
            <AlertCircle className="w-4 h-4 text-yellow-400 mt-0.5 flex-shrink-0" />
            <div>
              <div className="text-sm font-semibold text-yellow-400 mb-1">Resource Limited</div>
              <div className="text-xs text-yellow-300">
                Instance count capped at 10 (maximum recommended). Consider breaking down task further.
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Reasoning Tooltip */}
      <div className="p-3 bg-gray-900 rounded-md">
        <div className="flex items-start gap-2">
          <Info className="w-4 h-4 text-gray-400 mt-0.5 flex-shrink-0" />
          <div>
            <div className="text-xs font-semibold text-gray-400 uppercase mb-1">Strategy</div>
            <p className="text-sm text-gray-300 leading-relaxed">{reasoning}</p>
          </div>
        </div>
      </div>

      {/* Complexity Context */}
      <div className="mt-3 pt-3 border-t border-gray-700">
        <div className="text-xs text-gray-500">
          Optimized for <span className="font-semibold text-gray-400">{complexityClass}</span> complexity
        </div>
      </div>
    </div>
  );
};

/**
 * Default export
 */
export default InstanceRecommendation;
