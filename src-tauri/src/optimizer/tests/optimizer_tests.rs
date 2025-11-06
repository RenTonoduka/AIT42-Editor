//! Comprehensive integration tests for SubtaskOptimizer
//!
//! These tests verify:
//! - Integration with LLM estimator (mocked via cache)
//! - Ω-theory bounds validation
//! - Performance requirements (<500ms)
//! - Error handling
//! - Edge cases

use crate::optimizer::{
    ComplexityClass, ComplexityEstimate, OptimizationResult, OptimizerError, SubtaskOptimizer,
};
use std::time::{Duration, Instant};

/// Helper function to create test optimizer
fn create_optimizer() -> SubtaskOptimizer {
    SubtaskOptimizer::new("sk-ant-test-key-integration".to_string()).unwrap()
}

/// Helper to create mock estimate
fn mock_estimate(complexity: &str, subtasks: usize, confidence: f64) -> ComplexityEstimate {
    ComplexityEstimate {
        complexity_class: complexity.to_string(),
        reasoning: format!("This task has {} complexity", complexity),
        recommended_subtasks: subtasks,
        confidence,
    }
}

#[tokio::test]
async fn test_constant_complexity_task() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(1)", 1, 0.95);
    optimizer.insert_cached("Simple config change", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("Simple config change", 0)
        .await
        .unwrap();

    assert_eq!(result.complexity_class, ComplexityClass::Constant);
    assert_eq!(result.recommended_subtasks, 1);
    assert_eq!(result.confidence, 0.95);
}

#[tokio::test]
async fn test_logarithmic_complexity_task() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(log n)", 2, 0.88);
    optimizer.insert_cached("Implement binary search", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("Implement binary search", 0)
        .await
        .unwrap();

    assert_eq!(result.complexity_class, ComplexityClass::Logarithmic);
    assert_eq!(result.recommended_subtasks, 2);
    assert!((2..=3).contains(&result.recommended_subtasks));
}

#[tokio::test]
async fn test_linear_complexity_task() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(n)", 4, 0.85);
    optimizer.insert_cached("Implement CRUD API", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("Implement CRUD API", 0)
        .await
        .unwrap();

    assert_eq!(result.complexity_class, ComplexityClass::Linear);
    assert_eq!(result.recommended_subtasks, 4);
    assert!((3..=5).contains(&result.recommended_subtasks));
}

#[tokio::test]
async fn test_linearithmic_complexity_task() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(n log n)", 5, 0.82);
    optimizer.insert_cached("Implement merge sort", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("Implement merge sort", 0)
        .await
        .unwrap();

    assert_eq!(result.complexity_class, ComplexityClass::Linearithmic);
    assert_eq!(result.recommended_subtasks, 5);
    assert!((4..=6).contains(&result.recommended_subtasks));
}

#[tokio::test]
async fn test_quadratic_complexity_task() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(n²)", 7, 0.78);
    optimizer.insert_cached("Matrix multiplication", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("Matrix multiplication", 0)
        .await
        .unwrap();

    assert_eq!(result.complexity_class, ComplexityClass::Quadratic);
    assert_eq!(result.recommended_subtasks, 7);
    assert!((5..=10).contains(&result.recommended_subtasks));
}

#[tokio::test]
async fn test_exponential_complexity_task() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(2^n)", 12, 0.75);
    optimizer.insert_cached("Solve N-Queens problem", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("Solve N-Queens problem", 0)
        .await
        .unwrap();

    assert_eq!(result.complexity_class, ComplexityClass::Exponential);
    assert_eq!(result.recommended_subtasks, 12);
    assert!((8..=15).contains(&result.recommended_subtasks));
}

