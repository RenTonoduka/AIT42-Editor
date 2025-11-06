//! Integration tests for A/B testing framework

use super::*;
use llm_estimator::ComplexityEstimate;
use omega_theory::ComplexityClass;

fn create_mock_estimate(
    complexity: &str,
    subtasks: usize,
    confidence: f64,
) -> ComplexityEstimate {
    ComplexityEstimate {
        complexity_class: complexity.to_string(),
        reasoning: "Mock test reasoning".to_string(),
        recommended_subtasks: subtasks,
        confidence,
    }
}

#[tokio::test]
async fn test_ab_test_runner_creation() {
    let strategy_a = Box::new(KeywordStrategy::new());
    let strategy_b_inner = LLMStrategy::new("sk-ant-test-key".to_string()).unwrap();
    let strategy_b = Box::new(strategy_b_inner);
    let test_cases = vec![TestCase::new(
        "test-1",
        "Test task",
        ComplexityClass::Linear,
        4,
        "Test",
    )];

    let runner = ABTestRunner::new(strategy_a, strategy_b, test_cases);
    assert_eq!(runner.test_cases.len(), 1);
}

#[tokio::test]
async fn test_keyword_strategy_on_test_cases() {
    let strategy = KeywordStrategy::new();
    let test_cases = get_all_test_cases();

    let mut correct_count = 0;

    for test_case in &test_cases {
        let result = strategy.optimize(&test_case.task_description).unwrap();

        if result.complexity_class == test_case.expected_complexity {
            correct_count += 1;
        }

        // All results should have low confidence
        assert_eq!(result.confidence, 0.6);

        // All results should be fast
        assert!(result.latency_ms < 10);
    }

    // Keyword strategy should get reasonable accuracy (40-100%)
    // Note: The carefully crafted keywords actually work quite well!
    let accuracy = correct_count as f64 / test_cases.len() as f64;
    println!("Keyword strategy accuracy: {:.1}%", accuracy * 100.0);
    assert!(accuracy > 0.4, "Accuracy {} should be > 0.4", accuracy);
}

#[tokio::test]
async fn test_llm_strategy_with_mocked_estimates() {
    let strategy = LLMStrategy::new("sk-ant-test-key".to_string()).unwrap();

    // Pre-warm cache with perfect estimates
    let test_cases = get_all_test_cases();
    for test_case in &test_cases {
        let mock_estimate = create_mock_estimate(
            &test_case.expected_complexity.to_string(),
            test_case.expected_subtasks,
            0.90,
        );
        strategy.insert_cached(&test_case.task_description, mock_estimate);
    }

    // Run strategy on all test cases
    let mut correct_count = 0;

    for test_case in &test_cases {
        let result = strategy.optimize_async(&test_case.task_description).await.unwrap();

        if result.complexity_class == test_case.expected_complexity {
            correct_count += 1;
        }

        // Should have high confidence
        assert!(result.confidence >= 0.85);
    }

    // With perfect mocks, should get 100% accuracy
    let accuracy = correct_count as f64 / test_cases.len() as f64;
    assert_eq!(accuracy, 1.0);
}

#[tokio::test]
async fn test_strategy_metrics_calculation() {
    let strategy_a = Box::new(KeywordStrategy::new());
    let strategy_b_inner = LLMStrategy::new("sk-ant-test-key".to_string()).unwrap();

    // Pre-warm LLM cache with perfect estimates
    let test_cases = get_all_test_cases();
    for test_case in &test_cases {
        let mock_estimate = create_mock_estimate(
            &test_case.expected_complexity.to_string(),
            test_case.expected_subtasks,
            0.90,
        );
        strategy_b_inner.insert_cached(&test_case.task_description, mock_estimate);
    }

    let strategy_b = Box::new(strategy_b_inner);

    let runner = ABTestRunner::new(strategy_a, strategy_b, test_cases);
    let result = runner.run().await.unwrap();

    // Verify metrics structure
    assert_eq!(result.strategy_a_metrics.total_cases, 30);
    assert_eq!(result.strategy_b_metrics.total_cases, 30);

    // Strategy A should have lower accuracy
    assert!(result.strategy_a_metrics.accuracy < 0.8);

    // Strategy B should have higher accuracy (perfect mocks)
    assert!(result.strategy_b_metrics.accuracy > 0.95);

    // Strategy A should be faster
    assert!(result.strategy_a_metrics.avg_latency_ms < result.strategy_b_metrics.avg_latency_ms);

    // Strategy B should have higher confidence
    assert!(
        result.strategy_b_metrics.avg_confidence > result.strategy_a_metrics.avg_confidence
    );
}

