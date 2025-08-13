// Web layer middleware implementations

// Modern Auth.js v5 middleware (replaces Casbin)
pub mod modern_auth;

// Legacy middleware (will be removed in Phase 6)
pub mod permission_middleware;
pub mod module_auth_middleware;
pub mod module_permission_middleware;
pub mod rate_limiter;
pub mod error_handling;
// Casbin middleware removed - using modern JWT auth
pub mod auth_monitoring;
pub mod policy_validator;

// Modern middleware exports
pub use modern_auth::{
    modern_jwt_auth_middleware,
    cors_middleware,
    request_logging_middleware
};

// Legacy middleware exports (will be removed in Phase 6)
pub use permission_middleware::{permission_middleware};
pub use module_auth_middleware::{module_auth_casbin_middleware, ModuleAuthCtx, ModuleAccess, AccessLevel, UserModuleAccess, ApiKeyAccess};

// Legacy auth_middleware and module_auth_middleware removed - use casbin_auth::casbin_auth_middleware
pub use module_permission_middleware::{module_casbin_middleware as module_permission_middleware};
pub use rate_limiter::{UnifiedRateLimiter, InMemoryRateLimiter, RateLimiter, RateLimitConfig, RateLimitResult, RateLimitError, ClientId, RateLimitStatus};
pub use error_handling::{
    error_handling_middleware, 
    error_recovery_middleware,
    app_error_to_response,
    extract_error_context_from_request,
    ErrorCircuitBreaker,
    ErrorResponseFormat
};
// Casbin auth exports removed - using modern JWT auth