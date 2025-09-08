// Client Information Value Object
// OAuth2/OIDC client metadata and validation

use serde::{Serialize, Deserialize};
use std::collections::HashSet;

/// OAuth2/OIDC client information with validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientInformation {
    client_id: ClientId,
    client_type: ClientType,
    redirect_uris: HashSet<RedirectUri>,
    allowed_scopes: HashSet<super::Scope>,
    metadata: ClientMetadata,
}

impl ClientInformation {
    /// Create new client information with validation
    pub fn new(
        client_id: ClientId,
        client_type: ClientType,
        redirect_uris: Vec<RedirectUri>,
        allowed_scopes: Vec<super::Scope>,
    ) -> Result<Self, ClientError> {
        let redirect_uri_set: HashSet<_> = redirect_uris.into_iter().collect();
        let scope_set: HashSet<_> = allowed_scopes.into_iter().collect();
        
        // Validate business rules
        Self::validate_client_configuration(&client_type, &redirect_uri_set, &scope_set)?;
        
        Ok(Self {
            client_id,
            client_type,
            redirect_uris: redirect_uri_set,
            allowed_scopes: scope_set,
            metadata: ClientMetadata::default(),
        })
    }
    
    /// Validate redirect URI for this client
    pub fn is_redirect_uri_allowed(&self, uri: &RedirectUri) -> bool {
        self.redirect_uris.contains(uri)
    }
    
    /// Check if client is allowed to request specific scopes
    pub fn can_request_scopes(&self, requested: &[super::Scope]) -> bool {
        requested.iter().all(|scope| self.allowed_scopes.contains(scope))
    }
    
    /// Check if this is a public client (cannot authenticate)
    pub fn is_public_client(&self) -> bool {
        matches!(self.client_type, ClientType::Public)
    }
    
    /// Check if this is a confidential client (can authenticate)
    pub fn is_confidential_client(&self) -> bool {
        matches!(self.client_type, ClientType::Confidential)
    }
    
    // Getters
    pub fn client_id(&self) -> &ClientId { &self.client_id }
    pub fn client_type(&self) -> &ClientType { &self.client_type }
    pub fn redirect_uris(&self) -> &HashSet<RedirectUri> { &self.redirect_uris }
    pub fn allowed_scopes(&self) -> &HashSet<super::Scope> { &self.allowed_scopes }
    pub fn metadata(&self) -> &ClientMetadata { &self.metadata }
    
    // Private validation
    fn validate_client_configuration(
        client_type: &ClientType,
        redirect_uris: &HashSet<RedirectUri>,
        scopes: &HashSet<super::Scope>,
    ) -> Result<(), ClientError> {
        // Must have at least one redirect URI
        if redirect_uris.is_empty() {
            return Err(ClientError::NoRedirectUris);
        }
        
        // Must have at least basic scopes
        if scopes.is_empty() {
            return Err(ClientError::NoScopes);
        }
        
        // Public clients have additional restrictions
        if matches!(client_type, ClientType::Public) {
            // Public clients cannot use localhost in production
            for uri in redirect_uris {
                if uri.is_localhost() && cfg!(not(debug_assertions)) {
                    return Err(ClientError::LocalhostNotAllowedInProduction);
                }
            }
        }
        
        Ok(())
    }
}

/// OAuth2 client identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId(String);

impl ClientId {
    pub fn new(id: String) -> Result<Self, ClientError> {
        if id.is_empty() {
            return Err(ClientError::EmptyClientId);
        }
        if id.len() > 255 {
            return Err(ClientError::ClientIdTooLong);
        }
        
        // Client ID should be URL-safe
        if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ClientError::InvalidClientIdFormat);
        }
        
        Ok(Self(id))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// OAuth2 client type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientType {
    /// Public client (cannot authenticate itself)
    Public,
    /// Confidential client (can authenticate itself)
    Confidential,
}

/// OAuth2 redirect URI with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RedirectUri(String);

