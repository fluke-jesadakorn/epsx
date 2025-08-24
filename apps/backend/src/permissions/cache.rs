// Permission caching system with Redis backend

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use crate::dom::values::UserId;
use crate::infra::cache::{Cache, CacheFactory};
use super::core::{PermissionResult, UserPermissionProfile};
use super::errors::{CacheError, CacheResult};

/// Cached permission entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPermission {
    pub user_id: UserId,
    pub permission: String,
    pub resource: String,
    pub result: PermissionResult,
    pub cached_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub cache_version: u64,
    pub hit_count: u64,
}

/// Permission cache configuration
#[derive(Debug, Clone)]
pub struct PermissionCacheConfig {
    pub default_ttl: Duration,
    pub max_cache_size: u64,
    pub cache_key_prefix: String,
    pub enable_negative_caching: bool,
    pub negative_cache_ttl: Duration,
    pub stats_collection_enabled: bool,
}

/// Permission cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCacheStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_errors: u64,
    pub hit_rate: f64,
    pub average_lookup_time_ms: f64,
    pub cache_size: u64,
    pub expired_entries: u64,
    pub evicted_entries: u64,
    pub last_updated: DateTime<Utc>,
}

/// Cache invalidation types
#[derive(Debug, Clone)]
pub enum CacheInvalidation {
    /// Invalidate all entries for a user
    User(UserId),
    /// Invalidate specific permission for user
    Permission(UserId, String),
    /// Invalidate by pattern (e.g., "user:123:*")
    Pattern(String),
    /// Invalidate all entries
    All,
    /// Invalidate by tag
    Tag(String),
}

/// Main permission cache trait
#[async_trait]
pub trait PermissionCache: Send + Sync {
    /// Get cached permission result
    async fn get(&self, cache_key: &str) -> CacheResult<Option<PermissionResult>>;
    
    /// Set permission result in cache
    async fn set(&self, cache_key: &str, result: &PermissionResult, ttl: Duration) -> CacheResult<()>;
    
    /// Get cached user permission profile
    async fn get_user_profile(&self, user_id: &UserId) -> CacheResult<Option<UserPermissionProfile>>;
    
    /// Set user permission profile in cache
    async fn set_user_profile(&self, profile: &UserPermissionProfile, ttl: Duration) -> CacheResult<()>;
    
    /// Invalidate cache entries
    async fn invalidate(&self, invalidation: CacheInvalidation) -> CacheResult<u64>;
    
    /// Get cache statistics
    async fn stats(&self) -> CacheResult<PermissionCacheStats>;
    
    /// Clear all cache entries
    async fn clear(&self) -> CacheResult<u64>;
    
    /// Check if cache is healthy
    async fn health_check(&self) -> CacheResult<bool>;
    
    /// Batch operations
    async fn get_batch(&self, keys: &[String]) -> CacheResult<HashMap<String, Option<PermissionResult>>>;
    async fn set_batch(&self, entries: &[(String, PermissionResult, Duration)]) -> CacheResult<()>;
    
    /// Cleanup expired entries
    async fn cleanup_expired(&self) -> CacheResult<u64>;
}

/// Redis-backed permission cache implementation
pub struct RedisPermissionCache {
    cache: Arc<dyn Cache>,
    config: PermissionCacheConfig,
    stats: PermissionCacheStats,
}

/// In-memory permission cache for testing
pub struct InMemoryPermissionCache {
    data: std::sync::RwLock<HashMap<String, CachedPermission>>,
    config: PermissionCacheConfig,
    stats: std::sync::Mutex<PermissionCacheStats>,
}

// Implementations

impl CachedPermission {
    pub fn new(
        user_id: UserId,
        permission: String,
        resource: String,
        result: PermissionResult,
        ttl: Duration,
    ) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            permission,
            resource,
            result,
            cached_at: now,
            expires_at: now + chrono::Duration::from_std(ttl).unwrap_or_default(),
            cache_version: 1,
            hit_count: 0,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    pub fn increment_hit_count(&mut self) {
        self.hit_count += 1;
    }
    
