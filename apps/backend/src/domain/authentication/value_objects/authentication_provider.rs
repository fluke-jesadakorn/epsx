// Authentication Provider Value Object
// Represents different authentication providers and their capabilities

use serde::{Serialize, Deserialize};
use std::collections::HashSet;

use super::Scope;

/// Authentication provider with capabilities and configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticationProvider {
    provider_type: ProviderType,
    provider_config: ProviderConfiguration,
    supported_scopes: HashSet<Scope>,
    capabilities: ProviderCapabilities,
}

impl AuthenticationProvider {
    /// Create new authentication provider
    pub fn new(
        provider_type: ProviderType,
        supported_scopes: Vec<Scope>,
    ) -> Self {
        let capabilities = ProviderCapabilities::from_provider_type(&provider_type);
        let provider_config = ProviderConfiguration::default_for_type(&provider_type);
        
        Self {
            provider_type,
            provider_config,
            supported_scopes: supported_scopes.into_iter().collect(),
            capabilities,
        }
    }
    
    /// Create Firebase provider (primary for EPSX)
    pub fn firebase() -> Self {
        let scopes = vec![
            Scope::OpenId,
            Scope::Profile,
            Scope::Email,
            Scope::EpsxAnalytics,
            Scope::EpsxTrading,
            Scope::EpsxNotifications,
        ];
        
        Self::new(ProviderType::Firebase, scopes)
    }
    
    /// Create OIDC provider for admin access
    pub fn oidc_admin() -> Self {
        let scopes = vec![
            Scope::OpenId,
            Scope::Profile,
            Scope::Email,
            Scope::EpsxAdmin,
        ];
        
        Self::new(ProviderType::OpenIdConnect, scopes)
    }
    
    /// Create internal service provider
    pub fn internal_service() -> Self {
        let scopes = vec![
            Scope::EpsxAnalytics,
            Scope::EpsxTrading,
            Scope::EpsxNotifications,
            Scope::EpsxAdmin,
        ];
        
        Self::new(ProviderType::Internal, scopes)
    }
    
    /// Validate that provider can handle requested scopes
    pub fn validate_capabilities(&self, requested_scopes: &[Scope]) -> Result<(), AuthProviderError> {
        // Check if all requested scopes are supported
        for scope in requested_scopes {
            if !self.supported_scopes.contains(scope) {
                return Err(AuthProviderError::UnsupportedScope {
                    provider: self.provider_type.clone(),
                    scope: scope.clone(),
                });
            }
        }
        
        // Provider-specific validations
        match &self.provider_type {
            ProviderType::Firebase => {
                // Firebase must include profile for user info
                if requested_scopes.contains(&Scope::OpenId) && !requested_scopes.contains(&Scope::Profile) {
                    return Err(AuthProviderError::InvalidScopeConfiguration(
                        "Firebase OIDC requires profile scope".to_string()
                    ));
                }
            },
            ProviderType::OpenIdConnect => {
                // Admin OIDC must include email for verification
                if requested_scopes.contains(&Scope::EpsxAdmin) && !requested_scopes.contains(&Scope::Email) {
                    return Err(AuthProviderError::InvalidScopeConfiguration(
                        "Admin access requires email scope".to_string()
                    ));
                }
            },
            ProviderType::Internal => {
                // Internal services cannot request OIDC scopes
                let oidc_scopes = [Scope::OpenId, Scope::Profile, Scope::Email];
                if requested_scopes.iter().any(|s| oidc_scopes.contains(s)) {
                    return Err(AuthProviderError::InvalidScopeConfiguration(
                        "Internal services cannot request OIDC scopes".to_string()
                    ));
                }
            },
        }
        
        Ok(())
    }
    
    /// Check if provider supports specific authentication method
    pub fn supports_method(&self, method: &AuthenticationMethod) -> bool {
        self.capabilities.supported_methods.contains(method)
    }
    
    /// Get token lifetime for this provider
    pub fn token_lifetime(&self) -> chrono::Duration {
        match self.provider_type {
            ProviderType::Firebase => chrono::Duration::hours(1),
            ProviderType::OpenIdConnect => chrono::Duration::hours(2),
            ProviderType::Internal => chrono::Duration::hours(24),
        }
    }
    
