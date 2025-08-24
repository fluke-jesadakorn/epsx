// Unified cache implementation with automatic Redis + in-memory fallback

use super::{Cache, CacheExt, CacheConfig, CacheStats, CacheError, InMemoryCache, RedisCache};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{warn, debug, info};

/// Cache health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheHealth {
    Healthy,
    Degraded,
    Failed,
}

/// Unified cache that automatically falls back from Redis to in-memory cache
pub struct UnifiedCache {
    redis_cache: Option<Arc<RedisCache>>,
    memory_cache: Arc<InMemoryCache>,
    redis_health: Arc<RwLock<CacheHealth>>,
    config: CacheConfig,
}

impl UnifiedCache {
    /// Create a new unified cache with Redis primary and in-memory fallback
    pub async fn new(redis_url: String, pool_size: u32, config: CacheConfig) -> Self {
        let memory_cache = Arc::new(InMemoryCache::new(config.clone()));
        
        // Attempt to create Redis cache
        let (redis_cache, initial_health) = match RedisCache::new(
            redis_url.clone(), 
            pool_size, 
            config.clone()
        ).await {
            Ok(redis) => {
                info!("UnifiedCache: Redis connection established successfully");
                (Some(Arc::new(redis)), CacheHealth::Healthy)
            }
            Err(e) => {
                warn!("UnifiedCache: Redis connection failed, using in-memory fallback: {}", e);
                (None, CacheHealth::Failed)
            }
        };

        Self {
            redis_cache,
            memory_cache,
            redis_health: Arc::new(RwLock::new(initial_health)),
            config,
        }
    }

    /// Check if Redis is healthy and attempt reconnection if needed
    async fn check_redis_health(&self) -> bool {
        let health = *self.redis_health.read().await;
        match health {
            CacheHealth::Healthy => true,
            CacheHealth::Degraded | CacheHealth::Failed => {
                // TODO: Implement periodic reconnection attempts
                false
            }
        }
    }

    /// Mark Redis as unhealthy
    async fn mark_redis_unhealthy(&self, error: &str) {
        let mut health = self.redis_health.write().await;
        let previous_health = *health;
        *health = CacheHealth::Failed;
        
        if previous_health != CacheHealth::Failed {
            warn!("UnifiedCache: Redis marked as unhealthy, falling back to memory cache: {}", error);
        }
    }

    /// Execute cache operation with automatic fallback
    async fn execute_with_fallback<T, F, Fut>(&self, operation_name: &str, redis_op: F) -> Result<T, CacheError> 
    where
        F: Fn(Arc<RedisCache>) -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, CacheError>> + Send,
        T: Send,
    {
        // Try Redis first if available and healthy
        if let Some(redis) = &self.redis_cache {
            if self.check_redis_health().await {
                match redis_op(redis.clone()).await {
                    Ok(result) => {
                        debug!("UnifiedCache: {} succeeded on Redis", operation_name);
                        return Ok(result);
                    }
                    Err(e) => {
                        self.mark_redis_unhealthy(&e.to_string()).await;
                        debug!("UnifiedCache: {} failed on Redis, will try memory cache: {}", operation_name, e);
                    }
                }
            }
        }

        // Redis failed or unavailable, operation should be handled by calling code
        Err(CacheError::ConnectionError("Redis unavailable, fallback needed".to_string()))
    }

    /// Execute read operation with automatic fallback to memory cache
    async fn read_with_fallback<T>(&self, key: &str) -> Result<Option<T>, CacheError>
    where
        T: for<'de> serde::Deserialize<'de> + Send + 'static,
    {
        // Try Redis first
        let redis_result = self.execute_with_fallback("get", |redis| async move {
            redis.get::<T>(key).await
        }).await;

        match redis_result {
            Ok(value) => Ok(value),
            Err(_) => {
                // Fallback to memory cache
                debug!("UnifiedCache: Falling back to memory cache for key: {}", key);
                self.memory_cache.get::<T>(key).await
            }
        }
    }

