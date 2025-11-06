//! Subtask Optimizer Commands
//!
//! Tauri commands for subtask count optimization using Ω-theory and LLM analysis.
//!
//! This module exposes the Rust optimizer backend (SubtaskOptimizer, InstanceCalculator)
//! to the TypeScript frontend via Tauri IPC.

use crate::optimizer::{
    ComplexityClass, ComplexityEstimate, InstanceCalculation, InstanceCalculator,
    MemoryAdjustment, OptimizationResult, OptimizerError, SubtaskOptimizer,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// Shared optimizer instance (lazy-initialized)
pub struct OptimizerState {
    optimizer: Arc<Mutex<Option<SubtaskOptimizer>>>,
    calculator: InstanceCalculator,
}

impl OptimizerState {
    pub fn new() -> Self {
        Self {
            optimizer: Arc::new(Mutex::new(None)),
            calculator: InstanceCalculator::new(),
        }
    }

    /// Initialize the optimizer if needed (called once at startup or first use)
    async fn ensure_initialized(&self) -> Result<(), String> {
        let mut optimizer_guard = self.optimizer.lock().await;

        // Initialize on first use
        if optimizer_guard.is_none() {
            debug!("Initializing SubtaskOptimizer from environment");
            let optimizer = SubtaskOptimizer::from_env().map_err(|e| {
                format!(
                    "Failed to initialize optimizer. Please ensure ANTHROPIC_API_KEY is set: {}",
                    e
                )
            })?;
            *optimizer_guard = Some(optimizer);
            info!("SubtaskOptimizer initialized successfully");
        }

        Ok(())
    }

    /// Clone the Arc<Mutex<>> for thread-safe access
    fn get_optimizer_handle(&self) -> Arc<Mutex<Option<SubtaskOptimizer>>> {
        Arc::clone(&self.optimizer)
    }
}

impl Default for OptimizerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Response for optimize_task command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizeTaskResponse {
    /// Detected complexity class (e.g., "Linear", "Quadratic")
    pub complexity_class: String,

    /// Recommended number of subtasks
    pub recommended_subtasks: usize,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Human-readable reasoning for the recommendation
    pub reasoning: String,

    /// Memory-based adjustment (v1.7.0+, may be null)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_adjustment: Option<MemoryAdjustmentResponse>,

    /// Original LLM estimate details (may be null)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_estimate: Option<ComplexityEstimateResponse>,
}

/// Memory adjustment details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryAdjustmentResponse {
    pub historical_success_rate: f64,
    pub adjustment: i32,
    pub reasoning: String,
}

/// LLM estimate details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexityEstimateResponse {
    pub complexity_class: String,
    pub confidence: f64,
    pub reasoning: String,
    pub estimated_subtasks: usize,
}

/// Response for calculate_instances command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculateInstancesResponse {
    /// Recommended number of Claude Code instances
    pub recommended_instances: usize,

    /// Average subtasks per instance (may be fractional)
    pub subtasks_per_instance: f64,

    /// True if calculation was limited by max_instances constraint
    pub resource_constrained: bool,

    /// Human-readable reasoning for the recommendation
    pub reasoning: String,
}

/// Response for get_complexity_info command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexityInfoResponse {
    /// Complexity class name (e.g., "Linear")
    pub class_name: String,

    /// Ω-notation (e.g., "Ω(n)")
    pub notation: String,

    /// Recommended subtask range (e.g., "3-5")
    pub subtask_range: String,

    /// Human-readable description
    pub description: String,

    /// Example use cases
    pub examples: Vec<String>,
}

/// Optimize subtask count for a task
///
/// Uses LLM analysis + Ω-theory to recommend optimal subtask decomposition.
///
/// # Arguments
/// * `task_description` - Task description to analyze
/// * `current_subtasks` - Current number of subtasks (0 if none)
///
/// # Returns
/// * `Ok(response)` - Optimization recommendation with reasoning
/// * `Err(message)` - User-friendly error message
///
/// # Performance
/// - First call: ~1-2s (LLM API call)
/// - Cached calls: ~1-5ms
///
/// # Example
/// ```typescript
/// const result = await invoke('optimize_task', {
///   taskDescription: 'Implement user authentication',
///   currentSubtasks: 0
/// });
/// console.log(`Recommended: ${result.recommended_subtasks} subtasks`);
/// ```
#[tauri::command]
pub async fn optimize_task(
    task_description: String,
    current_subtasks: usize,
    state: State<'_, OptimizerState>,
) -> Result<OptimizeTaskResponse, String> {
    info!(
        "Optimizing task: '{}' (current subtasks: {})",
        task_description, current_subtasks
    );

    // Validate input
    if task_description.trim().is_empty() {
        return Err("Task description cannot be empty".to_string());
    }

    // Ensure optimizer is initialized
    state.ensure_initialized().await?;

    // Clone Arc handle (cheap operation)
    let optimizer_handle = state.get_optimizer_handle();

    // Perform optimization (acquire lock inside the async operation)
    let result = {
        let optimizer_guard = optimizer_handle.lock().await;

        let optimizer = optimizer_guard
            .as_ref()
            .ok_or_else(|| "Optimizer not initialized".to_string())?;

        optimizer
            .optimize_subtask_count(&task_description, current_subtasks)
            .await
    }
    .map_err(|e| match e {
            OptimizerError::EstimationFailed(est_err) => {
                error!("LLM estimation failed: {}", est_err);
                format!(
                    "Failed to analyze task complexity. Please check your API key and network connection: {}",
                    est_err
                )
            }
            OptimizerError::Timeout(duration) => {
                error!("Optimization timeout: {:?}", duration);
                format!("Optimization timed out after {:?}. Please try again.", duration)
            }
            OptimizerError::InvalidInput(msg) => {
                error!("Invalid input: {}", msg);
                format!("Invalid input: {}", msg)
            }
        })?;

    info!(
        "Optimization complete: {} ({} subtasks, confidence: {:.2})",
        result.complexity_class, result.recommended_subtasks, result.confidence
    );

    // Convert to response format
    Ok(convert_optimization_result(result))
}

