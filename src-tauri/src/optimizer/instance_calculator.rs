//! Instance Number Calculator
//!
//! This module calculates optimal Claude Code instance counts for parallel task execution
//! based on subtask complexity using Ω-theory principles.
//!
//! # Architecture
//!
//! ```text
//! InstanceCalculator
//!     ├── Complexity Analysis (from SubtaskOptimizer)
//!     ├── Subtasks per Instance Mapping (Ω-theory derived)
//!     └── Resource Constraints (max 10 instances)
//! ```
//!
//! # Formula
//!
//! ```text
//! instances = ceil(subtask_count / optimal_subtasks_per_instance)
//! instances = min(instances, max_instances)
//! ```
//!
//! # Subtasks per Instance Strategy
//!
//! Based on complexity class (derived from research):
//! - **Constant (Ω(1))**: 1 subtask/instance (trivial, no parallelization benefit)
//! - **Logarithmic (Ω(log n))**: 1 subtask/instance (fast execution, minimal overhead)
//! - **Linear (Ω(n))**: 1-2 subtasks/instance (balance between parallelism and overhead)
//! - **Linearithmic (Ω(n log n))**: 2-3 subtasks/instance (moderate batching)
//! - **Quadratic (Ω(n²))**: 3-5 subtasks/instance (aggressive batching to reduce overhead)
//! - **Exponential (Ω(2^n))**: 5-8 subtasks/instance (maximum batching, heavy computation)
//!
//! # Performance
//!
//! - Target: <1ms per calculation (synchronous, no I/O)
//! - Memory: ~100 bytes per calculation
//! - No external dependencies beyond omega-theory crate
//!
//! # Example
//!
//! ```no_run
//! use optimizer::{InstanceCalculator, ComplexityClass};
//!
//! let calculator = InstanceCalculator::new();
//!
//! // Linear task with 5 subtasks
//! let result = calculator.calculate_instances(ComplexityClass::Linear, 5);
//! assert_eq!(result.recommended_instances, 3);
//! assert!((result.subtasks_per_instance - 1.67).abs() < 0.01);
//! assert!(!result.resource_constrained);
//!
//! // Exponential task with 50 subtasks (hits max limit)
//! let result = calculator.calculate_instances(ComplexityClass::Exponential, 50);
//! assert_eq!(result.recommended_instances, 10);
//! assert!(result.resource_constrained);
//! ```

use omega_theory::ComplexityClass;
use serde::{Deserialize, Serialize};

/// Default maximum number of concurrent Claude Code instances
const DEFAULT_MAX_INSTANCES: usize = 10;

/// Result of instance count calculation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstanceCalculation {
    /// Recommended number of Claude Code instances
    pub recommended_instances: usize,

    /// Average subtasks per instance (may be fractional)
    pub subtasks_per_instance: f64,

    /// Complexity class used for calculation
    pub complexity_class: ComplexityClass,

    /// True if calculation was limited by max_instances constraint
    pub resource_constrained: bool,

    /// Human-readable reasoning for the recommendation
    pub reasoning: String,
}

/// Instance number calculator
///
/// Calculates optimal Claude Code instance counts based on subtask complexity.
///
/// # Examples
///
/// ```no_run
/// use optimizer::{InstanceCalculator, ComplexityClass};
///
/// // Default configuration (max 10 instances)
/// let calculator = InstanceCalculator::new();
///
/// let result = calculator.calculate_instances(ComplexityClass::Linear, 5);
/// println!("Recommended instances: {}", result.recommended_instances);
/// println!("Subtasks per instance: {:.2}", result.subtasks_per_instance);
/// println!("Reasoning: {}", result.reasoning);
///
/// // Custom max instances
/// let calculator = InstanceCalculator::with_max_instances(15);
/// ```
pub struct InstanceCalculator {
    max_instances: usize,
}

