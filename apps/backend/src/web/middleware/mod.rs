// Web layer middleware implementations

// Security headers and enhanced monitoring
pub mod security_headers;

// Stateless authentication with RS256 JWT and granular permissions
pub mod stateless_auth;

// Clean authentication with granular permissions
pub mod clean_auth;

// Enhanced security validation middleware (RS256-only, comprehensive security)
pub mod enhanced_security_middleware;


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

// Clean auth exports
pub use clean_auth::{
  clean_auth_middleware,
  require_permission,
  AuthenticatedUser,
  PlatformContext,
};

// Enhanced security middleware exports
pub use enhanced_security_middleware::{
  enhanced_security_middleware,
  SecurityContext,
  SecurityMiddlewareConfig,
  SecurityEvent,
  extract_security_context,
  check_permission,
  create_secure_config,
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
