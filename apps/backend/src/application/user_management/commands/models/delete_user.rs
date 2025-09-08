// Delete User Command
// Represents the intent to delete a user from the system

use crate::domain::user_management::value_objects::FirebaseUid;
use crate::application::shared::{Command, ApplicationResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserCommand {
    pub firebase_uid: FirebaseUid,
}

impl DeleteUserCommand {
    pub fn new(firebase_uid: FirebaseUid) -> Self {
        Self { firebase_uid }
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
        // Basic validation - Firebase UID should not be empty
        if self.firebase_uid.as_str().is_empty() {
            return Err(crate::application::shared::ApplicationError::validation(
                "firebase_uid", 
                "Firebase UID cannot be empty".to_string()
            ));
        }
        Ok(())
    }
}