    /// Get refresh token lifetime for this provider  
    pub fn refresh_token_lifetime(&self) -> chrono::Duration {
        match self.provider_type {
            ProviderType::Firebase => chrono::Duration::days(30),
            ProviderType::OpenIdConnect => chrono::Duration::days(7),
            ProviderType::Internal => chrono::Duration::days(90),
        }
    }
    
    /// Check if provider requires additional security measures
    pub fn requires_mfa(&self, scopes: &[Scope]) -> bool {
        // Admin access always requires MFA
        if scopes.contains(&Scope::EpsxAdmin) {
            return true;
        }
        
        // Provider-specific MFA requirements
        match self.provider_type {
            ProviderType::Firebase => false,
            ProviderType::OpenIdConnect => true,
            ProviderType::Internal => false,
        }
    }
    
    // Getters
    pub fn provider_type(&self) -> &ProviderType { &self.provider_type }
    pub fn supported_scopes(&self) -> &HashSet<Scope> { &self.supported_scopes }
    pub fn capabilities(&self) -> &ProviderCapabilities { &self.capabilities }
    pub fn config(&self) -> &ProviderConfiguration { &self.provider_config }
}

/// Types of authentication providers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderType {
    /// Firebase Authentication (primary for EPSX users)
    Firebase,
    /// Standard OpenID Connect provider (admin access)
    OpenIdConnect,
    /// Internal service authentication
    Internal,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::Firebase => write!(f, "firebase"),
            ProviderType::OpenIdConnect => write!(f, "oidc"),
            ProviderType::Internal => write!(f, "internal"),
        }
    }
}

/// Provider capabilities and features
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub supported_methods: HashSet<AuthenticationMethod>,
    pub supportsrefresh_tokens: bool,
    pub supports_id_tokens: bool,
    pub supports_userinfo: bool,
    pub supports_revocation: bool,
    pub supports_introspection: bool,
}

impl ProviderCapabilities {
    fn from_provider_type(provider_type: &ProviderType) -> Self {
        match provider_type {
            ProviderType::Firebase => Self {
                supported_methods: [
                    AuthenticationMethod::Password,
                    AuthenticationMethod::OAuth2,
                    AuthenticationMethod::SocialLogin,
                ].into_iter().collect(),
                supportsrefresh_tokens: true,
                supports_id_tokens: true,
                supports_userinfo: true,
                supports_revocation: true,
                supports_introspection: false,
            },
            ProviderType::OpenIdConnect => Self {
                supported_methods: [
                    AuthenticationMethod::OAuth2,
                    AuthenticationMethod::PKCE,
                    AuthenticationMethod::ClientCredentials,
                ].into_iter().collect(),
                supportsrefresh_tokens: true,
                supports_id_tokens: true,
                supports_userinfo: true,
                supports_revocation: true,
                supports_introspection: true,
            },
            ProviderType::Internal => Self {
                supported_methods: [
                    AuthenticationMethod::ApiKey,
                    AuthenticationMethod::ClientCredentials,
                ].into_iter().collect(),
                supportsrefresh_tokens: false,
                supports_id_tokens: false,
                supports_userinfo: false,
                supports_revocation: true,
                supports_introspection: true,
            },
        }
    }
}

/// Authentication methods supported by providers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthenticationMethod {
    Password,
    OAuth2,
    PKCE,
    SocialLogin,
    ApiKey,
    ClientCredentials,
    Certificate,
}

/// Provider-specific configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderConfiguration {
    pub issuer_url: Option<String>,
    pub client_id: Option<String>,
    pub jwks_uri: Option<String>,
    pub userinfo_endpoint: Option<String>,
    pub revocation_endpoint: Option<String>,
    pub introspection_endpoint: Option<String>,
}

