// Validate Credentials Command
// Maps to token validation and authentication verification

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::domain::authentication::{
    SessionId, AuthenticatedUserId, Scope, SecurityContext, ProviderType
};

/// Command to validate authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateCredentialsCommand {
    /// Token to validate
    pub token: String,
    
    /// Expected token type
    pub token_type: TokenType,
    
    /// Required scopes for validation
    pub required_scopes: Vec<Scope>,
    
    /// Client context for security validation
    pub client_context: ClientContext,
}

/// Type of token being validated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    AccessToken,
    RefreshToken,
    IdToken,
}

/// Client context for security validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientContext {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_path: Option<String>,
    pub device_fingerprint: Option<String>,
}

impl ValidateCredentialsCommand {
    /// Create access token validation
    pub fn validate_access_token(
        token: String,
        required_scopes: Vec<Scope>,
    ) -> Self {
        Self {
            token,
            token_type: TokenType::AccessToken,
            required_scopes,
            client_context: ClientContext::empty(),
        }
    }
    
    /// Create refresh token validation
    pub fn validate_refresh_token(token: String) -> Self {
        Self {
            token,
            token_type: TokenType::RefreshToken,
            required_scopes: vec![],
            client_context: ClientContext::empty(),
        }
    }
    
    /// Create ID token validation
    pub fn validate_id_token(token: String) -> Self {
        Self {
            token,
            token_type: TokenType::IdToken,
            required_scopes: vec![Scope::OpenId],
            client_context: ClientContext::empty(),
        }
    }
    
    /// Add client context for security checks
    pub fn with_client_context(mut self, context: ClientContext) -> Self {
        self.client_context = context;
        self
    }
}

impl ClientContext {
    /// Create empty client context
    pub fn empty() -> Self {
        Self {
            ip_address: None,
            user_agent: None,
            request_path: None,
            device_fingerprint: None,
        }
    }
    
    /// Create client context from request headers
    pub fn from_request(
        ip_address: Option<String>,
        user_agent: Option<String>,
        request_path: Option<String>,
    ) -> Self {
        Self {
            ip_address,
            user_agent,
            request_path,
            device_fingerprint: None,
        }
    }
}

/// Response from credential validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateCredentialsResponse {
    /// Whether credentials are valid
    pub valid: bool,
    
    /// Authenticated user (if valid)
    pub user_id: Option<AuthenticatedUserId>,
    
    /// Associated session (if applicable)
    pub session_id: Option<SessionId>,
    
    /// Token expiry time
    pub expires_at: Option<DateTime<Utc>>,
    
    /// Granted scopes
    pub granted_scopes: Vec<String>,
    
    /// Authentication provider used
    pub provider_type: Option<ProviderType>,
    
    /// Security assessment
    pub security_assessment: SecurityAssessment,
    
    /// Validation metadata
    pub validation_metadata: ValidationMetadata,
}

/// Security assessment from validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAssessment {
    /// Risk level of the request
    pub risk_level: String,
    
    /// Security flags detected
    pub security_flags: Vec<String>,
    
    /// Whether additional verification is required
    pub requires_mfa: bool,
    
    /// Suspicious activity score (0-100)
    pub suspicious_score: f64,
}

/// Validation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetadata {
    /// When validation occurred
    pub validated_at: DateTime<Utc>,
    
    /// Validation duration in milliseconds
    pub validation_duration_ms: u64,
    
    /// JWT issuer (if applicable)
    pub token_issuer: Option<String>,
    
    /// JWT ID (if applicable)
    pub token_id: Option<String>,
}

impl ValidateCredentialsResponse {
    /// Create successful validation response
    pub fn success(
        user_id: AuthenticatedUserId,
        session_id: Option<SessionId>,
        expires_at: DateTime<Utc>,
        granted_scopes: Vec<Scope>,
        provider_type: ProviderType,
        security_context: &SecurityContext,
    ) -> Self {
        Self {
            valid: true,
            user_id: Some(user_id),
            session_id,
            expires_at: Some(expires_at),
            granted_scopes: granted_scopes.iter().map(|s| s.as_str().to_string()).collect(),
            provider_type: Some(provider_type),
            security_assessment: SecurityAssessment {
                risk_level: format!("{:?}", security_context.risk_level()),
                security_flags: security_context.security_flags()
                    .iter()
                    .map(|f| format!("{:?}", f))
                    .collect(),
                requires_mfa: false, // Would be determined by business rules
                suspicious_score: security_context.suspicious_score(),
            },
            validation_metadata: ValidationMetadata {
                validated_at: Utc::now(),
                validation_duration_ms: 0, // Would be measured in handler
                token_issuer: None,
                token_id: None,
            },
        }
    }
    