    /// Execute write operation on both caches (best effort)
    async fn write_to_both<T>(&self, key: &str, value: &T, ttl_seconds: Option<i64>) -> Result<(), CacheError>
    where
        T: serde::Serialize + Send + Sync,
    {
        // Always write to memory cache
        let memory_result = self.memory_cache.set(key, value, ttl_seconds).await;

        // Try to write to Redis (best effort)
        if let Some(redis) = &self.redis_cache {
            if self.check_redis_health().await {
                let _ = redis.set(key, value, ttl_seconds).await.map_err(|e| {
                    // Don't fail the operation if Redis write fails
                    warn!("UnifiedCache: Redis write failed for key {}: {}", key, e);
                });
            }
        }

        memory_result
    }
}

#[async_trait]
impl Cache for UnifiedCache {
    async fn get_raw(&self, key: &str) -> Result<Option<String>, CacheError> {
        // Try Redis first
        let redis_result = self.execute_with_fallback("get_raw", |redis| async move {
            redis.get_raw(key).await
        }).await;

        match redis_result {
            Ok(value) => Ok(value),
            Err(_) => {
                // Fallback to memory cache
                debug!("UnifiedCache: get_raw fallback to memory for key: {}", key);
                self.memory_cache.get_raw(key).await
            }
        }
    }

    async fn set_raw(&self, key: &str, value: &str, ttl_seconds: Option<i64>) -> Result<(), CacheError> {
        // Always write to memory cache first
        let memory_result = self.memory_cache.set_raw(key, value, ttl_seconds).await;

        // Try to write to Redis (best effort)
        if let Some(redis) = &self.redis_cache {
            if self.check_redis_health().await {
                if let Err(e) = redis.set_raw(key, value, ttl_seconds).await {
                    self.mark_redis_unhealthy(&e.to_string()).await;
                    debug!("UnifiedCache: Redis set_raw failed, but memory cache succeeded");
                }
            }
        }

        memory_result
    }

    async fn delete(&self, key: &str) -> Result<bool, CacheError> {
        let mut redis_deleted = false;
        
        // Try to delete from Redis first
        if let Some(redis) = &self.redis_cache {
            if self.check_redis_health().await {
                match redis.delete(key).await {
                    Ok(deleted) => redis_deleted = deleted,
                    Err(e) => {
                        self.mark_redis_unhealthy(&e.to_string()).await;
                    }
                }
            }
        }

        // Always delete from memory cache
        let memory_deleted = self.memory_cache.delete(key).await?;

        // Return true if deleted from either cache
        Ok(redis_deleted || memory_deleted)
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        // Try Redis first
        let redis_result = self.execute_with_fallback("exists", |redis| async move {
            redis.exists(key).await
        }).await;

        match redis_result {
            Ok(exists) => Ok(exists),
            Err(_) => {
                // Fallback to memory cache
                self.memory_cache.exists(key).await
            }
        }
    }

    async fn clear(&self) -> Result<(), CacheError> {
        // Try to clear Redis
        if let Some(redis) = &self.redis_cache {
            if self.check_redis_health().await {
                if let Err(e) = redis.clear().await {
                    self.mark_redis_unhealthy(&e.to_string()).await;
                }
            }
        }

        // Always clear memory cache
        self.memory_cache.clear().await
    }

    async fn stats(&self) -> Result<CacheStats, CacheError> {
        // Prioritize Redis stats if available
        if let Some(redis) = &self.redis_cache {
            if self.check_redis_health().await {
                match redis.stats().await {
                    Ok(mut stats) => {
                        // Augment with memory cache stats
                        if let Ok(memory_stats) = self.memory_cache.stats().await {
                            stats.active_entries += memory_stats.active_entries;
                            // Note: This is an approximation as we might have duplicates
                        }
                        return Ok(stats);
                    }
                    Err(e) => {
                        self.mark_redis_unhealthy(&e.to_string()).await;
                    }
                }
            }
        }

        // Fallback to memory cache stats
        self.memory_cache.stats().await
    }

    async fn delete_many(&self, keys: &[String]) -> Result<u64, CacheError> {
        let mut total_deleted = 0;

        // Try Redis first
        if let Some(redis) = &self.redis_cache {
            if self.check_redis_health().await {
                match redis.delete_many(keys).await {
                    Ok(deleted) => total_deleted += deleted,
                    Err(e) => {
                        self.mark_redis_unhealthy(&e.to_string()).await;
                    }
                }
            }
        }

        // Delete from memory cache
        let memory_deleted = self.memory_cache.delete_many(keys).await?;
        total_deleted += memory_deleted;

        Ok(total_deleted)
    }