    pub fn time_to_live(&self) -> Duration {
        let remaining = self.expires_at - Utc::now();
        Duration::from_secs(remaining.num_seconds().max(0) as u64)
    }
}

impl Default for PermissionCacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(super::constants::PERMISSION_CACHE_TTL_SECONDS),
            max_cache_size: super::constants::PERMISSION_CACHE_MAX_SIZE,
            cache_key_prefix: "perm:".to_string(),
            enable_negative_caching: true,
            negative_cache_ttl: Duration::from_secs(60), // 1 minute for negative results
            stats_collection_enabled: true,
        }
    }
}

impl Default for PermissionCacheStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            cache_errors: 0,
            hit_rate: 0.0,
            average_lookup_time_ms: 0.0,
            cache_size: 0,
            expired_entries: 0,
            evicted_entries: 0,
            last_updated: Utc::now(),
        }
    }
}

impl PermissionCacheStats {
    pub fn update_hit(&mut self, lookup_time_ms: f64) {
        self.total_requests += 1;
        self.cache_hits += 1;
        self.update_hit_rate();
        self.update_average_lookup_time(lookup_time_ms);
        self.last_updated = Utc::now();
    }
    
    pub fn update_miss(&mut self, lookup_time_ms: f64) {
        self.total_requests += 1;
        self.cache_misses += 1;
        self.update_hit_rate();
        self.update_average_lookup_time(lookup_time_ms);
        self.last_updated = Utc::now();
    }
    
    pub fn update_error(&mut self) {
        self.cache_errors += 1;
        self.last_updated = Utc::now();
    }
    
    fn update_hit_rate(&mut self) {
        if self.total_requests > 0 {
            self.hit_rate = self.cache_hits as f64 / self.total_requests as f64;
        }
    }
    
    fn update_average_lookup_time(&mut self, lookup_time_ms: f64) {
        // Exponential moving average
        let alpha = 0.1;
        self.average_lookup_time_ms = 
            alpha * lookup_time_ms + (1.0 - alpha) * self.average_lookup_time_ms;
    }
}

impl RedisPermissionCache {
    pub async fn new(config: PermissionCacheConfig) -> CacheResult<Self> {
        let cache = CacheFactory::from_env().await
            .map_err(|e| CacheError::ConnectionFailed { 
                reason: e.to_string() 
            })?;
        
        Ok(Self {
            cache,
            config,
            stats: PermissionCacheStats::default(),
        })
    }
    
    fn make_cache_key(&self, key: &str) -> String {
        format!("{}{}", self.config.cache_key_prefix, key)
    }
    
    fn make_user_profile_key(&self, user_id: &UserId) -> String {
        format!("{}profile:{}", self.config.cache_key_prefix, user_id.value())
    }
    
    async fn serialize_result(&self, result: &PermissionResult) -> CacheResult<String> {
        serde_json::to_string(result)
            .map_err(|e| CacheError::SerializationFailed { 
                reason: e.to_string() 
            })
    }
    
    async fn deserialize_result(&self, data: &str) -> CacheResult<PermissionResult> {
        serde_json::from_str(data)
            .map_err(|e| CacheError::DeserializationFailed { 
                reason: e.to_string() 
            })
    }
}

#[async_trait]
impl PermissionCache for RedisPermissionCache {
    async fn get(&self, cache_key: &str) -> CacheResult<Option<PermissionResult>> {
        let start_time = std::time::Instant::now();
        let full_key = self.make_cache_key(cache_key);
        
        match self.cache.get_raw(&full_key).await {
            Ok(Some(data)) => {
                let result = self.deserialize_result(&data).await?;
                let _lookup_time = start_time.elapsed().as_millis() as f64;
                
                // Note: This is simplified - in a real implementation, you'd need
                // to handle stats update in a thread-safe manner
                
                Ok(Some(result))
            }
            Ok(None) => {
                let _lookup_time = start_time.elapsed().as_millis() as f64;
                Ok(None)
            }
            Err(e) => Err(CacheError::BackendError { 
                reason: format!("Redis error: {}", e) 
            })
        }
    }
    
