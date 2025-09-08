// Unified Authentication module

// Core authentication components
pub mod routes;
pub mod password;
pub mod providers;
pub mod token_broker;
pub mod modern_routes;

// Main exports
pub use routes::AppState;
pub use password::{PasswordValidator, PasswordHasher, PasswordError, PasswordStrength};
// API key service moved to crate::infrastructure::adapters::services::api_key_service
pub use providers::*;
pub use token_broker::*;