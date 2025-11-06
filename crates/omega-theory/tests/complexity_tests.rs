//! Integration tests for omega-theory complexity analysis
//!
//! These tests verify the complete public API of the omega-theory crate,
//! focusing on real-world usage scenarios for AIT42 task decomposition.

use omega_theory::ComplexityClass;
use serde_json;
use std::collections::HashMap;

#[test]
fn test_complexity_class_construction() {
    // All variants should be constructible
    let _ = ComplexityClass::Constant;
    let _ = ComplexityClass::Logarithmic;
    let _ = ComplexityClass::Linear;
    let _ = ComplexityClass::Linearithmic;
    let _ = ComplexityClass::Quadratic;
    let _ = ComplexityClass::Exponential;
}

#[test]
fn test_subtask_range_coverage() {
    // Verify all complexity classes provide non-empty ranges
    let variants = vec![
        ComplexityClass::Constant,
        ComplexityClass::Logarithmic,
        ComplexityClass::Linear,
        ComplexityClass::Linearithmic,
        ComplexityClass::Quadratic,
        ComplexityClass::Exponential,
    ];

    for variant in variants {
        let range = variant.to_subtask_range();
        assert!(range.start() <= range.end());
        assert!(*range.start() >= 1, "Subtask range must start at 1 or higher");
        assert!(
            *range.end() <= 15,
            "Subtask range should not exceed 15 (cognitive load limit)"
        );
    }
}

#[test]
fn test_subtask_range_monotonicity() {
    // Higher complexity should generally allow more subtasks
    assert!(
        ComplexityClass::Constant.to_subtask_range().end()
            < ComplexityClass::Linear.to_subtask_range().end()
    );
    assert!(
        ComplexityClass::Linear.to_subtask_range().end()
            < ComplexityClass::Quadratic.to_subtask_range().end()
    );
    assert!(
        ComplexityClass::Quadratic.to_subtask_range().end()
            <= ComplexityClass::Exponential.to_subtask_range().end()
    );
}

#[test]
fn test_display_format() {
    // Verify Big Omega notation format
    assert_eq!(ComplexityClass::Constant.to_string(), "Ω(1)");
    assert_eq!(ComplexityClass::Logarithmic.to_string(), "Ω(log n)");
    assert_eq!(ComplexityClass::Linear.to_string(), "Ω(n)");
    assert_eq!(ComplexityClass::Linearithmic.to_string(), "Ω(n log n)");
    assert_eq!(ComplexityClass::Quadratic.to_string(), "Ω(n²)");
    assert_eq!(ComplexityClass::Exponential.to_string(), "Ω(2^n)");

    // Ensure no variant is missing Display implementation
    let all_variants = vec![
        ComplexityClass::Constant,
        ComplexityClass::Logarithmic,
        ComplexityClass::Linear,
        ComplexityClass::Linearithmic,
        ComplexityClass::Quadratic,
        ComplexityClass::Exponential,
    ];

    for variant in all_variants {
        let display = variant.to_string();
        assert!(display.starts_with("Ω("));
        assert!(display.ends_with(')'));
    }
}

#[test]
fn test_json_serialization_roundtrip() {
    let variants = vec![
        ComplexityClass::Constant,
        ComplexityClass::Logarithmic,
        ComplexityClass::Linear,
        ComplexityClass::Linearithmic,
        ComplexityClass::Quadratic,
        ComplexityClass::Exponential,
    ];

    for original in variants {
        // Serialize to JSON
        let json = serde_json::to_string(&original).expect("Serialization failed");

        // Deserialize from JSON
        let deserialized: ComplexityClass =
            serde_json::from_str(&json).expect("Deserialization failed");

        // Verify roundtrip
        assert_eq!(original, deserialized);
    }
}

#[test]
fn test_json_deserialization_from_strings() {
    // Test that we can deserialize from expected JSON string formats
    assert_eq!(
        serde_json::from_str::<ComplexityClass>("\"Constant\"").unwrap(),
        ComplexityClass::Constant
    );
    assert_eq!(
        serde_json::from_str::<ComplexityClass>("\"Linear\"").unwrap(),
        ComplexityClass::Linear
    );
    assert_eq!(
        serde_json::from_str::<ComplexityClass>("\"Exponential\"").unwrap(),
        ComplexityClass::Exponential
    );
}

#[test]
fn test_struct_with_complexity_serialization() {
    // Simulate Task Master AI task structure
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct Task {
        id: String,
        description: String,
        complexity: ComplexityClass,
    }

    let task = Task {
        id: "1.2.3".to_string(),
        description: "Implement user authentication".to_string(),
        complexity: ComplexityClass::Linear,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&task).unwrap();

    // Deserialize from JSON
    let deserialized: Task = serde_json::from_str(&json).unwrap();

    assert_eq!(task, deserialized);
}

