// Simplified OpenID Connect (OIDC) System
// Clean OIDC implementation with basic discovery, authorization, and token endpoints

// Core OIDC modules
pub mod routes;
pub mod discovery;
pub mod types;
pub mod authorization;
pub mod token;
pub mod token_exchange; // Firebase ID token → OIDC token exchange

// Standard OpenID Connect endpoints (RFC compliance)
pub mod revocation;
pub mod introspection;
pub mod session;

// Re-exports for clean interface
pub use routes::*;
pub use discovery::*;
pub use types::{AuthorizationRequest, AuthorizationResponse, UserInfoResponse}; // Specific types
pub use authorization::*;
pub use token::{oidc_token, oidc_userinfo, TokenErrorResponse, JsonForm}; // Use token.rs implementations
pub use token_exchange::exchange_firebase_token; // Firebase token exchange