// Permission resolution service for determining user access and resolving permission conflicts
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};

use crate::dom::entities::iam::Permission;
use crate::dom::entities::permission_profile::{PermissionProfile, PermissionProfileId};
use crate::dom::values::UserId;
use crate::infra::cache::{Cache, CacheFactory, CacheConfig, CacheBackend};

/// Service for resolving user permissions from multiple sources
pub struct PermissionResolver {
    /// Cache backend (can be in-memory or Redis)
    cache: Arc<dyn Cache>,
    /// Cache TTL in seconds
    cache_ttl: i64,
}

/// Cached permission data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CachedPermissions {
    permissions: Vec<Permission>,
    profiles: Vec<PermissionProfileId>,
    cached_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

/// User permission resolution result
#[derive(Debug, Clone)]
pub struct UserPermissionResolution {
    /// User ID
    pub user_id: UserId,
    /// Resolved permissions (after conflict resolution)
    pub effective_permissions: Vec<Permission>,
    /// Permission profiles contributing to this resolution
    pub contributing_profiles: Vec<PermissionProfileId>,
    /// Permission conflicts found and how they were resolved
    pub conflict_resolutions: Vec<PermissionConflict>,
    /// Timestamp of resolution
    pub resolved_at: DateTime<Utc>,
    /// Whether this result came from cache
    pub from_cache: bool,
}

/// Permission conflict and its resolution
#[derive(Debug, Clone)]
pub struct PermissionConflict {
    /// Resource where conflict occurred
    pub resource: String,
    /// Action where conflict occurred
    pub action: String,
    /// Conflicting permissions
    pub conflicting_permissions: Vec<Permission>,
    /// How the conflict was resolved
    pub resolution_strategy: ConflictResolutionStrategy,
    /// Final permission chosen
    pub resolved_permission: Permission,
}

/// Strategies for resolving permission conflicts
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictResolutionStrategy {
    /// Most permissive wins (e.g., "read:*" beats "read:own")
    MostPermissive,
    /// Most restrictive wins
    MostRestrictive,
    /// Profile priority based (admin profiles beat user profiles)
    ProfilePriority,
    /// Last assigned wins
    LastAssigned,
}

/// Permission evaluation context
#[derive(Debug, Clone)]
pub struct PermissionContext {
    /// User requesting permission
    pub user_id: UserId,
    /// Resource being accessed
    pub resource: String,
    /// Action being performed
    pub action: String,
    /// Additional context data
    pub context_data: HashMap<String, String>,
    /// Request timestamp
    pub requested_at: DateTime<Utc>,
}

/// Permission check result
#[derive(Debug, Clone)]
pub struct PermissionCheckResult {
    /// Whether permission is granted
    pub granted: bool,
    /// Reason for the decision
    pub reason: String,
    /// Permission that granted access (if any)
    pub granting_permission: Option<Permission>,
    /// Profile that provided the permission (if any)
    pub granting_profile: Option<PermissionProfileId>,
    /// Check timestamp
    pub checked_at: DateTime<Utc>,
}

impl PermissionResolver {
    /// Create new permission resolver with in-memory cache (default)
    pub async fn new() -> Result<Self, PermissionResolutionError> {
        let cache_config = CacheConfig {
            backend: CacheBackend::InMemory,
            default_ttl_seconds: 300, // 5 minutes
            max_entries: Some(10000),
            enable_compression: false,
        };
        
        let cache = CacheFactory::create(cache_config).await
            .map_err(|e| PermissionResolutionError::CacheError(e.to_string()))?;
        
        Ok(Self {
            cache,
            cache_ttl: 300,
        })
    }

    /// Create permission resolver with custom cache configuration
    pub async fn with_cache_config(cache_config: CacheConfig) -> Result<Self, PermissionResolutionError> {
        let cache_ttl = cache_config.default_ttl_seconds;
        let cache = CacheFactory::create(cache_config).await
            .map_err(|e| PermissionResolutionError::CacheError(e.to_string()))?;
        
        Ok(Self {
            cache,
            cache_ttl,
        })
    }