impl RedirectUri {
    pub fn new(uri: String) -> Result<Self, ClientError> {
        // Basic URL validation
        if uri.is_empty() {
            return Err(ClientError::EmptyRedirectUri);
        }
        
        // Must be valid URL
        if let Err(_) = url::Url::parse(&uri) {
            return Err(ClientError::InvalidRedirectUri);
        }
        
        // Must use HTTPS in production (except localhost)
        if !uri.starts_with("https://") && !uri.starts_with("http://localhost") && !uri.starts_with("http://127.0.0.1") {
            if cfg!(not(debug_assertions)) {
                return Err(ClientError::HttpsRequired);
            }
        }
        
        Ok(Self(uri))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    pub fn is_localhost(&self) -> bool {
        self.0.contains("localhost") || self.0.contains("127.0.0.1")
    }
}

impl std::fmt::Display for RedirectUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Client metadata for additional information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub logo_uri: Option<String>,
    pub contact_email: Option<String>,
    pub terms_of_service_uri: Option<String>,
    pub privacy_policy_uri: Option<String>,
}

impl Default for ClientMetadata {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            logo_uri: None,
            contact_email: None,
            terms_of_service_uri: None,
            privacy_policy_uri: None,
        }
    }
}

/// Client-related errors
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Client ID cannot be empty")]
    EmptyClientId,
    
    #[error("Client ID is too long (maximum 255 characters)")]
    ClientIdTooLong,
    
    #[error("Client ID contains invalid characters")]
    InvalidClientIdFormat,
    
    #[error("Client must have at least one redirect URI")]
    NoRedirectUris,
    
    #[error("Client must have at least one allowed scope")]
    NoScopes,
    
    #[error("Redirect URI cannot be empty")]
    EmptyRedirectUri,
    
    #[error("Invalid redirect URI format")]
    InvalidRedirectUri,
    
    #[error("HTTPS is required for redirect URIs in production")]
    HttpsRequired,
    
    #[error("Localhost redirect URIs are not allowed in production")]
    LocalhostNotAllowedInProduction,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::authentication::value_objects::Scope;
    
    #[test]
    fn valid_client_creation() {
        let client_id = ClientId::new("epsx-frontend".to_string()).unwrap();
        let redirect_uri = RedirectUri::new("https://app.epsx.io/callback".to_string()).unwrap();
        let scopes = vec![Scope::OpenId, Scope::Profile];
        
        let client = ClientInformation::new(
            client_id,
            ClientType::Public,
            vec![redirect_uri],
            scopes,
        );
        
        assert!(client.is_ok());
    }
    
    #[test]
    fn client_validation_failures() {
        let client_id = ClientId::new("test-client".to_string()).unwrap();
        
        // No redirect URIs
        let result = ClientInformation::new(
            client_id.clone(),
            ClientType::Public,
            vec![],
            vec![Scope::OpenId],
        );
        assert!(result.is_err());
        
        // No scopes
        let redirect_uri = RedirectUri::new("https://example.com/callback".to_string()).unwrap();
        let result = ClientInformation::new(
            client_id,
            ClientType::Public,
            vec![redirect_uri],
            vec![],
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn redirect_uri_validation() {
        // Valid HTTPS URI
        assert!(RedirectUri::new("https://app.epsx.io/callback".to_string()).is_ok());
        
        // Invalid URI format
        assert!(RedirectUri::new("not-a-url".to_string()).is_err());
        
        // Empty URI
        assert!(RedirectUri::new("".to_string()).is_err());
    }
    
    #[test]
    fn scope_authorization() {
        let client_id = ClientId::new("test-client".to_string()).unwrap();
        let redirect_uri = RedirectUri::new("https://example.com/callback".to_string()).unwrap();
        let allowed_scopes = vec![Scope::OpenId, Scope::Profile, Scope::EpsxAnalytics];
        
        let client = ClientInformation::new(
            client_id,
            ClientType::Confidential,
            vec![redirect_uri],
            allowed_scopes,
        ).unwrap();
        
        // Allowed scopes
        assert!(client.can_request_scopes(&[Scope::OpenId, Scope::Profile]));
        
        // Not allowed scope
        assert!(!client.can_request_scopes(&[Scope::EpsxAdmin]));
    }
}