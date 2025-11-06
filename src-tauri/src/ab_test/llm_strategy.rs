//! LLM-based optimization strategy (v1.6.0)
//!
//! This module wraps the SubtaskOptimizer (LLM + Ω-theory) for A/B testing.
//! It represents the new v1.6.0 approach that uses Claude API for intelligent
//! complexity estimation with mathematical validation.
//!
//! # Performance Characteristics
//!
//! - Latency: ~1-2s (API calls, with caching)
//! - Accuracy: ~90% (expected)
//! - Scalability: Limited by API rate limits
//! - Memory: ~1KB per cached result
//!
//! # Strategy Architecture
//!
//! ```text
//! LLMStrategy
//!     └── SubtaskOptimizer
//!         ├── CachedEstimator (LLM)
//!         └── Ω-Theory Validation
//! ```

use crate::ab_test::{OptimizationStrategy, OptimizationResult};
use crate::optimizer::SubtaskOptimizer;
use anyhow::Result;
use std::time::Instant;
use tracing::debug;

/// LLM-based optimization strategy (v1.6.0)
///
/// Uses SubtaskOptimizer with LLM + Ω-theory for intelligent
/// complexity estimation.
pub struct LLMStrategy {
    /// Strategy name for reporting
    name: String,

    /// Underlying optimizer
    optimizer: SubtaskOptimizer,
}

impl LLMStrategy {
    /// Create a new LLM strategy with the given API key
    ///
    /// # Arguments
    ///
    /// * `anthropic_api_key` - API key from console.anthropic.com
    ///
    /// # Errors
    ///
    /// Returns error if API key is invalid
    pub fn new(anthropic_api_key: String) -> Result<Self> {
        let optimizer = SubtaskOptimizer::new(anthropic_api_key)?;

        Ok(Self {
            name: "v1.6.0 LLM + Ω-Theory".to_string(),
            optimizer,
        })
    }

    /// Create LLM strategy from environment variable
    ///
    /// Reads `ANTHROPIC_API_KEY` from environment.
    ///
    /// # Errors
    ///
    /// Returns error if environment variable is not set
    pub fn from_env() -> Result<Self> {
        let optimizer = SubtaskOptimizer::from_env()?;

        Ok(Self {
            name: "v1.6.0 LLM + Ω-Theory".to_string(),
            optimizer,
        })
    }

    /// Pre-warm cache with estimate (for testing)
    ///
    /// This allows deterministic testing without actual API calls.
    pub fn insert_cached(
        &self,
        task_description: &str,
        estimate: llm_estimator::ComplexityEstimate,
    ) {
        self.optimizer.insert_cached(task_description, 0, estimate);
    }
}

impl LLMStrategy {
    /// Async version of optimize
    ///
    /// This is the underlying async implementation that can be called directly
    /// from async contexts to avoid nested runtime issues.
    pub async fn optimize_async(&self, task_description: &str) -> Result<OptimizationResult> {
        let start = Instant::now();

        let optimization = self
            .optimizer
            .optimize_subtask_count(task_description, 0)
            .await?;

        let latency = start.elapsed();

        debug!(
            "LLM strategy classified task as {} in {:?}",
            optimization.complexity_class, latency
        );

        Ok(OptimizationResult {
            strategy_name: self.name.clone(),
            complexity_class: optimization.complexity_class,
            recommended_subtasks: optimization.recommended_subtasks,
            confidence: optimization.confidence,
            reasoning: optimization.reasoning,
            latency_ms: latency.as_millis() as u64,
        })
    }
}

impl OptimizationStrategy for LLMStrategy {
    fn optimize(&self, task_description: &str) -> Result<OptimizationResult> {
        // Use futures::executor::block_on to avoid tokio runtime nesting issues
        // This works in both sync and async contexts
        futures::executor::block_on(self.optimize_async(task_description))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_estimator::ComplexityEstimate;
    use omega_theory::ComplexityClass;

    fn create_test_strategy() -> LLMStrategy {
        // Create with dummy API key for testing
        LLMStrategy::new("sk-ant-test-key".to_string()).await.unwrap()
    }

    fn create_mock_estimate(
        complexity: &str,
        subtasks: usize,
        confidence: f64,
    ) -> ComplexityEstimate {
        ComplexityEstimate {
            complexity_class: complexity.to_string(),
            reasoning: "Mock LLM reasoning for test".to_string(),
            recommended_subtasks: subtasks,
            confidence,
        }
    }

    #[tokio::test]
    async fn
    fn test_strategy_creation() {
        let strategy = create_test_strategy();
        assert_eq!(strategy.name(), "v1.6.0 LLM + Ω-Theory");
    }

    #[tokio::test]
    async fn
    fn test_strategy_from_env_missing_key() {
        // Remove env var if set
        std::env::remove_var("ANTHROPIC_API_KEY");

        let result = LLMStrategy::from_env();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_optimization_with_cached_estimate() {
        let strategy = create_test_strategy();

        // Pre-warm cache with mock estimate
        let mock_estimate = create_mock_estimate("Ω(n)", 4, 0.88);
        strategy.insert_cached("Test task for LLM strategy", mock_estimate);

        // Run optimization (should hit cache)
        let result = strategy.optimize_async("Test task for LLM strategy").await.await.unwrap();

        assert_eq!(result.strategy_name, "v1.6.0 LLM + Ω-Theory");
        assert_eq!(result.complexity_class, ComplexityClass::Linear);
        assert_eq!(result.recommended_subtasks, 4);
        assert_eq!(result.confidence, 0.88);
    }

    #[tokio::test]
    async fn
    fn test_constant_complexity() {
        let strategy = create_test_strategy();

        let mock_estimate = create_mock_estimate("Ω(1)", 1, 0.95);
        strategy.insert_cached("Add button", mock_estimate);

        let result = strategy.optimize_async("Add button").await.unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Constant);
        assert_eq!(result.recommended_subtasks, 1);
    }

    #[tokio::test]
    async fn
    fn test_logarithmic_complexity() {
        let strategy = create_test_strategy();

        let mock_estimate = create_mock_estimate("Ω(log n)", 2, 0.85);
        strategy.insert_cached("Binary search", mock_estimate);

        let result = strategy.optimize_async("Binary search").await.unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Logarithmic);
        assert_eq!(result.recommended_subtasks, 2);
    }