    /// Create permission resolver from environment variables
    /// Will use Redis if REDIS_URL is set, otherwise in-memory cache
    pub async fn from_env() -> Result<Self, PermissionResolutionError> {
        let cache = CacheFactory::from_env().await
            .map_err(|e| PermissionResolutionError::CacheError(e.to_string()))?;
        
        let cache_ttl = std::env::var("PERMISSION_CACHE_TTL")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .unwrap_or(300);
        
        Ok(Self {
            cache,
            cache_ttl,
        })
    }

    /// Resolve user permissions from multiple permission profiles
    pub async fn resolve_user_permissions(
        &self,
        user_id: &UserId,
        profiles: Vec<PermissionProfile>,
    ) -> Result<UserPermissionResolution, PermissionResolutionError> {
        let cache_key = format!("user_permissions:{}", user_id);
        
        // Check cache first
        if let Ok(Some(cached_raw)) = self.cache.get_raw(&cache_key).await {
            if let Ok(cached) = serde_json::from_str::<CachedPermissions>(&cached_raw) {
                if cached.expires_at > Utc::now() {
                    return Ok(UserPermissionResolution {
                        user_id: user_id.clone(),
                        effective_permissions: cached.permissions,
                        contributing_profiles: cached.profiles,
                        conflict_resolutions: Vec::new(), // Not stored in cache for now
                        resolved_at: cached.cached_at,
                        from_cache: true,
                    });
                }
            }
        }

        // Resolve permissions from profiles
        let mut all_permissions = Vec::new();
        let mut contributing_profiles = Vec::new();
        let mut permission_sources: HashMap<String, Vec<(Permission, PermissionProfileId)>> = HashMap::new();

        for profile in profiles {
            if !profile.is_active() {
                continue;
            }

            contributing_profiles.push(profile.id().clone());
            
            for permission in profile.default_permissions() {
                let resource_action = format!("{}:{}", permission.resource(), permission.action());
                permission_sources
                    .entry(resource_action)
                    .or_insert_with(Vec::new)
                    .push((permission.clone(), profile.id().clone()));
                
                all_permissions.push(permission.clone());
            }
        }

        // Resolve conflicts
        let (effective_permissions, conflict_resolutions) = self.resolve_conflicts(permission_sources);

        // Cache the result
        let resolved_at = Utc::now();
        let cached_permissions = CachedPermissions {
            permissions: effective_permissions.clone(),
            profiles: contributing_profiles.clone(),
            cached_at: resolved_at,
            expires_at: resolved_at + Duration::seconds(self.cache_ttl),
        };
        
        let cached_permissions_json = serde_json::to_string(&cached_permissions)
            .map_err(|e| PermissionResolutionError::CacheError(format!("Serialization error: {}", e)))?;
        if let Err(e) = self.cache.set_raw(&cache_key, &cached_permissions_json, Some(self.cache_ttl)).await {
            tracing::warn!("Failed to cache permissions for user {}: {}", user_id, e);
        }

        Ok(UserPermissionResolution {
            user_id: user_id.clone(),
            effective_permissions,
            contributing_profiles,
            conflict_resolutions,
            resolved_at,
            from_cache: false,
        })
    }

