// Cache infrastructure implementations

use std::sync::Arc;

pub mod memory_cache;
pub mod redis_cache;
pub mod unified_cache;
pub mod permission_cache;
pub mod plan_cache;
pub mod promotion_cache;
pub mod affiliate_cache;

// Re-export cache types
pub use memory_cache::*;
pub use redis_cache::*;
pub use unified_cache::*;
pub use permission_cache::*;
pub use plan_cache::*;
pub use promotion_cache::*;
pub use affiliate_cache::*;

// Legacy alias
pub use memory_cache::MemoryCache as InMemoryCache;

// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub default_ttl: u64,
    pub max_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: 3600, // 1 hour
            max_size: 1000,
        }
    }
}

// Cache trait
pub trait Cache: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: String, ttl: Option<u64>);
    fn delete(&self, key: &str);
    fn clear(&self);
}

// Cache factory
pub struct CacheFactory;

impl CacheFactory {
    pub async fn with_fallback() -> Box<dyn Cache> {
        // Try Redis first, fallback to memory cache
        if let Ok(redis_cache) = Self::try_redis().await {
            tracing::info!("✅ Using Redis cache with memory fallback");
            Box::new(redis_cache)
        } else {
            tracing::warn!("⚠️ Redis unavailable, using pure memory cache");
            Box::new(MemoryCache::new())
        }
    }
    
    pub async fn with_fallback_arc() -> Arc<dyn Cache> {
        // Try Redis first, fallback to memory cache
        if let Ok(redis_cache) = Self::try_redis().await {
            tracing::info!("✅ Using Redis cache with memory fallback");
            Arc::new(redis_cache)
        } else {
            tracing::warn!("⚠️ Redis unavailable, using pure memory cache");
            Arc::new(MemoryCache::new())
        }
    }
    
    async fn try_redis() -> Result<RedisCache, Box<dyn std::error::Error + Send + Sync>> {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let pool_size = std::env::var("REDIS_POOL_SIZE")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .unwrap_or(10);
            
        RedisCache::new(redis_url, pool_size, CacheConfig::default()).await
    }
    
    pub async fn with_redis_url(url: String) -> Box<dyn Cache> {
        if let Ok(redis_cache) = RedisCache::new(url, 10, CacheConfig::default()).await {
            Box::new(redis_cache)
        } else {
            tracing::warn!("⚠️ Custom Redis URL failed, using memory cache");
            Box::new(MemoryCache::new())
        }
    }
}

/// Extended cache operations for more complex scenarios
pub trait CacheExt: Cache {
    /// Get with typed deserialization
    fn get_typed<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.get(key).and_then(|json| serde_json::from_str(&json).ok())
    }
    
    /// Set with typed serialization
    fn set_typed<T: serde::Serialize>(&self, key: &str, value: &T, ttl: Option<u64>) {
        if let Ok(json) = serde_json::to_string(value) {
            self.set(key, json, ttl);
        }
    }
    
    /// Batch operations
    fn get_many(&self, keys: &[String]) -> Vec<Option<String>> {
        keys.iter().map(|key| self.get(key)).collect()
    }
}

// Blanket implementation for all Cache types
impl<T: Cache + ?Sized> CacheExt for T {}