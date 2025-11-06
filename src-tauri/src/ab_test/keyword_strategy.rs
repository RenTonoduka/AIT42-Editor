//! Keyword-based optimization strategy (v1.5.0 baseline)
//!
//! This module implements a simple keyword-matching strategy that represents
//! the baseline approach before LLM integration. It uses heuristic rules to
//! estimate task complexity based on keyword presence.
//!
//! # Performance Characteristics
//!
//! - Latency: <5ms (no API calls)
//! - Accuracy: ~60% (estimated)
//! - Scalability: O(n) where n = number of keywords
//! - Memory: Constant
//!
//! # Strategy Rules
//!
//! 1. Check for complexity keywords in descending order
//! 2. First match determines complexity class
//! 3. Subtask count = midpoint of Ω range
//! 4. Confidence = 0.6 (low, since it's heuristic)

use crate::ab_test::{OptimizationStrategy, OptimizationResult};
use anyhow::Result;
use omega_theory::ComplexityClass;
use std::time::Instant;
use tracing::debug;

/// Keyword-based optimization strategy
///
/// Uses simple keyword matching to estimate task complexity.
/// This represents the v1.5.0 baseline before LLM integration.
pub struct KeywordStrategy {
    /// Strategy name for reporting
    name: String,
}

impl KeywordStrategy {
    /// Create a new keyword strategy
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: "v1.5.0 Keyword Matching".to_string(),
        }
    }

    /// Analyze task description and classify complexity
    fn classify_complexity(&self, task_description: &str) -> ComplexityClass {
        let desc_lower = task_description.to_lowercase();

        // Check for exponential complexity keywords (most complex first)
        if self.contains_any(
            &desc_lower,
            &[
                "migrate",
                "rebuild",
                "refactor entire",
                "complete",
                "comprehensive",
                "distributed",
                "microservices",
                "genetic algorithm",
                "constraint satisfaction",
                "2-phase commit",
                "ci/cd pipeline",
                "multi-environment",
            ],
        ) {
            return ComplexityClass::Exponential;
        }

        // Check for quadratic complexity keywords
        if self.contains_any(
            &desc_lower,
            &[
                "e-commerce",
                "payment gateway",
                "social graph",
                "real-time collaboration",
                "nested comment",
                "conflict resolution",
                "matrix",
                "recommendation",
            ],
        ) {
            return ComplexityClass::Quadratic;
        }

        // Check for linearithmic complexity keywords
        if self.contains_any(
            &desc_lower,
            &[
                "sorting",
                "filtering",
                "full-text search",
                "indexing",
                "merge sort",
                "analytics",
                "aggregation",
                "collaborative filtering",
            ],
        ) {
            return ComplexityClass::Linearithmic;
        }

        // Check for linear complexity keywords
        if self.contains_any(
            &desc_lower,
            &[
                "crud",
                "authentication",
                "jwt",
                "api",
                "endpoint",
                "form",
                "validation",
                "pagination",
                "upload",
                "profile",
            ],
        ) {
            return ComplexityClass::Linear;
        }

        // Check for logarithmic complexity keywords
        if self.contains_any(
            &desc_lower,
            &[
                "binary search",
                "tree",
                "b-tree",
                "balanced",
                "divide-and-conquer",
                "logarithmic",
                "cache",
                "lru",
            ],
        ) {
            return ComplexityClass::Logarithmic;
        }

        // Check for constant complexity keywords
        if self.contains_any(
            &desc_lower,
            &[
                "button",
                "config",
                "constant",
                "typo",
                "color",
                "console.log",
                "debug",
                "fix typo",
                "change",
                "update configuration",
            ],
        ) {
            return ComplexityClass::Constant;
        }

        // Default to Linear if no keywords match
        ComplexityClass::Linear
    }

    /// Check if text contains any of the given keywords
    fn contains_any(&self, text: &str, keywords: &[&str]) -> bool {
        keywords.iter().any(|keyword| text.contains(keyword))
    }

    /// Get recommended subtask count for complexity class
    ///
    /// Returns the midpoint of the Ω-theory range for the given class.
    fn get_subtask_count(&self, complexity: ComplexityClass) -> usize {
        let range = complexity.to_subtask_range();
        (*range.start() + *range.end()) / 2
    }
}

