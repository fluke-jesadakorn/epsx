// Unified Authentication module

pub mod handlers;
pub mod routes;
pub mod password;
pub mod api_key_service;
pub mod providers;
pub mod token_broker;
pub mod casbin_claims_mapper;

pub use handlers::*;
pub use providers::*;
pub use token_broker::*;
pub use casbin_claims_mapper::*;

// Re-export handlers as multi_handlers for backward compatibility
pub mod multi_handlers {
    pub use super::handlers::*;
}
pub use routes::AppState;
pub use password::{PasswordValidator, PasswordHasher, PasswordError, PasswordStrength};
pub use api_key_service::{ApiKeyService, ApiKeyError};