    /// Check if user has specific permission
    pub async fn check_permission(
        &self,
        context: &PermissionContext,
        user_permissions: &[Permission],
    ) -> PermissionCheckResult {
        let resource_action = format!("{}:{}", context.resource, context.action);
        
        // Check for exact match
        for permission in user_permissions {
            let perm_resource_action = format!("{}:{}", permission.resource(), permission.action());
            
            if perm_resource_action == resource_action {
                return PermissionCheckResult {
                    granted: true,
                    reason: "Exact permission match".to_string(),
                    granting_permission: Some(permission.clone()),
                    granting_profile: None, // Would need additional context to determine this
                    checked_at: Utc::now(),
                };
            }
        }

        // Check for wildcard matches
        for permission in user_permissions {
            if self.matches_wildcard_permission(permission, &context.resource, &context.action) {
                return PermissionCheckResult {
                    granted: true,
                    reason: "Wildcard permission match".to_string(),
                    granting_permission: Some(permission.clone()),
                    granting_profile: None,
                    checked_at: Utc::now(),
                };
            }
        }

        PermissionCheckResult {
            granted: false,
            reason: "No matching permission found".to_string(),
            granting_permission: None,
            granting_profile: None,
            checked_at: Utc::now(),
        }
    }

    /// Check API access permission
    pub async fn check_api_access(
        &self,
        user_id: &UserId,
        user_permissions: &[Permission],
        api_path: &str,
        method: &str,
    ) -> PermissionCheckResult {
        let context = PermissionContext {
            user_id: user_id.clone(),
            resource: format!("api:{}", api_path),
            action: method.to_lowercase(),
            context_data: HashMap::new(),
            requested_at: Utc::now(),
        };

        self.check_permission(&context, user_permissions).await
    }

    /// Check route access permission
    pub async fn check_route_access(
        &self,
        user_id: &UserId,
        user_permissions: &[Permission],
        route: &str,
    ) -> PermissionCheckResult {
        let context = PermissionContext {
            user_id: user_id.clone(),
            resource: "route".to_string(),
            action: route.to_string(),
            context_data: HashMap::new(),
            requested_at: Utc::now(),
        };

        self.check_permission(&context, user_permissions).await
    }

    /// Resolve permission conflicts using configured strategy
    fn resolve_conflicts(
        &self,
        permission_sources: HashMap<String, Vec<(Permission, PermissionProfileId)>>,
    ) -> (Vec<Permission>, Vec<PermissionConflict>) {
        let mut effective_permissions = Vec::new();
        let mut conflicts = Vec::new();

        for (resource_action, permissions) in permission_sources {
            if permissions.len() == 1 {
                // No conflict
                effective_permissions.push(permissions[0].0.clone());
            } else {
                // Conflict - resolve using most permissive strategy for now
                let conflict = self.resolve_single_conflict(resource_action, permissions);
                effective_permissions.push(conflict.resolved_permission.clone());
                conflicts.push(conflict);
            }
        }

        (effective_permissions, conflicts)
    }

    /// Resolve a single permission conflict
    fn resolve_single_conflict(
        &self,
        resource_action: String,
        permissions: Vec<(Permission, PermissionProfileId)>,
    ) -> PermissionConflict {
        let parts: Vec<&str> = resource_action.split(':').collect();
        let resource = parts.get(0).unwrap_or(&"unknown").to_string();
        let action = parts.get(1).unwrap_or(&"unknown").to_string();

        // Use most permissive strategy by default
        let most_permissive = permissions.iter()
            .max_by(|a, b| self.compare_permission_permissiveness(&a.0, &b.0))
            .unwrap()
            .0.clone();

        PermissionConflict {
            resource,
            action,
            conflicting_permissions: permissions.iter().map(|(p, _)| p.clone()).collect(),
            resolution_strategy: ConflictResolutionStrategy::MostPermissive,
            resolved_permission: most_permissive,
        }
    }

    /// Compare permissions by permissiveness (wildcards are more permissive)
    fn compare_permission_permissiveness(&self, a: &Permission, b: &Permission) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        // "*" is most permissive
        match (a.action().as_ref(), b.action().as_ref()) {
            ("*", "*") => Ordering::Equal,
            ("*", _) => Ordering::Greater,
            (_, "*") => Ordering::Less,
            _ => {
                // Compare action specificity (less specific = more permissive)
                if a.action().len() < b.action().len() {
                    Ordering::Greater
                } else if a.action().len() > b.action().len() {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            }
        }
    }