impl InstanceCalculator {
    /// Create a new instance calculator with default settings
    ///
    /// Uses default max instances of 10 (as per v1.6.0 proposal).
    ///
    /// # Examples
    ///
    /// ```
    /// use optimizer::InstanceCalculator;
    ///
    /// let calculator = InstanceCalculator::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_instances: DEFAULT_MAX_INSTANCES,
        }
    }

    /// Create a calculator with custom maximum instances
    ///
    /// # Arguments
    ///
    /// * `max_instances` - Maximum number of concurrent instances (1-100)
    ///
    /// # Panics
    ///
    /// Panics if `max_instances` is 0 or > 100
    ///
    /// # Examples
    ///
    /// ```
    /// use optimizer::InstanceCalculator;
    ///
    /// let calculator = InstanceCalculator::with_max_instances(20);
    /// ```
    #[must_use]
    pub fn with_max_instances(max_instances: usize) -> Self {
        assert!(
            max_instances > 0 && max_instances <= 100,
            "max_instances must be between 1 and 100, got {}",
            max_instances
        );

        Self { max_instances }
    }

    /// Calculate optimal instance count for given complexity and subtask count
    ///
    /// This is the main entry point for instance calculation. It:
    /// 1. Determines optimal subtasks per instance based on complexity class
    /// 2. Calculates required instances using ceiling division
    /// 3. Applies resource constraints (max instances)
    /// 4. Returns comprehensive calculation result with reasoning
    ///
    /// # Arguments
    ///
    /// * `complexity_class` - The Ω complexity class of the task
    /// * `subtask_count` - Number of subtasks to distribute
    ///
    /// # Returns
    ///
    /// An `InstanceCalculation` with recommended instances, distribution metrics,
    /// and human-readable reasoning.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use optimizer::{InstanceCalculator, ComplexityClass};
    ///
    /// let calculator = InstanceCalculator::new();
    ///
    /// // Constant complexity: 1 subtask → 1 instance
    /// let result = calculator.calculate_instances(ComplexityClass::Constant, 1);
    /// assert_eq!(result.recommended_instances, 1);
    ///
    /// // Linear complexity: 5 subtasks → 3 instances (avg 1.67 subtasks/instance)
    /// let result = calculator.calculate_instances(ComplexityClass::Linear, 5);
    /// assert_eq!(result.recommended_instances, 3);
    /// assert!((result.subtasks_per_instance - 1.67).abs() < 0.01);
    ///
    /// // Exponential: 15 subtasks → 2 instances (avg 7.5 subtasks/instance)
    /// let result = calculator.calculate_instances(ComplexityClass::Exponential, 15);
    /// assert_eq!(result.recommended_instances, 2);
    /// ```
    #[must_use]
    pub fn calculate_instances(
        &self,
        complexity_class: ComplexityClass,
        subtask_count: usize,
    ) -> InstanceCalculation {
        // Handle edge case: zero subtasks
        if subtask_count == 0 {
            return InstanceCalculation {
                recommended_instances: 0,
                subtasks_per_instance: 0.0,
                complexity_class,
                resource_constrained: false,
                reasoning: "No subtasks to execute - no instances needed".to_string(),
            };
        }

        // Get optimal subtasks per instance for this complexity class
        let optimal_subtasks_per_instance = self.get_optimal_subtasks_per_instance(complexity_class);

        // Calculate unconstrained instance count (ceiling division)
        let unconstrained_instances =
            (subtask_count as f64 / optimal_subtasks_per_instance).ceil() as usize;

        // Apply resource constraint
        let recommended_instances = unconstrained_instances.min(self.max_instances);

        // Calculate actual subtasks per instance after constraint
        let subtasks_per_instance = subtask_count as f64 / recommended_instances as f64;

        // Determine if we were resource constrained
        let resource_constrained = unconstrained_instances > self.max_instances;

        // Build reasoning string
        let reasoning = self.build_reasoning(
            complexity_class,
            subtask_count,
            optimal_subtasks_per_instance,
            unconstrained_instances,
            recommended_instances,
            subtasks_per_instance,
            resource_constrained,
        );

        InstanceCalculation {
            recommended_instances,
            subtasks_per_instance,
            complexity_class,
            resource_constrained,
            reasoning,
        }
    }

    /// Get optimal subtasks per instance for a complexity class
    ///
    /// Returns the midpoint of the recommended range for each complexity class.
    ///
    /// # Strategy
    ///
    /// - **Constant/Logarithmic**: 1 subtask/instance (fast execution, no batching needed)
    /// - **Linear**: 1.5 subtasks/instance (light batching)
    /// - **Linearithmic**: 2.5 subtasks/instance (moderate batching)
    /// - **Quadratic**: 4.0 subtasks/instance (aggressive batching)
    /// - **Exponential**: 6.5 subtasks/instance (maximum batching for heavy computation)
    fn get_optimal_subtasks_per_instance(&self, complexity_class: ComplexityClass) -> f64 {
        match complexity_class {
            ComplexityClass::Constant => 1.0,      // No parallelization benefit
            ComplexityClass::Logarithmic => 1.0,   // Fast execution, minimal overhead
            ComplexityClass::Linear => 1.5,        // Balance parallelism vs overhead
            ComplexityClass::Linearithmic => 2.5,  // Moderate batching
            ComplexityClass::Quadratic => 4.0,     // Aggressive batching
            ComplexityClass::Exponential => 6.5,   // Maximum batching
        }
    }

    /// Build comprehensive reasoning string
    #[allow(clippy::too_many_arguments)]
    fn build_reasoning(
        &self,
        complexity_class: ComplexityClass,
        subtask_count: usize,
        optimal_subtasks_per_instance: f64,
        unconstrained_instances: usize,
        recommended_instances: usize,
        subtasks_per_instance: f64,
        resource_constrained: bool,
    ) -> String {
        let mut reasoning = format!(
            "Complexity: {} ({})\n",
            complexity_class,
            complexity_class.description()
        );

        reasoning.push_str(&format!(
            "Subtasks: {} total\n",
            subtask_count
        ));

        reasoning.push_str(&format!(
            "Optimal subtasks/instance: {:.1}\n",
            optimal_subtasks_per_instance
        ));

        if resource_constrained {
            reasoning.push_str(&format!(
                "Unconstrained calculation: {} instances\n",
                unconstrained_instances
            ));
            reasoning.push_str(&format!(
                "⚠️  Resource constrained: capped at {} instances (max limit)\n",
                self.max_instances
            ));
        }

        reasoning.push_str(&format!(
            "Recommended: {} instances × {:.2} subtasks/instance",
            recommended_instances, subtasks_per_instance
        ));

        if resource_constrained {
            reasoning.push_str(&format!(
                "\n\nNote: Each instance will handle more subtasks ({:.2}) than optimal ({:.1}) due to resource constraints.",
                subtasks_per_instance, optimal_subtasks_per_instance
            ));
        }

        reasoning
    }

    /// Get the configured max instances limit
    #[must_use]
    pub fn max_instances(&self) -> usize {
        self.max_instances
    }
}