    /// Create failed validation response
    pub fn failed(reason: ValidationFailureReason) -> Self {
        Self {
            valid: false,
            user_id: None,
            session_id: None,
            expires_at: None,
            granted_scopes: vec![],
            provider_type: None,
            security_assessment: SecurityAssessment {
                risk_level: "High".to_string(),
                security_flags: vec![format!("{:?}", reason)],
                requires_mfa: false,
                suspicious_score: match reason {
                    ValidationFailureReason::TokenExpired => 10.0,
                    ValidationFailureReason::InvalidToken => 25.0,
                    ValidationFailureReason::InsufficientScope => 5.0,
                    ValidationFailureReason::SuspiciousActivity => 80.0,
                    ValidationFailureReason::AccountLocked => 100.0,
                },
            },
            validation_metadata: ValidationMetadata {
                validated_at: Utc::now(),
                validation_duration_ms: 0,
                token_issuer: None,
                token_id: None,
            },
        }
    }
}

/// Reasons why validation might fail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationFailureReason {
    TokenExpired,
    InvalidToken,
    InsufficientScope,
    SuspiciousActivity,
    AccountLocked,
}

/// Validation for validate credentials command
impl ValidateCredentialsCommand {
    /// Validate the command itself
    pub fn validate(&self) -> Result<(), ValidateCredentialsValidationError> {
        // Token must not be empty
        if self.token.is_empty() {
            return Err(ValidateCredentialsValidationError::EmptyToken);
        }
        
        // For access tokens, we need scopes
        if matches!(self.token_type, TokenType::AccessToken) && self.required_scopes.is_empty() {
            return Err(ValidateCredentialsValidationError::NoScopesForAccessToken);
        }
        
        // ID tokens must include openid scope
        if matches!(self.token_type, TokenType::IdToken) && !self.required_scopes.contains(&Scope::OpenId) {
            return Err(ValidateCredentialsValidationError::IdTokenMustIncludeOpenId);
        }
        
        Ok(())
    }
}

/// Validation errors for validate credentials command
#[derive(Debug, thiserror::Error)]
pub enum ValidateCredentialsValidationError {
    #[error("Token cannot be empty")]
    EmptyToken,
    
    #[error("Access token validation requires scopes")]
    NoScopesForAccessToken,
    
    #[error("ID token validation must include openid scope")]
    IdTokenMustIncludeOpenId,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn validate_access_token_command() {
        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...".to_string();
        let scopes = vec![Scope::OpenId, Scope::Profile];
        
        let command = ValidateCredentialsCommand::validate_access_token(token.clone(), scopes);
        
        assert_eq!(command.token, token);
        assert_eq!(command.token_type, TokenType::AccessToken);
        assert_eq!(command.required_scopes.len(), 2);
        assert!(command.validate().is_ok());
    }
    
    #[test]
    fn validate_refresh_token_command() {
        let token = "rt_session123_jti456".to_string();
        
        let command = ValidateCredentialsCommand::validate_refresh_token(token.clone());
        
        assert_eq!(command.token, token);
        assert_eq!(command.token_type, TokenType::RefreshToken);
        assert!(command.required_scopes.is_empty());
        assert!(command.validate().is_ok());
    }
    
    #[test]
    fn validate_id_token_command() {
        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...".to_string();
        
        let command = ValidateCredentialsCommand::validate_id_token(token.clone());
        
        assert_eq!(command.token_type, TokenType::IdToken);
        assert!(command.required_scopes.contains(&Scope::OpenId));
        assert!(command.validate().is_ok());
    }
    
    #[test]
    fn validation_errors() {
        // Empty token
        let command = ValidateCredentialsCommand::validate_access_token("".to_string(), vec![Scope::OpenId]);
        assert!(command.validate().is_err());
        
        // Access token without scopes
        let command = ValidateCredentialsCommand::validate_access_token("token123".to_string(), vec![]);
        assert!(command.validate().is_err());
        
        // ID token without openid scope
        let mut command = ValidateCredentialsCommand::validate_id_token("token456".to_string());
        command.required_scopes = vec![Scope::Profile]; // Remove openid
        assert!(command.validate().is_err());
    }
    
    #[test]
    fn client_context() {
        let context = ClientContext::from_request(
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0 (...)".to_string()),
            Some("/api/v1/analytics".to_string()),
        );
        
        assert_eq!(context.ip_address, Some("192.168.1.1".to_string()));
        assert!(context.device_fingerprint.is_none());
    }
    
    #[test]
    fn security_assessment() {
        let security_context = SecurityContext::new();
        let response = ValidateCredentialsResponse::success(
            AuthenticatedUserId::from_verified_user(
                crate::domain::user_management::value_objects::UserId::new().unwrap()
            ),
            None,
            Utc::now() + chrono::Duration::hours(1),
            vec![Scope::OpenId, Scope::Profile],
            ProviderType::Firebase,
            &security_context,
        );
        
        assert!(response.valid);
        assert_eq!(response.security_assessment.risk_level, "Low");
        assert_eq!(response.security_assessment.suspicious_score, 0.0);
    }
}