/// Calculate optimal Claude Code instance count
///
/// Given a complexity class and subtask count, determines how many parallel
/// Claude Code instances should be used for optimal throughput.
///
/// # Arguments
/// * `complexity_class` - Complexity class name (e.g., "Linear", "Quadratic")
/// * `subtask_count` - Number of subtasks to execute
///
/// # Returns
/// * `Ok(response)` - Instance count recommendation with reasoning
/// * `Err(message)` - User-friendly error message
///
/// # Performance
/// - Execution time: <1ms (synchronous calculation)
///
/// # Example
/// ```typescript
/// const result = await invoke('calculate_instances', {
///   complexityClass: 'Linear',
///   subtaskCount: 5
/// });
/// console.log(`Use ${result.recommended_instances} instances`);
/// ```
#[tauri::command]
pub fn calculate_instances(
    complexity_class: String,
    subtask_count: usize,
    state: State<'_, OptimizerState>,
) -> Result<CalculateInstancesResponse, String> {
    debug!(
        "Calculating instances: complexity={}, subtasks={}",
        complexity_class, subtask_count
    );

    // Validate input
    if subtask_count == 0 {
        return Err("Subtask count must be greater than 0".to_string());
    }

    // Parse complexity class
    let complexity = parse_complexity_class(&complexity_class)?;

    // Calculate instances
    let result = state.calculator.calculate_instances(complexity, subtask_count);

    debug!(
        "Instance calculation complete: {} instances ({:.2} subtasks/instance)",
        result.recommended_instances, result.subtasks_per_instance
    );

    Ok(convert_instance_calculation(result))
}

