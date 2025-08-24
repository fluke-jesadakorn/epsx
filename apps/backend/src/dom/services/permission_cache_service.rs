use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};
use crate::infra::cache::{Cache, CacheExt};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedPermission {
    pub allowed: bool,
    pub expires_at: u64,
    pub cached_at: u64,
}

impl CachedPermission {
    pub fn new(allowed: bool, ttl: Duration) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Self {
            allowed,
            expires_at: now + ttl.as_secs(),
            cached_at: now,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        self.expires_at <= now
    }
}

/// Permission cache service with automatic Redis + in-memory fallback
pub struct PermissionCacheService {
    cache: Arc<dyn Cache>,
}

impl PermissionCacheService {
    /// Create new permission cache service with unified cache
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }

    /// Get cached permission result with automatic fallback
    pub async fn get_cached_permission(&self, user_id: &str, resource: &str, action: &str) -> Option<bool> {
        let key = self.create_permission_key(user_id, resource, action);
        
        match self.cache.get::<CachedPermission>(&key).await {
            Ok(Some(cached_permission)) => {
                if !cached_permission.is_expired() {
                    debug!(
                        "Cache hit for permission: user={}, resource={}, action={}, allowed={}",
                        user_id, resource, action, cached_permission.allowed
                    );
                    Some(cached_permission.allowed)
                } else {
                    debug!(
                        "Cached permission expired for user={}, resource={}, action={}",
                        user_id, resource, action
                    );
                    // Clean up expired entry
                    let _ = self.cache.delete(&key).await;
                    None
                }
            }
            Ok(None) => {
                debug!("Cache miss for permission: user={}, resource={}, action={}", user_id, resource, action);
                None
            }
            Err(e) => {
                warn!("Cache error when getting permission for user={}, resource={}, action={}: {}", 
                      user_id, resource, action, e);
                None
            }
        }
    }

    /// Cache permission result with automatic fallback
    pub async fn cache_permission(&self, user_id: &str, resource: &str, action: &str, allowed: bool, ttl: Duration) -> bool {
        let key = self.create_permission_key(user_id, resource, action);
        let cached_permission = CachedPermission::new(allowed, ttl);
        
        match self.cache.set(&key, &cached_permission, Some(ttl.as_secs() as i64)).await {
            Ok(_) => {
                debug!(
                    "Successfully cached permission: user={}, resource={}, action={}, allowed={}, ttl={}s",
                    user_id, resource, action, allowed, ttl.as_secs()
                );
                true
            }
            Err(e) => {
                warn!(
                    "Failed to cache permission for user={}, resource={}, action={}: {}",
                    user_id, resource, action, e
                );
                false
            }
        }
    }

    /// Invalidate cached permission for specific user/resource/action
    pub async fn invalidate_permission(&self, user_id: &str, resource: &str, action: &str) -> bool {
        let key = self.create_permission_key(user_id, resource, action);
        
        match self.cache.delete(&key).await {
            Ok(deleted) => {
                if deleted {
                    debug!("Invalidated cached permission: user={}, resource={}, action={}", user_id, resource, action);
                }
                deleted
            }
            Err(e) => {
                warn!("Failed to invalidate cached permission for user={}, resource={}, action={}: {}", 
                      user_id, resource, action, e);
                false
            }
        }
    }

    /// Invalidate all cached permissions for a user
    pub async fn invalidate_user_permissions(&self, user_id: &str) -> u64 {
        let pattern_key = format!("permissions:{}:*", user_id);
        
        // Note: This is a simplified implementation. For production, you might want
        // to track keys or use a more sophisticated pattern matching approach
        match self.cache.delete_many(&[pattern_key]).await {
            Ok(deleted) => {
                debug!("Invalidated {} cached permissions for user={}", deleted, user_id);
                deleted
            }
            Err(e) => {
                warn!("Failed to invalidate cached permissions for user={}: {}", user_id, e);
                0
            }
        }
    }

    /// Get cache statistics for permissions
    pub async fn get_cache_stats(&self) -> Result<crate::infra::cache::CacheStats, crate::infra::cache::CacheError> {
        self.cache.stats().await
    }

    /// Create permission cache key
    fn create_permission_key(&self, user_id: &str, resource: &str, action: &str) -> String {
        format!("permissions:{}:{}:{}", user_id, resource, action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::cache::CacheFactory;

    #[tokio::test]
    async fn test_permission_cache_basic_operations() {
        let cache = CacheFactory::with_fallback().await;
        let service = PermissionCacheService::new(cache);

        // Test cache miss
        assert_eq!(service.get_cached_permission("user1", "resource1", "read").await, None);

        // Test cache set
        assert!(service.cache_permission("user1", "resource1", "read", true, Duration::from_secs(300)).await);

        // Test cache hit
        assert_eq!(service.get_cached_permission("user1", "resource1", "read").await, Some(true));

        // Test invalidation
        assert!(service.invalidate_permission("user1", "resource1", "read").await);
        assert_eq!(service.get_cached_permission("user1", "resource1", "read").await, None);
    }

    #[test]
    fn test_cached_permission_expiration() {
        let permission = CachedPermission::new(true, Duration::from_secs(0)); // Already expired
        assert!(permission.is_expired());

        let permission = CachedPermission::new(true, Duration::from_secs(300)); // 5 minutes
        assert!(!permission.is_expired());
    }
}