    async fn set(&self, cache_key: &str, result: &PermissionResult, ttl: Duration) -> CacheResult<()> {
        let full_key = self.make_cache_key(cache_key);
        let serialized = self.serialize_result(result).await?;
        
        self.cache.set_raw(&full_key, &serialized, Some(ttl.as_secs() as i64)).await
            .map_err(|e| CacheError::BackendError { 
                reason: e.to_string() 
            })
    }
    
    async fn get_user_profile(&self, user_id: &UserId) -> CacheResult<Option<UserPermissionProfile>> {
        let key = self.make_user_profile_key(user_id);
        
        match self.cache.get_raw(&key).await {
            Ok(Some(data)) => {
                let profile: UserPermissionProfile = serde_json::from_str(&data)
                    .map_err(|e| CacheError::DeserializationFailed { 
                        reason: e.to_string() 
                    })?;
                Ok(Some(profile))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(CacheError::BackendError { 
                reason: e.to_string() 
            })
        }
    }
    
    async fn set_user_profile(&self, profile: &UserPermissionProfile, ttl: Duration) -> CacheResult<()> {
        let key = self.make_user_profile_key(&profile.user_id);
        let serialized = serde_json::to_string(profile)
            .map_err(|e| CacheError::SerializationFailed { 
                reason: e.to_string() 
            })?;
        
        self.cache.set_raw(&key, &serialized, Some(ttl.as_secs() as i64)).await
            .map_err(|e| CacheError::BackendError { 
                reason: e.to_string() 
            })
    }
    
    async fn invalidate(&self, invalidation: CacheInvalidation) -> CacheResult<u64> {
        match invalidation {
            CacheInvalidation::User(user_id) => {
                let _pattern = format!("{}*{}*", self.config.cache_key_prefix, user_id.value());
                // TODO: Implement pattern deletion - for now just return 0
                // self.cache.delete_pattern(&pattern).await
                Ok(0)
            }
            CacheInvalidation::Permission(user_id, permission) => {
                let key = super::utils::generate_permission_key(&user_id.value().to_string(), &permission, "*");
                let full_key = self.make_cache_key(&key);
                match self.cache.delete(&full_key).await {
                    Ok(_) => Ok(1),
                    Err(e) => Err(CacheError::InvalidationFailed { 
                        reason: e.to_string() 
                    })
                }
            }
            CacheInvalidation::Pattern(_pattern) => {
                // TODO: Implement pattern deletion - for now just return 0
                Ok(0)
            }
            CacheInvalidation::All => {
                // TODO: Implement pattern deletion - for now use clear()
                self.cache.clear().await
                    .map_err(|e| CacheError::InvalidationFailed { 
                        reason: e.to_string() 
                    })
                    .map(|_| 0)
            }
            CacheInvalidation::Tag(_tag) => {
                // Tag-based invalidation would require additional Redis structures
                // For now, return 0 as not implemented
                Ok(0)
            }
        }
    }
    
    async fn stats(&self) -> CacheResult<PermissionCacheStats> {
        // In a real implementation, you'd collect these stats from Redis
        // or maintain them in a separate store
        Ok(self.stats.clone())
    }
    
    async fn clear(&self) -> CacheResult<u64> {
        self.invalidate(CacheInvalidation::All).await
    }
    
    async fn health_check(&self) -> CacheResult<bool> {
        // Simple health check - try to set and get a test value
        let test_key = format!("{}health_check", self.config.cache_key_prefix);
        let test_value = "ok";
        
        match self.cache.set_raw(&test_key, test_value, Some(60)).await {
            Ok(_) => {
                match self.cache.get_raw(&test_key).await {
                    Ok(Some(value)) if value == test_value => {
                        let _ = self.cache.delete(&test_key).await;
                        Ok(true)
                    }
                    _ => Ok(false)
                }
            }
            Err(_) => Ok(false)
        }
    }
    
    async fn get_batch(&self, keys: &[String]) -> CacheResult<HashMap<String, Option<PermissionResult>>> {
        let mut results = HashMap::new();
        
        // In a real Redis implementation, you'd use MGET for efficiency
        for key in keys {
            let result = self.get(key).await?;
            results.insert(key.clone(), result);
        }
        
        Ok(results)
    }
    
    async fn set_batch(&self, entries: &[(String, PermissionResult, Duration)]) -> CacheResult<()> {
        // In a real Redis implementation, you'd use pipeline or MSET for efficiency
        for (key, result, ttl) in entries {
            self.set(key, result, *ttl).await?;
        }
        
        Ok(())
    }
    
    async fn cleanup_expired(&self) -> CacheResult<u64> {
        // Redis handles TTL expiration automatically, so this is a no-op
        // In other cache backends, you might need to manually clean up
        Ok(0)
    }
}

impl InMemoryPermissionCache {
    pub fn new(config: PermissionCacheConfig) -> Self {
        Self {
            data: std::sync::RwLock::new(HashMap::new()),
            config,
            stats: std::sync::Mutex::new(PermissionCacheStats::default()),
        }
    }
    
