// Refresh Tokens Command
// Maps to token refresh flow

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::domain::authentication::{SessionId, RefreshToken, Scope};

/// Command to refresh authentication tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokensCommand {
    /// Current session ID
    pub session_id: SessionId,
    
    /// Refresh token to validate
    pub refresh_token: String,
    
    /// Optional new scopes (for scope escalation)
    pub requested_scopes: Option<Vec<Scope>>,
}

impl RefreshTokensCommand {
    /// Create standard token refresh command
    pub fn new(session_id: SessionId, refresh_token: String) -> Self {
        Self {
            session_id,
            refresh_token,
            requested_scopes: None,
        }
    }
    
    /// Create token refresh with scope changes
    pub fn with_scopes(session_id: SessionId, refresh_token: String, scopes: Vec<Scope>) -> Self {
        Self {
            session_id,
            refresh_token,
            requested_scopes: Some(scopes),
        }
    }
    
    /// Add admin scopes to refresh request
    pub fn with_admin_escalation(mut self) -> Self {
        let mut scopes = self.requested_scopes.unwrap_or_default();
        if !scopes.contains(&Scope::EpsxAdmin) {
            scopes.push(Scope::EpsxAdmin);
        }
        if !scopes.contains(&Scope::Email) {
            scopes.push(Scope::Email);
        }
        self.requested_scopes = Some(scopes);
        self
    }
}

/// Response from token refresh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokensResponse {
    /// New access token
    pub access_token: String,
    
    /// New refresh token (if rotated)
    pub refresh_token: Option<String>,
    
    /// New ID token (if OIDC)
    pub id_token: Option<String>,
    
    /// Token expiry time
    pub expires_at: DateTime<Utc>,
    
    /// Token type
    pub token_type: String,
    
    /// Granted scopes
    pub scopes: Vec<String>,
    
    /// Whether tokens were rotated for security
    pub tokens_rotated: bool,
}

impl RefreshTokensResponse {
    /// Create response from refreshed tokens
    pub fn new(
        access_token: String,
        refresh_token: Option<String>,
        id_token: Option<String>,
        expires_at: DateTime<Utc>,
        scopes: Vec<Scope>,
        tokens_rotated: bool,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            id_token,
            expires_at,
            token_type: "Bearer".to_string(),
            scopes: scopes.iter().map(|s| s.as_str().to_string()).collect(),
            tokens_rotated,
        }
    }
    
    /// Get seconds until token expires
    pub fn expires_in_seconds(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds().max(0)
    }
}

/// Validation for refresh tokens command
impl RefreshTokensCommand {
    /// Validate refresh command
    pub fn validate(&self) -> Result<(), RefreshTokensValidationError> {
        // Refresh token must not be empty
        if self.refresh_token.is_empty() {
            return Err(RefreshTokensValidationError::EmptyRefreshToken);
        }
        
        // Refresh token should have valid format (basic check)
        if !self.refresh_token.starts_with("rt_") {
            return Err(RefreshTokensValidationError::InvalidRefreshTokenFormat);
        }
        
        // If requesting new scopes, validate them
        if let Some(scopes) = &self.requested_scopes {
            if scopes.is_empty() {
                return Err(RefreshTokensValidationError::EmptyScopes);
            }
            
            // OIDC scopes require openid
            let has_oidc_scopes = scopes.iter().any(|s| matches!(s,
                Scope::Profile | Scope::Email | Scope::Address | Scope::Phone
            ));
            
            if has_oidc_scopes && !scopes.contains(&Scope::OpenId) {
                return Err(RefreshTokensValidationError::MissingOpenIdScope);
            }
            
            // Admin scopes require email
            if scopes.contains(&Scope::EpsxAdmin) && !scopes.contains(&Scope::Email) {
                return Err(RefreshTokensValidationError::AdminRequiresEmail);
            }
        }
        
        Ok(())
    }
}

/// Validation errors for refresh tokens command
#[derive(Debug, thiserror::Error)]
pub enum RefreshTokensValidationError {
    #[error("Refresh token cannot be empty")]
    EmptyRefreshToken,
    
    #[error("Invalid refresh token format")]
    InvalidRefreshTokenFormat,
    
    #[error("Requested scopes cannot be empty")]
    EmptyScopes,
    
    #[error("OIDC scopes require 'openid' scope")]
    MissingOpenIdScope,
    
    #[error("Admin scopes require email access")]
    AdminRequiresEmail,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::authentication::SessionId;
    
    #[test]
    fn create_refresh_command() {
        let session_id = SessionId::generate();
        let refresh_token = "rt_test_session_jti123".to_string();
        
        let command = RefreshTokensCommand::new(session_id.clone(), refresh_token.clone());
        
        assert_eq!(command.session_id, session_id);
        assert_eq!(command.refresh_token, refresh_token);
        assert!(command.requested_scopes.is_none());
        assert!(command.validate().is_ok());
    }
    
    #[test]
    fn refresh_with_scopes() {
        let session_id = SessionId::generate();
        let refresh_token = "rt_test_session_jti456".to_string();
        let scopes = vec![Scope::OpenId, Scope::Profile, Scope::EpsxAnalytics];
        
        let command = RefreshTokensCommand::with_scopes(session_id, refresh_token, scopes.clone());
        
        assert_eq!(command.requested_scopes, Some(scopes));
        assert!(command.validate().is_ok());
    }
    
    #[test]
    fn admin_escalation() {
        let session_id = SessionId::generate();
        let refresh_token = "rt_test_session_jti789".to_string();
        
        let command = RefreshTokensCommand::new(session_id, refresh_token)
            .with_admin_escalation();
        
        let scopes = command.requested_scopes.as_ref().unwrap();
        assert!(scopes.contains(&Scope::EpsxAdmin));
        assert!(scopes.contains(&Scope::Email));
        assert!(command.validate().is_ok());
    }
    
    #[test]
    fn validation_errors() {
        let session_id = SessionId::generate();
        
        // Empty refresh token
        let command = RefreshTokensCommand::new(session_id.clone(), "".to_string());
        assert!(command.validate().is_err());
        
        // Invalid refresh token format
        let command = RefreshTokensCommand::new(session_id.clone(), "invalid_format".to_string());
        assert!(command.validate().is_err());
        
        // Empty scopes
        let command = RefreshTokensCommand::with_scopes(
            session_id.clone(),
            "rt_valid_token_123".to_string(),
            vec![]
        );
        assert!(command.validate().is_err());
        
        // OIDC without openid
        let command = RefreshTokensCommand::with_scopes(
            session_id.clone(),
            "rt_valid_token_456".to_string(),
            vec![Scope::Profile, Scope::Email]
        );
        assert!(command.validate().is_err());
        
        // Admin without email
        let command = RefreshTokensCommand::with_scopes(
            session_id,
            "rt_valid_token_789".to_string(),
            vec![Scope::OpenId, Scope::EpsxAdmin]
        );
        assert!(command.validate().is_err());
    }
}