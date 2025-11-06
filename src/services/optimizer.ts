/**
 * Subtask Optimizer API - Tauri command bindings for v1.6.0 optimizer backend
 *
 * This module provides TypeScript bindings for the Rust optimizer backend,
 * enabling intelligent subtask count optimization using Ω-theory and LLM analysis.
 *
 * @module services/optimizer
 * @version 1.6.0
 */

import { invoke } from '@tauri-apps/api/tauri';

// ============================================================================
// Type Definitions
// ============================================================================

/**
 * Response from optimize_task command
 *
 * Contains recommended subtask count based on complexity analysis.
 */
export interface OptimizeTaskResponse {
  /** Detected complexity class (e.g., "Linear", "Quadratic") */
  complexityClass: string;

  /** Recommended number of subtasks (1-15) */
  recommendedSubtasks: number;

  /** Confidence score from LLM analysis (0.0-1.0) */
  confidence: number;

  /** Human-readable reasoning for the recommendation */
  reasoning: string;

  /** Memory-based adjustment (v1.7.0+, may be null) */
  memoryAdjustment?: MemoryAdjustment;

  /** Original LLM estimate details (may be null) */
  llmEstimate?: ComplexityEstimate;
}

/**
 * Memory-based adjustment from historical data (v1.7.0+ placeholder)
 */
export interface MemoryAdjustment {
  /** Historical success rate for similar tasks (0.0-1.0) */
  historicalSuccessRate: number;

  /** Suggested adjustment to subtask count (-3 to +3) */
  adjustment: number;

  /** Reasoning for the adjustment */
  reasoning: string;
}

/**
 * LLM complexity estimate details
 */
export interface ComplexityEstimate {
  /** Complexity class detected by LLM */
  complexityClass: string;

  /** LLM confidence in the estimate (0.0-1.0) */
  confidence: number;

  /** LLM reasoning for classification */
  reasoning: string;

  /** LLM's suggested subtask count before Ω-theory validation */
  estimatedSubtasks: number;
}

/**
 * Response from calculate_instances command
 *
 * Determines optimal Claude Code instance count for parallel execution.
 */
export interface CalculateInstancesResponse {
  /** Recommended number of Claude Code instances (1-10) */
  recommendedInstances: number;

  /** Average subtasks per instance (may be fractional, e.g., 1.67) */
  subtasksPerInstance: number;

  /** True if calculation was limited by max_instances constraint (10) */
  resourceConstrained: boolean;

  /** Human-readable reasoning for the recommendation */
  reasoning: string;
}

/**
 * Response from get_complexity_info command
 *
 * Provides detailed information about a complexity class for UI tooltips.
 */
export interface ComplexityInfo {
  /** Complexity class name (e.g., "Linear") */
  className: string;

  /** Ω-notation (e.g., "Ω(n)", "Ω(n²)") */
  notation: string;

  /** Recommended subtask range (e.g., "3-5", "8-15") */
  subtaskRange: string;

  /** Human-readable description */
  description: string;

  /** Example use cases for this complexity class */
  examples: string[];
}

/**
 * Valid complexity class values
 */
export type ComplexityClass =
  | 'Constant'
  | 'Logarithmic'
  | 'Linear'
  | 'Linearithmic'
  | 'Quadratic'
  | 'Exponential';

// ============================================================================
// API Functions
// ============================================================================

/**
 * Optimize subtask count for a task
 *
 * Analyzes task description using Claude LLM and applies Ω-theory rules
 * to recommend optimal subtask decomposition.
 *
 * **Performance**:
 * - First call: ~1-2s (LLM API call)
 * - Cached calls: ~1-5ms
 *
 * **Errors**:
 * - Missing ANTHROPIC_API_KEY
 * - Network timeout
 * - Invalid task description (empty string)
 *
 * @param taskDescription - Task description to analyze (e.g., "Implement user authentication")
 * @param currentSubtasks - Current number of subtasks (0 if none, used for adjustment)
 * @returns Optimization recommendation with complexity class and reasoning
 *
 * @example
 * ```typescript
 * const result = await optimizeTask('Implement REST API for user management', 0);
 * console.log(`Complexity: ${result.complexityClass}`);
 * console.log(`Recommended subtasks: ${result.recommendedSubtasks}`);
 * console.log(`Reasoning: ${result.reasoning}`);
 * ```
 *
 * @throws {Error} If API key is missing, network fails, or input is invalid
 */
export async function optimizeTask(
  taskDescription: string,
  currentSubtasks: number = 0
): Promise<OptimizeTaskResponse> {
  if (!taskDescription || taskDescription.trim().length === 0) {
    throw new Error('Task description cannot be empty');
  }

  if (currentSubtasks < 0) {
    throw new Error('Current subtasks must be non-negative');
  }

  try {
    const response = await invoke<OptimizeTaskResponse>('optimize_task', {
      taskDescription,
      currentSubtasks,
    });
    return response;
  } catch (error) {
    // Re-throw with more context
    throw new Error(`Failed to optimize task: ${error}`);
  }
}

