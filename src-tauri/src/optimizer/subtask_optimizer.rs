//! Subtask Count Optimizer
//!
//! This module integrates the LLM-based complexity estimator with Ω-theory rules
//! to provide optimal subtask count recommendations.
//!
//! # Architecture
//!
//! ```text
//! SubtaskOptimizer
//!     ├── CachedEstimator (LLM-based analysis)
//!     │   └── AnthropicClient (Claude API)
//!     └── ComplexityClass::to_subtask_range() (Ω-theory rules)
//! ```
//!
//! # Performance
//!
//! - Target: <500ms per optimization
//! - Cache hit: ~1-5ms
//! - Cache miss: ~1-2s (LLM API call)
//! - Memory: ~1KB per cached result

use llm_estimator::{AnthropicClient, CachedEstimator, ComplexityEstimate, EstimatorError};
use omega_theory::ComplexityClass;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Errors that can occur during subtask optimization
#[derive(Debug, Error)]
pub enum OptimizerError {
    /// LLM API error
    #[error("LLM estimation failed: {0}")]
    EstimationFailed(#[from] EstimatorError),

    /// Timeout exceeded
    #[error("Optimization timeout exceeded ({0:?})")]
    Timeout(Duration),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Memory-based adjustment for future optimization (v1.7.0+)
///
/// This is a placeholder structure for the memory-based learning system
/// that will be implemented in v1.7.0.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryAdjustment {
    /// Historical success rate for similar tasks
    pub historical_success_rate: f64,

    /// Suggested adjustment to subtask count (-3 to +3)
    pub adjustment: i32,

    /// Reasoning for the adjustment
    pub reasoning: String,
}

/// Result of subtask count optimization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OptimizationResult {
    /// Detected complexity class (e.g., Ω(n))
    pub complexity_class: ComplexityClass,

    /// Recommended number of subtasks
    pub recommended_subtasks: usize,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Human-readable reasoning for the recommendation
    pub reasoning: String,

    /// Memory-based adjustment (v1.7.0+ placeholder)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_adjustment: Option<MemoryAdjustment>,

    /// Original LLM estimate before Ω-theory validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_estimate: Option<ComplexityEstimate>,
}

/// Subtask count optimizer
///
/// Combines LLM-based complexity estimation with Ω-theory rules to provide
/// optimal subtask decomposition recommendations.
///
/// # Example
///
/// ```no_run
/// use subtask_optimizer::SubtaskOptimizer;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let optimizer = SubtaskOptimizer::new("sk-ant-api-key".to_string())?;
///
/// let result = optimizer
///     .optimize_subtask_count("Implement user authentication with JWT", 0)
///     .await?;
///
/// println!("Complexity: {}", result.complexity_class);
/// println!("Recommended subtasks: {}", result.recommended_subtasks);
/// println!("Confidence: {:.2}", result.confidence);
/// # Ok(())
/// # }
/// ```
pub struct SubtaskOptimizer {
    estimator: CachedEstimator,
    timeout: Duration,
}

impl SubtaskOptimizer {
    /// Create a new optimizer with the given Anthropic API key
    ///
    /// # Arguments
    ///
    /// * `anthropic_api_key` - API key from console.anthropic.com
    ///
    /// # Errors
    ///
    /// Returns `OptimizerError::InvalidInput` if the API key is invalid
    pub fn new(anthropic_api_key: String) -> Result<Self, OptimizerError> {
        let client = AnthropicClient::new(anthropic_api_key)
            .map_err(|e| OptimizerError::InvalidInput(format!("Invalid API key: {}", e)))?;

        let estimator = CachedEstimator::new(client);

        Ok(Self {
            estimator,
            timeout: Duration::from_millis(500),
        })
    }