    #[tokio::test]
    async fn
    fn test_linear_complexity() {
        let strategy = create_test_strategy();

        let mock_estimate = create_mock_estimate("Ω(n)", 4, 0.90);
        strategy.insert_cached("CRUD API", mock_estimate);

        let result = strategy.optimize_async("CRUD API").await.unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Linear);
        assert_eq!(result.recommended_subtasks, 4);
    }

    #[tokio::test]
    async fn
    fn test_linearithmic_complexity() {
        let strategy = create_test_strategy();

        let mock_estimate = create_mock_estimate("Ω(n log n)", 5, 0.87);
        strategy.insert_cached("Full-text search", mock_estimate);

        let result = strategy.optimize_async("Full-text search").await.unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Linearithmic);
        assert_eq!(result.recommended_subtasks, 5);
    }

    #[tokio::test]
    async fn
    fn test_quadratic_complexity() {
        let strategy = create_test_strategy();

        let mock_estimate = create_mock_estimate("Ω(n²)", 8, 0.82);
        strategy.insert_cached("E-commerce platform", mock_estimate);

        let result = strategy.optimize_async("E-commerce platform").await.unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Quadratic);
        assert_eq!(result.recommended_subtasks, 8);
    }

    #[tokio::test]
    async fn
    fn test_exponential_complexity() {
        let strategy = create_test_strategy();

        let mock_estimate = create_mock_estimate("Ω(2^n)", 12, 0.78);
        strategy.insert_cached("Microservices migration", mock_estimate);

        let result = strategy.optimize_async("Microservices migration").await.unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Exponential);
        assert_eq!(result.recommended_subtasks, 12);
    }

    #[tokio::test]
    async fn
    fn test_omega_theory_adjustment() {
        let strategy = create_test_strategy();

        // Mock estimate with subtasks outside Ω bounds
        // Linear is 3..=5, but LLM suggests 10
        let mock_estimate = create_mock_estimate("Ω(n)", 10, 0.85);
        strategy.insert_cached("Over-estimated task", mock_estimate);

        let result = strategy.optimize_async("Over-estimated task").await.unwrap();

        // Should be adjusted to upper bound
        assert_eq!(result.complexity_class, ComplexityClass::Linear);
        assert_eq!(result.recommended_subtasks, 5); // Adjusted from 10 to 5
        assert!(result.reasoning.contains("Adjusted") || result.reasoning.contains("5"));
    }

    #[tokio::test]
    async fn
    fn test_reasoning_contains_llm_analysis() {
        let strategy = create_test_strategy();

        let mock_estimate = create_mock_estimate("Ω(n)", 4, 0.88);
        strategy.insert_cached("Auth system", mock_estimate);

        let result = strategy.optimize_async("Auth system").await.unwrap();

        // Reasoning should contain LLM output
        assert!(result.reasoning.contains("Mock LLM reasoning"));
        assert!(result.reasoning.contains("Ω"));
    }

    #[tokio::test]
    async fn
    fn test_high_confidence() {
        let strategy = create_test_strategy();

        let mock_estimate = create_mock_estimate("Ω(n²)", 7, 0.95);
        strategy.insert_cached("Complex system", mock_estimate);

        let result = strategy.optimize_async("Complex system").await.unwrap();

        assert_eq!(result.confidence, 0.95);
        assert!(result.confidence > 0.6); // Higher than keyword strategy
    }

    #[tokio::test]
    async fn
    fn test_latency_reasonable() {
        let strategy = create_test_strategy();

        let mock_estimate = create_mock_estimate("Ω(n)", 4, 0.85);
        strategy.insert_cached("Cached task", mock_estimate);

        let result = strategy.optimize_async("Cached task").await.unwrap();

        // With cache hit, should be fast (<100ms)
        assert!(
            result.latency_ms < 100,
            "Cached result should be fast, got {}ms",
            result.latency_ms
        );
    }

    #[tokio::test]
    async fn
    fn test_strategy_name_correct() {
        let strategy = create_test_strategy();
        let mock_estimate = create_mock_estimate("Ω(n)", 4, 0.85);
        strategy.insert_cached("Test", mock_estimate);

        let result = strategy.optimize_async("Test").await.unwrap();

        assert_eq!(result.strategy_name, "v1.6.0 LLM + Ω-Theory");
    }
}
