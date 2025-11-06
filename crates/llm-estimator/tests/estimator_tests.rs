//! Integration tests for LLM estimator
//!
//! These tests verify the full pipeline from API client through caching.
//! Tests are organized into:
//! - Mock tests (always run, no API key needed)
//! - Integration tests (require ANTHROPIC_API_KEY)

use llm_estimator::{
    AnthropicClient, CachedEstimator, ClientConfig, ComplexityEstimate, EstimatorError,
    ResponseParser,
};

// ============================================================================
// Mock Tests (No API Key Required)
// ============================================================================

#[test]
fn test_parser_simple_task() {
    let json = r#"{
        "complexity_class": "Ω(1)",
        "reasoning": "Simple configuration change",
        "recommended_subtasks": 1,
        "confidence": 0.9
    }"#;

    let estimate = ResponseParser::parse(json).unwrap();
    assert_eq!(estimate.complexity_class, "Ω(1)");
    assert_eq!(estimate.recommended_subtasks, 1);
    assert_eq!(estimate.confidence, 0.9);
}

#[test]
fn test_parser_complex_task() {
    let json = r#"{
        "complexity_class": "Ω(n²)",
        "reasoning": "Requires nested iteration over matrix data",
        "recommended_subtasks": 8,
        "confidence": 0.75
    }"#;

    let estimate = ResponseParser::parse(json).unwrap();
    assert_eq!(estimate.complexity_class, "Ω(n²)");
    assert_eq!(estimate.recommended_subtasks, 8);
}

