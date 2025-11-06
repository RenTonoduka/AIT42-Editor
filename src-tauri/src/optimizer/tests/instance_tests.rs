//! Integration tests for Instance Calculator
//!
//! This module tests the InstanceCalculator in isolation and in integration
//! with SubtaskOptimizer.

use crate::optimizer::{
    ComplexityClass, InstanceCalculation, InstanceCalculator, SubtaskOptimizer,
};

// ============================================================================
// UNIT TESTS - Basic Functionality
// ============================================================================

#[test]
fn test_new_calculator_has_default_max() {
    let calculator = InstanceCalculator::new();
    assert_eq!(calculator.max_instances(), 10);
}

#[test]
fn test_default_trait_implementation() {
    let calculator = InstanceCalculator::default();
    assert_eq!(calculator.max_instances(), 10);
}

#[test]
fn test_custom_max_instances() {
    let calculator = InstanceCalculator::with_max_instances(15);
    assert_eq!(calculator.max_instances(), 15);
}

#[test]
#[should_panic(expected = "max_instances must be between 1 and 100")]
fn test_zero_max_instances_panics() {
    let _ = InstanceCalculator::with_max_instances(0);
}

#[test]
#[should_panic(expected = "max_instances must be between 1 and 100")]
fn test_excessive_max_instances_panics() {
    let _ = InstanceCalculator::with_max_instances(101);
}

