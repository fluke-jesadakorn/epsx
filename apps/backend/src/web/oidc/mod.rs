// Simplified OpenID Connect (OIDC) System
// Clean OIDC implementation with basic discovery, authorization, and token endpoints

// Core OIDC modules
pub mod routes;
pub mod discovery;
pub mod types;
pub mod authorization;
pub mod token;

// Standard OpenID Connect endpoints (RFC compliance)
pub mod revocation;
pub mod introspection;
pub mod session;

// Refactored token service modules
pub mod token_generator;
pub mod token_validator;
pub mod refresh_handler;
pub mod introspection_service;
pub mod claims_processor;
pub mod crypto_manager;

// Re-exports for clean interface
pub use routes::*;
pub use discovery::*;
pub use types::{AuthorizationRequest, AuthorizationResponse, UserInfoResponse}; // Specific types
pub use authorization::*;
pub use token::{oidc_token, oidc_userinfo, TokenErrorResponse, JsonForm}; // Use token.rs implementations

// Re-export refactored modules for external use
pub use token_generator::TokenGenerator;
pub use token_validator::TokenValidator;
pub use refresh_handler::RefreshHandler;
pub use introspection_service::{IntrospectionService, oidc_introspect};
pub use claims_processor::ClaimsProcessor;
pub use crypto_manager::CryptoManager;