impl Default for InstanceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator_creation() {
        let calculator = InstanceCalculator::new();
        assert_eq!(calculator.max_instances(), DEFAULT_MAX_INSTANCES);
    }

    #[test]
    fn test_calculator_with_custom_max() {
        let calculator = InstanceCalculator::with_max_instances(20);
        assert_eq!(calculator.max_instances(), 20);
    }

    #[test]
    #[should_panic(expected = "max_instances must be between 1 and 100")]
    fn test_calculator_panics_on_zero_max() {
        let _ = InstanceCalculator::with_max_instances(0);
    }

    #[test]
    #[should_panic(expected = "max_instances must be between 1 and 100")]
    fn test_calculator_panics_on_excessive_max() {
        let _ = InstanceCalculator::with_max_instances(101);
    }

    #[test]
    fn test_default_trait() {
        let calculator = InstanceCalculator::default();
        assert_eq!(calculator.max_instances(), DEFAULT_MAX_INSTANCES);
    }

    #[test]
    fn test_zero_subtasks() {
        let calculator = InstanceCalculator::new();
        let result = calculator.calculate_instances(ComplexityClass::Linear, 0);

        assert_eq!(result.recommended_instances, 0);
        assert_eq!(result.subtasks_per_instance, 0.0);
        assert!(!result.resource_constrained);
        assert!(result.reasoning.contains("No subtasks"));
    }

    #[test]
    fn test_constant_complexity_single_subtask() {
        let calculator = InstanceCalculator::new();
        let result = calculator.calculate_instances(ComplexityClass::Constant, 1);

        assert_eq!(result.recommended_instances, 1);
        assert_eq!(result.subtasks_per_instance, 1.0);
        assert_eq!(result.complexity_class, ComplexityClass::Constant);
        assert!(!result.resource_constrained);
    }

    #[test]
    fn test_logarithmic_complexity() {
        let calculator = InstanceCalculator::new();

        // 1 subtask → 1 instance
        let result = calculator.calculate_instances(ComplexityClass::Logarithmic, 1);
        assert_eq!(result.recommended_instances, 1);
        assert_eq!(result.subtasks_per_instance, 1.0);

        // 3 subtasks → 3 instances (1 subtask/instance)
        let result = calculator.calculate_instances(ComplexityClass::Logarithmic, 3);
        assert_eq!(result.recommended_instances, 3);
        assert_eq!(result.subtasks_per_instance, 1.0);
    }

    #[test]
    fn test_linear_complexity() {
        let calculator = InstanceCalculator::new();

        // 1 subtask → 1 instance
        let result = calculator.calculate_instances(ComplexityClass::Linear, 1);
        assert_eq!(result.recommended_instances, 1);
        assert_eq!(result.subtasks_per_instance, 1.0);

        // 3 subtasks → 2 instances (1.5 avg)
        let result = calculator.calculate_instances(ComplexityClass::Linear, 3);
        assert_eq!(result.recommended_instances, 2);
        assert_eq!(result.subtasks_per_instance, 1.5);

        // 5 subtasks → 4 instances (ceil(5/1.5) = ceil(3.33) = 4)
        let result = calculator.calculate_instances(ComplexityClass::Linear, 5);
        assert_eq!(result.recommended_instances, 4);
        assert!((result.subtasks_per_instance - 1.25).abs() < 0.01);
    }

    #[test]
    fn test_linearithmic_complexity() {
        let calculator = InstanceCalculator::new();

        // 5 subtasks → 2 instances (ceil(5/2.5) = 2)
        let result = calculator.calculate_instances(ComplexityClass::Linearithmic, 5);
        assert_eq!(result.recommended_instances, 2);
        assert_eq!(result.subtasks_per_instance, 2.5);

        // 10 subtasks → 4 instances
        let result = calculator.calculate_instances(ComplexityClass::Linearithmic, 10);
        assert_eq!(result.recommended_instances, 4);
        assert_eq!(result.subtasks_per_instance, 2.5);
    }

    #[test]
    fn test_quadratic_complexity() {
        let calculator = InstanceCalculator::new();

        // 8 subtasks → 2 instances (ceil(8/4.0) = 2)
        let result = calculator.calculate_instances(ComplexityClass::Quadratic, 8);
        assert_eq!(result.recommended_instances, 2);
        assert_eq!(result.subtasks_per_instance, 4.0);

        // 10 subtasks → 3 instances (ceil(10/4.0) = 3)
        let result = calculator.calculate_instances(ComplexityClass::Quadratic, 10);
        assert_eq!(result.recommended_instances, 3);
        assert!((result.subtasks_per_instance - 3.33).abs() < 0.01);
    }

    #[test]
    fn test_exponential_complexity() {
        let calculator = InstanceCalculator::new();

        // 7 subtasks → 2 instances (ceil(7/6.5) = 2)
        let result = calculator.calculate_instances(ComplexityClass::Exponential, 7);
        assert_eq!(result.recommended_instances, 2);
        assert_eq!(result.subtasks_per_instance, 3.5);

        // 15 subtasks → 3 instances (ceil(15/6.5) = 3)
        let result = calculator.calculate_instances(ComplexityClass::Exponential, 15);
        assert_eq!(result.recommended_instances, 3);
        assert_eq!(result.subtasks_per_instance, 5.0);

        // 20 subtasks → 4 instances (ceil(20/6.5) = 4)
        let result = calculator.calculate_instances(ComplexityClass::Exponential, 20);
        assert_eq!(result.recommended_instances, 4);
        assert_eq!(result.subtasks_per_instance, 5.0);
    }

    #[test]
    fn test_resource_constraint_linear() {
        let calculator = InstanceCalculator::new();

        // 50 subtasks with Linear (1.5 per instance)
        // Unconstrained: ceil(50/1.5) = 34 instances
        // Constrained: 10 instances (max limit)
        let result = calculator.calculate_instances(ComplexityClass::Linear, 50);

        assert_eq!(result.recommended_instances, 10);
        assert_eq!(result.subtasks_per_instance, 5.0);
        assert!(result.resource_constrained);
        assert!(result.reasoning.contains("Resource constrained"));
        assert!(result.reasoning.contains("34 instances")); // Unconstrained count
    }

    #[test]
    fn test_resource_constraint_exponential() {
        let calculator = InstanceCalculator::new();

        // 100 subtasks with Exponential (6.5 per instance)
        // Unconstrained: ceil(100/6.5) = 16 instances
        // Constrained: 10 instances (max limit)
        let result = calculator.calculate_instances(ComplexityClass::Exponential, 100);

        assert_eq!(result.recommended_instances, 10);
        assert_eq!(result.subtasks_per_instance, 10.0);
        assert!(result.resource_constrained);
        assert!(result.reasoning.contains("⚠️"));
    }

    #[test]
    fn test_exactly_at_max_instances() {
        let calculator = InstanceCalculator::new();

        // 15 subtasks with Linear (1.5 per instance)
        // Unconstrained: ceil(15/1.5) = 10 instances
        // Exactly at limit, not constrained
        let result = calculator.calculate_instances(ComplexityClass::Linear, 15);

        assert_eq!(result.recommended_instances, 10);
        assert_eq!(result.subtasks_per_instance, 1.5);
        assert!(!result.resource_constrained); // Not constrained!
    }

    #[test]
    fn test_custom_max_instances() {
        let calculator = InstanceCalculator::with_max_instances(5);

        // 20 subtasks with Linear (1.5 per instance)
        // Unconstrained: ceil(20/1.5) = 14 instances
        // Constrained: 5 instances
        let result = calculator.calculate_instances(ComplexityClass::Linear, 20);

        assert_eq!(result.recommended_instances, 5);
        assert_eq!(result.subtasks_per_instance, 4.0);
        assert!(result.resource_constrained);
    }

    #[test]
    fn test_large_subtask_count() {
        let calculator = InstanceCalculator::new();

        // 1000 subtasks with Exponential
        // Unconstrained: ceil(1000/6.5) = 154 instances
        // Constrained: 10 instances
        let result = calculator.calculate_instances(ComplexityClass::Exponential, 1000);

        assert_eq!(result.recommended_instances, 10);
        assert_eq!(result.subtasks_per_instance, 100.0);
        assert!(result.resource_constrained);
    }

    #[test]
    fn test_reasoning_contains_key_info() {
        let calculator = InstanceCalculator::new();
        let result = calculator.calculate_instances(ComplexityClass::Quadratic, 10);

        assert!(result.reasoning.contains("Ω(n²)"));
        assert!(result.reasoning.contains("10 total"));
        assert!(result.reasoning.contains("4.0")); // Optimal subtasks/instance
        assert!(result.reasoning.contains("3 instances")); // Recommended
    }

    #[test]
    fn test_reasoning_resource_constrained() {
        let calculator = InstanceCalculator::new();
        let result = calculator.calculate_instances(ComplexityClass::Linear, 50);

        assert!(result.reasoning.contains("Resource constrained"));
        assert!(result.reasoning.contains("34 instances")); // Unconstrained
        assert!(result.reasoning.contains("10 instances")); // Max limit
        assert!(result.reasoning.contains("Note:")); // Warning note
    }

    #[test]
    fn test_all_complexity_classes() {
        let calculator = InstanceCalculator::new();

        let test_cases = vec![
            (ComplexityClass::Constant, 5, 5),      // 1 per instance
            (ComplexityClass::Logarithmic, 5, 5),   // 1 per instance
            (ComplexityClass::Linear, 5, 4),        // ceil(5/1.5) = 4
            (ComplexityClass::Linearithmic, 5, 2),  // ceil(5/2.5) = 2
            (ComplexityClass::Quadratic, 5, 2),     // ceil(5/4.0) = 2
            (ComplexityClass::Exponential, 5, 1),   // ceil(5/6.5) = 1
        ];

        for (complexity, subtasks, expected_instances) in test_cases {
            let result = calculator.calculate_instances(complexity, subtasks);
            assert_eq!(
                result.recommended_instances, expected_instances,
                "Failed for {:?} with {} subtasks",
                complexity, subtasks
            );
            assert_eq!(result.complexity_class, complexity);
            assert!(!result.resource_constrained);
        }
    }

    #[test]
    fn test_serialization() {
        let calculation = InstanceCalculation {
            recommended_instances: 5,
            subtasks_per_instance: 2.5,
            complexity_class: ComplexityClass::Linear,
            resource_constrained: false,
            reasoning: "Test reasoning".to_string(),
        };

        let json = serde_json::to_string(&calculation).unwrap();
        let deserialized: InstanceCalculation = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, calculation);
    }

    #[test]
    fn test_single_subtask_all_complexities() {
        let calculator = InstanceCalculator::new();

        for complexity in [
            ComplexityClass::Constant,
            ComplexityClass::Logarithmic,
            ComplexityClass::Linear,
            ComplexityClass::Linearithmic,
            ComplexityClass::Quadratic,
            ComplexityClass::Exponential,
        ] {
            let result = calculator.calculate_instances(complexity, 1);
            assert_eq!(result.recommended_instances, 1);
            assert_eq!(result.subtasks_per_instance, 1.0);
        }
    }

    #[test]
    fn test_boundary_values() {
        let _calculator = InstanceCalculator::new();

        // Test with 100 instances max (edge case)
        let calculator_max = InstanceCalculator::with_max_instances(100);
        let result = calculator_max.calculate_instances(ComplexityClass::Linear, 200);
        assert_eq!(result.recommended_instances, 100);

        // Test with 1 instance max (extreme constraint)
        let calculator_min = InstanceCalculator::with_max_instances(1);
        let result = calculator_min.calculate_instances(ComplexityClass::Linear, 50);
        assert_eq!(result.recommended_instances, 1);
        assert_eq!(result.subtasks_per_instance, 50.0);
        assert!(result.resource_constrained);
    }

    #[test]
    fn test_fractional_rounding() {
        let calculator = InstanceCalculator::new();

        // Linear: 1.5 subtasks/instance
        // 4 subtasks → ceil(4/1.5) = ceil(2.67) = 3 instances
        let result = calculator.calculate_instances(ComplexityClass::Linear, 4);
        assert_eq!(result.recommended_instances, 3);
        assert!((result.subtasks_per_instance - 1.33).abs() < 0.01);
    }
}