impl ProviderConfiguration {
    fn default_for_type(provider_type: &ProviderType) -> Self {
        match provider_type {
            ProviderType::Firebase => Self {
                issuer_url: Some("https://securetoken.google.com/epsx-449804".to_string()),
                client_id: Some("epsx-firebase-client".to_string()),
                jwks_uri: Some("https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com".to_string()),
                userinfo_endpoint: Some("https://www.googleapis.com/oauth2/v1/userinfo".to_string()),
                revocation_endpoint: None,
                introspection_endpoint: None,
            },
            ProviderType::OpenIdConnect => Self {
                issuer_url: Some("https://api.epsx.io".to_string()),
                client_id: Some("epsx-admin-client".to_string()),
                jwks_uri: Some("https://api.epsx.io/.well-known/jwks.json".to_string()),
                userinfo_endpoint: Some("https://api.epsx.io/oauth/userinfo".to_string()),
                revocation_endpoint: Some("https://api.epsx.io/oauth/revoke".to_string()),
                introspection_endpoint: Some("https://api.epsx.io/oauth/introspect".to_string()),
            },
            ProviderType::Internal => Self {
                issuer_url: Some("https://api.epsx.io".to_string()),
                client_id: Some("epsx-internal-services".to_string()),
                jwks_uri: None,
                userinfo_endpoint: None,
                revocation_endpoint: Some("https://api.epsx.io/internal/revoke".to_string()),
                introspection_endpoint: Some("https://api.epsx.io/internal/introspect".to_string()),
            },
        }
    }
}

/// Authentication provider errors
#[derive(Debug, thiserror::Error)]
pub enum AuthProviderError {
    #[error("Provider {provider:?} does not support scope {scope:?}")]
    UnsupportedScope {
        provider: ProviderType,
        scope: Scope,
    },
    
    #[error("Invalid scope configuration: {0}")]
    InvalidScopeConfiguration(String),
    
    #[error("Provider does not support authentication method")]
    UnsupportedAuthenticationMethod,
    
    #[error("Provider configuration is invalid")]
    InvalidConfiguration,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn firebase_provider_creation() {
        let provider = AuthenticationProvider::firebase();
        
        assert_eq!(provider.provider_type(), &ProviderType::Firebase);
        assert!(provider.supported_scopes().contains(&Scope::OpenId));
        assert!(provider.supported_scopes().contains(&Scope::EpsxAnalytics));
        assert!(provider.supports_method(&AuthenticationMethod::OAuth2));
    }
    
    #[test]
    fn oidc_admin_provider_validation() {
        let provider = AuthenticationProvider::oidc_admin();
        
        // Valid admin scopes
        let valid_scopes = vec![Scope::OpenId, Scope::Email, Scope::EpsxAdmin];
        assert!(provider.validate_capabilities(&valid_scopes).is_ok());
        
        // Invalid: admin without email
        let invalid_scopes = vec![Scope::OpenId, Scope::EpsxAdmin];
        assert!(provider.validate_capabilities(&invalid_scopes).is_err());
    }
    
    #[test]
    fn internal_provider_restrictions() {
        let provider = AuthenticationProvider::internal_service();
        
        // Internal services can access EPSX scopes
        let valid_scopes = vec![Scope::EpsxAnalytics, Scope::EpsxAdmin];
        assert!(provider.validate_capabilities(&valid_scopes).is_ok());
        
        // But cannot request OIDC scopes
        let invalid_scopes = vec![Scope::OpenId, Scope::EpsxAnalytics];
        assert!(provider.validate_capabilities(&invalid_scopes).is_err());
    }
    
    #[test]
    fn provider_mfa_requirements() {
        let firebase = AuthenticationProvider::firebase();
        let oidc_admin = AuthenticationProvider::oidc_admin();
        
        // Admin access requires MFA
        let admin_scopes = vec![Scope::EpsxAdmin];
        assert!(firebase.requires_mfa(&admin_scopes));
        assert!(oidc_admin.requires_mfa(&admin_scopes));
        
        // Regular scopes don't require MFA for Firebase
        let user_scopes = vec![Scope::OpenId, Scope::Profile];
        assert!(!firebase.requires_mfa(&user_scopes));
    }
    
    #[test]
    fn provider_token_lifetimes() {
        let firebase = AuthenticationProvider::firebase();
        let oidc = AuthenticationProvider::oidc_admin();
        let internal = AuthenticationProvider::internal_service();
        
        assert_eq!(firebase.token_lifetime(), chrono::Duration::hours(1));
        assert_eq!(oidc.token_lifetime(), chrono::Duration::hours(2));
        assert_eq!(internal.token_lifetime(), chrono::Duration::hours(24));
        
        assert_eq!(firebase.refresh_token_lifetime(), chrono::Duration::days(30));
        assert_eq!(oidc.refresh_token_lifetime(), chrono::Duration::days(7));
        assert_eq!(internal.refresh_token_lifetime(), chrono::Duration::days(90));
    }
}