    /// Create optimizer from environment variable
    ///
    /// Reads `ANTHROPIC_API_KEY` from environment.
    ///
    /// # Errors
    ///
    /// Returns error if environment variable is not set
    pub fn from_env() -> Result<Self, OptimizerError> {
        let client = AnthropicClient::from_env()
            .map_err(|e| OptimizerError::InvalidInput(format!("Missing API key: {}", e)))?;

        let estimator = CachedEstimator::new(client);

        Ok(Self {
            estimator,
            timeout: Duration::from_millis(500),
        })
    }

    /// Create optimizer with custom timeout
    ///
    /// # Arguments
    ///
    /// * `anthropic_api_key` - API key from console.anthropic.com
    /// * `timeout` - Maximum time to wait for optimization
    ///
    /// # Errors
    ///
    /// Returns error if API key is invalid
    pub fn with_timeout(
        anthropic_api_key: String,
        timeout: Duration,
    ) -> Result<Self, OptimizerError> {
        let mut optimizer = Self::new(anthropic_api_key)?;
        optimizer.timeout = timeout;
        Ok(optimizer)
    }

    /// Optimize subtask count for a given task
    ///
    /// This is the main entry point for the optimizer. It:
    /// 1. Calls the LLM to estimate complexity
    /// 2. Validates the estimate against Ω-theory rules
    /// 3. Adjusts recommendations to fit within Ω bounds
    /// 4. Returns a comprehensive optimization result
    ///
    /// # Arguments
    ///
    /// * `task_description` - The task to analyze
    /// * `current_subtasks` - Current number of subtasks (0 if new)
    ///
    /// # Returns
    ///
    /// An `OptimizationResult` with complexity class, recommended subtasks,
    /// confidence score, and reasoning.
    ///
    /// # Errors
    ///
    /// - `OptimizerError::EstimationFailed` if LLM call fails
    /// - `OptimizerError::Timeout` if optimization exceeds timeout
    /// - `OptimizerError::InvalidInput` if task description is empty
    ///
    /// # Performance
    ///
    /// - Cache hit: ~1-5ms
    /// - Cache miss: ~1-2s (LLM API call)
    /// - Target: <500ms with warm cache
    pub async fn optimize_subtask_count(
        &self,
        task_description: &str,
        current_subtasks: usize,
    ) -> Result<OptimizationResult, OptimizerError> {
        // Validate input
        if task_description.trim().is_empty() {
            return Err(OptimizerError::InvalidInput(
                "Task description cannot be empty".to_string(),
            ));
        }

        info!(
            "Optimizing subtask count for task (current: {}): {}",
            current_subtasks,
            task_description.chars().take(50).collect::<String>()
        );

        // Get LLM estimate with timeout
        let llm_estimate = tokio::time::timeout(self.timeout, async {
            self.estimator
                .estimate(task_description, current_subtasks)
                .await
        })
        .await
        .map_err(|_| OptimizerError::Timeout(self.timeout))?
        .map_err(OptimizerError::EstimationFailed)?;

        debug!("LLM estimate: {:?}", llm_estimate);

        // Parse complexity class
        let complexity_class = llm_estimate
            .to_complexity_class()
            .map_err(|e| OptimizerError::InvalidInput(format!("Invalid complexity class: {}", e)))?;

        // Get Ω-theory bounds for this complexity class
        let omega_range = complexity_class.to_subtask_range();

        // Validate and adjust LLM recommendation against Ω bounds
        let recommended_subtasks = if omega_range.contains(&llm_estimate.recommended_subtasks) {
            // LLM recommendation is within Ω bounds - use it
            debug!(
                "LLM recommendation {} is within Ω bounds {:?}",
                llm_estimate.recommended_subtasks, omega_range
            );
            llm_estimate.recommended_subtasks
        } else {
            // LLM recommendation is outside Ω bounds - adjust it
            let adjusted = if llm_estimate.recommended_subtasks < *omega_range.start() {
                *omega_range.start()
            } else {
                *omega_range.end()
            };

            warn!(
                "LLM recommendation {} outside Ω bounds {:?}, adjusted to {}",
                llm_estimate.recommended_subtasks, omega_range, adjusted
            );

            adjusted
        };

        // Build comprehensive reasoning
        let reasoning = self.build_reasoning(
            &complexity_class,
            &llm_estimate,
            recommended_subtasks,
            &omega_range,
        );

        // Memory adjustment placeholder (v1.7.0+)
        let memory_adjustment = None; // Future: query historical success rates

        Ok(OptimizationResult {
            complexity_class,
            recommended_subtasks,
            confidence: llm_estimate.confidence,
            reasoning,
            memory_adjustment,
            llm_estimate: Some(llm_estimate),
        })
    }

