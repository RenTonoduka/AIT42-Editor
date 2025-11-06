/**
 * Type definitions for Ω-theory task optimizer UI
 *
 * @module types/optimizer
 * @version 1.6.0
 */

import type {
  OptimizeTaskResponse,
  CalculateInstancesResponse,
  ComplexityInfo,
  ComplexityClass,
} from '@/services/optimizer';

// Re-export service types for convenience
export type {
  OptimizeTaskResponse,
  CalculateInstancesResponse,
  ComplexityInfo,
  ComplexityClass,
};

// ============================================================================
// UI State Types
// ============================================================================

/**
 * Optimizer workflow status
 */
export type OptimizerStatus = 'idle' | 'analyzing' | 'calculated' | 'error';

/**
 * Complete optimizer state
 */
export interface OptimizerState {
  /** Current workflow status */
  status: OptimizerStatus;

  /** Task optimization result (null until analyzed) */
  optimization: OptimizeTaskResponse | null;

  /** Instance calculation result (null until calculated) */
  instances: CalculateInstancesResponse | null;

  /** Complexity class detailed info (null until fetched) */
  complexityInfo: ComplexityInfo | null;

  /** Error message if status is 'error' */
  error: string | null;

  /** Task description being analyzed */
  taskDescription: string;
}

/**
 * Complexity class display configuration
 */
export interface ComplexityDisplay {
  /** Complexity class name */
  class: ComplexityClass;

  /** Display color (Tailwind color classes) */
  color: ComplexityColor;

  /** Ω-notation string */
  notation: string;

  /** Human-readable description */
  description: string;

  /** Icon name (from lucide-react) */
  icon: string;
}

/**
 * Color scheme for complexity classes
 */
export interface ComplexityColor {
  /** Background color (e.g., 'bg-green-500') */
  bg: string;

  /** Text color (e.g., 'text-green-100') */
  text: string;

  /** Border color (e.g., 'border-green-600') */
  border: string;

  /** Hover background (e.g., 'hover:bg-green-600') */
  hoverBg: string;

  /** Ring color for focus (e.g., 'ring-green-500') */
  ring: string;

  /** Gradient from (e.g., 'from-green-500') */
  gradientFrom: string;

  /** Gradient to (e.g., 'to-green-600') */
  gradientTo: string;
}

// ============================================================================
// Component Props Types
// ============================================================================

/**
 * Props for ComplexityBadge component
 */
export interface ComplexityBadgeProps {
  /** Complexity class name */
  complexityClass: ComplexityClass | string;

  /** Optional notation override (auto-fetched if not provided) */
  notation?: string;

  /** Optional size variant */
  size?: 'sm' | 'md' | 'lg';

  /** Show detailed tooltip on hover */
  showTooltip?: boolean;

  /** Additional CSS classes */
  className?: string;
}

/**
 * Props for TaskAnalyzer component
 */
export interface TaskAnalyzerProps {
  /** Initial task description (optional) */
  initialTask?: string;

  /** Callback when analysis completes */
  onAnalysisComplete?: (result: OptimizerState) => void;

  /** Callback when error occurs */
  onError?: (error: string) => void;

  /** Additional CSS classes */
  className?: string;
}

/**
 * Props for InstanceRecommendation component
 */
export interface InstanceRecommendationProps {
  /** Instance calculation result */
  instances: CalculateInstancesResponse;

  /** Recommended subtask count (for display) */
  recommendedSubtasks: number;

  /** Complexity class (for styling) */
  complexityClass: ComplexityClass | string;

  /** Additional CSS classes */
  className?: string;
}

/**
 * Props for OptimizerDemo component
 */
export interface OptimizerDemoProps {
  /** Sample tasks for quick testing */
  sampleTasks?: string[];

  /** Additional CSS classes */
  className?: string;
}

// ============================================================================
// Helper Types
// ============================================================================

/**
 * Sample task for demo/testing
 */
export interface SampleTask {
  /** Task title/label */
  title: string;

  /** Task description */
  description: string;

  /** Expected complexity (for validation) */
  expectedComplexity?: ComplexityClass;
}

/**
 * Analysis result export format
 */
export interface AnalysisExport {
  /** Timestamp of analysis */
  timestamp: string;

  /** Task description */
  task: string;

  /** Complexity class */
  complexity: string;

  /** Ω-notation */
  notation: string;

  /** Recommended subtasks */
  subtasks: number;

  /** Recommended instances */
  instances: number;

  /** Confidence score */
  confidence: number;

  /** Reasoning */
  reasoning: string;
}