/// Get detailed information about a complexity class
///
/// Provides Ω-notation, subtask ranges, descriptions, and examples for UI tooltips.
///
/// # Arguments
/// * `complexity_class` - Complexity class name (e.g., "Linear", "Quadratic")
///
/// # Returns
/// * `Ok(info)` - Detailed complexity class information
/// * `Err(message)` - Error if invalid complexity class
///
/// # Example
/// ```typescript
/// const info = await invoke('get_complexity_info', {
///   complexityClass: 'Linear'
/// });
/// console.log(`${info.notation}: ${info.description}`);
/// ```
#[tauri::command]
pub fn get_complexity_info(
    complexity_class: String,
) -> Result<ComplexityInfoResponse, String> {
    debug!("Getting complexity info for: {}", complexity_class);

    let complexity = parse_complexity_class(&complexity_class)?;

    Ok(match complexity {
        ComplexityClass::Constant => ComplexityInfoResponse {
            class_name: "Constant".to_string(),
            notation: "Ω(1)".to_string(),
            subtask_range: "1".to_string(),
            description: "Constant-time operations requiring no decomposition".to_string(),
            examples: vec![
                "Simple configuration changes".to_string(),
                "Single variable updates".to_string(),
                "Basic value lookups".to_string(),
            ],
        },
        ComplexityClass::Logarithmic => ComplexityInfoResponse {
            class_name: "Logarithmic".to_string(),
            notation: "Ω(log n)".to_string(),
            subtask_range: "2-3".to_string(),
            description: "Logarithmic operations with divide-and-conquer patterns".to_string(),
            examples: vec![
                "Binary search implementations".to_string(),
                "Tree traversals".to_string(),
                "Hierarchical data processing".to_string(),
            ],
        },
        ComplexityClass::Linear => ComplexityInfoResponse {
            class_name: "Linear".to_string(),
            notation: "Ω(n)".to_string(),
            subtask_range: "3-5".to_string(),
            description: "Linear operations processing collections or sequences".to_string(),
            examples: vec![
                "CRUD API implementations".to_string(),
                "List processing and transformations".to_string(),
                "Sequential workflows".to_string(),
            ],
        },
        ComplexityClass::Linearithmic => ComplexityInfoResponse {
            class_name: "Linearithmic".to_string(),
            notation: "Ω(n log n)".to_string(),
            subtask_range: "4-6".to_string(),
            description: "Efficient sorting and indexing operations".to_string(),
            examples: vec![
                "Merge sort implementations".to_string(),
                "Database indexing".to_string(),
                "Optimized search algorithms".to_string(),
            ],
        },
        ComplexityClass::Quadratic => ComplexityInfoResponse {
            class_name: "Quadratic".to_string(),
            notation: "Ω(n²)".to_string(),
            subtask_range: "5-10".to_string(),
            description: "Nested iterations or pairwise comparisons".to_string(),
            examples: vec![
                "Matrix operations".to_string(),
                "Nested loop algorithms".to_string(),
                "All-pairs computations".to_string(),
            ],
        },
        ComplexityClass::Exponential => ComplexityInfoResponse {
            class_name: "Exponential".to_string(),
            notation: "Ω(2^n)".to_string(),
            subtask_range: "8-15".to_string(),
            description: "Combinatorial or exhaustive search problems".to_string(),
            examples: vec![
                "Backtracking algorithms".to_string(),
                "Permutation generation".to_string(),
                "NP-complete problem solving".to_string(),
            ],
        },
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse complexity class string to enum
fn parse_complexity_class(class_str: &str) -> Result<ComplexityClass, String> {
    match class_str.to_lowercase().as_str() {
        "constant" => Ok(ComplexityClass::Constant),
        "logarithmic" => Ok(ComplexityClass::Logarithmic),
        "linear" => Ok(ComplexityClass::Linear),
        "linearithmic" => Ok(ComplexityClass::Linearithmic),
        "quadratic" => Ok(ComplexityClass::Quadratic),
        "exponential" => Ok(ComplexityClass::Exponential),
        _ => Err(format!(
            "Invalid complexity class '{}'. Valid values: Constant, Logarithmic, Linear, Linearithmic, Quadratic, Exponential",
            class_str
        )),
    }
}

/// Convert OptimizationResult to response format
fn convert_optimization_result(result: OptimizationResult) -> OptimizeTaskResponse {
    OptimizeTaskResponse {
        complexity_class: format!("{}", result.complexity_class),
        recommended_subtasks: result.recommended_subtasks,
        confidence: result.confidence,
        reasoning: result.reasoning,
        memory_adjustment: result.memory_adjustment.map(|ma| MemoryAdjustmentResponse {
            historical_success_rate: ma.historical_success_rate,
            adjustment: ma.adjustment,
            reasoning: ma.reasoning,
        }),
        llm_estimate: result.llm_estimate.map(|est| ComplexityEstimateResponse {
            complexity_class: format!("{}", est.complexity_class),
            confidence: est.confidence,
            reasoning: est.reasoning,
            estimated_subtasks: est.recommended_subtasks,
        }),
    }
}

/// Convert InstanceCalculation to response format
fn convert_instance_calculation(result: InstanceCalculation) -> CalculateInstancesResponse {
    CalculateInstancesResponse {
        recommended_instances: result.recommended_instances,
        subtasks_per_instance: result.subtasks_per_instance,
        resource_constrained: result.resource_constrained,
        reasoning: result.reasoning,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complexity_class() {
        assert_eq!(
            parse_complexity_class("Constant").unwrap(),
            ComplexityClass::Constant
        );
        assert_eq!(
            parse_complexity_class("linear").unwrap(),
            ComplexityClass::Linear
        );
        assert_eq!(
            parse_complexity_class("QUADRATIC").unwrap(),
            ComplexityClass::Quadratic
        );
        assert!(parse_complexity_class("Invalid").is_err());
    }

    #[test]
    fn test_calculate_instances_zero_subtasks() {
        let state = OptimizerState::new();
        let result = calculate_instances("Linear".to_string(), 0, State::from(&state));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be greater than 0"));
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
            assert!(result.is_ok());
            let info = result.unwrap();
            assert_eq!(info.class_name, class);
            assert!(!info.notation.is_empty());
            assert!(!info.description.is_empty());
            assert!(!info.examples.is_empty());
        }
    }

    #[test]
    fn test_optimizer_state_initialization() {
        let state = OptimizerState::new();
        // Verify initial state
        assert!(state.optimizer.lock().unwrap().is_none());
    }

    #[test]
    fn test_convert_instance_calculation() {
        let calc = InstanceCalculation {
            recommended_instances: 3,
            subtasks_per_instance: 1.67,
            complexity_class: ComplexityClass::Linear,
            resource_constrained: false,
            reasoning: "Test reasoning".to_string(),
        };

        let response = convert_instance_calculation(calc);
        assert_eq!(response.recommended_instances, 3);
        assert!((response.subtasks_per_instance - 1.67).abs() < 0.01);
        assert!(!response.resource_constrained);
    }
}
