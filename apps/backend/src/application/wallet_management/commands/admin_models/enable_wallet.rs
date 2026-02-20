use crate::application::shared::{ApplicationResult, Command};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct EnableWalletCommand {
    #[serde(default)]
    pub wallet_address: String,
    #[serde(default)]
    pub admin_wallet_address: String, // Who performed the action
    pub platforms_to_enable: Vec<String>,
    pub restore_permissions: bool,
    pub resume_subscriptions: bool,
    pub resolution_note: String,
}

impl Command for EnableWalletCommand {
    type Response = EnableWalletResponse;

    fn validate(&self) -> ApplicationResult<()> {
        if self.wallet_address.trim().is_empty() {
             return Err(crate::application::shared::ApplicationError::validation(
                "wallet_address",
                "Wallet address cannot be empty",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EnableWalletResponse {
    pub success: bool,
    pub message: String,
}
