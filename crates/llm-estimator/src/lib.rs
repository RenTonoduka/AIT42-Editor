//! LLM-based complexity estimation engine for AIT42 Editor
//!
//! This crate provides automatic task complexity classification using
//! Anthropic's Claude API with Big Omega notation.
//!
//! # Features
//!
//! - **LLM-powered analysis**: Uses Claude Sonnet 4.5 for intelligent classification
//! - **In-memory caching**: Reduces API calls and improves performance
//! - **Robust parsing**: Handles various LLM output formats (JSON, markdown, explanations)
//! - **Error handling**: Comprehensive validation and error recovery
//! - **Configurable**: Adjustable model, temperature, and timeout settings
//!
//! # Quick Start
//!
//! ```no_run
//! use llm_estimator::{AnthropicClient, CachedEstimator};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client from environment variable
//!     let client = AnthropicClient::from_env()?;
//!
//!     // Wrap with caching layer
//!     let estimator = CachedEstimator::new(client);
//!
//!     // Estimate complexity
//!     let estimate = estimator
//!         .estimate("Implement user authentication with JWT", 0)
//!         .await?;
//!
//!     println!("Complexity: {}", estimate.complexity_class);
//!     println!("Recommended subtasks: {}", estimate.recommended_subtasks);
//!     println!("Confidence: {:.2}", estimate.confidence);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    CachedEstimator                      │
//! │  (In-memory cache, statistics, LRU eviction)            │
//! └────────────────────┬────────────────────────────────────┘
//!                      │ Cache miss
//!                      ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │                  AnthropicClient                        │
//! │  (API calls, timeout handling, model config)            │
//! └────────────────────┬────────────────────────────────────┘
//!                      │ Build prompt
//!                      ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │                  PromptBuilder                          │
//! │  (Task description, context, subtask hints)             │
//! └────────────────────┬────────────────────────────────────┘
//!                      │ Generate prompt
//!                      ▼
//!                 Claude API
//!                      │ JSON response
//!                      ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │                 ResponseParser                          │
//! │  (JSON extraction, validation, error handling)          │
//! └────────────────────┬────────────────────────────────────┘
//!                      │ Validated estimate
//!                      ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │                ComplexityEstimate                       │
//! │  (complexity_class, reasoning, subtasks, confidence)    │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Environment Variables
//!
//! - `ANTHROPIC_API_KEY`: Required for API access (get from console.anthropic.com)
//!
//! # Performance
//!
//! - **API latency**: ~1-2s per request (model dependent)
//! - **Cache hit rate**: Typically >50% in production usage
//! - **Memory usage**: ~1KB per cached estimate (1000 entries ≈ 1MB)
//!
//! # Error Handling
//!
//! All errors are typed via `EstimatorError`:
//!
//! ```no_run
//! use llm_estimator::{AnthropicClient, EstimatorError};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = match AnthropicClient::from_env() {
//!     Ok(c) => c,
//!     Err(EstimatorError::MissingApiKey) => {
//!         eprintln!("Please set ANTHROPIC_API_KEY environment variable");
//!         return Ok(());
//!     }
//!     Err(e) => return Err(e.into()),
//! };
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod anthropic_client;
mod cache;
mod error;
mod prompt_builder;
mod response_parser;

// Public API
pub use anthropic_client::{AnthropicClient, ClientConfig};
pub use cache::{CacheStats, CachedEstimator};
pub use error::{EstimatorError, ParseError, Result};
pub use prompt_builder::PromptBuilder;
pub use response_parser::{ComplexityEstimate, ResponseParser};

// Re-export ComplexityClass from omega-theory for convenience
pub use omega_theory::ComplexityClass;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api_exports() {
        // Verify all public types are accessible
        let _: AnthropicClient;
        let _: CachedEstimator;
        let _: ComplexityEstimate;
        let _: ComplexityClass;
    }

    #[test]
    fn test_error_types() {
        // Verify error types are accessible
        let _: EstimatorError;
        let _: ParseError;
    }
}
