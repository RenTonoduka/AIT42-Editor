//! A/B Testing Framework for AIT42-Editor v1.6.0
//!
//! This module provides comprehensive A/B testing infrastructure to compare:
//! - **Strategy A (v1.5.0)**: Keyword-based complexity estimation
//! - **Strategy B (v1.6.0)**: LLM + Ω-theory optimization
//!
//! # Architecture
//!
//! ```text
//! ABTestRunner
//!     ├── KeywordStrategy (v1.5.0)
//!     │   └── Simple keyword matching
//!     ├── LLMStrategy (v1.6.0)
//!     │   └── SubtaskOptimizer (LLM + Ω-theory)
//!     ├── TestCases (30 ground truth labels)
//!     └── Statistics (t-test, Cohen's d, CI)
//! ```
//!
//! # Quick Start
//!
//! ```no_run
//! use ab_test::ABTestRunner;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let runner = ABTestRunner::from_env()?;
//! let result = runner.run().await?;
//!
//! println!("Winner: {}", result.comparison.winner);
//! println!("Accuracy improvement: {:.1}%", result.comparison.accuracy_diff * 100.0);
//! println!("P-value: {:.4}", result.comparison.p_value);
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Expectations
//!
//! | Metric | v1.5.0 (Keyword) | v1.6.0 (LLM+Ω) | Improvement |
//! |--------|------------------|----------------|-------------|
//! | Accuracy | ~60% | ~90% | +50% |
//! | Latency | <5ms | ~1-2s | -400x |
//! | Confidence | 0.6 | 0.85 | +42% |

use anyhow::Result;
use omega_theory::ComplexityClass;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

pub mod keyword_strategy;
pub mod llm_strategy;
pub mod statistics;
pub mod test_cases;

#[cfg(test)]
mod tests;

pub use keyword_strategy::KeywordStrategy;
pub use llm_strategy::LLMStrategy;
pub use statistics::ComparisonStats;
pub use test_cases::{TestCase, get_all_test_cases};

/// Optimization strategy trait
///
/// Implement this trait to create custom optimization strategies for A/B testing.
pub trait OptimizationStrategy: Send + Sync {
    /// Optimize subtask count for a task
    ///
    /// # Arguments
    ///
    /// * `task_description` - The task to analyze
    ///
    /// # Returns
    ///
    /// Optimization result with complexity class, subtask count, confidence, and latency
    fn optimize(&self, task_description: &str) -> Result<OptimizationResult>;

    /// Strategy name for reporting
    fn name(&self) -> &str;
}

/// Result of a single optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Strategy name (e.g., "v1.5.0 Keyword Matching")
    pub strategy_name: String,

    /// Detected complexity class
    pub complexity_class: ComplexityClass,

    /// Recommended subtask count
    pub recommended_subtasks: usize,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Human-readable reasoning
    pub reasoning: String,

    /// Latency in milliseconds
    pub latency_ms: u64,
}

/// Result of testing a single strategy on all test cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    /// Strategy name
    pub name: String,

    /// Overall accuracy (0.0-1.0)
    pub accuracy: f64,

    /// Average latency in milliseconds
    pub avg_latency_ms: u64,

    /// Average confidence (0.0-1.0)
    pub avg_confidence: f64,

    /// Total test cases
    pub total_cases: usize,

    /// Correct predictions
    pub correct_predictions: usize,

    /// Per-complexity class metrics
    pub per_complexity_metrics: HashMap<String, ComplexityMetrics>,

    /// Individual test case results
    pub results: Vec<TestCaseResult>,
}

/// Metrics for a specific complexity class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Complexity class name (e.g., "Ω(n)")
    pub complexity: String,

    /// Accuracy for this complexity class
    pub accuracy: f64,

    /// Number of test cases
    pub total: usize,

    /// Correct predictions
    pub correct: usize,
}

