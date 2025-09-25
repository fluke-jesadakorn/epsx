// Authentication Bounded Context
// Handles identity verification, token management, and session security
// Follows OIDC standards while maintaining clean domain boundaries

pub mod aggregates;
pub mod value_objects;
pub mod domain_services;
pub mod services;
pub mod ports;
pub mod events;
pub mod repositories;

// Re-export domain concepts with explicit imports to avoid conflicts
pub use aggregates::{AuthenticationSession, AuthenticationError, TerminationReason};
pub use value_objects::{
    // Core authentication types
    AuthenticatedUserId, Scope, ScopeSet, ScopeError,
    // Client and session types
    ClientId, ClientType, RedirectUri, SessionId,
    // Authentication provider types
    AuthenticationProvider, ProviderType, AuthenticationMethod,
    // Token types
    AccessToken, RefreshToken, IdToken, TokenError,
    SecureAccessToken, SecureAccessTokenClaims,
    // Security types
    ClientInformation, SecurityContext, RiskLevel, SecurityFlag
};
pub use services::{
    ExternalAuthService, InternalAuthService, SecureRefreshService,
    // ThreatDetectionService, // Removed - service deleted
    Web3AuthService, Web3PermissionService,
    Web3Challenge, Web3VerificationRequest, Web3AuthResult, Web3AuthError,
    Web3Permission, Web3PermissionType, PermissionVerificationRequest,
    PermissionVerificationResult, Web3PermissionError, VerificationMethod,
    Web3ChallengeRepositoryPort, Web3UserRepositoryPort, Web3PermissionRepositoryPort,
    BlockchainServicePort, NftBalance, TokenBalance, DaoMembership
};
pub use events::{
    AuthenticationSessionCreatedEvent, TokensIssuedEvent, TokensRefreshedEvent,
    AuthenticationSessionTerminatedEvent, SuspiciousActivityDetectedEvent
};
pub use repositories::{
    // Repository ports
    AuthenticationSessionRepositoryPort,
    // Service ports  
    TokenValidationServicePort, UserIdentityServicePort, SecurityMonitoringServicePort,
    // Supporting types
    TokenClaims, TokenIntrospectionResult, UserProfile, UserSubscription, 
    SubscriptionType, RiskScore, SecuritySummary, SecurityEvent, SecurityEventType
};

/// Authentication bounded context business rules and invariants

/// Authentication provider error types
#[derive(Debug, thiserror::Error)]
pub enum AuthProviderError {
    #[error("Provider validation failed: {message}")]
    ValidationFailed { message: String },
    
    #[error("Provider connection failed: {provider}")]
    ConnectionFailed { provider: String },
    
    #[error("Token validation failed")]
    TokenValidationFailed,
    
    #[error("Provider not supported: {provider}")]
    UnsupportedProvider { provider: String },
}
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