    fn cleanup_expired_internal(&self) -> u64 {
        let mut data = self.data.write().unwrap();
        let initial_size = data.len();
        
        data.retain(|_, cached| !cached.is_expired());
        
        let cleaned = initial_size - data.len();
        
        if let Ok(mut stats) = self.stats.lock() {
            stats.expired_entries += cleaned as u64;
            stats.cache_size = data.len() as u64;
        }
        
        cleaned as u64
    }
}

#[async_trait]
impl PermissionCache for InMemoryPermissionCache {
    async fn get(&self, cache_key: &str) -> CacheResult<Option<PermissionResult>> {
        let start_time = std::time::Instant::now();
        
        let data = self.data.read().unwrap();
        let result = if let Some(cached) = data.get(cache_key) {
            if cached.is_expired() {
                None
            } else {
                Some(cached.result.clone())
            }
        } else {
            None
        };
        
        let lookup_time = start_time.elapsed().as_millis() as f64;
        
        if let Ok(mut stats) = self.stats.lock() {
            if result.is_some() {
                stats.update_hit(lookup_time);
            } else {
                stats.update_miss(lookup_time);
            }
        }
        
        Ok(result)
    }
    
    async fn set(&self, cache_key: &str, result: &PermissionResult, ttl: Duration) -> CacheResult<()> {
        let user_id = UserId::new("unknown".to_string()); // Extract from key in real implementation
        let cached = CachedPermission::new(
            user_id,
            "unknown".to_string(),
            "unknown".to_string(),
            result.clone(),
            ttl,
        );
        
        let mut data = self.data.write().unwrap();
        
        // Check cache size limit
        if data.len() >= self.config.max_cache_size as usize {
            // Simple LRU eviction - remove oldest entry
            if let Some((oldest_key, _)) = data.iter()
                .min_by_key(|(_, cached)| cached.cached_at)
                .map(|(k, v)| (k.clone(), v.clone()))
            {
                data.remove(&oldest_key);
                
                if let Ok(mut stats) = self.stats.lock() {
                    stats.evicted_entries += 1;
                }
            }
        }
        
        data.insert(cache_key.to_string(), cached);
        
        if let Ok(mut stats) = self.stats.lock() {
            stats.cache_size = data.len() as u64;
        }
        
        Ok(())
    }
    
    async fn get_user_profile(&self, user_id: &UserId) -> CacheResult<Option<UserPermissionProfile>> {
        let _key = format!("profile:{}", user_id.value());
        
        // This is a simplified implementation - would need proper serialization
        // in a real cache
        Ok(None)
    }
    
