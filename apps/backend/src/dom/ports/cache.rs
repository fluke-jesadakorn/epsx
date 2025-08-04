// Domain cache port - abstracts caching infrastructure
use std::time::Duration;
use async_trait::async_trait;

/// Domain-level cache abstraction
#[async_trait]
pub trait DomainCache: Send + Sync {
    /// Get a raw value from cache as string
    async fn get_raw(&self, key: &str) -> Result<Option<String>, DomainCacheError>;

    /// Set a raw value in cache with optional TTL
    async fn set_raw(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<(), DomainCacheError>;

    /// Delete a value from cache
    async fn delete(&self, key: &str) -> Result<(), DomainCacheError>;

    /// Clear cache entries matching a pattern
    async fn clear_pattern(&self, pattern: &str) -> Result<u64, DomainCacheError>;

    /// Check if a key exists in cache
    async fn exists(&self, key: &str) -> Result<bool, DomainCacheError>;

    /// Get cache statistics for monitoring
    async fn stats(&self) -> Result<DomainCacheStats, DomainCacheError>;
}

/// Domain cache statistics
#[derive(Debug, Clone)]
pub struct DomainCacheStats {
    pub total_entries: u64,
    pub expired_entries: u64,
    pub active_entries: u64,
    pub memory_usage_bytes: Option<u64>,
    pub hit_count: Option<u64>,
    pub miss_count: Option<u64>,
    pub hit_rate: Option<f64>,
}

/// Domain cache errors
#[derive(Debug, thiserror::Error)]
pub enum DomainCacheError {
    #[error("Cache operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Cache service unavailable")]
    ServiceUnavailable,
    
    #[error("Invalid cache key: {0}")]
    InvalidKey(String),
    
    #[error("Cache timeout")]
    Timeout,
}

/// Extension trait for typed operations on DomainCache
#[async_trait]
pub trait DomainCacheExt {
    /// Get a typed value from cache
    async fn get<T>(&self, key: &str) -> Result<Option<T>, DomainCacheError>
    where
        T: serde::de::DeserializeOwned + Send;

    /// Set a typed value in cache with optional TTL
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), DomainCacheError>
    where
        T: serde::Serialize + Send + Sync;
}

/// Blanket implementation for all DomainCache implementors
#[async_trait]
impl<C: DomainCache + ?Sized> DomainCacheExt for C {
    async fn get<T>(&self, key: &str) -> Result<Option<T>, DomainCacheError>
    where
        T: serde::de::DeserializeOwned + Send,
    {
        match self.get_raw(key).await? {
            Some(raw_value) => {
                let value: T = serde_json::from_str(&raw_value)
                    .map_err(|e| DomainCacheError::SerializationError(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), DomainCacheError>
    where
        T: serde::Serialize + Send + Sync,
    {
        let raw_value = serde_json::to_string(value)
            .map_err(|e| DomainCacheError::SerializationError(e.to_string()))?;
        self.set_raw(key, &raw_value, ttl).await
    }
}