# llm-estimator

LLM-based complexity estimation engine for AIT42 Editor using Anthropic Claude API.

## Features

- **LLM-powered analysis**: Uses Claude Sonnet 4.5 for intelligent task complexity classification
- **In-memory caching**: Reduces API calls by up to 50%+ with automatic LRU eviction
- **Robust parsing**: Handles various LLM output formats (JSON, markdown, explanations)
- **Comprehensive error handling**: Type-safe error handling with detailed error messages
- **Configurable**: Adjustable model, temperature, timeout settings, and cache size

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-estimator = { path = "../llm-estimator" }
omega-theory = { path = "../omega-theory" }
tokio = { version = "1.35", features = ["full"] }
```

## Quick Start

```rust
use llm_estimator::{AnthropicClient, CachedEstimator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client from ANTHROPIC_API_KEY environment variable
    let client = AnthropicClient::from_env()?;

    // Wrap with caching layer
    let estimator = CachedEstimator::new(client);

    // Estimate complexity
    let estimate = estimator
        .estimate("Implement user authentication with JWT tokens", 0)
        .await?;

    println!("Complexity: {}", estimate.complexity_class);
    println!("Reasoning: {}", estimate.reasoning);
    println!("Recommended subtasks: {}", estimate.recommended_subtasks);
    println!("Confidence: {:.2}", estimate.confidence);

    Ok(())
}
```

## Usage Examples

### Basic Estimation

```rust
use llm_estimator::{AnthropicClient, CachedEstimator};

let client = AnthropicClient::from_env()?;
let estimator = CachedEstimator::new(client);

let estimate = estimator.estimate("Add logging statement", 0).await?;
// Expected: Ω(1) - Constant complexity
```

### With Additional Context

```rust
let estimate = client
    .estimate_complexity_with_context(
        "Add new API endpoint",
        0,
        Some("Legacy system with complex authentication and no tests"),
    )
    .await?;
// Context influences the complexity estimation
```

### Custom Configuration

```rust
use llm_estimator::{AnthropicClient, ClientConfig};

let config = ClientConfig {
    model: "claude-opus-4-20250514".to_string(),
    temperature: 0.2, // Even lower for maximum consistency
    max_tokens: 2048,
    timeout_secs: 60,
};

let client = AnthropicClient::with_config(api_key, config)?;
```

### Cache Statistics

```rust
let estimator = CachedEstimator::new(client);

// Make several requests...
for task in tasks {
    estimator.estimate(task, 0).await?;
}

// Check cache performance
let stats = estimator.stats();
println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
println!("Total requests: {}", stats.total_requests);
println!("Cache hits: {}", stats.cache_hits);
println!("Cache size: {}", stats.cache_size);
```

## Complexity Classes

The estimator classifies tasks into 6 Big Omega complexity tiers:

| Class | Notation | Description | Subtasks |
|-------|----------|-------------|----------|
| Constant | Ω(1) | Simple config changes, trivial operations | 1 |
| Logarithmic | Ω(log n) | Binary search, tree operations | 2-3 |
| Linear | Ω(n) | Standard CRUD, single-pass processing | 3-5 |
| Linearithmic | Ω(n log n) | Sorting, indexing, optimization | 4-6 |
| Quadratic | Ω(n²) | Matrix operations, nested loops | 5-10 |
| Exponential | Ω(2^n) | Combinatorial problems, brute-force | 8-15 |

## Environment Setup

1. Get your API key from [console.anthropic.com](https://console.anthropic.com/)

2. Set environment variable:
   ```bash
   export ANTHROPIC_API_KEY=sk-ant-xxxxx
   ```

3. Or use `.env` file:
   ```
   ANTHROPIC_API_KEY=sk-ant-xxxxx
   ```

## Error Handling

All errors are strongly typed via `EstimatorError`:

```rust
use llm_estimator::{AnthropicClient, EstimatorError};

match AnthropicClient::from_env() {
    Ok(client) => { /* Use client */ },
    Err(EstimatorError::MissingApiKey) => {
        eprintln!("Please set ANTHROPIC_API_KEY environment variable");
    },
    Err(EstimatorError::Timeout(secs)) => {
        eprintln!("Request timed out after {} seconds", secs);
    },
    Err(EstimatorError::ApiError(msg)) => {
        eprintln!("Anthropic API error: {}", msg);
    },
    Err(e) => {
        eprintln!("Other error: {}", e);
    },
}
```

## Performance

- **API latency**: ~1-2 seconds per request (model dependent)
- **Cache hit rate**: Typically >50% in production usage
- **Memory usage**: ~1KB per cached estimate (1000 entries ≈ 1MB)
- **Timeout**: Configurable (default 30s)

## Testing

```bash
# Run all tests (mock tests only, no API key needed)
cargo test -p llm-estimator

# Run with real API (requires ANTHROPIC_API_KEY)
ANTHROPIC_API_KEY=sk-ant-xxxxx cargo test -p llm-estimator

# Run specific test
cargo test -p llm-estimator test_parser_simple_task
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    CachedEstimator                      │
│  (In-memory cache, statistics, LRU eviction)            │
└────────────────────┬────────────────────────────────────┘
                     │ Cache miss
                     ▼
┌─────────────────────────────────────────────────────────┐
│                  AnthropicClient                        │
│  (API calls, timeout handling, model config)            │
└────────────────────┬────────────────────────────────────┘
                     │ Build prompt
                     ▼
┌─────────────────────────────────────────────────────────┐
│                  PromptBuilder                          │
│  (Task description, context, subtask hints)             │
└────────────────────┬────────────────────────────────────┘
                     │ Generate prompt
                     ▼
                Claude API
                     │ JSON response
                     ▼
┌─────────────────────────────────────────────────────────┐
│                 ResponseParser                          │
│  (JSON extraction, validation, error handling)          │
└────────────────────┬────────────────────────────────────┘
                     │ Validated estimate
                     ▼
┌─────────────────────────────────────────────────────────┐
│                ComplexityEstimate                       │
│  (complexity_class, reasoning, subtasks, confidence)    │
└─────────────────────────────────────────────────────────┘
```

## Integration with omega-theory

The estimator integrates seamlessly with the `omega-theory` crate:

```rust
use llm_estimator::{AnthropicClient, CachedEstimator};
use omega_theory::ComplexityClass;

let estimator = CachedEstimator::new(AnthropicClient::from_env()?);
let estimate = estimator.estimate("Task description", 0).await?;

// Convert to ComplexityClass enum
let complexity_class: ComplexityClass = estimate.to_complexity_class()?;
let subtask_range = complexity_class.to_subtask_range();

println!("Recommended: {:?} subtasks", subtask_range);
```

## Graceful Fallback

If the API key is not available, you can gracefully fallback to the placeholder logic in `omega-theory`:

```rust
use omega_theory::ComplexityClass;

let complexity = match AnthropicClient::from_env() {
    Ok(client) => {
        let estimator = CachedEstimator::new(client);
        let estimate = estimator.estimate(task, 0).await?;
        estimate.to_complexity_class()?
    },
    Err(_) => {
        // Fallback to placeholder logic
        ComplexityClass::from_description(task)
    }
};
```

## License

MIT OR Apache-2.0

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for development guidelines.
