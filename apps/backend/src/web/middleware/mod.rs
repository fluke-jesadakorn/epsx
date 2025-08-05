// Web layer middleware implementations

pub mod permission_middleware;
pub mod module_auth_middleware;
pub mod module_permission_middleware;
pub mod rate_limiter;
pub mod error_handling;
pub mod casbin_cache;
pub mod auth_monitoring;
pub mod policy_validator;
pub mod casbin_error_handler;

pub use permission_middleware::{permission_middleware};
pub use module_auth_middleware::{module_auth_casbin_middleware, module_auth_middleware, ModuleAuthCtx, ModuleAccess, AccessLevel, AuthCtx, UserModuleAccess, ApiKeyAccess};

// Re-export auth_middleware from module_auth_middleware for backward compatibility
pub use module_auth_middleware::module_auth_middleware as auth_middleware;
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