#[tokio::test]
async fn test_omega_bounds_enforcement_lower() {
    let optimizer = create_optimizer();

    // LLM suggests 1 subtask for Linear task (below Ω bounds of 3-5)
    let estimate = mock_estimate("Ω(n)", 1, 0.8);
    optimizer.insert_cached("Underestimated task", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("Underestimated task", 0)
        .await
        .unwrap();

    assert_eq!(result.complexity_class, ComplexityClass::Linear);
    assert_eq!(result.recommended_subtasks, 3); // Adjusted to lower bound
    assert!(result.reasoning.contains("Adjusted"));
}

#[tokio::test]
async fn test_omega_bounds_enforcement_upper() {
    let optimizer = create_optimizer();

    // LLM suggests 10 subtasks for Linear task (above Ω bounds of 3-5)
    let estimate = mock_estimate("Ω(n)", 10, 0.8);
    optimizer.insert_cached("Overestimated task", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("Overestimated task", 0)
        .await
        .unwrap();

    assert_eq!(result.complexity_class, ComplexityClass::Linear);
    assert_eq!(result.recommended_subtasks, 5); // Adjusted to upper bound
    assert!(result.reasoning.contains("Adjusted"));
}

#[tokio::test]
async fn test_omega_bounds_enforcement_all_classes() {
    let optimizer = create_optimizer();

    let test_cases = vec![
        ("Ω(1)", 5, 1, ComplexityClass::Constant, 1..=1), // Over
        ("Ω(log n)", 10, 3, ComplexityClass::Logarithmic, 2..=3), // Over
        ("Ω(n)", 1, 3, ComplexityClass::Linear, 3..=5),   // Under
        ("Ω(n log n)", 20, 6, ComplexityClass::Linearithmic, 4..=6), // Over
        ("Ω(n²)", 2, 5, ComplexityClass::Quadratic, 5..=10), // Under
        ("Ω(2^n)", 3, 8, ComplexityClass::Exponential, 8..=15), // Under
    ];

    for (i, (notation, llm_subtasks, expected, class, range)) in test_cases.iter().enumerate() {
        let task = format!("Bounds test {}", i);
        let estimate = mock_estimate(notation, *llm_subtasks, 0.8);
        optimizer.insert_cached(&task, 0, estimate);

        let result = optimizer.optimize_subtask_count(&task, 0).await.unwrap();

        assert_eq!(result.complexity_class, *class);
        assert_eq!(result.recommended_subtasks, *expected);
        assert!(range.contains(&result.recommended_subtasks));
    }
}

#[tokio::test]
async fn test_performance_cache_hit() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(n)", 4, 0.85);
    optimizer.insert_cached("Performance test", 0, estimate);

    let start = Instant::now();
    let _ = optimizer
        .optimize_subtask_count("Performance test", 0)
        .await
        .unwrap();
    let duration = start.elapsed();

    // Cache hit should be very fast (<50ms)
    assert!(
        duration < Duration::from_millis(50),
        "Cache hit took {:?}, expected <50ms",
        duration
    );
}

#[tokio::test]
async fn test_performance_multiple_operations() {
    let optimizer = create_optimizer();

    // Pre-warm cache with 10 tasks
    for i in 0..10 {
        let task = format!("Task {}", i);
        let estimate = mock_estimate("Ω(n)", 4, 0.85);
        optimizer.insert_cached(&task, 0, estimate);
    }

    let start = Instant::now();
    for i in 0..10 {
        let task = format!("Task {}", i);
        let _ = optimizer.optimize_subtask_count(&task, 0).await.unwrap();
    }
    let duration = start.elapsed();

    // 10 cache hits should complete in <500ms total
    assert!(
        duration < Duration::from_millis(500),
        "10 operations took {:?}, expected <500ms",
        duration
    );
}

#[tokio::test]
async fn test_error_empty_description() {
    let optimizer = create_optimizer();

    let result = optimizer.optimize_subtask_count("", 0).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        OptimizerError::InvalidInput(msg) => {
            assert!(msg.contains("empty"));
        }
        _ => panic!("Expected InvalidInput error"),
    }
}

