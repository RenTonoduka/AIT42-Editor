//! Big Omega (Ω) notation implementation for task complexity classification
//!
//! This module provides the `ComplexityClass` enum for categorizing tasks
//! based on their computational complexity using Big Omega notation.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::RangeInclusive;

/// Represents computational complexity classes using Big Omega (Ω) notation
///
/// Each variant corresponds to a well-known complexity class and provides
/// guidance on recommended subtask decomposition.
///
/// # Examples
///
/// ```
/// use omega_theory::ComplexityClass;
///
/// let complexity = ComplexityClass::Linear;
/// let subtasks = complexity.to_subtask_range();
/// assert_eq!(subtasks, 3..=5);
/// assert_eq!(complexity.to_string(), "Ω(n)");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplexityClass {
    /// Ω(1) - Constant time complexity
    ///
    /// Examples: Simple config changes, variable assignments, constant lookups
    Constant,

    /// Ω(log n) - Logarithmic time complexity
    ///
    /// Examples: Binary search, balanced tree operations, divide-and-conquer algorithms
    Logarithmic,

    /// Ω(n) - Linear time complexity
    ///
    /// Examples: Standard CRUD operations, single-pass array processing, simple iterations
    Linear,

    /// Ω(n log n) - Linearithmic time complexity
    ///
    /// Examples: Efficient sorting algorithms, index building, merge operations
    Linearithmic,

    /// Ω(n²) - Quadratic time complexity
    ///
    /// Examples: Nested loops, matrix operations, simple graph algorithms
    Quadratic,

    /// Ω(2^n) - Exponential time complexity
    ///
    /// Examples: Combinatorial problems, brute-force search, recursive backtracking
    Exponential,
}

impl ComplexityClass {
    /// Analyzes a task description and returns an appropriate complexity class
    ///
    /// **Note**: This is a placeholder implementation for v1.6.0. LLM-based analysis
    /// will be implemented in v1.7.0 (Task 1.2).
    ///
    /// # Arguments
    ///
    /// * `desc` - Task description text to analyze
    ///
    /// # Returns
    ///
    /// Currently returns `ComplexityClass::Linear` as a safe default for all inputs.
    ///
    /// # Examples
    ///
    /// ```
    /// use omega_theory::ComplexityClass;
    ///
    /// let complexity = ComplexityClass::from_description("Implement user authentication");
    /// assert_eq!(complexity, ComplexityClass::Linear);
    /// ```
    #[must_use]
    pub fn from_description(_desc: &str) -> Self {
        // Placeholder implementation - returns Linear for now
        // v1.7.0 will add LLM-based analysis via Anthropic API
        ComplexityClass::Linear
    }

    /// Returns the recommended range of subtasks for this complexity class
    ///
    /// Based on research-backed decomposition strategies:
    /// - Constant: 1 subtask (no decomposition needed)
    /// - Logarithmic: 2-3 subtasks (minimal decomposition)
    /// - Linear: 3-5 subtasks (standard decomposition)
    /// - Linearithmic: 4-6 subtasks (moderate decomposition)
    /// - Quadratic: 5-10 subtasks (aggressive decomposition)
    /// - Exponential: 8-15 subtasks (maximum decomposition)
    ///
    /// # Returns
    ///
    /// An inclusive range indicating the minimum and maximum recommended subtasks
    ///
    /// # Examples
    ///
    /// ```
    /// use omega_theory::ComplexityClass;
    ///
    /// assert_eq!(ComplexityClass::Constant.to_subtask_range(), 1..=1);
    /// assert_eq!(ComplexityClass::Linear.to_subtask_range(), 3..=5);
    /// assert_eq!(ComplexityClass::Quadratic.to_subtask_range(), 5..=10);
    /// ```
    #[must_use]
    pub fn to_subtask_range(&self) -> RangeInclusive<usize> {
        match self {
            ComplexityClass::Constant => 1..=1,
            ComplexityClass::Logarithmic => 2..=3,
            ComplexityClass::Linear => 3..=5,
            ComplexityClass::Linearithmic => 4..=6,
            ComplexityClass::Quadratic => 5..=10,
            ComplexityClass::Exponential => 8..=15,
        }
    }

