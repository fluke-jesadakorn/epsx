// Cache abstraction layer supporting both in-memory and Redis backends
use chrono::{DateTime, Utc};

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use crate::config::env::get_env_var;

pub mod memory_cache;
pub mod redis_cache;
pub mod unified_cache;
pub mod notification_cache;

// Re-export implementations
pub use memory_cache::InMemoryCache;
pub use redis_cache::RedisCache;
pub use unified_cache::UnifiedCache;
pub use notification_cache::{NotificationCache, NotificationCacheImpl, NotificationCacheConfig, NotificationCacheKeys, NotificationCacheStats};

/// Cache backend configuration
#[derive(Debug, Clone)]
pub enum CacheBackend {
    InMemory,
    Redis { url: String, pool_size: u32 },
    Unified { redis_url: String, pool_size: u32 },
}

impl Default for CacheBackend {
    fn default() -> Self {
        Self::InMemory
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub backend: CacheBackend,
    pub default_ttl_seconds: i64,
    pub max_entries: Option<usize>,
    pub enable_compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            backend: CacheBackend::InMemory,
            default_ttl_seconds: 300, // 5 minutes
            max_entries: Some(10000),
            enable_compression: false,
        }
    }
}

/// Generic cache trait that can be implemented by different backends
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get value from cache as JSON string
    async fn get_raw(&self, key: &str) -> Result<Option<String>, CacheError>;

    /// Set value in cache with TTL as JSON string
    async fn set_raw(&self, key: &str, value: &str, ttl_seconds: Option<i64>) -> Result<(), CacheError>;

    /// Get typed value from cache (object-safe version)
    async fn get_typed(&self, key: &str, _type_name: &str) -> Result<Option<String>, CacheError> {
        self.get_raw(key).await
    }

    /// Set typed value in cache (object-safe version)
    async fn set_typed(&self, key: &str, value: &str, ttl_seconds: Option<i64>, _type_name: &str) -> Result<(), CacheError> {
        self.set_raw(key, value, ttl_seconds).await
    }

    /// Delete value from cache
    async fn delete(&self, key: &str) -> Result<bool, CacheError>;

    /// Check if key exists in cache
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;

    /// Clear all cache entries
    async fn clear(&self) -> Result<(), CacheError>;

    /// Get cache statistics
    async fn stats(&self) -> Result<CacheStats, CacheError>;

    /// Delete multiple keys at once
    async fn delete_many(&self, keys: &[String]) -> Result<u64, CacheError>;

    /// Increment a numeric value
    async fn increment(&self, key: &str, delta: i64, ttl_seconds: Option<i64>) -> Result<i64, CacheError>;

    /// Set expiration time for existing key
    async fn expire(&self, key: &str, ttl_seconds: i64) -> Result<bool, CacheError>;

    /// Add value to set
    async fn set_add(&self, key: &str, value: &str, ttl_seconds: Option<i64>) -> Result<bool, CacheError>;

    /// Get cardinality of set  
    async fn set_card(&self, key: &str) -> Result<u64, CacheError>;

    /// Push value to list (object-safe version)
    async fn list_push(&self, key: &str, value: &str, ttl_seconds: Option<i64>) -> Result<u64, CacheError>;

    /// Get range from list (object-safe version)
    async fn list_range(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>, CacheError>;
}

