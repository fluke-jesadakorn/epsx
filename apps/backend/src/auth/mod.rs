// WEB3-FIRST AUTHENTICATION MODULE
// Clean, focused authentication system using Web3 wallet signatures

// ============================================================================
// UNIFIED PERMISSION SYSTEM - SINGLE SOURCE OF TRUTH
// ============================================================================
// This is the ONLY permission system in use. All legacy systems removed.

// UNIFIED PERMISSION SERVICE (DATABASE-BACKED - SINGLE SOURCE OF TRUTH)
pub mod unified_permission_service;

// UNIFIED WEB3 AUTHENTICATION (CURRENT - SINGLE SOURCE OF TRUTH)
pub mod auth_service;

// OPENID CONNECT INTEGRATION WITH WEB3
pub mod token_service;

// PERFORMANCE OPTIMIZATIONS
pub mod cache;

// LOCAL DEVELOPMENT SUPPORT

// CORE AUTH MODULES (Web3-First)
pub mod key_manager;
pub mod permissions;
pub mod granular_permissions;
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

// OPENID CONNECT TOKEN SERVICE (WEB3 + OPENID HYBRID)
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

pub use cleanup::{TokenCleanupService, CleanupConfig, CleanupResult, CleanupError, start_cleanup_service, manual_cleanup, get_cleanup_stats};


// ============================================================================
// TEST MODULES (only included in test builds)
// ============================================================================

#[cfg(test)]
pub mod tests;
