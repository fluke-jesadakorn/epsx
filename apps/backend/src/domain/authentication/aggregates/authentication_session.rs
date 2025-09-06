use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::shared_kernel::value_objects::SessionId;
// Authentication Session Aggregate Root
// Manages the complete authentication lifecycle including tokens, sessions, and security

use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};

use crate::domain::shared_kernel::AggregateRoot;
use super::super::value_objects::*;
use super::super::events::*;

/// Authentication Session Aggregate Root
/// Encapsulates all authentication state and business rules
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationSession {
    // Identity
    session_id: SessionId,
    user_id: AuthenticatedUserId,
    
    // Authentication context
    provider: AuthenticationProvider,
    client_info: ClientInformation,
    
    // Token management
    access_token: Option<AccessToken>,
    refresh_token: Option<RefreshToken>,
    id_token: Option<IdToken>,
    
    // Security tracking
    security_context: SecurityContext,
    
    // Session lifecycle
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    
    // Aggregate infrastructure
    version: u64,
    #[serde(skip)]
    uncommitted_events: Vec<Box<dyn crate::domain::shared_kernel::DomainEvent>>,
}

impl Clone for AuthenticationSession {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            user_id: self.user_id.clone(),
            provider: self.provider.clone(),
            client_info: self.client_info.clone(),
            access_token: self.access_token.clone(),
            refresh_token: self.refresh_token.clone(),
            id_token: self.id_token.clone(),
            security_context: self.security_context.clone(),
            created_at: self.created_at,
            last_activity: self.last_activity,
            expires_at: self.expires_at,
            version: self.version,
            uncommitted_events: Vec::new(), // Empty for cloned aggregates
        }
    }
}

impl AuthenticationSession {
    /// Create new authentication session
    pub fn create_new(
        user_id: AuthenticatedUserId,
        provider: AuthenticationProvider,
        client_info: ClientInformation,
        scopes: Vec<Scope>,
    ) -> Result<Self, AuthenticationError> {
        let now = Utc::now();
        let session_id = SessionId::generate();
        
        // Validate business rules
        Self::validate_authentication_requirements(&provider, &scopes)?;
        
        let mut session = Self {
            session_id: session_id.clone(),
            user_id: user_id.clone(),
            provider,
            client_info,
            access_token: None,
            refresh_token: None,
            id_token: None,
            security_context: SecurityContext::new(),
            created_at: now,
            last_activity: now,
            expires_at: now + Duration::hours(super::super::AuthenticationBoundedContext::SESSION_TIMEOUT_HOURS as i64),
            version: 1,
            uncommitted_events: Vec::new(),
        };
        
        // Generate initial tokens
        session.issue_tokens(scopes)?;
        
        // Record domain event
        session.add_event(AuthenticationSessionCreatedEvent::new(
            session_id,
            user_id,
            now,
        ));
        
        Ok(session)
    }
    
    /// Issue new tokens for this session
    pub fn issue_tokens(&mut self, scopes: Vec<Scope>) -> Result<(), AuthenticationError> {
        let now = Utc::now();
        
        // Validate session is still valid
        if self.is_expired() {
            return Err(AuthenticationError::SessionExpired);
        }
        
        // Generate new tokens
        let token_lifetime = Duration::hours(
            super::super::AuthenticationBoundedContext::MAX_TOKEN_LIFETIME_HOURS as i64
        );
        let refresh_lifetime = Duration::days(
            super::super::AuthenticationBoundedContext::MAX_REFRESH_TOKEN_LIFETIME_DAYS as i64
        );
        
        self.access_token = Some(AccessToken::generate(
            &self.user_id,
            &scopes,
            now + token_lifetime,
        )?);
        
        self.refresh_token = Some(RefreshToken::generate(
            &self.session_id,
            now + refresh_lifetime,
        )?);
        
        if scopes.contains(&Scope::OpenId) {
            self.id_token = Some(IdToken::generate(
                &self.user_id,
                &self.client_info,
                now + token_lifetime,
            )?);
        }
        
        // Update tracking
        self.last_activity = now;
        self.version += 1;
        
        // Record domain event
        self.add_event(TokensIssuedEvent::new(
            self.session_id.clone(),
            self.user_id.clone(),
            scopes,
        ));
        
        Ok(())
    }
    
    /// Refresh tokens using refresh token
    pub fn refresh_tokens(&mut self, refresh_token: &RefreshToken) -> Result<(), AuthenticationError> {
        // Validate refresh token matches
        match &self.refresh_token {
            Some(stored_token) if stored_token == refresh_token => {},
            _ => return Err(AuthenticationError::InvalidRefreshToken),
        }
        
        // Check if refresh token is expired
        if refresh_token.is_expired() {
            return Err(AuthenticationError::RefreshTokenExpired);
        }
        
        // Issue new tokens with same scopes
        let scopes = self.access_token
            .as_ref()
            .map(|token| token.scopes().clone())
            .unwrap_or_default();
        
        self.issue_tokens(scopes)?;
        
        // Record domain event
        self.add_event(TokensRefreshedEvent::new(
            self.session_id.clone(),
            self.user_id.clone(),
        ));
        
        Ok(())
    }
    
    /// Terminate this authentication session
    pub fn terminate(&mut self, reason: TerminationReason) -> Result<(), AuthenticationError> {
        // Revoke all tokens
        self.access_token = None;
        self.refresh_token = None;
        self.id_token = None;
        
        // Mark as expired
        self.expires_at = Utc::now();
        self.version += 1;
        
        // Record domain event
        self.add_event(AuthenticationSessionTerminatedEvent::new(
            self.session_id.clone(),
            self.user_id.clone(),
            reason,
        ));
        
        Ok(())
    }
    
