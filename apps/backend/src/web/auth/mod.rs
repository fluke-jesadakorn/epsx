// Unified Authentication module

// Modern Auth.js v5 handlers and routes
pub mod modern_handlers;
pub mod modern_routes;

// Legacy handlers
pub mod handlers;
pub mod routes;
pub mod password;
pub mod api_key_service;
pub mod providers;
pub mod token_broker;

// Registration API endpoints
pub mod registration;

// Modern exports
pub use modern_handlers::*;
pub use modern_routes::*;

// Legacy exports
pub use handlers::*;
pub use providers::*;
pub use token_broker::*;

// Registration exports
pub use registration::{register_user, check_email_availability, RegisterRequest, RegisterResponse};

// Re-export handlers as multi_handlers for backward compatibility
pub mod multi_handlers {
    pub use super::handlers::*;
}
pub use routes::AppState;
pub use password::{PasswordValidator, PasswordHasher, PasswordError, PasswordStrength};
pub use api_key_service::{ApiKeyService, ApiKeyError};