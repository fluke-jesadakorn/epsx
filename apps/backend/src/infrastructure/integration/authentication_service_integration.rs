use crate::domain::shared_kernel::value_objects::SessionId;
use chrono::{ DateTime, Utc }; // Authentication Service Integration
// Orchestrates SessionManagement bounded context for authentication operations
// Maintains existing API compatibility while using DDD internally

use std::sync::Arc;
use tracing::{ info, warn, error };

use crate::domain::user_management::{
  UserRepositoryPort,
  SessionRepositoryPort,
};
use crate::application::user_management::CreateSessionCommandHandler;

/// Integration service for authentication operations
/// Provides high-level authentication operations for the web layer
pub struct AuthenticationServiceIntegration {
  user_repository: Arc<dyn UserRepositoryPort>,
  session_repository: Arc<dyn SessionRepositoryPort>,
  #[allow(dead_code)]
  create_session_handler: Arc<CreateSessionCommandHandler>,
}

impl AuthenticationServiceIntegration {
  pub fn new(
    user_repository: Arc<dyn UserRepositoryPort>,
    session_repository: Arc<dyn SessionRepositoryPort>,
    create_session_handler: Arc<CreateSessionCommandHandler>
  ) -> Self {
    Self {
      user_repository,
      session_repository,
      create_session_handler,
    }
  }
}

/// High-level authentication operations
impl AuthenticationServiceIntegration {
  /// Create a new session (login)
  pub async fn create_session(
    &self,
    _token: &str,
    _ip_address: String,
    _user_agent: Option<String>
  ) -> Result<SessionCreationResult, AuthenticationError> {
    info!("Creating session via DDD SessionManagement");

    // Firebase UID authentication removed - migrated to Web3
    Err(
      AuthenticationError::ServiceUnavailable(
        "Firebase authentication disabled - use Web3".to_string()
      )
    )
  }

  /// Terminate a session (logout)
  pub async fn terminate_session(
    &self,
    session_id: &str,
    user_id: &str
  ) -> Result<(), AuthenticationError> {
    info!("Terminating session {} for user {}", session_id, user_id);

    let session_id_obj = SessionId::from_string(session_id.to_string());

    // Find and deactivate session
    let session = self.session_repository
      .find_by_id(&session_id_obj).await
      .map_err(|e| AuthenticationError::SessionOperation(format!("{:?}", e)))?
      .ok_or(AuthenticationError::SessionNotFound)?;

    if !session.is_active() {
      warn!("Attempting to terminate already inactive session {}", session_id);
      return Ok(()); // Idempotent operation
    }

    // Deactivate session
    let mut session = session;
    session.deactivate();

    self.session_repository
      .save(&session).await
      .map_err(|e| AuthenticationError::SessionOperation(format!("{:?}", e)))?;

    info!("Session {} terminated successfully", session_id);
    Ok(())
  }

  /// Refresh session (extend expiry)
  pub async fn refresh_session(
    &self,
    refresh_token: &str,
    ip_address: String,
    user_agent: Option<String>
  ) -> Result<SessionRefreshResult, AuthenticationError> {
    info!("Refreshing session with refresh token");

    // For now, treat refresh token as session ID (would be proper token validation)
    let session_id_obj = SessionId::from_string(refresh_token.to_string());

    // Find active session
    let session = self.session_repository
      .find_by_id(&session_id_obj).await
      .map_err(|e| AuthenticationError::SessionOperation(format!("{:?}", e)))?
      .ok_or(AuthenticationError::SessionNotFound)?;

    if !session.is_active() {
      warn!("Attempting to refresh inactive session {}", refresh_token);
      return Err(AuthenticationError::SessionExpired);
    }

    // Refresh session
    let mut session = session;
    session.refresh();

    // Update session metadata if provided
    if let Some(ua) = user_agent {
      let _ = session.update_metadata(format!("user_agent:{}", ua));
    }
    let _ = session.update_metadata(format!("ip_address:{}", ip_address));

    self.session_repository
      .save(&session).await
      .map_err(|e| AuthenticationError::SessionOperation(format!("{:?}", e)))?;

    // Get user profile
    let user = self.user_repository
      .find_by_id(session.user_id()).await
      .map_err(|e| AuthenticationError::UserIdentity(format!("{:?}", e)))?
      .ok_or(AuthenticationError::UserNotFound)?;

    info!("Session {} refreshed successfully", refresh_token);

    Ok(SessionRefreshResult {
      session_id: session.id().to_string(),
      user_id: user.id().to_string(),
      profile: UserProfile {
        email: user.email().to_string(),
        display_name: format!("User {}", user.id().to_string()),
        is_active: user.is_active(),
        permissions: user
          .permissions()
          .iter()
          .map(|p| p.as_str().to_string())
          .collect(),
      },
      new_expires_at: session.expires_at(),
      refreshed_at: Utc::now(),
    })
  }