    /// Update security context with new activity
    pub fn record_activity(&mut self, client_info: ClientInformation) -> Result<(), AuthenticationError> {
        // Update last activity
        self.last_activity = Utc::now();
        
        // Check for suspicious activity
        if self.security_context.detect_anomaly(&client_info) {
            self.add_event(SuspiciousActivityDetectedEvent::new(
                self.session_id.clone(),
                self.user_id.clone(),
                client_info.clone(),
            ));
            
            // Optionally terminate session for high-risk activities
            if self.security_context.is_high_risk() {
                self.terminate(TerminationReason::SecurityThreat)?;
            }
        }
        
        // Update client info
        self.client_info = client_info;
        self.version += 1;
        
        Ok(())
    }
    
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    /// Check if session is active (not expired and has valid tokens)
    pub fn is_active(&self) -> bool {
        !self.is_expired() && 
        self.access_token.as_ref().map_or(false, |t| !t.is_expired())
    }
    
    /// Get current access token if valid
    pub fn current_access_token(&self) -> Option<&AccessToken> {
        self.access_token.as_ref().filter(|token| !token.is_expired())
    }
    
    /// Get current refresh token if valid  
    pub fn current_refresh_token(&self) -> Option<&RefreshToken> {
        self.refresh_token.as_ref().filter(|token| !token.is_expired())
    }
    
    /// Get current ID token if valid
    pub fn current_id_token(&self) -> Option<&IdToken> {
        self.id_token.as_ref().filter(|token| !token.is_expired())
    }
    
    // Getters
    pub fn session_id(&self) -> &SessionId { &self.session_id }
    pub fn user_id(&self) -> &AuthenticatedUserId { &self.user_id }
    pub fn provider(&self) -> &AuthenticationProvider { &self.provider }
    pub fn client_info(&self) -> &ClientInformation { &self.client_info }
    pub fn security_context(&self) -> &SecurityContext { &self.security_context }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn last_activity(&self) -> DateTime<Utc> { self.last_activity }
    pub fn expires_at(&self) -> DateTime<Utc> { self.expires_at }
    
    // Private helpers
    fn validate_authentication_requirements(
        provider: &AuthenticationProvider,
        scopes: &[Scope],
    ) -> Result<(), AuthenticationError> {
        // Must include openid scope for OIDC compliance
        if !scopes.contains(&Scope::OpenId) {
            return Err(AuthenticationError::InvalidScopes("Missing required 'openid' scope".to_string()));
        }
        
        // Provider-specific validation
        provider.validate_capabilities(scopes)?;
        
        Ok(())
    }
    
    fn add_event(&mut self, event: impl Into<Box<dyn crate::domain::shared_kernel::DomainEvent>>) {
        self.uncommitted_events.push(event.into());
    }
}

impl AggregateRoot for AuthenticationSession {
    type Id = SessionId;
    
    fn id(&self) -> &Self::Id {
        &self.session_id
    }
    
    fn version(&self) -> u64 {
        self.version
    }
    
    fn increment_version(&mut self) {
        self.version += 1;
        self.touch();
    }
    
    fn uncommitted_events(&self) -> &[Box<dyn crate::domain::shared_kernel::DomainEvent>] {
        &self.uncommitted_events
    }
    
    fn mark_events_as_committed(&mut self) {
        self.uncommitted_events.clear();
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    fn updated_at(&self) -> DateTime<Utc> {
        self.last_activity
    }
    
    fn touch(&mut self) {
        self.last_activity = Utc::now();
    }
}

/// Authentication errors within the domain
#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
    #[error("Session has expired")]
    SessionExpired,
    
    #[error("Invalid refresh token")]
    InvalidRefreshToken,
    
    #[error("Refresh token has expired")]
    RefreshTokenExpired,
    
    #[error("Invalid scopes: {0}")]
    InvalidScopes(String),
    
    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),
    
    #[error("Security violation detected")]
    SecurityViolation,
}

impl From<super::super::value_objects::TokenError> for AuthenticationError {
    fn from(error: super::super::value_objects::TokenError) -> Self {
        use super::super::value_objects::TokenError;
        match error {
            TokenError::GenerationFailed(msg) => AuthenticationError::TokenGenerationFailed(msg),
            TokenError::InvalidToken(_) => AuthenticationError::InvalidRefreshToken,
            TokenError::Expired => AuthenticationError::RefreshTokenExpired,
            TokenError::SignatureVerificationFailed => AuthenticationError::InvalidRefreshToken,
        }
    }
}

impl From<super::super::value_objects::authentication_provider::AuthProviderError> for AuthenticationError {
    fn from(error: super::super::value_objects::authentication_provider::AuthProviderError) -> Self {
        use super::super::value_objects::authentication_provider::AuthProviderError;
        match error {
            AuthProviderError::UnsupportedScope { scope, .. } => {
                AuthenticationError::InvalidScopes(format!("Unsupported scope: {:?}", scope))
            },
            AuthProviderError::InvalidScopeConfiguration(msg) => {
                AuthenticationError::InvalidScopes(msg)
            },
            AuthProviderError::UnsupportedAuthenticationMethod => {
                AuthenticationError::SecurityViolation
            },
            AuthProviderError::InvalidConfiguration => {
                AuthenticationError::SecurityViolation
            },
        }
    }
}

/// Reasons for session termination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminationReason {
    UserLogout,
    AdminTermination,
    SecurityThreat,
    SessionExpiry,
    TokenRevocation,
}