#[tokio::test]
async fn test_error_whitespace_only_description() {
    let optimizer = create_optimizer();

    let result = optimizer.optimize_subtask_count("  \n\t  ", 0).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_very_long_description() {
    let optimizer = create_optimizer();

    let long_desc = "a".repeat(10000);
    let estimate = mock_estimate("Ω(n)", 4, 0.85);
    optimizer.insert_cached(&long_desc, 0, estimate);

    let result = optimizer.optimize_subtask_count(&long_desc, 0).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_special_characters_in_description() {
    let optimizer = create_optimizer();

    let special_desc = "Task with 特殊文字 and émojis 🚀 and symbols @#$%";
    let estimate = mock_estimate("Ω(n)", 4, 0.85);
    optimizer.insert_cached(special_desc, 0, estimate);

    let result = optimizer.optimize_subtask_count(special_desc, 0).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_current_subtasks_parameter() {
    let optimizer = create_optimizer();

    let task = "Same task different subtasks";

    // First estimate with 0 current subtasks
    let estimate1 = mock_estimate("Ω(n)", 4, 0.85);
    optimizer.insert_cached(task, 0, estimate1);

    // Second estimate with 5 current subtasks
    let estimate2 = mock_estimate("Ω(n)", 5, 0.80);
    optimizer.insert_cached(task, 5, estimate2);

    let result1 = optimizer.optimize_subtask_count(task, 0).await.unwrap();
    let result2 = optimizer.optimize_subtask_count(task, 5).await.unwrap();

    assert_eq!(result1.recommended_subtasks, 4);
    assert_eq!(result2.recommended_subtasks, 5);
}

#[tokio::test]
async fn test_cache_statistics() {
    let optimizer = create_optimizer();

    let initial_stats = optimizer.cache_stats();
    assert_eq!(initial_stats.total_requests, 0);
    assert_eq!(initial_stats.cache_hits, 0);

    // Pre-warm cache
    let estimate = mock_estimate("Ω(n)", 4, 0.85);
    optimizer.insert_cached("Stats test", 0, estimate);

    // Make requests
    let _ = optimizer.optimize_subtask_count("Stats test", 0).await;
    let _ = optimizer.optimize_subtask_count("Stats test", 0).await;

    let stats = optimizer.cache_stats();
    assert_eq!(stats.total_requests, 2);
    assert_eq!(stats.cache_hits, 2);
    assert_eq!(stats.cache_size, 1);
    assert_eq!(stats.hit_rate(), 1.0);
}

#[tokio::test]
async fn test_cache_clear() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(n)", 4, 0.85);
    optimizer.insert_cached("Clear test", 0, estimate);

    let stats_before = optimizer.cache_stats();
    assert_eq!(stats_before.cache_size, 1);

    optimizer.clear_cache();

    let stats_after = optimizer.cache_stats();
    assert_eq!(stats_after.cache_size, 0);
}

#[tokio::test]
async fn test_reasoning_completeness() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(n²)", 7, 0.88);
    optimizer.insert_cached("Reasoning test", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("Reasoning test", 0)
        .await
        .unwrap();

    // Verify reasoning contains all key components
    assert!(result.reasoning.contains("Ω(n²)")); // Complexity notation
    assert!(result.reasoning.contains("Quadratic")); // Class name
    assert!(result.reasoning.contains("5-10")); // Bounds
    assert!(result.reasoning.contains("88")); // Confidence percentage
    assert!(result.reasoning.contains("This task has Ω(n²) complexity")); // LLM reasoning
}

#[tokio::test]
async fn test_optimization_result_serialization() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(n)", 4, 0.85);
    optimizer.insert_cached("Serialization test", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("Serialization test", 0)
        .await
        .unwrap();

    // Serialize to JSON
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("Linear"));
    assert!(json.contains("\"recommended_subtasks\":4"));

    // Deserialize back
    let deserialized: OptimizationResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.complexity_class, result.complexity_class);
    assert_eq!(
        deserialized.recommended_subtasks,
        result.recommended_subtasks
    );
}

