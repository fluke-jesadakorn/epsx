// Authentication Value Objects
// Immutable domain concepts with validation and business rules

pub mod session_id;
pub mod tokens;
pub mod secure_tokens;
pub mod client_information;
pub mod security_context;
pub mod authentication_provider;
pub mod scopes;

// Token value objects
pub use tokens::{AccessToken, RefreshToken, IdToken, TokenError};
pub use secure_tokens::{SecureAccessToken, SecureAccessTokenClaims};

// Identity and session value objects  
pub use session_id::{SessionId, SessionIdError};
pub use client_information::{ClientInformation, ClientId, ClientType, RedirectUri};
pub use security_context::{SecurityContext, RiskLevel, SecurityFlag};
pub use authentication_provider::{AuthenticationProvider, ProviderType, AuthenticationMethod};
pub use scopes::{Scope, ScopeSet, ScopeError};

// User identity in authentication context
use crate::domain::shared_kernel::value_objects::UserId;

/// Authenticated user identifier - wraps UserId with authentication context
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AuthenticatedUserId(UserId);

impl AuthenticatedUserId {
    /// Create authenticated user ID from verified user
    pub fn from_verified_user(user_id: UserId) -> Self {
        Self(user_id)
    }
    
    /// Get the underlying user ID
    pub fn user_id(&self) -> &UserId {
        &self.0
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl std::fmt::Display for AuthenticatedUserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "auth:{}", self.0)
    }
}

impl From<UserId> for AuthenticatedUserId {
    fn from(user_id: UserId) -> Self {
        Self::from_verified_user(user_id)
    }
}