#[tokio::test]
async fn test_statistical_comparison() {
    let strategy_a = Box::new(KeywordStrategy::new());
    let strategy_b_inner = LLMStrategy::new("sk-ant-test-key".to_string()).unwrap();

    // Pre-warm LLM cache with perfect estimates
    let test_cases = get_all_test_cases();
    for test_case in &test_cases {
        let mock_estimate = create_mock_estimate(
            &test_case.expected_complexity.to_string(),
            test_case.expected_subtasks,
            0.90,
        );
        strategy_b_inner.insert_cached(&test_case.task_description, mock_estimate);
    }

    let strategy_b = Box::new(strategy_b_inner);

    let runner = ABTestRunner::new(strategy_a, strategy_b, test_cases);
    let result = runner.run().await.unwrap();

    // Verify comparison stats
    assert!(result.comparison.accuracy_diff > 0.0); // B better than A
    assert!(result.comparison.p_value < 0.05); // Statistically significant
    assert!(result.comparison.effect_size > 0.8); // Large effect
    assert!(result.comparison.is_significant);
    assert_eq!(result.comparison.effect_size_interpretation, "Large");

    // Winner should be Strategy B
    assert!(result.comparison.winner.contains("v1.6.0"));
}

#[tokio::test]
async fn test_per_complexity_metrics() {
    let strategy_a = Box::new(KeywordStrategy::new());
    let strategy_b_inner = LLMStrategy::new("sk-ant-test-key".to_string()).unwrap();

    let test_cases = get_all_test_cases();
    for test_case in &test_cases {
        let mock_estimate = create_mock_estimate(
            &test_case.expected_complexity.to_string(),
            test_case.expected_subtasks,
            0.90,
        );
        strategy_b_inner.insert_cached(&test_case.task_description, mock_estimate);
    }

    let strategy_b = Box::new(strategy_b_inner);

    let runner = ABTestRunner::new(strategy_a, strategy_b, test_cases);
    let result = runner.run().await.unwrap();

    // Verify per-complexity metrics exist for all classes
    let complexity_classes = vec![
        "Ω(1)",
        "Ω(log n)",
        "Ω(n)",
        "Ω(n log n)",
        "Ω(n²)",
        "Ω(2^n)",
    ];

    for complexity in complexity_classes {
        assert!(
            result
                .strategy_a_metrics
                .per_complexity_metrics
                .contains_key(complexity),
            "Missing metrics for {}",
            complexity
        );
        assert!(
            result
                .strategy_b_metrics
                .per_complexity_metrics
                .contains_key(complexity),
            "Missing metrics for {}",
            complexity
        );

        let metrics_a = &result.strategy_a_metrics.per_complexity_metrics[complexity];
        let metrics_b = &result.strategy_b_metrics.per_complexity_metrics[complexity];

        // Each complexity class should have 5 test cases
        assert_eq!(metrics_a.total, 5);
        assert_eq!(metrics_b.total, 5);

        // Strategy B should have perfect accuracy with mocked estimates
        assert_eq!(metrics_b.accuracy, 1.0);
    }
}

#[tokio::test]
async fn test_test_case_results() {
    let strategy_a = Box::new(KeywordStrategy::new());
    let strategy_b_inner = LLMStrategy::new("sk-ant-test-key".to_string()).unwrap();

    let test_cases = get_all_test_cases();
    for test_case in &test_cases {
        let mock_estimate = create_mock_estimate(
            &test_case.expected_complexity.to_string(),
            test_case.expected_subtasks,
            0.90,
        );
        strategy_b_inner.insert_cached(&test_case.task_description, mock_estimate);
    }

    let strategy_b = Box::new(strategy_b_inner);

    let runner = ABTestRunner::new(strategy_a, strategy_b, test_cases);
    let result = runner.run().await.unwrap();

    // Verify all test cases have results
    assert_eq!(result.strategy_a_metrics.results.len(), 30);
    assert_eq!(result.strategy_b_metrics.results.len(), 30);

    // Verify result structure
    for test_result in &result.strategy_a_metrics.results {
        assert!(!test_result.test_case_id.is_empty());
        assert!(!test_result.task_description.is_empty());
        assert!(!test_result.category.is_empty());
        assert!(test_result.confidence >= 0.0 && test_result.confidence <= 1.0);
    }
}

#[tokio::test]
async fn test_result_export_json() {
    let strategy_a = Box::new(KeywordStrategy::new());
    let strategy_b_inner = LLMStrategy::new("sk-ant-test-key".to_string()).unwrap();

    let test_cases = vec![TestCase::new(
        "test-1",
        "Test task",
        ComplexityClass::Linear,
        4,
        "Test",
    )];

    let mock_estimate = create_mock_estimate("Ω(n)", 4, 0.90);
    strategy_b_inner.insert_cached("Test task", mock_estimate);

    let strategy_b = Box::new(strategy_b_inner);

    let runner = ABTestRunner::new(strategy_a, strategy_b, test_cases);
    let result = runner.run().await.unwrap();

    // Test JSON export
    let json = result.to_json().unwrap();
    assert!(json.contains("strategy_a_metrics"));
    assert!(json.contains("strategy_b_metrics"));
    assert!(json.contains("comparison"));

    // Verify it can be deserialized
    let deserialized: ABTestResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.metadata.total_test_cases, 1);
}

