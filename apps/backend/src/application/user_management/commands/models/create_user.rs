use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::application::shared::{Command, ApplicationResult, ValidationUtils};
use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::user_management::value_objects::{Email, FirebaseUid};

/// Command to create a new user in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserCommand {
    /// Email address for the new user
    pub email: String,
    
    /// Firebase UID for authentication integration
    pub firebase_uid: String,
    
    /// Optional initial permissions to grant
    pub initial_permissions: Vec<String>,
    
    /// Whether to verify email immediately (for admin creation)
    pub email_verified: Option<bool>,
    
    /// Command metadata
    pub initiated_by: Option<String>,
    pub correlation_id: Option<String>,
}

/// Response after successful user creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserResponse {
    /// The ID of the newly created user
    pub user_id: UserId,
    
    /// The validated email address
    pub email: Email,
    
    /// Firebase UID
    pub firebase_uid: FirebaseUid,
    
    /// Permissions granted to the user
    pub permissions: Vec<String>,
    
    /// When the user was created
    pub created_at: DateTime<Utc>,
    
    /// Whether the user is active
    pub is_active: bool,
    
    /// Whether the email is verified
    pub email_verified: bool,
}

impl Command for CreateUserCommand {
    type Response = CreateUserResponse;
    
    fn validate(&self) -> ApplicationResult<()> {
        // Validate email
        if let Some(error) = ValidationUtils::required("email", &self.email) {
            return Err(crate::application::ApplicationError::validation(
                &error.field, &error.message
            ));
        }
        if let Some(error) = ValidationUtils::email_format("email", &self.email) {
            return Err(crate::application::ApplicationError::validation(
                &error.field, &error.message
            ));
        }
        
        // Validate firebase_uid
        if let Some(error) = ValidationUtils::required("firebase_uid", &self.firebase_uid) {
            return Err(crate::application::ApplicationError::validation(
                &error.field, &error.message
            ));
        }
        if let Some(error) = ValidationUtils::length("firebase_uid", &self.firebase_uid, 10, 128) {
            return Err(crate::application::ApplicationError::validation(
                &error.field, &error.message
            ));
        }
        
        // Validate permissions format if provided
        for permission in &self.initial_permissions {
            if permission.split(':').count() < 3 {
                return Err(crate::application::ApplicationError::validation(
                    "initial_permissions",
                    format!("Invalid permission format: {}", permission)
                ));
            }
        }
        
        Ok(())
    }
}

impl CreateUserCommand {
    /// Create a new CreateUserCommand with required fields
    pub fn new(email: String, firebase_uid: String) -> Self {
        Self {
            email,
            firebase_uid,
            initial_permissions: Vec::new(),
            email_verified: None,
            initiated_by: None,
            correlation_id: None,
        }
    }
    
    /// Add initial permissions to the command
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.initial_permissions = permissions;
        self
    }
    
    /// Set email as verified (for admin creation)
    pub fn with_email_verified(mut self, verified: bool) -> Self {
        self.email_verified = Some(verified);
        self
    }
    
    /// Set who initiated this command (for audit)
    pub fn initiated_by(mut self, user_id: String) -> Self {
        self.initiated_by = Some(user_id);
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
    
    #[test]
    fn create_user_command_validation_success() {
        let command = CreateUserCommand::new(
            "test@example.com".to_string(),
            "firebase_uid_123".to_string()
        );
        
        assert!(command.validate().is_ok());
    }
    
    #[test]
    fn create_user_command_validation_invalid_email() {
        let command = CreateUserCommand::new(
            "invalid-email".to_string(),
            "firebase_uid_123".to_string()
        );
        
        assert!(command.validate().is_err());
    }
    
    #[test]
    fn create_user_command_validation_empty_firebase_uid() {
        let command = CreateUserCommand::new(
            "test@example.com".to_string(),
            "".to_string()
        );
        
        assert!(command.validate().is_err());
    }
    
    #[test]
    fn create_user_command_validation_invalid_permissions() {
        let command = CreateUserCommand::new(
            "test@example.com".to_string(),
            "firebase_uid_123".to_string()
        ).with_permissions(vec!["invalid_permission".to_string()]);
        
        assert!(command.validate().is_err());
    }
    
    #[test]
    fn create_user_command_builder_pattern() {
        let command = CreateUserCommand::new(
            "test@example.com".to_string(),
            "firebase_uid_123".to_string()
        )
        .with_permissions(vec!["epsx:analytics:view".to_string()])
        .with_email_verified(true)
        .initiated_by("admin_user_id".to_string())
        .with_correlation_id("correlation_123".to_string());
        
        assert_eq!(command.initial_permissions.len(), 1);
        assert_eq!(command.email_verified, Some(true));
        assert_eq!(command.initiated_by, Some("admin_user_id".to_string()));
        assert_eq!(command.correlation_id, Some("correlation_123".to_string()));
    }
}