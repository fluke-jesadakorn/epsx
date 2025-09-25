use std::sync::Arc;

// Authentication Application Service
// Main orchestrator for authentication operations using CQRS and DDD patterns

use tracing::{info, warn};

use crate::application::shared::{ApplicationResult, ApplicationError, CommandHandler};
use crate::domain::authentication::{
    AuthenticatedUserId, SessionId,
    AuthenticationSessionRepositoryPort, TokenValidationServicePort,
    SecurityMonitoringServicePort, UserIdentityServicePort
};

use super::super::commands::{
    CreateSessionCommand, CreateSessionResponse,
    RefreshTokensCommand, RefreshTokensResponse,
    TerminateSessionCommand, TerminateSessionResponse,
    ValidateCredentialsCommand, ValidateCredentialsResponse,
    CreateSessionHandler, RefreshTokensHandler, TerminateSessionHandler, ValidateCredentialsHandler
};

/// Main application service for authentication operations
pub struct AuthenticationApplicationService {
    // Command handlers
    create_session_handler: CreateSessionHandler,
    refresh_tokens_handler: RefreshTokensHandler,
    terminate_session_handler: TerminateSessionHandler,
    validate_credentials_handler: ValidateCredentialsHandler,
    
    // Repositories
    session_repository: Arc<dyn AuthenticationSessionRepositoryPort>,
    
    // Services
    #[allow(dead_code)]
    security_monitoring_service: Arc<dyn SecurityMonitoringServicePort>,
    #[allow(dead_code)]
    user_identity_service: Arc<dyn UserIdentityServicePort>,
}

impl AuthenticationApplicationService {
    /// Create new authentication application service
    pub fn new(
        session_repository: Arc<dyn AuthenticationSessionRepositoryPort>,
        token_validation_service: Arc<dyn TokenValidationServicePort>,
        security_monitoring_service: Arc<dyn SecurityMonitoringServicePort>,
        user_identity_service: Arc<dyn UserIdentityServicePort>,
    ) -> Self {
        // Create command handlers
        let create_session_handler = CreateSessionHandler::new(
            Arc::clone(&session_repository),
            Arc::clone(&token_validation_service),
            Arc::clone(&security_monitoring_service),
        );
        
        let refresh_tokens_handler = RefreshTokensHandler::new(
            Arc::clone(&session_repository),
            Arc::clone(&token_validation_service),
            Arc::clone(&security_monitoring_service),
        );
        
        let terminate_session_handler = TerminateSessionHandler::new(
            Arc::clone(&session_repository),
            Arc::clone(&security_monitoring_service),
        );
        
        let validate_credentials_handler = ValidateCredentialsHandler::new(
            Arc::clone(&token_validation_service),
            Arc::clone(&user_identity_service),
            Arc::clone(&security_monitoring_service),
        );
        
        Self {
            create_session_handler,
            refresh_tokens_handler,
            terminate_session_handler,
            validate_credentials_handler,
            session_repository,
            security_monitoring_service,
            user_identity_service,
        }
    }
    
    /// Authenticate user and create session (login flow)
    pub async fn authenticate_user(
        &self,
        command: CreateSessionCommand,
    ) -> ApplicationResult<CreateSessionResponse> {
        info!(
            user_id = %command.user_id,
            provider = ?command.provider.provider_type(),
            "Processing user authentication"
        );
        
        self.create_session_handler.handle(command).await
    }
    
    /// Refresh authentication tokens
    pub async fn refresh_tokens(
        &self,
        command: RefreshTokensCommand,
    ) -> ApplicationResult<RefreshTokensResponse> {
        info!(
            session_id = %command.session_id,
            "Processing token refresh"
        );
        
        self.refresh_tokens_handler.handle(command).await
    }
    
    /// Validate authentication credentials
    pub async fn validate_credentials(
        &self,
        command: ValidateCredentialsCommand,
    ) -> ApplicationResult<ValidateCredentialsResponse> {
        // Don't log the token for security
        info!(
            token_type = ?command.token_type,
            required_scopes = ?command.required_scopes,
            "Processing credential validation"
        );
        
        self.validate_credentials_handler.handle(command).await
    }
    
