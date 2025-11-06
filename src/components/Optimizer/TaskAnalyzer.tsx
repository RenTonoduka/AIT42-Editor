/**
 * TaskAnalyzer Component - Main UI for task optimization analysis
 *
 * Provides input, analysis, and results display for Ω-theory task optimization.
 *
 * @module components/Optimizer/TaskAnalyzer
 * @version 1.6.0
 */

import React, { useState, useCallback } from 'react';
import {
  FileText,
  Play,
  RotateCcw,
  Loader2,
  CheckCircle,
  AlertCircle,
  TrendingUp,
  Layers,
  Target,
} from 'lucide-react';
import { useTaskOptimizer } from '@/hooks/useTaskOptimizer';
import { ComplexityBadge } from './ComplexityBadge';
import { InstanceRecommendation } from './InstanceRecommendation';
import type { TaskAnalyzerProps } from '@/types/optimizer';

// ============================================================================
// Component
// ============================================================================

/**
 * TaskAnalyzer Component
 *
 * **Features**:
 * - Task description input (textarea)
 * - "Analyze Task" button with loading state
 * - Results display:
 *   - Complexity badge
 *   - Recommended subtasks
 *   - Recommended instances
 *   - Confidence score
 *   - Reasoning text
 * - Error handling with user-friendly messages
 * - Clear/reset functionality
 * - Responsive design (mobile, tablet, desktop)
 *
 * @example
 * ```tsx
 * <TaskAnalyzer
 *   initialTask="Build e-commerce checkout"
 *   onAnalysisComplete={(result) => console.log(result)}
 * />
 * ```
 */
