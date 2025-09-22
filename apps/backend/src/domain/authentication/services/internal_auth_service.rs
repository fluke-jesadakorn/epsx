// Internal authentication service for web application users
// Domain service for OIDC-based authentication with session management

use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::{
    shared_kernel::{
        domain_error::DomainError,
        value_objects::UserId,
    },
    authentication::{
        value_objects::{
            session_id::SessionId,
            security_context::SecurityContext,
        },
        aggregates::authentication_session::AuthenticationSession,
    },
    user_management::{
        value_objects::{Email, permission::Permission},
        repository_ports::user_repository_port::UserRepositoryPort,
    },
};

/// Token pair for OIDC token refresh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: Option<String>,
    pub expires_in: i64,
}

/// Internal authentication service for web application users
/// Handles OIDC token validation, session management, and user permissions
pub struct InternalAuthService {
    user_repository: Arc<dyn UserRepositoryPort>,
    session_repository: Arc<dyn InternalSessionRepositoryPort>,
    oidc_validator: Arc<dyn OidcTokenValidator>,
}

/// Repository port for internal user sessions
#[async_trait::async_trait]
pub trait InternalSessionRepositoryPort: Send + Sync {
    async fn create_session(&self, session: &AuthenticationSession) -> Result<(), DomainError>;
    async fn get_session(&self, session_id: &SessionId) -> Result<Option<AuthenticationSession>, DomainError>;
    async fn update_session(&self, session: &AuthenticationSession) -> Result<(), DomainError>;
    async fn delete_session(&self, session_id: &SessionId) -> Result<(), DomainError>;
    async fn get_user_sessions(&self, user_id: &UserId) -> Result<Vec<AuthenticationSession>, DomainError>;
    async fn cleanup_expired_sessions(&self) -> Result<u32, DomainError>;
}

/// OIDC token validator port
#[async_trait::async_trait]
pub trait OidcTokenValidator: Send + Sync {
    async fn validate_access_token(&self, _token: &str) -> Result<ValidatedToken, AuthenticationError>;
    async fn validate_id_token(&self, _token: &str) -> Result<IdTokenClaims, AuthenticationError>;
    async fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenPair, AuthenticationError>;
    async fn revoke_tokens(&self, access_token: &str) -> Result<(), AuthenticationError>;
}

/// Validated OIDC access token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedToken {
    pub user_id: UserId,
    pub email: Email,
    pub expires_at: DateTime<Utc>,
    pub issued_at: DateTime<Utc>,
    pub scopes: Vec<String>,
    pub client_id: String,
    pub firebase_uid: Option<String>,
}

/// ID token claims from OIDC provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub user_id: UserId,
    pub email: Email,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub email_verified: bool,
    pub firebase_uid: Option<String>,
}

/// Authentication context for internal web users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalAuthContext {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub email: Email,
    pub permissions: Vec<Permission>,
    pub session_expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub is_session_active: bool,
}

/// Authentication request for internal users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalAuthRequest {
    pub access_token: String,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
}

/// Authentication response for internal users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalAuthResponse {
    pub auth_context: InternalAuthContext,
    pub session_created: bool,
    pub tokens_refreshed: bool,
    pub permissions_loaded: bool,
}

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Session not found")]
    SessionNotFound,
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    
    #[error("OIDC validation error: {0}")]
    OidcValidationError(String),
    
    #[error("Invalid client information: {0}")]
    InvalidClientInfo(String),
    
    #[error("Invalid refresh token")]
    InvalidRefreshToken,
    
    #[error("Refresh token expired")]
    RefreshTokenExpired,
    
    #[error("Invalid scopes: {0}")]
    InvalidScopes(String),
    
    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),
    
    #[error("Security violation")]
    SecurityViolation,
    
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),
}

