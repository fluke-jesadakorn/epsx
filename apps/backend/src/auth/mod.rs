// WEB3-FIRST AUTHENTICATION MODULE
// Clean, focused authentication system using Web3 wallet signatures

// ============================================================================
// UNIFIED PERMISSION SYSTEM - SINGLE SOURCE OF TRUTH
// ============================================================================
// This is the ONLY permission system in use. All legacy systems removed.

// UNIFIED PERMISSION SERVICE (NEW - DATABASE-BACKED)
pub mod unified_permission_service;

// UNIFIED WEB3 AUTHENTICATION (CURRENT - SINGLE SOURCE OF TRUTH)
pub mod auth_service;

// OPENID CONNECT INTEGRATION WITH WEB3
pub mod token_service;

// PERFORMANCE OPTIMIZATIONS
pub mod cache;

// LOCAL DEVELOPMENT SUPPORT

// ============================================================================
// LEGACY SYSTEMS (DEPRECATED - MIGRATION IN PROGRESS)
// ============================================================================
// These modules are DEPRECATED and scheduled for removal.
// Current dependencies still exist - do not delete until fully migrated.
//
// Migration paths:
// - permission_authority → Use unified_permission_service
// - permission_registry → Use middleware's is_public_route_for_auth()
// - route_protection → Use auth_middleware's require_permission()

#[deprecated(note = "Use unified_permission_service instead")]
pub mod permission_authority;
#[deprecated(note = "Route permissions handled by middleware")]
pub mod permission_registry;
#[deprecated(note = "Use auth_middleware::require_permission instead")]
pub mod route_protection;

// CORE AUTH MODULES (Web3-First)
pub mod key_manager;
pub mod permissions;
pub mod granular_permissions;
pub mod hierarchy_resolver;
pub mod policy_engine;
pub mod cleanup;
pub mod types;

// ============================================================================
// EXPORTS - UNIFIED PERMISSION SYSTEM
// ============================================================================

// UNIFIED PERMISSION SERVICE (PRIMARY - DATABASE-BACKED)
pub use unified_permission_service::{
    UnifiedPermissionService, PermissionDetail, PermissionSource as UnifiedPermissionSource,
    PermissionStats as UnifiedPermissionStats, GrantPermissionRequest, RevokePermissionRequest,
    AssignGroupRequest, RemoveGroupRequest
};

// WEB3-FIRST AUTH EXPORTS

// UNIFIED WEB3 AUTHENTICATION SERVICE (CURRENT - SINGLE SOURCE OF TRUTH)
pub use auth_service::{
    UnifiedWeb3AuthService, Web3Challenge, Web3VerificationRequest, Web3AuthResult,
    Web3Permission, Web3PermissionType, Web3AuthError
};

// Token validation uses OpenIDTokenService::validate_access_token() as SINGLE SOURCE OF TRUTH

// DEPRECATED EXPORTS (kept for backward compatibility - will be removed)
#[allow(deprecated)]
pub use permission_authority::{
    CentralizedPermissionAuthority, PermissionValidator, RoutePermissionResolver,
    PermissionResult, BulkPermissionResult, Permission, PermissionSource,
    ValidationContext, CacheConfig, create_permission_authority, create_high_performance_authority
};

#[allow(deprecated)]
pub use permission_registry::{
    DatabasePermissionRegistry, RoutePermissionMapping, RegisterRoutePermissionRequest,
    RouteResolution, create_permission_registry, create_high_performance_registry,
    get_default_route_permissions
};

#[allow(deprecated)]
pub use route_protection::{
    RequirePermission, PermissionGuard, PermissionMiddlewareBuilder, PermissionState,
    RouteValidationResult, HandlerPermissionExt, require_permission, require_admin,
    require_admin_permission
};

// OPENID CONNECT TOKEN SERVICE (NEW - WEB3 + OPENID HYBRID)
pub use token_service::{
    OpenIDTokenService, OpenIDTokenResponse, AccessTokenClaims, IdTokenClaims,
    Web3AuthTokenRequest, OpenIDTokenError, RefreshTokenInfo
};


// PERFORMANCE OPTIMIZATION EXPORTS
pub use cache::{
    SimplifiedAuthCache, SimplifiedCacheConfig, SimplifiedCacheStats,
    PermissionCacheEntry as SimplifiedPermissionCacheEntry,
    ChallengeCacheEntry as SimplifiedChallengeCacheEntry
};

// CORE AUTH EXPORTS (Web3-First)
pub use key_manager::KeyManager;
pub use permissions::{Permission as LegacyPermission, UserClaims, check_permission_access, PermissionError, require_permission_pure, PermissionSets};
pub use granular_permissions::{
    GranularPermissionClaim, PermissionSource as GranularPermissionSource, GranularPermissionSet, 
    PermissionValidationResult, ValidationContext as GranularValidationContext, GranularPermissionError
};
pub use hierarchy_resolver::{
    HierarchyResolver, PermissionHierarchy, InheritanceType, HierarchyResolution, 
    InheritanceChain, PermissionCache as HierarchyPermissionCache, HierarchyStats
};
pub use policy_engine::{
    PolicyEngine, DynamicPolicy, PolicyCondition, PolicyAction, PolicyDecision,
    PolicyEvaluationContext, PolicyTemplate, PolicyEvaluationResult
};

pub use cleanup::{TokenCleanupService, CleanupConfig, CleanupResult, CleanupError, start_cleanup_service, manual_cleanup, get_cleanup_stats};


// ============================================================================
// TEST MODULES (only included in test builds)
// ============================================================================

#[cfg(test)]
pub mod tests;
