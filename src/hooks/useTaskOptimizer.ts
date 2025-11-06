/**
 * useTaskOptimizer Hook - State management for task optimization workflow
 *
 * Manages async calls to Tauri backend for Ω-theory complexity analysis,
 * subtask optimization, and instance calculation.
 *
 * @module hooks/useTaskOptimizer
 * @version 1.6.0
 */

import { useState, useCallback } from 'react';
import {
  optimizeTask,
  calculateInstances,
  getComplexityInfo,
  type OptimizeTaskResponse,
  type CalculateInstancesResponse,
  type ComplexityInfo,
} from '@/services/optimizer';
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
   * Analyze task description (full workflow)
   */
  const analyze = useCallback(async (description: string, currentSubtasks: number = 0) => {
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

    if (currentSubtasks < 0) {
      setState({
        ...initialState,
        status: 'error',
        error: 'Current subtasks must be non-negative',
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
      // Step 1: Optimize subtask count (LLM analysis)
      let optimization: OptimizeTaskResponse;
      try {
        optimization = await optimizeTask(description, currentSubtasks);
      } catch (error: any) {
        throw new Error(formatOptimizeError(error));
      }

      // Step 2: Calculate instance count
      let instances: CalculateInstancesResponse;
      try {
        instances = await calculateInstances(
          optimization.complexityClass,
          optimization.recommendedSubtasks
        );
      } catch (error: any) {
        throw new Error(`Failed to calculate instances: ${error.message || error}`);
      }

      // Step 3: Get complexity info
      let complexityInfo: ComplexityInfo;
      try {
        complexityInfo = await getComplexityInfo(optimization.complexityClass);
      } catch (error: any) {
        throw new Error(`Failed to get complexity info: ${error.message || error}`);
      }

      // Success - update state
      setState({
        status: 'calculated',
        optimization,
        instances,
        complexityInfo,
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
        error: error.message || 'Unknown error occurred',
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
 * Format optimize_task error into user-friendly message
 */
function formatOptimizeError(error: any): string {
  const errorStr = error.message || error.toString() || 'Unknown error';

  // API key missing
  if (errorStr.includes('ANTHROPIC_API_KEY') || errorStr.includes('API key')) {
    return 'ANTHROPIC_API_KEY environment variable is not set. Please configure it in your system or .env file.';
  }

  // Network timeout
  if (errorStr.includes('timeout') || errorStr.includes('network')) {
    return 'Network timeout - please check your internet connection and try again.';
  }

  // Invalid response
  if (errorStr.includes('Invalid') || errorStr.includes('parse')) {
    return 'Received invalid response from backend. Please try again or contact support.';
  }

  // Rate limit
  if (errorStr.includes('rate limit') || errorStr.includes('429')) {
    return 'API rate limit exceeded. Please wait a moment and try again.';
  }

  // Generic error
  return `Analysis failed: ${errorStr}`;
}

/**
 * Default export
 */
export default useTaskOptimizer;
