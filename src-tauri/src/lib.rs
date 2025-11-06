//! AIT42 Editor Library
//!
//! This library provides the core functionality for the AIT42 Editor,
//! including the subtask optimizer and other components.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_name_repetitions)]

pub mod optimizer;

// Re-export commonly used types
pub use optimizer::{
    ComplexityClass, ComplexityEstimate, InstanceCalculation, InstanceCalculator, MemoryAdjustment,
    OptimizationResult, OptimizerError, SubtaskOptimizer,
};