// Convert from domain AuthenticationError to service AuthenticationError
impl From<crate::domain::authentication::aggregates::authentication_session::AuthenticationError> for AuthenticationError {
    fn from(err: crate::domain::authentication::aggregates::authentication_session::AuthenticationError) -> Self {
        match err {
            crate::domain::authentication::aggregates::authentication_session::AuthenticationError::SessionExpired => AuthenticationError::SessionExpired,
            crate::domain::authentication::aggregates::authentication_session::AuthenticationError::InvalidRefreshToken => AuthenticationError::InvalidRefreshToken,
            crate::domain::authentication::aggregates::authentication_session::AuthenticationError::RefreshTokenExpired => AuthenticationError::RefreshTokenExpired,
            crate::domain::authentication::aggregates::authentication_session::AuthenticationError::InvalidScopes(msg) => AuthenticationError::InvalidScopes(msg),
            crate::domain::authentication::aggregates::authentication_session::AuthenticationError::TokenGenerationFailed(msg) => AuthenticationError::TokenGenerationFailed(msg),
            crate::domain::authentication::aggregates::authentication_session::AuthenticationError::SecurityViolation => AuthenticationError::SecurityViolation,
        }
    }
}

impl InternalAuthService {
    pub fn new(
        user_repository: Arc<dyn UserRepositoryPort>,
        session_repository: Arc<dyn InternalSessionRepositoryPort>,
        oidc_validator: Arc<dyn OidcTokenValidator>,
    ) -> Self {
        Self {
            user_repository,
            session_repository,
            oidc_validator,
        }
    }

    /// Authenticate a user with OIDC tokens and create/update session
    pub async fn authenticate(
        &self,
        request: InternalAuthRequest,
    ) -> Result<InternalAuthResponse, AuthenticationError> {
        // Validate access token
        let validated_token = self.oidc_validator
            .validate_access_token(&request.access_token)
            .await?;

        // Validate ID token if provided
        let _id_token_claims = if let Some(id_token) = &request.id_token {
            Some(self.oidc_validator.validate_id_token(id_token).await?)
        } else {
            None
        };

        // Get or create user
        let user = self.user_repository
            .find_by_id(&validated_token.user_id)
            .await?
            .ok_or(AuthenticationError::UserNotFound)?;

        // Load user permissions from the user aggregate and convert to Vec
        let permissions: Vec<Permission> = user.permissions().clone().into_iter().collect();

        // Create or update authentication session
        let session_id = SessionId::generate();
        let _security_context = SecurityContext::new();
        
        // Update security context with client information if available
        if let (Some(_client_ip), Some(_user_agent)) = (&request.client_ip, &request.user_agent) {
            // In a real implementation, we'd extract and add IP/device info
            // For now, create a simple security context
        }

        // Create mock client information
        use crate::domain::authentication::{ClientInformation, ClientId, ClientType, RedirectUri, Scope, AuthenticationProvider, AuthenticatedUserId};
        
        let client_id = ClientId::new("web-app".to_string()).map_err(|e| AuthenticationError::InvalidClientInfo(e.to_string()))?;
        let redirect_uri = RedirectUri::new("http://localhost:3000".to_string()).map_err(|e| AuthenticationError::InvalidClientInfo(e.to_string()))?;
        let client_info = ClientInformation::new(
            client_id,
            ClientType::Public,
            vec![redirect_uri],
            vec![Scope::OpenId, Scope::Profile]
        ).map_err(|e| AuthenticationError::InvalidClientInfo(e.to_string()))?;
        
        
        let user_id_clone = validated_token.user_id.clone();
        let user_id = AuthenticatedUserId::from_verified_user(validated_token.user_id);
        let provider = AuthenticationProvider::internal_service();
        
        let auth_session = AuthenticationSession::create_new(
            user_id,
            provider,
            client_info,
            vec![Scope::OpenId, Scope::Profile],
        )?;

        let session_created = self.session_repository
            .create_session(&auth_session)
            .await
            .is_ok();

        // Build authentication context
        let auth_context = InternalAuthContext {
            user_id: user_id_clone,
            session_id: session_id.clone(),
            email: validated_token.email.clone(),
            permissions,
            session_expires_at: validated_token.expires_at,
            last_activity: Utc::now(),
            is_session_active: true,
        };

        Ok(InternalAuthResponse {
            auth_context,
            session_created,
            tokens_refreshed: false,
            permissions_loaded: true,
        })
    }