#[test]
fn test_boundary_max_instances_valid() {
    // Test boundary values that should work
    let calc1 = InstanceCalculator::with_max_instances(1);
    assert_eq!(calc1.max_instances(), 1);

    let calc100 = InstanceCalculator::with_max_instances(100);
    assert_eq!(calc100.max_instances(), 100);
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn test_zero_subtasks() {
    let calculator = InstanceCalculator::new();
    let result = calculator.calculate_instances(ComplexityClass::Linear, 0);

    assert_eq!(result.recommended_instances, 0);
    assert_eq!(result.subtasks_per_instance, 0.0);
    assert_eq!(result.complexity_class, ComplexityClass::Linear);
    assert!(!result.resource_constrained);
    assert!(result.reasoning.contains("No subtasks"));
}

#[test]
fn test_single_subtask_all_complexities() {
    let calculator = InstanceCalculator::new();

    let complexities = vec![
        ComplexityClass::Constant,
        ComplexityClass::Logarithmic,
        ComplexityClass::Linear,
        ComplexityClass::Linearithmic,
        ComplexityClass::Quadratic,
        ComplexityClass::Exponential,
    ];

    for complexity in complexities {
        let result = calculator.calculate_instances(complexity, 1);

        assert_eq!(
            result.recommended_instances, 1,
            "Single subtask should always yield 1 instance for {:?}",
            complexity
        );
        assert_eq!(result.subtasks_per_instance, 1.0);
        assert_eq!(result.complexity_class, complexity);
        assert!(!result.resource_constrained);
    }
}

#[test]
fn test_large_subtask_count() {
    let calculator = InstanceCalculator::new();

    // 1000 subtasks should hit max constraint for all complexity classes
    let result = calculator.calculate_instances(ComplexityClass::Exponential, 1000);

    assert_eq!(result.recommended_instances, 10);
    assert_eq!(result.subtasks_per_instance, 100.0);
    assert!(result.resource_constrained);
    assert!(result.reasoning.contains("Resource constrained"));
}

// ============================================================================
// CONSTANT COMPLEXITY - Ω(1)
// ============================================================================

#[test]
fn test_constant_complexity_basic() {
    let calculator = InstanceCalculator::new();

    // 1 subtask → 1 instance (trivial)
    let result = calculator.calculate_instances(ComplexityClass::Constant, 1);
    assert_eq!(result.recommended_instances, 1);
    assert_eq!(result.subtasks_per_instance, 1.0);
    assert!(!result.resource_constrained);
}

#[test]
fn test_constant_complexity_multiple() {
    let calculator = InstanceCalculator::new();

    // 5 subtasks → 5 instances (1 per instance, no batching)
    let result = calculator.calculate_instances(ComplexityClass::Constant, 5);
    assert_eq!(result.recommended_instances, 5);
    assert_eq!(result.subtasks_per_instance, 1.0);
    assert!(!result.resource_constrained);
}

#[test]
fn test_constant_complexity_max_constraint() {
    let calculator = InstanceCalculator::new();

    // 15 subtasks → 10 instances (hit max limit)
    let result = calculator.calculate_instances(ComplexityClass::Constant, 15);
    assert_eq!(result.recommended_instances, 10);
    assert_eq!(result.subtasks_per_instance, 1.5);
    assert!(result.resource_constrained);
}

// ============================================================================
// LOGARITHMIC COMPLEXITY - Ω(log n)
// ============================================================================

#[test]
fn test_logarithmic_complexity_basic() {
    let calculator = InstanceCalculator::new();

    // 1 subtask → 1 instance
    let result = calculator.calculate_instances(ComplexityClass::Logarithmic, 1);
    assert_eq!(result.recommended_instances, 1);
    assert_eq!(result.subtasks_per_instance, 1.0);
}

#[test]
fn test_logarithmic_complexity_multiple() {
    let calculator = InstanceCalculator::new();

    // 5 subtasks → 5 instances (1 per instance)
    let result = calculator.calculate_instances(ComplexityClass::Logarithmic, 5);
    assert_eq!(result.recommended_instances, 5);
    assert_eq!(result.subtasks_per_instance, 1.0);
}

#[test]
fn test_logarithmic_complexity_max_constraint() {
    let calculator = InstanceCalculator::new();

    // 20 subtasks → 10 instances (hit max)
    let result = calculator.calculate_instances(ComplexityClass::Logarithmic, 20);
    assert_eq!(result.recommended_instances, 10);
    assert_eq!(result.subtasks_per_instance, 2.0);
    assert!(result.resource_constrained);
}

// ============================================================================
// LINEAR COMPLEXITY - Ω(n)
// ============================================================================

#[test]
fn test_linear_complexity_basic() {
    let calculator = InstanceCalculator::new();

    // 3 subtasks with 1.5 per instance → ceil(3/1.5) = 2 instances
    let result = calculator.calculate_instances(ComplexityClass::Linear, 3);
    assert_eq!(result.recommended_instances, 2);
    assert_eq!(result.subtasks_per_instance, 1.5);
    assert!(!result.resource_constrained);
}

#[test]
fn test_linear_complexity_multiple() {
    let calculator = InstanceCalculator::new();

    // 5 subtasks → ceil(5/1.5) = 4 instances
    let result = calculator.calculate_instances(ComplexityClass::Linear, 5);
    assert_eq!(result.recommended_instances, 4);
    assert!((result.subtasks_per_instance - 1.25).abs() < 0.01);
}

#[test]
fn test_linear_complexity_max_constraint() {
    let calculator = InstanceCalculator::new();

    // 50 subtasks → ceil(50/1.5) = 34, but capped at 10
    let result = calculator.calculate_instances(ComplexityClass::Linear, 50);
    assert_eq!(result.recommended_instances, 10);
    assert_eq!(result.subtasks_per_instance, 5.0);
    assert!(result.resource_constrained);
    assert!(result.reasoning.contains("34 instances")); // Unconstrained count
}

#[test]
fn test_linear_complexity_exactly_at_max() {
    let calculator = InstanceCalculator::new();

    // 15 subtasks → ceil(15/1.5) = 10 instances (exactly at max, not constrained)
    let result = calculator.calculate_instances(ComplexityClass::Linear, 15);
    assert_eq!(result.recommended_instances, 10);
    assert_eq!(result.subtasks_per_instance, 1.5);
    assert!(!result.resource_constrained); // Not constrained!
}

// ============================================================================
// LINEARITHMIC COMPLEXITY - Ω(n log n)
// ============================================================================

#[test]
fn test_linearithmic_complexity_basic() {
    let calculator = InstanceCalculator::new();

    // 5 subtasks with 2.5 per instance → ceil(5/2.5) = 2 instances
    let result = calculator.calculate_instances(ComplexityClass::Linearithmic, 5);
    assert_eq!(result.recommended_instances, 2);
    assert_eq!(result.subtasks_per_instance, 2.5);
}

#[test]
fn test_linearithmic_complexity_exact_multiple() {
    let calculator = InstanceCalculator::new();

    // 10 subtasks → ceil(10/2.5) = 4 instances
    let result = calculator.calculate_instances(ComplexityClass::Linearithmic, 10);
    assert_eq!(result.recommended_instances, 4);
    assert_eq!(result.subtasks_per_instance, 2.5);
}

#[test]
fn test_linearithmic_complexity_fractional() {
    let calculator = InstanceCalculator::new();

    // 7 subtasks → ceil(7/2.5) = ceil(2.8) = 3 instances
    let result = calculator.calculate_instances(ComplexityClass::Linearithmic, 7);
    assert_eq!(result.recommended_instances, 3);
    assert!((result.subtasks_per_instance - 2.33).abs() < 0.01);
}

// ============================================================================
// QUADRATIC COMPLEXITY - Ω(n²)
// ============================================================================

#[test]
fn test_quadratic_complexity_basic() {
    let calculator = InstanceCalculator::new();

    // 8 subtasks with 4.0 per instance → ceil(8/4.0) = 2 instances
    let result = calculator.calculate_instances(ComplexityClass::Quadratic, 8);
    assert_eq!(result.recommended_instances, 2);
    assert_eq!(result.subtasks_per_instance, 4.0);
}

#[test]
fn test_quadratic_complexity_multiple() {
    let calculator = InstanceCalculator::new();

    // 10 subtasks → ceil(10/4.0) = 3 instances
    let result = calculator.calculate_instances(ComplexityClass::Quadratic, 10);
    assert_eq!(result.recommended_instances, 3);
    assert!((result.subtasks_per_instance - 3.33).abs() < 0.01);
}

#[test]
fn test_quadratic_complexity_small() {
    let calculator = InstanceCalculator::new();

    // 5 subtasks → ceil(5/4.0) = 2 instances
    let result = calculator.calculate_instances(ComplexityClass::Quadratic, 5);
    assert_eq!(result.recommended_instances, 2);
    assert_eq!(result.subtasks_per_instance, 2.5);
}

// ============================================================================
// EXPONENTIAL COMPLEXITY - Ω(2^n)
// ============================================================================

#[test]
fn test_exponential_complexity_small() {
    let calculator = InstanceCalculator::new();

    // 5 subtasks with 6.5 per instance → ceil(5/6.5) = 1 instance
    let result = calculator.calculate_instances(ComplexityClass::Exponential, 5);
    assert_eq!(result.recommended_instances, 1);
    assert_eq!(result.subtasks_per_instance, 5.0);
}

#[test]
fn test_exponential_complexity_medium() {
    let calculator = InstanceCalculator::new();

    // 15 subtasks → ceil(15/6.5) = 3 instances
    let result = calculator.calculate_instances(ComplexityClass::Exponential, 15);
    assert_eq!(result.recommended_instances, 3);
    assert_eq!(result.subtasks_per_instance, 5.0);
}

#[test]
fn test_exponential_complexity_large() {
    let calculator = InstanceCalculator::new();

    // 20 subtasks → ceil(20/6.5) = 4 instances
    let result = calculator.calculate_instances(ComplexityClass::Exponential, 20);
    assert_eq!(result.recommended_instances, 4);
    assert_eq!(result.subtasks_per_instance, 5.0);
}

#[test]
fn test_exponential_complexity_max_constraint() {
    let calculator = InstanceCalculator::new();

    // 100 subtasks → ceil(100/6.5) = 16, but capped at 10
    let result = calculator.calculate_instances(ComplexityClass::Exponential, 100);
    assert_eq!(result.recommended_instances, 10);
    assert_eq!(result.subtasks_per_instance, 10.0);
    assert!(result.resource_constrained);
}

// ============================================================================
// CUSTOM MAX INSTANCES TESTS
// ============================================================================

#[test]
fn test_custom_max_5_instances() {
    let calculator = InstanceCalculator::with_max_instances(5);

    // 20 subtasks with Linear → ceil(20/1.5) = 14, capped at 5
    let result = calculator.calculate_instances(ComplexityClass::Linear, 20);
    assert_eq!(result.recommended_instances, 5);
    assert_eq!(result.subtasks_per_instance, 4.0);
    assert!(result.resource_constrained);
}

#[test]
fn test_custom_max_1_instance() {
    let calculator = InstanceCalculator::with_max_instances(1);

    // All subtasks must go to 1 instance
    let result = calculator.calculate_instances(ComplexityClass::Linear, 50);
    assert_eq!(result.recommended_instances, 1);
    assert_eq!(result.subtasks_per_instance, 50.0);
    assert!(result.resource_constrained);
}

#[test]
fn test_custom_max_100_instances() {
    let calculator = InstanceCalculator::with_max_instances(100);

    // Large calculation that doesn't hit constraint
    let result = calculator.calculate_instances(ComplexityClass::Linear, 200);
    // ceil(200/1.5) = 134, but we set max to 100
    assert_eq!(result.recommended_instances, 100);
    assert!(result.resource_constrained);
}

// ============================================================================
// REASONING TESTS
// ============================================================================

#[test]
fn test_reasoning_contains_complexity_info() {
    let calculator = InstanceCalculator::new();
    let result = calculator.calculate_instances(ComplexityClass::Quadratic, 10);

    assert!(result.reasoning.contains("Ω(n²)"));
    assert!(result.reasoning.contains("Quadratic"));
    assert!(result.reasoning.contains("Nested loops"));
}

#[test]
fn test_reasoning_contains_subtask_count() {
    let calculator = InstanceCalculator::new();
    let result = calculator.calculate_instances(ComplexityClass::Linear, 7);

    assert!(result.reasoning.contains("7 total"));
}

#[test]
fn test_reasoning_contains_optimal_ratio() {
    let calculator = InstanceCalculator::new();
    let result = calculator.calculate_instances(ComplexityClass::Linearithmic, 10);

    assert!(result.reasoning.contains("2.5")); // Optimal subtasks/instance
}

#[test]
fn test_reasoning_shows_constraint_warning() {
    let calculator = InstanceCalculator::new();
    let result = calculator.calculate_instances(ComplexityClass::Linear, 50);

    assert!(result.reasoning.contains("⚠️"));
    assert!(result.reasoning.contains("Resource constrained"));
    assert!(result.reasoning.contains("capped at 10 instances"));
    assert!(result.reasoning.contains("Note:")); // Warning note
}

#[test]
fn test_reasoning_no_constraint_warning_when_not_constrained() {
    let calculator = InstanceCalculator::new();
    let result = calculator.calculate_instances(ComplexityClass::Linear, 5);

    assert!(!result.reasoning.contains("⚠️"));
    assert!(!result.reasoning.contains("Resource constrained"));
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

#[test]
fn test_instance_calculation_serialization() {
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
fn test_round_trip_serialization() {
    let calculator = InstanceCalculator::new();
    let original = calculator.calculate_instances(ComplexityClass::Quadratic, 15);

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: InstanceCalculation = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.recommended_instances, original.recommended_instances);
    assert_eq!(deserialized.subtasks_per_instance, original.subtasks_per_instance);
    assert_eq!(deserialized.complexity_class, original.complexity_class);
    assert_eq!(deserialized.resource_constrained, original.resource_constrained);
}

// ============================================================================
// FRACTIONAL ROUNDING TESTS
// ============================================================================

#[test]
fn test_ceiling_division_rounding() {
    let calculator = InstanceCalculator::new();

    // Linear: 1.5 per instance
    // 4 subtasks → ceil(4/1.5) = ceil(2.67) = 3
    let result = calculator.calculate_instances(ComplexityClass::Linear, 4);
    assert_eq!(result.recommended_instances, 3);
    assert!((result.subtasks_per_instance - 1.33).abs() < 0.01);
}

#[test]
fn test_exact_division_no_rounding() {
    let calculator = InstanceCalculator::new();

    // Linearithmic: 2.5 per instance
    // 5 subtasks → ceil(5/2.5) = 2 (exact)
    let result = calculator.calculate_instances(ComplexityClass::Linearithmic, 5);
    assert_eq!(result.recommended_instances, 2);
    assert_eq!(result.subtasks_per_instance, 2.5);
}

// ============================================================================
// COMPREHENSIVE COMPLEXITY MATRIX TEST
// ============================================================================

#[test]
fn test_all_complexities_matrix() {
    let calculator = InstanceCalculator::new();

    let test_cases = vec![
        // (complexity, subtasks, expected_instances)
        (ComplexityClass::Constant, 5, 5),       // 1.0 per instance
        (ComplexityClass::Logarithmic, 5, 5),    // 1.0 per instance
        (ComplexityClass::Linear, 5, 4),         // ceil(5/1.5) = 4
        (ComplexityClass::Linearithmic, 5, 2),   // ceil(5/2.5) = 2
        (ComplexityClass::Quadratic, 5, 2),      // ceil(5/4.0) = 2
        (ComplexityClass::Exponential, 5, 1),    // ceil(5/6.5) = 1
    ];

    for (complexity, subtasks, expected) in test_cases {
        let result = calculator.calculate_instances(complexity, subtasks);
        assert_eq!(
            result.recommended_instances, expected,
            "Failed for {:?} with {} subtasks",
            complexity, subtasks
        );
        assert_eq!(result.complexity_class, complexity);
    }
}

// ============================================================================
// INTEGRATION WITH SUBTASK OPTIMIZER (Workflow Test)
// ============================================================================

#[tokio::test]
async fn test_integration_with_subtask_optimizer() {
    // This test demonstrates the full workflow:
    // 1. SubtaskOptimizer analyzes task
    // 2. InstanceCalculator determines instance count

    // Create optimizer and calculator
    let optimizer = match SubtaskOptimizer::new("sk-ant-test-key".to_string()) {
        Ok(opt) => opt,
        Err(_) => return, // Skip if API key invalid (expected in CI)
    };
    let calculator = InstanceCalculator::new();

    // Pre-warm cache with mock estimate
    let mock_estimate = llm_estimator::ComplexityEstimate {
        complexity_class: "Ω(n)".to_string(),
        reasoning: "Standard CRUD operation".to_string(),
        recommended_subtasks: 4,
        confidence: 0.85,
    };
    optimizer.insert_cached("Test task", 0, mock_estimate);

    // Step 1: Optimize subtask count
    let opt_result = optimizer
        .optimize_subtask_count("Test task", 0)
        .await
        .unwrap();

    assert_eq!(opt_result.complexity_class, ComplexityClass::Linear);
    assert_eq!(opt_result.recommended_subtasks, 4);

    // Step 2: Calculate instance count
    let inst_result = calculator.calculate_instances(
        opt_result.complexity_class,
        opt_result.recommended_subtasks,
    );

    assert_eq!(inst_result.recommended_instances, 3); // ceil(4/1.5) = 3
    assert!((inst_result.subtasks_per_instance - 1.33).abs() < 0.01);
    assert!(!inst_result.resource_constrained);
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

#[test]
fn test_calculation_performance() {
    use std::time::Instant;

    let calculator = InstanceCalculator::new();

    // Perform 1000 calculations and measure time
    let start = Instant::now();
    for i in 1..=1000 {
        let _ = calculator.calculate_instances(ComplexityClass::Linear, i % 100 + 1);
    }
    let duration = start.elapsed();

    // Should complete in < 10ms (target: <1ms per calculation)
    assert!(
        duration.as_millis() < 10,
        "1000 calculations took {}ms (expected <10ms)",
        duration.as_millis()
    );
}

#[test]
fn test_calculation_determinism() {
    let calculator = InstanceCalculator::new();

    // Same input should always produce same output
    let result1 = calculator.calculate_instances(ComplexityClass::Linear, 7);
    let result2 = calculator.calculate_instances(ComplexityClass::Linear, 7);

    assert_eq!(result1.recommended_instances, result2.recommended_instances);
    assert_eq!(result1.subtasks_per_instance, result2.subtasks_per_instance);
    assert_eq!(result1.resource_constrained, result2.resource_constrained);
}
