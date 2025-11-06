//! In-memory caching layer for complexity estimates

use crate::anthropic_client::AnthropicClient;
use crate::error::Result;
use crate::response_parser::ComplexityEstimate;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total number of estimation requests
    pub total_requests: usize,

    /// Number of cache hits
    pub cache_hits: usize,

    /// Number of cache misses (API calls)
    pub cache_misses: usize,

    /// Current cache size
    pub cache_size: usize,
}

impl CacheStats {
    /// Calculate cache hit rate (0.0-1.0)
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_requests as f64
        }
    }
}

/// Cached estimator with in-memory cache
pub struct CachedEstimator {
    client: AnthropicClient,
    cache: Arc<Mutex<HashMap<String, ComplexityEstimate>>>,
    stats: Arc<Mutex<CacheStats>>,
    max_cache_size: usize,
}

impl CachedEstimator {
    /// Create a new cached estimator
    ///
    /// # Arguments
    ///
    /// * `client` - The Anthropic client to use for API calls
    pub fn new(client: AnthropicClient) -> Self {
        Self::with_max_size(client, 1000) // Default: cache up to 1000 entries
    }

    /// Create a new cached estimator with custom maximum cache size
    ///
    /// # Arguments
    ///
    /// * `client` - The Anthropic client to use for API calls
    /// * `max_size` - Maximum number of entries to cache
    pub fn with_max_size(client: AnthropicClient, max_size: usize) -> Self {
        info!("Initialized cached estimator with max size: {}", max_size);
        Self {
            client,
            cache: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(CacheStats::default())),
            max_cache_size: max_size,
        }
    }

    /// Estimate complexity with caching
    ///
    /// Checks cache first, falls back to API call if not found
    ///
    /// # Arguments
    ///
    /// * `task_description` - The task description to analyze
    /// * `current_subtasks` - Current number of subtasks (0 if new task)
    ///
    /// # Returns
    ///
    /// A `ComplexityEstimate` from cache or fresh API call
    ///
    /// # Errors
    ///
    /// Returns `EstimatorError` if API call fails (cache misses only)
    pub async fn estimate(
        &self,
        task_description: &str,
        current_subtasks: usize,
    ) -> Result<ComplexityEstimate> {
        self.estimate_with_context(task_description, current_subtasks, None)
            .await
    }

    /// Estimate complexity with caching and additional context
    ///
    /// # Arguments
    ///
    /// * `task_description` - The task description to analyze
    /// * `current_subtasks` - Current number of subtasks (0 if new task)
    /// * `context` - Optional additional context
    ///
    /// # Returns
    ///
    /// A `ComplexityEstimate` from cache or fresh API call
    ///
    /// # Errors
    ///
    /// Returns `EstimatorError` if API call fails (cache misses only)
    pub async fn estimate_with_context(
        &self,
        task_description: &str,
        current_subtasks: usize,
        context: Option<&str>,
    ) -> Result<ComplexityEstimate> {
        // Generate cache key
        let cache_key = self.generate_cache_key(task_description, current_subtasks, context);

        // Update total requests
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_requests += 1;
        }

        // Check cache
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached_estimate) = cache.get(&cache_key) {
                debug!("Cache hit for task: {}", task_description);
                let mut stats = self.stats.lock().unwrap();
                stats.cache_hits += 1;
                return Ok(cached_estimate.clone());
            }
        }

        // Cache miss - call API
        debug!("Cache miss for task: {}", task_description);
        let estimate = self
            .client
            .estimate_complexity_with_context(task_description, current_subtasks, context)
            .await?;

        // Update cache with LRU-style eviction
        {
            let mut cache = self.cache.lock().unwrap();

            // If cache is full, remove oldest entry (simple eviction strategy)
            if cache.len() >= self.max_cache_size {
                if let Some(first_key) = cache.keys().next().cloned() {
                    cache.remove(&first_key);
                    debug!("Evicted cache entry to make room");
                }
            }

            cache.insert(cache_key, estimate.clone());

            let mut stats = self.stats.lock().unwrap();
            stats.cache_misses += 1;
            stats.cache_size = cache.len();
        }

        Ok(estimate)
    }

    /// Generate a cache key from task description and parameters
    fn generate_cache_key(
        &self,
        task: &str,
        subtasks: usize,
        context: Option<&str>,
    ) -> String {
        match context {
            Some(ctx) => format!("{}:{}:{}", task, subtasks, ctx),
            None => format!("{}:{}", task, subtasks),
        }
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        let mut stats = self.stats.lock().unwrap();
        stats.cache_size = 0;
        info!("Cache cleared");
    }

    /// Get current cache statistics
    pub fn stats(&self) -> CacheStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    /// Get the number of entries currently in cache
    pub fn cache_size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Pre-warm cache with known estimates
    ///
    /// Useful for testing or when you have pre-computed estimates
    pub fn insert_cached(
        &self,
        task_description: &str,
        current_subtasks: usize,
        estimate: ComplexityEstimate,
    ) {
        let cache_key = self.generate_cache_key(task_description, current_subtasks, None);
        let mut cache = self.cache.lock().unwrap();

        // If cache is full, remove an entry (simple eviction strategy)
        if cache.len() >= self.max_cache_size {
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
                debug!("Evicted cache entry to make room");
            }
        }

        cache.insert(cache_key, estimate);
        let mut stats = self.stats.lock().unwrap();
        stats.cache_size = cache.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anthropic_client::ClientConfig;

    fn create_test_client() -> AnthropicClient {
        // Create client with dummy key for tests (won't make real API calls)
        AnthropicClient::with_config(
            "sk-ant-test".to_string(),
            ClientConfig::default(),
        )
        .unwrap()
    }

    fn create_test_estimate() -> ComplexityEstimate {
        ComplexityEstimate {
            complexity_class: "Ω(n)".to_string(),
            reasoning: "Test estimate".to_string(),
            recommended_subtasks: 3,
            confidence: 0.8,
        }
    }

    #[test]
    fn test_cache_creation() {
        let client = create_test_client();
        let estimator = CachedEstimator::new(client);
        assert_eq!(estimator.cache_size(), 0);
    }

    #[test]
    fn test_cache_stats_initial() {
        let client = create_test_client();
        let estimator = CachedEstimator::new(client);
        let stats = estimator.stats();

        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.cache_size, 0);
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_insert_cached() {
        let client = create_test_client();
        let estimator = CachedEstimator::new(client);

        estimator.insert_cached(
            "Test task",
            0,
            create_test_estimate(),
        );

        assert_eq!(estimator.cache_size(), 1);
        let stats = estimator.stats();
        assert_eq!(stats.cache_size, 1);
    }

    #[test]
    fn test_clear_cache() {
        let client = create_test_client();
        let estimator = CachedEstimator::new(client);

        estimator.insert_cached(
            "Test task",
            0,
            create_test_estimate(),
        );
        assert_eq!(estimator.cache_size(), 1);

        estimator.clear_cache();
        assert_eq!(estimator.cache_size(), 0);
    }

    #[test]
    fn test_cache_key_generation() {
        let client = create_test_client();
        let estimator = CachedEstimator::new(client);

        let key1 = estimator.generate_cache_key("Task A", 0, None);
        let key2 = estimator.generate_cache_key("Task A", 0, None);
        let key3 = estimator.generate_cache_key("Task B", 0, None);
        let key4 = estimator.generate_cache_key("Task A", 3, None);
        let key5 = estimator.generate_cache_key("Task A", 0, Some("context"));

        assert_eq!(key1, key2); // Same parameters = same key
        assert_ne!(key1, key3); // Different task
        assert_ne!(key1, key4); // Different subtasks
        assert_ne!(key1, key5); // Different context
    }

    #[test]
    fn test_max_cache_size() {
        let client = create_test_client();
        let estimator = CachedEstimator::with_max_size(client, 3);

        // Add 4 items (one more than max)
        for i in 0..4 {
            estimator.insert_cached(
                &format!("Task {}", i),
                0,
                create_test_estimate(),
            );
        }

        // Cache should not exceed max size (LRU eviction)
        assert!(estimator.cache_size() <= 3);
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let mut stats = CacheStats {
            total_requests: 10,
            cache_hits: 7,
            cache_misses: 3,
            cache_size: 5,
        };

        assert_eq!(stats.hit_rate(), 0.7);

        stats.total_requests = 0;
        assert_eq!(stats.hit_rate(), 0.0);
    }
}