#[tokio::test]
async fn test_concurrent_requests() {
    let optimizer = create_optimizer();

    // Pre-warm cache with multiple tasks
    for i in 0..5 {
        let task = format!("Concurrent task {}", i);
        let estimate = mock_estimate("Ω(n)", 3 + i, 0.85);
        optimizer.insert_cached(&task, 0, estimate);
    }

    // Execute concurrent requests
    let mut handles = vec![];
    for i in 0..5 {
        let task = format!("Concurrent task {}", i);
        let optimizer_clone = create_optimizer();

        // Re-warm cache for each clone (in real usage, optimizer would be shared)
        let estimate = mock_estimate("Ω(n)", 3 + i, 0.85);
        optimizer_clone.insert_cached(&task, 0, estimate);

        let handle = tokio::spawn(async move {
            optimizer_clone.optimize_subtask_count(&task, 0).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_idempotency() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(n)", 4, 0.85);
    optimizer.insert_cached("Idempotent task", 0, estimate);

    // Run same optimization multiple times
    let result1 = optimizer
        .optimize_subtask_count("Idempotent task", 0)
        .await
        .unwrap();
    let result2 = optimizer
        .optimize_subtask_count("Idempotent task", 0)
        .await
        .unwrap();
    let result3 = optimizer
        .optimize_subtask_count("Idempotent task", 0)
        .await
        .unwrap();

    // Results should be identical
    assert_eq!(result1.complexity_class, result2.complexity_class);
    assert_eq!(result1.recommended_subtasks, result2.recommended_subtasks);
    assert_eq!(result2.complexity_class, result3.complexity_class);
    assert_eq!(result2.recommended_subtasks, result3.recommended_subtasks);
}

#[tokio::test]
async fn test_memory_adjustment_placeholder() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(n)", 4, 0.85);
    optimizer.insert_cached("Memory test", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("Memory test", 0)
        .await
        .unwrap();

    // v1.6.0: memory_adjustment should be None (placeholder)
    assert!(result.memory_adjustment.is_none());
}

#[tokio::test]
async fn test_llm_estimate_preserved() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(n)", 4, 0.85);
    optimizer.insert_cached("LLM preservation test", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("LLM preservation test", 0)
        .await
        .unwrap();

    // Original LLM estimate should be preserved
    assert!(result.llm_estimate.is_some());
    let llm = result.llm_estimate.unwrap();
    assert_eq!(llm.complexity_class, "Ω(n)");
    assert_eq!(llm.recommended_subtasks, 4);
    assert_eq!(llm.confidence, 0.85);
}

#[tokio::test]
async fn test_confidence_score_preserved() {
    let optimizer = create_optimizer();

    let confidences = vec![0.5, 0.75, 0.88, 0.95, 1.0];

    for (i, confidence) in confidences.iter().enumerate() {
        let task = format!("Confidence test {}", i);
        let estimate = mock_estimate("Ω(n)", 4, *confidence);
        optimizer.insert_cached(&task, 0, estimate);

        let result = optimizer.optimize_subtask_count(&task, 0).await.unwrap();
        assert_eq!(result.confidence, *confidence);
    }
}

#[tokio::test]
async fn test_edge_case_minimum_subtasks() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(1)", 1, 0.95);
    optimizer.insert_cached("Minimum subtasks", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("Minimum subtasks", 0)
        .await
        .unwrap();

    assert_eq!(result.recommended_subtasks, 1);
}

#[tokio::test]
async fn test_edge_case_maximum_subtasks() {
    let optimizer = create_optimizer();

    let estimate = mock_estimate("Ω(2^n)", 15, 0.85);
    optimizer.insert_cached("Maximum subtasks", 0, estimate);

    let result = optimizer
        .optimize_subtask_count("Maximum subtasks", 0)
        .await
        .unwrap();

    assert_eq!(result.recommended_subtasks, 15);
    assert!((8..=15).contains(&result.recommended_subtasks));
}

#[test]
fn test_optimizer_error_types() {
    // Test that all error types are properly constructed
    let _err1 = OptimizerError::InvalidInput("test".to_string());
    let _err2 = OptimizerError::Timeout(Duration::from_secs(1));

    // Test error display
    let err = OptimizerError::InvalidInput("test error".to_string());
    let display = format!("{}", err);
    assert!(display.contains("test error"));
}

#[test]
fn test_complexity_class_enum_coverage() {
    // Ensure all ComplexityClass variants are handled
    let classes = vec![
        ComplexityClass::Constant,
        ComplexityClass::Logarithmic,
        ComplexityClass::Linear,
        ComplexityClass::Linearithmic,
        ComplexityClass::Quadratic,
        ComplexityClass::Exponential,
    ];

    for class in classes {
        let range = class.to_subtask_range();
        assert!(*range.start() >= 1);
        assert!(*range.end() <= 15);
        assert!(*range.start() <= *range.end());
    }
}