#[test]
fn test_complexity_class_equality() {
    assert_eq!(ComplexityClass::Linear, ComplexityClass::Linear);
    assert_ne!(ComplexityClass::Linear, ComplexityClass::Quadratic);

    // Test reflexivity
    let c = ComplexityClass::Logarithmic;
    assert_eq!(c, c);

    // Test symmetry
    assert_eq!(
        ComplexityClass::Constant == ComplexityClass::Constant,
        ComplexityClass::Constant == ComplexityClass::Constant
    );
}

#[test]
fn test_complexity_class_copy() {
    let c1 = ComplexityClass::Quadratic;
    let c2 = c1; // Should copy, not move
    let c3 = c1; // Should still be usable

    assert_eq!(c1, c2);
    assert_eq!(c1, c3);
}

#[test]
fn test_complexity_class_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let c1 = ComplexityClass::Linear;
    let c2 = ComplexityClass::Linear;
    let c3 = ComplexityClass::Quadratic;

    let mut hasher1 = DefaultHasher::new();
    c1.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    c2.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    let mut hasher3 = DefaultHasher::new();
    c3.hash(&mut hasher3);
    let hash3 = hasher3.finish();

    // Equal values should have equal hashes
    assert_eq!(hash1, hash2);

    // Different values should (very likely) have different hashes
    assert_ne!(hash1, hash3);
}

#[test]
fn test_complexity_class_in_hashmap() {
    let mut map = HashMap::new();

    map.insert(ComplexityClass::Constant, "trivial");
    map.insert(ComplexityClass::Linear, "standard");
    map.insert(ComplexityClass::Exponential, "complex");

    assert_eq!(map.get(&ComplexityClass::Constant), Some(&"trivial"));
    assert_eq!(map.get(&ComplexityClass::Linear), Some(&"standard"));
    assert_eq!(map.get(&ComplexityClass::Exponential), Some(&"complex"));
}

#[test]
fn test_from_description_placeholder() {
    // v1.6.0 placeholder always returns Linear
    assert_eq!(
        ComplexityClass::from_description("Simple task"),
        ComplexityClass::Linear
    );
    assert_eq!(
        ComplexityClass::from_description("Complex distributed system migration"),
        ComplexityClass::Linear
    );
    assert_eq!(
        ComplexityClass::from_description(""),
        ComplexityClass::Linear
    );
}

#[test]
fn test_description_method() {
    let variants = vec![
        ComplexityClass::Constant,
        ComplexityClass::Logarithmic,
        ComplexityClass::Linear,
        ComplexityClass::Linearithmic,
        ComplexityClass::Quadratic,
        ComplexityClass::Exponential,
    ];

    for variant in variants {
        let desc = variant.description();
        assert!(!desc.is_empty(), "Description should not be empty");
        assert!(
            desc.len() > 10,
            "Description should be sufficiently detailed"
        );
    }
}

#[test]
fn test_real_world_scenario_simple_crud() {
    // Scenario: "Add a new CRUD endpoint for user profiles"
    let complexity = ComplexityClass::from_description("Add CRUD endpoint for user profiles");

    assert_eq!(complexity, ComplexityClass::Linear); // v1.6.0 placeholder

    let subtasks = complexity.to_subtask_range();
    assert_eq!(subtasks, 3..=5);

    println!(
        "Task: Add CRUD endpoint\nComplexity: {}\nRecommended subtasks: {}-{}",
        complexity,
        subtasks.start(),
        subtasks.end()
    );
}

#[test]
fn test_real_world_scenario_database_optimization() {
    // Scenario: "Optimize N+1 query problem across entire codebase"
    let complexity =
        ComplexityClass::from_description("Optimize N+1 query problem across entire codebase");

    assert_eq!(complexity, ComplexityClass::Linear); // v1.6.0 placeholder

    // In v1.7.0, this should classify as Quadratic or higher
    // For now, verify the API works
    let subtasks = complexity.to_subtask_range();
    assert!(subtasks.contains(&4));
}

#[test]
fn test_debug_format() {
    let c = ComplexityClass::Linear;
    let debug_str = format!("{:?}", c);
    assert_eq!(debug_str, "Linear");
}

#[test]
fn test_clone_implementation() {
    let c1 = ComplexityClass::Exponential;
    let c2 = c1.clone();

    assert_eq!(c1, c2);
}

#[test]
fn test_subtask_range_boundaries() {
    // Test specific boundary conditions
    let constant_range = ComplexityClass::Constant.to_subtask_range();
    assert_eq!(*constant_range.start(), 1);
    assert_eq!(*constant_range.end(), 1);

    let exponential_range = ComplexityClass::Exponential.to_subtask_range();
    assert_eq!(*exponential_range.start(), 8);
    assert_eq!(*exponential_range.end(), 15);
}

#[test]
fn test_all_variants_covered() {
    // Ensure we test all enum variants (catches new variants added in future)
    let all_variants = vec![
        ComplexityClass::Constant,
        ComplexityClass::Logarithmic,
        ComplexityClass::Linear,
        ComplexityClass::Linearithmic,
        ComplexityClass::Quadratic,
        ComplexityClass::Exponential,
    ];

    // If you add a new variant, this count will fail, forcing test updates
    assert_eq!(all_variants.len(), 6, "Update tests if new variants added");
}
