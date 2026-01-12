use chrono::{ DateTime, Utc };
use serde::{ Serialize, Deserialize };

use crate::domain::shared_kernel::value_objects::{ UserId, SessionId };

use crate::application::shared::{ Command, ApplicationResult, ValidationUtils };

/// Command to create a new user session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionCommand {
  /// ID of the user creating the session
  pub wallet_address: String,

  /// Access token for the session
  pub access_token: String,

  /// Optional refresh token
  pub refresh_token: Option<String>,

  /// When the session should expire
  pub expires_at: DateTime<Utc>,

  /// Client IP address
  pub ip_address: Option<String>,

  /// User agent string
  pub user_agent: Option<String>,

  /// Command metadata
  pub correlation_id: Option<String>,
}

/// Response after successful session creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
  /// The ID of the newly created session
  pub sid: SessionId,

  /// The user ID who owns the session
  pub wallet_address: UserId,

  /// When the session was created
  pub created_at: DateTime<Utc>,

  /// When the session expires
  pub expires_at: DateTime<Utc>,

  /// Whether the session is valid
  pub is_valid: bool,
}

impl Command for CreateSessionCommand {
  type Response = CreateSessionResponse;

  fn validate(&self) -> ApplicationResult<()> {
    // Validate wallet address
    if
      let Some(error) = ValidationUtils::required(
        "wallet_address",
        &self.wallet_address
      )
    {
      return Err(
        crate::application::ApplicationError::validation(
          &error.field,
          &error.message
        )
      );
    }

    // Validate access token
    if
      let Some(error) = ValidationUtils::required(
        "access_token",
        &self.access_token
      )
    {
      return Err(
        crate::application::ApplicationError::validation(
          &error.field,
          &error.message
        )
      );
    }

    // Validate expiration is in the future
    if self.expires_at <= Utc::now() {
      return Err(
        crate::application::ApplicationError::validation(
          "expires_at",
          "Session expiration must be in the future"
        )
      );
    }

    Ok(())
  }
}

impl CreateSessionCommand {
  /// Create a new CreateSessionCommand
  pub fn new(
    wallet_address: String,
    access_token: String,
    expires_at: DateTime<Utc>
  ) -> Self {
    Self {
      wallet_address,
      access_token,
      refresh_token: None,
      expires_at,
      ip_address: None,
      user_agent: None,
      correlation_id: None,
    }
  }

  /// Set refresh token
  pub fn withrefresh_token(mut self, refresh_token: String) -> Self {
    self.refresh_token = Some(refresh_token);
    self
  }

  /// Set client information
  pub fn with_client_info(
    mut self,
    ip_address: Option<String>,
    user_agent: Option<String>
  ) -> Self {
    self.ip_address = ip_address;
    self.user_agent = user_agent;
    self
  }

  /// Set correlation ID for tracing
  pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
    self.correlation_id = Some(correlation_id);
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Duration;

  #[test]
  fn create_session_command_validation_success() {
    let future_time = Utc::now() + Duration::hours(1);
    let command = CreateSessionCommand::new(
      "user_123".to_string(),
      "access_token_123".to_string(),
      future_time
    );

    assert!(command.validate().is_ok());
  }

  #[test]
  fn create_session_command_validation_emptyuser_id() {
    let future_time = Utc::now() + Duration::hours(1);
    let command = CreateSessionCommand::new(
      "".to_string(),
      "access_token_123".to_string(),
      future_time
    );

    assert!(command.validate().is_err());
  }

  #[test]
  fn create_session_command_validation_past_expiration() {
    let past_time = Utc::now() - Duration::hours(1);
    let command = CreateSessionCommand::new(
      "user_123".to_string(),
      "access_token_123".to_string(),
      past_time
    );

    assert!(command.validate().is_err());
  }
}
