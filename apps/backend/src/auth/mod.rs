// NEW UNIFIED AUTH SERVICES (Phase 2 Refactoring)
pub mod authentication_service;
pub mod authorization_service;
pub mod session_service;

// SECURE JWT SERVICE - RS256 ONLY (Security Critical)
pub mod secure_jwt_service;

// WEB3 AUTHENTICATION MODULES (New)
pub mod web3_auth_service;
pub mod web3_permission_service;
pub mod multi_chain_cache_service;

// UNIFIED AUTHENTICATION SERVICE (Web3-First)
pub mod unified_auth_service;
// Migration service removed - Web3-first approach eliminates need for Firebase migration
// pub mod migration_service;
pub mod unified_permission_service;
pub mod enhanced_bearer_token_service;

// PERFORMANCE OPTIMIZATIONS
// pub mod auth_performance_cache; // TODO: Fix compilation issues
// pub mod optimized_unified_auth_service; // TODO: Fix compilation issues  
pub mod simplified_auth_cache;

// LEGACY MODULES (to be phased out gradually)
pub mod jwt;
// Firebase flow module removed - replaced by Web3-first authentication
pub mod tokens;
pub mod key_manager;
pub mod permissions; // New permission-only system
pub mod granular_permissions; // Clean granular permission system
pub mod hierarchy_resolver; // Permission hierarchy resolution
pub mod policy_engine; // Dynamic policy evaluation engine
pub mod revocation;
pub mod refresh_tokens;
pub mod refresh_token_service;
pub mod scopes;
pub mod cleanup;
pub mod session_cleanup_service;
pub mod session_security_service;
// pub mod roles; // Removed - using permissions-based system only

// Separated JWT systems
pub mod admin_jwt;
pub mod user_jwt;


// NEW UNIFIED SERVICE EXPORTS (Phase 2 Refactoring)
pub use authentication_service::{AuthenticationService, AuthClaims, AuthResult};
pub use authorization_service::{AuthorizationService, Permission as NewPermission, AuthContext, AuthDecision, PermissionStats};
pub use session_service::{SessionService, SessionInfo, RefreshToken, CreateSessionRequest, SessionStats, SecurityEvent, SecurityEventType, DeviceInfo};

// SECURE JWT SERVICE EXPORTS (Security Critical - RS256 Only)
pub use secure_jwt_service::{
    SecureJWTService, SecureJWTClaims, TokenRequest, TokenValidationResult, JWTSecurityError
};

// WEB3 AUTHENTICATION EXPORTS (New)
pub use web3_auth_service::{Web3AuthService, AuthChallenge, VerifyRequest as Web3VerifyRequest, AuthResult as Web3AuthResult};
pub use web3_permission_service::{
    Web3PermissionService, PermissionInfo as Web3PermissionInfo, NFTConfig, TokenConfig, 
    DAOProposal, PermissionVerificationResult
};
pub use multi_chain_cache_service::{
    MultiChainCacheService, NetworkCacheConfig, CachedVerification, CacheStatistics,
    NetworkStats, CacheHealthStatus, CacheWarmingService
};

// UNIFIED AUTHENTICATION SERVICE EXPORTS (Web3-First)
pub use unified_auth_service::{
    UnifiedAuthService, AuthMethod, AuthChallenge as UnifiedAuthChallenge, 
    AuthChallengeData, UnifiedVerifyRequest, VerificationData, UnifiedAuthResult,
    UserProfile, AuthContext as UnifiedAuthContext
};

// MIGRATION SERVICE EXPORTS - REMOVED (Web3-First Migration)
// Migration service removed - new users should register directly with Web3 wallets
// pub use migration_service::{
//     MigrationService, MigrationStatus, MigrationResult, MigrationPlan,
//     DataPreservationPlan, BatchMigrationRequest, SingleUserMigration, MigrationStats
// };

// UNIFIED PERMISSION SERVICE EXPORTS
pub use unified_permission_service::{
    UnifiedPermissionService, UnifiedPermission, PermissionSource as UnifiedPermissionSource, PermissionCheck,
    PermissionResult, BulkPermissionCheck, BulkPermissionResult, AccessLevel,
    GrantPermissionRequest, PermissionStats as UnifiedPermissionStats
};

