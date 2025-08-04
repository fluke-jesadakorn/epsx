// Unified Authentication module

pub mod handlers;
pub mod routes;
pub mod password;
pub mod api_key_service;

pub use handlers::*;

// Re-export handlers as multi_handlers for backward compatibility
pub mod multi_handlers {
    pub use super::handlers::*;
}
pub use routes::AppState;
pub use password::{PasswordValidator, PasswordHasher, PasswordError, PasswordStrength};
pub use api_key_service::{ApiKeyService, ApiKeyError};