    async fn set_user_profile(&self, _profile: &UserPermissionProfile, _ttl: Duration) -> CacheResult<()> {
        // Simplified implementation
        Ok(())
    }
    
    async fn invalidate(&self, invalidation: CacheInvalidation) -> CacheResult<u64> {
        let mut data = self.data.write().unwrap();
        let initial_size = data.len();
        
        match invalidation {
            CacheInvalidation::User(user_id) => {
                data.retain(|_, cached| cached.user_id != user_id);
            }
            CacheInvalidation::Permission(user_id, permission) => {
                data.retain(|_, cached| {
                    !(cached.user_id == user_id && cached.permission == permission)
                });
            }
            CacheInvalidation::Pattern(pattern) => {
                // Simple pattern matching (would be more sophisticated in real implementation)
                data.retain(|key, _| !key.contains(&pattern.replace("*", "")));
            }
            CacheInvalidation::All => {
                data.clear();
            }
            CacheInvalidation::Tag(_) => {
                // Not implemented for in-memory cache
            }
        }
        
        let removed = initial_size - data.len();
        
        if let Ok(mut stats) = self.stats.lock() {
            stats.cache_size = data.len() as u64;
        }
        
        Ok(removed as u64)
    }
    
    async fn stats(&self) -> CacheResult<PermissionCacheStats> {
        let stats = self.stats.lock().unwrap().clone();
        Ok(stats)
    }
    
    async fn clear(&self) -> CacheResult<u64> {
        self.invalidate(CacheInvalidation::All).await
    }
    
    async fn health_check(&self) -> CacheResult<bool> {
        // Always healthy for in-memory cache
        Ok(true)
    }
    
    async fn get_batch(&self, keys: &[String]) -> CacheResult<HashMap<String, Option<PermissionResult>>> {
        let mut results = HashMap::new();
        
        for key in keys {
            let result = self.get(key).await?;
            results.insert(key.clone(), result);
        }
        
        Ok(results)
    }
    
    async fn set_batch(&self, entries: &[(String, PermissionResult, Duration)]) -> CacheResult<()> {
        for (key, result, ttl) in entries {
            self.set(key, result, *ttl).await?;
        }
        
        Ok(())
    }
    
    async fn cleanup_expired(&self) -> CacheResult<u64> {
        Ok(self.cleanup_expired_internal())
    }
}

// Cache factory for creating cache instances
pub struct PermissionCacheFactory;

impl PermissionCacheFactory {
    pub async fn create_redis_cache(config: PermissionCacheConfig) -> CacheResult<Box<dyn PermissionCache>> {
        let cache = RedisPermissionCache::new(config).await?;
        Ok(Box::new(cache))
    }
    
    pub fn create_memory_cache(config: PermissionCacheConfig) -> Box<dyn PermissionCache> {
        Box::new(InMemoryPermissionCache::new(config))
    }
    
    pub async fn create_from_config(use_redis: bool, config: PermissionCacheConfig) -> CacheResult<Box<dyn PermissionCache>> {
        if use_redis {
            Self::create_redis_cache(config).await
        } else {
            Ok(Self::create_memory_cache(config))
        }
    }
}

// Display implementations
impl fmt::Display for CacheInvalidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheInvalidation::User(user_id) => write!(f, "User({})", user_id.value()),
            CacheInvalidation::Permission(user_id, permission) => {
                write!(f, "Permission({}:{})", user_id.value(), permission)
            }
            CacheInvalidation::Pattern(pattern) => write!(f, "Pattern({})", pattern),
            CacheInvalidation::All => write!(f, "All"),
            CacheInvalidation::Tag(tag) => write!(f, "Tag({})", tag),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::entities::iam::PackageTier;
    
