// Create Session Command Handler
// Handles authentication session creation with full OIDC compliance

use async_trait::async_trait;
use tracing::{info, warn};

use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use std::sync::Arc;
use crate::domain::authentication::{
    AuthenticationSession, AuthenticationError, SessionId,
    AuthenticationSessionRepositoryPort, TokenValidationServicePort,
    SecurityMonitoringServicePort
};

use super::super::{CreateSessionCommand, CreateSessionResponse};

/// Handler for creating authentication sessions
pub struct CreateSessionHandler {
    session_repository: Arc<dyn AuthenticationSessionRepositoryPort>,
    token_validation_service: Arc<dyn TokenValidationServicePort>,
    security_monitoring_service: Arc<dyn SecurityMonitoringServicePort>,
}

impl CreateSessionHandler {
    pub fn new(
        session_repository: Arc<dyn AuthenticationSessionRepositoryPort>,
        token_validation_service: Arc<dyn TokenValidationServicePort>,
        security_monitoring_service: Arc<dyn SecurityMonitoringServicePort>,
    ) -> Self {
        Self {
            session_repository,
            token_validation_service,
            security_monitoring_service,
        }
    }
}

#[async_trait]
impl CommandHandler<CreateSessionCommand> for CreateSessionHandler {
    async fn handle(&self, command: CreateSessionCommand) -> ApplicationResult<CreateSessionResponse> {
        info!(
            user_id = %command.user_id,
            provider = ?command.provider.provider_type(),
            scopes = ?command.scopes,
            "Creating new authentication session"
        );
        
        // Validate command
        command.validate()
            .map_err(|e| ApplicationError::validation("command", e.to_string()))?;
        
        // Security pre-checks
        self.perform_security_checks(&command).await?;
        
        // Create new authentication session
        let mut session = AuthenticationSession::create_new(
            command.user_id.clone(),
            command.provider.clone(),
            command.client_info.clone(),
            command.scopes.clone(),
        )
        .map_err(|e| ApplicationError::DomainError(e.to_string()))?;
        
        // Record client activity if provided
        if command.client_ip.is_some() || command.user_agent.is_some() {
            if let Err(e) = session.record_activity(command.client_info.clone()) {
                warn!(
                    session_id = %session.session_id(),
                    error = %e,
                    "Failed to record initial client activity"
                );
            }
        }
        
        // Save session to repository
        self.session_repository.save(&session).await
            .map_err(|e| ApplicationError::InfrastructureError(e.to_string()))?;
        
        // Extract tokens from session
        let access_token = session.current_access_token()
            .ok_or_else(|| ApplicationError::BusinessLogicError("No access token generated".to_string()))?;
        
        let refresh_token = session.current_refresh_token()
            .map(|t| t.token().to_string());
        
        let id_token = session.current_id_token()
            .map(|t| t.token().to_string());
        
        // Security monitoring
        self.security_monitoring_service.record_session_creation(
            &command.user_id.to_string(),
            command.client_ip.as_deref().unwrap_or("unknown"),
            &session.session_id().to_string(),
        ).await.map_err(|e| {
            warn!(error = %e, "Failed to record session creation in security monitoring");
            // Don't fail the entire operation for monitoring failures
        }).ok();
        
        // Create response - convert SessionId to expected type
        let session_id_str = session.session_id().to_string();
        let response_session_id = SessionId::from_string(session_id_str)
            .map_err(|e| ApplicationError::validation("session_id", &e.to_string()))?;
        let response = CreateSessionResponse::new(
            response_session_id,
            access_token.token().to_string(),
            refresh_token,
            id_token,
            access_token.expires_at(),
            command.scopes,
        );
        
        info!(
            session_id = %response.session_id,
            expires_in = response.expires_in_seconds(),
            has_refresh_token = response.has_refresh_token(),
            has_id_token = response.has_id_token(),
            "Authentication session created successfully"
        );
        
        Ok(response)
    }
}

impl CreateSessionHandler {
    /// Perform security checks before creating session
    async fn perform_security_checks(&self, command: &CreateSessionCommand) -> ApplicationResult<()> {
        // Check for suspicious patterns
        if let Some(ref ip) = command.client_ip {
            if self.security_monitoring_service.is_suspicious_ip(ip).await
                .map_err(|e| ApplicationError::InfrastructureError(e.to_string()))? {
                
                warn!(
                    ip = ip,
                    user_id = %command.user_id,
                    "Suspicious IP detected for session creation"
                );
                
                return Err(ApplicationError::SecurityError(
                    "Authentication from suspicious IP address".to_string()
                ));
            }
        }
        
        // Check rate limiting for user
        let user_id_str = command.user_id.to_string();
        if self.security_monitoring_service.is_rate_limited(&user_id_str).await
            .map_err(|e| ApplicationError::InfrastructureError(e.to_string()))? {
            
            warn!(
                user_id = %command.user_id,
                "Rate limit exceeded for session creation"
            );
            
            return Err(ApplicationError::SecurityError(
                "Too many authentication attempts".to_string()
            ));
        }
        
        // Validate provider can handle requested scopes
        command.provider.validate_capabilities(&command.scopes)
            .map_err(|e| ApplicationError::validation("command", e.to_string()))?;
        
        // Additional admin scope validation
        if command.scopes.contains(&crate::domain::authentication::Scope::EpsxAdmin) {
            self.validate_admin_access(&command.user_id).await?;
        }
        
        Ok(())
    }
    
