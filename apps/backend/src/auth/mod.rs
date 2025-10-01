// WEB3-FIRST AUTHENTICATION MODULE  
// Clean, focused authentication system using Web3 wallet signatures

// UNIFIED WEB3 AUTHENTICATION (NEW - SINGLE SOURCE OF TRUTH)
pub mod unified_web3_auth_service;
pub mod unified_web3_permission_service;

// CENTRALIZED PERMISSION AUTHORITY (NEW - HIGH PERFORMANCE)
pub mod permission_authority;
pub mod permission_registry;
pub mod route_protection;

// OPENID CONNECT INTEGRATION WITH WEB3 (NEW)
pub mod openid_token_service;


// PERFORMANCE OPTIMIZATIONS
pub mod simplified_auth_cache;

// DYNAMIC GROUP SYSTEM MODULES
pub mod dynamic_group_rules_engine;
pub mod group_template_system;

// CORE AUTH MODULES (Web3-First)
pub mod key_manager;
pub mod permissions;
pub mod granular_permissions;
pub mod hierarchy_resolver;
pub mod policy_engine;
pub mod cleanup;
pub mod web3_shared_types;

// WEB3-FIRST AUTH EXPORTS

// UNIFIED WEB3 AUTHENTICATION SERVICE (NEW - SINGLE SOURCE OF TRUTH)
pub use unified_web3_auth_service::{
    UnifiedWeb3AuthService, Web3Challenge, Web3VerificationRequest, Web3AuthResult,
    Web3Permission, Web3PermissionType, Web3AuthError
};

// UNIFIED WEB3 PERMISSION SERVICE (NEW - WALLET-FIRST SYSTEM)
pub use unified_web3_permission_service::{
    UnifiedWeb3PermissionService, PermissionStats, GroupMembership
};

// CENTRALIZED PERMISSION AUTHORITY (NEW - HIGH PERFORMANCE)
pub use permission_authority::{
    CentralizedPermissionAuthority, PermissionValidator, RoutePermissionResolver,
    PermissionResult, BulkPermissionResult, Permission, PermissionSource,
    ValidationContext, CacheConfig, create_permission_authority, create_high_performance_authority
};

// DATABASE-DRIVEN PERMISSION REGISTRY (NEW - DYNAMIC ROUTE MAPPING)
pub use permission_registry::{
    DatabasePermissionRegistry, RoutePermissionMapping, RegisterRoutePermissionRequest,
    RouteResolution, create_permission_registry, create_high_performance_registry,
    get_default_route_permissions
};

// ROUTE PROTECTION SYSTEM (NEW - DECORATORS AND GUARDS)
pub use route_protection::{
    RequirePermission, PermissionGuard, PermissionMiddlewareBuilder, PermissionState,
    RouteValidationResult, HandlerPermissionExt, require_permission, require_admin,
    require_admin_permission
};

// OPENID CONNECT TOKEN SERVICE (NEW - WEB3 + OPENID HYBRID)
pub use openid_token_service::{
    OpenIDTokenService, OpenIDTokenResponse, AccessTokenClaims, IdTokenClaims,
    Web3AuthTokenRequest, OpenIDTokenError, RefreshTokenInfo
};


// PERFORMANCE OPTIMIZATION EXPORTS
pub use simplified_auth_cache::{
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

// DYNAMIC GROUP SYSTEM EXPORTS
pub use dynamic_group_rules_engine::{
    DynamicGroupRulesEngine, DynamicRule, RuleCondition, LogicOperator, 
    RuleActions, RuleEvaluationResult, ConditionOperator, RuleType,
    UserContext, UserBehavioralData, Web3UserData, EvaluationContext
};
pub use group_template_system::{
    GroupTemplateSystem, GroupTemplate, TemplateCategory, EvaluationFrequency,
    TemplateParameters, ParameterDefinition, ParameterType, ValidationRule
};

// ============================================================================
// TEST MODULES (only included in test builds)
// ============================================================================

#[cfg(test)]
pub mod tests;