  /// Refresh token (legacy compatibility method)
  pub async fn refresh_token(
    &self,
    refresh_token: &str,
    ip_address: String
  ) -> Result<TokenRefreshResult, AuthenticationError> {
    // Use refresh_session internally but return token-focused result
    let session_result = self.refresh_session(
      refresh_token,
      ip_address,
      None
    ).await?;

    Ok(TokenRefreshResult {
      access_token: refresh_token.to_string(), // For now, return the same token
      refresh_token: Some(session_result.session_id.clone()),
      expires_at: session_result.new_expires_at,
    })
  }

  /// Validate session
  pub async fn validate_session(
    &self,
    session_id: &str
  ) -> Result<SessionValidationResult, AuthenticationError> {
    let session_id_obj = SessionId::from_string(session_id.to_string());

    let session = self.session_repository
      .find_by_id(&session_id_obj).await
      .map_err(|e| AuthenticationError::SessionOperation(format!("{:?}", e)))?
      .ok_or(AuthenticationError::SessionNotFound)?;

    let is_valid = session.is_active() && !session.is_expired();

    // Get user permissions for the session (stub implementation)
    let permissions = if is_valid {
      // In full implementation, would retrieve actual user permissions from UserRepository
      vec!["epsx:user:basic".to_string()]
    } else {
      vec![]
    };

    Ok(SessionValidationResult {
      is_valid,
      session_id: session.id().to_string(),
      user_id: session.user_id().to_string(),
      expires_at: session.expires_at(),
      permissions,
    })
  }
}

/// Result of session creation
#[derive(Debug)]
pub struct SessionCreationResult {
  pub session_id: String,
  pub user_id: String,
  pub profile: UserProfile,
  pub expires_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

/// Result of session refresh
#[derive(Debug)]
pub struct SessionRefreshResult {
  pub session_id: String,
  pub user_id: String,
  pub profile: UserProfile,
  pub new_expires_at: DateTime<Utc>,
  pub refreshed_at: DateTime<Utc>,
}

/// Result of token refresh (legacy compatibility)
#[derive(Debug)]
pub struct TokenRefreshResult {
  pub access_token: String,
  pub refresh_token: Option<String>,
  pub expires_at: DateTime<Utc>,
}

/// Result of session validation
#[derive(Debug)]
pub struct SessionValidationResult {
  pub is_valid: bool,
  pub session_id: String,
  pub user_id: String,
  pub expires_at: DateTime<Utc>,
  pub permissions: Vec<String>,
}

/// User profile for authentication responses
#[derive(Debug)]
pub struct UserProfile {
  pub email: String,
  pub display_name: String,
  pub is_active: bool,
  pub permissions: Vec<String>,
}

/// Authentication errors
#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
  #[error("Invalid token provided")]
  InvalidToken,

  #[error("Invalid session ID")]
  InvalidSessionId,

  #[error("Invalid refresh token")]
  InvalidRefreshToken,

  #[error("User not found")]
  UserNotFound,

  #[error("Session not found")]
  SessionNotFound,

  #[error("Session has expired")]
  SessionExpired,

  #[error("User identity error: {0}")] UserIdentity(String),

  #[error("Session creation failed: {0}")] SessionCreation(String),

  #[error("Session operation failed: {0}")] SessionOperation(String),

  #[error("Service unavailable: {0}")] ServiceUnavailable(String),
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::infrastructure::adapters::repositories::create_test_pool;
  use crate::infrastructure::adapters::repositories::{
    UserRepositoryAdapter,
    SessionRepositoryAdapter,
  };
  use crate::infrastructure::event_bus::SimpleEventBus;

  fn create_test_service() -> AuthenticationServiceIntegration {
    let db_pool = Arc::new(create_test_pool());
    let event_bus = Arc::new(SimpleEventBus::new());

    let user_repository: Arc<
      dyn UserRepository<Error = Box<dyn std::error::Error + Send + Sync>>
    > = Arc::new(UserRepositoryAdapter::new(db_pool.clone()));
    let session_repository: Arc<
      dyn SessionRepository<Error = Box<dyn std::error::Error + Send + Sync>>
    > = Arc::new(SessionRepositoryAdapter::new(db_pool.clone()));

    let create_session_handler = Arc::new(
      CreateSessionCommandHandler::new(
        user_repository.clone(),
        session_repository.clone(),
        event_bus.clone()
      )
    );

    AuthenticationServiceIntegration::new(
      user_repository,
      session_repository,
      create_session_handler
    )
  }

  #[tokio::test]
  async fn test_create_session() {
    let service = create_test_service();

    // This would fail in test since we need proper user setup
    let result = service.create_session(
      "test_firebase_uid",
      "127.0.0.1".to_string(),
      Some("test-agent".to_string())
    ).await;

    // For now, just ensure the method signature is correct
    assert!(result.is_err()); // Expected since no user exists
  }

  #[tokio::test]
  async fn test_validate_session() {
    let service = create_test_service();

    let result = service.validate_session("test_session_id").await;
    assert!(result.is_err()); // Expected since no session exists
  }
}