export const TaskAnalyzer: React.FC<TaskAnalyzerProps> = ({
  initialTask = '',
  onAnalysisComplete,
  onError,
  className = '',
}) => {
  const [taskDescription, setTaskDescription] = useState(initialTask);
  const { state, analyze, reset, isAnalyzing, isCompleted, hasError } = useTaskOptimizer();

  // Handle analyze button click
  const handleAnalyze = useCallback(async () => {
    if (!taskDescription.trim()) {
      return;
    }

    await analyze(taskDescription, 0);

    // Notify parent
    if (state.status === 'calculated' && onAnalysisComplete) {
      onAnalysisComplete(state);
    }
    if (state.status === 'error' && onError) {
      onError(state.error || 'Unknown error');
    }
  }, [taskDescription, analyze, state, onAnalysisComplete, onError]);

  // Handle reset
  const handleReset = useCallback(() => {
    setTaskDescription('');
    reset();
  }, [reset]);

  // Handle Enter key (Cmd+Enter to analyze)
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        handleAnalyze();
      }
    },
    [handleAnalyze]
  );

  return (
    <div className={`bg-gray-800 border border-gray-700 rounded-lg shadow-lg ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 bg-gray-750 border-b border-gray-700">
        <div className="flex items-center gap-2">
          <FileText className="w-5 h-5 text-blue-400" />
          <h2 className="text-lg font-semibold text-white">Ω-Theory Task Analyzer</h2>
        </div>
        {isCompleted && (
          <button
            onClick={handleReset}
            className="
              px-3 py-1 text-sm text-gray-300 hover:text-white
              hover:bg-gray-700 rounded transition-colors
              flex items-center gap-1.5
            "
            aria-label="Reset analyzer"
          >
            <RotateCcw className="w-4 h-4" />
            Reset
          </button>
        )}
      </div>

      {/* Input Section */}
      <div className="p-4 space-y-4">
        <div>
          <label htmlFor="task-description" className="block text-sm font-medium text-gray-300 mb-2">
            Task Description
          </label>
          <textarea
            id="task-description"
            value={taskDescription}
            onChange={(e) => setTaskDescription(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Enter task description (e.g., 'Implement REST API for user management')"
            className="
              w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded-md
              text-gray-100 placeholder-gray-500
              focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent
              resize-none transition-all
            "
            rows={4}
            disabled={isAnalyzing}
            aria-describedby="task-description-hint"
          />
          <div id="task-description-hint" className="mt-1 text-xs text-gray-500">
            Press <kbd className="px-1 py-0.5 bg-gray-700 rounded">⌘</kbd> + <kbd className="px-1 py-0.5 bg-gray-700 rounded">Enter</kbd> to analyze
          </div>
        </div>

        {/* Analyze Button */}
        <button
          onClick={handleAnalyze}
          disabled={!taskDescription.trim() || isAnalyzing}
          className="
            w-full px-4 py-2.5 rounded-md font-medium text-white
            transition-all duration-200
            flex items-center justify-center gap-2
            disabled:opacity-50 disabled:cursor-not-allowed
            bg-gradient-to-r from-blue-600 to-purple-600
            hover:from-blue-500 hover:to-purple-500
            shadow-md hover:shadow-lg
          "
          aria-busy={isAnalyzing}
        >
          {isAnalyzing ? (
            <>
              <Loader2 className="w-5 h-5 animate-spin" />
              Analyzing Task...
            </>
          ) : (
            <>
              <Play className="w-5 h-5" />
              Analyze Task
            </>
          )}
        </button>
      </div>

      {/* Results Section */}
      {isCompleted && state.optimization && state.instances && state.complexityInfo && (
        <div className="p-4 pt-0 space-y-4">
          {/* Success Header */}
          <div className="flex items-center gap-2 text-green-400">
            <CheckCircle className="w-5 h-5" />
            <span className="font-semibold">Analysis Complete</span>
          </div>

          {/* Complexity Badge */}
          <div>
            <div className="text-sm font-medium text-gray-400 mb-2">Detected Complexity</div>
            <ComplexityBadge
              complexityClass={state.optimization.complexityClass}
              notation={state.complexityInfo.notation}
              size="lg"
              showTooltip={true}
            />
          </div>

          {/* Key Metrics Grid */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {/* Recommended Subtasks */}
            <div className="p-4 bg-gray-900 border border-gray-700 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Layers className="w-4 h-4 text-blue-400" />
                <span className="text-xs font-semibold text-gray-400 uppercase">Subtasks</span>
              </div>
              <div className="text-3xl font-bold text-blue-400">
                {state.optimization.recommendedSubtasks}
              </div>
              <div className="text-xs text-gray-500 mt-1">
                {state.complexityInfo.subtaskRange} range
              </div>
            </div>

            {/* Recommended Instances */}
            <div className="p-4 bg-gray-900 border border-gray-700 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Target className="w-4 h-4 text-purple-400" />
                <span className="text-xs font-semibold text-gray-400 uppercase">Instances</span>
              </div>
              <div className="text-3xl font-bold text-purple-400">
                {state.instances.recommendedInstances}
              </div>
              <div className="text-xs text-gray-500 mt-1">
                {state.instances.subtasksPerInstance.toFixed(2)} per instance
              </div>
            </div>

            {/* Confidence Score */}
            <div className="p-4 bg-gray-900 border border-gray-700 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <TrendingUp className="w-4 h-4 text-green-400" />
                <span className="text-xs font-semibold text-gray-400 uppercase">Confidence</span>
              </div>
              <div className="text-3xl font-bold text-green-400">
                {(state.optimization.confidence * 100).toFixed(0)}%
              </div>
              {/* Confidence Bar */}
              <div className="mt-2 h-1.5 bg-gray-700 rounded-full overflow-hidden">
                <div
                  className="h-full bg-green-500 transition-all duration-500"
                  style={{ width: `${state.optimization.confidence * 100}%` }}
                  role="progressbar"
                  aria-valuenow={state.optimization.confidence * 100}
                  aria-valuemin={0}
                  aria-valuemax={100}
                />
              </div>
            </div>
          </div>

          {/* Reasoning */}
          <div className="p-4 bg-gray-900 border border-gray-700 rounded-lg">
            <div className="text-sm font-semibold text-gray-400 uppercase mb-2">Analysis Reasoning</div>
            <p className="text-sm text-gray-300 leading-relaxed whitespace-pre-wrap">
              {state.optimization.reasoning}
            </p>
          </div>

          {/* Instance Recommendation */}
          <InstanceRecommendation
            instances={state.instances}
            recommendedSubtasks={state.optimization.recommendedSubtasks}
            complexityClass={state.optimization.complexityClass}
          />
        </div>
      )}

      {/* Error State */}
      {hasError && state.error && (
        <div className="p-4 pt-0">
          <div className="p-4 bg-red-900/20 border border-red-600/30 rounded-lg">
            <div className="flex items-start gap-3">
              <AlertCircle className="w-5 h-5 text-red-400 mt-0.5 flex-shrink-0" />
              <div>
                <div className="text-sm font-semibold text-red-400 mb-1">Analysis Failed</div>
                <p className="text-sm text-red-300 leading-relaxed">{state.error}</p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

/**
 * Default export
 */
export default TaskAnalyzer;
