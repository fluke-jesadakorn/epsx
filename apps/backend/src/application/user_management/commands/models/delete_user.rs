// Delete User Command
// Represents the intent to delete a user from the system

use crate::domain::shared_kernel::value_objects::UserId;
use crate::application::shared::{Command, ApplicationResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserCommand {
    pub wallet_address: UserId,
}

impl DeleteUserCommand {
    pub fn new(wallet_address: UserId) -> Self {
        Self { wallet_address }
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
        // Basic validation - Wallet address should not be empty
        if self.wallet_address.to_string().is_empty() {
            return Err(crate::application::shared::ApplicationError::validation(
                "wallet_address", 
                "Wallet address cannot be empty".to_string()
            ));
        }
        Ok(())
    }
}