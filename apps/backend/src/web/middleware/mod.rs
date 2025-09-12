// Web layer middleware implementations

// Security headers and enhanced monitoring
pub mod security_headers;

// Stateless authentication with RS256 JWT and granular permissions
pub mod stateless_auth;

// Modern Auth.js v5 middleware (replaces Casbin)
pub mod modern_auth;

// Clean authentication with granular permissions
pub mod clean_auth;

// Separated authentication middleware
pub mod admin_auth;
pub mod user_auth;

// Contextual middleware for different access patterns
pub mod contextual_middleware;

// Core middleware modules
pub mod rate_limiter;
pub mod error_handling;
pub mod auth_monitoring;
pub mod policy_validator;

// Security headers and monitoring exports
pub use security_headers::{
  security_headers_middleware,
  csp_middleware,
  request_id_middleware,
  performance_headers_middleware,
  enhanced_cors_middleware,
  enhanced_security_monitoring_middleware,
};

// Modern middleware exports
pub use modern_auth::{
  modern_jwt_auth_middleware,
  cors_middleware,
  request_logging_middleware,
  AuthCtx,
};

// Clean auth exports
pub use clean_auth::{
  clean_auth_middleware,
  require_permission,
  AuthenticatedUser,
  PlatformContext,
};

// Separated auth middleware exports
pub use admin_auth::{
  admin_auth_middleware,
  require_admin_permission_middleware,
  AuthenticatedAdmin,
  AdminPlatformContext,
  AdminSecurityInfo,
};

pub use user_auth::{
  user_auth_middleware,
  require_user_permission_middleware,
  AuthenticatedUser as AuthenticatedUserV2,
  UserPlatformContext,
  UserCacheInfo,
};

pub use rate_limiter::{
  UnifiedRateLimiter,
  RateLimiter,
  RateLimitConfig,
  RateLimitResult,
  RateLimitError,
  ClientId,
  RateLimitStatus,
};

// Contextual middleware exports for different access patterns
pub use contextual_middleware::{
  internal_middleware_stack,
  external_middleware_stack,
  admin_middleware_stack,
  ResourceTracker,
};
pub use error_handling::{
  error_handling_middleware,
  error_recovery_middleware,
  app_error_to_response,
  extract_error_context_from_request,
  ErrorCircuitBreaker,
  ErrorResponseFormat,
};

// Stateless auth exports
pub use stateless_auth::{
  stateless_auth_middleware,
  AuthenticationError,
};
