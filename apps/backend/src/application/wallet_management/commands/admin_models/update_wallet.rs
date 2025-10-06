// Admin Update Wallet Command
// Command to update wallet information (is_active, metadata)

use crate::application::shared::{ApplicationResult, Command};
use crate::application::wallet_management::queries::admin_models::WalletDetailDto;
use serde::{Deserialize, Serialize};

// ============================================================================
// COMMAND
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWalletCommand {
    pub wallet_address: String,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

impl Command for UpdateWalletCommand {
    type Response = UpdateWalletResponse;

    fn validate(&self) -> ApplicationResult<()> {
        // Validate wallet address is not empty
        if self.wallet_address.trim().is_empty() {
            return Err(crate::application::shared::ApplicationError::validation(
                "wallet_address",
                "Wallet address cannot be empty",
            ));
        }

        // Validate wallet address format (basic check)
        if !self.wallet_address.starts_with("0x") {
            return Err(crate::application::shared::ApplicationError::validation(
                "wallet_address",
                "Wallet address must start with 0x",
            ));
        }

        // Validate at least one field is provided
        if self.is_active.is_none() && self.metadata.is_none() {
            return Err(crate::application::shared::ApplicationError::validation(
                "update_fields",
                "At least one field must be provided for update",
            ));
        }

        Ok(())
    }
}

// ============================================================================
// RESPONSE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWalletResponse {
    pub success: bool,
    pub wallet: WalletDetailDto,
    pub message: String,
}
