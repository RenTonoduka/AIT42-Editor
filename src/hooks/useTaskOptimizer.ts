/**
 * useTaskOptimizer Hook - State management for task optimization workflow
 *
 * Manages async calls to Tauri backend for Ω-theory complexity analysis,
 * using Claude Code CLI for meta-analysis (self-analysis).
 *
 * @module hooks/useTaskOptimizer
 * @version 1.6.0-omega
 */

import { useState, useCallback } from 'react';
import { tauriApi, type ClaudeCodeAnalysisRequest } from '@/services/tauri';
import type { OptimizerState } from '@/types/optimizer';

// ============================================================================
// Hook Interface
// ============================================================================

interface UseTaskOptimizerReturn {
  /** Current optimizer state */
  state: OptimizerState;

  /** Analyze a task description (full workflow) */
  analyze: (description: string, currentSubtasks?: number) => Promise<void>;

  /** Reset to idle state */
  reset: () => void;

  /** Check if currently analyzing */
  isAnalyzing: boolean;

  /** Check if analysis completed successfully */
  isCompleted: boolean;

  /** Check if error occurred */
  hasError: boolean;
}

// ============================================================================
// Initial State
// ============================================================================

const initialState: OptimizerState = {
  status: 'idle',
  optimization: null,
  instances: null,
  complexityInfo: null,
  error: null,
  taskDescription: '',
};

// ============================================================================
// Hook Implementation
// ============================================================================

/**
 * Custom React hook for task optimization workflow
 *
 * **Workflow**:
 * 1. User calls `analyze(description)`
 * 2. Status: `idle` → `analyzing`
 * 3. Call `optimizeTask()` backend (LLM analysis)
 * 4. Call `calculateInstances()` backend
 * 5. Call `getComplexityInfo()` backend
 * 6. Status: `analyzing` → `calculated` | `error`
 *
 * **Performance**:
 * - First analysis: ~1-2s (LLM call)
 * - Subsequent analyses: ~1-2s (new LLM calls per task)
 *
 * **Error Handling**:
 * - Missing ANTHROPIC_API_KEY → user-friendly error
 * - Network timeout → retry suggestion
 * - Invalid input → validation error
 *
 * @returns Optimizer state and control functions
 *
 * @example
 * ```typescript
 * function MyComponent() {
 *   const { state, analyze, reset, isAnalyzing } = useTaskOptimizer();
 *
 *   const handleAnalyze = async () => {
 *     await analyze('Implement user authentication');
 *   };
 *
 *   if (isAnalyzing) return <Spinner />;
 *   if (state.status === 'calculated') {
 *     return (
 *       <div>
 *         <p>Complexity: {state.optimization.complexityClass}</p>
 *         <p>Subtasks: {state.optimization.recommendedSubtasks}</p>
 *       </div>
 *     );
 *   }
 * }
 * ```
 */
export function useTaskOptimizer(): UseTaskOptimizerReturn {
  const [state, setState] = useState<OptimizerState>(initialState);

  /**
   * Analyze task description using Claude Code (meta-analysis)
   */
  const analyze = useCallback(async (description: string, _currentSubtasks: number = 0) => {
    // Validation
    if (!description || description.trim().length === 0) {
      setState({
        ...initialState,
        status: 'error',
        error: 'Task description cannot be empty',
        taskDescription: description,
      });
      return;
    }

    // Start analysis
    setState({
      ...initialState,
      status: 'analyzing',
      taskDescription: description,
    });

    try {
      // Execute Claude Code meta-analysis
      const request: ClaudeCodeAnalysisRequest = {
        task: description.trim(),
        model: 'sonnet',  // Use Sonnet 4.5 for analysis
        timeoutSeconds: 120,
      };

      const response = await tauriApi.analyzeTaskWithClaudeCode(request);

      // Map complexity class to notation
      const notationMap: Record<string, string> = {
        'Logarithmic': 'Ω(log n)',
        'Linear': 'Ω(n)',
        'Quadratic': 'Ω(n²)',
        'Exponential': 'Ω(2ⁿ)',
      };

      // Success - update state with Claude Code analysis results
      setState({
        status: 'calculated',
        optimization: {
          complexityClass: response.complexityClass,
          recommendedSubtasks: response.recommendedSubtasks,
          confidence: response.confidence,
          reasoning: response.reasoning,
        },
        instances: {
          recommendedInstances: response.recommendedInstances,
          subtasksPerInstance: response.recommendedSubtasks / response.recommendedInstances,
          resourceConstrained: false,
          reasoning: `Claude Code分析に基づく推奨：${response.recommendedInstances}インスタンス`,
        },
        complexityInfo: {
          className: response.complexityClass,
          notation: notationMap[response.complexityClass] || 'Ω(n)',
          subtaskRange: getSubtaskRange(response.complexityClass),
          description: `${response.complexityClass}複雑度のタスク`,
          examples: [],
        },
        error: null,
        taskDescription: description,
      });
    } catch (error: any) {
      // Error - update state with user-friendly message
      setState({
        status: 'error',
        optimization: null,
        instances: null,
        complexityInfo: null,
        error: error.message || 'Claude Code分析に失敗しました',
        taskDescription: description,
      });
    }
  }, []);

  /**
   * Reset to idle state
   */
  const reset = useCallback(() => {
    setState(initialState);
  }, []);

  /**
   * Computed flags for convenience
   */
  const isAnalyzing = state.status === 'analyzing';
  const isCompleted = state.status === 'calculated';
  const hasError = state.status === 'error';

  return {
    state,
    analyze,
    reset,
    isAnalyzing,
    isCompleted,
    hasError,
  };
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Get subtask range for complexity class
 */
function getSubtaskRange(complexityClass: string): string {
  switch (complexityClass) {
    case 'Logarithmic':
      return '2-3';
    case 'Linear':
      return '3-5';
    case 'Quadratic':
      return '5-8';
    case 'Exponential':
      return '8-15';
    default:
      return '3-5';
  }
}

/**
 * Default export
 */
export default useTaskOptimizer;