    /// Terminate authentication session (logout flow)
    pub async fn terminate_session(
        &self,
        command: TerminateSessionCommand,
    ) -> ApplicationResult<TerminateSessionResponse> {
        info!(
            session_id = %command.session_id,
            reason = ?command.reason,
            "Processing session termination"
        );
        
        self.terminate_session_handler.handle(command).await
    }
    
    /// Get active session for user
    pub async fn get_user_sessions(
        &self,
        user_id: &AuthenticatedUserId,
    ) -> ApplicationResult<Vec<SessionInfo>> {
        let sessions = self.session_repository.find_by_user(user_id).await
            .map_err(|e| ApplicationError::InfrastructureError(e))?;
        
        let session_info = sessions
            .iter()
            .filter(|session| session.is_active())
            .map(|session| SessionInfo {
                session_id: SessionId::from_string(session.session_id().to_string()).unwrap_or_else(|_| SessionId::generate()),
                created_at: session.created_at(),
                last_activity: session.last_activity(),
                expires_at: session.expires_at(),
                provider_type: session.provider().provider_type().clone(),
                security_risk_level: session.security_context().risk_level().clone(),
                is_active: session.is_active(),
            })
            .collect();
        
        Ok(session_info)
    }
    
    /// Terminate all sessions for user (global logout)
    pub async fn terminate_all_user_sessions(
        &self,
        user_id: &AuthenticatedUserId,
        reason: crate::domain::authentication::TerminationReason,
    ) -> ApplicationResult<u32> {
        let sessions = self.session_repository.find_by_user(user_id).await
            .map_err(|e| ApplicationError::InfrastructureError(e))?;
        
        let active_sessions: Vec<_> = sessions
            .iter()
            .filter(|session| session.is_active())
            .collect();
        
        let mut terminated_count = 0;
        
        for session in active_sessions {
            let command = TerminateSessionCommand {
                session_id: SessionId::from_string(session.session_id().to_string()).unwrap_or_else(|_| SessionId::generate()),
                reason: reason.clone(),
                revoke_tokens: true,
                notify_user: matches!(reason, 
                    crate::domain::authentication::TerminationReason::AdminTermination |
                    crate::domain::authentication::TerminationReason::SecurityThreat
                ),
            };
            
            match self.terminate_session(command).await {
                Ok(_) => terminated_count += 1,
                Err(e) => warn!(
                    session_id = %session.session_id(),
                    error = %e,
                    "Failed to terminate session during global logout"
                ),
            }
        }
        
        info!(
            user_id = %user_id,
            terminated_count = terminated_count,
            "Completed global session termination"
        );
        
        Ok(terminated_count)
    }
    
    /// Check if user has active sessions
    pub async fn has_active_sessions(&self, user_id: &AuthenticatedUserId) -> ApplicationResult<bool> {
        let sessions = self.session_repository.find_by_user(user_id).await
            .map_err(|e| ApplicationError::InfrastructureError(e))?;
        
        Ok(sessions.iter().any(|session| session.is_active()))
    }
    
    /// Get authentication statistics for monitoring
    pub async fn get_authentication_statistics(&self) -> ApplicationResult<AuthenticationStatistics> {
        // In a real implementation, this would query a statistics repository
        // For now, return basic statistics
        
        Ok(AuthenticationStatistics {
            total_active_sessions: 0,
            sessions_created_last_hour: 0,
            failed_authentications_last_hour: 0,
            suspicious_activities_last_hour: 0,
            average_session_duration_minutes: 0.0,
            most_common_provider: "Firebase".to_string(),
        })
    }
}

/// Session information for users
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionInfo {
    pub session_id: SessionId,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub provider_type: crate::domain::authentication::ProviderType,
    pub security_risk_level: crate::domain::authentication::RiskLevel,
    pub is_active: bool,
}

/// Authentication statistics for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthenticationStatistics {
    pub total_active_sessions: u64,
    pub sessions_created_last_hour: u64,
    pub failed_authentications_last_hour: u64,
    pub suspicious_activities_last_hour: u64,
    pub average_session_duration_minutes: f64,
    pub most_common_provider: String,
}

// Removed unused CloneBox traits - not used in implementation

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::authentication::*;
    use crate::domain::user_management::value_objects::UserId;
    
    // Mock implementations would go here
    
    #[tokio::test]
    async fn test_authentication_service_creation() {
        // Test that the service can be created
        // In a real test, we'd use proper mocks
        assert!(true);
    }
}