// ENHANCED BEARER TOKEN SERVICE EXPORTS
pub use enhanced_bearer_token_service::{
    EnhancedBearerTokenService, EnhancedTokenClaims, Web3PermissionClaim, NftHolding,
    TokenBalance, DaoMembership, RateLimitTier, CacheHints as EnhancedCacheHints, ApiKeyRequest, ApiKey,
    TokenValidation, EnterpriseTeam, TeamMember, TeamRole
};

// PERFORMANCE OPTIMIZATION EXPORTS
// TODO: Re-enable when compilation issues are fixed
// pub use auth_performance_cache::{AuthPerformanceCache, AuthCacheConfig};
// pub use optimized_unified_auth_service::{OptimizedUnifiedAuthService, AuthPerformanceMonitor};

pub use simplified_auth_cache::{
    SimplifiedAuthCache, SimplifiedCacheConfig, SimplifiedCacheStats,
    PermissionCacheEntry as SimplifiedPermissionCacheEntry, 
    ChallengeCacheEntry as SimplifiedChallengeCacheEntry
};

// LEGACY EXPORTS (to be phased out gradually)
pub use jwt::{Service as JWTService, Claims, User, Error as JWTError, UserData};
pub use key_manager::KeyManager;
// Firebase flow exports removed - functionality replaced by Web3-first auth
pub use tokens::{TokenRequest as LegacyTokenRequest, TokenResponse, ErrorResponse as TokenError};
pub use revocation::{TokenRevocationService, RevokedToken, RevocationError, TOKEN_REVOCATION_SERVICE};
pub use refresh_tokens::{RefreshTokenData, RefreshTokenRotation, RefreshTokenError, REFRESH_TOKEN_SERVICE};
// Disabled to avoid conflicts with new session service
// pub use refresh_token_service::{RefreshTokenService, RefreshTokenConfig, DeviceInfo, CreateRefreshTokenRequest, RefreshTokenResponse};
pub use scopes::{ScopeService, Scope, ValidatedScopes, ScopeError, SCOPE_SERVICE};
pub use cleanup::{TokenCleanupService, CleanupConfig, CleanupResult, CleanupError, start_cleanup_service, manual_cleanup, get_cleanup_stats};
pub use session_cleanup_service::{SessionCleanupService, SessionCleanupConfig, CleanupStats, CleanupHealthStatus, init_global_cleanup_service, start_global_cleanup_service, run_manual_cleanup, get_cleanup_health};
// Disabled to avoid conflicts with new session service
// pub use session_security_service::{SessionSecurityService, SessionSecurityConfig, DeviceFingerprint, GeoLocation, SecurityEvent, SecurityEventType, SecurityAnalysisResult, UserSessionInfo};
// Role-based exports removed - using permissions-based system only
pub use permissions::{Permission, UserClaims, check_permission_access, PermissionError, require_permission_pure, PermissionSets};
pub use granular_permissions::{
    GranularPermissionClaim, PermissionSource, GranularPermissionSet, 
    PermissionValidationResult, ValidationContext, GranularPermissionError
};
pub use hierarchy_resolver::{
    HierarchyResolver, PermissionHierarchy, InheritanceType, HierarchyResolution, 
    InheritanceChain, PermissionCache, HierarchyStats
};
pub use policy_engine::{
    PolicyEngine, DynamicPolicy, PolicyCondition, PolicyAction, PolicyDecision,
    PolicyEvaluationContext, PolicyTemplate, PolicyEvaluationResult
};

// Separated JWT exports
pub use admin_jwt::{
    AdminJWTService, AdminJWTClaims, AdminValidationResult, AdminSecurityContext,
    AdminPermissionMatrix, PrivilegedOperationContext, RiskAssessment
};
pub use user_jwt::{
    UserJWTService, UserJWTClaims, UserValidationResult, UserContext,
    UserPermissionSet, UserSubscription, CacheHints
};

// Create simplified global JWT service
lazy_static::lazy_static! {
    pub static ref JWT: jwt::Service = jwt::Service::new()
        .expect("Failed to initialize JWT service");
}

