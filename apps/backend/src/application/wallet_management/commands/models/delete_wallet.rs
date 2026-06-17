// Delete Wallet Command
// Represents the intent to delete a wallet from the system

use epsx_contracts::value_objects::UserId;
use crate::application::shared::{Command, ApplicationResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteWalletCommand {
    pub wallet_address: UserId,
}

impl DeleteWalletCommand {
    pub fn new(wallet_address: UserId) -> Self {
        Self { wallet_address }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteWalletResponse {
    pub message: String,
}

impl DeleteWalletResponse {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl Command for DeleteWalletCommand {
    type Response = DeleteWalletResponse;

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