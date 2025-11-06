//! Ground truth test cases for A/B testing
//!
//! This module contains 30 manually verified test cases covering all complexity classes.
//! Each test case represents a realistic task with:
//! - Unique ID for tracking
//! - Task description (input to optimization strategies)
//! - Expected complexity class (ground truth label)
//! - Expected subtask count (ground truth target)
//!
//! # Ground Truth Labeling Process
//!
//! Each test case was manually reviewed and labeled by analyzing:
//! 1. Task scope and dependencies
//! 2. Required technical complexity
//! 3. Number of logical decomposition steps
//! 4. Integration requirements
//! 5. Historical similar task outcomes

use omega_theory::ComplexityClass;
use serde::{Deserialize, Serialize};

/// Test case with ground truth labels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestCase {
    /// Unique identifier (e.g., "tc-001")
    pub id: String,

    /// Task description (input to strategies)
    pub task_description: String,

    /// Ground truth complexity class (manually verified)
    pub expected_complexity: ComplexityClass,

    /// Ground truth subtask count (manually verified)
    pub expected_subtasks: usize,

    /// Category for analysis (e.g., "UI", "Backend", "Infrastructure")
    pub category: String,
}

impl TestCase {
    /// Create a new test case
    pub fn new(
        id: impl Into<String>,
        task_description: impl Into<String>,
        expected_complexity: ComplexityClass,
        expected_subtasks: usize,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            task_description: task_description.into(),
            expected_complexity,
            expected_subtasks,
            category: category.into(),
        }
    }
}

/// Get all test cases for A/B testing
///
/// Returns 30 diverse test cases covering all complexity classes:
/// - 5 Constant (Ω(1))
/// - 5 Logarithmic (Ω(log n))
/// - 5 Linear (Ω(n))
/// - 5 Linearithmic (Ω(n log n))
/// - 5 Quadratic (Ω(n²))
/// - 5 Exponential (Ω(2^n))
#[must_use]
pub fn get_all_test_cases() -> Vec<TestCase> {
    vec![
        // ===== CONSTANT (Ω(1)) - 5 cases =====
        TestCase::new(
            "tc-001",
            "Add a button to the homepage",
            ComplexityClass::Constant,
            1,
            "UI",
        ),
        TestCase::new(
            "tc-002",
            "Update configuration constant from 100 to 200",
            ComplexityClass::Constant,
            1,
            "Config",
        ),
        TestCase::new(
            "tc-003",
            "Fix typo in error message",
            ComplexityClass::Constant,
            1,
            "Maintenance",
        ),
        TestCase::new(
            "tc-004",
            "Change button color from blue to green",
            ComplexityClass::Constant,
            1,
            "UI",
        ),
        TestCase::new(
            "tc-005",
            "Add console.log for debugging",
            ComplexityClass::Constant,
            1,
            "Debug",
        ),
        // ===== LOGARITHMIC (Ω(log n)) - 5 cases =====
        TestCase::new(
            "tc-006",
            "Implement binary search for user lookup",
            ComplexityClass::Logarithmic,
            2,
            "Algorithm",
        ),
        TestCase::new(
            "tc-007",
            "Add balanced tree indexing for fast queries",
            ComplexityClass::Logarithmic,
            3,
            "Database",
        ),
        TestCase::new(
            "tc-008",
            "Implement divide-and-conquer algorithm for finding median",
            ComplexityClass::Logarithmic,
            3,
            "Algorithm",
        ),
        TestCase::new(
            "tc-009",
            "Build B-tree index for product catalog",
            ComplexityClass::Logarithmic,
            2,
            "Database",
        ),
        TestCase::new(
            "tc-010",
            "Create logarithmic time cache lookup with LRU eviction",
            ComplexityClass::Logarithmic,
            3,
            "Performance",
        ),
        // ===== LINEAR (Ω(n)) - 5 cases =====
        TestCase::new(
            "tc-011",
            "Implement user authentication with JWT",
            ComplexityClass::Linear,
            4,
            "Backend",
        ),
        TestCase::new(
            "tc-012",
            "Create CRUD endpoints for blog posts",
            ComplexityClass::Linear,
            4,
            "Backend",
        ),
        TestCase::new(
            "tc-013",
            "Build user profile page with form validation",
            ComplexityClass::Linear,
            5,
            "UI",
        ),
        TestCase::new(
            "tc-014",
            "Add pagination to product listing API",
            ComplexityClass::Linear,
            3,
            "Backend",
        ),
        TestCase::new(
            "tc-015",
            "Implement file upload with progress tracking",
            ComplexityClass::Linear,
            4,
            "Feature",
        ),
        // ===== LINEARITHMIC (Ω(n log n)) - 5 cases =====
        TestCase::new(
            "tc-016",
            "Add sorting and filtering to data table",
            ComplexityClass::Linearithmic,
            5,
            "UI",
        ),
        TestCase::new(
            "tc-017",
            "Build full-text search with indexing",
            ComplexityClass::Linearithmic,
            6,
            "Search",
        ),
        TestCase::new(
            "tc-018",
            "Implement merge sort for large dataset processing",
            ComplexityClass::Linearithmic,
            4,
            "Algorithm",
        ),
        TestCase::new(
            "tc-019",
            "Create recommendation engine with collaborative filtering",
            ComplexityClass::Linearithmic,
            6,
            "ML",
        ),
        TestCase::new(
            "tc-020",
            "Build analytics dashboard with aggregation and sorting",
            ComplexityClass::Linearithmic,
            5,
            "Analytics",
        ),
        // ===== QUADRATIC (Ω(n²)) - 5 cases =====
        TestCase::new(
            "tc-021",
            "Implement social graph with friend recommendations",
            ComplexityClass::Quadratic,
            8,
            "Social",
        ),
        TestCase::new(
            "tc-022",
            "Build e-commerce platform with payment gateway integration",
            ComplexityClass::Quadratic,
            8,
            "E-commerce",
        ),
        TestCase::new(
            "tc-023",
            "Create real-time collaboration system with conflict resolution",
            ComplexityClass::Quadratic,
            9,
            "Collaboration",
        ),
        TestCase::new(
            "tc-024",
            "Implement nested comment system with threading and replies",
            ComplexityClass::Quadratic,
            7,
            "Feature",
        ),
        TestCase::new(
            "tc-025",
            "Build matrix operations library with multiplication",
            ComplexityClass::Quadratic,
            6,
            "Algorithm",
        ),
        // ===== EXPONENTIAL (Ω(2^n)) - 5 cases =====
        TestCase::new(
            "tc-026",
            "Migrate entire monolithic application to microservices architecture",
            ComplexityClass::Exponential,
            12,
            "Architecture",
        ),
        TestCase::new(
            "tc-027",
            "Build complete CI/CD pipeline with multi-environment deployment",
            ComplexityClass::Exponential,
            10,
            "DevOps",
        ),
        TestCase::new(
            "tc-028",
            "Implement genetic algorithm for optimization with constraint satisfaction",
            ComplexityClass::Exponential,
            11,
            "Algorithm",
        ),
        TestCase::new(
            "tc-029",
            "Create distributed transaction system with 2-phase commit across 5 services",
            ComplexityClass::Exponential,
            13,
            "Distributed",
        ),
        TestCase::new(
            "tc-030",
            "Build comprehensive security framework with authentication, authorization, encryption, audit logging, and compliance",
            ComplexityClass::Exponential,
            15,
            "Security",
        ),
    ]
}