    #[test]
    fn test_cached_permission_expiration() {
        let user_id = UserId::new("test_user".to_string());
        let result = PermissionResult::Granted(super::super::core::PermissionGrant {
            request_id: uuid::Uuid::new_v4(),
            user_id: user_id.clone(),
            granted_permissions: vec![],
            granted_at: Utc::now(),
            granted_by: UserId::new("system".to_string()),
            expires_at: None,
            conditions: None,
        });
        
        let cached = CachedPermission::new(
            user_id,
            "test:permission".to_string(),
            "resource".to_string(),
            result,
            Duration::from_secs(1),
        );
        
        assert!(!cached.is_expired());
        
        // Test with past expiration
        let expired_cached = CachedPermission {
            expires_at: Utc::now() - chrono::Duration::seconds(1),
            ..cached
        };
        
        assert!(expired_cached.is_expired());
    }
    
    #[test]
    fn test_cache_config_defaults() {
        let config = PermissionCacheConfig::default();
        
        assert_eq!(config.default_ttl, Duration::from_secs(300));
        assert_eq!(config.cache_key_prefix, "perm:");
        assert!(config.enable_negative_caching);
        assert!(config.stats_collection_enabled);
    }
    
    #[test]
    fn test_cache_stats_updates() {
        let mut stats = PermissionCacheStats::default();
        
        stats.update_hit(10.0);
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.hit_rate, 1.0);
        
        stats.update_miss(15.0);
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }
    
    #[tokio::test]
    async fn test_in_memory_cache_basic_operations() {
        let config = PermissionCacheConfig::default();
        let cache = InMemoryPermissionCache::new(config);
        
        let user_id = UserId::new("test_user".to_string());
        let result = PermissionResult::Granted(super::super::core::PermissionGrant {
            request_id: uuid::Uuid::new_v4(),
            user_id: user_id.clone(),
            granted_permissions: vec![],
            granted_at: Utc::now(),
            granted_by: UserId::new("system".to_string()),
            expires_at: None,
            conditions: None,
        });
        
        let key = "test_key";
        
        // Test cache miss
        let get_result = cache.get(key).await.unwrap();
        assert!(get_result.is_none());
        
        // Test cache set
        cache.set(key, &result, Duration::from_secs(300)).await.unwrap();
        
        // Test cache hit
        let get_result = cache.get(key).await.unwrap();
        assert!(get_result.is_some());
        
        // Test cache stats
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }
    
    #[tokio::test]
    async fn test_cache_invalidation() {
        let config = PermissionCacheConfig::default();
        let cache = InMemoryPermissionCache::new(config);
        
        let user_id = UserId::new("test_user".to_string());
        let result = PermissionResult::Granted(super::super::core::PermissionGrant {
            request_id: uuid::Uuid::new_v4(),
            user_id: user_id.clone(),
            granted_permissions: vec![],
            granted_at: Utc::now(),
            granted_by: UserId::new("system".to_string()),
            expires_at: None,
            conditions: None,
        });
        
        // Set some test data
        cache.set("key1", &result, Duration::from_secs(300)).await.unwrap();
        cache.set("key2", &result, Duration::from_secs(300)).await.unwrap();
        
        // Test pattern invalidation
        let invalidated = cache.invalidate(CacheInvalidation::Pattern("key".to_string())).await.unwrap();
        assert_eq!(invalidated, 2);
        
        // Verify entries are gone
        let get_result = cache.get("key1").await.unwrap();
        assert!(get_result.is_none());
    }
    
    #[tokio::test]
    async fn test_cache_health_check() {
        let config = PermissionCacheConfig::default();
        let cache = InMemoryPermissionCache::new(config);
        
        let health = cache.health_check().await.unwrap();
        assert!(health);
    }
    
    #[test]
    fn test_cache_invalidation_display() {
        let user_id = UserId::new("test_user".to_string());
        
        let invalidation = CacheInvalidation::User(user_id);
        assert_eq!(format!("{}", invalidation), "User(test_user)");
        
        let invalidation = CacheInvalidation::Pattern("test:*".to_string());
        assert_eq!(format!("{}", invalidation), "Pattern(test:*)");
        
        let invalidation = CacheInvalidation::All;
        assert_eq!(format!("{}", invalidation), "All");
    }
}