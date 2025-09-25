// Web layer middleware implementations - Optimized and minimal
// Phase 10: Reduced from 13 middleware files to 3 essential ones

// Essential security middleware (core functionality)
pub mod security_headers;

// Authentication middleware (actively used)
pub mod stateless_auth;

// Web3 wallet authentication middleware (replaces OIDC/JWT)
pub mod web3_auth_middleware;

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

// Stateless auth exports (legacy - will be removed)
pub use stateless_auth::{
  stateless_auth_middleware,
  AuthenticationError,
};

// Web3 auth exports (new wallet-first authentication)
pub use web3_auth_middleware::{
  web3_auth_middleware,
  Web3AuthContext,
  Web3AuthError,
  get_web3_context,
  require_web3_auth,
  require_permission,
  require_admin,
  require_group,
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