/// Result of running a single test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    /// Test case ID
    pub test_case_id: String,

    /// Task description
    pub task_description: String,

    /// Expected complexity class
    pub expected_complexity: ComplexityClass,

    /// Predicted complexity class
    pub predicted_complexity: ComplexityClass,

    /// Expected subtask count
    pub expected_subtasks: usize,

    /// Predicted subtask count
    pub predicted_subtasks: usize,

    /// Was the complexity classification correct?
    pub is_correct: bool,

    /// Confidence score
    pub confidence: f64,

    /// Latency in milliseconds
    pub latency_ms: u64,

    /// Category (e.g., "UI", "Backend")
    pub category: String,
}

/// Complete A/B test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestResult {
    /// Metrics for Strategy A (v1.5.0)
    pub strategy_a_metrics: StrategyMetrics,

    /// Metrics for Strategy B (v1.6.0)
    pub strategy_b_metrics: StrategyMetrics,

    /// Statistical comparison
    pub comparison: ComparisonStats,

    /// Test metadata
    pub metadata: TestMetadata,
}

/// Test execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetadata {
    /// Number of test cases
    pub total_test_cases: usize,

    /// Total execution time in milliseconds
    pub total_duration_ms: u64,

    /// Timestamp when test started
    pub timestamp: String,
}

/// A/B test runner
///
/// Orchestrates the execution of two optimization strategies on a set of test cases
/// and produces comprehensive comparison metrics.
pub struct ABTestRunner {
    strategy_a: Box<dyn OptimizationStrategy>,
    strategy_b: Box<dyn OptimizationStrategy>,
    test_cases: Vec<TestCase>,
}

impl ABTestRunner {
    /// Create a new A/B test runner with custom strategies
    ///
    /// # Arguments
    ///
    /// * `strategy_a` - First strategy (typically v1.5.0 baseline)
    /// * `strategy_b` - Second strategy (typically v1.6.0 LLM)
    /// * `test_cases` - Test cases with ground truth labels
    pub fn new(
        strategy_a: Box<dyn OptimizationStrategy>,
        strategy_b: Box<dyn OptimizationStrategy>,
        test_cases: Vec<TestCase>,
    ) -> Self {
        Self {
            strategy_a,
            strategy_b,
            test_cases,
        }
    }

    /// Create default A/B test runner from environment
    ///
    /// - Strategy A: Keyword-based (v1.5.0)
    /// - Strategy B: LLM + Ω-theory (v1.6.0)
    /// - Test cases: All 30 ground truth cases
    ///
    /// Requires `ANTHROPIC_API_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// Returns error if API key is not set or invalid
    pub fn from_env() -> Result<Self> {
        let strategy_a = Box::new(KeywordStrategy::new());
        let strategy_b = Box::new(LLMStrategy::from_env()?);
        let test_cases = get_all_test_cases();

        Ok(Self::new(strategy_a, strategy_b, test_cases))
    }

    /// Run the A/B test
    ///
    /// Executes both strategies on all test cases and performs statistical analysis.
    ///
    /// # Returns
    ///
    /// Complete test results with metrics and statistical comparison
    pub async fn run(self) -> Result<ABTestResult> {
        let start = std::time::Instant::now();
        let timestamp = chrono::Utc::now().to_rfc3339();

        info!("Starting A/B test with {} test cases", self.test_cases.len());

        // Run Strategy A
        info!("Running Strategy A: {}", self.strategy_a.name());
        let strategy_a_metrics = self.run_strategy(&*self.strategy_a).await?;

        // Run Strategy B
        info!("Running Strategy B: {}", self.strategy_b.name());
        let strategy_b_metrics = self.run_strategy(&*self.strategy_b).await?;

        // Perform statistical comparison
        info!("Calculating statistical comparison");
        let comparison = self.compare_strategies(&strategy_a_metrics, &strategy_b_metrics)?;

        let total_duration_ms = start.elapsed().as_millis() as u64;

        info!(
            "A/B test complete. Winner: {}. Accuracy diff: {:.1}%. P-value: {:.4}",
            comparison.winner,
            comparison.accuracy_diff * 100.0,
            comparison.p_value
        );

        Ok(ABTestResult {
            strategy_a_metrics,
            strategy_b_metrics,
            comparison,
            metadata: TestMetadata {
                total_test_cases: self.test_cases.len(),
                total_duration_ms,
                timestamp,
            },
        })
    }

