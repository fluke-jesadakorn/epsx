// Authentication Bounded Context
// Handles identity verification, token management, and session security
// Follows OIDC standards while maintaining clean domain boundaries

pub mod aggregates;
pub mod value_objects;
pub mod domain_services;
pub mod ports;
pub mod events;
pub mod repositories;

// Re-export domain concepts
pub use aggregates::*;
pub use value_objects::*;
pub use domain_services::*;
pub use ports::*;
pub use events::*;
pub use repositories::*;

/// Authentication bounded context business rules and invariants
pub struct AuthenticationBoundedContext;

impl AuthenticationBoundedContext {
    /// Core authentication business rules
    pub const MAX_TOKEN_LIFETIME_HOURS: u32 = 24;
    pub const MAX_REFRESH_TOKEN_LIFETIME_DAYS: u32 = 30;
    pub const MAX_FAILED_ATTEMPTS: u32 = 5;
    pub const SESSION_TIMEOUT_HOURS: u32 = 8;
    
    /// OIDC compliance requirements
    pub const REQUIRED_SCOPES: &'static [&'static str] = &["openid", "profile", "email"];
    pub const SUPPORTED_GRANT_TYPES: &'static [&'static str] = &["authorization_code", "refresh_token"];
    pub const SUPPORTED_RESPONSE_TYPES: &'static [&'static str] = &["code"];
}