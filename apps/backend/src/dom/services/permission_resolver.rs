// Unified permission service - consolidates AuthorizationService, PermissionResolver, and PolicyEngine
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::dom::entities::iam::Permission;
use crate::dom::entities::permission_profile::{PermissionProfile, PermissionProfileId};
use crate::dom::values::{UserId, Role};
use crate::dom::entities::user::User;
use crate::dom::ports::DomainCache;
use super::permission_cache_service::PermissionCacheService;
use super::policy_engine::{PolicyEngine, EvaluationContext, EvaluationDecision};

/// Unified Permission Service - consolidates permission resolution, authorization, and policy evaluation
pub struct UnifiedPermissionService {
    /// Cache backend (domain abstraction)
    cache: Arc<dyn DomainCache>,
    /// Cache TTL in seconds
    _cache_ttl: i64,
    /// Enhanced permission caching service
    permission_cache: PermissionCacheService,
    /// Policy evaluation engine
    policy_engine: PolicyEngine,
}

/// For backward compatibility - re-export as PermissionResolver
pub type PermissionResolver = UnifiedPermissionService;

/// Authorization error types (consolidated from app layer)
#[derive(Debug, thiserror::Error)]
pub enum AuthorizationError {
    #[error("Insufficient permissions: required '{required}', user role '{user_role}'")]
    InsufficientPermissions {
        required: String,
        user_role: String,
    },
    
    #[error("Insufficient role: required '{required}', current '{current}'")]
    InsufficientRole {
        required: String,
        current: String,
    },
    
    #[error("Access denied: user '{user_id}' cannot access resource '{resource}'")]
    AccessDenied {
        user_id: String,
        resource: String,
    },
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

impl UnifiedPermissionService {
    /// Create new unified permission service with dependency injection
    pub fn new(
        cache: Arc<dyn DomainCache>,
        permission_cache: PermissionCacheService,
    ) -> Self {
        let policy_engine = PolicyEngine::new();
        
        Self {
            cache,
            _cache_ttl: 300,
            permission_cache,
            policy_engine,
        }
    }

    /// Create unified permission service with custom cache TTL
    pub fn with_cache_ttl(
        cache: Arc<dyn DomainCache>,
        permission_cache: PermissionCacheService,
        cache_ttl: i64,
    ) -> Self {
        let policy_engine = PolicyEngine::new();
        
        Self {
            cache,
            _cache_ttl: cache_ttl,
            permission_cache,
            policy_engine,
        }
    }

