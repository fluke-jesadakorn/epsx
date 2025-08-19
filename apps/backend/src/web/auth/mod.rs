// Unified Authentication module

// Core authentication components
pub mod handlers;
pub mod routes;
pub mod password;
pub mod api_key_service;
pub mod providers;
pub mod token_broker;
pub mod modern_routes;

// Main exports
pub use handlers::*;
pub use routes::AppState;
pub use password::{PasswordValidator, PasswordHasher, PasswordError, PasswordStrength};
pub use api_key_service::{ApiKeyService, ApiKeyError};
pub use providers::*;
pub use token_broker::*;