    /// Build comprehensive reasoning string
    fn build_reasoning(
        &self,
        complexity_class: &ComplexityClass,
        llm_estimate: &ComplexityEstimate,
        final_subtasks: usize,
        omega_range: &std::ops::RangeInclusive<usize>,
    ) -> String {
        let mut reasoning = format!(
            "Complexity Class: {} ({})\n",
            complexity_class,
            complexity_class.description()
        );

        reasoning.push_str(&format!("LLM Analysis: {}\n", llm_estimate.reasoning));

        reasoning.push_str(&format!(
            "Ω-Theory Bounds: {}-{} subtasks\n",
            omega_range.start(),
            omega_range.end()
        ));

        if llm_estimate.recommended_subtasks != final_subtasks {
            reasoning.push_str(&format!(
                "Adjusted: {} → {} (enforcing Ω bounds)\n",
                llm_estimate.recommended_subtasks, final_subtasks
            ));
        }

        reasoning.push_str(&format!("Confidence: {:.1}%", llm_estimate.confidence * 100.0));

        reasoning
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> llm_estimator::CacheStats {
        self.estimator.stats()
    }

    /// Clear the internal cache
    pub fn clear_cache(&self) {
        self.estimator.clear_cache();
    }

    /// Pre-warm cache with known estimate
    ///
    /// Useful for testing or when you have pre-computed estimates.
    pub fn insert_cached(
        &self,
        task_description: &str,
        current_subtasks: usize,
        estimate: ComplexityEstimate,
    ) {
        self.estimator
            .insert_cached(task_description, current_subtasks, estimate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_optimizer() -> SubtaskOptimizer {
        // Create optimizer with dummy key for unit tests
        SubtaskOptimizer::new("sk-ant-test-key".to_string()).unwrap()
    }

    fn create_mock_estimate(
        complexity: &str,
        subtasks: usize,
        confidence: f64,
    ) -> ComplexityEstimate {
        ComplexityEstimate {
            complexity_class: complexity.to_string(),
            reasoning: "Mock reasoning".to_string(),
            recommended_subtasks: subtasks,
            confidence,
        }
    }

    #[test]
    fn test_optimizer_creation() {
        let optimizer = create_test_optimizer();
        assert_eq!(optimizer.cache_stats().total_requests, 0);
    }

    #[test]
    fn test_optimizer_from_env_missing_key() {
        // Remove env var if set
        std::env::remove_var("ANTHROPIC_API_KEY");

        let result = SubtaskOptimizer::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_optimizer_with_custom_timeout() {
        let timeout = Duration::from_secs(10);
        let optimizer =
            SubtaskOptimizer::with_timeout("sk-ant-test-key".to_string(), timeout).unwrap();
        assert_eq!(optimizer.timeout, timeout);
    }

    #[tokio::test]
    async fn test_empty_task_description() {
        let optimizer = create_test_optimizer();
        let result = optimizer.optimize_subtask_count("", 0).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OptimizerError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn test_whitespace_only_task_description() {
        let optimizer = create_test_optimizer();
        let result = optimizer.optimize_subtask_count("   \n\t  ", 0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_optimization_with_cached_estimate() {
        let optimizer = create_test_optimizer();

        // Pre-warm cache with mock estimate
        let mock_estimate = create_mock_estimate("Ω(n)", 4, 0.85);
        optimizer.insert_cached("Test task", 0, mock_estimate);

        // This should hit cache
        let result = optimizer.optimize_subtask_count("Test task", 0).await;
        assert!(result.is_ok());

        let optimization = result.unwrap();
        assert_eq!(optimization.complexity_class, ComplexityClass::Linear);
        assert_eq!(optimization.recommended_subtasks, 4);
        assert_eq!(optimization.confidence, 0.85);
    }

    #[tokio::test]
    async fn test_llm_recommendation_within_omega_bounds() {
        let optimizer = create_test_optimizer();

        // Linear complexity: Ω bounds are 3-5 subtasks
        // Mock estimate within bounds: 4 subtasks
        let mock_estimate = create_mock_estimate("Ω(n)", 4, 0.9);
        optimizer.insert_cached("Test task within bounds", 0, mock_estimate);

        let result = optimizer
            .optimize_subtask_count("Test task within bounds", 0)
            .await
            .unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Linear);
        assert_eq!(result.recommended_subtasks, 4); // Should keep LLM recommendation
    }

    #[tokio::test]
    async fn test_llm_recommendation_below_omega_bounds() {
        let optimizer = create_test_optimizer();

        // Linear complexity: Ω bounds are 3-5 subtasks
        // Mock estimate below bounds: 1 subtask
        let mock_estimate = create_mock_estimate("Ω(n)", 1, 0.8);
        optimizer.insert_cached("Test task below bounds", 0, mock_estimate);

        let result = optimizer
            .optimize_subtask_count("Test task below bounds", 0)
            .await
            .unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Linear);
        assert_eq!(result.recommended_subtasks, 3); // Adjusted to lower bound
    }

    #[tokio::test]
    async fn test_llm_recommendation_above_omega_bounds() {
        let optimizer = create_test_optimizer();

        // Linear complexity: Ω bounds are 3-5 subtasks
        // Mock estimate above bounds: 10 subtasks
        let mock_estimate = create_mock_estimate("Ω(n)", 10, 0.7);
        optimizer.insert_cached("Test task above bounds", 0, mock_estimate);

        let result = optimizer
            .optimize_subtask_count("Test task above bounds", 0)
            .await
            .unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Linear);
        assert_eq!(result.recommended_subtasks, 5); // Adjusted to upper bound
    }

    #[tokio::test]
    async fn test_all_complexity_classes() {
        let optimizer = create_test_optimizer();

        let test_cases = vec![
            ("Ω(1)", ComplexityClass::Constant, 1..=1),
            ("Ω(log n)", ComplexityClass::Logarithmic, 2..=3),
            ("Ω(n)", ComplexityClass::Linear, 3..=5),
            ("Ω(n log n)", ComplexityClass::Linearithmic, 4..=6),
            ("Ω(n²)", ComplexityClass::Quadratic, 5..=10),
            ("Ω(2^n)", ComplexityClass::Exponential, 8..=15),
        ];

        for (i, (notation, expected_class, range)) in test_cases.iter().enumerate() {
            let task_desc = format!("Test task {}", i);
            let mid_subtasks = (*range.start() + *range.end()) / 2;

            let mock_estimate = create_mock_estimate(notation, mid_subtasks, 0.85);
            optimizer.insert_cached(&task_desc, 0, mock_estimate);

            let result = optimizer.optimize_subtask_count(&task_desc, 0).await.unwrap();

            assert_eq!(result.complexity_class, *expected_class);
            assert!(range.contains(&result.recommended_subtasks));
        }
    }

    #[tokio::test]
    async fn test_optimization_result_structure() {
        let optimizer = create_test_optimizer();

        let mock_estimate = create_mock_estimate("Ω(n)", 4, 0.88);
        optimizer.insert_cached("Complete task", 0, mock_estimate);

        let result = optimizer
            .optimize_subtask_count("Complete task", 0)
            .await
            .unwrap();

        // Verify all fields are populated
        assert_eq!(result.complexity_class, ComplexityClass::Linear);
        assert_eq!(result.recommended_subtasks, 4);
        assert_eq!(result.confidence, 0.88);
        assert!(!result.reasoning.is_empty());
        assert!(result.llm_estimate.is_some());
        assert!(result.memory_adjustment.is_none()); // v1.7.0 placeholder
    }

    #[tokio::test]
    async fn test_reasoning_contains_key_information() {
        let optimizer = create_test_optimizer();

        let mock_estimate = create_mock_estimate("Ω(n²)", 7, 0.92);
        optimizer.insert_cached("Reasoning test", 0, mock_estimate);

        let result = optimizer
            .optimize_subtask_count("Reasoning test", 0)
            .await
            .unwrap();

        // Reasoning should contain key elements
        assert!(result.reasoning.contains("Ω(n²)"));
        assert!(result.reasoning.contains("Mock reasoning"));
        assert!(result.reasoning.contains("5-10")); // Quadratic bounds
        assert!(result.reasoning.contains("92")); // Confidence percentage
    }

    #[tokio::test]
    async fn test_cache_hit_on_repeated_requests() {
        let optimizer = create_test_optimizer();

        let mock_estimate = create_mock_estimate("Ω(n)", 3, 0.8);
        optimizer.insert_cached("Repeated task", 0, mock_estimate);

        // First request
        let _ = optimizer.optimize_subtask_count("Repeated task", 0).await.unwrap();

        // Second request (should hit cache)
        let _ = optimizer.optimize_subtask_count("Repeated task", 0).await.unwrap();

        let stats = optimizer.cache_stats();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 2);
    }

    #[test]
    fn test_clear_cache() {
        let optimizer = create_test_optimizer();

        let mock_estimate = create_mock_estimate("Ω(n)", 4, 0.8);
        optimizer.insert_cached("Cache test", 0, mock_estimate);

        let stats_before = optimizer.cache_stats();
        assert_eq!(stats_before.cache_size, 1);

        optimizer.clear_cache();

        let stats_after = optimizer.cache_stats();
        assert_eq!(stats_after.cache_size, 0);
    }

    #[tokio::test]
    async fn test_different_current_subtasks() {
        let optimizer = create_test_optimizer();

        // Same task but different current subtask counts
        let mock_estimate1 = create_mock_estimate("Ω(n)", 4, 0.8);
        let mock_estimate2 = create_mock_estimate("Ω(n)", 5, 0.85);

        optimizer.insert_cached("Same task", 0, mock_estimate1);
        optimizer.insert_cached("Same task", 3, mock_estimate2);

        let result1 = optimizer.optimize_subtask_count("Same task", 0).await.unwrap();
        let result2 = optimizer.optimize_subtask_count("Same task", 3).await.unwrap();

        assert_eq!(result1.recommended_subtasks, 4);
        assert_eq!(result2.recommended_subtasks, 5);
    }

    #[test]
    fn test_serialization_of_optimization_result() {
        let result = OptimizationResult {
            complexity_class: ComplexityClass::Linear,
            recommended_subtasks: 4,
            confidence: 0.85,
            reasoning: "Test reasoning".to_string(),
            memory_adjustment: None,
            llm_estimate: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: OptimizationResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, result);
    }

    #[test]
    fn test_memory_adjustment_structure() {
        let adjustment = MemoryAdjustment {
            historical_success_rate: 0.92,
            adjustment: 1,
            reasoning: "Similar tasks historically needed +1 subtask".to_string(),
        };

        assert_eq!(adjustment.historical_success_rate, 0.92);
        assert_eq!(adjustment.adjustment, 1);
    }
}
