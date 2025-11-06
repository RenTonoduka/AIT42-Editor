//! Error types for LLM-based complexity estimation

use thiserror::Error;

/// Main error type for the LLM estimator
#[derive(Error, Debug)]
pub enum EstimatorError {
    /// Anthropic API errors
    #[error("Anthropic API error: {0}")]
    ApiError(String),

    /// Response parsing failed
    #[error("Response parsing failed: {0}")]
    ParseError(#[from] ParseError),

    /// Request timeout
    #[error("Request timeout after {0}s")]
    Timeout(u64),

    /// Missing API key
    #[error("Missing ANTHROPIC_API_KEY environment variable")]
    MissingApiKey,

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Errors that occur during response parsing
#[derive(Error, Debug)]
pub enum ParseError {
    /// Invalid JSON structure
    #[error("Invalid JSON response: {0}")]
    InvalidJson(String),

    /// Invalid complexity class string
    #[error("Invalid complexity class: {0}")]
    InvalidComplexityClass(String),

    /// Subtask count out of valid range
    #[error("Invalid subtask count {0}: must be between 1 and 20")]
    InvalidSubtaskCount(usize),

    /// Confidence score out of valid range
    #[error("Invalid confidence score {0}: must be between 0.0 and 1.0")]
    InvalidConfidence(f64),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// LLM returned non-JSON response
    #[error("LLM returned non-JSON response (may contain markdown or explanations)")]
    NonJsonResponse,
}

/// Result type alias for estimator operations
pub type Result<T> = std::result::Result<T, EstimatorError>;
