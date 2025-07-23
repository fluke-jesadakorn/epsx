// Web layer middleware implementations

pub mod auth_middleware;

pub use auth_middleware::{auth_middleware, require_permission, AuthCtx, AuthenticatedRequest};