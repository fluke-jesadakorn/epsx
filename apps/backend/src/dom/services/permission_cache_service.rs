// Permission and quota caching service for improved performance
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};

use crate::dom::values::{UserId, Role};
use crate::dom::entities::iam::{IamRole, Permission};
use crate::web::middleware::module_auth_middleware::{UserModuleAccess, ModuleQuotas};
use crate::infra::cache::{Cache, CacheExt, CacheError};

/// Service for caching user permissions and quota information
pub struct PermissionCacheService {
    cache: Arc<dyn Cache>,
    permissions_ttl: i64,  // TTL for permission cache (seconds)
    quota_ttl: i64,        // TTL for quota cache (seconds)
}

/// Cached user permission data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedUserPermissions {
    pub user_id: UserId,
    pub role: Role,
    pub iam_roles: Vec<IamRole>,
    pub permissions: Vec<Permission>,
    pub computed_permissions: Vec<String>, // Flattened permission strings
    pub cached_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Cached user quota status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedQuotaStatus {
    pub user_id: UserId,
    pub module_access: Vec<UserModuleAccess>,
    pub effective_quotas: std::collections::HashMap<String, ModuleQuotas>,
    pub current_usage: std::collections::HashMap<String, QuotaUsage>,
    pub cached_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Current quota usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaUsage {
    pub module_name: String,
    pub api_calls_used: i32,
    pub daily_usage: i32,
    pub monthly_usage: i32,
    pub last_reset_at: DateTime<Utc>,
    pub custom_usage: std::collections::HashMap<String, i32>,
}

/// Cache statistics report with hit rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatsReport {
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
    pub total_entries: usize,
    pub memory_usage_bytes: u64,
}

/// API key permission cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedApiKeyPermissions {
    pub key_id: uuid::Uuid,
    pub client_name: String,
    pub allowed_modules: Vec<UserModuleAccess>,
    pub rate_limits: std::collections::HashMap<String, i32>,
    pub cached_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// User permission resolution from cache
#[derive(Debug, Clone)]
pub struct CachedPermissionResolution {
    pub user_id: UserId,
    pub effective_permissions: Vec<String>,
    pub module_access: Vec<UserModuleAccess>,
    pub cached_at: DateTime<Utc>,
    pub from_cache: bool,
}

/// Quota check result from cache
#[derive(Debug, Clone)]
pub struct CachedQuotaCheckResult {
    pub user_id: UserId,
    pub module_name: String,
    pub quota_type: String,
    pub available: i32,
    pub used: i32,
    pub limit: i32, // -1 for unlimited
    pub can_consume: bool,
    pub cached_at: DateTime<Utc>,
}

