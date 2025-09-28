use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use crate::application::shared::{Command, ApplicationResult};
use crate::domain::user_management::value_objects::Permission;
/// Command to update user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserCommand {
    pub wallet_address: String,
    pub email: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub email_verified: Option<bool>,
}

impl UpdateUserCommand {
    pub fn new(wallet_address: String) -> Self {
        Self {
            wallet_address,
            email: None,
            permissions: None,
            is_active: None,
            email_verified: None,
        }
    }
    
    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }
    
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = Some(permissions);
        self
    }
    
    pub fn with_active_status(mut self, is_active: bool) -> Self {
        self.is_active = Some(is_active);
        self
    }
    
    pub fn with_email_verified(mut self, email_verified: bool) -> Self {
        self.email_verified = Some(email_verified);
        self
    }
}

/// Response after successful user update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserResponse {
    pub wallet_address: String,
    pub email: String,
    pub email_verified: bool,
    pub is_active: bool,
    pub permissions: HashSet<Permission>,
}

impl Command for UpdateUserCommand {
    type Response = UpdateUserResponse;
    
    fn validate(&self) -> ApplicationResult<()> {
        // TODO: Implement validation
        Ok(())
    }
}