#[test]
fn test_parser_with_markdown_wrapper() {
    let response = r#"```json
{
    "complexity_class": "Ω(n log n)",
    "reasoning": "Sorting algorithm implementation",
    "recommended_subtasks": 5,
    "confidence": 0.88
}
```"#;

    let estimate = ResponseParser::parse(response).unwrap();
    assert_eq!(estimate.complexity_class, "Ω(n log n)");
}

#[test]
fn test_parser_with_explanation() {
    let response = r#"Based on my analysis, this task requires:

{
    "complexity_class": "Ω(n)",
    "reasoning": "Linear scan through user database",
    "recommended_subtasks": 4,
    "confidence": 0.85
}

This should provide good decomposition."#;

    let estimate = ResponseParser::parse(response).unwrap();
    assert_eq!(estimate.complexity_class, "Ω(n)");
}

#[test]
fn test_parser_invalid_complexity_class() {
    let json = r#"{
        "complexity_class": "O(n)",
        "reasoning": "Wrong notation",
        "recommended_subtasks": 3,
        "confidence": 0.8
    }"#;

    let result = ResponseParser::parse(json);
    assert!(result.is_err());
}

#[test]
fn test_parser_invalid_subtask_count_zero() {
    let json = r#"{
        "complexity_class": "Ω(n)",
        "reasoning": "Test",
        "recommended_subtasks": 0,
        "confidence": 0.8
    }"#;

    let result = ResponseParser::parse(json);
    assert!(result.is_err());
}

#[test]
fn test_parser_invalid_subtask_count_high() {
    let json = r#"{
        "complexity_class": "Ω(n)",
        "reasoning": "Test",
        "recommended_subtasks": 100,
        "confidence": 0.8
    }"#;

    let result = ResponseParser::parse(json);
    assert!(result.is_err());
}

#[test]
fn test_parser_invalid_confidence() {
    let json = r#"{
        "complexity_class": "Ω(n)",
        "reasoning": "Test",
        "recommended_subtasks": 3,
        "confidence": 1.5
    }"#;

    let result = ResponseParser::parse(json);
    assert!(result.is_err());
}

#[test]
fn test_client_creation_empty_key() {
    let result = AnthropicClient::new(String::new());
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        EstimatorError::InvalidConfig(_)
    ));
}

#[test]
fn test_client_from_env_missing_key() {
    // Temporarily unset env var
    std::env::remove_var("ANTHROPIC_API_KEY");

    let result = AnthropicClient::from_env();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EstimatorError::MissingApiKey));
}

#[test]
fn test_client_config_defaults() {
    let config = ClientConfig::default();
    assert_eq!(config.model, "claude-sonnet-4-5-20250929");
    assert_eq!(config.temperature, 0.3);
    assert_eq!(config.max_tokens, 1024);
    assert_eq!(config.timeout_secs, 30);
}

// ============================================================================
// Integration Tests (Require ANTHROPIC_API_KEY)
// ============================================================================

/// Helper to check if API key is available
fn api_key_available() -> bool {
    std::env::var("ANTHROPIC_API_KEY").is_ok()
}

/// Helper to create mock estimate for tests
fn create_mock_estimate(complexity: &str, subtasks: usize) -> ComplexityEstimate {
    ComplexityEstimate {
        complexity_class: complexity.to_string(),
        reasoning: "Mock estimate for testing".to_string(),
        recommended_subtasks: subtasks,
        confidence: 0.8,
    }
}

#[tokio::test]
async fn test_cached_estimator_mock() {
    // This test doesn't require API key - uses manual cache insertion
    let client = AnthropicClient::with_config(
        "sk-ant-test-key".to_string(),
        ClientConfig::default(),
    )
    .unwrap();

    let estimator = CachedEstimator::new(client);

    // Pre-populate cache
    estimator.insert_cached(
        "Simple config change",
        0,
        create_mock_estimate("Ω(1)", 1),
    );

    // Verify cache works
    assert_eq!(estimator.cache_size(), 1);
    let stats = estimator.stats();
    assert_eq!(stats.cache_size, 1);
}

#[tokio::test]
async fn test_real_api_simple_task() {
    if !api_key_available() {
        println!("Skipping real API test: ANTHROPIC_API_KEY not set");
        return;
    }

    let client = AnthropicClient::from_env().unwrap();
    let estimator = CachedEstimator::new(client);

    let estimate = estimator
        .estimate("Add a logging statement to existing function", 0)
        .await
        .unwrap();

    // Should be classified as Constant (Ω(1))
    assert_eq!(estimate.complexity_class, "Ω(1)");
    assert!(estimate.recommended_subtasks <= 2);
    assert!(estimate.confidence > 0.7);

    println!("✓ Simple task: {}", estimate.complexity_class);
    println!("  Reasoning: {}", estimate.reasoning);
    println!("  Subtasks: {}", estimate.recommended_subtasks);
    println!("  Confidence: {:.2}", estimate.confidence);
}

#[tokio::test]
async fn test_real_api_linear_task() {
    if !api_key_available() {
        println!("Skipping real API test: ANTHROPIC_API_KEY not set");
        return;
    }

    let client = AnthropicClient::from_env().unwrap();
    let estimator = CachedEstimator::new(client);

    let estimate = estimator
        .estimate("Implement REST API endpoint for user profile retrieval", 0)
        .await
        .unwrap();

    // Should be classified as Linear (Ω(n)) or similar
    assert!(
        estimate.complexity_class == "Ω(n)" || estimate.complexity_class == "Ω(log n)"
    );
    assert!(estimate.recommended_subtasks >= 2);
    assert!(estimate.confidence > 0.6);

    println!("✓ Linear task: {}", estimate.complexity_class);
    println!("  Reasoning: {}", estimate.reasoning);
    println!("  Subtasks: {}", estimate.recommended_subtasks);
    println!("  Confidence: {:.2}", estimate.confidence);
}

#[tokio::test]
async fn test_real_api_complex_task() {
    if !api_key_available() {
        println!("Skipping real API test: ANTHROPIC_API_KEY not set");
        return;
    }

    let client = AnthropicClient::from_env().unwrap();
    let estimator = CachedEstimator::new(client);

    let estimate = estimator
        .estimate(
            "Implement matrix multiplication with optimization and parallel processing",
            0,
        )
        .await
        .unwrap();

    // Should be classified as Quadratic (Ω(n²)) or higher
    assert!(
        estimate.complexity_class == "Ω(n²)"
        || estimate.complexity_class == "Ω(n log n)"
        || estimate.complexity_class == "Ω(n)"
    );
    assert!(estimate.recommended_subtasks >= 3);

    println!("✓ Complex task: {}", estimate.complexity_class);
    println!("  Reasoning: {}", estimate.reasoning);
    println!("  Subtasks: {}", estimate.recommended_subtasks);
    println!("  Confidence: {:.2}", estimate.confidence);
}

#[tokio::test]
async fn test_cache_effectiveness() {
    if !api_key_available() {
        println!("Skipping cache test: ANTHROPIC_API_KEY not set");
        return;
    }

    let client = AnthropicClient::from_env().unwrap();
    let estimator = CachedEstimator::new(client);

    let task = "Implement user authentication system";

    // First call - cache miss
    let _estimate1 = estimator.estimate(task, 0).await.unwrap();
    let stats1 = estimator.stats();
    assert_eq!(stats1.cache_misses, 1);
    assert_eq!(stats1.cache_hits, 0);

    // Second call - cache hit
    let _estimate2 = estimator.estimate(task, 0).await.unwrap();
    let stats2 = estimator.stats();
    assert_eq!(stats2.cache_misses, 1);
    assert_eq!(stats2.cache_hits, 1);

    // Hit rate should be 50%
    assert!((stats2.hit_rate() - 0.5).abs() < 0.01);

    println!("✓ Cache effectiveness:");
    println!("  Total requests: {}", stats2.total_requests);
    println!("  Cache hits: {}", stats2.cache_hits);
    println!("  Cache misses: {}", stats2.cache_misses);
    println!("  Hit rate: {:.2}%", stats2.hit_rate() * 100.0);
}

#[tokio::test]
async fn test_batch_estimation() {
    if !api_key_available() {
        println!("Skipping batch test: ANTHROPIC_API_KEY not set");
        return;
    }

    let client = AnthropicClient::from_env().unwrap();
    let estimator = CachedEstimator::new(client);

    let tasks = vec![
        "Update README documentation",
        "Add input validation",
        "Implement search algorithm",
        "Optimize database query performance",
    ];

    println!("✓ Batch estimation results:");
    for task in tasks {
        let estimate = estimator.estimate(task, 0).await.unwrap();
        println!(
            "  {} → {} ({} subtasks, {:.2} confidence)",
            task, estimate.complexity_class, estimate.recommended_subtasks, estimate.confidence
        );
    }

    let stats = estimator.stats();
    println!("\nCache stats:");
    println!("  Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    println!("  Cache size: {}", stats.cache_size);
}

#[tokio::test]
async fn test_context_aware_estimation() {
    if !api_key_available() {
        println!("Skipping context test: ANTHROPIC_API_KEY not set");
        return;
    }

    let client = AnthropicClient::from_env().unwrap();

    // Without context
    let estimate1 = client
        .estimate_complexity("Add new API endpoint", 0)
        .await
        .unwrap();

    // With context (should influence complexity)
    let estimate2 = client
        .estimate_complexity_with_context(
            "Add new API endpoint",
            0,
            Some("Legacy system with complex authentication and no tests"),
        )
        .await
        .unwrap();

    println!("✓ Context-aware estimation:");
    println!("  Without context: {} ({} subtasks)",
        estimate1.complexity_class, estimate1.recommended_subtasks);
    println!("  With context: {} ({} subtasks)",
        estimate2.complexity_class, estimate2.recommended_subtasks);

    // Context should generally increase complexity/subtasks
    // (though not guaranteed, so we just print for observation)
}
