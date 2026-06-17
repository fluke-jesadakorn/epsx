// Web layer middleware implementations - Web3-first and minimal
// Phase 7.2: Pure Web3 authentication only, OIDC/JWT removed

// Essential security middleware (core functionality)
// Moved to `epsx-web-middleware` in wave 10 prep (no backend coupling).
// Re-exported here so existing call sites keep working.
pub use epsx_web_middleware::security_headers::{
    security_headers_middleware,
    request_id_middleware,
    RequestId,
};

// Web3 wallet authentication middleware (pure SIWE - improved implementation)
pub mod auth_middleware;

// OpenID Connect Bearer token authentication middleware
pub mod bearer_middleware;

// Rate limiting for API protection
pub mod governor_limiter;
pub mod rate_limiter;
pub mod rate_limit_middleware;
pub mod multi_level_rate_limiter;
pub mod usage_tracking_middleware;

// CRITICAL: Bulletproof Permission Validation Middleware (Phase 1.2)
// THE SINGLE SOURCE OF TRUTH for all permission enforcement
pub mod permission_validation_middleware;

// Web3 auth exports (improved wallet-first authentication)
pub use auth_middleware::{
    web3_auth_middleware,
    Web3AuthContext,
    Web3AuthError,
    get_web3_context,
    require_web3_auth,
    require_permission,
    require_admin,
    has_any_permission,
    AuthMethod,
};

// OpenID Bearer auth exports (standard OpenID Connect)
pub use bearer_middleware::{
  bearer_middleware,
  optional_bearer_middleware,
  OpenIDUserContext,
  UnifiedErrorResponse,
  ErrorDetails,
  extract_user_context,
  require_user_context,
  check_user_permission,
  create_permission_denied_error,
};

// Rate limiter exports (used in validation)
pub use rate_limiter::{
  UnifiedRateLimiter,
  RateLimitConfig,
  ClientId,
};

// Web3 rate limiting exports (used for Web3 API protection)
pub use rate_limit_middleware::{
  web3_rate_limit_middleware,
  unified_rate_limit_middleware,
};

// CRITICAL: Permission validation exports (THE SINGLE SOURCE OF TRUTH)
pub use permission_validation_middleware::{
  permission_validation_middleware,
  perm_guard,
};

// Multi-level rate limiter exports (3-tier rate limiting)
pub use multi_level_rate_limiter::{
    MultiLevelRateLimiter,
    MultiLevelRateLimitResult,
    GlobalRateLimitConfig,
    PlanRateLimits,
    RateLimitLevel,
};

// Usage tracking exports
pub use usage_tracking_middleware::usage_tracking_middleware;