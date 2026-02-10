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
pub mod granular_permissions;

// ============================================================================
// EXPORTS - UNIFIED PERMISSION SYSTEM
// ============================================================================

// UNIFIED PERMISSION SERVICE (PRIMARY - DATABASE-BACKED)
pub use unified_permission_service::{
    UnifiedPermissionService, PermissionDetail, PermissionSource as UnifiedPermissionSource,
    PermissionStats as UnifiedPermissionStats, GrantPermissionRequest, RevokePermissionRequest,
    AssignPlanRequest, RemovePlanRequest
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
pub use granular_permissions::{
    GranularPermissionClaim, PermissionSource as GranularPermissionSource, GranularPermissionSet, 
    PermissionValidationResult, ValidationContext as GranularValidationContext, GranularPermissionError
};

// ============================================================================
// TEST MODULES (only included in test builds)
// ============================================================================

#[cfg(test)]
pub mod tests;
