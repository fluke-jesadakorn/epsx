// Casbin policy caching middleware for improved performance

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cache entry for Casbin policy decisions
#[derive(Debug, Clone)]
struct PolicyCacheEntry {
    result: bool,
    cached_at: Instant,
    ttl: Duration,
}

impl PolicyCacheEntry {
    fn new(result: bool, ttl: Duration) -> Self {
        Self {
            result,
            cached_at: Instant::now(),
            ttl,
        }
    }
    
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// Policy decision cache for Casbin enforcement
#[derive(Debug)]
pub struct CasbinPolicyCache {
    cache: Arc<RwLock<HashMap<String, PolicyCacheEntry>>>,
    default_ttl: Duration,
    max_entries: usize,
}

impl CasbinPolicyCache {
    pub fn new(default_ttl: Duration, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            max_entries,
        }
    }
    
    /// Create cache key from enforcement parameters
    fn create_cache_key(user: &str, resource: &str, action: &str) -> String {
        format!("{}:{}:{}", user, resource, action)
    }
    
    /// Get cached policy decision
    pub async fn get(&self, user: &str, resource: &str, action: &str) -> Option<bool> {
        let key = Self::create_cache_key(user, resource, action);
        let cache = self.cache.read().await;
        
        if let Some(entry) = cache.get(&key) {
            if !entry.is_expired() {
                tracing::debug!("Cache hit for policy: {}", key);
                return Some(entry.result);
            } else {
                tracing::debug!("Cache entry expired for policy: {}", key);
            }
        }
        
        None
    }
    
    /// Cache policy decision
    pub async fn set(&self, user: &str, resource: &str, action: &str, result: bool) {
        let key = Self::create_cache_key(user, resource, action);
        let entry = PolicyCacheEntry::new(result, self.default_ttl);
        
        let mut cache = self.cache.write().await;
        
        // Cleanup expired entries if cache is getting full
        if cache.len() >= self.max_entries {
            self.cleanup_expired_entries(&mut cache).await;
        }
        
        // If still full after cleanup, remove oldest entries
        if cache.len() >= self.max_entries {
            self.evict_oldest_entries(&mut cache, self.max_entries / 4).await;
        }
        
        cache.insert(key.clone(), entry);
        tracing::debug!("Cached policy decision: {} = {}", key, result);
    }
    
    /// Remove policy decision from cache
    pub async fn invalidate(&self, user: &str, resource: &str, action: &str) {
        let key = Self::create_cache_key(user, resource, action);
        let mut cache = self.cache.write().await;
        
        if cache.remove(&key).is_some() {
            tracing::debug!("Invalidated cache entry: {}", key);
        }
    }
    
    /// Invalidate all cache entries for a user
    pub async fn invalidate_user(&self, user: &str) {
        let prefix = format!("{}:", user);
        let mut cache = self.cache.write().await;
        let keys_to_remove: Vec<String> = cache
            .keys()
            .filter(|key| key.starts_with(&prefix))
            .cloned()
            .collect();
        
        for key in keys_to_remove {
            cache.remove(&key);
        }
        
        tracing::debug!("Invalidated all cache entries for user: {}", user);
    }
    
    /// Clear all cached entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        let count = cache.len();
        cache.clear();
        tracing::info!("Cleared {} cached policy decisions", count);
    }
    
    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let total_entries = cache.len();
        let expired_entries = cache
            .values()
            .filter(|entry| entry.is_expired())
            .count();
        
        CacheStats {
            total_entries,
            expired_entries,
            active_entries: total_entries - expired_entries,
            max_entries: self.max_entries,
            default_ttl: self.default_ttl,
        }
    }
    
    /// Cleanup expired entries from cache
    async fn cleanup_expired_entries(&self, cache: &mut HashMap<String, PolicyCacheEntry>) {
        let keys_to_remove: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in keys_to_remove {
            cache.remove(&key);
        }
        
        tracing::debug!("Cleaned up expired cache entries");
    }
    
    /// Evict oldest entries to make room
    async fn evict_oldest_entries(&self, cache: &mut HashMap<String, PolicyCacheEntry>, count: usize) {
        let mut entries: Vec<(String, Instant)> = cache
            .iter()
            .map(|(key, entry)| (key.clone(), entry.cached_at))
            .collect();
        
        // Sort by cache time (oldest first)
        entries.sort_by(|a, b| a.1.cmp(&b.1));
        
        // Remove oldest entries
        for (key, _) in entries.into_iter().take(count) {
            cache.remove(&key);
        }
        
        tracing::debug!("Evicted {} oldest cache entries", count);
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
    pub max_entries: usize,
    pub default_ttl: Duration,
}

impl Default for CasbinPolicyCache {
    fn default() -> Self {
        Self::new(
            Duration::from_secs(300), // 5 minutes default TTL
            10000, // Max 10k cache entries
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = CasbinPolicyCache::new(Duration::from_secs(1), 1000);
        
        // Test cache miss
        assert_eq!(cache.get("user1", "resource1", "read").await, None);
        
        // Test cache set and hit
        cache.set("user1", "resource1", "read", true).await;
        assert_eq!(cache.get("user1", "resource1", "read").await, Some(true));
        
        // Test cache expiration
        sleep(Duration::from_secs(2)).await;
        assert_eq!(cache.get("user1", "resource1", "read").await, None);
    }
    
    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = CasbinPolicyCache::new(Duration::from_secs(60), 1000);
        
        // Set cache entries
        cache.set("user1", "resource1", "read", true).await;
        cache.set("user1", "resource2", "write", false).await;
        cache.set("user2", "resource1", "read", true).await;
        
        // Test specific invalidation
        cache.invalidate("user1", "resource1", "read").await;
        assert_eq!(cache.get("user1", "resource1", "read").await, None);
        assert_eq!(cache.get("user1", "resource2", "write").await, Some(false));
        
        // Test user invalidation
        cache.invalidate_user("user1").await;
        assert_eq!(cache.get("user1", "resource2", "write").await, None);
        assert_eq!(cache.get("user2", "resource1", "read").await, Some(true));
    }
    
    #[tokio::test]
    async fn test_cache_stats() {
        let cache = CasbinPolicyCache::new(Duration::from_millis(100), 1000);
        
        // Add some entries
        cache.set("user1", "resource1", "read", true).await;
        cache.set("user2", "resource2", "write", false).await;
        
        let stats = cache.stats().await;
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.active_entries, 2);
        assert_eq!(stats.expired_entries, 0);
        
        // Wait for expiration
        sleep(Duration::from_millis(200)).await;
        
        let stats = cache.stats().await;
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.active_entries, 0);
        assert_eq!(stats.expired_entries, 2);
    }
}