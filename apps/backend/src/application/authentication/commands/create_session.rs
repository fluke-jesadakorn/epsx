// Create Authentication Session Command
// Maps to login/authentication flow

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::domain::authentication::{
    SessionId, AuthenticatedUserId, AuthenticationProvider, ClientInformation, Scope
};

/// Command to create a new authentication session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionCommand {
    /// User being authenticated
    pub user_id: AuthenticatedUserId,
    
    /// Authentication provider used
    pub provider: AuthenticationProvider,
    
    /// OAuth2/OIDC client information
    pub client_info: ClientInformation,
    
    /// Requested scopes
    pub scopes: Vec<Scope>,
    
    /// Client metadata for security tracking
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
}

impl CreateSessionCommand {
    /// Create command for standard OIDC login
    pub fn for_oidc_login(
        user_id: AuthenticatedUserId,
        client_info: ClientInformation,
        scopes: Vec<Scope>,
    ) -> Self {
        Self {
            user_id,
            provider: AuthenticationProvider::oidc_admin(),
            client_info,
            scopes,
            client_ip: None,
            user_agent: None,
            device_fingerprint: None,
        }
    }
    
    /// Create command for Firebase login
    pub fn for_firebase_login(
        user_id: AuthenticatedUserId,
        client_info: ClientInformation,
    ) -> Self {
        let scopes = vec![
            Scope::OpenId,
            Scope::Profile,
            Scope::Email,
            Scope::EpsxAnalytics,
            Scope::EpsxTrading,
        ];
        
        Self {
            user_id,
            provider: AuthenticationProvider::firebase(),
            client_info,
            scopes,
            client_ip: None,
            user_agent: None,
            device_fingerprint: None,
        }
    }
    
    /// Add client metadata for security tracking
    pub fn with_client_metadata(mut self, ip: Option<String>, user_agent: Option<String>, device_fingerprint: Option<String>) -> Self {
        self.client_ip = ip;
        self.user_agent = user_agent;
        self.device_fingerprint = device_fingerprint;
        self
    }
    
    /// Add admin scopes for elevated access
    pub fn with_admin_scopes(mut self) -> Self {
        if !self.scopes.contains(&Scope::EpsxAdmin) {
            self.scopes.push(Scope::EpsxAdmin);
        }
        if !self.scopes.contains(&Scope::Email) {
            self.scopes.push(Scope::Email);
        }
        self
    }
}

/// Response containing session and tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    /// Session identifier
    pub session_id: SessionId,
    
    /// Access token for API calls
    pub access_token: String,
    
    /// Refresh token for token renewal
    pub refresh_token: Option<String>,
    
    /// ID token for OIDC (if applicable)
    pub id_token: Option<String>,
    
    /// Token expiry time
    pub expires_at: DateTime<Utc>,
    
    /// Token type (typically "Bearer")
    pub token_type: String,
    
    /// Granted scopes
    pub scopes: Vec<String>,
}

impl CreateSessionResponse {
    /// Create response from session and tokens
    pub fn new(
        session_id: SessionId,
        access_token: String,
        refresh_token: Option<String>,
        id_token: Option<String>,
        expires_at: DateTime<Utc>,
        scopes: Vec<Scope>,
    ) -> Self {
        Self {
            session_id,
            access_token,
            refresh_token,
            id_token,
            expires_at,
            token_type: "Bearer".to_string(),
            scopes: scopes.iter().map(|s| s.as_str().to_string()).collect(),
        }
    }
    
    /// Check if response includes OIDC ID token
    pub fn has_id_token(&self) -> bool {
        self.id_token.is_some()
    }
    
    /// Check if response includes refresh token
    pub fn has_refresh_token(&self) -> bool {
        self.refresh_token.is_some()
    }
    
    /// Get token expiry duration from now
    pub fn expires_in_seconds(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds().max(0)
    }
}