/// Cache extension trait providing convenience methods for typed operations
#[async_trait]
pub trait CacheExt: Cache {
    /// Get value from cache with deserialization
    async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
    {
        match self.get_raw(key).await? {
            Some(raw_value) => {
                let value: T = serde_json::from_str(&raw_value)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set value in cache with serialization
    async fn set<T>(&self, key: &str, value: &T, ttl_seconds: Option<i64>) -> Result<(), CacheError>
    where
        T: Serialize + Send + Sync,
    {
        let raw_value = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        self.set_raw(key, &raw_value, ttl_seconds).await
    }

    /// Push typed value to list with serialization
    async fn list_push_typed<T>(&self, key: &str, value: &T, ttl_seconds: Option<i64>) -> Result<u64, CacheError>
    where
        T: Serialize + Send + Sync,
    {
        let raw_value = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        self.list_push(key, &raw_value, ttl_seconds).await
    }

    /// Get typed range from list with deserialization
    async fn list_range_typed<T>(&self, key: &str, start: i64, stop: i64) -> Result<Vec<T>, CacheError>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
    {
        let raw_values = self.list_range(key, start, stop).await?;
        let mut typed_values = Vec::new();
        
        for raw_value in raw_values {
            let value: T = serde_json::from_str(&raw_value)
                .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
            typed_values.push(value);
        }
        
        Ok(typed_values)
    }
}

/// Blanket implementation for all Cache implementors
impl<T: ?Sized + Cache> CacheExt for T {}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: u64,
    pub expired_entries: u64,
    pub active_entries: u64,
    pub memory_usage_bytes: Option<u64>,
    pub hit_count: Option<u64>,
    pub miss_count: Option<u64>,
    pub hit_rate: Option<f64>,
}

/// Cache errors
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Cache full: cannot add more entries")]
    CacheFull,
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Cache factory for creating cache instances
pub struct CacheFactory;

impl CacheFactory {
    /// Create cache instance based on configuration
    pub async fn create(config: CacheConfig) -> Result<Arc<dyn Cache>, CacheError> {
        match config.backend.clone() {
            CacheBackend::InMemory => {
                Ok(Arc::new(InMemoryCache::new(config)))
            },
            CacheBackend::Redis { url, pool_size } => {
                Ok(Arc::new(RedisCache::new(url, pool_size, config).await?))
            },
            CacheBackend::Unified { redis_url, pool_size } => {
                Ok(Arc::new(UnifiedCache::new(redis_url, pool_size, config).await))
            },
        }
    }

    /// Create cache from environment variables with automatic fallback
    pub async fn from_env() -> Result<Arc<dyn Cache>, CacheError> {
        let backend = if let Ok(redis_url) = get_env_var("REDIS_URL") {
            let pool_size = get_env_var("REDIS_POOL_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10);
            
            // Use Unified cache for automatic Redis + in-memory fallback
            CacheBackend::Unified { redis_url, pool_size }
        } else {
            CacheBackend::InMemory
        };

        let ttl = get_env_var("CACHE_TTL_SECONDS")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .unwrap_or(300);

        let max_entries = get_env_var("CACHE_MAX_ENTRIES")
            .ok()
            .and_then(|s| s.parse().ok());

        let config = CacheConfig {
            backend,
            default_ttl_seconds: ttl,
            max_entries,
            enable_compression: get_env_var("CACHE_ENABLE_COMPRESSION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        };

        Self::create(config).await
    }

    /// Create cache with smart fallback - attempts Redis, falls back to in-memory
    pub async fn with_fallback() -> Arc<dyn Cache> {
        match Self::from_env().await {
            Ok(cache) => cache,
            Err(e) => {
                tracing::warn!("Failed to create cache from env, using in-memory fallback: {}", e);
                Arc::new(InMemoryCache::new(CacheConfig::default()))
            }
        }
    }
}

/// Cached entry wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEntry<T> {
    pub value: T,
    pub cached_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl<T> CachedEntry<T> {
    pub fn new(value: T, ttl_seconds: i64) -> Self {
        let now = Utc::now();
        Self {
            value,
            cached_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_seconds),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn ttl_remaining(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds().max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert!(matches!(config.backend, CacheBackend::InMemory));
        assert_eq!(config.default_ttl_seconds, 300);
        assert_eq!(config.max_entries, Some(10000));
        assert!(!config.enable_compression);
    }

    #[test]
    fn test_cached_entry() {
        let entry = CachedEntry::new("test_value".to_string(), 300);
        assert!(!entry.is_expired());
        assert!(entry.ttl_remaining() > 0);
        assert_eq!(entry.value, "test_value");
    }
}