    /// Run a single strategy on all test cases
    async fn run_strategy(
        &self,
        strategy: &dyn OptimizationStrategy,
    ) -> Result<StrategyMetrics> {
        let mut results = Vec::new();
        let mut total_latency = 0u64;
        let mut total_confidence = 0.0;
        let mut correct_count = 0usize;

        for test_case in &self.test_cases {
            // Run optimization
            let opt_result = match strategy.optimize(&test_case.task_description) {
                Ok(r) => r,
                Err(e) => {
                    warn!(
                        "Strategy {} failed on test case {}: {}",
                        strategy.name(),
                        test_case.id,
                        e
                    );
                    continue;
                }
            };

            // Check if correct
            let is_correct = opt_result.complexity_class == test_case.expected_complexity;
            if is_correct {
                correct_count += 1;
            }

            total_latency += opt_result.latency_ms;
            total_confidence += opt_result.confidence;

            results.push(TestCaseResult {
                test_case_id: test_case.id.clone(),
                task_description: test_case.task_description.clone(),
                expected_complexity: test_case.expected_complexity,
                predicted_complexity: opt_result.complexity_class,
                expected_subtasks: test_case.expected_subtasks,
                predicted_subtasks: opt_result.recommended_subtasks,
                is_correct,
                confidence: opt_result.confidence,
                latency_ms: opt_result.latency_ms,
                category: test_case.category.clone(),
            });
        }

        let total_cases = results.len();
        let accuracy = if total_cases > 0 {
            correct_count as f64 / total_cases as f64
        } else {
            0.0
        };

        let avg_latency_ms = if total_cases > 0 {
            total_latency / total_cases as u64
        } else {
            0
        };

        let avg_confidence = if total_cases > 0 {
            total_confidence / total_cases as f64
        } else {
            0.0
        };

        // Calculate per-complexity metrics
        let per_complexity_metrics = self.calculate_per_complexity_metrics(&results);

        Ok(StrategyMetrics {
            name: strategy.name().to_string(),
            accuracy,
            avg_latency_ms,
            avg_confidence,
            total_cases,
            correct_predictions: correct_count,
            per_complexity_metrics,
            results,
        })
    }

    /// Calculate metrics for each complexity class
    fn calculate_per_complexity_metrics(
        &self,
        results: &[TestCaseResult],
    ) -> HashMap<String, ComplexityMetrics> {
        let mut metrics: HashMap<String, (usize, usize)> = HashMap::new();

        for result in results {
            let complexity_str = result.expected_complexity.to_string();
            let entry = metrics.entry(complexity_str.clone()).or_insert((0, 0));
            entry.0 += 1; // total
            if result.is_correct {
                entry.1 += 1; // correct
            }
        }

        metrics
            .into_iter()
            .map(|(complexity, (total, correct))| {
                let accuracy = if total > 0 {
                    correct as f64 / total as f64
                } else {
                    0.0
                };

                (
                    complexity.clone(),
                    ComplexityMetrics {
                        complexity,
                        accuracy,
                        total,
                        correct,
                    },
                )
            })
            .collect()
    }

