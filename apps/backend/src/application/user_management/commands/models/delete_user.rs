// Delete User Command
// Represents the intent to delete a user from the system

use crate::domain::shared_kernel::value_objects::UserId;
use crate::application::shared::{Command, ApplicationResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserCommand {
    pub user_id: UserId,
}

impl DeleteUserCommand {
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserResponse {
    pub message: String,
}

impl DeleteUserResponse {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl Command for DeleteUserCommand {
    type Response = DeleteUserResponse;
    
    fn validate(&self) -> ApplicationResult<()> {
        // Basic validation - User ID should not be empty
        if self.user_id.to_string().is_empty() {
            return Err(crate::application::shared::ApplicationError::validation(
                "user_id", 
                "User ID cannot be empty".to_string()
            ));
        }
        Ok(())
    }
}