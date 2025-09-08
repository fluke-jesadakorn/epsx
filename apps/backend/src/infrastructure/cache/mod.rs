// Cache infrastructure implementations

pub mod memory_cache;
pub mod redis_cache;
pub mod unified_cache;
pub mod permission_cache;

// Re-export cache types
pub use memory_cache::*;
pub use redis_cache::*;
pub use unified_cache::*;
pub use permission_cache::*;

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
        Box::new(MemoryCache::new())
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