/**
 * Calculate optimal Claude Code instance count
 *
 * Given a complexity class and subtask count, determines how many parallel
 * Claude Code instances should be used for optimal throughput.
 *
 * **Formula**: `instances = ceil(subtasks / optimal_subtasks_per_instance)`
 *
 * **Performance**: <1ms (synchronous calculation, no I/O)
 *
 * **Strategy by Complexity**:
 * - Constant (Ω(1)): 1 subtask/instance (no parallelization benefit)
 * - Logarithmic (Ω(log n)): 1 subtask/instance (fast execution)
 * - Linear (Ω(n)): 1-2 subtasks/instance
 * - Linearithmic (Ω(n log n)): 2-3 subtasks/instance
 * - Quadratic (Ω(n²)): 3-5 subtasks/instance (aggressive batching)
 * - Exponential (Ω(2^n)): 5-8 subtasks/instance (maximum batching)
 *
 * @param complexityClass - Complexity class name (case-insensitive)
 * @param subtaskCount - Number of subtasks to execute (must be > 0)
 * @returns Instance count recommendation with reasoning
 *
 * @example
 * ```typescript
 * const result = await calculateInstances('Linear', 5);
 * console.log(`Use ${result.recommendedInstances} instances`);
 * console.log(`${result.subtasksPerInstance.toFixed(2)} subtasks per instance`);
 *
 * if (result.resourceConstrained) {
 *   console.log('Note: Limited to 10 instances maximum');
 * }
 * ```
 *
 * @throws {Error} If complexity class is invalid or subtask count is 0
 */
export async function calculateInstances(
  complexityClass: ComplexityClass | string,
  subtaskCount: number
): Promise<CalculateInstancesResponse> {
  if (subtaskCount <= 0) {
    throw new Error('Subtask count must be greater than 0');
  }

  try {
    const response = await invoke<CalculateInstancesResponse>('calculate_instances', {
      complexityClass,
      subtaskCount,
    });
    return response;
  } catch (error) {
    throw new Error(`Failed to calculate instances: ${error}`);
  }
}

/**
 * Get detailed information about a complexity class
 *
 * Provides Ω-notation, subtask ranges, descriptions, and examples
 * for displaying in UI tooltips or help dialogs.
 *
 * **Performance**: <1ms (constant-time lookup)
 *
 * @param complexityClass - Complexity class name (case-insensitive)
 * @returns Detailed complexity class information
 *
 * @example
 * ```typescript
 * const info = await getComplexityInfo('Linear');
 * console.log(`${info.notation}: ${info.description}`);
 * console.log(`Recommended subtasks: ${info.subtaskRange}`);
 * console.log('Examples:');
 * info.examples.forEach(ex => console.log(`  - ${ex}`));
 * ```
 *
 * @throws {Error} If complexity class is invalid
 */
export async function getComplexityInfo(
  complexityClass: ComplexityClass | string
): Promise<ComplexityInfo> {
  try {
    const response = await invoke<ComplexityInfo>('get_complexity_info', {
      complexityClass,
    });
    return response;
  } catch (error) {
    throw new Error(`Failed to get complexity info: ${error}`);
  }
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Validate that a string is a valid complexity class
 *
 * @param value - String to validate
 * @returns True if valid complexity class
 */
export function isValidComplexityClass(value: string): value is ComplexityClass {
  const validClasses: ComplexityClass[] = [
    'Constant',
    'Logarithmic',
    'Linear',
    'Linearithmic',
    'Quadratic',
    'Exponential',
  ];
  return validClasses.includes(value as ComplexityClass);
}

/**
 * Get all valid complexity class values
 *
 * @returns Array of valid complexity class names
 */
export function getAllComplexityClasses(): ComplexityClass[] {
  return ['Constant', 'Logarithmic', 'Linear', 'Linearithmic', 'Quadratic', 'Exponential'];
}

/**
 * Perform full task analysis (optimize + calculate instances)
 *
 * Convenience function that combines optimize_task and calculate_instances
 * into a single workflow.
 *
 * @param taskDescription - Task description to analyze
 * @param currentSubtasks - Current number of subtasks (0 if none)
 * @returns Combined optimization and instance calculation results
 *
 * @example
 * ```typescript
 * const analysis = await analyzeTask('Build e-commerce checkout flow');
 *
 * console.log(`Task: "${analysis.taskDescription}"`);
 * console.log(`Complexity: ${analysis.optimization.complexityClass}`);
 * console.log(`Subtasks: ${analysis.optimization.recommendedSubtasks}`);
 * console.log(`Instances: ${analysis.instances.recommendedInstances}`);
 * console.log(`Reasoning: ${analysis.optimization.reasoning}`);
 * ```
 */
export async function analyzeTask(
  taskDescription: string,
  currentSubtasks: number = 0
): Promise<{
  taskDescription: string;
  optimization: OptimizeTaskResponse;
  instances: CalculateInstancesResponse;
  complexityInfo: ComplexityInfo;
}> {
  // Step 1: Optimize subtask count
  const optimization = await optimizeTask(taskDescription, currentSubtasks);

  // Step 2: Calculate instance count
  const instances = await calculateInstances(
    optimization.complexityClass,
    optimization.recommendedSubtasks
  );

  // Step 3: Get complexity info
  const complexityInfo = await getComplexityInfo(optimization.complexityClass);

  return {
    taskDescription,
    optimization,
    instances,
    complexityInfo,
  };
}

// ============================================================================
// Exports
// ============================================================================

/**
 * Default export: All optimizer API functions
 */
export default {
  optimizeTask,
  calculateInstances,
  getComplexityInfo,
  analyzeTask,
  isValidComplexityClass,
  getAllComplexityClasses,
};