    /// Check if permission matches using wildcard rules
    fn matches_wildcard_permission(
        &self,
        permission: &Permission,
        resource: &str,
        action: &str,
    ) -> bool {
        // Check resource match
        let resource_match = permission.resource() == "*" 
            || permission.resource() == resource
            || (permission.resource().ends_with("*") && resource.starts_with(&permission.resource()[..permission.resource().len()-1]));

        // Check action match
        let action_match = permission.action() == "*"
            || permission.action() == action
            || (permission.action().ends_with("*") && action.starts_with(&permission.action()[..permission.action().len()-1]));

        resource_match && action_match
    }

    /// Clear cache for specific user
    pub async fn invalidate_user_cache(&self, user_id: &UserId) -> Result<(), PermissionResolutionError> {
        let cache_key = format!("user_permissions:{}", user_id);
        self.cache.delete(&cache_key).await
            .map_err(|e| PermissionResolutionError::CacheError(e.to_string()))?;
        Ok(())
    }

    /// Clear all cached permissions
    pub async fn clear_cache(&self) -> Result<(), PermissionResolutionError> {
        self.cache.clear().await
            .map_err(|e| PermissionResolutionError::CacheError(e.to_string()))?;
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<PermissionCacheStats, PermissionResolutionError> {
        let stats = self.cache.stats().await
            .map_err(|e| PermissionResolutionError::CacheError(e.to_string()))?;

        Ok(PermissionCacheStats {
            total_entries: stats.total_entries,
            expired_entries: stats.expired_entries,
            active_entries: stats.active_entries,
            memory_usage_bytes: stats.memory_usage_bytes,
            hit_count: stats.hit_count,
            miss_count: stats.miss_count,
            hit_rate: stats.hit_rate,
        })
    }
}

/// Permission cache statistics
#[derive(Debug, Clone)]
pub struct PermissionCacheStats {
    pub total_entries: u64,
    pub expired_entries: u64,
    pub active_entries: u64,
    pub memory_usage_bytes: Option<u64>,
    pub hit_count: Option<u64>,
    pub miss_count: Option<u64>,
    pub hit_rate: Option<f64>,
}

/// Permission resolution errors
#[derive(Debug, thiserror::Error)]
pub enum PermissionResolutionError {
    #[error("Failed to resolve permissions: {0}")]
    ResolutionFailed(String),
    
    #[error("Invalid permission format: {0}")]
    InvalidPermissionFormat(String),
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Conflict resolution failed: {0}")]
    ConflictResolutionFailed(String),
}

// Note: Default impl removed since constructor is now async

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::entities::iam::Permission;

    #[tokio::test]
    async fn test_permission_wildcard_matching() {
        let resolver = PermissionResolver::new().await.unwrap();
        
        let wildcard_perm = Permission::new("api:*".to_string(), "read".to_string());
        assert!(resolver.matches_wildcard_permission(
            &wildcard_perm,
            "api:users",
            "read"
        ));
        
        let specific_perm = Permission::new("api:users".to_string(), "read".to_string());
        assert!(resolver.matches_wildcard_permission(
            &specific_perm,
            "api:users",
            "read"
        ));
        
        assert!(!resolver.matches_wildcard_permission(
            &specific_perm,
            "api:posts",
            "read"
        ));
    }

    #[tokio::test]
    async fn test_permission_comparison() {
        let resolver = PermissionResolver::new().await.unwrap();
        
        let wildcard_perm = Permission::new("resource".to_string(), "*".to_string());
        let specific_perm = Permission::new("resource".to_string(), "read".to_string());
        
        assert_eq!(
            resolver.compare_permission_permissiveness(&wildcard_perm, &specific_perm),
            std::cmp::Ordering::Greater
        );
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let resolver = PermissionResolver::new().await.unwrap();
        let stats = resolver.get_cache_stats().await.unwrap();
        
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.active_entries, 0);
        assert_eq!(stats.expired_entries, 0);
    }
}