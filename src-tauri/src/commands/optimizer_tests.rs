//! Integration tests for optimizer Tauri commands
//!
//! These tests verify the Tauri IPC layer works correctly with the optimizer backend.

#[cfg(test)]
mod tests {
    use crate::commands::optimizer::{
        calculate_instances, get_complexity_info, optimize_task, OptimizerState,
    };
    use tauri::State;

    /// Helper to create OptimizerState for testing
    fn create_test_state() -> OptimizerState {
        OptimizerState::new()
    }

    /// Helper to create State wrapper
    fn wrap_state(state: &OptimizerState) -> State<'_, OptimizerState> {
        State::from(state)
    }

    // ========================================================================
    // optimize_task tests
    // ========================================================================

    #[tokio::test]
    async fn test_optimize_task_empty_description() {
        let state = create_test_state();
        let result = optimize_task(String::new(), 0, wrap_state(&state)).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[tokio::test]
    async fn test_optimize_task_whitespace_only() {
        let state = create_test_state();
        let result = optimize_task("   ".to_string(), 0, wrap_state(&state)).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[tokio::test]
    #[ignore = "Requires ANTHROPIC_API_KEY environment variable"]
    async fn test_optimize_task_simple_linear() {
        let state = create_test_state();
        let result = optimize_task(
            "Implement CRUD operations for users".to_string(),
            0,
            wrap_state(&state),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should be Linear complexity
        assert_eq!(response.complexity_class, "Linear");
        assert!(response.recommended_subtasks >= 3);
        assert!(response.recommended_subtasks <= 5);
        assert!(response.confidence > 0.0);
        assert!(response.confidence <= 1.0);
        assert!(!response.reasoning.is_empty());
    }

    #[tokio::test]
    #[ignore = "Requires ANTHROPIC_API_KEY environment variable"]
    async fn test_optimize_task_quadratic() {
        let state = create_test_state();
        let result = optimize_task(
            "Implement matrix multiplication algorithm".to_string(),
            0,
            wrap_state(&state),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should be Quadratic complexity
        assert_eq!(response.complexity_class, "Quadratic");
        assert!(response.recommended_subtasks >= 5);
        assert!(response.recommended_subtasks <= 10);
    }

    #[tokio::test]
    #[ignore = "Requires ANTHROPIC_API_KEY environment variable"]
    async fn test_optimize_task_constant() {
        let state = create_test_state();
        let result = optimize_task(
            "Update configuration variable".to_string(),
            0,
            wrap_state(&state),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should be Constant complexity
        assert_eq!(response.complexity_class, "Constant");
        assert_eq!(response.recommended_subtasks, 1);
    }

    #[tokio::test]
    #[ignore = "Requires ANTHROPIC_API_KEY environment variable"]
    async fn test_optimize_task_with_current_subtasks() {
        let state = create_test_state();
        let result = optimize_task(
            "Implement user authentication".to_string(),
            3,
            wrap_state(&state),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.recommended_subtasks > 0);
    }

    #[tokio::test]
    #[ignore = "Requires ANTHROPIC_API_KEY environment variable"]
    async fn test_optimize_task_caching() {
        let state = create_test_state();
        let task_desc = "Implement sorting algorithm".to_string();

        // First call (should hit LLM)
        let start1 = std::time::Instant::now();
        let result1 = optimize_task(task_desc.clone(), 0, wrap_state(&state)).await;
        let duration1 = start1.elapsed();

        assert!(result1.is_ok());

        // Second call (should hit cache)
        let start2 = std::time::Instant::now();
        let result2 = optimize_task(task_desc, 0, wrap_state(&state)).await;
        let duration2 = start2.elapsed();

        assert!(result2.is_ok());

        // Cache hit should be significantly faster (<50ms vs ~1-2s)
        assert!(duration2 < duration1 / 10);
    }

    // ========================================================================
    // calculate_instances tests
    // ========================================================================

    #[test]
    fn test_calculate_instances_zero_subtasks() {
        let state = create_test_state();
        let result = calculate_instances("Linear".to_string(), 0, wrap_state(&state));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be greater than 0"));
    }

    #[test]
    fn test_calculate_instances_invalid_complexity() {
        let state = create_test_state();
        let result = calculate_instances("Invalid".to_string(), 5, wrap_state(&state));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid complexity class"));
    }

    #[test]
    fn test_calculate_instances_constant() {
        let state = create_test_state();
        let result = calculate_instances("Constant".to_string(), 1, wrap_state(&state));

        assert!(result.is_ok());
        let response = result.unwrap();

        assert_eq!(response.recommended_instances, 1);
        assert_eq!(response.subtasks_per_instance, 1.0);
        assert!(!response.resource_constrained);
    }

    #[test]
    fn test_calculate_instances_linear_small() {
        let state = create_test_state();
        let result = calculate_instances("Linear".to_string(), 5, wrap_state(&state));

        assert!(result.is_ok());
        let response = result.unwrap();

        assert_eq!(response.recommended_instances, 3);
        assert!((response.subtasks_per_instance - 1.67).abs() < 0.01);
        assert!(!response.resource_constrained);
        assert!(!response.reasoning.is_empty());
    }

    #[test]
    fn test_calculate_instances_linear_large() {
        let state = create_test_state();
        let result = calculate_instances("Linear".to_string(), 20, wrap_state(&state));

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should hit max limit
        assert_eq!(response.recommended_instances, 10);
        assert_eq!(response.subtasks_per_instance, 2.0);
        assert!(response.resource_constrained);
    }

    #[test]
    fn test_calculate_instances_quadratic() {
        let state = create_test_state();
        let result = calculate_instances("Quadratic".to_string(), 10, wrap_state(&state));

        assert!(result.is_ok());
        let response = result.unwrap();

        // Quadratic: 3-5 subtasks per instance
        assert!(response.recommended_instances >= 2);
        assert!(response.recommended_instances <= 4);
        assert!((response.subtasks_per_instance >= 2.5) && (response.subtasks_per_instance <= 5.0));
    }

    #[test]
    fn test_calculate_instances_exponential() {
        let state = create_test_state();
        let result = calculate_instances("Exponential".to_string(), 50, wrap_state(&state));

        assert!(result.is_ok());
        let response = result.unwrap();

        // Exponential: 5-8 subtasks per instance, should hit max
        assert_eq!(response.recommended_instances, 10);
        assert!(response.resource_constrained);
    }

    #[test]
    fn test_calculate_instances_case_insensitive() {
        let state = create_test_state();

        let result1 = calculate_instances("linear".to_string(), 5, wrap_state(&state));
        let result2 = calculate_instances("LINEAR".to_string(), 5, wrap_state(&state));
        let result3 = calculate_instances("Linear".to_string(), 5, wrap_state(&state));

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());

        // All should produce same result
        assert_eq!(
            result1.unwrap().recommended_instances,
            result2.unwrap().recommended_instances
        );
        assert_eq!(
            result2.unwrap().recommended_instances,
            result3.unwrap().recommended_instances
        );
    }

    // ========================================================================
    // get_complexity_info tests
    // ========================================================================

    #[test]
    fn test_get_complexity_info_invalid() {
        let result = get_complexity_info("InvalidClass".to_string());

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid complexity class"));
    }

    #[test]
    fn test_get_complexity_info_constant() {
        let result = get_complexity_info("Constant".to_string());

        assert!(result.is_ok());
        let info = result.unwrap();

        assert_eq!(info.class_name, "Constant");
        assert_eq!(info.notation, "Ω(1)");
        assert_eq!(info.subtask_range, "1");
        assert!(!info.description.is_empty());
        assert!(!info.examples.is_empty());
    }

    #[test]
    fn test_get_complexity_info_logarithmic() {
        let result = get_complexity_info("Logarithmic".to_string());

        assert!(result.is_ok());
        let info = result.unwrap();

        assert_eq!(info.class_name, "Logarithmic");
        assert_eq!(info.notation, "Ω(log n)");
        assert_eq!(info.subtask_range, "2-3");
        assert!(info.examples.contains(&"Binary search implementations".to_string()));
    }

    #[test]
    fn test_get_complexity_info_linear() {
        let result = get_complexity_info("Linear".to_string());

        assert!(result.is_ok());
        let info = result.unwrap();

        assert_eq!(info.class_name, "Linear");
        assert_eq!(info.notation, "Ω(n)");
        assert_eq!(info.subtask_range, "3-5");
        assert!(info.examples.contains(&"CRUD API implementations".to_string()));
    }

    #[test]
    fn test_get_complexity_info_linearithmic() {
        let result = get_complexity_info("Linearithmic".to_string());

        assert!(result.is_ok());
        let info = result.unwrap();

        assert_eq!(info.class_name, "Linearithmic");
        assert_eq!(info.notation, "Ω(n log n)");
        assert_eq!(info.subtask_range, "4-6");
    }

    #[test]
    fn test_get_complexity_info_quadratic() {
        let result = get_complexity_info("Quadratic".to_string());

        assert!(result.is_ok());
        let info = result.unwrap();

        assert_eq!(info.class_name, "Quadratic");
        assert_eq!(info.notation, "Ω(n²)");
        assert_eq!(info.subtask_range, "5-10");
        assert!(info.examples.contains(&"Matrix operations".to_string()));
    }

    #[test]
    fn test_get_complexity_info_exponential() {
        let result = get_complexity_info("Exponential".to_string());

        assert!(result.is_ok());
        let info = result.unwrap();

        assert_eq!(info.class_name, "Exponential");
        assert_eq!(info.notation, "Ω(2^n)");
        assert_eq!(info.subtask_range, "8-15");
        assert!(info.examples.contains(&"Backtracking algorithms".to_string()));
    }

    #[test]
    fn test_get_complexity_info_all_classes() {
        let classes = vec![
            "Constant",
            "Logarithmic",
            "Linear",
            "Linearithmic",
            "Quadratic",
            "Exponential",
        ];

        for class in classes {
            let result = get_complexity_info(class.to_string());
            assert!(result.is_ok(), "Failed for class: {}", class);

            let info = result.unwrap();
            assert_eq!(info.class_name, class);
            assert!(!info.notation.is_empty());
            assert!(!info.subtask_range.is_empty());
            assert!(!info.description.is_empty());
            assert!(!info.examples.is_empty());
            assert!(info.examples.len() >= 3);
        }
    }

    #[test]
    fn test_get_complexity_info_case_insensitive() {
        let result1 = get_complexity_info("linear".to_string());
        let result2 = get_complexity_info("LINEAR".to_string());
        let result3 = get_complexity_info("Linear".to_string());

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());

        let info1 = result1.unwrap();
        let info2 = result2.unwrap();
        let info3 = result3.unwrap();

        assert_eq!(info1.class_name, info2.class_name);
        assert_eq!(info2.class_name, info3.class_name);
    }

    // ========================================================================
    // Integration tests
    // ========================================================================

    #[tokio::test]
    #[ignore = "Requires ANTHROPIC_API_KEY environment variable"]
    async fn test_full_workflow_linear() {
        let state = create_test_state();

        // Step 1: Optimize task
        let optimize_result = optimize_task(
            "Implement user authentication API".to_string(),
            0,
            wrap_state(&state),
        )
        .await;

        assert!(optimize_result.is_ok());
        let optimization = optimize_result.unwrap();

        // Step 2: Calculate instances
        let instances_result = calculate_instances(
            optimization.complexity_class.clone(),
            optimization.recommended_subtasks,
            wrap_state(&state),
        );

        assert!(instances_result.is_ok());
        let instances = instances_result.unwrap();

        // Step 3: Get complexity info
        let info_result = get_complexity_info(optimization.complexity_class.clone());

        assert!(info_result.is_ok());
        let info = info_result.unwrap();

        // Verify consistency
        assert_eq!(info.class_name, optimization.complexity_class);
        assert!(instances.recommended_instances > 0);
        assert!(instances.subtasks_per_instance > 0.0);
    }

    #[tokio::test]
    #[ignore = "Requires ANTHROPIC_API_KEY environment variable"]
    async fn test_full_workflow_quadratic() {
        let state = create_test_state();

        // Optimize a quadratic task
        let optimization = optimize_task(
            "Implement all-pairs shortest path algorithm".to_string(),
            0,
            wrap_state(&state),
        )
        .await
        .unwrap();

        assert_eq!(optimization.complexity_class, "Quadratic");

        // Calculate instances
        let instances = calculate_instances(
            optimization.complexity_class.clone(),
            optimization.recommended_subtasks,
            wrap_state(&state),
        )
        .unwrap();

        // Quadratic tasks should batch more subtasks per instance
        assert!(instances.subtasks_per_instance >= 2.0);
    }

    #[test]
    fn test_optimizer_state_creation() {
        let state = create_test_state();
        // Verify state is initialized
        assert!(state.optimizer.lock().unwrap().is_none());
    }

    #[test]
    fn test_optimizer_state_lazy_initialization() {
        let state = create_test_state();

        // Initial state should be None
        assert!(state.optimizer.lock().unwrap().is_none());

        // After get_optimizer() call (if API key available), should be Some
        // This is tested implicitly in the optimize_task tests
    }
}
