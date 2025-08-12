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
pub mod casbin_auth;

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
pub use casbin_auth::{
    casbin_auth_middleware,
    casbin_auth_middleware_with_config,
    require_role,
    require_permission,
    CasbinAuthConfig
};