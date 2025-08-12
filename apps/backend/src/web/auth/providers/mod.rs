// Multi-Provider Authentication System
// Provider abstraction layer for handling different authentication providers

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::dom::values::{UserId, Email, Role};
use crate::core::types::AppError;

pub mod firebase_provider;
pub mod oidc_provider;

/// Authentication provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderType {
    Firebase,
    OIDC,
    Custom,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::Firebase => write!(f, "firebase"),
            ProviderType::OIDC => write!(f, "oidc"),
            ProviderType::Custom => write!(f, "custom"),
        }
    }
}

/// Unified user claims that all providers must return
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    /// Backend user ID (consistent across providers)
    pub user_id: UserId,
    /// User email
    pub email: Email,
    /// User role
    pub role: Role,
    /// User permissions (for Casbin)
    pub permissions: Vec<String>,
    /// Provider-specific user ID (Firebase UID, OIDC sub, etc.)
    pub provider_user_id: String,
    /// Which provider authenticated this user
    pub provider: ProviderType,
    /// Token expiry
    pub expires_at: DateTime<Utc>,
    /// JWT issued at timestamp
    pub iat: u64,
    /// JWT expiry timestamp  
    pub exp: u64,
    /// Subscription tier
    pub subscription_tier: Option<String>,
    /// Additional claims
    pub extra_claims: HashMap<String, serde_json::Value>,
}

impl UserClaims {
    pub fn new(
        user_id: UserId,
        email: Email,
        role: Role,
        permissions: Vec<String>,
        provider_user_id: String,
        provider: ProviderType,
        expires_at: DateTime<Utc>,
        iat: u64,
        exp: u64,
        subscription_tier: Option<String>,
    ) -> Self {
        Self {
            user_id,
            email,
            role,
            permissions,
            provider_user_id,
            provider,
            expires_at,
            iat,
            exp,
            subscription_tier,
            extra_claims: HashMap::new(),
        }
    }

    /// Add extra claim
    pub fn with_claim(mut self, key: String, value: serde_json::Value) -> Self {
        self.extra_claims.insert(key, value);
        self
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Token pair for refresh flows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub token_type: String,
}

/// Authentication errors specific to providers
#[derive(Debug, thiserror::Error)]
pub enum AuthProviderError {
    #[error("Token validation failed: {0}")]
    TokenValidationFailed(String),
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Provider configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Invalid token format")]
    InvalidTokenFormat,
    
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Provider internal error: {0}")]
    InternalError(String),
}

impl From<AuthProviderError> for AppError {
    fn from(err: AuthProviderError) -> Self {
        match err {
            AuthProviderError::TokenExpired => AppError::unauthorized("Token expired"),
            AuthProviderError::InvalidToken => AppError::unauthorized("Invalid token"),
            AuthProviderError::InvalidTokenFormat => AppError::bad_request("Invalid token format"),
            AuthProviderError::PermissionDenied => AppError::forbidden("Permission denied"),
            AuthProviderError::UserNotFound => AppError::bad_request("User not found"),
            AuthProviderError::ConfigurationError(msg) => AppError::internal_error(&format!("Configuration error: {}", msg)),
            AuthProviderError::NetworkError(msg) => AppError::internal_error(&format!("Network error: {}", msg)),
            AuthProviderError::TokenValidationFailed(msg) => AppError::unauthorized(&format!("Token validation failed: {}", msg)),
            AuthProviderError::InternalError(msg) => AppError::internal_error(&format!("Provider error: {}", msg)),
        }
    }
}

/// Core authentication provider trait
/// All authentication providers must implement this trait
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Validate a token and return user claims
    async fn validate_token(&self, token: &str) -> Result<UserClaims, AuthProviderError>;
    
    /// Refresh an access token using a refresh token
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair, AuthProviderError>;
    
    /// Get provider name for logging and debugging
    fn provider_name(&self) -> &'static str;
    
    /// Get provider type
    fn provider_type(&self) -> ProviderType;
    
    /// Provider priority for conflict resolution (higher = more priority)
    /// Firebase: 100, OIDC: 90, Custom: 80
    fn priority(&self) -> u8;
    
    /// Check if this provider can handle the given token format
    fn can_handle_token(&self, token: &str) -> bool;
    
    /// Optional: Get user info from provider (for admin purposes)
    async fn get_user_info(&self, _user_id: &str) -> Result<serde_json::Value, AuthProviderError> {
        Err(AuthProviderError::InternalError("Not implemented".to_string()))
    }
}

/// Provider registry for managing multiple authentication providers
pub struct ProviderRegistry {
    providers: Vec<Box<dyn AuthProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }
    
    /// Register a new provider
    pub fn register<P: AuthProvider + 'static>(mut self, provider: P) -> Self {
        self.providers.push(Box::new(provider));
        // Sort by priority (highest first)
        self.providers.sort_by(|a, b| b.priority().cmp(&a.priority()));
        self
    }
    
    /// Find the best provider for a given token
    pub fn find_provider_for_token(&self, token: &str) -> Option<&Box<dyn AuthProvider>> {
        self.providers.iter().find(|provider| provider.can_handle_token(token))
    }
    
    /// Get provider by type
    pub fn get_provider(&self, provider_type: &ProviderType) -> Option<&Box<dyn AuthProvider>> {
        self.providers.iter().find(|provider| &provider.provider_type() == provider_type)
    }
    
    /// Get all providers
    pub fn providers(&self) -> &[Box<dyn AuthProvider>] {
        &self.providers
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_type_display() {
        assert_eq!(ProviderType::Firebase.to_string(), "firebase");
        assert_eq!(ProviderType::OIDC.to_string(), "oidc");
        assert_eq!(ProviderType::Custom.to_string(), "custom");
    }
    
    #[test]
    fn test_user_claims_creation() {
        let user_id = UserId::generate();
        let email = Email::new("test@example.com".to_string()).unwrap();
        let role = Role::User;
        let permissions = vec!["read".to_string(), "write".to_string()];
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        
        let claims = UserClaims::new(
            user_id.clone(),
            email.clone(),
            role,
            permissions.clone(),
            "firebase_uid_123".to_string(),
            ProviderType::Firebase,
            expires_at,
        );
        
        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.email, email);
        assert_eq!(claims.permissions, permissions);
        assert_eq!(claims.provider, ProviderType::Firebase);
        assert!(!claims.is_expired());
    }
    
    #[test]
    fn test_provider_registry() {
        let registry = ProviderRegistry::new();
        assert_eq!(registry.providers().len(), 0);
    }
}