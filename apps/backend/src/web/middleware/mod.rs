// Web layer middleware implementations

pub mod auth_middleware;
pub mod permission_middleware;
pub mod module_auth_middleware;
pub mod module_permission_middleware;
pub mod rate_limiter;

pub use auth_middleware::{auth_middleware, require_permission, AuthCtx, AuthenticatedRequest};
pub use permission_middleware::{permission_middleware};
pub use module_auth_middleware::{module_auth_middleware, ModuleAuthCtx, ModuleAccess, AccessLevel};
pub use module_permission_middleware::{module_permission_middleware};
pub use rate_limiter::{InMemoryRateLimiter, RateLimiter, RateLimitConfig, RateLimitResult};