/// Validation for create session command
impl CreateSessionCommand {
    /// Validate command business rules
    pub fn validate(&self) -> Result<(), CreateSessionValidationError> {
        // Must have at least one scope
        if self.scopes.is_empty() {
            return Err(CreateSessionValidationError::NoScopes);
        }
        
        // OIDC scopes require openid
        let has_oidc_scopes = self.scopes.iter().any(|s| matches!(s, 
            Scope::Profile | Scope::Email | Scope::Address | Scope::Phone
        ));
        
        if has_oidc_scopes && !self.scopes.contains(&Scope::OpenId) {
            return Err(CreateSessionValidationError::MissingOpenIdScope);
        }
        
        // Admin scopes require email
        if self.scopes.contains(&Scope::EpsxAdmin) && !self.scopes.contains(&Scope::Email) {
            return Err(CreateSessionValidationError::AdminRequiresEmail);
        }
        
        // Validate provider supports requested scopes
        self.provider.validate_capabilities(&self.scopes)
            .map_err(CreateSessionValidationError::ProviderValidation)?;
        
        Ok(())
    }
}

/// Validation errors for create session command
#[derive(Debug, thiserror::Error)]
pub enum CreateSessionValidationError {
    #[error("Session must have at least one scope")]
    NoScopes,
    
    #[error("OIDC scopes require 'openid' scope")]
    MissingOpenIdScope,
    
    #[error("Admin scopes require email access")]
    AdminRequiresEmail,
    
    #[error("Provider validation failed: {0}")]
    ProviderValidation(#[from] crate::domain::authentication::AuthProviderError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::authentication::{ClientId, ClientType, RedirectUri};
    use crate::domain::user_management::value_objects::UserId;
    
    #[test]
    fn create_oidc_login_command() {
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let client_id = ClientId::new("test-client".to_string()).unwrap();
        let redirect_uri = RedirectUri::new("https://app.epsx.io/callback".to_string()).unwrap();
        let client_info = ClientInformation::new(
            client_id,
            ClientType::Public,
            vec![redirect_uri],
            vec![Scope::OpenId, Scope::Profile],
        ).unwrap();
        
        let command = CreateSessionCommand::for_oidc_login(
            user_id,
            client_info,
            vec![Scope::OpenId, Scope::Profile, Scope::Email],
        );
        
        assert!(command.validate().is_ok());
        assert_eq!(command.provider.provider_type(), &crate::domain::authentication::ProviderType::OpenIdConnect);
    }
    
    #[test]
    fn firebase_login_command() {
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let client_id = ClientId::new("firebase-client".to_string()).unwrap();
        let redirect_uri = RedirectUri::new("https://app.epsx.io/callback".to_string()).unwrap();
        let client_info = ClientInformation::new(
            client_id,
            ClientType::Public,
            vec![redirect_uri],
            vec![Scope::OpenId, Scope::Profile, Scope::EpsxAnalytics],
        ).unwrap();
        
        let command = CreateSessionCommand::for_firebase_login(user_id, client_info);
        
        assert!(command.validate().is_ok());
        assert!(command.scopes.contains(&Scope::OpenId));
        assert!(command.scopes.contains(&Scope::EpsxAnalytics));
    }
    
    #[test]
    fn validation_errors() {
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let client_info = ClientInformation::new(
            ClientId::new("test".to_string()).unwrap(),
            ClientType::Public,
            vec![RedirectUri::new("https://example.com".to_string()).unwrap()],
            vec![Scope::OpenId],
        ).unwrap();
        
        // No scopes
        let mut command = CreateSessionCommand::for_oidc_login(user_id.clone(), client_info.clone(), vec![]);
        assert!(command.validate().is_err());
        
        // OIDC scope without openid
        command.scopes = vec![Scope::Profile, Scope::Email];
        assert!(command.validate().is_err());
        
        // Admin without email
        command.scopes = vec![Scope::OpenId, Scope::EpsxAdmin];
        assert!(command.validate().is_err());
    }
}