impl Default for KeywordStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationStrategy for KeywordStrategy {
    fn optimize(&self, task_description: &str) -> Result<OptimizationResult> {
        let start = Instant::now();

        // Classify complexity
        let complexity_class = self.classify_complexity(task_description);

        // Get recommended subtasks (midpoint of range)
        let recommended_subtasks = self.get_subtask_count(complexity_class);

        // Build reasoning
        let reasoning = format!(
            "Keyword-based classification: {}\nDetected complexity: {}\nΩ-theory range: {:?}\nRecommended: {} subtasks (midpoint)",
            complexity_class.description(),
            complexity_class,
            complexity_class.to_subtask_range(),
            recommended_subtasks
        );

        let latency = start.elapsed();

        debug!(
            "Keyword strategy classified task as {} in {:?}",
            complexity_class, latency
        );

        Ok(OptimizationResult {
            strategy_name: self.name.clone(),
            complexity_class,
            recommended_subtasks,
            confidence: 0.6, // Low confidence for heuristic approach
            reasoning,
            latency_ms: latency.as_millis() as u64,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_classification() {
        let strategy = KeywordStrategy::new();
        let result = strategy.optimize("Add a button to the homepage").unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Constant);
        assert_eq!(result.recommended_subtasks, 1);
        assert_eq!(result.confidence, 0.6);
    }

    #[test]
    fn test_logarithmic_classification() {
        let strategy = KeywordStrategy::new();
        let result = strategy
            .optimize("Implement binary search for user lookup")
            .unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Logarithmic);
        assert_eq!(result.recommended_subtasks, 2); // Midpoint of 2..=3
    }

    #[test]
    fn test_linear_classification() {
        let strategy = KeywordStrategy::new();
        let result = strategy
            .optimize("Implement user authentication with JWT")
            .unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Linear);
        assert_eq!(result.recommended_subtasks, 4); // Midpoint of 3..=5
    }

    #[test]
    fn test_linearithmic_classification() {
        let strategy = KeywordStrategy::new();
        let result = strategy
            .optimize("Build full-text search with indexing")
            .unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Linearithmic);
        assert_eq!(result.recommended_subtasks, 5); // Midpoint of 4..=6
    }

    #[test]
    fn test_quadratic_classification() {
        let strategy = KeywordStrategy::new();
        let result = strategy
            .optimize("Build e-commerce platform with payment gateway integration")
            .unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Quadratic);
        assert_eq!(result.recommended_subtasks, 7); // Midpoint of 5..=10
    }

    #[test]
    fn test_exponential_classification() {
        let strategy = KeywordStrategy::new();
        let result = strategy
            .optimize("Migrate entire monolithic application to microservices architecture")
            .unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Exponential);
        assert_eq!(result.recommended_subtasks, 11); // Midpoint of 8..=15
    }

    #[test]
    fn test_default_to_linear() {
        let strategy = KeywordStrategy::new();
        let result = strategy.optimize("Do xyz task with abc feature").unwrap();

        // Should default to Linear since no keywords match
        assert_eq!(result.complexity_class, ComplexityClass::Linear);
        assert_eq!(result.recommended_subtasks, 4);
    }

    #[test]
    fn test_case_insensitive() {
        let strategy = KeywordStrategy::new();

        let result1 = strategy.optimize("IMPLEMENT AUTHENTICATION WITH JWT").unwrap();
        let result2 = strategy.optimize("implement authentication with jwt").unwrap();

        assert_eq!(result1.complexity_class, result2.complexity_class);
    }

    #[test]
    fn test_latency() {
        let strategy = KeywordStrategy::new();
        let result = strategy.optimize("Test task").unwrap();

        // Keyword strategy should be very fast (<5ms)
        assert!(result.latency_ms < 5, "Latency should be under 5ms");
    }

    #[test]
    fn test_reasoning_contains_key_info() {
        let strategy = KeywordStrategy::new();
        let result = strategy
            .optimize("Create CRUD endpoints for blog posts")
            .unwrap();

        assert!(result.reasoning.contains("Keyword-based"));
        assert!(result.reasoning.contains("Ω(n)"));
        assert!(result.reasoning.contains("midpoint"));
    }

    #[test]
    fn test_strategy_name() {
        let strategy = KeywordStrategy::new();
        assert_eq!(strategy.name(), "v1.5.0 Keyword Matching");
    }

    #[test]
    fn test_multiple_keywords_priority() {
        let strategy = KeywordStrategy::new();

        // Task with both "migrate" (exponential) and "crud" (linear)
        // Should prioritize exponential since it's checked first
        let result = strategy.optimize("Migrate CRUD API to new framework").unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Exponential);
    }

    #[test]
    fn test_empty_description() {
        let strategy = KeywordStrategy::new();
        let result = strategy.optimize("").unwrap();

        // Should default to Linear
        assert_eq!(result.complexity_class, ComplexityClass::Linear);
    }

    #[test]
    fn test_partial_keyword_match() {
        let strategy = KeywordStrategy::new();

        // "authentication" should match even in "re-authentication"
        let result = strategy.optimize("Implement re-authentication flow").unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Linear);
    }
}