    /// Compare two strategies statistically
    fn compare_strategies(
        &self,
        metrics_a: &StrategyMetrics,
        metrics_b: &StrategyMetrics,
    ) -> Result<ComparisonStats> {
        // Calculate differences
        let accuracy_diff = metrics_b.accuracy - metrics_a.accuracy;
        let latency_diff = metrics_b.avg_latency_ms as i64 - metrics_a.avg_latency_ms as i64;
        let confidence_diff = metrics_b.avg_confidence - metrics_a.avg_confidence;

        // Prepare paired scores for statistical tests
        // Score = 1.0 if correct, 0.0 if incorrect
        let scores_a: Vec<f64> = metrics_a
            .results
            .iter()
            .map(|r| if r.is_correct { 1.0 } else { 0.0 })
            .collect();

        let scores_b: Vec<f64> = metrics_b
            .results
            .iter()
            .map(|r| if r.is_correct { 1.0 } else { 0.0 })
            .collect();

        // Perform statistical tests (handle small sample sizes)
        let (p_value, effect_size, accuracy_ci) = if scores_a.len() < 2 {
            // Not enough samples for statistical tests, use simple comparison
            (
                if accuracy_diff.abs() > 0.0 { 0.5 } else { 1.0 }, // No significance with small n
                0.0, // No effect size
                [accuracy_diff, accuracy_diff], // Point estimate
            )
        } else {
            (
                statistics::paired_t_test(&scores_a, &scores_b)?,
                statistics::cohens_d(&scores_a, &scores_b)?,
                statistics::confidence_interval(&scores_a, &scores_b)?,
            )
        };

        // Determine winner
        let is_significant = p_value < 0.05;
        let winner = if is_significant && accuracy_diff > 0.0 {
            metrics_b.name.clone()
        } else if is_significant && accuracy_diff < 0.0 {
            metrics_a.name.clone()
        } else {
            "Tie (no significant difference)".to_string()
        };

        let effect_size_interpretation = statistics::interpret_effect_size(effect_size);

        Ok(ComparisonStats {
            accuracy_diff,
            latency_diff,
            confidence_diff,
            p_value,
            effect_size,
            winner,
            is_significant,
            effect_size_interpretation,
            accuracy_ci,
        })
    }
}

impl ABTestResult {
    /// Export results to JSON string
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Export results to CSV string
    pub fn to_csv(&self) -> Result<String> {
        let mut csv = String::new();

        // Header
        csv.push_str("test_case_id,category,expected_complexity,strategy_a_prediction,strategy_a_correct,strategy_b_prediction,strategy_b_correct\n");

        // Data rows
        for (result_a, result_b) in self
            .strategy_a_metrics
            .results
            .iter()
            .zip(&self.strategy_b_metrics.results)
        {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                result_a.test_case_id,
                result_a.category,
                result_a.expected_complexity,
                result_a.predicted_complexity,
                result_a.is_correct,
                result_b.predicted_complexity,
                result_b.is_correct
            ));
        }

        Ok(csv)
    }

    /// Generate a summary report
    pub fn summary(&self) -> String {
        format!(
            r#"
=== A/B Test Results ===

Strategy A: {}
- Accuracy: {:.1}%
- Avg Latency: {}ms
- Avg Confidence: {:.2}

Strategy B: {}
- Accuracy: {:.1}%
- Avg Latency: {}ms
- Avg Confidence: {:.2}

Comparison:
- Accuracy Improvement: {:+.1}%
- Latency Change: {:+}ms
- Confidence Change: {:+.2}
- P-value: {:.4} ({})
- Effect Size: {:.2} ({})
- Winner: {}

Total Test Cases: {}
Total Duration: {}ms
"#,
            self.strategy_a_metrics.name,
            self.strategy_a_metrics.accuracy * 100.0,
            self.strategy_a_metrics.avg_latency_ms,
            self.strategy_a_metrics.avg_confidence,
            self.strategy_b_metrics.name,
            self.strategy_b_metrics.accuracy * 100.0,
            self.strategy_b_metrics.avg_latency_ms,
            self.strategy_b_metrics.avg_confidence,
            self.comparison.accuracy_diff * 100.0,
            self.comparison.latency_diff,
            self.comparison.confidence_diff,
            self.comparison.p_value,
            if self.comparison.is_significant {
                "significant"
            } else {
                "not significant"
            },
            self.comparison.effect_size,
            self.comparison.effect_size_interpretation,
            self.comparison.winner,
            self.metadata.total_test_cases,
            self.metadata.total_duration_ms
        )
    }
}
