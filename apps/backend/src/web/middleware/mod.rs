// Web layer middleware implementations

// Security headers and enhanced monitoring
pub mod security_headers;

// Modern Auth.js v5 middleware (replaces Casbin)
pub mod modern_auth;

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
  add_deprecation_headers,
};

// Modern middleware exports
pub use modern_auth::{
  modern_jwt_auth_middleware,
  cors_middleware,
  request_logging_middleware,
  AuthCtx,
};

// All legacy permission middleware removed - replaced by simple roles
pub use rate_limiter::{
  UnifiedRateLimiter,
  RateLimiter,
  RateLimitConfig,
  RateLimitResult,
  RateLimitError,
  ClientId,
  RateLimitStatus,
};
pub use error_handling::{
  error_handling_middleware,
  error_recovery_middleware,
  app_error_to_response,
  extract_error_context_from_request,
  ErrorCircuitBreaker,
  ErrorResponseFormat,
};