/// Get test cases by complexity class
pub fn get_test_cases_by_complexity(complexity: ComplexityClass) -> Vec<TestCase> {
    get_all_test_cases()
        .into_iter()
        .filter(|tc| tc.expected_complexity == complexity)
        .collect()
}

/// Get test case statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseStats {
    /// Total number of test cases
    pub total_cases: usize,
    /// Number of cases per complexity class
    pub cases_per_complexity: std::collections::HashMap<String, usize>,
    /// Unique categories in test cases
    pub categories: std::collections::HashSet<String>,
}

/// Calculate statistics for all test cases
#[must_use]
pub fn get_test_case_stats() -> TestCaseStats {
    let cases = get_all_test_cases();
    let mut cases_per_complexity = std::collections::HashMap::new();
    let mut categories = std::collections::HashSet::new();

    for case in &cases {
        *cases_per_complexity
            .entry(case.expected_complexity.to_string())
            .or_insert(0) += 1;
        categories.insert(case.category.clone());
    }

    TestCaseStats {
        total_cases: cases.len(),
        cases_per_complexity,
        categories,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_test_cases_count() {
        let cases = get_all_test_cases();
        assert_eq!(cases.len(), 30, "Should have exactly 30 test cases");
    }

    #[test]
    fn test_complexity_distribution() {
        let cases = get_all_test_cases();

        // Count cases per complexity class
        let mut counts = std::collections::HashMap::new();
        for case in &cases {
            *counts.entry(case.expected_complexity).or_insert(0) += 1;
        }

        // Verify 5 cases per complexity class
        assert_eq!(counts.get(&ComplexityClass::Constant), Some(&5));
        assert_eq!(counts.get(&ComplexityClass::Logarithmic), Some(&5));
        assert_eq!(counts.get(&ComplexityClass::Linear), Some(&5));
        assert_eq!(counts.get(&ComplexityClass::Linearithmic), Some(&5));
        assert_eq!(counts.get(&ComplexityClass::Quadratic), Some(&5));
        assert_eq!(counts.get(&ComplexityClass::Exponential), Some(&5));
    }

    #[test]
    fn test_unique_ids() {
        let cases = get_all_test_cases();
        let ids: std::collections::HashSet<_> = cases.iter().map(|c| &c.id).collect();
        assert_eq!(ids.len(), 30, "All test case IDs should be unique");
    }

    #[test]
    fn test_expected_subtasks_within_omega_bounds() {
        let cases = get_all_test_cases();

        for case in cases {
            let omega_range = case.expected_complexity.to_subtask_range();
            assert!(
                omega_range.contains(&case.expected_subtasks),
                "Test case {} has expected_subtasks={} which is outside Ω bounds {:?} for {}",
                case.id,
                case.expected_subtasks,
                omega_range,
                case.expected_complexity
            );
        }
    }

    #[test]
    fn test_no_empty_descriptions() {
        let cases = get_all_test_cases();
        for case in cases {
            assert!(
                !case.task_description.trim().is_empty(),
                "Test case {} has empty description",
                case.id
            );
        }
    }

    #[test]
    fn test_get_test_cases_by_complexity() {
        let linear_cases = get_test_cases_by_complexity(ComplexityClass::Linear);
        assert_eq!(linear_cases.len(), 5);
        assert!(linear_cases.iter().all(|c| c.expected_complexity == ComplexityClass::Linear));
    }

    #[test]
    fn test_test_case_stats() {
        let stats = get_test_case_stats();
        assert_eq!(stats.total_cases, 30);
        assert_eq!(stats.cases_per_complexity.len(), 6); // 6 complexity classes
        assert!(stats.categories.len() >= 10); // At least 10 different categories
    }

    #[test]
    fn test_test_case_serialization() {
        let case = TestCase::new(
            "test-id",
            "Test description",
            ComplexityClass::Linear,
            4,
            "Test",
        );

        let json = serde_json::to_string(&case).unwrap();
        let deserialized: TestCase = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, case);
    }
}