    async fn increment(&self, key: &str, delta: i64, ttl_seconds: Option<i64>) -> Result<i64, CacheError> {
        // Try Redis first (preferred for atomic operations)
        let redis_result = self.execute_with_fallback("increment", |redis| async move {
            redis.increment(key, delta, ttl_seconds).await
        }).await;

        match redis_result {
            Ok(value) => {
                // Also update memory cache for consistency
                let _ = self.memory_cache.increment(key, delta, ttl_seconds).await;
                Ok(value)
            }
            Err(_) => {
                // Fallback to memory cache
                warn!("UnifiedCache: increment fallback to memory cache (less reliable for distributed systems)");
                self.memory_cache.increment(key, delta, ttl_seconds).await
            }
        }
    }

    async fn expire(&self, key: &str, ttl_seconds: i64) -> Result<bool, CacheError> {
        let mut redis_success = false;

        // Try Redis first
        if let Some(redis) = &self.redis_cache {
            if self.check_redis_health().await {
                match redis.expire(key, ttl_seconds).await {
                    Ok(success) => redis_success = success,
                    Err(e) => {
                        self.mark_redis_unhealthy(&e.to_string()).await;
                    }
                }
            }
        }

        // Update memory cache
        let memory_success = self.memory_cache.expire(key, ttl_seconds).await?;

        Ok(redis_success || memory_success)
    }

    async fn set_add(&self, key: &str, value: &str, ttl_seconds: Option<i64>) -> Result<bool, CacheError> {
        // Try Redis first
        let redis_result = self.execute_with_fallback("set_add", |redis| async move {
            redis.set_add(key, value, ttl_seconds).await
        }).await;

        match redis_result {
            Ok(added) => {
                // Also update memory cache
                let _ = self.memory_cache.set_add(key, value, ttl_seconds).await;
                Ok(added)
            }
            Err(_) => {
                // Fallback to memory cache
                self.memory_cache.set_add(key, value, ttl_seconds).await
            }
        }
    }

    async fn set_card(&self, key: &str) -> Result<u64, CacheError> {
        // Try Redis first
        let redis_result = self.execute_with_fallback("set_card", |redis| async move {
            redis.set_card(key).await
        }).await;

        match redis_result {
            Ok(count) => Ok(count),
            Err(_) => {
                // Fallback to memory cache
                self.memory_cache.set_card(key).await
            }
        }
    }

    async fn list_push(&self, key: &str, value: &str, ttl_seconds: Option<i64>) -> Result<u64, CacheError> {
        // Try Redis first
        let redis_result = self.execute_with_fallback("list_push", |redis| async move {
            redis.list_push(key, value, ttl_seconds).await
        }).await;

        match redis_result {
            Ok(length) => {
                // Also update memory cache
                let _ = self.memory_cache.list_push(key, value, ttl_seconds).await;
                Ok(length)
            }
            Err(_) => {
                // Fallback to memory cache
                self.memory_cache.list_push(key, value, ttl_seconds).await
            }
        }
    }

    async fn list_range(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>, CacheError> {
        // Try Redis first
        let redis_result = self.execute_with_fallback("list_range", |redis| async move {
            redis.list_range(key, start, stop).await
        }).await;

        match redis_result {
            Ok(values) => Ok(values),
            Err(_) => {
                // Fallback to memory cache
                self.memory_cache.list_range(key, start, stop).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::cache::CacheExt;

    #[tokio::test]
    async fn test_unified_cache_fallback() {
        // Test with invalid Redis URL to trigger fallback
        let cache = UnifiedCache::new(
            "redis://invalid:6379".to_string(),
            10,
            CacheConfig::default(),
        ).await;

        // Should work with memory cache fallback
        cache.set("test_key", &"test_value", Some(300)).await.unwrap();
        let value: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_unified_cache_memory_operations() {
        let cache = UnifiedCache::new(
            "redis://invalid:6379".to_string(),
            10,
            CacheConfig::default(),
        ).await;

        // Test basic operations
        assert!(cache.set_raw("key1", "value1", Some(300)).await.is_ok());
        assert_eq!(cache.get_raw("key1").await.unwrap(), Some("value1".to_string()));
        assert!(cache.exists("key1").await.unwrap());
        assert!(cache.delete("key1").await.unwrap());
        assert!(!cache.exists("key1").await.unwrap());
    }
}