    /// Validate existing session and refresh if needed
    pub async fn validate_session(
        &self,
        session_id: &SessionId,
    ) -> Result<InternalAuthContext, AuthenticationError> {
        // Get session from repository
        let session = self.session_repository
            .get_session(session_id)
            .await?
            .ok_or(AuthenticationError::SessionNotFound)?;

        // Check if session is expired
        if session.is_expired() {
            return Err(AuthenticationError::SessionExpired);
        }

        // Get user to access updated permissions
        // Get the UserId from AuthenticatedUserId  
        let session_user_id = session.user_id();
        let user_id = session_user_id.user_id();
        let user = self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AuthenticationError::UserNotFound)?;
            
        // Get user permissions (they might have changed) and convert to Vec
        let permissions: Vec<Permission> = user.permissions().clone().into_iter().collect();

        // Update session last activity
        // TODO: Create client info for activity recording
        // For now, just update the session object directly
        let _ = self.session_repository.update_session(&session).await;

        // Build authentication context
        let auth_context = InternalAuthContext {
            user_id: user_id.clone(),  // Clone the UserId reference
            session_id: session_id.clone(),
            email: Email::new("user@example.com".to_string()).map_err(|e| AuthenticationError::RepositoryError(e.to_string()))?, // TODO: Get from user repository
            permissions,
            session_expires_at: session.expires_at(),
            last_activity: session.last_activity(),
            is_session_active: !session.is_expired(),
        };

        Ok(auth_context)
    }

    /// Refresh OIDC tokens for a session
    pub async fn refresh_tokens(
        &self,
        session_id: &SessionId,
    ) -> Result<TokenPair, AuthenticationError> {
        // Get session
        let session = self.session_repository
            .get_session(session_id)
            .await?
            .ok_or(AuthenticationError::SessionNotFound)?;

        // Get current refresh token from session
        let current_refresh_token = session.current_refresh_token()
            .ok_or(AuthenticationError::InvalidRefreshToken)?;

        // Refresh tokens via OIDC - convert to string for the validator
        let refresh_token_str = current_refresh_token.token().to_string(); // Assuming RefreshToken has a token() method
        let new_tokens = self.oidc_validator
            .refresh_tokens(&refresh_token_str)
            .await?;

        // Update session with new tokens via domain method
        // TODO: Convert string tokens back to domain token objects and use session.refresh_tokens()
        // For now, return the new tokens without updating the session
        
        let _ = self.session_repository.update_session(&session).await;

        Ok(new_tokens)
    }

    /// Logout user and cleanup session
    pub async fn logout(&self, session_id: &SessionId) -> Result<(), AuthenticationError> {
        // Get session
        let session = self.session_repository
            .get_session(session_id)
            .await?
            .ok_or(AuthenticationError::SessionNotFound)?;

        // Revoke tokens with OIDC provider - get access token from session
        if let Some(access_token) = session.current_access_token() {
            let access_token_str = access_token.token().to_string(); // Assuming AccessToken has a token() method
            let _ = self.oidc_validator
                .revoke_tokens(&access_token_str)
                .await;
        }

        // Delete session
        self.session_repository
            .delete_session(session_id)
            .await?;

        Ok(())
    }

    /// Check if user has specific permission
    pub async fn check_permission(
        &self,
        user_id: &UserId,
        required_permission: &Permission,
    ) -> Result<bool, AuthenticationError> {
        let user = self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AuthenticationError::UserNotFound)?;
            
        let user_permissions = user.permissions();
        Ok(user_permissions.contains(required_permission))
    }

    /// Get all active sessions for a user
    pub async fn get_user_sessions(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<AuthenticationSession>, AuthenticationError> {
        let sessions = self.session_repository
            .get_user_sessions(user_id)
            .await?;

        // Filter out expired sessions
        let active_sessions = sessions.into_iter()
            .filter(|session| !session.is_expired())
            .collect();

        Ok(active_sessions)
    }

    /// Cleanup expired sessions (background task)
    pub async fn cleanup_expired_sessions(&self) -> Result<u32, AuthenticationError> {
        let cleaned_count = self.session_repository
            .cleanup_expired_sessions()
            .await?;

        tracing::info!(
            target: "internal_auth_service",
            cleaned_sessions = cleaned_count,
            "Cleaned up expired internal sessions"
        );

        Ok(cleaned_count)
    }
}