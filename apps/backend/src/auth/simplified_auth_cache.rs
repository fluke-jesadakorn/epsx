// Simplified Authentication Cache Service
// Core caching functionality for Web3-first authentication system

use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

// use super::unified_permission_service::{UnifiedPermission, AccessLevel}; // Removed - service no longer exists

// Temporary types for compilation
#[derive(Debug, Clone)]
pub struct UnifiedPermission;

#[derive(Debug, Clone)]
pub enum AccessLevel {
    Read,
    Write,
    Admin,
}

/// Simplified cache entry for user permissions
#[derive(Debug, Clone)]
pub struct PermissionCacheEntry {
    pub permissions: Vec<UnifiedPermission>,
    pub access_level: AccessLevel,
    pub cached_at: Instant,
    pub expires_at: Instant,
}

/// SIWE challenge cache entry
#[derive(Debug, Clone)]
pub struct ChallengeCacheEntry {
    pub challenge: String,
    pub wallet_address: String,
    pub nonce: String,
    pub expires_at: Instant,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct SimplifiedCacheConfig {
    pub permission_cache_ttl: Duration,
    pub challenge_cache_ttl: Duration,
    pub max_cache_size: usize,
    pub cleanup_interval: Duration,
}

impl Default for SimplifiedCacheConfig {
    fn default() -> Self {
        Self {
            permission_cache_ttl: Duration::from_secs(1800), // 30 minutes
            challenge_cache_ttl: Duration::from_secs(300),   // 5 minutes
            max_cache_size: 10000,
            cleanup_interval: Duration::from_secs(600),      // 10 minutes
        }
    }
}

/// Cache statistics
#[derive(Debug, Default, Clone)]
pub struct SimplifiedCacheStats {
    pub permission_hits: u64,
    pub permission_misses: u64,
    pub challenge_hits: u64,
    pub challenge_misses: u64,
    pub total_evictions: u64,
}

impl SimplifiedCacheStats {
    pub fn permission_hit_rate(&self) -> f64 {
        let total = self.permission_hits + self.permission_misses;
        if total > 0 {
            self.permission_hits as f64 / total as f64
        } else {
            0.0
        }
    }
    
    pub fn challenge_hit_rate(&self) -> f64 {
        let total = self.challenge_hits + self.challenge_misses;
        if total > 0 {
            self.challenge_hits as f64 / total as f64
        } else {
            0.0
        }
    }
    
    pub fn overall_hit_rate(&self) -> f64 {
        let total_hits = self.permission_hits + self.challenge_hits;
        let total_requests = total_hits + self.permission_misses + self.challenge_misses;
        
        if total_requests > 0 {
            total_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }
}

/// Simplified authentication cache service
pub struct SimplifiedAuthCache {
    config: SimplifiedCacheConfig,
    permission_cache: DashMap<String, PermissionCacheEntry>,
    challenge_cache: DashMap<String, ChallengeCacheEntry>,
    stats: Arc<RwLock<SimplifiedCacheStats>>,
}

impl SimplifiedAuthCache {
    pub fn new(config: SimplifiedCacheConfig) -> Self {
        let cache = Self {
            config,
            permission_cache: DashMap::new(),
            challenge_cache: DashMap::new(),
            stats: Arc::new(RwLock::new(SimplifiedCacheStats::default())),
        };
        
        // Start background cleanup
        cache.start_cleanup_task();
        cache
    }
    