impl PermissionCacheService {
    /// Create new permission cache service
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self {
            cache,
            permissions_ttl: 300, // 5 minutes
            quota_ttl: 60,        // 1 minute for quotas (more frequent updates)
        }
    }

    /// Create with custom TTL values
    pub fn with_ttl(cache: Arc<dyn Cache>, permissions_ttl: i64, quota_ttl: i64) -> Self {
        Self {
            cache,
            permissions_ttl,
            quota_ttl,
        }
    }

    // ========================================
    // USER PERMISSION CACHING
    // ========================================

    /// Cache user permissions
    pub async fn cache_user_permissions(
        &self,
        user_id: &UserId,
        role: Role,
        iam_roles: Vec<IamRole>,
        permissions: Vec<Permission>,
        computed_permissions: Vec<String>,
    ) -> Result<(), CacheError> {
        let now = Utc::now();
        let cached_permissions = CachedUserPermissions {
            user_id: user_id.clone(),
            role,
            iam_roles,
            permissions,
            computed_permissions,
            cached_at: now,
            expires_at: now + Duration::seconds(self.permissions_ttl),
        };

        let cache_key = format!("user_permissions:{}", user_id);
        self.cache.set(&cache_key, &cached_permissions, Some(self.permissions_ttl)).await?;
        
        tracing::debug!("Cached permissions for user {} (expires in {}s)", user_id, self.permissions_ttl);
        Ok(())
    }

    /// Get cached user permissions
    pub async fn get_user_permissions(&self, user_id: &UserId) -> Result<Option<CachedUserPermissions>, CacheError> {
        let cache_key = format!("user_permissions:{}", user_id);
        match self.cache.get::<CachedUserPermissions>(&cache_key).await? {
            Some(cached) => {
                if cached.expires_at > Utc::now() {
                    tracing::debug!("Cache hit for user permissions: {}", user_id);
                    Ok(Some(cached))
                } else {
                    tracing::debug!("Cache expired for user permissions: {}", user_id);
                    self.cache.delete(&cache_key).await?;
                    Ok(None)
                }
            }
            None => {
                tracing::debug!("Cache miss for user permissions: {}", user_id);
                Ok(None)
            }
        }
    }

    /// Check if user has specific permission (cached)
    pub async fn check_user_permission(
        &self,
        user_id: &UserId,
        resource: &str,
        action: &str,
    ) -> Result<Option<bool>, CacheError> {
        if let Some(cached_perms) = self.get_user_permissions(user_id).await? {
            let required_permission = format!("{}:{}", resource, action);
            
            // Check for exact match
            if cached_perms.computed_permissions.contains(&required_permission) {
                return Ok(Some(true));
            }
            
            // Check for wildcard matches
            for perm in &cached_perms.computed_permissions {
                if self.matches_permission(perm, resource, action) {
                    return Ok(Some(true));
                }
            }
            
            Ok(Some(false))
        } else {
            Ok(None) // Not in cache
        }
    }

    // ========================================
    // QUOTA STATUS CACHING
    // ========================================

    /// Cache user quota status
    pub async fn cache_quota_status(
        &self,
        user_id: &UserId,
        module_access: Vec<UserModuleAccess>,
        effective_quotas: std::collections::HashMap<String, ModuleQuotas>,
        current_usage: std::collections::HashMap<String, QuotaUsage>,
    ) -> Result<(), CacheError> {
        let now = Utc::now();
        let cached_quota = CachedQuotaStatus {
            user_id: user_id.clone(),
            module_access,
            effective_quotas,
            current_usage,
            cached_at: now,
            expires_at: now + Duration::seconds(self.quota_ttl),
        };

        let cache_key = format!("user_quotas:{}", user_id);
        self.cache.set(&cache_key, &cached_quota, Some(self.quota_ttl)).await?;
        
        tracing::debug!("Cached quota status for user {} (expires in {}s)", user_id, self.quota_ttl);
        Ok(())
    }

    /// Get cached quota status
    pub async fn get_quota_status(&self, user_id: &UserId) -> Result<Option<CachedQuotaStatus>, CacheError> {
        let cache_key = format!("user_quotas:{}", user_id);
        match self.cache.get::<CachedQuotaStatus>(&cache_key).await? {
            Some(cached) => {
                if cached.expires_at > Utc::now() {
                    tracing::debug!("Cache hit for user quota status: {}", user_id);
                    Ok(Some(cached))
                } else {
                    tracing::debug!("Cache expired for user quota status: {}", user_id);
                    self.cache.delete(&cache_key).await?;
                    Ok(None)
                }
            }
            None => {
                tracing::debug!("Cache miss for user quota status: {}", user_id);
                Ok(None)
            }
        }
    }

    /// Check quota availability for a specific module and quota type
    pub async fn check_quota_availability(
        &self,
        user_id: &UserId,
        module_name: &str,
        quota_type: &str,
        amount: i32,
    ) -> Result<Option<CachedQuotaCheckResult>, CacheError> {
        if let Some(cached_quota) = self.get_quota_status(user_id).await? {
            if let (Some(quotas), Some(usage)) = (
                cached_quota.effective_quotas.get(module_name),
                cached_quota.current_usage.get(module_name),
            ) {
                let (limit, used) = match quota_type {
                    "api_calls" => (quotas.api_calls.unwrap_or(-1), usage.api_calls_used),
                    "daily_limit" => (quotas.daily_limit.unwrap_or(-1), usage.daily_usage),
                    "monthly_limit" => (quotas.monthly_limit.unwrap_or(-1), usage.monthly_usage),
                    custom => (
                        quotas.custom_limits.get(custom).copied().unwrap_or(-1),
                        usage.custom_usage.get(custom).copied().unwrap_or(0)
                    ),
                };

                let available = if limit == -1 { i32::MAX } else { limit - used };
                let can_consume = limit == -1 || available >= amount;

                return Ok(Some(CachedQuotaCheckResult {
                    user_id: user_id.clone(),
                    module_name: module_name.to_string(),
                    quota_type: quota_type.to_string(),
                    available,
                    used,
                    limit,
                    can_consume,
                    cached_at: cached_quota.cached_at,
                }));
            }
        }
        
        Ok(None)
    }

    /// Update quota usage (increment)
    pub async fn increment_quota_usage(
        &self,
        user_id: &UserId,
        module_name: &str,
        quota_type: &str,
        amount: i32,
    ) -> Result<(), CacheError> {
        let cache_key = format!("user_quotas:{}", user_id);
        
        if let Some(mut cached_quota) = self.cache.get::<CachedQuotaStatus>(&cache_key).await? {
            if let Some(usage) = cached_quota.current_usage.get_mut(module_name) {
                match quota_type {
                    "api_calls" => usage.api_calls_used += amount,
                    "daily_limit" => usage.daily_usage += amount,
                    "monthly_limit" => usage.monthly_usage += amount,
                    custom => {
                        let current = usage.custom_usage.get(custom).copied().unwrap_or(0);
                        usage.custom_usage.insert(custom.to_string(), current + amount);
                    }
                }

                // Update cache with new usage
                self.cache.set(&cache_key, &cached_quota, Some(self.quota_ttl)).await?;
                tracing::debug!("Updated quota usage for user {} module {} type {} (+{})", 
                               user_id, module_name, quota_type, amount);
            }
        }

        Ok(())
    }

    // ========================================
    // API KEY CACHING
    // ========================================

    /// Cache API key permissions
    pub async fn cache_api_key_permissions(
        &self,
        key_id: uuid::Uuid,
        client_name: String,
        allowed_modules: Vec<UserModuleAccess>,
        rate_limits: std::collections::HashMap<String, i32>,
    ) -> Result<(), CacheError> {
        let now = Utc::now();
        let cached_api_key = CachedApiKeyPermissions {
            key_id,
            client_name,
            allowed_modules,
            rate_limits,
            cached_at: now,
            expires_at: now + Duration::seconds(self.permissions_ttl),
        };

        let cache_key = format!("api_key_permissions:{}", key_id);
        self.cache.set(&cache_key, &cached_api_key, Some(self.permissions_ttl)).await?;
        
        tracing::debug!("Cached API key permissions for key {} (expires in {}s)", key_id, self.permissions_ttl);
        Ok(())
    }

    /// Get cached API key permissions
    pub async fn get_api_key_permissions(&self, key_id: &uuid::Uuid) -> Result<Option<CachedApiKeyPermissions>, CacheError> {
        let cache_key = format!("api_key_permissions:{}", key_id);
        match self.cache.get::<CachedApiKeyPermissions>(&cache_key).await? {
            Some(cached) => {
                if cached.expires_at > Utc::now() {
                    tracing::debug!("Cache hit for API key permissions: {}", key_id);
                    Ok(Some(cached))
                } else {
                    tracing::debug!("Cache expired for API key permissions: {}", key_id);
                    self.cache.delete(&cache_key).await?;
                    Ok(None)
                }
            }
            None => {
                tracing::debug!("Cache miss for API key permissions: {}", key_id);
                Ok(None)
            }
        }
    }

    // ========================================
    // CACHE MANAGEMENT
    // ========================================

    /// Invalidate all cache entries for a user
    pub async fn invalidate_user_cache(&self, user_id: &UserId) -> Result<(), CacheError> {
        let keys = vec![
            format!("user_permissions:{}", user_id),
            format!("user_quotas:{}", user_id),
        ];
        
        self.cache.delete_many(&keys).await?;
        tracing::info!("Invalidated cache for user: {}", user_id);
        Ok(())
    }

    /// Invalidate API key cache
    pub async fn invalidate_api_key_cache(&self, key_id: &uuid::Uuid) -> Result<(), CacheError> {
        let cache_key = format!("api_key_permissions:{}", key_id);
        self.cache.delete(&cache_key).await?;
        tracing::info!("Invalidated cache for API key: {}", key_id);
        Ok(())
    }

    /// Clear all permission and quota caches
    pub async fn clear_all_caches(&self) -> Result<(), CacheError> {
        // For Redis, we'd need to use pattern matching to delete specific prefixes
        // For now, we'll clear the entire cache (use with caution in production)
        self.cache.clear().await?;
        tracing::warn!("Cleared all permission and quota caches");
        Ok(())
    }

    /// Preload permissions for a batch of users
    pub async fn preload_user_permissions(
        &self,
        users_data: Vec<(UserId, Role, Vec<IamRole>, Vec<Permission>, Vec<String>)>,
    ) -> Result<usize, CacheError> {
        let mut successful_count = 0;
        
        for (user_id, role, iam_roles, permissions, computed_permissions) in users_data {
            match self.cache_user_permissions(&user_id, role, iam_roles, permissions, computed_permissions).await {
                Ok(_) => {
                    successful_count += 1;
                    tracing::debug!("Preloaded permissions for user: {}", user_id);
                },
                Err(e) => {
                    tracing::warn!("Failed to preload permissions for user {}: {}", user_id, e);
                }
            }
        }
        
        tracing::info!("Preloaded permissions for {} users", successful_count);
        Ok(successful_count)
    }

    /// Get cache hit rate statistics
    pub async fn get_cache_stats(&self) -> Result<CacheStatsReport, CacheError> {
        let stats = self.cache.stats().await?;
        Ok(CacheStatsReport {
            hit_count: stats.hit_count.unwrap_or(0),
            miss_count: stats.miss_count.unwrap_or(0),
            hit_rate: if let (Some(hits), Some(misses)) = (stats.hit_count, stats.miss_count) {
                if hits + misses > 0 {
                    hits as f64 / (hits + misses) as f64 * 100.0
                } else {
                    0.0
                }
            } else {
                stats.hit_rate.unwrap_or(0.0) * 100.0
            },
            total_entries: stats.total_entries as usize,
            memory_usage_bytes: stats.memory_usage_bytes.unwrap_or(0),
        })
    }

    // ========================================
    // HELPER METHODS
    // ========================================

    /// Check if a permission pattern matches the required resource and action
    fn matches_permission(&self, permission: &str, resource: &str, action: &str) -> bool {
        let parts: Vec<&str> = permission.split(':').collect();
        if parts.len() != 2 {
            return false;
        }

        let perm_resource = parts[0];
        let perm_action = parts[1];

        // Check resource match
        let resource_match = perm_resource == "*" 
            || perm_resource == resource
            || (perm_resource.ends_with('*') 
                && resource.starts_with(&perm_resource[..perm_resource.len()-1]));

        // Check action match
        let action_match = perm_action == "*"
            || perm_action == action
            || (perm_action.ends_with('*') 
                && action.starts_with(&perm_action[..perm_action.len()-1]));

        resource_match && action_match
    }

    /// Warm up cache for a user (preload permissions and quotas)
    pub async fn warm_up_user_cache(
        &self,
        user_id: &UserId,
        // These would come from the actual repositories in real implementation
        _load_permissions: impl Fn(&UserId) -> Vec<String>,
        _load_quotas: impl Fn(&UserId) -> std::collections::HashMap<String, ModuleQuotas>,
    ) -> Result<(), CacheError> {
        // Implementation would call the actual repository methods to load fresh data
        // and cache it for the user
        tracing::info!("Cache warm-up requested for user: {}", user_id);
        // TODO: Implement actual cache warming
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::cache::{CacheFactory, CacheConfig, CacheBackend};

    #[tokio::test]
    async fn test_permission_matching() {
        let cache = CacheFactory::create(CacheConfig::default()).await.unwrap();
        let service = PermissionCacheService::new(cache);

        // Test exact match
        assert!(service.matches_permission("api:read", "api", "read"));
        assert!(!service.matches_permission("api:read", "api", "write"));

        // Test wildcard resource
        assert!(service.matches_permission("*:read", "api", "read"));
        assert!(service.matches_permission("*:read", "users", "read"));

        // Test wildcard action
        assert!(service.matches_permission("api:*", "api", "read"));
        assert!(service.matches_permission("api:*", "api", "write"));

        // Test prefix wildcard
        assert!(service.matches_permission("api:*", "api", "read"));
        assert!(service.matches_permission("api*:read", "api_v1", "read"));
    }

    #[tokio::test]
    async fn test_permission_caching() {
        let cache = CacheFactory::create(CacheConfig::default()).await.unwrap();
        let service = PermissionCacheService::new(cache);
        
        let user_id = UserId::generate();
        let permissions = vec!["api:read".to_string(), "users:*".to_string()];
        
        // Cache should be empty initially
        assert!(service.get_user_permissions(&user_id).await.unwrap().is_none());
        
        // Cache permissions
        service.cache_user_permissions(
            &user_id,
            Role::User,
            vec![],
            vec![],
            permissions.clone(),
        ).await.unwrap();
        
        // Should now be in cache
        let cached = service.get_user_permissions(&user_id).await.unwrap().unwrap();
        assert_eq!(cached.user_id, user_id);
        assert_eq!(cached.computed_permissions, permissions);
        
        // Permission checks should work
        assert_eq!(service.check_user_permission(&user_id, "api", "read").await.unwrap(), Some(true));
        assert_eq!(service.check_user_permission(&user_id, "users", "delete").await.unwrap(), Some(true));
        assert_eq!(service.check_user_permission(&user_id, "admin", "access").await.unwrap(), Some(false));
    }
}