    /// Validate admin access permissions
    async fn validate_admin_access(&self, user_id: &crate::domain::authentication::AuthenticatedUserId) -> ApplicationResult<()> {
        // In a real implementation, this would check user permissions
        // For now, we'll assume the user ID itself indicates admin status
        
        let user_id_str = user_id.to_string();
        if !user_id_str.contains("admin") && !user_id_str.contains("super") {
            warn!(
                user_id = %user_id,
                "Non-admin user requesting admin scopes"
            );
            
            return Err(ApplicationError::AuthorizationError(
                "User does not have admin privileges".to_string()
            ));
        }
        
        Ok(())
    }
}

// Error conversion for domain errors
impl From<AuthenticationError> for ApplicationError {
    fn from(error: AuthenticationError) -> Self {
        match error {
            AuthenticationError::SessionExpired => 
                ApplicationError::BusinessLogicError("Session expired".to_string()),
            AuthenticationError::InvalidRefreshToken => 
                ApplicationError::validation("token", "Invalid refresh token"),
            AuthenticationError::RefreshTokenExpired => 
                ApplicationError::BusinessLogicError("Refresh token expired".to_string()),
            AuthenticationError::InvalidScopes(msg) => 
                ApplicationError::validation("scopes", format!("Invalid scopes: {}", msg)),
            AuthenticationError::TokenGenerationFailed(msg) => 
                ApplicationError::InfrastructureError(format!("Token generation failed: {}", msg)),
            AuthenticationError::SecurityViolation => 
                ApplicationError::SecurityError("Security violation detected".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use async_trait::async_trait;
    
    use crate::domain::authentication::*;
    use crate::domain::user_management::value_objects::UserId;
    
    // Mock implementations for testing
    struct MockSessionRepository;
    struct MockTokenValidationService;
    struct MockSecurityMonitoringService;
    
    #[async_trait]
    impl AuthenticationSessionRepositoryPort for MockSessionRepository {
        async fn save(&self, _session: &AuthenticationSession) -> Result<(), String> {
            Ok(())
        }
        
        async fn find_by_id(&self, _id: &SessionId) -> Result<Option<AuthenticationSession>, String> {
            Ok(None)
        }
        
        async fn find_by_user(&self, _user_id: &AuthenticatedUserId) -> Result<Vec<AuthenticationSession>, String> {
            Ok(vec![])
        }
        
        async fn delete(&self, _id: &SessionId) -> Result<(), String> {
            Ok(())
        }
    }
    
    #[async_trait]
    impl TokenValidationServicePort for MockTokenValidationService {
        async fn validate_access_token(&self, _token: &str) -> Result<bool, String> {
            Ok(true)
        }
        
        async fn validate_refresh_token(&self, _token: &str) -> Result<bool, String> {
            Ok(true)
        }
    }
    
    #[async_trait]
    impl SecurityMonitoringServicePort for MockSecurityMonitoringService {
        async fn record_session_creation(&self, _session_id: &SessionId, _user_id: &AuthenticatedUserId, _ip: Option<&str>) -> Result<(), String> {
            Ok(())
        }
        
        async fn is_suspicious_ip(&self, _ip: &str) -> Result<bool, String> {
            Ok(false)
        }
        
        async fn is_rate_limited(&self, _user_id: &str) -> Result<bool, String> {
            Ok(false)
        }
    }
    
    fn create_test_handler() -> CreateSessionHandler {
        CreateSessionHandler::new(
            Box::new(MockSessionRepository),
            Box::new(MockTokenValidationService),
            Box::new(MockSecurityMonitoringService),
        )
    }
    
    #[tokio::test]
    async fn test_create_session_success() {
        let handler = create_test_handler();
        
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let client_id = ClientId::new("test-client".to_string()).unwrap();
        let redirect_uri = RedirectUri::new("https://app.epsx.io/callback".to_string()).unwrap();
        let client_info = ClientInformation::new(
            client_id,
            ClientType::Public,
            vec![redirect_uri],
            vec![Scope::OpenId, Scope::Profile],
        ).unwrap();
        
        let command = CreateSessionCommand::for_firebase_login(user_id, client_info);
        
        let result = handler.handle(command).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(!response.access_token.is_empty());
        assert!(response.has_refresh_token());
        assert!(response.expires_in_seconds() > 0);
    }
}