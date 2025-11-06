//! Ω-Theory: Complexity analysis framework for AIT42 Editor
//!
//! This crate provides tools for analyzing task complexity using various
//! mathematical concepts related to "Omega" (Ω):
//!
//! - **Big Omega (Ω) Notation**: Asymptotic lower bounds for algorithm complexity
//! - **Prime Omega (Ω)**: Number-theoretic function counting prime factors (v1.7.0+)
//! - **Chaitin's Omega (Ω)**: Algorithmic information theory and halting probability (v1.7.0+)
//!
//! # Version 1.6.0 Features
//!
//! This initial release focuses on Big Omega notation for task complexity classification:
//!
//! - `ComplexityClass` enum with 6 complexity tiers (Ω(1) through Ω(2^n))
//! - Subtask decomposition recommendations based on complexity
//! - Serialization/deserialization support for integration with Task Master AI
//!
//! # Usage Example
//!
//! ```
//! use omega_theory::ComplexityClass;
//!
//! // Classify a task
//! let complexity = ComplexityClass::Linear;
//!
//! // Get recommended subtask count
//! let subtasks = complexity.to_subtask_range();
//! assert_eq!(subtasks, 3..=5);
//!
//! // Display in Big Omega notation
//! println!("Task complexity: {}", complexity); // "Ω(n)"
//!
//! // Serialize for storage
//! let json = serde_json::to_string(&complexity).unwrap();
//! ```
//!
//! # Future Roadmap (v1.7.0+)
//!
//! - LLM-based task description analysis for automatic complexity classification
//! - Prime Omega function for number-theoretic complexity measures
//! - Chaitin's Omega for algorithmic randomness and halting analysis
//! - Integration with AIT42 multi-agent system for adaptive task decomposition
//!
//! # Architecture
//!
//! ```text
//! omega-theory/
//! ├── big_omega.rs          ✅ v1.6.0 (ComplexityClass enum)
//! ├── prime_omega.rs        🔜 v1.7.0 (Prime factorization)
//! └── chaitins_omega.rs     🔜 v1.7.0 (Algorithmic information)
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod big_omega;
pub mod chaitins_omega;
pub mod prime_omega;

// Re-export primary types for convenience
pub use big_omega::ComplexityClass;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api() {
        // Ensure ComplexityClass is accessible from crate root
        let _ = ComplexityClass::Linear;
    }

    #[test]
    fn test_module_structure() {
        // Verify all modules compile
        let _ = big_omega::ComplexityClass::Constant;
        // prime_omega and chaitins_omega are placeholders (will panic if called)
    }
}
