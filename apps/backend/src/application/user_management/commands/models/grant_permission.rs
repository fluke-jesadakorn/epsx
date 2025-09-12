use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::application::shared::{Command, ApplicationResult, ValidationUtils};
use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::user_management::value_objects::Permission;

/// Command to grant a permission to a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantPermissionCommand {
    /// ID of the user to grant permission to
    pub user_id: String,
    
    /// Permission string to grant (e.g., "admin:users:manage")
    pub permission: String,
    
    /// Optional expiration time for the permission
    pub expires_at: Option<DateTime<Utc>>,
    
    /// Who is granting this permission (for audit)
    pub granted_by: Option<String>,
    
    /// Reason for granting the permission
    pub reason: Option<String>,
    
    /// Command metadata
    pub correlation_id: Option<String>,
}

/// Response after successful permission grant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantPermissionResponse {
    /// The user ID who received the permission
    pub user_id: UserId,
    
    /// The permission that was granted
    pub permission: Permission,
    
    /// Who granted the permission
    pub granted_by: Option<UserId>,
    
    /// When the permission was granted
    pub granted_at: DateTime<Utc>,
    
    /// When the permission expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
}

impl Command for GrantPermissionCommand {
    type Response = GrantPermissionResponse;
    
    fn validate(&self) -> ApplicationResult<()> {
        // Validate user ID
        if let Some(error) = ValidationUtils::required("user_id", &self.user_id) {
            return Err(crate::application::ApplicationError::validation(
                &error.field,
                &error.message
            ));
        }
        
        // Validate permission format
        if let Some(error) = ValidationUtils::required("permission", &self.permission) {
            return Err(crate::application::ApplicationError::validation(
                &error.field,
                &error.message
            ));
        }
        
        // Validate permission format (platform:resource:action)
        if self.permission.split(':').count() < 3 {
            return Err(crate::application::ApplicationError::validation(
                "permission",
                "Permission must be in format 'platform:resource:action'"
            ));
        }
        
        // Validate expiration is in the future if provided
        if let Some(expires_at) = self.expires_at {
            if expires_at <= Utc::now() {
                return Err(crate::application::ApplicationError::validation(
                    "expires_at",
                    "Expiration time must be in the future"
                ));
            }
        }
        
        Ok(())
    }
}

impl GrantPermissionCommand {
    /// Create a new GrantPermissionCommand
    pub fn new(user_id: String, permission: String) -> Self {
        Self {
            user_id,
            permission,
            expires_at: None,
            granted_by: None,
            reason: None,
            correlation_id: None,
        }
    }
    
    /// Set expiration time for the permission
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Set who is granting the permission
    pub fn granted_by(mut self, user_id: String) -> Self {
        self.granted_by = Some(user_id);
        self
    }
    
    /// Set reason for granting the permission
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
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
    fn grant_permission_command_validation_success() {
        let command = GrantPermissionCommand::new(
            "user_123".to_string(),
            "admin:users:manage".to_string()
        );
        
        assert!(command.validate().is_ok());
    }
    
    #[test]
    fn grant_permission_command_validation_emptyuser_id() {
        let command = GrantPermissionCommand::new(
            "".to_string(),
            "admin:users:manage".to_string()
        );
        
        assert!(command.validate().is_err());
    }
    
    #[test]
    fn grant_permission_command_validation_invalid_permission_format() {
        let command = GrantPermissionCommand::new(
            "user_123".to_string(),
            "invalid_format".to_string()
        );
        
        assert!(command.validate().is_err());
    }
    
    #[test]
    fn grant_permission_command_validation_past_expiration() {
        let past_time = Utc::now() - Duration::hours(1);
        let command = GrantPermissionCommand::new(
            "user_123".to_string(),
            "admin:users:manage".to_string()
        ).with_expiration(past_time);
        
        assert!(command.validate().is_err());
    }
    
    #[test]
    fn grant_permission_command_builder_pattern() {
        let future_time = Utc::now() + Duration::hours(1);
        let command = GrantPermissionCommand::new(
            "user_123".to_string(),
            "admin:users:manage".to_string()
        )
        .with_expiration(future_time)
        .granted_by("admin_user".to_string())
        .with_reason("Temporary admin access".to_string())
        .with_correlation_id("corr_123".to_string());
        
        assert_eq!(command.expires_at, Some(future_time));
        assert_eq!(command.granted_by, Some("admin_user".to_string()));
        assert_eq!(command.reason, Some("Temporary admin access".to_string()));
        assert_eq!(command.correlation_id, Some("corr_123".to_string()));
    }
}