    /// Resolve user permissions from multiple permission profiles
    pub async fn resolve_user_permissions(
        &self,
        user_id: &UserId,
        profiles: Vec<PermissionProfile>,
    ) -> Result<UserPermissionResolution, PermissionResolutionError> {
        
        // Check enhanced cache first
        if let Ok(Some(cached)) = self.permission_cache.get_user_permissions(user_id).await {
            return Ok(UserPermissionResolution {
                user_id: user_id.clone(),
                effective_permissions: cached.permissions,
                contributing_profiles: cached.iam_roles.iter().map(|r| PermissionProfileId::new(r.id().to_string())).collect(),
                conflict_resolutions: Vec::new(), // Not stored in cache for now
                resolved_at: cached.cached_at,
                from_cache: true,
            });
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

        // Cache the result using enhanced caching service
        let resolved_at = Utc::now();
        let computed_permissions: Vec<String> = effective_permissions.iter()
            .map(|p| format!("{}:{}", p.resource(), p.action()))
            .collect();
        
        // Cache using the enhanced service
        if let Err(e) = self.permission_cache.cache_user_permissions(
            user_id,
            crate::dom::values::Role::User, // Would get actual role from profiles
            vec![], // Would convert profiles to IAM roles
            effective_permissions.clone(),
            computed_permissions,
        ).await {
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
        // Try cache first for faster lookups
        if let Ok(Some(cached_result)) = self.permission_cache.check_user_permission(
            &context.user_id,
            &context.resource,
            &context.action,
        ).await {
            return PermissionCheckResult {
                granted: cached_result,
                reason: if cached_result { "Cached permission match" } else { "Cached permission denied" }.to_string(),
                granting_permission: None, // Would need to store this in cache
                granting_profile: None,
                checked_at: Utc::now(),
            };
        }

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
        // Use enhanced cache service for better cache invalidation
        self.permission_cache.invalidate_user_cache(user_id).await
            .map_err(|e| PermissionResolutionError::CacheError(e.to_string()))?;
        
        // Also clear old cache format for backward compatibility
        let cache_key = format!("user_permissions:{}", user_id);
        if let Err(e) = self.cache.delete(&cache_key).await {
            tracing::warn!("Failed to clear legacy cache for user {}: {}", user_id, e);
        }
        
        Ok(())
    }

    /// Clear all cached permissions
    pub async fn clear_cache(&self) -> Result<(), PermissionResolutionError> {
        self.permission_cache.clear_all_caches().await
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

    // ========================================
    // QUOTA CACHING METHODS
    // ========================================

    /// Cache quota status for a user
    pub async fn cache_user_quota_status(
        &self,
        user_id: &UserId,
        module_access: Vec<crate::web::middleware::module_auth_middleware::UserModuleAccess>,
        effective_quotas: std::collections::HashMap<String, crate::web::middleware::module_auth_middleware::ModuleQuotas>,
    ) -> Result<(), PermissionResolutionError> {
        let current_usage = std::collections::HashMap::new(); // Would be populated from actual usage data
        
        self.permission_cache.cache_quota_status(user_id, module_access, effective_quotas, current_usage).await
            .map_err(|e| PermissionResolutionError::CacheError(e.to_string()))?;
        
        Ok(())
    }

    /// Check quota availability from cache
    pub async fn check_cached_quota_availability(
        &self,
        user_id: &UserId,
        module_name: &str,
        quota_type: &str,
        amount: i32,
    ) -> Result<Option<bool>, PermissionResolutionError> {
        match self.permission_cache.check_quota_availability(user_id, module_name, quota_type, amount).await {
            Ok(Some(result)) => Ok(Some(result.can_consume)),
            Ok(None) => Ok(None), // Not in cache
            Err(e) => Err(PermissionResolutionError::CacheError(e.to_string())),
        }
    }

    /// Increment quota usage in cache
    pub async fn increment_cached_quota_usage(
        &self,
        user_id: &UserId,
        module_name: &str,
        quota_type: &str,
        amount: i32,
    ) -> Result<(), PermissionResolutionError> {
        self.permission_cache.increment_quota_usage(user_id, module_name, quota_type, amount).await
            .map_err(|e| PermissionResolutionError::CacheError(e.to_string()))?;
        
        Ok(())
    }

    /// Get the permission cache service for direct access
    pub fn permission_cache(&self) -> &PermissionCacheService {
        &self.permission_cache
    }
    
    // ========================================
    // AUTHORIZATION METHODS (from app layer)
    // ========================================
    
    
    /// Check if user has required role level (consolidated from AuthorizationService)
    pub fn check_role_level(&self, user: &User, required_role: &Role) -> Result<(), AuthorizationError> {
        if user.role().hierarchy_level() >= required_role.hierarchy_level() {
            Ok(())
        } else {
            Err(AuthorizationError::InsufficientRole {
                required: required_role.to_string(),
                current: user.role().to_string(),
            })
        }
    }
    
    /// Check if user can access another user's data (consolidated from AuthorizationService)
    pub fn check_user_access(&self, requesting_user: &User, target_user_id: &UserId) -> Result<(), AuthorizationError> {
        // Users can access their own data
        if requesting_user.id() == target_user_id {
            return Ok(());
        }
        
        // Admins can access other users' data
        if requesting_user.has_perm("read:all_data") {
            return Ok(());
        }
        
        Err(AuthorizationError::AccessDenied {
            user_id: requesting_user.id().to_string(),
            resource: target_user_id.to_string(),
        })
    }
    
    /// Evaluate policy-based permissions using policy engine
    pub async fn evaluate_policy_permission(
        &self,
        context: EvaluationContext,
    ) -> Result<EvaluationDecision, PermissionResolutionError> {
        // Use the integrated policy engine for complex policy evaluation
        self.policy_engine.evaluate(&context).await
            .map_err(|e| PermissionResolutionError::ResolutionFailed(e.to_string()))
    }
    
    /// Get the policy engine for direct access
    pub fn policy_engine(&self) -> &PolicyEngine {
        &self.policy_engine
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

// Re-export for convenience and backward compatibility
pub use self::UnifiedPermissionService as AuthorizationService;

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