#[tokio::test]
async fn test_result_export_csv() {
    let strategy_a = Box::new(KeywordStrategy::new());
    let strategy_b_inner = LLMStrategy::new("sk-ant-test-key".to_string()).unwrap();

    let test_cases = vec![TestCase::new(
        "test-1",
        "Test task",
        ComplexityClass::Linear,
        4,
        "Test",
    )];

    let mock_estimate = create_mock_estimate("Ω(n)", 4, 0.90);
    strategy_b_inner.insert_cached("Test task", mock_estimate);

    let strategy_b = Box::new(strategy_b_inner);

    let runner = ABTestRunner::new(strategy_a, strategy_b, test_cases);
    let result = runner.run().await.unwrap();

    // Test CSV export
    let csv = result.to_csv().unwrap();
    assert!(csv.contains("test_case_id,category,expected_complexity"));
    assert!(csv.contains("test-1"));
    assert!(csv.contains("Test"));

    // Should have header + 1 data row
    assert_eq!(csv.lines().count(), 2);
}

#[tokio::test]
async fn test_result_summary() {
    let strategy_a = Box::new(KeywordStrategy::new());
    let strategy_b_inner = LLMStrategy::new("sk-ant-test-key".to_string()).unwrap();

    let test_cases = vec![TestCase::new(
        "test-1",
        "Test task",
        ComplexityClass::Linear,
        4,
        "Test",
    )];

    let mock_estimate = create_mock_estimate("Ω(n)", 4, 0.90);
    strategy_b_inner.insert_cached("Test task", mock_estimate);

    let strategy_b = Box::new(strategy_b_inner);

    let runner = ABTestRunner::new(strategy_a, strategy_b, test_cases);
    let result = runner.run().await.unwrap();

    // Test summary generation
    let summary = result.summary();
    assert!(summary.contains("A/B Test Results"));
    assert!(summary.contains("Strategy A:"));
    assert!(summary.contains("Strategy B:"));
    assert!(summary.contains("Comparison:"));
    assert!(summary.contains("Winner:"));
    assert!(summary.contains("Accuracy"));
    assert!(summary.contains("P-value"));
    assert!(summary.contains("Effect Size"));
}

#[tokio::test]
async fn test_metadata() {
    let strategy_a = Box::new(KeywordStrategy::new());
    let strategy_b_inner = LLMStrategy::new("sk-ant-test-key".to_string()).unwrap();

    let test_cases = vec![TestCase::new(
        "test-1",
        "Test task",
        ComplexityClass::Linear,
        4,
        "Test",
    )];

    let mock_estimate = create_mock_estimate("Ω(n)", 4, 0.90);
    strategy_b_inner.insert_cached("Test task", mock_estimate);

    let strategy_b = Box::new(strategy_b_inner);

    let runner = ABTestRunner::new(strategy_a, strategy_b, test_cases);
    let result = runner.run().await.unwrap();

    // Verify metadata
    assert_eq!(result.metadata.total_test_cases, 1);
    assert!(result.metadata.total_duration_ms > 0);
    assert!(!result.metadata.timestamp.is_empty());

    // Verify timestamp is valid RFC3339
    assert!(chrono::DateTime::parse_from_rfc3339(&result.metadata.timestamp).is_ok());
}

#[tokio::test]
async fn test_confidence_interval() {
    let strategy_a = Box::new(KeywordStrategy::new());
    let strategy_b_inner = LLMStrategy::new("sk-ant-test-key".to_string()).unwrap();

    let test_cases = get_all_test_cases();
    for test_case in &test_cases {
        let mock_estimate = create_mock_estimate(
            &test_case.expected_complexity.to_string(),
            test_case.expected_subtasks,
            0.90,
        );
        strategy_b_inner.insert_cached(&test_case.task_description, mock_estimate);
    }

    let strategy_b = Box::new(strategy_b_inner);

    let runner = ABTestRunner::new(strategy_a, strategy_b, test_cases);
    let result = runner.run().await.unwrap();

    // Verify confidence interval
    let ci = result.comparison.accuracy_ci;
    assert!(ci[0] < ci[1]); // Lower < Upper
    assert!(ci[0] > 0.0); // Should be positive (B > A)
    assert!(ci[1] < 1.0); // Should be less than 100%
}

#[test]
fn test_optimization_result_serialization() {
    let result = OptimizationResult {
        strategy_name: "Test Strategy".to_string(),
        complexity_class: ComplexityClass::Linear,
        recommended_subtasks: 4,
        confidence: 0.85,
        reasoning: "Test reasoning".to_string(),
        latency_ms: 100,
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: OptimizationResult = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.strategy_name, "Test Strategy");
    assert_eq!(deserialized.complexity_class, ComplexityClass::Linear);
}

#[test]
fn test_comparison_stats_serialization() {
    let stats = ComparisonStats {
        accuracy_diff: 0.3,
        latency_diff: 1500,
        confidence_diff: 0.25,
        p_value: 0.001,
        effect_size: 1.2,
        winner: "Strategy B".to_string(),
        is_significant: true,
        effect_size_interpretation: "Large".to_string(),
        accuracy_ci: [0.25, 0.35],
    };

    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: ComparisonStats = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.winner, "Strategy B");
    assert!(deserialized.is_significant);
}