    /// Returns a human-friendly description of the complexity class
    ///
    /// # Examples
    ///
    /// ```
    /// use omega_theory::ComplexityClass;
    ///
    /// assert_eq!(
    ///     ComplexityClass::Linear.description(),
    ///     "Linear - Standard CRUD operations, single-pass processing"
    /// );
    /// ```
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            ComplexityClass::Constant => "Constant - Simple config changes, trivial operations",
            ComplexityClass::Logarithmic => "Logarithmic - Binary search, tree operations",
            ComplexityClass::Linear => "Linear - Standard CRUD operations, single-pass processing",
            ComplexityClass::Linearithmic => "Linearithmic - Sorting, indexing, efficient algorithms",
            ComplexityClass::Quadratic => "Quadratic - Nested loops, matrix operations",
            ComplexityClass::Exponential => "Exponential - Combinatorial problems, brute-force search",
        }
    }
}

impl fmt::Display for ComplexityClass {
    /// Formats the complexity class as Big Omega notation
    ///
    /// # Examples
    ///
    /// ```
    /// use omega_theory::ComplexityClass;
    ///
    /// assert_eq!(ComplexityClass::Constant.to_string(), "Ω(1)");
    /// assert_eq!(ComplexityClass::Linear.to_string(), "Ω(n)");
    /// assert_eq!(ComplexityClass::Quadratic.to_string(), "Ω(n²)");
    /// assert_eq!(ComplexityClass::Exponential.to_string(), "Ω(2^n)");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let notation = match self {
            ComplexityClass::Constant => "Ω(1)",
            ComplexityClass::Logarithmic => "Ω(log n)",
            ComplexityClass::Linear => "Ω(n)",
            ComplexityClass::Linearithmic => "Ω(n log n)",
            ComplexityClass::Quadratic => "Ω(n²)",
            ComplexityClass::Exponential => "Ω(2^n)",
        };
        write!(f, "{notation}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_class_display() {
        assert_eq!(ComplexityClass::Constant.to_string(), "Ω(1)");
        assert_eq!(ComplexityClass::Logarithmic.to_string(), "Ω(log n)");
        assert_eq!(ComplexityClass::Linear.to_string(), "Ω(n)");
        assert_eq!(ComplexityClass::Linearithmic.to_string(), "Ω(n log n)");
        assert_eq!(ComplexityClass::Quadratic.to_string(), "Ω(n²)");
        assert_eq!(ComplexityClass::Exponential.to_string(), "Ω(2^n)");
    }

    #[test]
    fn test_subtask_ranges() {
        assert_eq!(ComplexityClass::Constant.to_subtask_range(), 1..=1);
        assert_eq!(ComplexityClass::Logarithmic.to_subtask_range(), 2..=3);
        assert_eq!(ComplexityClass::Linear.to_subtask_range(), 3..=5);
        assert_eq!(ComplexityClass::Linearithmic.to_subtask_range(), 4..=6);
        assert_eq!(ComplexityClass::Quadratic.to_subtask_range(), 5..=10);
        assert_eq!(ComplexityClass::Exponential.to_subtask_range(), 8..=15);
    }

    #[test]
    fn test_from_description_placeholder() {
        // Test placeholder implementation (always returns Linear)
        assert_eq!(
            ComplexityClass::from_description("Simple config change"),
            ComplexityClass::Linear
        );
        assert_eq!(
            ComplexityClass::from_description("Implement authentication system"),
            ComplexityClass::Linear
        );
        assert_eq!(
            ComplexityClass::from_description("Optimize database queries"),
            ComplexityClass::Linear
        );
    }

    #[test]
    fn test_equality_and_comparison() {
        assert_eq!(ComplexityClass::Linear, ComplexityClass::Linear);
        assert_ne!(ComplexityClass::Linear, ComplexityClass::Quadratic);

        // Test that enum is Copy
        let c1 = ComplexityClass::Constant;
        let c2 = c1;
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_serialization() {
        let complexity = ComplexityClass::Linear;
        let json = serde_json::to_string(&complexity).unwrap();
        assert_eq!(json, "\"Linear\"");

        let deserialized: ComplexityClass = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ComplexityClass::Linear);
    }

    #[test]
    fn test_all_variants_serialization() {
        let variants = vec![
            ComplexityClass::Constant,
            ComplexityClass::Logarithmic,
            ComplexityClass::Linear,
            ComplexityClass::Linearithmic,
            ComplexityClass::Quadratic,
            ComplexityClass::Exponential,
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: ComplexityClass = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn test_description() {
        assert!(ComplexityClass::Constant.description().contains("Simple"));
        assert!(ComplexityClass::Linear.description().contains("CRUD"));
        assert!(ComplexityClass::Quadratic.description().contains("Nested"));
    }
}
