//! Subtask Count Optimizer Module
//!
//! This module provides intelligent subtask count optimization by combining
//! LLM-based complexity analysis with Ω-theory mathematical bounds.
//!
//! # Quick Start
//!
//! ```no_run
//! use optimizer::{SubtaskOptimizer, OptimizationResult};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create optimizer
//! let optimizer = SubtaskOptimizer::from_env()?;
//!
//! // Optimize a task
//! let result = optimizer
//!     .optimize_subtask_count("Implement user authentication", 0)
//!     .await?;
//!
//! println!("Complexity: {}", result.complexity_class);
//! println!("Recommended subtasks: {}", result.recommended_subtasks);
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! The optimizer uses a three-layer approach:
//!
//! 1. **LLM Analysis**: Claude analyzes task description for complexity patterns
//! 2. **Ω-Theory Validation**: Validates against mathematical complexity bounds
//! 3. **Memory Learning** (v1.7.0+): Adjusts based on historical success rates
//!
//! # Complexity Classes
//!
//! | Class | Notation | Subtasks | Examples |
//! |-------|----------|----------|----------|
//! | Constant | Ω(1) | 1 | Config changes, simple edits |
//! | Logarithmic | Ω(log n) | 2-3 | Binary search, tree operations |
//! | Linear | Ω(n) | 3-5 | CRUD operations, iterations |
//! | Linearithmic | Ω(n log n) | 4-6 | Sorting, indexing |
//! | Quadratic | Ω(n²) | 5-10 | Nested loops, matrix ops |
//! | Exponential | Ω(2^n) | 8-15 | Combinatorial, backtracking |

pub mod instance_calculator;
pub mod subtask_optimizer;

#[cfg(test)]
mod tests;

// Re-export main types
pub use instance_calculator::{InstanceCalculation, InstanceCalculator};
pub use subtask_optimizer::{
    MemoryAdjustment, OptimizerError, OptimizationResult, SubtaskOptimizer,
};

// Re-export types from dependencies for convenience
pub use llm_estimator::{ComplexityEstimate, EstimatorError};
pub use omega_theory::ComplexityClass;

#[cfg(test)]
mod module_tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all public types are accessible
        let _: SubtaskOptimizer;
        let _: OptimizationResult;
        let _: OptimizerError;
        let _: MemoryAdjustment;
        let _: ComplexityClass;
        let _: ComplexityEstimate;
        let _: InstanceCalculator;
        let _: InstanceCalculation;
    }
}