    /// Get cached permissions
    pub async fn get_permissions(
        &self,
        wallet_address: &str,
        additional_context: Option<&str>,
    ) -> Option<(Vec<UnifiedPermission>, AccessLevel)> {
        let cache_key = self.make_permission_key(wallet_address, additional_context);
        
        if let Some(entry) = self.permission_cache.get(&cache_key) {
            if Instant::now() < entry.expires_at {
                let mut stats = self.stats.write().await;
                stats.permission_hits += 1;
                
                debug!(
                    user_id = %wallet_address,
                    wallet_address = ?wallet_address,
                    "Permission cache hit"
                );
                
                return Some((entry.permissions.clone(), entry.access_level.clone()));
            } else {
                self.permission_cache.remove(&cache_key);
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.permission_misses += 1;
        None
    }
    
    /// Cache user permissions
    pub async fn cache_permissions(
        &self,
        wallet_address: &str,
        additional_context: Option<&str>,
        permissions: Vec<UnifiedPermission>,
        access_level: AccessLevel,
    ) {
        let cache_key = self.make_permission_key(wallet_address, additional_context);
        let now = Instant::now();
        
        let entry = PermissionCacheEntry {
            permissions,
            access_level,
            cached_at: now,
            expires_at: now + self.config.permission_cache_ttl,
        };
        
        self.permission_cache.insert(cache_key, entry);
        
        debug!(
            user_id = %wallet_address,
            wallet_address = ?wallet_address,
            ttl_seconds = self.config.permission_cache_ttl.as_secs(),
            "Cached user permissions"
        );
        
        self.enforce_cache_limits().await;
    }
    
    /// Get cached challenge
    pub async fn get_challenge(&self, nonce: &str) -> Option<ChallengeCacheEntry> {
        if let Some(entry) = self.challenge_cache.get(nonce) {
            if Instant::now() < entry.expires_at {
                let mut stats = self.stats.write().await;
                stats.challenge_hits += 1;
                
                debug!(nonce = %nonce, "Challenge cache hit");
                return Some(entry.clone());
            } else {
                self.challenge_cache.remove(nonce);
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.challenge_misses += 1;
        None
    }
    
    /// Cache SIWE challenge
    pub async fn cache_challenge(&self, nonce: &str, challenge: String, wallet_address: String) {
        let now = Instant::now();
        
        let entry = ChallengeCacheEntry {
            challenge,
            wallet_address,
            nonce: nonce.to_string(),
            expires_at: now + self.config.challenge_cache_ttl,
        };
        
        self.challenge_cache.insert(nonce.to_string(), entry);
        
        debug!(
            nonce = %nonce,
            ttl_seconds = self.config.challenge_cache_ttl.as_secs(),
            "Cached SIWE challenge"
        );
    }
    
    /// Invalidate user permissions
    pub async fn invalidate_user_permissions(&self, wallet_address: &Uuid) {
        let keys_to_remove: Vec<String> = self.permission_cache
            .iter()
            .filter_map(|entry| {
                if entry.key().starts_with(&wallet_address.to_string()) {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();
        
        for key in keys_to_remove {
            self.permission_cache.remove(&key);
        }
        
        info!(user_id = %wallet_address, "Invalidated user permission cache");
    }
    
    /// Get cache statistics
    pub async fn get_stats(&self) -> SimplifiedCacheStats {
        self.stats.read().await.clone()
    }
    
    /// Clear all caches
    pub async fn clear_all(&self) {
        self.permission_cache.clear();
        self.challenge_cache.clear();
        
        let mut stats = self.stats.write().await;
        *stats = SimplifiedCacheStats::default();
        
        info!("Cleared all authentication caches");
    }
    
    /// Get performance metrics as JSON
    pub async fn get_performance_json(&self) -> Result<serde_json::Value> {
        let stats = self.get_stats().await;
        
        Ok(serde_json::json!({
            "permission_cache": {
                "hits": stats.permission_hits,
                "misses": stats.permission_misses,
                "hit_rate": stats.permission_hit_rate(),
                "size": self.permission_cache.len()
            },
            "challenge_cache": {
                "hits": stats.challenge_hits,
                "misses": stats.challenge_misses,
                "hit_rate": stats.challenge_hit_rate(),
                "size": self.challenge_cache.len()
            },
            "overall": {
                "hit_rate": stats.overall_hit_rate(),
                "total_evictions": stats.total_evictions
            }
        }))
    }
    
    // Private helper methods
    
    fn make_permission_key(&self, wallet_address: &str, additional_context: Option<&str>) -> String {
        match additional_context {
            Some(context) => format!("{}:{}", wallet_address, context),
            None => wallet_address.to_string(),
        }
    }
    
    async fn enforce_cache_limits(&self) {
        if self.permission_cache.len() > self.config.max_cache_size {
            self.evict_oldest_permissions().await;
        }
    }
    
    async fn evict_oldest_permissions(&self) {
        let mut oldest_key: Option<String> = None;
        let mut oldest_time = Instant::now();
        
        for entry in self.permission_cache.iter() {
            if entry.cached_at < oldest_time {
                oldest_time = entry.cached_at;
                oldest_key = Some(entry.key().clone());
            }
        }
        
        if let Some(key) = oldest_key {
            self.permission_cache.remove(&key);
            let mut stats = self.stats.write().await;
            stats.total_evictions += 1;
        }
    }
    
    fn start_cleanup_task(&self) {
        let permission_cache = self.permission_cache.clone();
        let challenge_cache = self.challenge_cache.clone();
        let interval = self.config.cleanup_interval;
        
        tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(interval);
            
            loop {
                cleanup_interval.tick().await;
                
                let now = Instant::now();
                let mut expired_count = 0;
                
                // Cleanup expired permission entries
                permission_cache.retain(|_, entry| {
                    if now >= entry.expires_at {
                        expired_count += 1;
                        false
                    } else {
                        true
                    }
                });
                
                // Cleanup expired challenge entries
                challenge_cache.retain(|_, entry| {
                    if now >= entry.expires_at {
                        expired_count += 1;
                        false
                    } else {
                        true
                    }
                });
                
                if expired_count > 0 {
                    debug!(
                        expired_entries = expired_count,
                        permission_cache_size = permission_cache.len(),
                        challenge_cache_size = challenge_cache.len(),
                        "Completed cache cleanup"
                    );
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_permission_caching() {
        let config = SimplifiedCacheConfig {
            permission_cache_ttl: Duration::from_millis(100),
            ..Default::default()
        };
        let cache = SimplifiedAuthCache::new(config);
        
        let user_id = Uuid::new_v4();
        let permissions = vec![];
        let access_level = AccessLevel::Free;
        
        // Test cache miss
        let result = cache.get_permissions(&user_id, None).await;
        assert!(result.is_none());
        
        // Cache the permissions
        cache.cache_permissions(&user_id, None, permissions.clone(), access_level.clone()).await;
        
        // Test cache hit
        let result = cache.get_permissions(&user_id, None).await;
        assert!(result.is_some());
        
        // Wait for expiry
        sleep(Duration::from_millis(150)).await;
        
        // Test cache miss after expiry
        let result = cache.get_permissions(&user_id, None).await;
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_cache_stats() {
        let cache = SimplifiedAuthCache::new(SimplifiedCacheConfig::default());
        let user_id = Uuid::new_v4();
        
        // Generate cache misses
        cache.get_permissions(&user_id, None).await;
        cache.get_permissions(&user_id, None).await;
        
        let stats = cache.get_stats().await;
        assert_eq!(stats.permission_misses, 2);
        assert_eq!(stats.permission_hit_rate(), 0.0);
    }
}