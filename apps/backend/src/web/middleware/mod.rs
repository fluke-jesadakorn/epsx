// Web layer middleware implementations - Web3-first and minimal
// Phase 7.2: Pure Web3 authentication only, OIDC/JWT removed

// Essential security middleware (core functionality)
pub mod security_headers;

// Web3 wallet authentication middleware (pure SIWE)
pub mod web3_auth_middleware;

// OpenID Connect Bearer token authentication middleware
pub mod openid_bearer_auth_middleware;

// Rate limiting for API protection
pub mod rate_limiter;
pub mod rate_limit_middleware;

// ⚡ CRITICAL: Bulletproof Permission Validation Middleware (Phase 1.2)
// THE SINGLE SOURCE OF TRUTH for all permission enforcement
pub mod permission_validation_middleware;

// Security headers exports (only functions actually used)
pub use security_headers::{
  security_headers_middleware,
  request_id_middleware,
};

// Web3 auth exports (pure wallet-first authentication)
pub use web3_auth_middleware::{
  web3_auth_middleware,
  Web3AuthContext,
  Web3AuthError,
  get_web3_context,
  require_web3_auth,
  require_permission,
  require_admin,
};

// OpenID Bearer auth exports (standard OpenID Connect)
pub use openid_bearer_auth_middleware::{
  openid_bearer_auth_middleware,
  optional_openid_bearer_auth_middleware,
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

// ⚡ CRITICAL: Permission validation exports (THE SINGLE SOURCE OF TRUTH)
pub use permission_validation_middleware::{
  permission_validation_middleware,
};