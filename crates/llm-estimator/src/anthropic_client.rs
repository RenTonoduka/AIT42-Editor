//! Anthropic Claude API client for complexity estimation

use crate::error::{EstimatorError, Result};
use crate::prompt_builder::PromptBuilder;
use crate::response_parser::{ComplexityEstimate, ResponseParser};
use anthropic_sdk::Client;
use serde_json::json;
use std::time::Duration;
use tracing::{debug, info};

/// Configuration for the Anthropic client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Claude model to use (default: claude-sonnet-4-5-20250929)
    pub model: String,

    /// Temperature for sampling (0.0-1.0, default: 0.3 for consistency)
    pub temperature: f32,

    /// Maximum tokens in response (default: 1024)
    pub max_tokens: i32,

    /// Request timeout in seconds (default: 30)
    pub timeout_secs: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-5-20250929".to_string(),
            temperature: 0.3, // Low temperature for consistent classification
            max_tokens: 1024,
            timeout_secs: 30,
        }
    }
}

/// Anthropic Claude API client for complexity estimation
#[derive(Debug)]
pub struct AnthropicClient {
    api_key: String,
    config: ClientConfig,
}

impl AnthropicClient {
    /// Create a new Anthropic client
    ///
    /// # Arguments
    ///
    /// * `api_key` - Anthropic API key
    ///
    /// # Errors
    ///
    /// Returns `EstimatorError::InvalidConfig` if the API key is empty
    pub fn new(api_key: String) -> Result<Self> {
        Self::with_config(api_key, ClientConfig::default())
    }

    /// Create a new Anthropic client with custom configuration
    ///
    /// # Arguments
    ///
    /// * `api_key` - Anthropic API key
    /// * `config` - Client configuration
    ///
    /// # Errors
    ///
    /// Returns `EstimatorError::InvalidConfig` if the API key is empty
    pub fn with_config(api_key: String, config: ClientConfig) -> Result<Self> {
        if api_key.is_empty() {
            return Err(EstimatorError::InvalidConfig(
                "API key cannot be empty".to_string(),
            ));
        }

        info!(
            "Initialized Anthropic client with model: {}",
            config.model
        );

        Ok(Self { api_key, config })
    }

    /// Create a client from environment variable
    ///
    /// Reads `ANTHROPIC_API_KEY` from environment
    ///
    /// # Errors
    ///
    /// Returns `EstimatorError::MissingApiKey` if the environment variable is not set
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| EstimatorError::MissingApiKey)?;

        Self::new(api_key)
    }

    /// Estimate task complexity using Claude
    ///
    /// # Arguments
    ///
    /// * `task_description` - The task description to analyze
    /// * `current_subtasks` - Current number of subtasks (0 if new task)
    ///
    /// # Returns
    ///
    /// A `ComplexityEstimate` with classification, reasoning, and recommendations
    ///
    /// # Errors
    ///
    /// Returns `EstimatorError` if:
    /// - API request fails
    /// - Response parsing fails
    /// - Request times out
    pub async fn estimate_complexity(
        &self,
        task_description: &str,
        current_subtasks: usize,
    ) -> Result<ComplexityEstimate> {
        self.estimate_complexity_with_context(task_description, current_subtasks, None)
            .await
    }

    /// Estimate task complexity with additional context
    ///
    /// # Arguments
    ///
    /// * `task_description` - The task description to analyze
    /// * `current_subtasks` - Current number of subtasks (0 if new task)
    /// * `context` - Optional additional context for the LLM
    ///
    /// # Returns
    ///
    /// A `ComplexityEstimate` with classification, reasoning, and recommendations
    ///
    /// # Errors
    ///
    /// Returns `EstimatorError` if:
    /// - API request fails
    /// - Response parsing fails
    /// - Request times out
    pub async fn estimate_complexity_with_context(
        &self,
        task_description: &str,
        current_subtasks: usize,
        context: Option<&str>,
    ) -> Result<ComplexityEstimate> {
        debug!(
            "Estimating complexity for task: {} (current subtasks: {})",
            task_description, current_subtasks
        );

        // Build prompt
        let mut builder = PromptBuilder::new()
            .with_task(task_description)
            .with_current_subtasks(current_subtasks);

        if let Some(ctx) = context {
            builder = builder.with_context(ctx);
        }

        let prompt = builder.build();

        // Build messages array
        let messages = json!([
            {
                "role": "user",
                "content": prompt
            }
        ]);

        // Create request using builder pattern
        let request = Client::new()
            .auth(&self.api_key)
            .model(&self.config.model)
            .max_tokens(self.config.max_tokens)
            .temperature(self.config.temperature)
            .messages(&messages)
            .stream(false) // Disable streaming for simpler handling
            .build()
            .map_err(|e| EstimatorError::ApiError(e.to_string()))?;

        // Collect response text using callback
        let response_text = std::sync::Arc::new(tokio::sync::Mutex::new(String::new()));
        let response_text_clone = response_text.clone();

        // Make API call with timeout and collect response via callback
        let timeout_duration = Duration::from_secs(self.config.timeout_secs);
        tokio::time::timeout(timeout_duration, request.execute(|text: String| {
            let response_text = response_text_clone.clone();
            async move {
                let mut locked = response_text.lock().await;
                locked.push_str(&text);
            }
        }))
        .await
        .map_err(|_| EstimatorError::Timeout(self.config.timeout_secs))?
        .map_err(|e| EstimatorError::ApiError(e.to_string()))?;

        // Extract collected response
        let content_text = response_text.lock().await.clone();

        debug!("Received response: {}", content_text);

        // Parse response
        let estimate = ResponseParser::parse(&content_text)?;

        info!(
            "Estimated complexity: {} (confidence: {:.2})",
            estimate.complexity_class, estimate.confidence
        );

        Ok(estimate)
    }

    /// Get the current model name
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// Get the current temperature setting
    pub fn temperature(&self) -> f32 {
        self.config.temperature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ClientConfig::default();
        assert_eq!(config.model, "claude-sonnet-4-5-20250929");
        assert!((config.temperature - 0.3).abs() < 0.01);
        assert_eq!(config.max_tokens, 1024);
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_client_creation_with_empty_key() {
        let result = AnthropicClient::new(String::new());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EstimatorError::InvalidConfig(_)
        ));
    }

    #[test]
    fn test_client_config_customization() {
        let config = ClientConfig {
            model: "claude-opus-4-20250514".to_string(),
            temperature: 0.5,
            max_tokens: 2048,
            timeout_secs: 60,
        };

        assert_eq!(config.model, "claude-opus-4-20250514");
        assert!((config.temperature - 0.5).abs() < 0.01);
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(config.timeout_secs, 60);
    }

    // Note: Integration tests that make real API calls are in tests/estimator_tests.rs
    // These tests only verify the client structure and configuration
}
