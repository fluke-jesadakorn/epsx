// In-memory cache implementation

use super::{Cache, CacheExt, CacheConfig, CacheStats, CacheError, CachedEntry};
use async_trait::async_trait;
use chrono::Utc;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory cache implementation
pub struct InMemoryCache {
    storage: Arc<RwLock<HashMap<String, String>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStatsInternal>>,
}

#[derive(Debug, Default)]
struct CacheStatsInternal {
    hit_count: u64,
    miss_count: u64,
    set_count: u64,
    delete_count: u64,
}

impl InMemoryCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStatsInternal::default())),
        }
    }

    async fn serialize_value<T: Serialize>(&self, value: &T) -> Result<String, CacheError> {
        serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))
    }


    async fn cleanup_expired(&self) {
        let mut storage = self.storage.write().await;
        let _now = Utc::now();
        
        storage.retain(|_, value| {
            if let Ok(entry) = serde_json::from_str::<CachedEntry<serde_json::Value>>(value) {
                !entry.is_expired()
            } else {
                // Keep entries that can't be parsed as they might be direct values
                true
            }
        });
    }

    async fn check_capacity(&self) -> Result<(), CacheError> {
        if let Some(max_entries) = self.config.max_entries {
            let storage = self.storage.read().await;
            if storage.len() >= max_entries {
                drop(storage);
                self.cleanup_expired().await;
                
                let storage = self.storage.read().await;
                if storage.len() >= max_entries {
                    return Err(CacheError::CacheFull);
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Cache for InMemoryCache {
    async fn get_raw(&self, key: &str) -> Result<Option<String>, CacheError> {
        let storage = self.storage.read().await;
        let mut stats = self.stats.write().await;

        if let Some(data) = storage.get(key) {
            // Check if it's an expired cached entry
            if let Ok(entry) = serde_json::from_str::<CachedEntry<serde_json::Value>>(data) {
                if !entry.is_expired() {
                    stats.hit_count += 1;
                    Ok(Some(data.clone()))
                } else {
                    stats.miss_count += 1;
                    Ok(None)
                }
            } else {
                // Direct value
                stats.hit_count += 1;
                Ok(Some(data.clone()))
            }
        } else {
            stats.miss_count += 1;
            Ok(None)
        }
    }

    async fn set_raw(&self, key: &str, value: &str, _ttl_seconds: Option<i64>) -> Result<(), CacheError> {
        self.check_capacity().await?;

        let mut storage = self.storage.write().await;
        let mut stats = self.stats.write().await;
        
        storage.insert(key.to_string(), value.to_string());
        stats.set_count += 1;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, CacheError> {
        let mut storage = self.storage.write().await;
        let mut stats = self.stats.write().await;
        
        let existed = storage.remove(key).is_some();
        if existed {
            stats.delete_count += 1;
        }
        
        Ok(existed)
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let storage = self.storage.read().await;
        
        if let Some(data) = storage.get(key) {
            // Check if it's an expired cached entry
            if let Ok(entry) = serde_json::from_str::<CachedEntry<serde_json::Value>>(data) {
                Ok(!entry.is_expired())
            } else {
                // Direct value, assume it exists
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    async fn clear(&self) -> Result<(), CacheError> {
        let mut storage = self.storage.write().await;
        storage.clear();
        Ok(())
    }

    async fn stats(&self) -> Result<CacheStats, CacheError> {
        self.cleanup_expired().await;
        
        let storage = self.storage.read().await;
        let stats = self.stats.read().await;
        
        let total_entries = storage.len() as u64;
        let hit_rate = if stats.hit_count + stats.miss_count > 0 {
            Some(stats.hit_count as f64 / (stats.hit_count + stats.miss_count) as f64)
        } else {
            None
        };

        // Estimate memory usage
        let memory_usage = storage.iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>() as u64;

        Ok(CacheStats {
            total_entries,
            expired_entries: 0, // Cleaned up
            active_entries: total_entries,
            memory_usage_bytes: Some(memory_usage),
            hit_count: Some(stats.hit_count),
            miss_count: Some(stats.miss_count),
            hit_rate,
        })
    }


    async fn delete_many(&self, keys: &[String]) -> Result<u64, CacheError> {
        let mut deleted_count = 0;
        
        for key in keys {
            if self.delete(key).await? {
                deleted_count += 1;
            }
        }
        
        Ok(deleted_count)
    }

    async fn increment(&self, key: &str, delta: i64, ttl_seconds: Option<i64>) -> Result<i64, CacheError> {
        let storage = self.storage.read().await;
        
        let current_value = if let Some(data) = storage.get(key) {
            // Try to get as cached entry of i64
            if let Ok(entry) = serde_json::from_str::<CachedEntry<i64>>(data) {
                if !entry.is_expired() {
                    entry.value
                } else {
                    0
                }
            } else {
                // Try direct deserialization
                serde_json::from_str::<i64>(data).unwrap_or(0)
            }
        } else {
            0
        };
        
        drop(storage);
        
        let new_value = current_value + delta;
        self.set(key, &new_value, ttl_seconds).await?;
        
        Ok(new_value)
    }

    async fn expire(&self, key: &str, ttl_seconds: i64) -> Result<bool, CacheError> {
        let storage = self.storage.read().await;
        
        if let Some(data) = storage.get(key) {
            // Try to parse as cached entry and update expiration
            if let Ok(mut entry) = serde_json::from_str::<CachedEntry<serde_json::Value>>(data) {
                drop(storage);
                
                entry.expires_at = Utc::now() + chrono::Duration::seconds(ttl_seconds);
                let serialized = self.serialize_value(&entry).await?;
                
                let mut storage = self.storage.write().await;
                storage.insert(key.to_string(), serialized);
                Ok(true)
            } else {
                // Can't update expiration for direct values
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_basic_operations() {
        let config = CacheConfig::default();
        let cache = InMemoryCache::new(config);

        // Test set and get
        cache.set("test_key", &"test_value", Some(300)).await.unwrap();
        let value: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Test exists
        assert!(cache.exists("test_key").await.unwrap());
        assert!(!cache.exists("nonexistent").await.unwrap());

        // Test delete
        assert!(cache.delete("test_key").await.unwrap());
        assert!(!cache.exists("test_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_expiration() {
        let config = CacheConfig::default();
        let cache = InMemoryCache::new(config);

        // Set with very short TTL
        cache.set("expire_test", &"value", Some(1)).await.unwrap();
        assert!(cache.exists("expire_test").await.unwrap());

        // Wait for expiration
        sleep(Duration::from_secs(2)).await;
        assert!(!cache.exists("expire_test").await.unwrap());
    }

    #[tokio::test]
    async fn test_increment() {
        let config = CacheConfig::default();
        let cache = InMemoryCache::new(config);

        // Test increment on new key
        let result = cache.increment("counter", 5, Some(300)).await.unwrap();
        assert_eq!(result, 5);

        // Test increment on existing key
        let result = cache.increment("counter", 3, Some(300)).await.unwrap();
        assert_eq!(result, 8);
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let config = CacheConfig::default();
        let cache = InMemoryCache::new(config);

        // Test set_many
        let mut entries = HashMap::new();
        entries.insert("key1".to_string(), "value1".to_string());
        entries.insert("key2".to_string(), "value2".to_string());
        
        cache.set_many(entries, Some(300)).await.unwrap();

        // Test get_many
        let keys = vec!["key1".to_string(), "key2".to_string(), "key3".to_string()];
        let results: HashMap<String, String> = cache.get_many(&keys).await.unwrap();
        
        assert_eq!(results.len(), 2);
        assert_eq!(results.get("key1"), Some(&"value1".to_string()));
        assert_eq!(results.get("key2"), Some(&"value2".to_string()));

        // Test delete_many
        let deleted = cache.delete_many(&keys).await.unwrap();
        assert_eq!(deleted, 2);
    }
}